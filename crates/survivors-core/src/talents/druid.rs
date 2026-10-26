//! Audited Druid identities; conversions are explicit until forms and feral attacks exist.
use super::{TalentDefinition, TalentStat};
const BALANCE: &[&str] = &["wrath", "moonfire", "starfire"];
pub(super) const DEFINITIONS: &[TalentDefinition] = &[
    TalentDefinition { id: 831, max_rank: 1, name: "Nature's Swiftness", description: "Rejuvenation: 15% shorter cooldown, multiplying individual cooldown ranks and the 20-point Restoration milestone (Survivors automatic instant-cast conversion)", stat: TalentStat::SpellCooldown("rejuvenation"), per_rank: 0.15 },
    TalentDefinition { id: 844, max_rank: 1, name: "Swiftmend", description: "Rejuvenation immediately heals 5% maximum health once per successful cast; adds to the 30-point Restoration milestone for 10% total. Does not consume healing over time (Survivors automatic conversion)", stat: TalentStat::SpellHeal("rejuvenation"), per_rank: 0.05 },
    TalentDefinition { id: 764, max_rank: 2, name: "Nature's Reach", description: "Wrath, Moonfire, Starfire, Entangling Roots and Hurricane: +10% targeting range per rank. Hurricane's radius and target counts are unchanged; excludes Insect Swarm, Thorns and healing", stat: TalentStat::SpellGroupRange(&["wrath", "moonfire", "starfire", "entangling_roots", "hurricane"]), per_rank: 0.10 },
    TalentDefinition { id: 762, max_rank: 5, name: "Improved Wrath", description: "Wrath: 4% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellCooldown("wrath"), per_rank: 0.04 },
    TalentDefinition { id: 763, max_rank: 5, name: "Improved Moonfire", description: "Moonfire: +2% direct and periodic damage and +2 percentage points critical chance per rank", stat: TalentStat::SpellDamageAndCrit("moonfire"), per_rank: 0.02 },
    TalentDefinition { id: 782, max_rank: 3, name: "Improved Thorns", description: "Thorns: +10% retaliation damage per rank", stat: TalentStat::SpellDamage("thorns"), per_rank: 0.1 },
    TalentDefinition { id: 792, max_rank: 5, name: "Vengeance", description: "Wrath, Moonfire and Starfire: +10 percentage points critical damage per rank", stat: TalentStat::SpellGroupCritDamage(BALANCE), per_rank: 0.1 },
    TalentDefinition { id: 784, max_rank: 5, name: "Improved Starfire", description: "Starfire: 3% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellCooldown("starfire"), per_rank: 0.03 },
    TalentDefinition { id: 783, max_rank: 3, name: "Moonglow", description: "Wrath, Moonfire, Starfire and Rejuvenation: 3% shorter cooldown per rank (Survivors mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(&["wrath", "moonfire", "starfire", "rejuvenation"]), per_rank: 0.03 },
    TalentDefinition { id: 790, max_rank: 5, name: "Moonfury", description: "Wrath, Moonfire and Starfire: +2% direct and periodic damage per rank", stat: TalentStat::SpellGroupDamage(BALANCE), per_rank: 0.02 },
    TalentDefinition { id: 794, max_rank: 5, name: "Thick Hide", description: "+8 hero armor per rank (Survivors conversion)", stat: TalentStat::Armor, per_rank: 8.0 },
    TalentDefinition { id: 807, max_rank: 2, name: "Feline Swiftness", description: "+5% hero movement speed per rank (Survivors conversion; no form required)", stat: TalentStat::Speed, per_rank: 0.05 },
    TalentDefinition { id: 798, max_rank: 3, name: "Sharpened Claws", description: "+2 percentage points hero critical chance per rank (Survivors conversion; no form required)", stat: TalentStat::Crit, per_rank: 0.02 },
    TalentDefinition { id: 830, max_rank: 3, name: "Improved Rejuvenation", description: "Rejuvenation: +5% healing per rank", stat: TalentStat::SpellHealing("rejuvenation"), per_rank: 0.05 },
    TalentDefinition { id: 828, max_rank: 5, name: "Gift of Nature", description: "Rejuvenation: +2% healing per rank", stat: TalentStat::SpellHealing("rejuvenation"), per_rank: 0.02 },
    TalentDefinition { id: 821, max_rank: 5, name: "Improved Mark of the Wild", description: "+4 hero armor per rank (Survivors permanent Mark of the Wild conversion)", stat: TalentStat::Armor, per_rank: 4.0 },
    TalentDefinition { id: 824, max_rank: 5, name: "Improved Healing Touch", description: "Rejuvenation: 3% shorter cooldown per rank (Survivors Healing Touch cast-time conversion)", stat: TalentStat::SpellCooldown("rejuvenation"), per_rank: 0.03 },
    TalentDefinition { id: 823, max_rank: 5, name: "Nature's Focus", description: "Rejuvenation: +2% healing per rank (Survivors healing reliability conversion; no spell pushback)", stat: TalentStat::SpellHealing("rejuvenation"), per_rank: 0.02 },
    TalentDefinition { id: 841, max_rank: 5, name: "Subtlety", description: "+1 percentage point hero damage reduction per rank (Survivors threat-reduction conversion)", stat: TalentStat::Reduction, per_rank: 0.01 },
    TalentDefinition { id: 843, max_rank: 5, name: "Tranquil Spirit", description: "Rejuvenation: 2% shorter cooldown per rank (Survivors healing mana-cost conversion)", stat: TalentStat::SpellCooldown("rejuvenation"), per_rank: 0.02 },
 ];
pub(super) fn apply_milestones(points: [u32; 3], key: &str, effect: &mut super::CastEffect) {
    if points[0] >= 10 && key == "entangling_roots" {
        effect.cooldown *= 0.85;
    }
    if points[0] >= 20 && key == "moonfire" {
        effect.damage *= 1.25;
    }
    if points[0] >= 30 && BALANCE.contains(&key) {
        effect.crit += 0.05;
    }
    if key == "thorns" {
        if points[1] >= 10 {
            effect.cooldown *= 0.85;
        }
        if points[1] >= 20 {
            effect.shield_fraction = 0.05;
        }
        if points[1] >= 30 {
            effect.heal_fraction = 0.03;
        }
    }
    if key == "rejuvenation" {
        if points[2] >= 10 {
            effect.damage *= 1.2; // Heal spells store total healing in damage.
        }
        if points[2] >= 20 {
            effect.cooldown *= 0.85;
        }
        if points[2] >= 30 {
            effect.heal_fraction = 0.05;
        }
    }
}

pub(super) fn description(tree: usize, threshold: u32) -> Option<&'static str> {
    match (tree, threshold) {
        (0, 10) => Some("Entangling Roots: 15% shorter cooldown (Survivors automatic Nature's Grasp)"),
        (0, 20) => Some("Moonfire: +25% direct and periodic damage, multiplying individual ranks (Survivors Improved Moonfire milestone)"),
        (0, 30) => Some("Wrath, Moonfire and Starfire: +5 percentage points critical chance; periodic ticks do not crit (Survivors Moonkin Form; no form required)"),
        (1, 10) => Some("Thorns: 15% shorter cooldown (Survivors Ferocity; feral attacks remain separate work)"),
        (1, 20) => Some("Thorns grants a non-stacking 5% maximum-health absorb for 3 seconds once per successful cast (Survivors Heart of the Wild; no form required)"),
        (1, 30) => Some("Thorns heals 3% maximum health once per successful cast, not per retaliation (Survivors Leader of the Pack; no form required)"),
        (2, 10) => Some("Rejuvenation: +20% total healing, multiplying individual healing ranks (Survivors Improved Healing Touch)"),
        (2, 20) => Some("Rejuvenation: 15% shorter cooldown, multiplying Moonglow (Survivors automatic Nature's Swiftness)"),
        (2, 30) => Some("Rejuvenation also immediately heals 5% maximum health once per successful cast; its healing over time remains intact (Survivors automatic Swiftmend)"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::talents::{stat_adjustment, CastEffect, Milestones};
    #[test]
    fn druid_training_caps_composes_and_resets() {
        let base = Milestones {
            class: 8,
            ..Default::default()
        };
        for rank in [0, 1, 3, 5, 255] {
            let ranks: Vec<_> = DEFINITIONS.iter().map(|n| (0, n.id, rank)).collect();
            let trained = base.with_ranks(&ranks);
            let r = f32::from(rank.min(5));
            let short = f32::from(rank.min(3));
            for (key, damage, cooldown, crit, crit_damage) in [
                (
                    "wrath",
                    1. + 0.02 * r,
                    (1. - 0.04 * r) * (1. - 0.03 * short),
                    0.,
                    0.1 * r,
                ),
                (
                    "moonfire",
                    (1. + 0.02 * r).powi(2),
                    1. - 0.03 * short,
                    0.02 * r,
                    0.1 * r,
                ),
                (
                    "starfire",
                    1. + 0.02 * r,
                    (1. - 0.03 * r) * (1. - 0.03 * short),
                    0.,
                    0.1 * r,
                ),
                ("thorns", 1. + 0.1 * short, 1., 0., 0.),
                (
                    "rejuvenation",
                    (1. + 0.05 * short) * (1. + 0.02 * r).powi(2),
                    (1. - 0.03 * short)
                        * (1. - 0.03 * r)
                        * (1. - 0.02 * r)
                        * if rank > 0 { 0.85 } else { 1. },
                    0.,
                    0.,
                ),
            ] {
                let e = trained.cast(key);
                assert!((e.damage - damage).abs() < 0.00001, "{key}");
                assert!((e.cooldown - cooldown).abs() < 0.00001, "{key}");
                assert!((e.crit - crit).abs() < 0.00001, "{key}");
                assert!((e.crit_damage - crit_damage).abs() < 0.00001, "{key}");
                assert_eq!(
                    (e.radius, e.duration, e.extra_targets, e.heal_fraction),
                    (
                        1.,
                        1.,
                        0,
                        if key == "rejuvenation" && rank > 0 {
                            0.05
                        } else {
                            0.
                        }
                    )
                );
            }
            for key in ["hurricane", "entangling_roots", "insect_swarm", "fireball"] {
                let range = if matches!(key, "hurricane" | "entangling_roots") {
                    1. + 0.10 * f32::from(rank.min(2))
                } else {
                    1.
                };
                assert_eq!(
                    trained.cast(key),
                    CastEffect {
                        range,
                        ..Default::default()
                    }
                );
            }
            let bonus = stat_adjustment(8, &ranks);
            assert_eq!(bonus.armor, 12. * r);
            assert!((bonus.reduction - 0.01 * r).abs() < 0.00001);
            assert!((bonus.speed - 0.05 * f32::from(rank.min(2))).abs() < 0.00001);
            assert!((bonus.crit - 0.02 * short).abs() < 0.00001);
            assert_eq!(trained.with_ranks(&[]), base);
        }
    }
    #[test]
    fn restoration_nodes_are_individually_scoped_and_multiply_with_milestones() {
        for rank in [0, 1, 5, 255] {
            let r = f32::from(rank.min(5));
            for (id, healing, cooldown) in [
                (824, 1., 1. - 0.03 * r),
                (823, 1. + 0.02 * r, 1.),
                (843, 1., 1. - 0.02 * r),
            ] {
                let base = Milestones {
                    class: 8,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, id, rank)]);
                assert!((trained.cast("rejuvenation").damage - healing).abs() < 0.00001);
                assert!((trained.cast("rejuvenation").cooldown - cooldown).abs() < 0.00001);
                for key in ["wrath", "moonfire", "thorns", "holy_nova"] {
                    assert_eq!(trained.cast(key), CastEffect::default());
                }
                let milestones = Milestones {
                    points: [0, 0, 30],
                    ..base
                }
                .with_ranks(&[(2, id, rank)])
                .cast("rejuvenation");
                assert!((milestones.damage - healing * 1.2).abs() < 0.00001);
                assert!((milestones.cooldown - cooldown * 0.85).abs() < 0.00001);
                assert_eq!(milestones.heal_fraction, 0.05);
            }
        }
    }

    #[test]
    fn druid_previews_identify_healing_and_both_moonfire_benefits() {
        let node = |id| DEFINITIONS.iter().find(|n| n.id == id).unwrap();
        assert_eq!(
            node(763).rank_preview(1),
            "This talent: 2 » 4% damage; 2 » 4% critical chance next rank"
        );
        assert_eq!(
            node(763).rank_preview(255),
            "This talent: 10% damage; 10% critical chance (maximum rank)"
        );
        assert_eq!(
            node(830).rank_preview(1),
            "This talent: 5 to 10% healing next rank"
        );
        assert_eq!(
            node(828).rank_preview(255),
            "This talent: 10% healing (maximum rank)"
        );
    }
    #[test]
    fn druid_ranks_do_not_leak_to_other_classes() {
        let ranks: Vec<_> = DEFINITIONS.iter().map(|n| (0, n.id, 255)).collect();
        for class in 0..8 {
            let trained = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&ranks);
            for key in BALANCE.iter().chain(["thorns", "rejuvenation"].iter()) {
                assert_eq!(trained.cast(key), CastEffect::default());
            }
        }
    }
}

#[cfg(test)]
mod milestone_tests {
    use super::*;
    use crate::talents::{CastEffect, Milestones};

    #[test]
    fn druid_milestones_unlock_at_each_threshold_and_stay_scoped() {
        let base = Milestones {
            class: 8,
            ..Default::default()
        };
        for (tree, keys) in [
            (0, &["entangling_roots", "moonfire", "wrath"][..]),
            (1, &["thorns", "thorns", "thorns"][..]),
            (2, &["rejuvenation", "rejuvenation", "rejuvenation"][..]),
        ] {
            for (index, key) in keys.iter().enumerate() {
                let threshold = (index as u32 + 1) * 10;
                let mut before = base;
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                assert!(description(tree, threshold).is_some());
                assert_ne!(before.cast(key), after.cast(key));
                assert_eq!(after.cast("fireball"), CastEffect::default());
            }
        }
        let all = Milestones {
            points: [30; 3],
            ..base
        }
        .with_ranks(&[
            (0, 763, 5),
            (0, 783, 3),
            (0, 790, 5),
            (0, 782, 3),
            (2, 830, 3),
            (2, 828, 5),
        ]);
        assert!((all.cast("moonfire").damage - 1.25 * 1.1 * 1.1).abs() < 0.00001);
        assert!((all.cast("moonfire").crit - 0.15).abs() < 0.00001);
        assert!((all.cast("rejuvenation").damage - 1.2 * 1.15 * 1.1).abs() < 0.00001);
        assert!((all.cast("rejuvenation").cooldown - 0.85 * 0.91).abs() < 0.00001);
        assert!((all.cast("thorns").damage - 1.3).abs() < 0.00001);
        assert_eq!(all.cast("thorns").shield_fraction, 0.05);
        assert_eq!(all.cast("thorns").shield_seconds, 3.);
        assert_eq!(all.cast("thorns").heal_fraction, 0.03);
        assert_eq!(all.cast("rejuvenation").heal_fraction, 0.05);
        for key in [
            "wrath",
            "moonfire",
            "starfire",
            "hurricane",
            "insect_swarm",
            "entangling_roots",
            "thorns",
            "rejuvenation",
        ] {
            let e = all.cast(key);
            assert_eq!((e.radius, e.duration, e.extra_targets), (1., 1., 0));
        }
        for key in ["hurricane", "insect_swarm"] {
            assert_eq!(all.cast(key), CastEffect::default());
        }
    }
}

#[cfg(test)]
mod restoration_capstones {
    use crate::talents::Milestones;

    #[test]
    fn individual_restoration_cooldown_and_heal_compose_and_reset() {
        for class in 0..9 {
            for points in [0, 19, 20, 29, 30, 40] {
                let base = Milestones {
                    class,
                    points: [0, 0, points],
                    ..Default::default()
                };
                for rank in [0, 1, 255] {
                    let trained = base.with_ranks(&[(2, 831, rank), (2, 844, rank)]);
                    let before = base.cast("rejuvenation");
                    let after = trained.cast("rejuvenation");
                    let active = class == 8 && rank > 0;
                    assert!(
                        (after.cooldown - before.cooldown * if active { 0.85 } else { 1. }).abs()
                            < 0.00001
                    );
                    assert!(
                        (after.heal_fraction
                            - before.heal_fraction
                            - if active { 0.05 } else { 0. })
                        .abs()
                            < 0.00001
                    );
                    assert_eq!(
                        (
                            after.damage,
                            after.duration,
                            after.radius,
                            after.extra_targets
                        ),
                        (
                            before.damage,
                            before.duration,
                            before.radius,
                            before.extra_targets
                        )
                    );
                    assert_eq!(trained.with_ranks(&[]).cast("rejuvenation"), before);
                    for key in ["wrath", "thorns", "holy_shock", "renew"] {
                        assert_eq!(trained.cast(key), base.cast(key));
                    }
                }
            }
        }
    }
}
