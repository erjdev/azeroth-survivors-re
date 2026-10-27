//! Authored automatic Beast Mastery milestones for the Survivors combat wolf.
use super::{Milestones, TalentStat};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PetBonus {
    pub health: f32,
    pub mend: f32,
    pub recovery: f32,
    pub intimidation: bool,
    pub spirit_bond: f32,
    pub damage: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub frenzy_chance: f32,
    pub speed: f32,
}

impl Default for PetBonus {
    fn default() -> Self {
        Self {
            health: 1.0,
            mend: 0.0,
            recovery: 10.0,
            intimidation: false,
            spirit_bond: 0.0,
            damage: 1.0,
            cooldown: 1.0,
            crit: 0.0,
            frenzy_chance: 0.0,
            speed: 1.0,
        }
    }
}

/// A critical-triggered buff; repeated procs refresh duration rather than stack haste.
#[derive(Clone, Copy, Debug, Default)]
pub struct PetFrenzy {
    remaining: f32,
}
impl PetFrenzy {
    pub fn advance(&mut self, seconds: f32) {
        if seconds.is_finite() && seconds > 0.0 {
            self.remaining = (self.remaining - seconds).max(0.0);
        }
    }
    pub fn critical(&mut self, chance: f32, roll: f32) {
        if (0.0..1.0).contains(&roll) && roll < chance.clamp(0.0, 1.0) {
            self.remaining = 8.0;
        }
    }
    pub fn haste(self) -> f32 {
        if self.remaining > 0.0 {
            0.3
        } else {
            0.0
        }
    }
}

impl Milestones {
    /// Fixed windows use gameplay time; skipped intervals never accumulate charges.
    pub fn deterrence_remaining(self, elapsed: f32) -> f32 {
        if self.class != 2
            || self.trained_value(TalentStat::Deterrence) <= 0.0
            || !elapsed.is_finite()
            || elapsed < 0.0
        {
            return 0.0;
        }
        (10.0 - elapsed % 180.0).max(0.0)
    }

    pub fn deterrence_multiplier(self, elapsed: f32) -> f32 {
        if self.deterrence_remaining(elapsed) > 0.0 {
            1.0 - self.trained_value(TalentStat::Deterrence)
        } else {
            1.0
        }
    }

    /// Run time drives Wrath, so pause and selection screens cannot consume the buff.
    pub fn hunter_pet(self, elapsed: f32) -> PetBonus {
        let mut bonus = PetBonus::default();
        if self.class != 2 {
            return bonus;
        }
        bonus.intimidation = self.trained_value(TalentStat::PetIntimidation) > 0.0;
        bonus.recovery *= 1.0 - self.trained_value(TalentStat::PetRecovery);
        bonus.mend = self.trained_value(TalentStat::PetMend);
        bonus.spirit_bond = self.trained_value(TalentStat::SpiritBond);
        bonus.health += self.trained_value(TalentStat::PetHealth);
        bonus.damage *= 1.0 + self.trained_value(TalentStat::PetDamage);
        bonus.crit = self.trained_value(TalentStat::PetCrit);
        bonus.frenzy_chance = self.trained_value(TalentStat::PetFrenzy);
        bonus.speed += self.trained_value(TalentStat::PetSpeed);
        bonus.cooldown *= 1.0 - self.trained_value(TalentStat::PetCooldown);
        if self.points[0] >= 10 {
            bonus.damage *= 1.2;
        }
        if self.points[0] >= 20 {
            bonus.cooldown *= 0.8;
        }
        let trained_wrath = self.trained_value(TalentStat::PetWrath) > 0.0;
        let wrath_duration = if trained_wrath { 18.0 } else { 10.0 };
        if self.points[0] >= 30
            && elapsed.is_finite()
            && elapsed >= 0.0
            && elapsed % 30.0 < wrath_duration
        {
            bonus.damage *= if trained_wrath { 2.0 } else { 1.5 };
        }
        bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mend_is_rank_bounded_pet_only_and_removed_with_training() {
        let node = super::super::definitions(2)
            .iter()
            .find(|n| n.id == 1385)
            .unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 1% maximum health"));
        assert!(node
            .inline_description(1)
            .unwrap()
            .contains("1 » 2% maximum health"));
        for rank in [0u8, 1, 2, 255] {
            let base = Milestones {
                class: 2,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(0, 1385, rank)]);
            let mut expected = base.hunter_pet(0.0);
            expected.mend = 0.01 * f32::from(rank.min(2));
            assert_eq!(trained.hunter_pet(0.0), expected);
            assert_eq!(
                trained.with_ranks(&[]).hunter_pet(0.0),
                base.hunter_pet(0.0)
            );
            for key in ["arcane_shot", "volley", "frostbolt"] {
                assert_eq!(trained.cast(key), base.cast(key));
            }
        }
        for class in [0, 1, 3, 4, 5, 6, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 1385, 2)]);
            assert_eq!(other.hunter_pet(0.0), PetBonus::default());
        }
    }

    #[test]
    fn discipline_composes_with_milestone_and_frenzy_without_affecting_hero() {
        let node = super::super::definitions(2)
            .iter()
            .find(|node| node.id == 1390)
            .unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 10% shorter bite cooldown"));
        assert!(node
            .inline_description(1)
            .unwrap()
            .contains("10 » 20% shorter bite cooldown"));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
        for rank in [0u8, 1, 2, 255] {
            for points in [0, 19, 20, 30] {
                let base = Milestones {
                    class: 2,
                    points: [points, 0, 0],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, 1390, rank)]);
                let multiplier = 1.0 - 0.1 * f32::from(rank.min(2));
                let mut expected = base.hunter_pet(0.0);
                expected.cooldown *= multiplier;
                assert_eq!(trained.hunter_pet(0.0), expected);
                assert_eq!(
                    trained.with_ranks(&[]).hunter_pet(0.0),
                    base.hunter_pet(0.0)
                );
                for key in ["arcane_shot", "serpent_sting", "volley"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
                let mut frenzy = PetFrenzy::default();
                frenzy.critical(1.0, 0.0);
                let actual_interval =
                    2.0 * trained.hunter_pet(0.0).cooldown / (1.0 + frenzy.haste());
                let expected_interval =
                    2.0 * multiplier * if points >= 20 { 0.8 } else { 1.0 } / 1.3;
                assert!((actual_interval - expected_interval).abs() < 0.00001);
            }
        }
        for class in [0, 1, 3, 4, 5, 6, 7, 8] {
            assert_eq!(
                Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, 1390, 2)])
                .hunter_pet(0.0),
                PetBonus::default()
            );
        }
    }

    #[test]
    fn pet_training_is_bounded_resettable_and_multiplies_milestones() {
        let base = Milestones {
            class: 2,
            points: [30, 0, 0],
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (0, 1389, 255),
            (0, 1396, 255),
            (0, 1393, 255),
            (0, 1397, 255),
            (0, 1391, 255),
        ]);
        let bonus = trained.hunter_pet(0.0);
        assert!((bonus.damage - 2.16).abs() < 0.00001);
        assert!((bonus.health - 1.15).abs() < 0.00001);
        assert_eq!(bonus.cooldown, 0.8);
        assert!((bonus.crit - 0.1).abs() < 0.00001);
        assert_eq!(bonus.frenzy_chance, 1.0);
        assert_eq!(bonus.speed, 1.3);
        assert_eq!(trained.with_ranks(&[]), base);
        for rank in 0..=5 {
            let m = base.with_ranks(&[(0, 1396, rank), (0, 1393, rank), (0, 1397, rank)]);
            let b = m.hunter_pet(10.0);
            assert!((b.damage - 1.2 * (1.0 + 0.04 * f32::from(rank))).abs() < 0.00001);
            assert!((b.crit - 0.02 * f32::from(rank)).abs() < 0.00001);
            assert!((b.frenzy_chance - 0.2 * f32::from(rank)).abs() < 0.00001);
            assert_eq!(m.cast("arcane_shot"), super::super::CastEffect::default());
        }
        for id in [1396, 1393, 1397, 1391] {
            let node = super::super::talent_definition(2, id).unwrap();
            assert!(!node.rank_preview(0).contains("pending"));
        }
    }

    #[test]
    fn recovery_and_bond_are_rank_bounded_and_pet_class_only() {
        for (rank, recovery, bond) in [
            (0, 10.0, 0.0),
            (1, 8.0, 0.01),
            (2, 6.0, 0.02),
            (255, 6.0, 0.02),
        ] {
            let m = Milestones {
                class: 2,
                ..Default::default()
            }
            .with_ranks(&[(0, 1625, rank), (0, 1388, rank)]);
            let b = m.hunter_pet(0.0);
            assert!((b.recovery - recovery).abs() < 0.0001);
            assert_eq!(b.spirit_bond, bond);
            assert_eq!(m.with_ranks(&[]).hunter_pet(0.0), PetBonus::default());
            for class in [0, 1, 3, 4, 5, 6, 7, 8] {
                assert_eq!(
                    Milestones { class, ..m }.hunter_pet(0.0),
                    PetBonus::default()
                );
            }
        }
    }

    #[test]
    fn trained_wrath_replaces_bonus_extends_window_and_requires_milestone() {
        let base = Milestones {
            class: 2,
            points: [30, 0, 0],
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 1386, 255)]);
        for elapsed in [0.0, 9.999, 10.0, 17.999, 30.0, 47.999] {
            assert!((trained.hunter_pet(elapsed).damage - 2.4).abs() < 0.0001);
        }
        for elapsed in [18.0, 29.999, 48.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(trained.hunter_pet(elapsed).damage, 1.2);
        }
        assert!((trained.with_ranks(&[]).hunter_pet(0.0).damage - 1.8).abs() < 0.0001);
        assert_eq!(
            Milestones {
                points: [29, 0, 0],
                ..trained
            }
            .hunter_pet(0.0)
            .damage,
            1.2
        );
        for key in ["arcane_shot", "multi_shot", "volley", "explosive_trap"] {
            assert_eq!(trained.cast(key), super::super::CastEffect::default());
        }
    }

    #[test]
    fn frenzy_expires_and_refreshes_without_stacking() {
        let mut state = PetFrenzy::default();
        for roll in [-1.0, 0.2, 1.0, f32::NAN] {
            state.critical(0.2, roll);
        }
        assert_eq!(state.haste(), 0.0);
        state.critical(0.2, 0.199);
        assert_eq!(state.haste(), 0.3);
        state.advance(7.5);
        state.critical(1.0, 0.5);
        state.advance(7.5);
        assert_eq!(state.haste(), 0.3);
        state.advance(0.5);
        assert_eq!(state.haste(), 0.0);
        state.critical(0.0, 0.0);
        assert_eq!(state.haste(), 0.0);
    }

    #[test]
    fn thresholds_and_periodic_wrath_are_pet_only() {
        let at = |points| Milestones {
            class: 2,
            points: [points, 0, 0],
            ..Default::default()
        };
        assert_eq!(at(9).hunter_pet(0.0), PetBonus::default());
        assert_eq!(at(10).hunter_pet(0.0).damage, 1.2);
        assert_eq!(at(19).hunter_pet(0.0).cooldown, 1.0);
        assert_eq!(at(20).hunter_pet(0.0).cooldown, 0.8);
        assert_eq!(at(29).hunter_pet(0.0).damage, 1.2);
        for elapsed in [0.0, 9.999, 30.0, 39.999, 60.0] {
            assert!((at(30).hunter_pet(elapsed).damage - 1.8).abs() < 0.00001);
        }
        for elapsed in [10.0, 29.999, 40.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(at(30).hunter_pet(elapsed).damage, 1.2);
        }
        for class in 0..9 {
            if class == 2 {
                continue;
            }
            assert_eq!(
                Milestones {
                    class,
                    points: [51; 3],
                    ..Default::default()
                }
                .hunter_pet(0.0),
                PetBonus::default()
            );
        }
        for tree in [1, 2] {
            let mut other = at(0);
            other.points[tree] = 51;
            assert_eq!(other.hunter_pet(0.0), PetBonus::default());
        }
        for key in ["arcane_shot", "multi_shot", "volley", "explosive_trap"] {
            assert_eq!(at(51).cast(key), super::super::CastEffect::default());
        }
        for threshold in [10, 20, 30] {
            assert!(super::super::description(2, 0, threshold).is_some());
        }
    }
}

#[cfg(test)]
mod deterrence_tests {
    use super::*;
    #[test]
    fn deterrence_windows_use_run_time_and_capped_hunter_training() {
        for class in 0..9 {
            for rank in [0, 1, 255] {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(2, 1308, rank)]);
                for (elapsed, remaining) in [
                    (0., 10.),
                    (1.5, 8.5),
                    (9.99, 0.01),
                    (10., 0.),
                    (179.99, 0.),
                    (180., 10.),
                    (189., 1.),
                    (190., 0.),
                    (905., 5.),
                    (920., 0.),
                ] {
                    let expected = if class == 2 && rank > 0 {
                        remaining
                    } else {
                        0.
                    };
                    assert!((trained.deterrence_remaining(elapsed) - expected).abs() < 0.0001);
                    assert_eq!(
                        trained.deterrence_multiplier(elapsed),
                        if expected > 0. { 0.5 } else { 1. }
                    );
                }
                for elapsed in [-1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.deterrence_remaining(elapsed), 0.);
                    assert_eq!(trained.deterrence_multiplier(elapsed), 1.);
                }
            }
        }
        let node = super::super::talent_definition(2, 1308).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 50% less incoming damage"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}
