//! Automatic Mage cooldown mechanics use run time, excluding paused time.
use super::{Milestones, TalentStat, MAGE_ARCANE, MAGE_FIRE, MAGE_FROST};

#[derive(Clone, Copy, Debug)]
pub struct ColdSnapClock {
    next: f32,
}

impl Default for ColdSnapClock {
    fn default() -> Self {
        Self { next: 90.0 }
    }
}

impl ColdSnapClock {
    /// A long frame consumes at most one reset; missed intervals never become charges.
    pub fn trigger(&mut self, talents: Milestones, elapsed: f32) -> bool {
        if talents.class != 6
            || talents.trained_value(TalentStat::ColdSnap) <= 0.0
            || !elapsed.is_finite()
            || elapsed < self.next
        {
            return false;
        }
        self.next = (elapsed / 90.0).floor() * 90.0 + 90.0;
        true
    }
}

impl Milestones {
    pub fn cold_snap_affects(self, key: &str) -> bool {
        self.class == 6 && MAGE_FROST.contains(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snap_obeys_run_time_rank_scope_and_does_not_bank_missed_intervals() {
        for rank in [0, 1, 255] {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(2, 69, rank)]);
            let mut clock = ColdSnapClock::default();
            for time in [f32::NAN, f32::INFINITY, -1.0, 0.0, 89.99] {
                assert!(!clock.trigger(trained, time));
            }
            assert_eq!(clock.trigger(trained, 90.0), rank > 0);
            assert!(!clock.trigger(trained, 90.0));
            assert!(!clock.trigger(trained, 179.99));
            assert_eq!(clock.trigger(trained, 450.0), rank > 0);
            assert!(!clock.trigger(trained, 450.0));
            assert!(!clock.trigger(trained, 539.99));
            assert_eq!(clock.trigger(trained, 540.0), rank > 0);
            for key in MAGE_FROST {
                assert!(trained.cold_snap_affects(key));
            }
            for key in ["fireball", "arcane_explosion", "frost_shock", "volley"] {
                assert!(!trained.cold_snap_affects(key));
            }
        }
        for class in [0, 1, 2, 3, 4, 5, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(2, 69, 1)]);
            assert!(!ColdSnapClock::default().trigger(other, 90.0));
        }
        let node = super::super::definitions(6)
            .iter()
            .find(|n| n.id == 69)
            .unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 1 automatic reset"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}

/// One automatic burst window, consumed only after a successful Mage cast.
#[derive(Clone, Copy, Debug, Default)]
pub struct ArcanePowerClock {
    next: f32,
    until: f32,
}

impl ArcanePowerClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.0
            || talents.class != 6
            || !(MAGE_ARCANE.contains(&key)
                || MAGE_FIRE.contains(&key)
                || MAGE_FROST.contains(&key))
            || (elapsed >= self.until && elapsed < self.next)
        {
            return 0.0;
        }
        talents.trained_value(TalentStat::ArcanePower)
    }

    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if elapsed >= self.next && self.bonus(talents, key, elapsed) > 0.0 {
            self.until = elapsed + 15.0;
            self.next = elapsed + 180.0;
        }
    }

    pub fn remaining(self, talents: Milestones, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.0
            || talents.class != 6
            || talents.trained_value(TalentStat::ArcanePower) <= 0.0
        {
            return 0.0;
        }
        (self.until - elapsed).max(0.0)
    }
}

#[cfg(test)]
mod arcane_power_tests {
    use super::*;
    #[test]
    fn burst_waits_for_a_cast_expires_and_recovers_without_banked_charges() {
        for rank in [0, 1, 255] {
            let base = Milestones {
                class: 6,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(0, 87, rank)]);
            let expected = if rank == 0 { 0.0 } else { 0.3 };
            let mut clock = ArcanePowerClock::default();
            for elapsed in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(trained, "fireball", elapsed);
                assert_eq!(clock.bonus(trained, "fireball", elapsed), 0.0);
                assert_eq!(clock.remaining(trained, elapsed), 0.0);
            }
            clock.consume(trained, "smite", 0.0);
            assert_eq!(clock.remaining(trained, 0.0), 0.0);
            assert_eq!(clock.bonus(trained, "fireball", 1000.0), expected);
            clock.consume(trained, "fireball", 1000.0);
            for key in MAGE_ARCANE.iter().chain(MAGE_FIRE).chain(MAGE_FROST) {
                assert_eq!(clock.bonus(trained, key, 1014.99), expected);
            }
            clock.consume(trained, "frostbolt", 1014.0);
            assert_eq!(clock.bonus(trained, "fireball", 1015.0), 0.0);
            assert_eq!(clock.bonus(trained, "fireball", 1179.99), 0.0);
            assert_eq!(clock.bonus(trained, "fireball", 1180.0), expected);
            assert_eq!(clock.bonus(base, "fireball", 1180.0), 0.0);
            assert_eq!(clock.bonus(trained, "smite", 1001.0), 0.0);
            for class in [0, 3, 8] {
                assert_eq!(
                    clock.bonus(Milestones { class, ..trained }, "fireball", 1001.0),
                    0.0
                );
            }
        }
        assert!(super::super::talent_definition(6, 87)
            .unwrap()
            .inline_description(0)
            .unwrap()
            .starts_with("Mage spells: +0 » 30% direct and periodic damage"));
    }
}

/// Automatic Fire burst; only landed direct hits advance it, once before echoes.
#[derive(Clone, Copy, Debug, Default)]
pub struct CombustionClock {
    next: f32,
    hits: u8,
    criticals: u8,
}

impl CombustionClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < self.next
            || elapsed < 0.0
            || talents.class != 6
            || !MAGE_FIRE.contains(&key)
        {
            return 0.0;
        }
        (talents.trained_value(TalentStat::Combustion) * (1.0 + f32::from(self.hits))).min(1.0)
    }

    pub fn hit(
        &mut self,
        talents: Milestones,
        key: &str,
        damage: f32,
        critical: bool,
        elapsed: f32,
    ) {
        if !damage.is_finite() || damage <= 0.0 || self.bonus(talents, key, elapsed) <= 0.0 {
            return;
        }
        self.hits = self.hits.saturating_add(1).min(9);
        self.criticals += u8::from(critical);
        if self.criticals == 3 {
            self.next = elapsed + 180.0;
            self.hits = 0;
            self.criticals = 0;
        }
    }

    pub fn progress(self, talents: Milestones, elapsed: f32) -> Option<(f32, u8)> {
        let bonus = self.bonus(talents, "fireball", elapsed);
        (self.hits > 0 && bonus > 0.0).then_some((bonus, self.criticals))
    }
}

#[cfg(test)]
mod combustion_tests {
    use super::*;
    #[test]
    fn fire_hits_build_then_three_criticals_recover_without_banked_bursts() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        for rank in [0, 1, 255] {
            let talents = base.with_ranks(&[(1, 36, rank)]);
            let mut clock = CombustionClock::default();
            let initial = if rank > 0 { 0.1 } else { 0.0 };
            for key in MAGE_FIRE {
                assert_eq!(clock.bonus(talents, key, 0.0), initial);
            }
            for key in [
                "frostbolt",
                "arcane_explosion",
                "immolate",
                "Ignite",
                "Thorns",
            ] {
                clock.hit(talents, key, 10.0, true, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for damage in [0.0, -1.0, f32::NAN, f32::INFINITY] {
                clock.hit(talents, "fireball", damage, true, 0.0);
            }
            for elapsed in [-1.0, f32::NAN, f32::INFINITY] {
                clock.hit(talents, "fireball", 10.0, true, elapsed);
                assert_eq!(clock.bonus(talents, "fireball", elapsed), 0.0);
            }
            assert_eq!(clock.progress(talents, 0.0), None);
            for _ in 0..20 {
                clock.hit(talents, "fireball", 10.0, false, 0.0);
            }
            assert_eq!(
                clock.bonus(talents, "fireball", 0.0),
                if rank > 0 { 1.0 } else { 0.0 }
            );
            clock.hit(talents, "flamestrike", 10.0, true, 5.0);
            clock.hit(talents, "blast_wave", 10.0, true, 5.0);
            assert_eq!(
                clock.progress(talents, 5.0),
                if rank > 0 { Some((1.0, 2)) } else { None }
            );
            clock.hit(talents, "pyroblast", 10.0, true, 5.0);
            assert_eq!(clock.bonus(talents, "fireball", 184.99), 0.0);
            assert_eq!(clock.bonus(talents, "fireball", 185.0), initial);
            assert_eq!(clock.bonus(talents, "fireball", 10000.0), initial);
            assert_eq!(clock.progress(talents, 185.0), None);
            assert_eq!(clock.bonus(base, "fireball", 185.0), 0.0);
            for class in [0, 1, 2, 3, 4, 5, 7, 8] {
                assert_eq!(
                    clock.bonus(Milestones { class, ..talents }, "fireball", 185.0),
                    0.0
                );
            }
        }
        let node = super::super::talent_definition(6, 36).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("+0 » 10% critical chance"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod presence_tests {
    use super::*;
    #[test]
    fn presence_adds_to_the_milestone_only_for_mage_missiles() {
        for class in 0..9 {
            for points in [0, 19, 20, 30] {
                let base = Milestones {
                    class,
                    points: [points, 0, 0],
                    ..Default::default()
                };
                for rank in [0, 1, 255] {
                    let trained = base.with_ranks(&[(0, 86, rank)]);
                    for key in crate::data::CLASSES
                        .iter()
                        .flat_map(|c| c.pool.iter().copied())
                    {
                        let before = base.cast(key);
                        let after = trained.cast(key);
                        if class == 6 && key == "arcane_missiles" && rank > 0 {
                            assert_eq!(
                                after.cooldown_advance,
                                Some(("arcane_explosion", if points >= 20 { 4. } else { 2. }))
                            );
                            assert_eq!(
                                super::super::CastEffect {
                                    cooldown_advance: before.cooldown_advance,
                                    ..after
                                },
                                before
                            );
                        } else {
                            assert_eq!(after, before);
                        }
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        let node = super::super::talent_definition(6, 86).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 2 sec"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod mana_shield_tests {
    use super::*;
    #[test]
    fn mana_shield_is_in_mage_offers_and_training_only_changes_its_cooldown() {
        let shield = crate::data::ability("mana_shield");
        assert_eq!(shield.kind, crate::data::Kind::Shield);
        assert_eq!(
            (
                shield.spell,
                shield.damage,
                shield.cooldown,
                shield.duration
            ),
            (1463, 40., 18., 10.)
        );
        for (class, catalog) in crate::data::CLASSES.iter().enumerate() {
            assert_eq!(catalog.ability_keys().contains(&"mana_shield"), class == 6);
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, 83, rank)]);
                for def in crate::data::ABILITIES {
                    let before = base.cast(def.key);
                    let after = trained.cast(def.key);
                    if class == 6 && def.key == "mana_shield" {
                        assert!(
                            (after.cooldown
                                - before.cooldown * (1. - 0.1 * f32::from(rank.min(2))))
                            .abs()
                                < 0.00001
                        );
                        assert_eq!(
                            super::super::CastEffect {
                                cooldown: before.cooldown,
                                ..after
                            },
                            before
                        );
                    } else {
                        assert_eq!(after, before);
                    }
                }
            }
        }
        let node = super::super::talent_definition(6, 83).unwrap();
        assert!(node.inline_description(1).unwrap().contains("10 » 20%"));
    }
}

/// A successful hit queues one immediate extra cast; no accumulated charges.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClearcastingClock {
    next: f32,
    pending: bool,
}

impl ClearcastingClock {
    pub fn chance(self, talents: Milestones, key: &str, damage: f32, now: f32) -> f32 {
        if talents.class != 6
            || !now.is_finite()
            || now < 0.0
            || now < self.next
            || self.pending
            || !damage.is_finite()
            || damage <= 0.0
            || !(MAGE_ARCANE.contains(&key)
                || MAGE_FIRE.contains(&key)
                || MAGE_FROST.contains(&key))
        {
            return 0.0;
        }
        talents.trained_value(TalentStat::Clearcasting)
    }

    pub fn trigger(
        &mut self,
        talents: Milestones,
        key: &str,
        damage: f32,
        now: f32,
        haste: f32,
        roll: f32,
    ) -> bool {
        let chance = self.chance(talents, key, damage, now);
        if chance <= 0.0
            || !haste.is_finite()
            || !roll.is_finite()
            || !(0.0..1.0).contains(&roll)
            || roll >= chance
        {
            return false;
        }
        self.next = now + 5.0 / (1.0 + haste).max(0.1);
        self.pending = true;
        true
    }

    /// Dispatch once on the next cast update, even if no valid target remains.
    pub fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod clearcasting_tests {
    use super::*;
    #[test]
    fn chance_cooldown_and_single_dispatch_obey_rank_class_and_active_time() {
        for rank in 0..=5 {
            let talents = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(0, 75, rank)]);
            let mut clock = ClearcastingClock::default();
            assert!(
                (clock.chance(talents, "fireball", 1.0, 0.0) - 0.02 * f32::from(rank)).abs()
                    < 0.00001
            );
            assert_eq!(
                clock.trigger(talents, "fireball", 1.0, 0.0, 1.0, 0.0),
                rank > 0
            );
            assert_eq!(clock.take(), rank > 0);
            assert!(!clock.take());
            if rank > 0 {
                assert!(!clock.trigger(talents, "fireball", 1.0, 2.49, 1.0, 0.0));
                assert!(clock.trigger(talents, "arcane_missiles", 1.0, 2.5, 0.0, 0.0));
                assert!(!clock.trigger(talents, "fireball", 1.0, 100.0, 0.0, 0.0));
                assert!(clock.take());
                assert!(clock.trigger(talents, "frostbolt", 1.0, 100.0, 0.0, 0.0));
            }
            for class in [0, 1, 2, 3, 4, 5, 7, 8] {
                assert_eq!(
                    ClearcastingClock::default().chance(
                        Milestones { class, ..talents },
                        "fireball",
                        1.0,
                        0.0
                    ),
                    0.0
                );
            }
        }
    }
    #[test]
    fn rejects_invalid_hits_rolls_and_times_without_spending_the_proc() {
        let talents = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(0, 75, 5)]);
        let mut clock = ClearcastingClock::default();
        for key in ["Ignite", "Thorns", "WeaponAttack", "smite", "mana_shield"] {
            assert!(!clock.trigger(talents, key, 1.0, 0.0, 0.0, 0.0));
        }
        for damage in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(!clock.trigger(talents, "fireball", damage, 0.0, 0.0, 0.0));
        }
        for now in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(!clock.trigger(talents, "fireball", 1.0, now, 0.0, 0.0));
        }
        for roll in [-1.0, 0.1, 1.0, f32::NAN, f32::INFINITY] {
            assert!(!clock.trigger(talents, "fireball", 1.0, 0.0, 0.0, roll));
        }
        assert!(!clock.take());
        assert!(clock.trigger(talents, "fireball", 1.0, 0.0, 0.0, 0.09));
        let node = super::super::talent_definition(6, 75).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 2%"));
        assert!(!node.inline_description(5).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::super::{CastEffect, MAGE_ABILITIES, MAGE_DAMAGE};
    use super::*;

    #[test]
    fn scopes_track_the_mage_pool_and_damage_excludes_defenses() {
        let pool = crate::data::CLASSES[6].ability_keys();
        assert_eq!(MAGE_ABILITIES.len(), pool.len());
        assert!(pool.iter().all(|key| MAGE_ABILITIES.contains(key)));
        assert_eq!(MAGE_DAMAGE.len() + 2, MAGE_ABILITIES.len());
        for key in MAGE_ABILITIES {
            assert_eq!(
                MAGE_DAMAGE.contains(key),
                !matches!(*key, "mana_shield" | "counterspell")
            );
        }
    }

    #[test]
    fn conversion_ranks_change_only_their_named_effect_and_remove_provisional_damage() {
        for (tree, id, cap, per_rank, damage) in [
            (0, 85, 1, 0.05, true),
            (0, 1142, 3, 0.02, false),
            (0, 77, 5, 0.02, false),
            (1, 23, 2, 0.05, false),
        ] {
            for class in 0..9 {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                for rank in 0..=cap + 1 {
                    let ranks = [(tree, id, rank)];
                    let trained = base.with_ranks(&ranks);
                    for def in crate::data::ABILITIES {
                        let before = base.cast(def.key);
                        let after = trained.cast(def.key);
                        let eligible = class == 6
                            && if damage {
                                MAGE_DAMAGE.contains(&def.key)
                            } else if id == 23 {
                                MAGE_FIRE.contains(&def.key)
                            } else {
                                MAGE_ABILITIES.contains(&def.key)
                            };
                        let amount = per_rank * f32::from(rank.min(cap));
                        let expected = if eligible && damage {
                            CastEffect {
                                damage: before.damage * (1.0 + amount),
                                ..before
                            }
                        } else if eligible {
                            CastEffect {
                                cooldown: before.cooldown * (1.0 - amount),
                                ..before
                            }
                        } else {
                            before
                        };
                        assert_eq!(
                            after, expected,
                            "class={class} node={id} rank={rank} key={}",
                            def.key
                        );
                    }
                    let adjustment = super::super::stat_adjustment(class, &ranks);
                    assert!(
                        (adjustment.might
                            - if class == 6 {
                                -0.01 * f32::from(rank.min(cap))
                            } else {
                                0.0
                            })
                        .abs()
                            < 0.00001
                    );
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
            let node = super::super::talent_definition(6, id).unwrap();
            assert!(node.inline_description(0).unwrap().contains("0 » "));
            assert!(!node.inline_description(cap).unwrap().contains(" » "));
        }
    }

    #[test]
    fn recovery_bonuses_compose_without_affecting_the_clearcasting_clock() {
        let talents = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(0, 1142, 3), (0, 77, 5), (1, 23, 2), (0, 75, 5)]);
        for key in MAGE_ABILITIES {
            let expected = 0.94 * 0.90 * if MAGE_FIRE.contains(key) { 0.90 } else { 1.0 };
            assert!((talents.cast(key).cooldown - expected).abs() < 0.00001);
        }
        let mut clock = ClearcastingClock::default();
        assert!(clock.trigger(talents, "fireball", 1.0, 0.0, 0.0, 0.0));
        assert!(clock.take());
        assert!(!clock.trigger(talents, "fireball", 1.0, 4.99, 0.0, 0.0));
        assert!(clock.trigger(talents, "fireball", 1.0, 5.0, 0.0, 0.0));
    }
}

/// Survivors mana-return conversion: one recovery refund per haste-scaled interval.
#[derive(Clone, Copy, Debug, Default)]
pub struct MasterOfElementsClock {
    next: f32,
}

impl MasterOfElementsClock {
    pub fn refund(
        &mut self,
        talents: Milestones,
        key: &str,
        damage: f32,
        critical: bool,
        now: f32,
        haste: f32,
    ) -> f32 {
        if talents.class != 6
            || !critical
            || !damage.is_finite()
            || damage <= 0.0
            || !now.is_finite()
            || now < 0.0
            || now < self.next
            || !haste.is_finite()
            || !(MAGE_FIRE.contains(&key) || MAGE_FROST.contains(&key))
        {
            return 0.0;
        }
        let fraction = talents.trained_value(TalentStat::MasterOfElements);
        if fraction > 0.0 {
            self.next = now + 1.0 / (1.0 + haste).max(0.1);
        }
        fraction
    }
}

#[cfg(test)]
mod master_of_elements_tests {
    use super::*;

    #[test]
    fn every_rank_school_and_class_obeys_the_shared_haste_deadline() {
        for class in 0..9 {
            for rank in 0..=4 {
                let talents = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 1639, rank)]);
                for ability in crate::data::ABILITIES {
                    let mut clock = MasterOfElementsClock::default();
                    let expected = if class == 6
                        && (MAGE_FIRE.contains(&ability.key) || MAGE_FROST.contains(&ability.key))
                    {
                        rank.min(3) as f32 * 0.1
                    } else {
                        0.0
                    };
                    assert!(
                        (clock.refund(talents, ability.key, 1.0, true, 0.0, 1.0) - expected).abs()
                            < 0.00001
                    );
                    if expected > 0.0 {
                        assert_eq!(
                            clock.refund(talents, "frostbolt", 1.0, true, 0.49, 1.0),
                            0.0
                        );
                        assert!(
                            (clock.refund(talents, "frostbolt", 1.0, true, 0.5, 1.0) - expected)
                                .abs()
                                < 0.00001
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_hits_never_spend_a_refund_and_reset_is_fresh() {
        let talents = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(1, 1639, 3)]);
        let mut clock = MasterOfElementsClock::default();
        for (damage, critical, now, haste) in [
            (0.0, true, 0.0, 0.0),
            (f32::NAN, true, 0.0, 0.0),
            (1.0, false, 0.0, 0.0),
            (1.0, true, -1.0, 0.0),
            (1.0, true, f32::INFINITY, 0.0),
            (1.0, true, 0.0, f32::NAN),
        ] {
            assert_eq!(
                clock.refund(talents, "fireball", damage, critical, now, haste),
                0.0
            );
        }
        assert!(clock.refund(talents, "fireball", 1.0, true, 0.0, 0.0) > 0.0);
        assert_eq!(clock.refund(talents, "fireball", 1.0, true, 0.99, 0.0), 0.0);
        assert!(
            MasterOfElementsClock::default().refund(talents, "fireball", 1.0, true, 0.0, 0.0) > 0.0
        );
        let node = super::super::talent_definition(6, 1639).unwrap();
        assert!(node.inline_description(1).unwrap().contains("10 » 20%"));
    }
}
