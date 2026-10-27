//! Automatic Mage cooldown mechanics use run time, excluding paused time.
use super::{Milestones, TalentStat, MAGE_FROST};

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
