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
    fn endurance_is_capped_and_scoped_to_sprint() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(1, 204, rank)]);
                for key in ROGUE_SPELLS
                    .iter()
                    .copied()
                    .chain(["sprint", "charge", "fireball"])
                {
                    let mut expected = base.cast(key);
                    if class == 3 && key == "sprint" {
                        expected.cooldown *= 1.0 - 0.1 * f32::from(rank.min(2));
                    }
                    assert_eq!(trained.cast(key), expected);
                }
            }
        }
        let node = super::super::talent_definition(3, 204).unwrap();
        assert!(node
            .inline_description(1)
            .unwrap()
            .starts_with("Sprint: 10 » 20% shorter cooldown"));
        assert!(!node.inline_description(2).unwrap().contains(" » "));
    }
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

/// One automatic burst window, consumed only after a successful Rogue cast.
#[derive(Clone, Copy, Debug, Default)]
pub struct AdrenalineRushClock {
    next: f32,
    until: f32,
}

impl AdrenalineRushClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.0
            || talents.class != 3
            || !ROGUE_SPELLS.contains(&key)
            || (elapsed >= self.until && elapsed < self.next)
        {
            return 0.0;
        }
        talents.trained_value(TalentStat::AdrenalineRush)
    }

    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if elapsed >= self.next && self.bonus(talents, key, elapsed) > 0.0 {
            self.until = elapsed + 15.0;
            self.next = elapsed + 300.0;
        }
    }

    pub fn remaining(self, talents: Milestones, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.0
            || talents.class != 3
            || talents.trained_value(TalentStat::AdrenalineRush) <= 0.0
        {
            return 0.0;
        }
        (self.until - elapsed).max(0.0)
    }
}

#[cfg(test)]
mod adrenaline_rush_tests {
    use super::*;
    #[test]
    fn burst_waits_for_a_cast_expires_and_recovers_without_banked_charges() {
        for rank in [0, 1, 255] {
            let base = Milestones {
                class: 3,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(1, 205, rank)]);
            let expected = if rank == 0 { 0.0 } else { 0.5 };
            let mut clock = AdrenalineRushClock::default();
            for elapsed in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(trained, "sinister_strike", elapsed);
                assert_eq!(clock.bonus(trained, "sinister_strike", elapsed), 0.0);
                assert_eq!(clock.remaining(trained, elapsed), 0.0);
            }
            clock.consume(trained, "smite", 0.0);
            assert_eq!(clock.remaining(trained, 0.0), 0.0);
            assert_eq!(clock.bonus(trained, "sinister_strike", 1000.0), expected);
            clock.consume(trained, "sinister_strike", 1000.0);
            for key in ROGUE_SPELLS {
                assert_eq!(clock.bonus(trained, key, 1014.99), expected);
            }
            clock.consume(trained, "rupture", 1014.0);
            assert_eq!(clock.bonus(trained, "sinister_strike", 1015.0), 0.0);
            assert_eq!(clock.bonus(trained, "sinister_strike", 1299.99), 0.0);
            assert_eq!(clock.bonus(trained, "sinister_strike", 1300.0), expected);
            assert_eq!(clock.bonus(base, "sinister_strike", 1300.0), 0.0);
            assert_eq!(clock.bonus(trained, "smite", 1001.0), 0.0);
            for class in [0, 6, 8] {
                assert_eq!(
                    clock.bonus(Milestones { class, ..trained }, "sinister_strike", 1001.0),
                    0.0
                );
            }
        }
        assert!(super::super::talent_definition(3, 205)
            .unwrap()
            .inline_description(0)
            .unwrap()
            .starts_with("Rogue abilities: 0 » 50% shorter cooldown"));
    }
}
