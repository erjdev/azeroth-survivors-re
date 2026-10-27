//! Audited 1.12.1 identities with explicit Survivors spell and stat conversions.
use super::{TalentDefinition, TalentStat};

pub(super) const LIGHTNING: &[&str] = &["lightning_bolt", "chain_lightning"];
pub(super) const SHOCKS: &[&str] = &["earth_shock", "flame_shock", "frost_shock"];
pub(super) const TOTEMS: &[&str] = &["searing_totem", "magma_totem"];
const CASTS: &[&str] = &[
    "lightning_bolt",
    "chain_lightning",
    "earth_shock",
    "flame_shock",
    "frost_shock",
];

pub(super) const DEFINITIONS: &[TalentDefinition] = &[
    TalentDefinition { id: 586, max_rank: 5, name: "Improved Healing Wave", description: "Searing Totem and Magma Totem: 4% shorter deployment cooldown per rank (Survivors healing cast-time conversion). Healing still requires the 10-point Restoration milestone; pulse rate and healing per deployment are unchanged", stat: TalentStat::SpellGroupCooldown(TOTEMS), per_rank: 0.04 },
    TalentDefinition { id: 593, max_rank: 5, name: "Tidal Focus", description: "Searing Totem and Magma Totem: 2% shorter deployment cooldown per rank, multiplying other cooldown training (Survivors healing mana-cost conversion). Healing per deployment is unchanged", stat: TalentStat::SpellGroupCooldown(TOTEMS), per_rank: 0.02 },
    TalentDefinition { id: 601, max_rank: 5, name: "Anticipation", description: "+1% hero damage reduction per rank (Survivors dodge conversion; applies to all incoming damage)", stat: TalentStat::Reduction, per_rank: 0.01 },
    TalentDefinition { id: 1643, max_rank: 5, name: "Weapon Mastery", description: "+2% hero damage per rank, including spells and totems (Survivors weapon-damage conversion)", stat: TalentStat::Damage, per_rank: 0.02 },
    TalentDefinition { id: 602, max_rank: 5, name: "Flurry", description: "Direct lightning and shock criticals grant 5% haste per rank for 3 seconds; further criticals refresh without stacking. Totems, Lightning Shield and periodic damage cannot trigger it (Survivors spell-critical conversion)", stat: TalentStat::FlurryHaste, per_rank: 0.05 },
    TalentDefinition { id: 605, max_rank: 2, name: "Improved Ghost Wolf", description: "+5% hero movement speed per rank (Survivors Ghost Wolf conversion; no form required)", stat: TalentStat::Speed, per_rank: 0.05 },
    TalentDefinition { id: 1640, max_rank: 3, name: "Elemental Warding", description: "+2% hero damage reduction per rank (Survivors elemental-resistance conversion; applies to all incoming damage)", stat: TalentStat::Reduction, per_rank: 0.02 },
    TalentDefinition { id: 582, max_rank: 1, name: "Totemic Mastery", description: "Searing Totem: +20% attack range. Magma Totem: +20% pulse radius. Damage, duration, deployment healing and target count are unchanged (Survivors totem-range conversion)", stat: TalentStat::TotemicMastery, per_rank: 0.20 },
    TalentDefinition { id: 588, max_rank: 5, name: "Restorative Totems", description: "Searing Totem and Magma Totem deployment: +5% healing per rank, adding to Purification. Requires the 10-point Restoration healing milestone; excludes damage, shields and regeneration (Survivors healing-totem conversion)", stat: TalentStat::TotemHealing, per_rank: 0.05 },
    TalentDefinition { id: 592, max_rank: 5, name: "Purification", description: "Searing Totem and Magma Totem deployment: +2% healing per rank, adding to Restorative Totems. Requires the 10-point Restoration healing milestone; excludes damage, shields and regeneration (Survivors healing scope)", stat: TalentStat::TotemHealing, per_rank: 0.02 },
    TalentDefinition { id: 1641, max_rank: 2, name: "Storm Reach", description: "Lightning Bolt and Chain Lightning: +10% initial targeting range per rank. Chain Lightning bounce radius and target count are unchanged", stat: TalentStat::SpellGroupRange(LIGHTNING), per_rank: 0.10 },
    TalentDefinition { id: 564, max_rank: 5, name: "Convection", description: "Lightning Bolt, Chain Lightning and shocks: 2% shorter cooldown per rank (Survivors mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(CASTS), per_rank: 0.02 },
    TalentDefinition { id: 563, max_rank: 5, name: "Concussion", description: "Lightning Bolt, Chain Lightning and shocks: +2% direct and periodic damage per rank", stat: TalentStat::SpellGroupDamage(CASTS), per_rank: 0.02 },
    TalentDefinition { id: 561, max_rank: 3, name: "Call of Flame", description: "Searing Totem and Magma Totem: +5% damage per rank", stat: TalentStat::SpellGroupDamage(TOTEMS), per_rank: 0.05 },
    TalentDefinition { id: 575, max_rank: 5, name: "Reverberation", description: "Earth, Flame and Frost Shock: 3% shorter cooldown per rank", stat: TalentStat::SpellGroupCooldown(SHOCKS), per_rank: 0.03 },
    TalentDefinition { id: 562, max_rank: 5, name: "Call of Thunder", description: "Lightning Bolt and Chain Lightning: +1 percentage point critical chance per rank", stat: TalentStat::SpellGroupCrit(LIGHTNING), per_rank: 0.01 },
    TalentDefinition { id: 721, max_rank: 5, name: "Lightning Mastery", description: "Lightning Bolt and Chain Lightning: 4% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellGroupCooldown(LIGHTNING), per_rank: 0.04 },
    TalentDefinition { id: 565, max_rank: 1, name: "Elemental Fury", description: "Lightning Bolt, Chain Lightning and shocks: +50 percentage points critical damage", stat: TalentStat::SpellGroupCritDamage(CASTS), per_rank: 0.5 },
    TalentDefinition { id: 614, max_rank: 5, name: "Ancestral Knowledge", description: "+4 maximum health per rank (Survivors mana conversion)", stat: TalentStat::Health, per_rank: 4.0 },
    TalentDefinition { id: 613, max_rank: 5, name: "Thundering Strikes", description: "+1 percentage point hero critical chance per rank, including spells and totems (Survivors conversion)", stat: TalentStat::Crit, per_rank: 0.01 },
    TalentDefinition { id: 607, max_rank: 3, name: "Improved Lightning Shield", description: "Lightning Shield: +5% damage per rank", stat: TalentStat::SpellDamage("lightning_shield"), per_rank: 0.05 },
    TalentDefinition { id: 615, max_rank: 5, name: "Toughness", description: "+8 hero armor per rank (Survivors conversion)", stat: TalentStat::Armor, per_rank: 8.0 },
    TalentDefinition { id: 595, max_rank: 5, name: "Totemic Focus", description: "Searing Totem and Magma Totem: 3% shorter deployment cooldown per rank (Survivors mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(TOTEMS), per_rank: 0.03 },
    TalentDefinition { id: 594, max_rank: 5, name: "Tidal Mastery", description: "Lightning Bolt, Chain Lightning and shocks: +1 percentage point critical chance per rank; deployment healing cannot critically strike", stat: TalentStat::SpellGroupCrit(CASTS), per_rank: 0.01 },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::talents::{stat_adjustment, CastEffect, Milestones};

    #[test]
    fn training_composes_with_milestones_caps_ranks_and_resets() {
        let base = Milestones {
            class: 5,
            points: [20, 20, 20],
            ..Default::default()
        };
        for rank in [0, 1, 3, 5, 6, 255] {
            let ranks: Vec<_> = DEFINITIONS.iter().map(|node| (0, node.id, rank)).collect();
            let trained = base.with_ranks(&ranks);
            let r = f32::from(rank.min(5));
            let short = f32::from(rank.min(3));
            let lightning = trained.cast("lightning_bolt");
            assert!((lightning.damage - (1. + 0.02 * r)).abs() < 0.00001);
            assert!((lightning.cooldown - 0.9 * (1. - 0.02 * r) * (1. - 0.04 * r)).abs() < 0.00001);
            assert!((lightning.crit - 0.1 - 0.02 * r).abs() < 0.00001);
            assert_eq!(lightning.crit_damage, if rank == 0 { 0. } else { 0.5 });
            let shock = trained.cast("flame_shock");
            assert!(
                (shock.cooldown - 0.9 * 0.85 * (1. - 0.02 * r) * (1. - 0.03 * r)).abs() < 0.00001
            );
            assert!((shock.crit - 0.01 * r).abs() < 0.00001);
            for key in TOTEMS {
                let effect = trained.cast(key);
                assert!((effect.damage - 1.2 * (1. + 0.05 * short)).abs() < 0.00001);
                assert!(
                    (effect.cooldown - (1. - 0.03 * r) * (1. - 0.04 * r) * (1. - 0.02 * r)).abs()
                        < 0.00001
                );
                assert_eq!(effect.crit_damage, 0.);
                assert_eq!(
                    (effect.radius, effect.duration, effect.extra_targets),
                    (
                        if rank > 0 && *key == "magma_totem" {
                            1.2
                        } else {
                            1.
                        },
                        1.,
                        0
                    )
                );
            }
            let shield = trained.cast("lightning_shield");
            assert!((shield.damage - 1.2 * (1. + 0.05 * short)).abs() < 0.00001);
            assert_eq!(shield.cooldown, 1.);
            assert_eq!(trained.cast("fireball"), CastEffect::default());
            let bonus = stat_adjustment(5, &ranks);
            assert_eq!(bonus.health, 4. * r);
            assert_eq!(bonus.armor, 8. * r);
            assert!((bonus.speed - 0.05 * f32::from(rank.min(2))).abs() < 0.00001);
            assert!((bonus.reduction - 0.02 * short - 0.01 * r).abs() < 0.00001);
            assert!((bonus.crit - 0.01 * r).abs() < 0.00001);
            let supported: u32 = DEFINITIONS
                .iter()
                .map(|node| u32::from(rank.min(node.max_rank)))
                .sum();
            assert!((bonus.might + 0.01 * supported as f32 - 0.02 * r).abs() < 0.00001);
            assert_eq!(trained.with_ranks(&[]), base);
        }
    }

    #[test]
    fn foreign_and_unknown_nodes_cannot_train_shaman_spells() {
        let ranks: Vec<_> = DEFINITIONS.iter().map(|node| (0, node.id, 255)).collect();
        for class in [0, 1, 2, 3, 4, 6, 7, 8] {
            let trained = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&ranks);
            for key in CASTS
                .iter()
                .chain(TOTEMS)
                .chain(["lightning_shield"].iter())
            {
                assert_eq!(trained.cast(key), CastEffect::default());
            }
        }
        let base = Milestones {
            class: 5,
            ..Default::default()
        };
        assert_eq!(base.with_ranks(&[(0, u32::MAX, 255)]), base);
    }
}

#[cfg(test)]
mod healing_tests {
    use crate::talents::{talent_definition, Milestones};

    #[test]
    fn deployment_healing_requires_unlock_and_caps_additive_training() {
        for class in 0..9 {
            for points in [0, 9, 10, 30, 51] {
                let base = Milestones {
                    class,
                    points: [0, 0, points],
                    ..Default::default()
                };
                for rank in [0, 1, 3, 5, 255] {
                    let trained = base.with_ranks(&[(2, 588, rank), (2, 592, rank)]);
                    for key in [
                        "searing_totem",
                        "magma_totem",
                        "lightning_shield",
                        "earth_shock",
                        "renew",
                        "rejuvenation",
                        "holy_nova",
                    ] {
                        let mut expected = base.cast(key);
                        if class == 5 && super::TOTEMS.contains(&key) {
                            expected.heal_fraction *= 1. + 0.07 * f32::from(rank.min(5));
                        }
                        let actual = trained.cast(key);
                        assert!((actual.heal_fraction - expected.heal_fraction).abs() < 0.000001);
                        expected.heal_fraction = actual.heal_fraction;
                        assert_eq!(actual, expected);
                        assert_eq!(
                            trained.passive_multiplier(key),
                            base.passive_multiplier(key)
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        assert_eq!(
            talent_definition(5, 588).unwrap().rank_preview(1),
            "This talent: 5 to 10% totem deployment healing next rank"
        );
        assert_eq!(
            talent_definition(5, 592).unwrap().rank_preview(255),
            "This talent: 10% totem deployment healing (maximum rank)"
        );
    }
}

#[cfg(test)]
mod mastery_tests {
    use crate::talents::{talent_definition, Milestones};

    #[test]
    fn mastery_extends_only_the_relevant_totem_distance() {
        for class in 0..9 {
            for points in [[0; 3], [20; 3]] {
                let base = Milestones {
                    class,
                    points,
                    ..Default::default()
                };
                for rank in [0, 1, 255] {
                    let trained = base.with_ranks(&[(2, 582, rank)]);
                    for key in crate::data::CLASSES[5]
                        .pool
                        .iter()
                        .copied()
                        .chain(["fireball", "consecration"])
                    {
                        let mut expected = base.cast(key);
                        if class == 5 && rank > 0 {
                            if key == "searing_totem" {
                                expected.range *= 1.2;
                            }
                            if key == "magma_totem" {
                                expected.radius *= 1.2;
                            }
                        }
                        assert_eq!(trained.cast(key), expected, "class {class} {key}");
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        assert_eq!(
            talent_definition(5, 582).unwrap().rank_preview(0),
            "This talent: 0 to 20% Searing range and Magma radius next rank"
        );
    }
}

#[cfg(test)]
mod survival_tests {
    use crate::talents::{stat_adjustment, talent_definition};

    #[test]
    fn survival_training_is_class_scoped_and_previews_each_rank() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 255] {
                let bonus = stat_adjustment(class, &[(0, 1640, rank), (1, 605, rank)]);
                let (speed, reduction) = if class == 5 {
                    (0.05 * f32::from(rank.min(2)), 0.02 * f32::from(rank.min(3)))
                } else {
                    (0., 0.)
                };
                assert_eq!(bonus.speed, speed);
                assert_eq!(bonus.reduction, reduction);
                assert_eq!(stat_adjustment(class, &[]).speed, 0.);
                assert_eq!(stat_adjustment(class, &[]).reduction, 0.);
            }
        }
        assert_eq!(
            talent_definition(5, 605)
                .unwrap()
                .inline_description(1)
                .unwrap(),
            "+5 » 10% hero movement speed (Survivors Ghost Wolf conversion; no form required)"
        );
        assert_eq!(talent_definition(5, 1640).unwrap().inline_description(2).unwrap(),
            "+4 » 6% hero damage reduction (Survivors elemental-resistance conversion; applies to all incoming damage)");
    }
}

#[cfg(test)]
mod flurry_tests {
    use crate::talents::Milestones;

    #[test]
    fn flurry_requires_shaman_training_and_eligible_direct_criticals() {
        for class in 0..9 {
            for rank in [0, 1, 3, 5, 255] {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 602, rank)]);
                let expected = if class == 5 {
                    0.05 * f32::from(rank.min(5))
                } else {
                    0.
                };
                assert_eq!(trained.flurry_haste(false), 0.);
                assert_eq!(trained.flurry_haste(true), expected);
                for key in super::CASTS {
                    assert_eq!(trained.triggers_flurry(key, 10., true), expected > 0.);
                    for damage in [0., -1., f32::NAN, f32::INFINITY] {
                        assert!(!trained.triggers_flurry(key, damage, true));
                    }
                    assert!(!trained.triggers_flurry(key, 10., false));
                }
                for key in [
                    "searing_totem",
                    "magma_totem",
                    "lightning_shield",
                    "cleave",
                    "fireball",
                    "unknown",
                ] {
                    assert!(!trained.triggers_flurry(key, 10., true));
                }
                assert_eq!(trained.with_ranks(&[]).flurry_haste(true), 0.);
            }
        }
    }
}

#[cfg(test)]
mod enhancement_tests {
    use crate::talents::{stat_adjustment, talent_definition};

    #[test]
    fn enhancement_stats_cap_stack_and_replace_fallback_damage() {
        for class in 0..9 {
            for rank in [0, 1, 4, 5, 255] {
                let trained = stat_adjustment(class, &[(1, 601, rank), (1, 1643, rank)]);
                let r = if class == 5 {
                    f32::from(rank.min(5))
                } else {
                    0.
                };
                assert!((trained.reduction - 0.01 * r).abs() < 0.000001);
                // The two supported nodes remove their old 1%-per-point fallback.
                assert!(trained.might.abs() < 0.000001);
                assert_eq!(stat_adjustment(class, &[]).reduction, 0.);
            }
        }
        let stacked = stat_adjustment(5, &[(1, 601, 5), (0, 1640, 3)]);
        assert!((stacked.reduction - 0.11).abs() < 0.000001);
        assert_eq!(talent_definition(5, 601).unwrap().inline_description(4).unwrap(),
            "+4 » 5% hero damage reduction (Survivors dodge conversion; applies to all incoming damage)");
        assert_eq!(talent_definition(5, 1643).unwrap().inline_description(4).unwrap(),
            "+8 » 10% hero damage, including spells and totems (Survivors weapon-damage conversion)");
    }
}

#[cfg(test)]
mod healing_cadence_tests {
    use crate::talents::{talent_definition, Milestones};

    #[test]
    fn healing_cadence_changes_only_totem_deployment_cooldown() {
        for class in 0..9 {
            for points in [0, 9, 10, 30] {
                let base = Milestones {
                    class,
                    points: [0, 0, points],
                    ..Default::default()
                };
                for rank in [0, 1, 4, 5, 255] {
                    let trained = base.with_ranks(&[(2, 586, rank), (2, 593, rank)]);
                    for key in crate::data::CLASSES[5]
                        .pool
                        .iter()
                        .copied()
                        .chain(["renew", "rejuvenation"])
                    {
                        let mut expected = base.cast(key);
                        if class == 5 && super::TOTEMS.contains(&key) {
                            let r = f32::from(rank.min(5));
                            expected.cooldown *= (1. - 0.04 * r) * (1. - 0.02 * r);
                        }
                        assert_eq!(trained.cast(key), expected);
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        assert!(talent_definition(5, 586)
            .unwrap()
            .inline_description(4)
            .unwrap()
            .contains("16 » 20% shorter deployment cooldown"));
        assert!(talent_definition(5, 593)
            .unwrap()
            .inline_description(4)
            .unwrap()
            .contains("8 » 10% shorter deployment cooldown"));
    }
}
