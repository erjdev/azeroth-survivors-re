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
                assert!((effect.cooldown - (1. - 0.03 * r)).abs() < 0.00001);
                assert_eq!(effect.crit_damage, 0.);
                assert_eq!(
                    (effect.radius, effect.duration, effect.extra_targets),
                    (1., 1., 0)
                );
            }
            let shield = trained.cast("lightning_shield");
            assert!((shield.damage - 1.2 * (1. + 0.05 * short)).abs() < 0.00001);
            assert_eq!(shield.cooldown, 1.);
            assert_eq!(trained.cast("fireball"), CastEffect::default());
            let bonus = stat_adjustment(5, &ranks);
            assert_eq!(bonus.health, 4. * r);
            assert_eq!(bonus.armor, 8. * r);
            assert!((bonus.crit - 0.01 * r).abs() < 0.00001);
            let supported: u32 = DEFINITIONS
                .iter()
                .map(|node| u32::from(rank.min(node.max_rank)))
                .sum();
            assert!((bonus.might + 0.01 * supported as f32).abs() < 0.00001);
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
