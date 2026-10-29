//! Audited Warlock identities with shared Survivors spell effects.
use super::{TalentDefinition, TalentStat};
const SHADOW: &[&str] = &[
    "shadow_bolt",
    "corruption",
    "curse_of_agony",
    "death_coil",
    "shadowburn",
];
const FIRE: &[&str] = &["immolate", "rain_of_fire", "hellfire", "searing_pain"];
const DESTRUCTION: &[&str] = &[
    "shadow_bolt",
    "immolate",
    "rain_of_fire",
    "hellfire",
    "searing_pain",
    "shadowburn",
];
pub(super) const DEFINITIONS: &[TalentDefinition] = &[
    TalentDefinition { id: 1006, max_rank: 3, name: "Improved Curse of Weakness", description: "Curse of Weakness: 6–20% stronger reduction to each physical hit. Requires acquiring the curse; duration and targets are unchanged. Replaces your other curse on each affected enemy", stat: TalentStat::ImprovedCurseWeakness, per_rank: 1.0 },
    TalentDefinition { id: 1005, max_rank: 5, name: "Suppression", description: "Corruption, Curse of Agony and Death Coil: +2% direct and periodic damage per rank. Excludes other Warlock spells and other-class abilities (Survivors hit-chance conversion)", stat: TalentStat::SpellGroupDamage(&["corruption", "curse_of_agony", "death_coil"]), per_rank: 0.02 },
    TalentDefinition { id: 968, max_rank: 1, name: "Conflagrate", description: "Immolate advances an owned Shadowburn cooldown by 2 sec once per successful cast; adds to the 30-point Destruction milestone for 4 sec total. Does not consume the burn, grant Shadowburn or store a charge (Survivors automatic conversion)", stat: TalentStat::Conflagrate, per_rank: 2.0 },
    TalentDefinition { id: 1004, max_rank: 5, name: "Improved Drain Life", description: "Death Coil: +2% drain healing per rank, based on enemy health actually removed. Damage, cooldown and Healthstone healing are unchanged. Requires acquiring Death Coil (Survivors Drain Life conversion)", stat: TalentStat::ImprovedDrainLife, per_rank: 0.02 },
    TalentDefinition { id: 1282, max_rank: 1, name: "Soul Link", description: "Death Coil grants a non-stacking absorb for 5% maximum health lasting 3 sec, once per successful cast. Adds to the Demonology 30-point milestone for 10% total. Requires acquiring Death Coil; preserves stronger shields (Survivors automatic conversion; no demon required)", stat: TalentStat::SpellShield("death_coil"), per_rank: 0.05 },
    TalentDefinition { id: 1081, max_rank: 1, name: "Curse of Exhaustion", description: "Curse of Agony applications slow targets by 10% for 12 sec. Requires acquiring Curse of Agony; damage and target count are unchanged. Refreshes without stacking and preserves stronger slows (Survivors curse conversion)", stat: TalentStat::CurseExhaustion, per_rank: 0.10 },
    TalentDefinition { id: 1082, max_rank: 4, name: "Improved Curse of Exhaustion", description: "Curse of Exhaustion: +5% additional slow per rank. Requires the base talent; bosses cannot be slowed by more than 25%. Duration and damage are unchanged (Survivors conversion)", stat: TalentStat::ImprovedCurseExhaustion, per_rank: 0.05 },
    TalentDefinition { id: 985, max_rank: 2, name: "Intensity", description: "Rain of Fire: +10% active duration per rank. Pulse interval, damage per pulse, radius and cooldown are unchanged (Survivors channel-resilience conversion; no channel required)", stat: TalentStat::SpellDuration("rain_of_fire"), per_rank: 0.10 },
    TalentDefinition { id: 982, max_rank: 5, name: "Aftermath", description: "Destruction hits and area pulses: 2% chance per rank to slow enemies by 50% for 5 sec. Bosses are slowed by 25%; stronger slows are preserved. Excludes attached damage-over-time ticks; refreshes without stacking (Survivors daze conversion)", stat: TalentStat::Aftermath, per_rank: 0.02 },
    TalentDefinition { id: 986, max_rank: 2, name: "Pyroclasm", description: "Rain of Fire and Hellfire pulses: 13% chance per rank to stun each damaged enemy for 3 sec. Boss duration is reduced; longer stuns are preserved. One roll per hit, not per multistrike; refreshes without stacking (Survivors conversion)", stat: TalentStat::Pyroclasm, per_rank: 0.13 },
    TalentDefinition { id: 1061, max_rank: 1, name: "Amplify Curse", description: "Curse of Agony: +20% periodic damage, multiplying other training. Requires acquiring the spell during the run (Survivors automatic conversion)", stat: TalentStat::SpellDamage("curse_of_agony"), per_rank: 0.20 },
    TalentDefinition { id: 963, max_rank: 1, name: "Shadowburn", description: "Shadowburn: +20% damage, multiplying other training. Requires acquiring Shadowburn during the run; does not grant the spell (Survivors conversion)", stat: TalentStat::SpellDamage("shadowburn"), per_rank: 0.20 },
    TalentDefinition { id: 1221, max_rank: 2, name: "Improved Healthstone", description: "Death Coil heals an additional 1% maximum health per rank once per successful cast. Adds to the Demonology 10-point milestone; drain healing is unchanged (Survivors automatic conversion)", stat: TalentStat::SpellHeal("death_coil"), per_rank: 0.01 },
    TalentDefinition { id: 1002, max_rank: 2, name: "Nightfall", description: "Corruption ticks: 2% chance per rank to ready an owned Shadow Bolt. Once per damaging tick, not per multistrike; no stored charge. Separate from the 20-point milestone (Survivors instant-cast conversion)", stat: TalentStat::Nightfall, per_rank: 0.02 },
    TalentDefinition { id: 1041, max_rank: 1, name: "Siphon Life", description: "Corruption and Curse of Agony heal 5% of enemy health actually removed; adds to the 30-point Affliction milestone for 10% total. Excludes overkill and other spells (Survivors automatic conversion)", stat: TalentStat::SiphonLife, per_rank: 0.05 },
    TalentDefinition { id: 1021, max_rank: 2, name: "Grim Reach", description: "Corruption, Curse of Agony, Curse of Weakness and Death Coil: +10% targeting range per rank; damage, duration and target counts are unchanged", stat: TalentStat::SpellGroupRange(&["corruption", "curse_of_agony", "curse_of_weakness", "death_coil"]), per_rank: 0.10 },
    TalentDefinition { id: 964, max_rank: 2, name: "Destructive Reach", description: "Shadow Bolt, Immolate, Searing Pain, Shadowburn and Rain of Fire: +10% targeting range per rank. Rain of Fire's radius is unchanged; excludes player-centered Hellfire", stat: TalentStat::SpellGroupRange(&["shadow_bolt", "immolate", "searing_pain", "shadowburn", "rain_of_fire"]), per_rank: 0.10 },
    TalentDefinition { id: 1003, max_rank: 5, name: "Improved Corruption", description: "Corruption: 4% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellCooldown("corruption"), per_rank: 0.04 },
    TalentDefinition { id: 1284, max_rank: 3, name: "Improved Curse of Agony", description: "Curse of Agony: +2% periodic damage per rank", stat: TalentStat::SpellDamage("curse_of_agony"), per_rank: 0.02 },
    TalentDefinition { id: 1042, max_rank: 5, name: "Shadow Mastery", description: "Shadow Bolt, Corruption, Curse of Agony, Death Coil and Shadowburn: +2% damage per rank", stat: TalentStat::SpellGroupDamage(SHADOW), per_rank: 0.02 },
    TalentDefinition { id: 1223, max_rank: 5, name: "Demonic Embrace", description: "+5 maximum health per rank (Survivors stamina conversion)", stat: TalentStat::Health, per_rank: 5.0 },
    TalentDefinition { id: 944, max_rank: 5, name: "Improved Shadow Bolt", description: "Shadow Bolt: +4% damage per rank (Survivors conversion from critical-hit vulnerability)", stat: TalentStat::SpellDamage("shadow_bolt"), per_rank: 0.04 },
    TalentDefinition { id: 941, max_rank: 5, name: "Cataclysm", description: "Destruction spells: 2% shorter cooldown per rank (Survivors mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(DESTRUCTION), per_rank: 0.02 },
    TalentDefinition { id: 943, max_rank: 5, name: "Bane", description: "Shadow Bolt and Immolate: 4% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellGroupCooldown(&["shadow_bolt", "immolate"]), per_rank: 0.04 },
    TalentDefinition { id: 981, max_rank: 5, name: "Devastation", description: "Destruction spells: +1 percentage point critical chance per rank", stat: TalentStat::SpellGroupCrit(DESTRUCTION), per_rank: 0.01 },
    TalentDefinition { id: 965, max_rank: 5, name: "Improved Searing Pain", description: "Searing Pain: +2 percentage points critical chance per rank", stat: TalentStat::SpellCrit("searing_pain"), per_rank: 0.02 },
    TalentDefinition { id: 961, max_rank: 5, name: "Improved Immolate", description: "Immolate: +5% direct and periodic damage per rank (Survivors conversion)", stat: TalentStat::SpellDamage("immolate"), per_rank: 0.05 },
    TalentDefinition { id: 967, max_rank: 1, name: "Ruin", description: "Destruction spells: +50 percentage points critical damage", stat: TalentStat::SpellGroupCritDamage(DESTRUCTION), per_rank: 0.5 },
    TalentDefinition { id: 966, max_rank: 5, name: "Emberstorm", description: "Immolate, Rain of Fire, Hellfire and Searing Pain: +2% damage per rank", stat: TalentStat::SpellGroupDamage(FIRE), per_rank: 0.02 },
 ];
pub(super) fn apply_milestones(points: [u32; 3], key: &str, effect: &mut super::CastEffect) {
    if key == "corruption" {
        if points[0] >= 10 {
            effect.cooldown *= 0.9;
        }
        if points[0] >= 20 {
            effect.cooldown_advance = Some(("shadow_bolt", 1.0));
        }
    }
    if key == "death_coil" {
        if points[1] >= 10 {
            effect.heal_fraction = 0.03;
        }
        if points[1] >= 30 {
            effect.shield_fraction = 0.05;
        }
    }
    if points[1] >= 20 && (SHADOW.contains(&key) || FIRE.contains(&key)) {
        effect.damage *= 1.1;
    }
    if points[2] >= 10 && key == "shadow_bolt" {
        effect.damage *= 1.25;
    }
    if points[2] >= 20 && DESTRUCTION.contains(&key) {
        effect.crit_damage += 0.2;
    }
    if points[2] >= 30 && key == "immolate" {
        effect.cooldown_advance = Some(("shadowburn", 2.0));
    }
}

pub(super) fn description(tree: usize, threshold: u32) -> Option<&'static str> {
    match (tree,threshold) {
        (0,10) => Some("Corruption: 10% shorter cooldown, multiplying individual ranks (Survivors Improved Corruption milestone)"),
        (0,20) => Some("Corruption advances an owned Shadow Bolt cooldown by 1 second once per successful cast, not per target or tick (Survivors automatic Nightfall)"),
        (0,30) => Some("Corruption and Curse of Agony heal 5% of enemy health actually removed; overkill cannot add healing (Survivors Siphon Life)"),
        (1,10) => Some("Death Coil heals 3% maximum health once per successful cast, in addition to its drain (Survivors Improved Healthstone)"),
        (1,20) => Some("Warlock spells: +10% direct and periodic damage (Survivors automatic Demonic Sacrifice; no demon required)"),
        (1,30) => Some("Death Coil grants a non-stacking 5% maximum-health absorb for 3 seconds once per successful cast (Survivors Soul Link; no demon required)"),
        (2,10) => Some("Shadow Bolt: +25% damage, multiplying individual ranks (Survivors Improved Shadow Bolt milestone)"),
        (2,20) => Some("Destruction spells: +20 percentage points critical damage, adding to individual Ruin ranks"),
        (2,30) => Some("Immolate advances an owned Shadowburn cooldown by 2 seconds once per successful cast, not per target or burn tick (Survivors automatic Conflagrate)"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::talents::{stat_adjustment, CastEffect, Milestones};
    #[test]
    fn conflagrate_composes_with_milestone_without_changing_immolate() {
        for points in [0, 29, 30] {
            let base = Milestones {
                class: 7,
                points: [0, 0, points],
                ..Default::default()
            };
            for rank in [0, 1, 255] {
                let trained = base.with_ranks(&[(2, 968, rank)]);
                let mut expected = base.cast("immolate");
                if rank > 0 {
                    expected.cooldown_advance =
                        Some(("shadowburn", if points >= 30 { 4. } else { 2. }));
                }
                assert_eq!(trained.cast("immolate"), expected);
                assert_eq!(
                    trained.with_ranks(&[]).cast("immolate"),
                    base.cast("immolate")
                );
                for key in ["corruption", "shadowburn", "rain_of_fire", "fireball"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
        let node = DEFINITIONS.iter().find(|node| node.id == 968).unwrap();
        let (text, values) = node.inline_description_with_values(0).unwrap();
        assert_eq!(&text[values[0].clone()], "0 » 2 sec");
        assert!(text.contains("4 sec total"));
        let (text, values) = node.inline_description_with_values(1).unwrap();
        assert_eq!(&text[values[0].clone()], "2 sec");
    }

    #[test]
    fn improved_drain_life_changes_only_owned_spell_drain_and_resets() {
        use crate::data::Effect;
        let base = Milestones {
            class: 7,
            points: [30; 3],
            ..Default::default()
        };
        for rank in [0, 1, 5, 255] {
            let trained = base.with_ranks(&[(0, 1004, rank)]);
            let Effect::Drain(fraction) = trained.control("death_coil", Effect::Drain(0.5)) else {
                panic!("drain lost")
            };
            assert!((fraction - 0.5 * (1.0 + 0.02 * f32::from(rank.min(5)))).abs() < 0.00001);
            for key in SHADOW.iter().chain(FIRE) {
                assert_eq!(trained.cast(key), base.cast(key));
            }
            assert_eq!(
                trained.control("corruption", Effect::Drain(0.5)),
                Effect::Drain(0.5)
            );
            assert_eq!(trained.control("death_coil", Effect::None), Effect::None);
            assert_eq!(
                trained
                    .with_ranks(&[])
                    .control("death_coil", Effect::Drain(0.5)),
                Effect::Drain(0.5)
            );
        }
    }
    #[test]
    fn soul_link_adds_to_milestone_and_refunds_independently() {
        for points in [0, 29, 30] {
            let base = Milestones {
                class: 7,
                points: [0, points, 0],
                ..Default::default()
            };
            for rank in [0, 1, 255] {
                let trained = base.with_ranks(&[(1, 1282, rank)]);
                let before = base.cast("death_coil");
                let mut expected = before;
                expected.shield_fraction += if rank > 0 { 0.05 } else { 0. };
                expected.shield_seconds = if expected.shield_fraction > 0. {
                    3.
                } else {
                    0.
                };
                assert_eq!(trained.cast("death_coil"), expected);
                assert_eq!(trained.with_ranks(&[]).cast("death_coil"), before);
                for key in ["corruption", "shadow_bolt", "fireball"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
    }
    #[test]
    fn warlock_ranks_compose_cap_and_reset_without_changing_geometry() {
        let base = Milestones {
            class: 7,
            ..Default::default()
        };
        for rank in [0, 1, 3, 5, 255] {
            let ranks: Vec<_> = DEFINITIONS.iter().map(|node| (0, node.id, rank)).collect();
            let trained = base.with_ranks(&ranks);
            let r = f32::from(rank.min(5));
            for (key, damage, cooldown, crit, crit_damage) in [
                (
                    "shadow_bolt",
                    (1. + 0.04 * r) * (1. + 0.02 * r),
                    (1. - 0.02 * r) * (1. - 0.04 * r),
                    0.01 * r,
                    if rank > 0 { 0.5 } else { 0. },
                ),
                (
                    "immolate",
                    (1. + 0.05 * r) * (1. + 0.02 * r),
                    (1. - 0.02 * r) * (1. - 0.04 * r),
                    0.01 * r,
                    if rank > 0 { 0.5 } else { 0. },
                ),
                (
                    "corruption",
                    (1. + 0.02 * r) * (1. + 0.02 * r),
                    1. - 0.04 * r,
                    0.,
                    0.,
                ),
                (
                    "curse_of_agony",
                    (1. + 0.02 * f32::from(rank.min(3)))
                        * (1. + 0.02 * r)
                        * (1. + 0.02 * r)
                        * if rank > 0 { 1.2 } else { 1.0 },
                    1.,
                    0.,
                    0.,
                ),
                (
                    "searing_pain",
                    1. + 0.02 * r,
                    1. - 0.02 * r,
                    0.03 * r,
                    if rank > 0 { 0.5 } else { 0. },
                ),
                ("death_coil", (1. + 0.02 * r) * (1. + 0.02 * r), 1., 0., 0.),
            ] {
                let effect = trained.cast(key);
                assert!((effect.damage - damage).abs() < 0.00001, "{key}");
                assert!((effect.cooldown - cooldown).abs() < 0.00001, "{key}");
                assert!((effect.crit - crit).abs() < 0.00001, "{key}");
                assert_eq!(effect.crit_damage, crit_damage);
                assert_eq!(
                    (effect.radius, effect.duration, effect.extra_targets),
                    (
                        1.,
                        if key == "rain_of_fire" {
                            1. + 0.1 * f32::from(rank.min(2))
                        } else {
                            1.
                        },
                        0
                    )
                );
            }
            assert_eq!(trained.cast("fireball"), CastEffect::default());
            assert_eq!(stat_adjustment(7, &ranks).health, 5. * r);
            assert_eq!(trained.with_ranks(&[]), base);
        }
    }
    #[test]
    fn warlock_training_is_class_scoped_and_unknown_ids_are_inert() {
        let ranks: Vec<_> = DEFINITIONS.iter().map(|node| (0, node.id, 255)).collect();
        for class in [0, 1, 2, 3, 4, 5, 6, 8] {
            let trained = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&ranks);
            for key in SHADOW.iter().chain(FIRE) {
                assert_eq!(trained.cast(key), CastEffect::default());
            }
        }
        let base = Milestones {
            class: 7,
            ..Default::default()
        };
        assert_eq!(base.with_ranks(&[(0, u32::MAX, 255)]), base);
    }
}

#[cfg(test)]
mod milestone_tests {
    use super::*;
    use crate::talents::{CastEffect, Milestones};
    #[test]
    fn milestones_unlock_at_exact_thresholds_and_compose_with_training() {
        let base = Milestones {
            class: 7,
            ..Default::default()
        };
        for tree in 0..3 {
            for threshold in [10, 20, 30] {
                let mut before = base;
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                assert!(description(tree, threshold).is_some());
                let cast_changed = SHADOW
                    .iter()
                    .chain(FIRE)
                    .any(|k| before.cast(k) != after.cast(k));
                assert!(
                    cast_changed
                        || before.vampiric_healing("corruption", 100.)
                            != after.vampiric_healing("corruption", 100.)
                );
                assert_eq!(after.cast("fireball"), CastEffect::default());
            }
        }
        let all = Milestones {
            points: [30; 3],
            ..base
        }
        .with_ranks(&[(0, 1003, 5), (0, 1042, 5), (2, 944, 5), (2, 967, 1)]);
        assert!((all.cast("shadow_bolt").damage - 1.1 * 1.25 * 1.1 * 1.2).abs() < 0.00001);
        assert!((all.cast("corruption").cooldown - 0.72).abs() < 0.00001);
        assert!((all.cast("shadow_bolt").crit_damage - 0.7).abs() < 0.00001);
        assert_eq!(
            all.cast("corruption").cooldown_advance,
            Some(("shadow_bolt", 1.))
        );
        assert_eq!(
            all.cast("immolate").cooldown_advance,
            Some(("shadowburn", 2.))
        );
        assert_eq!(
            (
                all.cast("death_coil").heal_fraction,
                all.cast("death_coil").shield_fraction,
                all.cast("death_coil").shield_seconds
            ),
            (0.03, 0.05, 3.)
        );
        for key in SHADOW.iter().chain(FIRE) {
            let e = all.cast(key);
            assert_eq!((e.radius, e.duration, e.extra_targets), (1., 1., 0));
        }
    }
    #[test]
    fn siphon_life_requires_affliction_threshold_and_scoped_positive_damage() {
        for points in [0, 29, 30, 31] {
            let talent = Milestones {
                class: 7,
                points: [points, 30, 30],
                ..Default::default()
            };
            for key in ["corruption", "curse_of_agony"] {
                assert_eq!(
                    talent.vampiric_healing(key, 100.),
                    if points >= 30 { 5. } else { 0. }
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(talent.vampiric_healing(key, damage), 0.);
                }
            }
            for key in ["shadow_bolt", "immolate", "death_coil", "shadow_word_pain"] {
                assert_eq!(talent.vampiric_healing(key, 100.), 0.);
            }
        }
        for class in [0, 1, 2, 3, 4, 5, 6, 8] {
            assert_eq!(
                Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                }
                .vampiric_healing("corruption", 100.),
                0.
            );
        }
    }
}

#[cfg(test)]
mod siphon_tests {
    use crate::talents::Milestones;

    #[test]
    fn siphon_training_composes_with_milestone_and_clears_on_spec_reset() {
        for points in [0, 20, 29, 30, 40] {
            let base = Milestones {
                class: 7,
                points: [points, 0, 0],
                ..Default::default()
            };
            for rank in [0, 1, 255] {
                let trained = base.with_ranks(&[(0, 1041, rank)]);
                let fraction =
                    if points >= 30 { 0.05 } else { 0.0 } + if rank > 0 { 0.05 } else { 0.0 };
                for key in ["corruption", "curse_of_agony"] {
                    assert_eq!(trained.vampiric_healing(key, 100.), 100. * fraction);
                    for damage in [0., -1., f32::NAN, f32::INFINITY] {
                        assert_eq!(trained.vampiric_healing(key, damage), 0.);
                    }
                    assert_eq!(
                        trained.with_ranks(&[]).vampiric_healing(key, 100.),
                        base.vampiric_healing(key, 100.)
                    );
                }
                for key in ["death_coil", "shadow_bolt", "immolate", "shadow_word_pain"] {
                    assert_eq!(trained.vampiric_healing(key, 100.), 0.);
                }
            }
        }
        for class in [0, 1, 2, 3, 4, 5, 6, 8] {
            let trained = Milestones {
                class,
                points: [30; 3],
                ..Default::default()
            }
            .with_ranks(&[(0, 1041, 255)]);
            assert_eq!(trained.vampiric_healing("corruption", 100.), 0.);
        }
    }
}

#[cfg(test)]
mod nightfall_tests {
    use crate::talents::Milestones;

    #[test]
    fn nightfall_requires_individual_training_and_positive_corruption_damage() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, 1002, rank)]);
                assert_eq!(
                    trained.nightfall_chance("corruption", 10.),
                    if class == 7 {
                        0.02 * f32::from(rank.min(2))
                    } else {
                        0.
                    }
                );
                for key in ["curse_of_agony", "shadow_bolt", "immolate", "drain_life"] {
                    assert_eq!(trained.nightfall_chance(key, 10.), 0.);
                }
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.nightfall_chance("corruption", damage), 0.);
                }
                assert_eq!(
                    trained.with_ranks(&[]).nightfall_chance("corruption", 10.),
                    0.
                );
            }
        }
    }
}

#[cfg(test)]
mod additional_training_tests {
    use crate::talents::Milestones;

    #[test]
    fn new_nodes_are_scoped_capped_and_preserve_other_cast_fields() {
        for class in 0..9 {
            for points in [[0; 3], [30; 3]] {
                let base = Milestones {
                    class,
                    points,
                    ..Default::default()
                };
                for rank in [0, 1, 2, 255] {
                    let trained =
                        base.with_ranks(&[(0, 1061, rank), (1, 1221, rank), (2, 963, rank)]);
                    for spell in crate::data::ABILITIES {
                        let mut expected = base.cast(spell.key);
                        if class == 7 {
                            if rank > 0 && matches!(spell.key, "curse_of_agony" | "shadowburn") {
                                expected.damage *= 1.2;
                            }
                            if spell.key == "death_coil" {
                                expected.heal_fraction += 0.01 * f32::from(rank.min(2));
                            }
                        }
                        assert_eq!(trained.cast(spell.key), expected, "{class} {}", spell.key);
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }
}

impl super::Milestones {
    /// Direct hits and area pulse messages only; attached DoT ticks have no control rolls.
    pub fn aftermath_chance(self, key: &str, damage: f32) -> f32 {
        if self.class == 7 && damage.is_finite() && damage > 0. && DESTRUCTION.contains(&key) {
            self.trained_value(TalentStat::Aftermath)
        } else {
            0.
        }
    }

    pub fn pyroclasm_chance(self, key: &str, damage: f32) -> f32 {
        if self.class == 7
            && damage.is_finite()
            && damage > 0.
            && matches!(key, "rain_of_fire" | "hellfire")
        {
            self.trained_value(TalentStat::Pyroclasm)
        } else {
            0.
        }
    }
}

#[cfg(test)]
mod control_tests {
    use super::*;
    use crate::talents::{talent_definition, Milestones};
    #[test]
    fn destruction_control_is_scoped_capped_and_resets() {
        for class in 0..9 {
            for rank in [0, 1, 2, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 982, rank), (2, 986, rank)]);
                for key in DESTRUCTION.iter().copied().chain([
                    "corruption",
                    "curse_of_agony",
                    "death_coil",
                    "fireball",
                ]) {
                    for damage in [0., -1., f32::NAN, f32::INFINITY, 10.] {
                        let valid = class == 7 && damage.is_finite() && damage > 0.;
                        assert_eq!(
                            trained.aftermath_chance(key, damage),
                            if valid && DESTRUCTION.contains(&key) {
                                0.02 * f32::from(rank.min(5))
                            } else {
                                0.
                            }
                        );
                        assert_eq!(
                            trained.pyroclasm_chance(key, damage),
                            if valid && matches!(key, "rain_of_fire" | "hellfire") {
                                0.13 * f32::from(rank.min(2))
                            } else {
                                0.
                            }
                        );
                    }
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        for id in [982, 986] {
            let node = talent_definition(7, id).unwrap();
            assert!(node.inline_description(0).unwrap().contains("0 »"));
            assert!(!node
                .inline_description(node.max_rank)
                .unwrap()
                .contains('»'));
        }
    }
}

#[cfg(test)]
mod intensity_tests {
    use super::*;
    use crate::talents::{talent_definition, CastEffect, Milestones};
    #[test]
    fn intensity_extends_only_rain_of_fire_and_refunds_cleanly() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 985, rank)]);
                for key in FIRE.iter().copied().chain(["shadow_bolt", "blizzard"]) {
                    let before = base.cast(key);
                    let after = trained.cast(key);
                    let duration = if class == 7 && key == "rain_of_fire" {
                        1. + 0.1 * f32::from(rank.min(2))
                    } else {
                        1.
                    };
                    assert_eq!(after.duration, before.duration * duration);
                    assert_eq!(
                        CastEffect {
                            duration: before.duration,
                            ..after
                        },
                        before
                    );
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        assert!(talent_definition(7, 985)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .starts_with("Rain of Fire: +10 » 20% active duration"));
    }
}

#[cfg(test)]
mod exhaustion_tests {
    use crate::talents::{talent_definition, Milestones};
    #[test]
    fn exhaustion_requires_unlock_and_is_scoped_capped_and_refundable() {
        for class in 0..9 {
            for unlocked in [false, true] {
                for rank in [0, 1, 4, 255] {
                    let base = Milestones {
                        class,
                        ..Default::default()
                    };
                    let trained =
                        base.with_ranks(&[(0, 1081, u8::from(unlocked)), (0, 1082, rank)]);
                    for boss in [false, true] {
                        let expected = if class == 7 && unlocked {
                            let speed = 0.9 - 0.05 * f32::from(rank.min(4));
                            Some((if boss { speed.max(0.75) } else { speed }, 12.))
                        } else {
                            None
                        };
                        assert_eq!(trained.secondary_slow("curse_of_agony", boss), expected);
                        for key in ["corruption", "death_coil", "immolate", "demoralizing_shout"] {
                            assert_eq!(
                                trained.secondary_slow(key, boss),
                                base.secondary_slow(key, boss)
                            );
                        }
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        for id in [1081, 1082] {
            let node = talent_definition(7, id).unwrap();
            assert!(node.inline_description(0).unwrap().contains("0 »"));
            assert!(!node
                .inline_description(node.max_rank)
                .unwrap()
                .contains('»'));
        }
    }
}
