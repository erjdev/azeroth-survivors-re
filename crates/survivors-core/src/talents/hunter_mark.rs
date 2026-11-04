//! Acquired Hunter mark power; Survivors rank growth is authored.
impl crate::talents::Milestones {
    /// Authored eight-rank Survivors curve; training modifies power, not all damage.
    pub fn hunters_mark_power(self, ability_rank: u8, amulet_power: f32) -> f32 {
        if ability_rank == 0 {
            return 0.0;
        }
        let base = [20.0, 30.0, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0]
            [usize::from(ability_rank.min(crate::data::MAX_ABILITY_LEVEL) - 1)];
        let trained = if self.class == 2 {
            self.trained_value(crate::talents::TalentStat::HunterMarkPower)
        } else {
            0.0
        };
        let amulet = if amulet_power.is_finite() {
            amulet_power.max(0.0)
        } else {
            0.0
        };
        base * (1.0 + trained) * (1.0 + amulet)
    }
}

#[cfg(test)]
mod tests {
    use crate::{data, talents::Milestones};
    #[test]
    fn rank_training_and_own_amulet_are_separate_and_training_never_grants_mark() {
        let curve = [20.0, 30.0, 45.0, 60.0, 75.0, 90.0, 110.0, 130.0];
        for class in 0..9 {
            for training in 0..=7 {
                let t = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 1343, training)]);
                assert_eq!(t.hunters_mark_power(0, 0.05), 0.0);
                for rank in 1..=data::MAX_ABILITY_LEVEL {
                    let expected = curve[usize::from(rank - 1)]
                        * (1.0
                            + if class == 2 {
                                0.03 * f32::from(training.min(5))
                            } else {
                                0.0
                            });
                    assert!((t.hunters_mark_power(rank, 0.0) - expected).abs() < 0.0001);
                    assert!((t.hunters_mark_power(rank, 0.05) - expected * 1.05).abs() < 0.0001);
                    assert_eq!(
                        t.hunters_mark_power(rank, f32::NAN),
                        t.hunters_mark_power(rank, 0.0)
                    );
                }
                assert_eq!(t.hunters_mark_power(255, 0.0), t.hunters_mark_power(8, 0.0));
            }
        }
    }
}
