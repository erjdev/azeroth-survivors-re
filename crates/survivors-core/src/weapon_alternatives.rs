//! Extra weapon choices keep the original 54 collection identities stable.
//! Identity source: docs/SURVIVORS-GEAR-VISUALS.md; acquisition is authored Survivors tuning.
use crate::{gear_tier::GearTier, gear_visuals::GearVisual, supplemental_gear::SupplementalPiece};

pub const CLASS: usize = 0;
pub const SLOT: usize = 8;
pub const ZONE: usize = 0;
pub const BOSS: &str = "Hogger";

#[derive(Clone, Copy, Debug)]
pub struct WeaponAlternative {
    pub class: usize,
    pub key: &'static str,
    pub name: &'static str,
    pub visual: GearVisual,
}

impl WeaponAlternative {
    pub fn piece(self) -> SupplementalPiece {
        SupplementalPiece {
            key: self.key.to_owned(),
            name: self.name,
            tier: GearTier::Common,
            class: self.class,
            slot: SLOT,
            visual: self.visual,
        }
    }
}

const WEAPONS: [WeaponAlternative; 2] = [
    WeaponAlternative {
        class: CLASS,
        key: "weapon:warrior:worn-mace",
        name: "Worn Mace",
        visual: GearVisual {
            item_id: 36,
            display_id: 5194,
            inventory_type: 21,
            item_class: 2,
            subclass: 4,
            material: 2,
            sheath: 3,
        },
    },
    WeaponAlternative {
        class: CLASS,
        key: "weapon:warrior:short-spear",
        name: "Short Spear",
        visual: GearVisual {
            item_id: 15810,
            display_id: 26500,
            inventory_type: 17,
            item_class: 2,
            subclass: 6,
            material: 1,
            sheath: 1,
        },
    },
];

const ROGUE_WEAPONS: [WeaponAlternative; 2] = [
    WeaponAlternative {
        class: 3,
        key: "weapon:rogue:worn-mace",
        name: "Worn Mace",
        visual: WEAPONS[0].visual,
    },
    WeaponAlternative {
        class: 3,
        key: "weapon:rogue:right-handed-brass-knuckles",
        name: "Right-Handed Brass Knuckles",
        visual: GearVisual {
            item_id: 15905,
            display_id: 26592,
            inventory_type: 21,
            item_class: 2,
            subclass: 13,
            material: 1,
            sheath: 7,
        },
    },
];

pub fn choices(class: usize) -> &'static [WeaponAlternative] {
    match class {
        CLASS => &WEAPONS,
        3 => &ROGUE_WEAPONS,
        _ => &[],
    }
}

pub fn by_key(key: &str) -> Option<SupplementalPiece> {
    WEAPONS
        .iter()
        .chain(ROGUE_WEAPONS.iter())
        .find(|weapon| weapon.key == key)
        .map(|weapon| weapon.piece())
}

/// One guaranteed alternative, independent of the existing head-piece roll.
pub fn drop_key(
    class: usize,
    zone: usize,
    creature: &str,
    boss: bool,
    roll: f32,
) -> Option<&'static str> {
    if choices(class).is_empty()
        || zone != ZONE
        || creature != BOSS
        || !boss
        || !(0.0..1.0).contains(&roll)
    {
        return None;
    }
    Some(choices(class)[usize::from(roll >= 0.5)].key)
}

pub fn source_label(class: usize) -> String {
    let options = choices(class);
    if options.is_empty() {
        return String::new();
    }
    format!("{} · {BOSS}: one random Common alternative weapon for {}s ({} or {}; 50% each), alongside the existing head-piece roll. Duplicates add no power and never change equipment or appearance. Keen Eye does not change this guaranteed reward.", crate::campaign::ZONES[ZONE].name, crate::data::CLASSES[class].name, options[0].name, options[1].name)
}

impl WeaponAlternative {
    pub fn kind_label(self) -> &'static str {
        match self.visual.subclass {
            4 => "One-handed mace",
            6 => "Two-handed polearm",
            13 => "Main-hand fist weapon",
            _ => "Weapon",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        campaign,
        gear_loadout::{Loadout, MeleeKind, MeleeWeapon},
        supplemental_gear::{self, Inventory},
    };

    #[test]
    fn source_is_class_zone_and_boss_scoped_with_exact_roll_boundaries() {
        assert!(campaign::DROPS.iter().any(|row| row.zone == ZONE
            && row.creature == BOSS
            && crate::encounters::drop_is_boss(row)));
        for (roll, index) in [(0.0, 0), (0.49999, 0), (0.5, 1), (0.99999, 1)] {
            assert_eq!(
                drop_key(CLASS, ZONE, BOSS, true, roll),
                Some(WEAPONS[index].key)
            );
        }
        for class in (1..=crate::data::CLASSES.len()).filter(|&class| class != 3) {
            assert!(choices(class).is_empty());
            assert_eq!(drop_key(class, ZONE, BOSS, true, 0.0), None);
        }
        for zone in 1..=campaign::ZONES.len() {
            assert_eq!(drop_key(CLASS, zone, BOSS, true, 0.0), None);
        }
        assert_eq!(drop_key(CLASS, ZONE, "Mother Fang", true, 0.0), None);
        assert_eq!(drop_key(CLASS, ZONE, BOSS, false, 0.0), None);
        for roll in [-1.0, 1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(drop_key(CLASS, ZONE, BOSS, true, roll), None);
        }
    }

    #[test]
    fn stable_keys_reject_aliases_and_ownership_does_not_stack() {
        let mut owned = Inventory::default();
        for weapon in choices(CLASS) {
            assert_eq!(supplemental_gear::by_key(weapon.key), Some(weapon.piece()));
            assert!(!owned.collect(1, weapon.key));
            assert!(owned.collect(CLASS, weapon.key));
            assert!(!owned.collect(CLASS, weapon.key));
        }
        for key in [
            "weapon:warrior:worn-mace:1",
            "weapon:paladin:worn-mace",
            "weapon:warrior:36",
            "weapon:warrior:short-spear ",
            "weapon:warrior:",
        ] {
            assert!(supplemental_gear::by_key(key).is_none());
            assert!(!owned.collect(CLASS, key));
        }
        assert_eq!(owned.keys().count(), 2);
    }

    #[test]
    fn equipped_type_and_common_power_are_independent_of_appearance() {
        let pieces = [0; campaign::PIECES];
        let mut owned = Inventory::default();
        let mut loadout = Loadout::default();
        assert!(!loadout.wear_supplemental(CLASS, &owned, SLOT, WEAPONS[0].key));
        for weapon in WEAPONS {
            assert!(owned.collect(CLASS, weapon.key));
        }
        for (index, kind, two_handed) in
            [(0, MeleeKind::Mace, false), (1, MeleeKind::Polearm, true)]
        {
            let key = WEAPONS[index].key;
            assert!(!loadout.wear_supplemental(1, &owned, SLOT, key));
            assert!(!loadout.wear_supplemental(CLASS, &owned, 7, key));
            assert!(loadout.wear_supplemental(CLASS, &owned, SLOT, key));
            assert!(loadout.show_supplemental(CLASS, &owned, SLOT, WEAPONS[1 - index].key));
            assert_eq!(
                loadout.melee_weapon(CLASS, &pieces, &owned),
                Some(MeleeWeapon { kind, two_handed })
            );
            let bonus = loadout.bonus(CLASS, &pieces, &owned);
            let expected = GearTier::Common.piece_bonus(SLOT);
            assert_eq!(bonus.might, expected.might);
            assert_eq!(bonus.health, expected.health);
            assert_eq!(loadout.raid_pieces(CLASS, &owned), 0);
        }
        assert!(pieces.iter().all(|count| *count == 0));
    }
}

#[cfg(test)]
mod rogue_tests {
    use super::*;
    use crate::{
        campaign,
        gear_loadout::{Loadout, MeleeKind},
        supplemental_gear::Inventory,
    };
    #[test]
    fn rogue_rewards_have_separate_keys_class_ownership_and_no_duplicate_power() {
        let mut inventory = Inventory::default();
        let pieces = [0; campaign::PIECES];
        let mut loadout = Loadout::default();
        for (index, weapon) in choices(3).iter().enumerate() {
            assert_eq!(
                drop_key(3, ZONE, BOSS, true, index as f32 * 0.5),
                Some(weapon.key)
            );
            assert_eq!(by_key(weapon.key), Some(weapon.piece()));
            assert!(!inventory.collect(0, weapon.key));
            assert!(inventory.collect(3, weapon.key));
            assert!(!inventory.collect(3, weapon.key));
            assert!(loadout.wear_supplemental(3, &inventory, SLOT, weapon.key));
            let melee = loadout.melee_weapon(3, &pieces, &inventory).unwrap();
            assert_eq!(
                melee.kind,
                if index == 0 {
                    MeleeKind::Mace
                } else {
                    MeleeKind::Fist
                }
            );
            assert!(!melee.two_handed);
            assert_eq!(
                loadout.bonus(3, &pieces, &inventory).might,
                GearTier::Common.piece_bonus(SLOT).might
            );
            assert!(!loadout.wear_supplemental(0, &inventory, SLOT, weapon.key));
        }
        assert!(loadout.show_supplemental(3, &inventory, SLOT, choices(3)[0].key));
        assert_eq!(
            loadout.melee_weapon(3, &pieces, &inventory).unwrap().kind,
            MeleeKind::Fist
        );
        assert_eq!(
            loadout
                .visible(3, &pieces, &inventory, SLOT)
                .unwrap()
                .visual
                .subclass,
            4
        );
        assert_eq!(loadout.raid_pieces(3, &inventory), 0);
        assert_eq!(inventory.keys().count(), 2);
        assert!(!source_label(3).contains("Warrior"));
        assert!(source_label(3).contains("Right-Handed Brass Knuckles"));
        assert!(source_label(usize::MAX).is_empty());
        assert_eq!(choices(3)[1].kind_label(), "Main-hand fist weapon");
        assert_eq!(choices(0)[1].kind_label(), "Two-handed polearm");
        for roll in [-1.0, 1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(drop_key(3, ZONE, BOSS, true, roll), None);
        }
        assert_eq!(drop_key(3, ZONE, BOSS, false, 0.0), None);
        assert_eq!(drop_key(3, ZONE + 1, BOSS, true, 0.0), None);
        assert_eq!(drop_key(3, ZONE, "Mother Fang", true, 0.0), None);
    }
}
