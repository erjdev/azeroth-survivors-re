//! User-approved no-mana conversion: half the triggering heal's final recovery.
use super::{Milestones, TalentStat};

impl Milestones {
    pub fn illumination_chance(self, source: &str, healed: f32, critical: bool) -> f32 {
        if self.class == 1
            && critical
            && healed.is_finite()
            && healed > 0.
            && matches!(source, "holy_light" | "flash_of_light")
        {
            self.trained_value(TalentStat::Illumination)
        } else {
            0.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chance_requires_effective_critical_paladin_healing_and_active_training() {
        for class in 0..9 {
            for rank in 0..=6 {
                let t = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, 1461, rank)]);
                for key in [
                    "holy_light",
                    "flash_of_light",
                    "holy_shock",
                    "healing_wave",
                    "holy_nova",
                    "regrowth",
                    "consecration",
                ] {
                    for critical in [false, true] {
                        let expected = if class == 1
                            && critical
                            && matches!(key, "holy_light" | "flash_of_light")
                        {
                            0.2 * f32::from(rank.min(5))
                        } else {
                            0.
                        };
                        assert_eq!(t.illumination_chance(key, 1., critical), expected);
                        for healed in [0., -1., f32::NAN, f32::INFINITY] {
                            assert_eq!(t.illumination_chance(key, healed, critical), 0.);
                        }
                        assert_eq!(t.with_ranks(&[]).illumination_chance(key, 1., critical), 0.);
                    }
                }
            }
        }
    }
    #[test]
    fn chance_growth_is_inline_without_changing_the_refund_fraction() {
        let node = super::super::talent_definition(1, 1461).unwrap();
        for rank in 0..=5 {
            let (text, ranges) = node.inline_description_with_values(rank).unwrap();
            let expected = if rank == 5 {
                "100% chance".to_owned()
            } else {
                format!("{} » {}% chance", rank * 20, (rank + 1) * 20)
            };
            assert!(ranges.iter().any(|r| text[r.clone()] == expected));
            assert!(text.contains("half that cast's recovery time"));
        }
    }
}
