//! Audited Druid identities; conversions are explicit until forms and feral attacks exist.
use super::{TalentDefinition, TalentStat};
const BALANCE: &[&str] = &["wrath", "moonfire", "starfire"];
pub(super) const DEFINITIONS: &[TalentDefinition] = &[
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
 ];
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
                    (1. + 0.05 * short) * (1. + 0.02 * r),
                    1. - 0.03 * short,
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
                    (1., 1., 0, 0.)
                );
            }
            for key in ["hurricane", "entangling_roots", "insect_swarm", "fireball"] {
                assert_eq!(trained.cast(key), CastEffect::default());
            }
            let bonus = stat_adjustment(8, &ranks);
            assert_eq!(bonus.armor, 8. * r);
            assert!((bonus.speed - 0.05 * f32::from(rank.min(2))).abs() < 0.00001);
            assert!((bonus.crit - 0.02 * short).abs() < 0.00001);
            assert_eq!(trained.with_ranks(&[]), base);
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
