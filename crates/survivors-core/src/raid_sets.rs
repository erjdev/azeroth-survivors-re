//! Initial Survivors raid armor builds. These are not vanilla item-set effects.
use crate::campaign::Bonus;

pub const NAMES: [&str; 9] = [
    "Wrath",
    "Judgement",
    "Dragonstalker",
    "Bloodfang",
    "Transcendence",
    "Ten Storms",
    "Netherwind",
    "Nemesis",
    "Stormrage",
];

/// Cumulative 2/4/6-piece bonuses. Only distinct, owned, worn armor slots count.
/// The loadout resolves ownership; cosmetics and ordinary Epic variants never count.
pub fn bonus(class: usize, pieces: usize) -> Bonus {
    let mut b = Bonus::default();
    if class >= NAMES.len() {
        return b;
    }
    if pieces >= 2 {
        match class {
            0 | 1 => b.armor = 24.,
            2 | 3 => b.speed = 0.04,
            4 | 8 => b.health = 30.,
            _ => b.pickup = 0.15,
        }
    }
    if pieces >= 4 {
        match class {
            0 | 3 => b.crit = 0.04,
            1 | 5 | 6 => b.area = 0.10,
            2 => b.multistrike = 0.08,
            4 | 8 => b.cooldown = 0.06,
            7 => b.leech = 0.02,
            _ => unreachable!(),
        }
    }
    if pieces >= 6 {
        match class {
            0 | 2 | 3 => b.crit_damage = 0.25,
            1 | 4 | 8 => b.reduction = 0.04,
            5 | 6 => b.cooldown = 0.08,
            7 => b.might = 0.10,
            _ => unreachable!(),
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gear_tier::GearTier, supplemental_gear};

    #[test]
    fn catalog_variants_have_stable_distinct_identity_and_verified_art() {
        let mut keys = std::collections::HashSet::new();
        for class in 0..9 {
            for slot in 0..8 {
                let piece = supplemental_gear::piece(class, slot, GearTier::RaidEpic).unwrap();
                assert!(keys.insert(piece.key.clone()));
                assert_eq!(supplemental_gear::by_key(&piece.key), Some(piece.clone()));
                assert_eq!(
                    piece.visual,
                    crate::gear_visuals::piece(class, slot, 3).unwrap()
                );
                assert_eq!(piece.visual.item_class, 4);
                assert_eq!(piece.tier.legacy_band(), None);
            }
            assert!(supplemental_gear::piece(class, 8, GearTier::RaidEpic).is_none());
        }
        assert_eq!(keys.len(), 72);
    }

    #[test]
    fn thresholds_are_cumulative_and_build_specific() {
        for n in 0..=9 {
            let warrior = bonus(0, n);
            assert_eq!(warrior.armor, if n >= 2 { 24. } else { 0. });
            assert_eq!(warrior.crit, if n >= 4 { 0.04 } else { 0. });
            assert_eq!(warrior.crit_damage, if n >= 6 { 0.25 } else { 0. });
            let warlock = bonus(7, n);
            assert_eq!(warlock.pickup, if n >= 2 { 0.15 } else { 0. });
            assert_eq!(warlock.leech, if n >= 4 { 0.02 } else { 0. });
            assert_eq!(warlock.might, if n >= 6 { 0.10 } else { 0. });
            assert_eq!(bonus(99, n).might, 0.);
            for class in 0..9 {
                assert_eq!(bonus(class, n).targets, 0);
            }
        }
    }
}
