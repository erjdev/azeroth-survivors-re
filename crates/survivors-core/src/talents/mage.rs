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
