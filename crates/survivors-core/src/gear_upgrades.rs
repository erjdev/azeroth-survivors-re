//! Authored uncommon-tier appearances from the pinned vanilla dungeon-upgrade sets.
//! Only visual identity is imported; Survivors owns rarity, power and acquisition rules.
use crate::gear_visuals::GearVisual;

type Item = (&'static str, u32, u32, u8, u8, u8, i32, u8);

pub(crate) fn piece(class: usize, slot: usize) -> Option<(&'static str, GearVisual)> {
    let &(name, item_id, display_id, inventory_type, item_class, subclass, material, sheath) =
        ARMOR.get(class)?.get(slot)?;
    Some((
        name,
        GearVisual {
            item_id,
            display_id,
            inventory_type,
            item_class,
            subclass,
            material,
            sheath,
        },
    ))
}

const ARMOR: [[Item; 8]; 9] = [
    // Vanilla item set 511.
    [
        ("Helm of Heroism", 21999, 34614, 1, 4, 4, 6, 0),
        ("Spaulders of Heroism", 22001, 34616, 3, 4, 4, 6, 0),
        ("Breastplate of Heroism", 21997, 34617, 5, 4, 4, 6, 0),
        ("Belt of Heroism", 21994, 34610, 6, 4, 4, 6, 0),
        ("Legplates of Heroism", 22000, 34615, 7, 4, 4, 6, 0),
        ("Boots of Heroism", 21995, 34611, 8, 4, 4, 6, 0),
        ("Bracers of Heroism", 21996, 34612, 9, 4, 4, 6, 0),
        ("Gauntlets of Heroism", 21998, 34613, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 516.
    [
        ("Soulforge Helm", 22091, 34524, 1, 4, 4, 6, 0),
        ("Soulforge Spaulders", 22093, 34526, 3, 4, 4, 6, 0),
        ("Soulforge Breastplate", 22089, 34519, 5, 4, 4, 6, 0),
        ("Soulforge Belt", 22086, 34520, 6, 4, 4, 6, 0),
        ("Soulforge Legplates", 22092, 34525, 7, 4, 4, 6, 0),
        ("Soulforge Boots", 22087, 34521, 8, 4, 4, 6, 0),
        ("Soulforge Bracers", 22088, 34522, 9, 4, 4, 6, 0),
        ("Soulforge Gauntlets", 22090, 34523, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 515.
    [
        ("Beastmaster's Cap", 22013, 34649, 1, 4, 3, 5, 0),
        ("Beastmaster's Mantle", 22016, 34651, 3, 4, 3, 5, 0),
        ("Beastmaster's Tunic", 22060, 34645, 5, 4, 3, 5, 0),
        ("Beastmaster's Belt", 22010, 34646, 6, 4, 3, 5, 0),
        ("Beastmaster's Pants", 22017, 34652, 7, 4, 3, 5, 0),
        ("Beastmaster's Boots", 22061, 34648, 8, 4, 3, 5, 0),
        ("Beastmaster's Bindings", 22011, 34647, 9, 4, 3, 5, 0),
        ("Beastmaster's Gloves", 22015, 34650, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 512.
    [
        ("Darkmantle Cap", 22005, 34700, 1, 4, 2, 8, 0),
        ("Darkmantle Spaulders", 22008, 34688, 3, 4, 2, 8, 0),
        ("Darkmantle Tunic", 22009, 34689, 5, 4, 2, 8, 0),
        ("Darkmantle Belt", 22002, 34699, 6, 4, 2, 8, 0),
        ("Darkmantle Pants", 22007, 34687, 7, 4, 2, 8, 0),
        ("Darkmantle Boots", 22003, 34684, 8, 4, 2, 8, 0),
        ("Darkmantle Bracers", 22004, 34685, 9, 4, 2, 8, 0),
        ("Darkmantle Gloves", 22006, 34686, 10, 4, 2, 8, 0),
    ],
    // Vanilla item set 514.
    [
        ("Virtuous Crown", 22080, 34630, 1, 4, 1, 7, 0),
        ("Virtuous Mantle", 22082, 34632, 3, 4, 1, 7, 0),
        ("Virtuous Robe", 22083, 34633, 20, 4, 1, 7, 0),
        ("Virtuous Belt", 22078, 34628, 6, 4, 1, 7, 0),
        ("Virtuous Skirt", 22085, 34635, 7, 4, 1, 7, 0),
        ("Virtuous Sandals", 22084, 34634, 8, 4, 1, 7, 0),
        ("Virtuous Bracers", 22079, 34629, 9, 4, 1, 7, 0),
        ("Virtuous Gloves", 22081, 34631, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 519.
    [
        ("Coif of The Five Thunders", 22097, 34693, 1, 4, 3, 5, 0),
        (
            "Pauldrons of The Five Thunders",
            22101,
            34697,
            3,
            4,
            3,
            5,
            0,
        ),
        ("Vest of The Five Thunders", 22102, 34698, 5, 4, 3, 5, 0),
        ("Cord of The Five Thunders", 22098, 34694, 6, 4, 3, 5, 0),
        ("Kilt of The Five Thunders", 22100, 34696, 7, 4, 3, 5, 0),
        ("Boots of The Five Thunders", 22096, 34692, 8, 4, 3, 5, 0),
        ("Bindings of The Five Thunders", 22095, 34691, 9, 4, 3, 5, 0),
        (
            "Gauntlets of The Five Thunders",
            22099,
            34695,
            10,
            4,
            3,
            5,
            0,
        ),
    ],
    // Vanilla item set 517.
    [
        ("Sorcerer's Crown", 22065, 34602, 1, 4, 1, 7, 0),
        ("Sorcerer's Mantle", 22068, 34597, 3, 4, 1, 7, 0),
        ("Sorcerer's Robes", 22069, 34596, 20, 4, 1, 7, 0),
        ("Sorcerer's Belt", 22062, 34599, 6, 4, 1, 7, 0),
        ("Sorcerer's Leggings", 22067, 34598, 7, 4, 1, 7, 0),
        ("Sorcerer's Boots", 22064, 34782, 8, 4, 1, 7, 0),
        ("Sorcerer's Bindings", 22063, 34601, 9, 4, 1, 7, 0),
        ("Sorcerer's Gloves", 22066, 34600, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 518.
    [
        ("Deathmist Mask", 22074, 34624, 1, 4, 1, 7, 0),
        ("Deathmist Mantle", 22073, 34623, 3, 4, 1, 7, 0),
        ("Deathmist Robe", 22075, 34625, 20, 4, 1, 7, 0),
        ("Deathmist Belt", 22070, 34620, 6, 4, 1, 7, 0),
        ("Deathmist Leggings", 22072, 34622, 7, 4, 1, 7, 0),
        ("Deathmist Sandals", 22076, 34626, 8, 4, 1, 7, 0),
        ("Deathmist Bracers", 22071, 34621, 9, 4, 1, 7, 0),
        ("Deathmist Wraps", 22077, 34627, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 513.
    [
        ("Feralheart Cowl", 22109, 34639, 1, 4, 2, 8, 0),
        ("Feralheart Spaulders", 22112, 34643, 3, 4, 2, 8, 0),
        ("Feralheart Vest", 22113, 34644, 5, 4, 2, 8, 0),
        ("Feralheart Belt", 22106, 34637, 6, 4, 2, 8, 0),
        ("Feralheart Kilt", 22111, 34642, 7, 4, 2, 8, 0),
        ("Feralheart Boots", 22107, 34638, 8, 4, 2, 8, 0),
        ("Feralheart Bracers", 22108, 34641, 9, 4, 2, 8, 0),
        ("Feralheart Gloves", 22110, 34640, 10, 4, 2, 8, 0),
    ],
];
