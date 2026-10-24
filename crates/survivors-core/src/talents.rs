//! Survivors-specific milestone effects, independent of engine spell visuals and DBC ranks.

/// One catalog node and its proposed rank, for validating a single talent tree.
#[derive(Clone, Copy, Debug)]
pub struct RankedTalent {
    pub id: u32,
    pub row: u32,
    pub max_rank: u8,
    pub rank: u8,
    /// Required rank is one-based; DBC's zero-based prerequisite rank is converted by the adapter.
    pub prerequisite: Option<(u32, u32)>,
}

/// Validate the result of a spend or refund. Only earlier rows support a row gate;
/// neither that row's own ranks nor points in deeper rows can hold it open.
pub fn valid_tree(nodes: &[RankedTalent]) -> bool {
    nodes.iter().enumerate().all(|(index, node)| {
        if nodes[..index].iter().any(|other| other.id == node.id) || node.rank > node.max_rank {
            return false;
        }
        if node.rank == 0 {
            return true;
        }
        let earlier: u32 = nodes
            .iter()
            .filter(|other| other.row < node.row)
            .map(|other| u32::from(other.rank))
            .sum();
        if earlier < node.row.saturating_mul(5) {
            return false;
        }
        node.prerequisite.is_none_or(|(id, required)| {
            required > 0
                && id != node.id
                && nodes
                    .iter()
                    .any(|other| other.id == id && u32::from(other.rank) >= required)
        })
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Milestones {
    pub class: usize,
    pub points: [u32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CastEffect {
    pub damage: f32,
    pub radius: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub extra_targets: usize,
    pub heal_fraction: f32,
    pub wrath_advance: f32,
    pub storm_fraction: f32,
}

impl Default for CastEffect {
    fn default() -> Self {
        Self {
            damage: 1.0,
            radius: 1.0,
            duration: 1.0,
            cooldown: 1.0,
            crit: 0.0,
            extra_targets: 0,
            heal_fraction: 0.0,
            wrath_advance: 0.0,
            storm_fraction: 0.0,
        }
    }
}

impl Milestones {
    fn paladin(self, tree: usize, threshold: u32) -> bool {
        self.class == 1 && self.points[tree] >= threshold
    }

    pub fn cast(self, key: &str) -> CastEffect {
        let mut effect = CastEffect::default();
        match key {
            "holy_shock" => {
                if self.paladin(0, 10) {
                    effect.crit = 0.20;
                }
                if self.paladin(0, 20) {
                    effect.heal_fraction = 0.03;
                }
                if self.paladin(0, 30) {
                    effect.cooldown = 0.80;
                }
            }
            "consecration" if self.paladin(1, 10) => {
                effect.radius = 1.25;
                effect.duration = 1.25;
            }
            "hammer_of_justice" if self.paladin(1, 30) => effect.extra_targets = 2,
            "judgement" => {
                if self.paladin(2, 10) {
                    effect.damage = 1.25;
                }
                if self.paladin(2, 20) {
                    effect.wrath_advance = 1.0;
                }
                if self.paladin(2, 30) {
                    effect.storm_fraction = 0.5;
                }
            }
            _ => {}
        }
        effect
    }

    /// Ardent Defender reduces the portion of a blow below 35% health, including a
    /// blow crossing that threshold. A large hit cannot skip the protection entirely.
    pub fn incoming(self, damage: f32, hp: f32, max_hp: f32) -> f32 {
        if !self.paladin(1, 20) {
            return damage;
        }
        let above = (hp - max_hp * 0.35).max(0.0).min(damage);
        above + (damage - above) * 0.8
    }
}

pub fn description(class: usize, tree: usize, threshold: u32) -> Option<&'static str> {
    if class != 1 {
        return None;
    }
    Some(match (tree, threshold) {
        (0, 10) => "Holy Shock: +20% critical chance",
        (0, 20) => "Holy Shock heals 3% maximum health once per cast",
        (0, 30) => "Holy Shock: 20% shorter cooldown",
        (1, 10) => "Consecration: +25% radius and duration",
        (1, 20) => "20% less damage below 35% health",
        (1, 30) => "Hammer of Justice: +2 targets",
        (2, 10) => "Judgement: +25% damage",
        (2, 20) => "Judgement advances Hammer of Wrath by 1 second",
        (2, 30) => "Judgement releases a 6-yard Divine Storm for 50% damage",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u32, row: u32, rank: u8) -> RankedTalent {
        RankedTalent {
            id,
            row,
            rank,
            max_rank: 5,
            prerequisite: None,
        }
    }

    #[test]
    fn refund_cannot_use_deeper_points_to_support_their_own_row() {
        let mut nodes = [node(1, 0, 5), node(2, 1, 5), node(3, 2, 1)];
        assert!(valid_tree(&nodes));
        nodes[0].rank = 4;
        assert!(!valid_tree(&nodes));
        // Eleven total points still do not unlock row two with only five below it.
        let invalid = [node(1, 0, 5), node(2, 2, 5), node(3, 2, 1)];
        assert!(!valid_tree(&invalid));
        // Remove the deepest rank first, then refund surplus earlier-row points.
        nodes[0].rank = 5;
        nodes[2].rank = 0;
        nodes[1].rank = 4;
        assert!(valid_tree(&nodes));
    }

    #[test]
    fn prerequisite_rank_and_catalog_caps_survive_refunds() {
        let mut nodes = [node(1, 0, 5), node(2, 0, 5), node(3, 1, 1)];
        nodes[2].max_rank = 1;
        nodes[2].prerequisite = Some((1, 5));
        assert!(valid_tree(&nodes));
        nodes[0].rank = 4;
        assert!(
            !valid_tree(&nodes),
            "enough row points cannot replace a prerequisite"
        );
        nodes[0].rank = 5;
        nodes[2].rank = 2;
        assert!(!valid_tree(&nodes));
        nodes[2].rank = 0;
        nodes[0].rank = 4;
        assert!(valid_tree(&nodes));
    }

    #[test]
    fn each_paladin_milestone_opens_at_its_threshold() {
        for tree in 0..3 {
            for threshold in [10, 20, 30] {
                let mut before = Milestones {
                    class: 1,
                    points: [0; 3],
                };
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                let changed = [
                    "holy_shock",
                    "consecration",
                    "hammer_of_justice",
                    "judgement",
                ]
                .iter()
                .any(|key| before.cast(key) != after.cast(key))
                    || before.incoming(100.0, 20.0, 100.0) != after.incoming(100.0, 20.0, 100.0);
                assert!(changed, "tree {tree}, threshold {threshold}");
                assert!(description(1, tree, threshold).is_some());
            }
        }
    }

    #[test]
    fn effects_do_not_leak_to_other_classes_or_spells() {
        for class in [0, 2, 3, 4, 5, 6, 7, 8] {
            let m = Milestones {
                class,
                points: [51; 3],
            };
            for key in [
                "holy_shock",
                "consecration",
                "hammer_of_justice",
                "judgement",
            ] {
                assert_eq!(m.cast(key), CastEffect::default());
            }
            assert_eq!(m.incoming(100.0, 20.0, 100.0), 100.0);
        }
        let m = Milestones {
            class: 1,
            points: [51; 3],
        };
        assert_eq!(m.cast("fireball"), CastEffect::default());
        assert_eq!(m.cast("consecration").extra_targets, 0);
    }

    #[test]
    fn ardent_defender_splits_threshold_crossing_hits() {
        let m = Milestones {
            class: 1,
            points: [0, 20, 0],
        };
        assert_eq!(m.incoming(10.0, 100.0, 100.0), 10.0);
        assert_eq!(m.incoming(20.0, 45.0, 100.0), 18.0);
        assert_eq!(m.incoming(20.0, 20.0, 100.0), 16.0);
    }
}

/// A supported individual stat talent. The listed bonus replaces the temporary 1% damage
/// per point for this node; tree milestones continue to count all spent points.
#[derive(Clone, Copy, Debug)]
pub struct StatTalent {
    pub id: u32,
    pub max_rank: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub stat: TalentStat,
    pub per_rank: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum TalentStat {
    Damage,
    Cooldown,
    Armor,
    Crit,
    Reduction,
    Speed,
}

pub const PALADIN_STATS: &[StatTalent] = &[
    StatTalent {
        id: 1450,
        max_rank: 5,
        name: "Divine Strength",
        description: "+2% damage per rank",
        stat: TalentStat::Damage,
        per_rank: 0.02,
    },
    StatTalent {
        id: 1449,
        max_rank: 5,
        name: "Divine Intellect",
        description: "+1% cooldown reduction per rank",
        stat: TalentStat::Cooldown,
        per_rank: 0.01,
    },
    StatTalent {
        id: 1422,
        max_rank: 5,
        name: "Improved Devotion Aura",
        description: "+8 armor per rank",
        stat: TalentStat::Armor,
        per_rank: 8.0,
    },
    StatTalent {
        id: 1630,
        max_rank: 3,
        name: "Precision",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
    StatTalent {
        id: 1423,
        max_rank: 5,
        name: "Toughness",
        description: "+6 armor per rank",
        stat: TalentStat::Armor,
        per_rank: 6.0,
    },
    StatTalent {
        id: 1629,
        max_rank: 5,
        name: "Anticipation",
        description: "+1% damage reduction per rank",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    StatTalent {
        id: 1401,
        max_rank: 5,
        name: "Improved Blessing of Might",
        description: "+2% damage per rank",
        stat: TalentStat::Damage,
        per_rank: 0.02,
    },
    StatTalent {
        id: 1403,
        max_rank: 5,
        name: "Deflection",
        description: "+1% damage reduction per rank",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    StatTalent {
        id: 1411,
        max_rank: 5,
        name: "Conviction",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
    StatTalent {
        id: 1634,
        max_rank: 2,
        name: "Pursuit of Justice",
        description: "+4% movement speed per rank",
        stat: TalentStat::Speed,
        per_rank: 0.04,
    },
];

pub fn stat_talent(class: usize, id: u32) -> Option<&'static StatTalent> {
    (class == 1)
        .then(|| PALADIN_STATS.iter().find(|node| node.id == id))
        .flatten()
}

/// Convert active saved DBC ranks into stat adjustments. Unknown nodes retain their
/// temporary baseline; never guess an effect from a spell name or localization.
pub fn stat_adjustment(class: usize, ranks: &[(u8, u32, u8)]) -> crate::campaign::Bonus {
    let mut bonus = crate::campaign::Bonus::default();
    if class != 1 {
        return bonus;
    }
    for node in PALADIN_STATS {
        let rank = ranks
            .iter()
            .filter(|(_, id, _)| *id == node.id)
            .map(|(_, _, rank)| *rank)
            .max()
            .unwrap_or(0)
            .min(node.max_rank);
        let value = node.per_rank * f32::from(rank);
        bonus.might -= 0.01 * f32::from(rank);
        match node.stat {
            TalentStat::Damage => bonus.might += value,
            TalentStat::Cooldown => bonus.cooldown += value,
            TalentStat::Armor => bonus.armor += value,
            TalentStat::Crit => bonus.crit += value,
            TalentStat::Reduction => bonus.reduction += value,
            TalentStat::Speed => bonus.speed += value,
        }
    }
    bonus
}

#[cfg(test)]
mod stat_talent_tests {
    use super::*;

    #[test]
    fn each_named_stat_talent_replaces_its_generic_damage_at_every_rank() {
        for node in PALADIN_STATS {
            for rank in 0..=node.max_rank {
                let mut total = crate::campaign::Bonus {
                    might: 0.01 * f32::from(rank),
                    ..Default::default()
                };
                total.add(stat_adjustment(1, &[(0, node.id, rank)]));
                let value = node.per_rank * f32::from(rank);
                let actual = match node.stat {
                    TalentStat::Damage => total.might,
                    TalentStat::Cooldown => total.cooldown,
                    TalentStat::Armor => total.armor,
                    TalentStat::Crit => total.crit,
                    TalentStat::Reduction => total.reduction,
                    TalentStat::Speed => total.speed,
                };
                assert!(
                    (actual - value).abs() < 0.00001,
                    "{} rank {rank}",
                    node.name
                );
                if !matches!(node.stat, TalentStat::Damage) {
                    assert!(total.might.abs() < 0.00001);
                }
            }
        }
    }

    #[test]
    fn unsupported_classes_and_nodes_receive_no_invented_adjustment() {
        assert_eq!(stat_adjustment(0, &[(0, 1422, 5)]).armor, 0.0);
        assert_eq!(stat_adjustment(1, &[(0, 999999, 5)]).might, 0.0);
        assert_eq!(
            stat_adjustment(1, &[(0, 1422, 255), (0, 1422, 5)]).armor,
            40.0
        );
    }
}
