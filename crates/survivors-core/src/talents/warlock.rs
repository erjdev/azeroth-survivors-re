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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::talents::{stat_adjustment, CastEffect, Milestones};
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
                ("corruption", 1. + 0.02 * r, 1. - 0.04 * r, 0., 0.),
                (
                    "curse_of_agony",
                    (1. + 0.02 * f32::from(rank.min(3))) * (1. + 0.02 * r),
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
                ("death_coil", 1. + 0.02 * r, 1., 0., 0.),
            ] {
                let effect = trained.cast(key);
                assert!((effect.damage - damage).abs() < 0.00001, "{key}");
                assert!((effect.cooldown - cooldown).abs() < 0.00001, "{key}");
                assert!((effect.crit - crit).abs() < 0.00001, "{key}");
                assert_eq!(effect.crit_damage, crit_damage);
                assert_eq!(
                    (effect.radius, effect.duration, effect.extra_targets),
                    (1., 1., 0)
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
