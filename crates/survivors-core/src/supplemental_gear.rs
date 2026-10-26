//! Supplemental gear identities from the pinned vanilla template source.
//! See docs/SURVIVORS-GEAR-VISUALS.md. Stats and acquisition are Survivors rules.
use crate::{gear_tier::GearTier, gear_visuals::GearVisual};

type Item = (&'static str, u32, u32, u8, u8, u8, i32, u8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupplementalPiece {
    pub key: String,
    pub name: &'static str,
    pub tier: GearTier,
    pub class: usize,
    pub slot: usize,
    pub visual: GearVisual,
}

const CLASS_KEYS: [&str; 9] = [
    "warrior", "paladin", "hunter", "rogue", "priest", "shaman", "mage", "warlock", "druid",
];
const SLOT_KEYS: [&str; 9] = [
    "head",
    "shoulders",
    "chest",
    "waist",
    "legs",
    "feet",
    "wrist",
    "hands",
    "main-hand",
];

/// A verified identity only; catalog presence does not grant ownership or a drop source.
pub fn piece(class: usize, slot: usize, tier: GearTier) -> Option<SupplementalPiece> {
    let class_key = CLASS_KEYS.get(class)?;
    let slot_key = SLOT_KEYS.get(slot)?;
    if tier != GearTier::Poor {
        return None;
    }
    let armor_family = [0, 0, 1, 2, 3, 1, 3, 3, 2][class];
    let &(name, item_id, display_id, inventory_type, item_class, subclass, material, sheath) =
        if slot == 8 {
            &WEAPONS[class]
        } else {
            &ARMOR[armor_family][slot]
        };
    Some(SupplementalPiece {
        key: format!("poor:{class_key}:{slot_key}"),
        name,
        tier,
        class,
        slot,
        visual: GearVisual {
            item_id,
            display_id,
            inventory_type,
            item_class,
            subclass,
            material,
            sheath,
        },
    })
}

/// Stable keys are independent of display names, item IDs and legacy collection indexes.
pub fn by_key(key: &str) -> Option<SupplementalPiece> {
    let mut parts = key.split(':');
    if parts.next()? != "poor" {
        return None;
    }
    let class_key = parts.next()?;
    let class = CLASS_KEYS.iter().position(|&v| v == class_key)?;
    let slot_key = parts.next()?;
    let slot = SLOT_KEYS.iter().position(|&v| v == slot_key)?;
    if parts.next().is_some() {
        return None;
    }
    piece(class, slot, GearTier::Poor)
}

const ARMOR: [[Item; 8]; 4] = [
    [
        ("Light Plate Helmet", 8755, 28847, 1, 4, 4, 5, 0),
        ("Light Plate Shoulderpads", 8086, 28403, 3, 4, 4, 6, 0),
        ("Light Plate Chestpiece", 8080, 28398, 5, 4, 4, 6, 0),
        ("Light Plate Belt", 8081, 28399, 6, 4, 4, 6, 0),
        ("Light Plate Pants", 8085, 28402, 7, 4, 4, 6, 0),
        ("Light Plate Boots", 8082, 28404, 8, 4, 4, 6, 0),
        ("Light Plate Bracers", 8083, 9388, 9, 4, 4, 6, 0),
        ("Light Plate Gloves", 8084, 28401, 10, 4, 4, 6, 0),
    ],
    [
        ("Double Mail Coif", 8748, 28389, 1, 4, 3, 5, 0),
        ("Double Mail Shoulderpads", 3814, 6914, 3, 4, 3, 5, 0),
        ("Double Mail Vest", 3815, 977, 5, 4, 3, 5, 0),
        ("Double Mail Belt", 3808, 6902, 6, 4, 3, 5, 0),
        ("Double Mail Pants", 3813, 687, 7, 4, 3, 5, 0),
        ("Double Mail Boots", 3809, 6903, 8, 4, 3, 5, 0),
        ("Double Mail Bracers", 3810, 6904, 9, 4, 3, 5, 0),
        ("Double Mail Gloves", 3812, 6905, 10, 4, 3, 5, 0),
    ],
    [
        ("Hardened Leather Helm", 8747, 21298, 1, 4, 2, 8, 0),
        ("Hardened Leather Shoulderpads", 3806, 11270, 3, 4, 2, 8, 0),
        ("Hardened Leather Tunic", 3807, 19040, 5, 4, 2, 8, 0),
        ("Hardened Leather Belt", 3800, 19042, 6, 4, 2, 8, 0),
        ("Hardened Leather Pants", 3805, 19041, 7, 4, 2, 8, 0),
        ("Hardened Leather Boots", 3801, 19043, 8, 4, 2, 8, 0),
        ("Hardened Leather Bracers", 3802, 14803, 9, 4, 2, 8, 0),
        ("Hardened Leather Gloves", 3804, 19044, 10, 4, 2, 8, 0),
    ],
    [
        ("Interlaced Cowl", 8746, 18416, 1, 4, 1, 7, 0),
        ("Interlaced Shoulderpads", 3798, 14091, 3, 4, 1, 7, 0),
        ("Interlaced Vest", 3799, 16568, 5, 4, 1, 7, 0),
        ("Interlaced Belt", 3792, 12424, 6, 4, 1, 7, 0),
        ("Interlaced Pants", 3797, 14711, 7, 4, 1, 7, 0),
        ("Interlaced Boots", 3793, 6190, 8, 4, 1, 7, 0),
        ("Interlaced Bracers", 3794, 16571, 9, 4, 1, 7, 0),
        ("Interlaced Gloves", 3796, 16569, 10, 4, 1, 7, 0),
    ],
];
const WEAPONS: [Item; 9] = [
    ("Beaten Battle Axe", 1417, 8501, 17, 2, 1, 1, 1),
    ("Cracked Sledge", 1414, 19525, 17, 2, 5, 2, 1),
    ("Cracked Shortbow", 2773, 2786, 15, 2, 2, 2, 0),
    ("Sharpened Letter Opener", 2138, 6460, 13, 2, 15, 1, 3),
    ("Withered Staff", 1411, 20442, 17, 2, 10, 2, 2),
    ("Carpenter's Mallet", 1415, 19613, 21, 2, 4, 2, 3),
    ("Rough Wooden Staff", 1515, 20421, 17, 2, 10, 2, 2),
    ("Chipped Quarterstaff", 1813, 20413, 17, 2, 10, 2, 2),
    ("Cedar Walking Stick", 1822, 20385, 17, 2, 10, 2, 2),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn poor_catalog_has_unique_stable_keys_and_class_appropriate_slots() {
        let mut keys = std::collections::HashSet::new();
        for class in 0..9 {
            for slot in 0..9 {
                let item = piece(class, slot, GearTier::Poor).unwrap();
                assert!(keys.insert(item.key.clone()));
                assert_eq!(by_key(&item.key), Some(item.clone()));
                assert_eq!((item.class, item.slot), (class, slot));
                assert!(item.visual.item_id > 0 && item.visual.display_id > 0);
                if slot < 8 {
                    assert_eq!(item.visual.inventory_type, [1, 3, 5, 6, 7, 8, 9, 10][slot]);
                    assert_eq!(item.visual.item_class, 4);
                    assert_eq!(item.visual.subclass, [4, 4, 3, 2, 1, 3, 1, 1, 2][class]);
                } else {
                    assert_eq!(item.visual.item_class, 2);
                }
            }
        }
        assert_eq!(keys.len(), 81);
        for key in [
            "",
            "poor:mage",
            "poor:mage:hands:extra",
            "raid-epic:mage:hands",
            "poor:unknown:hands",
            "poor:mage:off-hand",
        ] {
            assert!(by_key(key).is_none());
        }
        assert!(piece(9, 0, GearTier::Poor).is_none());
        assert!(piece(0, 9, GearTier::Poor).is_none());
        assert!(piece(0, 0, GearTier::RaidEpic).is_none());
    }
}
