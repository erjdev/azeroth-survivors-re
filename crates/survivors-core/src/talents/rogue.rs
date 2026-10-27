//! Automatic Rogue cooldowns advance only with run time.
use super::{Milestones, TalentStat};

#[derive(Clone, Copy, Debug, Default)]
pub struct ColdBloodClock {
    next: f32,
}

impl ColdBloodClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if talents.class == 3
            && matches!(
                key,
                "sinister_strike" | "eviscerate" | "gouge" | "kidney_shot"
            )
            && elapsed.is_finite()
            && elapsed >= self.next
        {
            talents.trained_value(TalentStat::ColdBlood)
        } else {
            0.0
        }
    }

    /// Call only after a successful cast. Waiting never banks additional charges.
    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if self.bonus(talents, key, elapsed) > 0.0 {
            self.next = elapsed + 180.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_blood_scopes_and_recovers_one_cast_without_banking_charges() {
        for rank in [0, 1, 255] {
            let talents = Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(0, 280, rank)]);
            let expected = f32::from(rank > 0);
            let mut clock = ColdBloodClock::default();
            for key in ["sinister_strike", "eviscerate", "gouge", "kidney_shot"] {
                assert_eq!(clock.bonus(talents, key, 0.0), expected);
            }
            for key in ["blade_flurry", "rupture", "fireball", "volley", ""] {
                clock.consume(talents, key, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for time in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(talents, "gouge", time);
                assert_eq!(clock.bonus(talents, "gouge", time), 0.0);
            }
            assert_eq!(clock.bonus(talents, "gouge", 0.0), expected);
            clock.consume(talents, "gouge", 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 179.99), 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 180.0), expected);
            clock.consume(talents, "gouge", 1000.0);
            assert_eq!(clock.bonus(talents, "gouge", 1179.99), 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 1180.0), expected);
            assert_eq!(
                ColdBloodClock::default().bonus(talents, "gouge", 0.0),
                expected
            );
        }
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 280, 1)]);
            assert_eq!(ColdBloodClock::default().bonus(talents, "gouge", 0.0), 0.0);
        }
        let node = super::super::definitions(3)
            .iter()
            .find(|n| n.id == 280)
            .unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 100%"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}
