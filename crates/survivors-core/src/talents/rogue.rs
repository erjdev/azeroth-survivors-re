//! Automatic Rogue cooldown mechanics use run time, excluding paused time.
use super::{Milestones, TalentStat, ROGUE_SPELLS};

#[derive(Clone, Copy, Debug)]
pub struct PreparationClock {
    next: f32,
}

impl Default for PreparationClock {
    fn default() -> Self {
        Self { next: 180.0 }
    }
}

impl PreparationClock {
    /// A long frame consumes at most one reset; missed intervals never become charges.
    pub fn trigger(&mut self, talents: Milestones, elapsed: f32) -> bool {
        if talents.class != 3
            || talents.trained_value(TalentStat::Preparation) <= 0.0
            || !elapsed.is_finite()
            || elapsed < self.next
        {
            return false;
        }
        self.next = (elapsed / 180.0).floor() * 180.0 + 180.0;
        true
    }
}

impl Milestones {
    pub fn preparation_affects(self, key: &str) -> bool {
        self.class == 3 && ROGUE_SPELLS.contains(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_obeys_run_time_rank_scope_and_does_not_bank_missed_intervals() {
        for rank in [0, 1, 255] {
            let trained = Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(2, 284, rank)]);
            let mut clock = PreparationClock::default();
            for time in [f32::NAN, f32::INFINITY, -1.0, 0.0, 179.99] {
                assert!(!clock.trigger(trained, time));
            }
            assert_eq!(clock.trigger(trained, 180.0), rank > 0);
            assert!(!clock.trigger(trained, 180.0));
            assert!(!clock.trigger(trained, 359.99));
            assert_eq!(clock.trigger(trained, 900.0), rank > 0);
            assert!(!clock.trigger(trained, 900.0));
            assert!(!clock.trigger(trained, 1079.99));
            assert_eq!(clock.trigger(trained, 1080.0), rank > 0);
            for key in ROGUE_SPELLS {
                assert!(trained.preparation_affects(key));
            }
            for key in ["fireball", "arcane_explosion", "frost_shock", "volley"] {
                assert!(!trained.preparation_affects(key));
            }
        }
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(2, 284, 1)]);
            assert!(!PreparationClock::default().trigger(other, 180.0));
        }
        let node = super::super::definitions(3)
            .iter()
            .find(|n| n.id == 284)
            .unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 1 automatic reset"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}
