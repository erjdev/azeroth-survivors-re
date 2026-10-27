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
