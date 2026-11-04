//! Automatic Power Infusion follows active run time; no manual target or mana pool.
use super::{Milestones, TalentStat, PRIEST_HOLY, PRIEST_SHADOW};

#[derive(Clone, Copy, Debug, Default)]
pub struct PowerInfusionClock {
    next: f32,
    until: f32,
}
impl PowerInfusionClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.
            || talents.class != 4
            || !(PRIEST_HOLY.contains(&key)
                || PRIEST_SHADOW.contains(&key)
                || key == "greater_heal")
            || (elapsed >= self.until && elapsed < self.next)
        {
            return 0.;
        }
        talents.trained_value(TalentStat::PowerInfusion)
    }
    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if elapsed >= self.next && self.bonus(talents, key, elapsed) > 0. {
            self.until = elapsed + 15.;
            self.next = elapsed + 180.;
        }
    }
    pub fn remaining(self, talents: Milestones, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.
            || talents.class != 4
            || talents.trained_value(TalentStat::PowerInfusion) <= 0.
        {
            return 0.;
        }
        (self.until - elapsed).max(0.)
    }
    pub fn active_bonus(self, talents: Milestones, elapsed: f32) -> f32 {
        if self.remaining(talents, elapsed) > 0. {
            talents.trained_value(TalentStat::PowerInfusion)
        } else {
            0.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn improved_fade_is_capped_and_scoped_to_escape() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 483, rank)]);
                for key in PRIEST_HOLY
                    .iter()
                    .chain(PRIEST_SHADOW)
                    .copied()
                    .chain(["fade", "charge", "fireball"])
                {
                    let mut expected = base.cast(key);
                    if class == 4 && key == "fade" {
                        expected.cooldown *= 1.0 - 0.1 * f32::from(rank.min(2));
                    }
                    assert_eq!(trained.cast(key), expected);
                }
            }
        }
        let node = super::super::talent_definition(4, 483).unwrap();
        assert!(node
            .inline_description(1)
            .unwrap()
            .starts_with("Fade: 10 » 20% shorter cooldown"));
        assert!(!node.inline_description(2).unwrap().contains(" » "));
    }

    #[test]
    fn infusion_and_inner_focus_recover_independently() {
        let trained = Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(&[(0, 322, 1), (0, 348, 1)]);
        let mut infusion = PowerInfusionClock::default();
        let mut focus = super::super::CriticalCastClock::default();
        infusion.consume(trained, "shadow_word_pain", 0.);
        focus.consume(trained, "shadow_word_pain", 0.);
        assert_eq!(focus.bonus(trained, "smite", 1.), 0.25);
        infusion.consume(trained, "smite", 1.);
        focus.consume(trained, "smite", 1.);
        assert_eq!(infusion.remaining(trained, 1.), 14.);
        assert_eq!(focus.bonus(trained, "smite", 180.), 0.);
        assert_eq!(infusion.bonus(trained, "smite", 180.), 0.2);
        assert_eq!(focus.bonus(trained, "smite", 181.), 0.25);
    }

    #[test]
    fn infusion_waits_for_success_and_cannot_refresh_or_bank_charges() {
        for class in [0, 4, 6] {
            for rank in [0, 1, 255] {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, 322, rank)]);
                let expected = if class == 4 && rank > 0 { 0.2 } else { 0. };
                let mut clock = PowerInfusionClock::default();
                for time in [f32::NAN, f32::INFINITY, -1.] {
                    clock.consume(trained, "smite", time);
                    assert_eq!(clock.bonus(trained, "smite", time), 0.);
                    assert_eq!(clock.active_bonus(trained, time), 0.);
                }
                for key in ["renew", "power_word_shield", "fireball"] {
                    clock.consume(trained, key, 0.);
                    assert_eq!(clock.bonus(trained, key, 0.), 0.);
                }
                assert_eq!(clock.active_bonus(trained, 1000.), 0.);
                clock.consume(trained, "smite", 1000.);
                for key in PRIEST_HOLY.iter().chain(PRIEST_SHADOW) {
                    assert_eq!(clock.bonus(trained, key, 1014.99), expected);
                }
                assert_eq!(clock.active_bonus(trained, 1014.99), expected);
                clock.consume(trained, "holy_nova", 1014.);
                assert_eq!(clock.active_bonus(trained, 1015.), 0.);
                assert_eq!(clock.bonus(trained, "smite", 1179.99), 0.);
                assert_eq!(clock.bonus(trained, "smite", 1180.), expected);
                assert_eq!(clock.active_bonus(trained.with_ranks(&[]), 1001.), 0.);
            }
        }
        let node = super::super::talent_definition(4, 322).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("+0 » 20% Priest"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}

/// Outstanding healing rolls forward without multiplying or replaying paid healing.
#[derive(Clone, Copy, Debug, Default)]
pub struct BlessedRecovery {
    amount: f32,
    remaining: f32,
}

impl BlessedRecovery {
    fn fraction(talents: Milestones) -> f32 {
        if talents.class == 4 {
            talents.trained_value(TalentStat::BlessedRecovery)
        } else {
            0.
        }
    }

    pub fn hit(&mut self, talents: Milestones, health_damage: f32) {
        let fraction = Self::fraction(talents);
        if fraction > 0. && health_damage.is_finite() && health_damage > 0. {
            self.amount += health_damage * fraction;
            self.remaining = 6.;
        }
    }

    pub fn advance(&mut self, talents: Milestones, dt: f32) -> f32 {
        if Self::fraction(talents) <= 0. {
            *self = Self::default();
            return 0.;
        }
        if !dt.is_finite() || dt <= 0. || self.remaining <= 0. {
            return 0.;
        }
        let elapsed = dt.min(self.remaining);
        let heal = self.amount * (elapsed / self.remaining);
        self.amount = (self.amount - heal).max(0.);
        self.remaining = (self.remaining - elapsed).max(0.);
        heal
    }

    pub fn remaining(self, talents: Milestones) -> f32 {
        if Self::fraction(talents) > 0. {
            self.remaining
        } else {
            0.
        }
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn recovery_conserves_healing_across_refreshes_and_large_frames() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 255] {
                let talents = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 1636, rank)]);
                let fraction = if class == 4 {
                    f32::from(rank.min(3)) * 0.07
                } else {
                    0.
                };
                let mut recovery = BlessedRecovery::default();
                recovery.hit(talents, 100.);
                for invalid in [0., -1., f32::NAN, f32::INFINITY] {
                    recovery.hit(talents, invalid);
                    assert_eq!(recovery.advance(talents, invalid), 0.);
                }
                let first = recovery.advance(talents, 3.);
                assert!((first - 50. * fraction).abs() < 0.0001);
                recovery.hit(talents, 50.);
                let rest = recovery.advance(talents, 30.);
                assert!((first + rest - 150. * fraction).abs() < 0.0001);
                assert_eq!(recovery.advance(talents, 6.), 0.);
                assert_eq!(recovery.remaining(talents), 0.);
                recovery.hit(talents, 100.);
                assert_eq!(recovery.advance(talents.with_ranks(&[]), 1.), 0.);
                assert_eq!(recovery.remaining(talents), 0.);
            }
        }
        let node = super::super::talent_definition(4, 1636).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 7%"));
        assert!(node.inline_description(2).unwrap().contains("14 » 21%"));
    }
}

#[cfg(test)]
mod inspiration_tests {
    use super::*;
    #[test]
    fn inspiration_is_capped_scoped_and_does_not_grant_healing_or_spells() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            for rank in [0, 1, 2, 3, 255] {
                let trained = base.with_ranks(&[(1, 361, rank), (1, 361, rank)]);
                let expected = if class == 4 {
                    1. + f32::from(rank.min(3)) * 0.08
                } else {
                    1.
                };
                assert_eq!(trained.inspiration_multiplier(true), expected);
                assert_eq!(trained.inspiration_multiplier(false), 1.);
                assert_eq!(trained.with_ranks(&[]).inspiration_multiplier(true), 1.);
                for key in ["holy_nova", "renew", "smite", "holy_shock"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
        let node = super::super::talent_definition(4, 361).unwrap();
        assert!(node
            .inline_description(2)
            .unwrap()
            .contains("16 » 24% armor"));
    }
}

#[cfg(test)]
mod greater_heal_tests {
    use super::*;

    #[test]
    fn greater_heal_training_is_scoped_composed_and_refundable() {
        for class in 0..9 {
            for rank in [0, 1, 3, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[
                    (1, 408, rank),
                    (1, 1181, rank),
                    (1, 401, rank),
                    (1, 404, rank),
                ]);
                let mut expected = base.cast("greater_heal");
                if class == 4 {
                    expected.damage *= 1. + 0.02 * f32::from(rank.min(5));
                    expected.cooldown *=
                        (1. - 0.05 * f32::from(rank.min(3))) * (1. - 0.04 * f32::from(rank.min(5)));
                    expected.crit += 0.01 * f32::from(rank.min(5));
                }
                let actual = trained.cast("greater_heal");
                assert!((actual.cooldown - expected.cooldown).abs() < 0.000001);
                assert_eq!(actual.damage, expected.damage);
                assert_eq!(actual.crit, expected.crit);
                assert_eq!(
                    trained.with_ranks(&[]).cast("greater_heal"),
                    base.cast("greater_heal")
                );
                assert_eq!(
                    base.with_ranks(&[(1, 408, rank)]).cast("holy_nova"),
                    base.cast("holy_nova")
                );
                assert_eq!(
                    base.with_ranks(&[(1, 403, rank), (0, 1202, rank)])
                        .cast("greater_heal"),
                    base.cast("greater_heal")
                );
            }
        }
    }
}
