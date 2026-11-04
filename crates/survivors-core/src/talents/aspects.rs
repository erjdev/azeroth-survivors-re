//! Acquired Hunter aspects; base values follow the install, rank growth is authored.
use super::{Milestones, TalentStat};

impl Milestones {
    /// Monkey 13163 supplies 8 points; Survivors ability ranks add 2 points each.
    /// Only its own legendary amulet multiplies this chance, after training.
    pub fn monkey_dodge(self, ability_rank: u8, amulet_power: f32) -> f32 {
        if ability_rank == 0 {
            return 0.0;
        }
        let rank = ability_rank.min(crate::data::MAX_ABILITY_LEVEL);
        let base = 0.08 + 0.02 * f32::from(rank - 1);
        let power = if amulet_power.is_finite() {
            amulet_power.max(0.0)
        } else {
            0.0
        };
        let trained = if self.class == 2 {
            self.trained_value(TalentStat::MonkeyDodge)
        } else {
            0.0
        };
        ((base + trained) * (1.0 + power)).min(0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquired_aspect_rank_training_and_amulet_are_independent_and_bounded() {
        for class in 0..9 {
            for ability_rank in [0, 1, 2, 8, 255] {
                for training in [0, 1, 5, 255] {
                    let talents = Milestones {
                        class,
                        ..Default::default()
                    }
                    .with_ranks(&[(0, 1381, training)]);
                    let expected = if ability_rank > 0 {
                        0.08 + 0.02
                            * f32::from(ability_rank.min(crate::data::MAX_ABILITY_LEVEL) - 1)
                            + if class == 2 {
                                0.01 * f32::from(training.min(5))
                            } else {
                                0.0
                            }
                    } else {
                        0.0
                    };
                    assert!((talents.monkey_dodge(ability_rank, 0.0) - expected).abs() < 0.00001);
                    assert!(
                        (talents.monkey_dodge(ability_rank, 0.05) - expected * 1.05).abs()
                            < 0.00001
                    );
                    assert_eq!(
                        talents.monkey_dodge(ability_rank, f32::NAN),
                        talents.monkey_dodge(ability_rank, 0.0)
                    );
                    assert_eq!(
                        talents.with_ranks(&[]).monkey_dodge(ability_rank, 0.0),
                        Milestones {
                            class,
                            ..Default::default()
                        }
                        .monkey_dodge(ability_rank, 0.0)
                    );
                }
            }
        }
    }
}
