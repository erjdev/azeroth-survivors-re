//! Extra weapon choices keep the original 54 collection identities stable.
//! Identity source: docs/SURVIVORS-GEAR-VISUALS.md; acquisition is authored Survivors tuning.
use crate::{gear_tier::GearTier, gear_visuals::GearVisual, supplemental_gear::SupplementalPiece};

pub const CLASS: usize = 0;
pub const SLOT: usize = 8;
pub const ZONE: usize = 0;
pub const BOSS: &str = "Hogger";

#[derive(Clone, Copy, Debug)]
pub struct WeaponAlternative {
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
            class: CLASS,
            slot: SLOT,
            visual: self.visual,
        }
    }
}

const WEAPONS: [WeaponAlternative; 2] = [
    WeaponAlternative {
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

pub fn choices(class: usize) -> &'static [WeaponAlternative] {
    if class == CLASS {
        &WEAPONS
    } else {
        &[]
    }
}

pub fn by_key(key: &str) -> Option<SupplementalPiece> {
    WEAPONS
        .iter()
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
    if class != CLASS || zone != ZONE || creature != BOSS || !boss || !(0.0..1.0).contains(&roll) {
        return None;
    }
    Some(WEAPONS[usize::from(roll >= 0.5)].key)
}

pub fn source_label() -> String {
    format!("{} · {BOSS}: one random Common alternative weapon for Warriors (Worn Mace or Short Spear; 50% each), alongside the existing head-piece roll. Duplicates add no power and never change equipment or appearance. Keen Eye does not change this guaranteed reward.", crate::campaign::ZONES[ZONE].name)
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
        for class in 1..=crate::data::CLASSES.len() {
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
