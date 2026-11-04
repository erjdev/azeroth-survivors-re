//! Spell-critical retaliation uses effective health loss, with a maximum-health cap.
use super::{Milestones, TalentStat};

impl Milestones {
    pub fn eye_for_an_eye(self, health_lost: f32, max_health: f32, critical: bool) -> f32 {
        if self.class != 1
            || !critical
            || !health_lost.is_finite()
            || health_lost <= 0.
            || !max_health.is_finite()
            || max_health <= 0.
        {
            return 0.;
        }
        (health_lost * self.trained_value(TalentStat::EyeForAnEye)).min(max_health * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eye_for_an_eye_is_class_rank_critical_scoped_and_capped() {
        for class in 0..9 {
            for rank in 0..=3 {
                let t = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(2, 1632, rank)]);
                let expected = if class == 1 {
                    15. * f32::from(rank.min(2))
                } else {
                    0.
                };
                assert!((t.eye_for_an_eye(100., 1000., true) - expected).abs() < 0.001);
                assert_eq!(t.eye_for_an_eye(100., 1000., false), 0.);
                assert_eq!(t.with_ranks(&[]).eye_for_an_eye(100., 1000., true), 0.);
                for invalid in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(t.eye_for_an_eye(invalid, 1000., true), 0.);
                    assert_eq!(t.eye_for_an_eye(100., invalid, true), 0.);
                }
                assert_eq!(
                    t.eye_for_an_eye(10000., 100., true),
                    if expected > 0. { 50. } else { 0. }
                );
            }
        }
    }
    #[test]
    fn eye_for_an_eye_highlights_return_fraction_without_changing_cap() {
        let node = super::super::talent_definition(1, 1632).unwrap();
        for rank in 0..=2 {
            let (text, ranges) = node.inline_description_with_values(rank).unwrap();
            let expected = if rank == 2 {
                "30%".to_owned()
            } else {
                format!("{} » {}%", rank * 15, (rank + 1) * 15)
            };
            assert!(ranges.iter().any(|r| text[r.clone()] == expected));
            assert!(text.contains("capped at 50%"));
        }
    }
}
