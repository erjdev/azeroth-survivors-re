use super::{definitions, Milestones, TalentStat};
use crate::creature::CreatureType;

impl Milestones {
    /// Owner damage at settlement; spell criticals receive the second slaying factor.
    /// Never bake this into a shared hit, splash copy or periodic damage snapshot.
    pub fn creature_damage_multiplier(self, kind: CreatureType, critical_hit: bool) -> f32 {
        definitions(self.class)
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                let TalentStat::CreatureDamage { mask, critical } = node.stat else {
                    return None;
                };
                kind.matches_mask(mask).then(|| {
                    let factor =
                        1.0 + node.per_rank * f32::from(self.training[index].min(node.max_rank));
                    factor
                        * if critical && critical_hit {
                            factor
                        } else {
                            1.0
                        }
                })
            })
            .product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CreatureType::*;

    #[test]
    fn all_ranks_categories_and_critical_paths_follow_the_dbc_masks() {
        for (class, id, mask, max_rank, crit_bonus) in [
            (3, 274, 211, 2, false),
            (2, 1623, 19, 3, true),
            (2, 1301, 64, 3, true),
        ] {
            for rank in 0..=max_rank + 1 {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, id, rank)]);
                for kind in [
                    Unknown,
                    Beast,
                    Dragonkin,
                    Demon,
                    Elemental,
                    Giant,
                    Undead,
                    Humanoid,
                    Critter,
                    Mechanical,
                    Unspecified,
                    Totem,
                ] {
                    for crit in [false, true] {
                        let factor = if kind.matches_mask(mask) {
                            1.0 + 0.01 * f32::from(rank.min(max_rank))
                        } else {
                            1.0
                        };
                        let expected = factor * if crit && crit_bonus { factor } else { 1.0 };
                        assert!(
                            (trained.creature_damage_multiplier(kind, crit) - expected).abs()
                                < 0.00001
                        );
                    }
                }
                assert_eq!(
                    trained
                        .with_ranks(&[])
                        .creature_damage_multiplier(Beast, true),
                    1.0
                );
            }
        }
    }

    #[test]
    fn owner_bonuses_never_leak_into_global_or_cast_snapshots() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(0, 274, 2), (2, 1623, 3), (2, 1301, 3)]);
            for key in [
                "sinister_strike",
                "instant_poison",
                "deadly_poison",
                "arcane_shot",
                "rupture",
            ] {
                assert_eq!(base.cast(key), trained.cast(key));
            }
            if !matches!(class, 2 | 3) {
                assert_eq!(trained.creature_damage_multiplier(Humanoid, true), 1.0);
            }
        }
    }
}
