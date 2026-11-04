//! Audited Druid identities with explicit Survivors conversions and permanent form scopes.
use super::{TalentDefinition, TalentStat};
const CASTER_SPELLS: &[&str] = &[
    "wrath",
    "moonfire",
    "starfire",
    "hurricane",
    "insect_swarm",
    "entangling_roots",
    "thorns",
    "rejuvenation",
    "regrowth",
    "healing_touch",
    "tranquility",
];
const HEALING_SPELLS: &[&str] = &["rejuvenation", "regrowth", "healing_touch", "tranquility"];
const FERAL_ATTACKS: &[&str] = &["Claw", "Maul", "rake", "swipe"];
const FEROCITY_ABILITIES: &[&str] = &["Claw", "Maul", "rake", "swipe", "demoralizing_roar"];
const DIRECT_FERAL_ATTACKS: &[&str] = &["Claw", "Maul", "rake", "swipe", "shred", "ferocious_bite"];
const ALL_FERAL_ATTACKS: &[&str] = &[
    "Claw",
    "Maul",
    "rake",
    "swipe",
    "shred",
    "ferocious_bite",
    "bash",
    "pounce",
];
const BALANCE: &[&str] = &["wrath", "moonfire", "starfire"];
pub(super) const DEFINITIONS: &[TalentDefinition] = &[
    TalentDefinition { id: 795, max_rank: 5, name: "Feral Aggression", description: "Demoralizing Roar: +8% reduction strength per rank. Ferocious Bite: +3% damage per rank. Multiplies existing effects; does not grant either ability. Roar requires Bear; Bite requires Cat", stat: TalentStat::FeralAggression, per_rank: 1.0 },
    TalentDefinition { id: 800, max_rank: 2, name: "Blood Frenzy", description: "While Cat is selected: effective Claw, Rake and Shred criticals have 50% chance per rank to recover 0.5 sec of that attack's remaining cooldown. Shared internal cooldown: 1 sec, reduced by haste. Excludes bleeds, pets, splash copies and generated hits; no stored refunds or combo points", stat: TalentStat::BloodFrenzy, per_rank: 0.5 },
    TalentDefinition { id: 801, max_rank: 2, name: "Primal Fury", description: "While Bear is selected: effective Maul and Swipe criticals have 50% chance per rank to recover 0.5 sec of that attack's remaining cooldown. Shared internal cooldown: 1 sec, reduced by haste. Excludes periodic damage, pets, splash copies and generated hits; no stored refunds (Survivors Rage conversion)", stat: TalentStat::PrimalFury, per_rank: 0.5 },
    TalentDefinition { id: 797, max_rank: 2, name: "Brutal Impact", description: "Bash and Pounce: +0.5 sec stun duration per rank, added after ability rank scaling. Does not extend Pounce's bleed or grant either ability; Bash requires Bear and Pounce requires Cat", stat: TalentStat::FeralStunDuration, per_rank: 0.5 },
    TalentDefinition { id: 802, max_rank: 2, name: "Improved Shred", description: "Shred: 10% shorter cooldown per rank, multiplying haste and other recovery bonuses. Does not grant the spell; requires Cat for Shred (Survivors Energy-cost conversion)", stat: TalentStat::SpellCooldown("shred"), per_rank: 0.10 },
    TalentDefinition { id: 788, max_rank: 1, name: "Omen of Clarity", description: "Effective Claw, Maul, Rake, Swipe, Shred and Ferocious Bite hits: 10% chance to fire extra Wrath. Internal cooldown: 5 sec, reduced by haste. Uses owned Wrath rank or rank 1; takes no slot and leaves normal attack timers unchanged. Excludes bleeds, pets, splash copies and generated hits; no stored charges (Survivors Clearcasting conversion)", stat: TalentStat::OmenOfClarity, per_rank: 0.10 },
    TalentDefinition { id: 799, max_rank: 5, name: "Feral Instinct", description: "While Cat or Bear is selected: Claw, Maul, Rake, Swipe, Shred, Ferocious Bite, Bash, Pounce and Demoralizing Roar have 3% shorter cooldown per rank, multiplying other recovery bonuses. No benefit to caster spells or other forms (Survivors threat and stealth conversion)", stat: TalentStat::FeralCooldown, per_rank: 0.03 },
    TalentDefinition { id: 822, max_rank: 5, name: "Furor", description: "While Cat or Bear is selected: Claw, Maul, Rake, Swipe, Shred, Ferocious Bite, Bash, Pounce and Demoralizing Roar have 2% shorter cooldown per rank, multiplying other recovery bonuses. Active throughout the run; no benefit to caster spells or other forms (Survivors Rage/Energy conversion)", stat: TalentStat::FeralCooldown, per_rank: 0.02 },
    TalentDefinition { id: 842, max_rank: 2, name: "Improved Tranquility", description: "Tranquility: 10% shorter cooldown per rank, multiplying other recovery bonuses. Does not grant the spell or change its five healing pulses (Survivors healing-threat conversion)", stat: TalentStat::SpellCooldown("tranquility"), per_rank: 0.10 },
    TalentDefinition { id: 781, max_rank: 3, name: "Natural Shapeshifter", description: "While a permanent Cat, Bear, Moonkin or Tree of Life form is selected: Druid abilities and automatic Claw/Maul have 2% shorter cooldown per rank, multiplying other recovery bonuses. No benefit without a form (Survivors shapeshifting mana-cost conversion)", stat: TalentStat::FormCooldown, per_rank: 0.02 },
    TalentDefinition { id: 829, max_rank: 3, name: "Reflection", description: "Druid caster and healing spells: 2% shorter cooldown per rank, multiplying other recovery bonuses. Excludes Rake, Swipe, Claw and Maul (Survivors mana-regeneration conversion)", stat: TalentStat::SpellGroupCooldown(CASTER_SPELLS), per_rank: 0.02 },
    TalentDefinition { id: 796, max_rank: 5, name: "Ferocity", description: "Claw, Maul, Rake, Swipe and Demoralizing Roar: 4% shorter cooldown per rank, multiplying haste. Does not change other spells or bleed duration (Survivors Energy/Rage-cost conversion)", stat: TalentStat::SpellGroupCooldown(FEROCITY_ABILITIES), per_rank: 0.04 },
    TalentDefinition { id: 805, max_rank: 2, name: "Savage Fury", description: "Claw, Maul, Rake and Swipe: +10% direct and periodic damage per rank. Multiplies other damage bonuses; does not change other spells or healing", stat: TalentStat::SpellGroupDamage(FERAL_ATTACKS), per_rank: 0.10 },
    TalentDefinition { id: 803, max_rank: 3, name: "Predatory Strikes", description: "Cat and Bear: gain attack power equal to 50% of your current run level per rank. Each 14 attack power adds 1 base damage per second to automatic Claw/Maul before damage bonuses; no effect on spells or healing (Survivors attack-power conversion)", stat: TalentStat::PredatoryStrikes, per_rank: 0.5 },
    TalentDefinition { id: 791, max_rank: 5, name: "Natural Weapons", description: "Claw, Maul, Rake, Swipe, Shred, Ferocious Bite, Bash and Pounce: +2% direct and periodic physical damage per rank. Multiplies Savage Fury; does not change other spells, healing or pets", stat: TalentStat::SpellGroupDamage(ALL_FERAL_ATTACKS), per_rank: 0.02 },
    TalentDefinition { id: 1_000_001, max_rank: 1, name: "Cat Form", description: "+20% hero movement speed. Permanently become a Cat for the entire run, gaining automatic Claw outside the six ability slots and starting with Rake alongside Wrath; keeps your spell pool. Excludes Bear, Moonkin and Tree of Life; refund this form before choosing another (Survivors form tuning)", stat: TalentStat::Speed, per_rank: 0.20 },
    TalentDefinition { id: 1_000_002, max_rank: 1, name: "Bear Form", description: "+25% maximum health. Permanently become a Bear for the entire run, gaining automatic Maul outside the six ability slots and starting with Swipe alongside Wrath; keeps your spell pool. Excludes Cat, Moonkin and Tree of Life; refund this form before choosing another (Survivors form tuning)", stat: TalentStat::MaximumHealth, per_rank: 0.25 },
    TalentDefinition { id: 793, max_rank: 1, name: "Moonkin Form", description: "Wrath, Moonfire and Starfire: +5 percentage points critical chance. Permanently become a Moonkin for the entire run. Excludes Cat, Bear and Tree of Life; refund this form before choosing another (Survivors form tuning; adds to the Balance milestone)", stat: TalentStat::SpellGroupCrit(BALANCE), per_rank: 0.05 },
    TalentDefinition { id: 1_000_003, max_rank: 1, name: "Tree of Life", description: "Rejuvenation, Regrowth, Healing Touch and Tranquility: +20% healing. Permanently become a Tree of Life for the entire run. Excludes Cat, Bear and Moonkin; refund this form before choosing another (Survivors form tuning)", stat: TalentStat::SpellGroupHealing(HEALING_SPELLS), per_rank: 0.20 },
    TalentDefinition { id: 825, max_rank: 5, name: "Improved Regrowth", description: "Regrowth: +10 percentage points direct-healing critical chance per rank. Critical heals restore 150%; periodic healing cannot crit. Does not grant the spell", stat: TalentStat::SpellCrit("regrowth"), per_rank: 0.10 },
    TalentDefinition { id: 789, max_rank: 1, name: "Nature's Grace", description: "Damaging Druid spell criticals grant one refreshing 15 sec charge: the next successful Wrath or Starfire has 0.5 sec shorter recovery, after other cooldown modifiers. Failed casts and instant spells do not consume it (Survivors cast-time conversion)", stat: TalentStat::NaturesGrace, per_rank: 0.5 },
    TalentDefinition { id: 827, max_rank: 1, name: "Insect Swarm", description: "Insect Swarm applies +2% reduced outgoing damage to affected enemies for 10 sec, including bosses. Stronger reductions take priority. Damage, cooldown and targeting are unchanged; does not grant the spell (Survivors hit-chance conversion)", stat: TalentStat::InsectSwarmWeakening, per_rank: 0.02 },
    TalentDefinition { id: 808, max_rank: 5, name: "Heart of the Wild", description: "+4% maximum health per rank, including equipped health and multiplying passive, renown and racial health bonuses. Does not heal current health (Survivors stamina conversion; no form required)", stat: TalentStat::MaximumHealth, per_rank: 0.04 },
    TalentDefinition { id: 809, max_rank: 1, name: "Leader of the Pack", description: "Cat and Bear Claw, Maul, Rake, Swipe, Shred and Ferocious Bite: +3 percentage points critical chance, adding to Sharpened Claws and equipment. Bleeds cannot critically strike; no effect on other spells, healing, pets or other forms (Survivors self-only aura conversion)", stat: TalentStat::FeralAttackCrit, per_rank: 0.03 },
    TalentDefinition { id: 761, max_rank: 1, name: "Nature's Grasp", description: "Entangling Roots: 15% shorter cooldown, multiplying Improved Entangling Roots and the 10-point Balance milestone (Survivors automatic defensive-root conversion)", stat: TalentStat::SpellCooldown("entangling_roots"), per_rank: 0.15 },
    TalentDefinition { id: 921, max_rank: 4, name: "Improved Nature's Grasp", description: "Entangling Roots: +10% root duration per rank; bosses retain 30% duration (Survivors automatic-root reliability conversion)", stat: TalentStat::SpellDuration("entangling_roots"), per_rank: 0.10 },
    TalentDefinition { id: 787, max_rank: 3, name: "Improved Entangling Roots", description: "Entangling Roots: 3% shorter cooldown per rank, multiplying Nature's Grasp and the 10-point Balance milestone (Survivors cast reliability conversion; no spell pushback)", stat: TalentStat::SpellCooldown("entangling_roots"), per_rank: 0.03 },
    TalentDefinition { id: 831, max_rank: 1, name: "Nature's Swiftness", description: "Rejuvenation and Healing Touch: 15% shorter cooldown, multiplying individual cooldown ranks (Survivors automatic instant-cast conversion)", stat: TalentStat::SpellGroupCooldown(&["rejuvenation", "healing_touch"]), per_rank: 0.15 },
    TalentDefinition { id: 844, max_rank: 1, name: "Swiftmend", description: "Rejuvenation immediately heals 5% maximum health once per successful cast; adds to the 30-point Restoration milestone for 10% total before healing bonuses. Does not consume healing over time (Survivors automatic conversion)", stat: TalentStat::SpellHeal("rejuvenation"), per_rank: 0.05 },
    TalentDefinition { id: 764, max_rank: 2, name: "Nature's Reach", description: "Wrath, Moonfire, Starfire, Entangling Roots and Hurricane: +10% targeting range per rank. Hurricane's radius and target counts are unchanged; excludes Insect Swarm, Thorns and healing", stat: TalentStat::SpellGroupRange(&["wrath", "moonfire", "starfire", "entangling_roots", "hurricane"]), per_rank: 0.10 },
    TalentDefinition { id: 762, max_rank: 5, name: "Improved Wrath", description: "Wrath: 4% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellCooldown("wrath"), per_rank: 0.04 },
    TalentDefinition { id: 763, max_rank: 5, name: "Improved Moonfire", description: "Moonfire: +2% direct and periodic damage and +2 percentage points critical chance per rank", stat: TalentStat::SpellDamageAndCrit("moonfire"), per_rank: 0.02 },
    TalentDefinition { id: 782, max_rank: 3, name: "Improved Thorns", description: "Thorns: +10% retaliation damage per rank", stat: TalentStat::SpellDamage("thorns"), per_rank: 0.1 },
    TalentDefinition { id: 792, max_rank: 5, name: "Vengeance", description: "Wrath, Moonfire and Starfire: +10 percentage points critical damage per rank", stat: TalentStat::SpellGroupCritDamage(BALANCE), per_rank: 0.1 },
    TalentDefinition { id: 784, max_rank: 5, name: "Improved Starfire", description: "Starfire: 3% shorter cooldown per rank (Survivors cast-time conversion)", stat: TalentStat::SpellCooldown("starfire"), per_rank: 0.03 },
    TalentDefinition { id: 783, max_rank: 3, name: "Moonglow", description: "Wrath, Moonfire, Starfire, Rejuvenation, Regrowth, Healing Touch and Tranquility: 3% shorter cooldown per rank (Survivors mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(&["wrath", "moonfire", "starfire", "rejuvenation", "regrowth", "healing_touch"]), per_rank: 0.03 },
    TalentDefinition { id: 790, max_rank: 5, name: "Moonfury", description: "Wrath, Moonfire and Starfire: +2% direct and periodic damage per rank", stat: TalentStat::SpellGroupDamage(BALANCE), per_rank: 0.02 },
    TalentDefinition { id: 794, max_rank: 5, name: "Thick Hide", description: "+8 hero armor per rank (Survivors conversion)", stat: TalentStat::Armor, per_rank: 8.0 },
    TalentDefinition { id: 807, max_rank: 2, name: "Feline Swiftness", description: "Cat Form only: +15% outdoor movement speed and +2 percentage points melee dodge chance per rank. Dodge works indoors too; movement multiplies other speed bonuses", stat: TalentStat::FelineSwiftness, per_rank: 0.15 },
    TalentDefinition { id: 798, max_rank: 3, name: "Sharpened Claws", description: "Cat and Bear Claw, Maul, Rake, Swipe, Shred and Ferocious Bite: +2 percentage points critical chance per rank. Bleeds cannot critically strike; no effect on other spells, healing, pets or other forms", stat: TalentStat::FeralAttackCrit, per_rank: 0.02 },
    TalentDefinition { id: 830, max_rank: 3, name: "Improved Rejuvenation", description: "Rejuvenation: +5% healing per rank", stat: TalentStat::SpellHealing("rejuvenation"), per_rank: 0.05 },
    TalentDefinition { id: 828, max_rank: 5, name: "Gift of Nature", description: "Rejuvenation, Regrowth, Healing Touch and Tranquility: +2% healing per rank, including direct and periodic healing. Does not change critical chance, duration or damage", stat: TalentStat::SpellGroupHealing(HEALING_SPELLS), per_rank: 0.02 },
    TalentDefinition { id: 821, max_rank: 5, name: "Improved Mark of the Wild", description: "+4 hero armor per rank (Survivors permanent Mark of the Wild conversion)", stat: TalentStat::Armor, per_rank: 4.0 },
    TalentDefinition { id: 824, max_rank: 5, name: "Improved Healing Touch", description: "Rejuvenation and Healing Touch: 3% shorter cooldown per rank (Survivors Healing Touch cast-time conversion)", stat: TalentStat::SpellGroupCooldown(&["rejuvenation", "healing_touch"]), per_rank: 0.03 },
    TalentDefinition { id: 823, max_rank: 5, name: "Nature's Focus", description: "Rejuvenation, Regrowth, Healing Touch and Tranquility: +2% direct and periodic healing per rank, multiplying Gift of Nature. Does not change critical chance or recovery (Survivors healing reliability conversion; no spell pushback)", stat: TalentStat::SpellGroupHealing(HEALING_SPELLS), per_rank: 0.02 },
    TalentDefinition { id: 841, max_rank: 5, name: "Subtlety", description: "Rejuvenation, Regrowth, Healing Touch and Tranquility: 2% shorter cooldown per rank, multiplying other recovery bonuses. Does not reduce incoming damage (Survivors healing-threat conversion)", stat: TalentStat::SpellGroupCooldown(HEALING_SPELLS), per_rank: 0.02 },
    TalentDefinition { id: 843, max_rank: 5, name: "Tranquil Spirit", description: "Rejuvenation, Healing Touch and Tranquility: 2% shorter cooldown per rank (Survivors healing mana-cost conversion)", stat: TalentStat::SpellGroupCooldown(&["rejuvenation", "healing_touch", "tranquility"]), per_rank: 0.02 },
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
        (1, 10) => Some("Thorns: 15% shorter cooldown (independent Feral milestone)"),
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
            let ranks: Vec<_> = DEFINITIONS
                .iter()
                .filter(|n| super::super::forms::DruidForm::from_talent(n.id).is_none())
                .map(|n| (0, n.id, rank))
                .collect();
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
                let cooldown = cooldown
                    * (1. - 0.02 * short)
                    * if key == "rejuvenation" {
                        1. - 0.02 * r
                    } else {
                        1.
                    };
                let e = trained.cast(key);
                assert!((e.damage - damage).abs() < 0.00001, "{key}");
                assert!((e.cooldown - cooldown).abs() < 0.00001, "{key}");
                assert!((e.crit - crit).abs() < 0.00001, "{key}");
                assert!((e.crit_damage - crit_damage).abs() < 0.00001, "{key}");
                assert_eq!((e.radius, e.duration, e.extra_targets), (1., 1., 0));
                let expected_heal = if key == "rejuvenation" && rank > 0 {
                    0.05 * damage
                } else {
                    0.
                };
                assert!((e.heal_fraction - expected_heal).abs() < 0.00001);
            }
            for key in ["hurricane", "entangling_roots", "insect_swarm", "fireball"] {
                let range = if matches!(key, "hurricane" | "entangling_roots") {
                    1. + 0.10 * f32::from(rank.min(2))
                } else {
                    1.
                };
                let mut expected = CastEffect {
                    range,
                    cooldown: (if key == "entangling_roots" {
                        (1. - 0.15 * f32::from(rank.min(1))) * (1. - 0.03 * short)
                    } else {
                        1.
                    }) * if key != "fireball" {
                        1. - 0.02 * short
                    } else {
                        1.
                    },
                    duration: if key == "entangling_roots" {
                        1. + 0.10 * f32::from(rank.min(4))
                    } else {
                        1.
                    },
                    ..Default::default()
                };
                let actual = trained.cast(key);
                assert!((actual.cooldown - expected.cooldown).abs() < 0.00001);
                expected.cooldown = actual.cooldown;
                assert_eq!(actual, expected);
            }
            let bonus = stat_adjustment(8, &ranks);
            assert_eq!(bonus.armor, 12. * r);
            assert_eq!(bonus.reduction, 0.0);
            assert_eq!(
                bonus.speed, 0.,
                "Feline Swiftness requires Cat and outdoor context"
            );
            assert_eq!(bonus.crit, 0.0);
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
                assert!((milestones.heal_fraction - 0.05 * healing).abs() < 0.00001);
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
            for key in BALANCE
                .iter()
                .chain(["thorns", "rejuvenation", "entangling_roots"].iter())
            {
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
        assert!((all.cast("rejuvenation").heal_fraction - 0.05 * 1.15 * 1.1).abs() < 0.00001);
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

#[cfg(test)]
mod healing_composition_tests {
    use crate::talents::Milestones;

    #[test]
    fn healing_training_scales_both_portions_without_unlocking_or_changing_other_casts() {
        for points in [0, 29, 30] {
            for swiftmend in [0, 1] {
                let base = Milestones {
                    class: 8,
                    points: [0, 0, points],
                    ..Default::default()
                }
                .with_ranks(&[(2, 844, swiftmend)]);
                for rank in [0, 1, 5, 255] {
                    let mut ranks = [
                        (2, 844, swiftmend),
                        (2, 830, rank),
                        (2, 828, rank),
                        (2, 823, rank),
                    ];
                    let trained = base.with_ranks(&ranks);
                    ranks.reverse();
                    assert_eq!(
                        trained.cast("rejuvenation"),
                        base.with_ranks(&ranks).cast("rejuvenation")
                    );
                    let scale = (1. + 0.05 * f32::from(rank.min(3)))
                        * (1. + 0.02 * f32::from(rank.min(5))).powi(2);
                    let before = base.cast("rejuvenation");
                    let actual = trained.cast("rejuvenation");
                    assert!((actual.damage - before.damage * scale).abs() < 0.00001);
                    assert!((actual.heal_fraction - before.heal_fraction * scale).abs() < 0.00001);
                    assert_eq!(actual.cooldown, before.cooldown);
                    for key in ["thorns", "moonfire", "wrath", "holy_nova"] {
                        assert_eq!(trained.cast(key), base.cast(key));
                    }
                    assert_eq!(trained.with_ranks(&[(2, 844, swiftmend)]), base);
                }
            }
        }
    }
}

#[cfg(test)]
mod root_training_tests {
    use crate::talents::{talent_definition, Milestones};

    #[test]
    fn roots_compose_at_milestone_boundaries_and_reset() {
        for points in [0, 9, 10, 30] {
            let base = Milestones {
                class: 8,
                points: [points, 0, 0],
                ..Default::default()
            };
            for rank in [0, 1, 3, 4, 255] {
                let ranks = [(0, 761, rank), (0, 921, rank), (0, 787, rank)];
                let trained = base.with_ranks(&ranks);
                let before = base.cast("entangling_roots");
                let after = trained.cast("entangling_roots");
                let scale =
                    (1. - 0.15 * f32::from(rank.min(1))) * (1. - 0.03 * f32::from(rank.min(3)));
                assert!((after.cooldown - before.cooldown * scale).abs() < 0.00001);
                assert!((after.duration - (1. + 0.1 * f32::from(rank.min(4)))).abs() < 0.00001);
                assert_eq!(
                    (after.damage, after.radius, after.extra_targets),
                    (before.damage, before.radius, before.extra_targets)
                );
                assert_eq!(trained.with_ranks(&[]), base);
                for key in ["wrath", "moonfire", "hurricane", "thorns", "frost_nova"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
        let duration = talent_definition(8, 921).unwrap();
        assert_eq!(duration.inline_description(1).unwrap(), "Entangling Roots: +10 » 20% root duration; bosses retain 30% duration (Survivors automatic-root reliability conversion)");
    }
}

#[cfg(test)]
mod feral_crit_tests {
    use crate::talents::{forms::DruidForm, stat_adjustment, talent_definition, Milestones};

    #[test]
    fn feral_crit_stacks_caps_refunds_and_never_leaks_to_spells_or_other_forms() {
        for form in [
            None,
            Some(DruidForm::Cat),
            Some(DruidForm::Bear),
            Some(DruidForm::Moonkin),
            Some(DruidForm::TreeOfLife),
        ] {
            for claws in [0, 1, 3, 255] {
                for leader in [0, 1, 255] {
                    let mut ranks = vec![(1, 798, claws), (1, 809, leader)];
                    if let Some(form) = form {
                        ranks.push((form.tree(), form.talent(), 1));
                    }
                    let base = Milestones {
                        class: 8,
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&ranks);
                    let expected = if matches!(form, Some(DruidForm::Cat | DruidForm::Bear)) {
                        0.02 * f32::from(claws.min(3)) + 0.03 * f32::from(leader.min(1))
                    } else {
                        0.
                    };
                    for key in ["Claw", "Maul", "rake", "swipe"] {
                        assert!((trained.cast(key).crit - expected).abs() < 0.00001);
                    }
                    assert_eq!(stat_adjustment(8, &ranks).crit, 0.);
                    let form_only: Vec<_> = ranks
                        .iter()
                        .copied()
                        .filter(|(_, id, _)| *id != 798 && *id != 809)
                        .collect();
                    for key in [
                        "wrath",
                        "moonfire",
                        "starfire",
                        "regrowth",
                        "rejuvenation",
                        "thorns",
                        "Weapon Attack",
                    ] {
                        assert_eq!(
                            trained.cast(key),
                            base.with_ranks(&form_only).cast(key),
                            "{key}"
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                    for class in 0..8 {
                        assert_eq!(
                            Milestones {
                                class,
                                ..Default::default()
                            }
                            .with_ranks(&ranks)
                            .cast("Claw")
                            .crit,
                            0.
                        );
                    }
                }
            }
        }
        for (id, rank, delta) in [
            (798, 2, "4 » 6 percentage points"),
            (809, 0, "0 » 3 percentage points"),
        ] {
            let (text, spans) = talent_definition(8, id)
                .unwrap()
                .inline_description_with_values(rank)
                .unwrap();
            assert!(text.contains(delta), "{text}");
            assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
            assert!(text.contains("Cat and Bear Claw, Maul, Rake, Swipe, Shred and Ferocious Bite"));
        }
    }
}

#[cfg(test)]
mod heart_tests {
    use crate::talents::{stat_adjustment, talent_definition, Milestones};
    #[test]
    fn heart_scales_health_only_with_bounded_ranks() {
        for rank in [0, 1, 3, 5, 255] {
            let ranks = [(1, 808, rank)];
            for class in 0..9 {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&ranks);
                let r = if class == 8 {
                    f32::from(rank.min(5))
                } else {
                    0.0
                };
                assert!((trained.health_multiplier() - 1.0 - 0.04 * r).abs() < 0.00001);
                assert!((stat_adjustment(class, &ranks).might + 0.01 * r).abs() < 0.00001);
                assert_eq!(trained.with_ranks(&[]).health_multiplier(), 1.0);
                for key in ["wrath", "rejuvenation", "thorns", "moonfire"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
        assert!(talent_definition(8, 808)
            .unwrap()
            .inline_description(0)
            .unwrap()
            .starts_with("+0 » 4% maximum health"));
    }
}

#[cfg(test)]
mod swarm_tests {
    use crate::{
        data::Effect,
        talents::{talent_definition, Milestones},
    };
    #[test]
    fn weakening_is_scoped_capped_and_does_not_change_damage_or_other_control() {
        for class in 0..9 {
            for rank in [0, 1, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 827, rank)]);
                for key in ["insect_swarm", "moonfire", "thorns", "scorpid_sting"] {
                    let active = class == 8 && rank > 0 && key == "insect_swarm";
                    match trained.control(key, Effect::None) {
                        Effect::Weaken(m, t) if active => {
                            assert_eq!(m, 0.98);
                            assert_eq!(t, 10.);
                        }
                        Effect::None if !active => {}
                        _ => panic!("unexpected control for class {class}, rank {rank}, {key}"),
                    }
                    assert!(matches!(
                        trained.with_ranks(&[]).control(key, Effect::None),
                        Effect::None
                    ));
                    assert!(matches!(
                        trained.control(key, Effect::Weaken(0.5, 3.)),
                        Effect::Weaken(0.5, 3.)
                    ));
                    assert_eq!(trained.cast(key), base.cast(key));
                }
            }
        }
        let node = talent_definition(8, 827).unwrap();
        let (text, ranges) = node.inline_description_with_values(0).unwrap();
        assert_eq!(&text[ranges[0].clone()], "+0 » 2% reduced outgoing damage");
        assert!(text.contains("for 10 sec"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod regrowth_tests {
    use crate::talents::{talent_definition, Milestones};

    #[test]
    fn regrowth_training_caps_scopes_and_resets_without_affecting_other_heals() {
        for class in 0..9 {
            for rank in [0, 1, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 825, rank)]);
                let mut expected = base.cast("regrowth");
                if class == 8 {
                    expected.crit += 0.1 * f32::from(rank.min(5));
                }
                assert_eq!(trained.cast("regrowth"), expected);
                assert_eq!(trained.cast("rejuvenation"), base.cast("rejuvenation"));
                assert_eq!(trained.cast("wrath"), base.cast("wrath"));
                assert_eq!(
                    trained.with_ranks(&[]).cast("regrowth"),
                    base.cast("regrowth")
                );
            }
        }
        let (text, values) = talent_definition(8, 825)
            .unwrap()
            .inline_description_with_values(1)
            .unwrap();
        assert!(text.contains("10 » 20"), "{text}");
        assert!(!values.is_empty());
        assert!(text.contains("150%"));
        assert!(text.contains("periodic") || text.contains("Periodic"));
    }
}

#[cfg(test)]
mod healing_group_tests {
    use crate::talents::{talent_definition, CastEffect, Milestones};

    #[test]
    fn gift_and_moonglow_cover_both_heals_without_granting_unrelated_bonuses() {
        for rank in [0, 1, 3, 5, 255] {
            let base = Milestones {
                class: 8,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(2, 828, rank), (0, 783, rank)]);
            let healing = 1. + 0.02 * f32::from(rank.min(5));
            let cooldown = 1. - 0.03 * f32::from(rank.min(3));
            for key in ["rejuvenation", "regrowth"] {
                assert_eq!(
                    trained.cast(key),
                    CastEffect {
                        damage: healing,
                        cooldown,
                        ..Default::default()
                    }
                );
            }
            for key in ["wrath", "moonfire", "starfire"] {
                assert_eq!(
                    trained.cast(key),
                    CastEffect {
                        cooldown,
                        ..Default::default()
                    }
                );
            }
            for key in ["thorns", "insect_swarm", "hurricane", "renew", "holy_light"] {
                assert_eq!(trained.cast(key), base.cast(key), "{key}");
            }
            assert_eq!(trained.with_ranks(&[]), base);
            let other = Milestones {
                class: 4,
                ..Default::default()
            };
            assert_eq!(
                other
                    .with_ranks(&[(2, 828, rank), (0, 783, rank)])
                    .cast("regrowth"),
                other.cast("regrowth")
            );
        }
        let (text, spans) = talent_definition(8, 828)
            .unwrap()
            .inline_description_with_values(4)
            .unwrap();
        assert!(text.contains("8 » 10% healing"), "{text}");
        assert!(!spans.is_empty());
        assert!(text.contains("Regrowth"));
    }
}

#[cfg(test)]
mod focus_tests {
    use crate::talents::{talent_definition, CastEffect, Milestones};

    #[test]
    fn focus_scopes_both_heals_stacks_with_gift_and_refunds_cleanly() {
        for class in 0..9 {
            for rank in [0, 1, 3, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 823, rank)]);
                let factor = if class == 8 {
                    1. + 0.02 * f32::from(rank.min(5))
                } else {
                    1.
                };
                for key in ["rejuvenation", "regrowth"] {
                    assert_eq!(
                        trained.cast(key),
                        CastEffect {
                            damage: factor,
                            ..Default::default()
                        }
                    );
                    let combined = base.with_ranks(&[(2, 823, rank), (2, 828, rank)]);
                    assert!((combined.cast(key).damage - factor * factor).abs() < 0.00001);
                }
                for key in ["wrath", "thorns", "renew", "holy_light"] {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        let (text, spans) = talent_definition(8, 823)
            .unwrap()
            .inline_description_with_values(4)
            .unwrap();
        assert!(text.contains("Regrowth"));
        assert!(
            text.contains("8 » 10% direct and periodic healing"),
            "{text}"
        );
        assert!(!spans.is_empty());
    }
}

#[cfg(test)]
mod feral_training_tests {
    use crate::{
        data,
        talents::{talent_definition, Milestones},
    };

    #[test]
    fn ferocity_and_fury_scope_clamp_and_refund_without_changing_other_casts() {
        for class in 0..9 {
            for rank in [0, 1, 2, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(1, 796, rank), (1, 805, rank)]);
                for key in
                    data::ABILITIES
                        .iter()
                        .map(|d| d.key)
                        .chain(["Claw", "Maul", "Weapon Attack"])
                {
                    let mut expected = base.cast(key);
                    if class == 8 && ["Claw", "Maul", "rake", "swipe"].contains(&key) {
                        expected.cooldown *= 1.0 - 0.04 * f32::from(rank.min(5));
                        expected.damage *= 1.0 + 0.10 * f32::from(rank.min(2));
                    }
                    if class == 8 && key == "demoralizing_roar" {
                        expected.cooldown *= 1.0 - 0.04 * f32::from(rank.min(5));
                    }
                    assert_eq!(
                        trained.cast(key),
                        expected,
                        "class={class} rank={rank} key={key}"
                    );
                    assert_eq!(trained.with_ranks(&[]).cast(key), base.cast(key));
                }
            }
        }
        for (id, delta) in [(796, "4 » 8%"), (805, "10 » 20%")] {
            let (text, spans) = talent_definition(8, id)
                .unwrap()
                .inline_description_with_values(1)
                .unwrap();
            assert!(text.contains(delta), "{text}");
            assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
        }
    }
}

#[cfg(test)]
mod natural_weapons_scope_tests {
    use crate::{
        data,
        talents::{talent_definition, Milestones},
    };

    #[test]
    fn natural_weapons_is_physical_only_and_multiplies_savage_fury() {
        for class in 0..9 {
            for rank in [0, 1, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, 791, rank)]);
                for key in
                    data::ABILITIES
                        .iter()
                        .map(|d| d.key)
                        .chain(["Claw", "Maul", "Weapon Attack"])
                {
                    let mut expected = base.cast(key);
                    if class == 8
                        && [
                            "Claw",
                            "Maul",
                            "rake",
                            "swipe",
                            "shred",
                            "ferocious_bite",
                            "bash",
                            "pounce",
                        ]
                        .contains(&key)
                    {
                        expected.damage *= 1.0 + 0.02 * f32::from(rank.min(5));
                    }
                    assert_eq!(trained.cast(key), expected, "{class}/{rank}/{key}");
                    assert_eq!(trained.with_ranks(&[]).cast(key), base.cast(key));
                }
            }
        }
        let trained = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(0, 791, 5), (1, 805, 2)]);
        for key in ["Claw", "Maul", "rake", "swipe"] {
            assert!((trained.cast(key).damage - 1.32).abs() < 0.00001);
        }
        let (text, spans) = talent_definition(8, 791)
            .unwrap()
            .inline_description_with_values(4)
            .unwrap();
        assert!(text.contains("8 » 10% direct and periodic physical damage"));
        assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
    }
}

#[cfg(test)]
mod restoration_recovery_tests {
    use crate::{
        data,
        talents::{stat_adjustment, talent_definition, Milestones},
    };

    #[test]
    fn recovery_conversions_are_scoped_capped_and_refundable_without_defense() {
        for class in 0..9 {
            for rank in [0, 1, 3, 5, 255] {
                let ranks = [(2, 841, rank), (2, 829, rank)];
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&ranks);
                for key in
                    data::ABILITIES
                        .iter()
                        .map(|d| d.key)
                        .chain(["Claw", "Maul", "Weapon Attack"])
                {
                    let mut expected = base.cast(key);
                    if class == 8 {
                        if super::CASTER_SPELLS.contains(&key) {
                            expected.cooldown *= 1.0 - 0.02 * f32::from(rank.min(3));
                        }
                        if super::HEALING_SPELLS.contains(&key) {
                            expected.cooldown *= 1.0 - 0.02 * f32::from(rank.min(5));
                        }
                    }
                    assert_eq!(trained.cast(key), expected, "{class}/{rank}/{key}");
                    assert_eq!(trained.with_ranks(&[]).cast(key), base.cast(key));
                }
                assert_eq!(stat_adjustment(class, &ranks).reduction, 0.0);
            }
        }
        let base = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(0, 783, 3), (2, 824, 5), (2, 843, 5)]);
        let trained = base.with_ranks(&[
            (0, 783, 3),
            (2, 824, 5),
            (2, 843, 5),
            (2, 841, 5),
            (2, 829, 3),
        ]);
        for key in super::HEALING_SPELLS {
            assert!(
                (trained.cast(key).cooldown - base.cast(key).cooldown * 0.9 * 0.94).abs() < 0.00001
            );
        }
        for id in [841, 829] {
            let (text, spans) = talent_definition(8, id)
                .unwrap()
                .inline_description_with_values(1)
                .unwrap();
            assert!(text.contains("2 » 4% shorter cooldown"));
            assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
        }
    }
}

#[cfg(test)]
mod tranquility_tests {
    use crate::{
        data,
        talents::{forms::DruidForm, talent_definition, Milestones},
    };
    #[test]
    fn tranquility_has_scoped_healing_recovery_and_no_critical_or_target_growth() {
        assert!(data::CLASSES[8].pool.contains(&"tranquility"));
        for class in 0..9 {
            for rank in [0, 1, 2, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 842, rank)]);
                for ability in data::ABILITIES {
                    let mut expected = base.cast(ability.key);
                    if class == 8 && ability.key == "tranquility" {
                        expected.cooldown *= 1. - 0.1 * f32::from(rank.min(2));
                    }
                    assert_eq!(trained.cast(ability.key), expected);
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        let form = DruidForm::TreeOfLife;
        let effect = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[
            (form.tree(), form.talent(), 1),
            (2, 828, 5),
            (2, 823, 5),
            (2, 841, 5),
            (2, 829, 3),
            (2, 843, 5),
            (2, 842, 2),
            (0, 781, 3),
        ])
        .cast("tranquility");
        assert!((effect.damage - 1.2 * 1.1 * 1.1).abs() < 0.00001);
        assert!((effect.cooldown - 0.9 * 0.94 * 0.9 * 0.8 * 0.94).abs() < 0.00001);
        assert_eq!(effect.crit, 0.);
        assert_eq!(effect.extra_targets, 0);
        let (text, spans) = talent_definition(8, 842)
            .unwrap()
            .inline_description_with_values(1)
            .unwrap();
        assert!(text.contains("10 » 20% shorter cooldown"));
        assert!(spans.iter().any(|s| text[s.clone()].contains('»')));
    }
}

#[cfg(test)]
mod feral_recovery_tests {
    use crate::{
        data,
        talents::{forms::DruidForm, talent_definition, Milestones},
    };

    #[test]
    fn feral_recovery_is_scoped_clamped_refundable_and_multiplicative() {
        for class in 0..9 {
            for form in [
                None,
                Some(DruidForm::Cat),
                Some(DruidForm::Bear),
                Some(DruidForm::Moonkin),
                Some(DruidForm::TreeOfLife),
            ] {
                let form_ranks: Vec<_> = form
                    .into_iter()
                    .map(|f| (f.tree(), f.talent(), 1))
                    .collect();
                let base = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&form_ranks);
                for rank in [0, 1, 3, 5, 255] {
                    let mut ranks = form_ranks.clone();
                    ranks.extend([(1, 799, rank), (2, 822, rank)]);
                    let trained = base.with_ranks(&ranks);
                    for key in data::ABILITIES
                        .iter()
                        .map(|a| a.key)
                        .chain(["Claw", "Maul", "unknown"])
                    {
                        let mut expected = base.cast(key);
                        if class == 8
                            && matches!(form, Some(DruidForm::Cat | DruidForm::Bear))
                            && matches!(
                                key,
                                "Claw"
                                    | "Maul"
                                    | "rake"
                                    | "swipe"
                                    | "shred"
                                    | "ferocious_bite"
                                    | "bash"
                                    | "pounce"
                                    | "demoralizing_roar"
                            )
                        {
                            let r = f32::from(rank.min(5));
                            expected.cooldown *= (1. - 0.03 * r) * (1. - 0.02 * r);
                        }
                        assert!((trained.cast(key).cooldown - expected.cooldown).abs() < 0.00001);
                        expected.cooldown = trained.cast(key).cooldown;
                        assert_eq!(trained.cast(key), expected);
                    }
                    assert_eq!(trained.with_ranks(&form_ranks), base);
                }
            }
        }
        let combined = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[
            (1, 1_000_001, 1),
            (1, 799, 5),
            (2, 822, 5),
            (1, 796, 5),
            (0, 781, 3),
        ]);
        assert!((combined.cast("Claw").cooldown - 0.85 * 0.9 * 0.8 * 0.94).abs() < 0.00001);
        for (id, text) in [(799, "3 » 6%"), (822, "2 » 4%")] {
            assert!(talent_definition(8, id)
                .unwrap()
                .inline_description(1)
                .unwrap()
                .contains(text));
        }
    }
}

/// Resource-free Omen: one extra Wrath, with an independent haste-scaled cooldown.
#[derive(Clone, Copy, Debug, Default)]
pub struct OmenClock {
    next: f32,
    pending: bool,
}

impl OmenClock {
    pub fn chance(self, talents: super::Milestones, key: &str, damage: f32, now: f32) -> f32 {
        if talents.class != 8
            || !now.is_finite()
            || now < 0.
            || now < self.next
            || self.pending
            || !damage.is_finite()
            || damage <= 0.
            || !DIRECT_FERAL_ATTACKS.contains(&key)
        {
            return 0.;
        }
        talents.trained_value(TalentStat::OmenOfClarity)
    }

    pub fn trigger(
        &mut self,
        talents: super::Milestones,
        key: &str,
        damage: f32,
        now: f32,
        haste: f32,
        roll: f32,
    ) -> bool {
        let chance = self.chance(talents, key, damage, now);
        if chance <= 0.
            || !haste.is_finite()
            || !roll.is_finite()
            || !(0.0..1.0).contains(&roll)
            || roll >= chance
        {
            return false;
        }
        self.next = now + 5. / (1. + haste).max(0.1);
        self.pending = true;
        true
    }

    /// Dispatch once at the next active cast update; target loss never banks a charge.
    pub fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod omen_tests {
    use super::*;
    use crate::talents::Milestones;

    #[test]
    fn omen_is_scoped_and_has_a_haste_scaled_nonbanking_clock() {
        for class in 0..9 {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 788, 255)]);
            for key in ["Claw", "Maul", "rake", "swipe", "wrath", "unknown"] {
                assert_eq!(
                    OmenClock::default().chance(talents, key, 1., 0.),
                    if class == 8 && DIRECT_FERAL_ATTACKS.contains(&key) {
                        0.1
                    } else {
                        0.
                    }
                );
            }
        }
        let talents = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(0, 788, 1)]);
        let mut clock = OmenClock::default();
        assert!(!clock.trigger(talents, "Claw", 1., 0., 1., 0.1));
        assert!(clock.trigger(talents, "Claw", 1., 0., 1., 0.));
        assert_eq!(clock.chance(talents, "Claw", 1., 100.), 0.);
        assert!(clock.take());
        assert!(!clock.take());
        assert_eq!(clock.chance(talents, "Claw", 1., 2.49), 0.);
        assert_eq!(clock.chance(talents, "Claw", 1., 2.5), 0.1);
        for (damage, now, haste, roll) in [
            (0., 3., 0., 0.),
            (f32::NAN, 3., 0., 0.),
            (1., f32::NAN, 0., 0.),
            (1., 3., f32::NAN, 0.),
            (1., 3., 0., f32::NAN),
        ] {
            assert!(!clock.trigger(talents, "Claw", damage, now, haste, roll));
        }
        assert_eq!(clock.chance(talents.with_ranks(&[]), "Claw", 1., 3.), 0.);
    }
}

#[cfg(test)]
mod shred_tests {
    use crate::{
        data,
        talents::{forms::DruidForm, talent_definition, Milestones},
    };
    #[test]
    fn shred_is_acquired_only_in_cat_and_its_recovery_talent_is_scoped() {
        assert!(data::CLASSES[8].pool.contains(&"shred"));
        for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
            let ranks: Vec<_> = form
                .into_iter()
                .map(|f| (f.tree(), f.talent(), 1))
                .collect();
            let base = Milestones {
                class: 8,
                ..Default::default()
            }
            .with_ranks(&ranks);
            assert_eq!(base.ability_unlocked("shred"), form == Some(DruidForm::Cat));
            assert!(!base.starting_abilities("wrath").contains(&"shred"));
            for rank in [0, 1, 2, 255] {
                let mut ranks = ranks.clone();
                ranks.push((1, 802, rank));
                let trained = base.with_ranks(&ranks);
                for ability in data::ABILITIES {
                    let mut expected = base.cast(ability.key);
                    if ability.key == "shred" {
                        expected.cooldown *= 1. - 0.1 * f32::from(rank.min(2));
                    }
                    assert_eq!(trained.cast(ability.key), expected);
                }
            }
        }
        assert!(talent_definition(8, 802)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .contains("10 » 20%"));
    }
}

#[cfg(test)]
mod feral_control_tests {
    use crate::{
        data::{self, Effect},
        talents::{forms::DruidForm, talent_definition, Milestones},
    };

    #[test]
    fn feral_control_is_acquired_form_scoped_and_adds_only_stun_time() {
        for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
            for rank in [0, 1, 2, 255] {
                let mut ranks: Vec<_> = form
                    .into_iter()
                    .map(|f| (f.tree(), f.talent(), 1))
                    .collect();
                ranks.push((1, 797, rank));
                let trained = Milestones {
                    class: 8,
                    ..Default::default()
                }
                .with_ranks(&ranks);
                for (key, required) in [("bash", DruidForm::Bear), ("pounce", DruidForm::Cat)] {
                    assert!(data::CLASSES[8].pool.contains(&key));
                    assert_eq!(trained.ability_unlocked(key), form == Some(required));
                    assert!(!trained.starting_abilities("wrath").contains(&key));
                    assert_eq!(
                        trained.control(key, Effect::Stun(2.8)),
                        Effect::Stun(2.8 + 0.5 * f32::from(rank.min(2)))
                    );
                    assert_eq!(trained.control(key, Effect::Root(2.)), Effect::Root(2.));
                    assert_eq!(trained.cast(key).duration, 1.);
                }
                assert_eq!(
                    trained.control("hammer_of_justice", Effect::Stun(3.)),
                    Effect::Stun(3.)
                );
            }
        }
        let (text, spans) = talent_definition(8, 797)
            .unwrap()
            .inline_description_with_values(1)
            .unwrap();
        assert!(text.contains("0.5 » 1 sec"), "{text}");
        assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
    }
}

/// Permanent feral criticals grant immediate recovery, never resources or stored charges.
#[derive(Clone, Copy, Debug, Default)]
pub struct FeralCriticalClock {
    next: f32,
}

impl FeralCriticalClock {
    /// The caller supplies health actually removed by an original critical hit.
    pub fn chance(
        self,
        talents: super::Milestones,
        source: &str,
        critical_damage: f32,
        now: f32,
    ) -> f32 {
        if talents.class != 8
            || !critical_damage.is_finite()
            || critical_damage <= 0.
            || !now.is_finite()
            || now < 0.
            || now < self.next
        {
            return 0.;
        }
        use super::forms::DruidForm;
        let stat = match talents.druid_form() {
            Some(DruidForm::Bear) if matches!(source, "Maul" | "swipe") => TalentStat::PrimalFury,
            Some(DruidForm::Cat) if matches!(source, "Claw" | "rake" | "shred") => {
                TalentStat::BloodFrenzy
            }
            _ => return 0.,
        };
        talents.trained_value(stat)
    }

    /// Call only while the source attack has recovery left to reduce.
    pub fn refund(
        &mut self,
        talents: super::Milestones,
        source: &str,
        critical_damage: f32,
        now: f32,
        haste: f32,
        roll: f32,
    ) -> f32 {
        let chance = self.chance(talents, source, critical_damage, now);
        if chance <= 0.
            || !haste.is_finite()
            || !roll.is_finite()
            || !(0.0..1.0).contains(&roll)
            || roll >= chance
        {
            return 0.;
        }
        self.next = now + 1. / (1. + haste).max(0.1);
        0.5
    }
}

#[cfg(test)]
mod primal_fury_tests {
    use super::*;
    use crate::talents::{forms::DruidForm, Milestones};

    fn talents(form: DruidForm, rank: u8) -> Milestones {
        Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(form.tree(), form.talent(), 1), (1, 801, rank)])
    }

    #[test]
    fn fury_is_bear_only_scoped_and_clamped_with_one_shared_hasted_cooldown() {
        for form in DruidForm::ALL {
            for rank in [0, 1, 2, 255] {
                let talents = talents(form, rank);
                for source in ["Maul", "swipe", "Claw", "rake", "bash", "pounce", "wrath"] {
                    let eligible = form == DruidForm::Bear && matches!(source, "Maul" | "swipe");
                    assert_eq!(
                        FeralCriticalClock::default().chance(talents, source, 1., 0.),
                        if eligible {
                            0.5 * f32::from(rank.min(2))
                        } else {
                            0.
                        }
                    );
                }
            }
        }
        let talents = talents(DruidForm::Bear, 2);
        let mut clock = FeralCriticalClock::default();
        assert_eq!(clock.refund(talents, "Maul", 1., 0., 1., 0.9), 0.5);
        assert_eq!(clock.refund(talents, "swipe", 1., 0., 1., 0.), 0.);
        assert_eq!(clock.refund(talents, "swipe", 1., 0.49, 1., 0.), 0.);
        assert_eq!(clock.refund(talents, "swipe", 1., 0.5, 1., 0.), 0.5);
        assert_eq!(clock.refund(talents, "Maul", 1., 100., 0., 0.), 0.5);
        assert_eq!(clock.refund(talents, "Maul", 1., 100., 0., 0.), 0.);
        assert_eq!(clock.chance(talents.with_ranks(&[]), "Maul", 1., 101.), 0.);
    }

    #[test]
    fn invalid_hits_failed_rolls_and_foreign_classes_do_not_spend_fury() {
        let talents = talents(DruidForm::Bear, 1);
        let mut clock = FeralCriticalClock::default();
        for damage in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(clock.refund(talents, "Maul", damage, 0., 0., 0.), 0.);
        }
        for now in [-1., f32::NAN, f32::INFINITY] {
            assert_eq!(clock.refund(talents, "Maul", 1., now, 0., 0.), 0.);
        }
        for roll in [-0.1, 0.5, 1., f32::NAN] {
            assert_eq!(clock.refund(talents, "Maul", 1., 0., 0., roll), 0.);
        }
        assert_eq!(clock.refund(talents, "Maul", 1., 0., f32::NAN, 0.), 0.);
        assert_eq!(clock.refund(talents, "Maul", 1., 0., 0., 0.49), 0.5);
        for class in 0..8 {
            let foreign = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(1, 1_000_002, 1), (1, 801, 2)]);
            assert_eq!(
                FeralCriticalClock::default().chance(foreign, "Maul", 1., 0.),
                0.
            );
        }
        let (text, spans) = crate::talents::talent_definition(8, 801)
            .unwrap()
            .inline_description_with_values(1)
            .unwrap();
        assert!(text.contains("50 » 100% chance"), "{text}");
        assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
        assert!(text.contains("0.5 sec"));
    }
}

#[cfg(test)]
mod blood_frenzy_tests {
    use super::*;
    use crate::talents::{forms::DruidForm, Milestones};

    #[test]
    fn cat_criticals_require_the_selected_form_and_share_recovery() {
        for form in DruidForm::ALL {
            for rank in [0, 1, 2, 255] {
                let talents = Milestones {
                    class: 8,
                    ..Default::default()
                }
                .with_ranks(&[(form.tree(), form.talent(), 1), (1, 800, rank)]);
                for source in ["Claw", "rake", "shred", "Maul", "swipe", "pounce", "wrath"] {
                    let expected =
                        if form == DruidForm::Cat && matches!(source, "Claw" | "rake" | "shred") {
                            0.5 * f32::from(rank.min(2))
                        } else {
                            0.
                        };
                    assert_eq!(
                        FeralCriticalClock::default().chance(talents, source, 1., 0.),
                        expected
                    );
                }
            }
        }
        let talents = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(1, DruidForm::Cat.talent(), 1), (1, 800, 1)]);
        let mut clock = FeralCriticalClock::default();
        assert_eq!(clock.refund(talents, "Claw", 1., 0., 1., 0.5), 0.);
        assert_eq!(clock.refund(talents, "Claw", 1., 0., 1., 0.49), 0.5);
        assert_eq!(clock.refund(talents, "rake", 1., 0.49, 1., 0.), 0.);
        assert_eq!(clock.refund(talents, "shred", 1., 0.5, 1., 0.), 0.5);
        assert_eq!(clock.refund(talents, "rake", 1., 100., 0., 0.), 0.5);
        assert_eq!(clock.refund(talents, "Claw", 1., 100., 0., 0.), 0.);
        let (text, spans) = crate::talents::talent_definition(8, 800)
            .unwrap()
            .inline_description_with_values(1)
            .unwrap();
        assert!(text.contains("50 » 100% chance"), "{text}");
        assert!(spans.iter().any(|span| text[span.clone()].contains('»')));
    }
}

#[cfg(test)]
mod bite_tests {
    use super::*;
    use crate::{
        data,
        talents::{forms::DruidForm, Milestones},
    };

    #[test]
    fn bite_is_acquired_cat_physical_damage_without_builder_or_starter_effects() {
        assert!(data::CLASSES[8].pool.contains(&"ferocious_bite"));
        for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
            let mut ranks = vec![
                (0, 791, 5),
                (1, 798, 3),
                (1, 809, 1),
                (1, 800, 2),
                (0, 788, 1),
            ];
            if let Some(form) = form {
                ranks.push((form.tree(), form.talent(), 1));
            }
            let talents = Milestones {
                class: 8,
                ..Default::default()
            }
            .with_ranks(&ranks);
            assert_eq!(
                talents.ability_unlocked("ferocious_bite"),
                form == Some(DruidForm::Cat)
            );
            let kit = talents.starting_abilities("wrath");
            assert!(!kit.contains(&"ferocious_bite"));
            assert!(kit.len() <= 3);
            assert_eq!(
                FeralCriticalClock::default().chance(talents, "ferocious_bite", 10., 0.),
                0.
            );
            assert_eq!(talents.cast("ferocious_bite").damage, 1.1);
            let expected_crit = if matches!(form, Some(DruidForm::Cat | DruidForm::Bear)) {
                0.09
            } else {
                0.
            };
            assert!((talents.cast("ferocious_bite").crit - expected_crit).abs() < 0.00001);
        }
    }
}

#[cfg(test)]
mod feral_aggression_tests {
    use crate::{
        data::{self, Effect},
        talents::{talent_definition, Milestones},
    };

    #[test]
    fn aggression_scopes_both_bonuses_caps_ranks_and_refunds_without_grants() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 4, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(1, 795, rank)]);
                let ranks = if class == 8 {
                    f32::from(rank.min(5))
                } else {
                    0.
                };
                for def in data::ABILITIES {
                    let mut expected = base.cast(def.key);
                    if def.key == "ferocious_bite" {
                        expected.damage *= 1. + ranks * 0.03;
                    }
                    assert_eq!(
                        trained.cast(def.key),
                        expected,
                        "{class}/{rank}/{}",
                        def.key
                    );
                    let effect = Effect::MeleeWeaken(0.2, 11.);
                    let expected = if def.key == "demoralizing_roar" {
                        Effect::MeleeWeaken(0.2 * (1. + ranks * 0.08), 11.)
                    } else {
                        effect
                    };
                    assert_eq!(trained.control(def.key, effect), expected);
                    assert_eq!(trained.with_ranks(&[]).cast(def.key), base.cast(def.key));
                    assert_eq!(
                        trained.with_ranks(&[]).control(def.key, effect),
                        base.control(def.key, effect)
                    );
                    assert_eq!(
                        trained.ability_unlocked(def.key),
                        base.ability_unlocked(def.key)
                    );
                }
                assert_eq!(
                    trained.starting_abilities(data::CLASSES[class].start),
                    base.starting_abilities(data::CLASSES[class].start)
                );
            }
        }
        let trained = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(1, 795, 5), (0, 791, 5)]);
        assert!((trained.cast("ferocious_bite").damage - 1.265).abs() < 0.00001);
        assert!(
            matches!(trained.control("demoralizing_roar", Effect::MeleeWeaken(0.2, 10.)), Effect::MeleeWeaken(n, 10.) if (n - 0.28).abs() < 0.00001)
        );
    }

    #[test]
    fn aggression_highlights_both_independent_values_without_repeating_units() {
        let node = talent_definition(8, 795).unwrap();
        for rank in 0..=5 {
            let (text, spans) = node.inline_description_with_values(rank).unwrap();
            assert_eq!(spans.len(), 2);
            let values: Vec<_> = spans.into_iter().map(|s| text[s].to_owned()).collect();
            let expected = if rank < 5 {
                vec![
                    format!("+{} » {}% reduction strength", rank * 8, (rank + 1) * 8),
                    format!("+{} » {}% damage", rank * 3, (rank + 1) * 3),
                ]
            } else {
                vec!["+40% reduction strength".into(), "+15% damage".into()]
            };
            assert_eq!(values, expected);
        }
        assert_eq!(node.inline_description(255), node.inline_description(5));
    }
}
