//! Named gear visuals verified against a pinned vanilla item-template catalog.
//! Source and unresolved identities: docs/SURVIVORS-GEAR-VISUALS.md.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearVisual {
    pub item_id: u32,
    pub display_id: u32,
    pub inventory_type: u8,
}

/// Unknown and ambiguous names deliberately have no visual fallback.
pub fn named(name: &str) -> Option<GearVisual> {
    let (item_id, display_id, inventory_type) = match name {
        "Field Plate Helmet" => (9290, 25839, 1),
        "Field Plate Pauldrons" => (9292, 27361, 3),
        "Field Plate Armor" => (9286, 27356, 5),
        "Field Plate Girdle" => (9288, 27359, 6),
        "Field Plate Leggings" => (9291, 27360, 7),
        "Field Plate Boots" => (9289, 27357, 8),
        "Field Plate Vambraces" => (9285, 27362, 9),
        "Field Plate Gauntlets" => (9287, 27358, 10),
        "Whirlwind Axe" => (6975, 22734, 17),
        "Skullflame Shield" => (1168, 30993, 14),
        "Thunderfury, Blessed Blade of the Windseeker" => (19019, 30606, 13),
        "Lightforge Helm" => (16727, 31207, 1),
        "Lightforge Spaulders" => (16729, 29971, 3),
        "Lightforge Breastplate" => (16726, 29969, 5),
        "Lightforge Belt" => (16723, 29966, 6),
        "Lightforge Legplates" => (16728, 29972, 7),
        "Lightforge Boots" => (16725, 29967, 8),
        "Lightforge Bracers" => (16722, 29968, 9),
        "Lightforge Gauntlets" => (16724, 29970, 10),
        "Quel'Serrar" => (18348, 30994, 21),
        "Aegis of the Scarlet Commander" => (7726, 18751, 14),
        "Corrupted Ashbringer" => (22691, 35097, 17),
        "Beaststalker's Cap" => (16677, 31410, 1),
        "Beaststalker's Mantle" => (16679, 31409, 3),
        "Beaststalker's Tunic" => (16674, 31402, 5),
        "Beaststalker's Belt" => (16680, 31404, 6),
        "Beaststalker's Pants" => (16678, 31403, 7),
        "Beaststalker's Boots" => (16675, 31408, 8),
        "Beaststalker's Bindings" => (16681, 31405, 9),
        "Beaststalker's Gloves" => (16676, 31406, 10),
        "Blackcrow" => (12651, 22929, 26),
        "Ancient Sinew Wrapped Lamina" => (18714, 31162, 18),
        "Rhok'delar, Longbow of the Ancient Keepers" => (18713, 31338, 15),
        "Shadowcraft Cap" => (16707, 28180, 1),
        "Shadowcraft Spaulders" => (16708, 28179, 3),
        "Shadowcraft Tunic" => (16721, 28160, 5),
        "Shadowcraft Belt" => (16713, 28177, 6),
        "Shadowcraft Pants" => (16709, 28161, 7),
        "Shadowcraft Boots" => (16711, 28162, 8),
        "Shadowcraft Bracers" => (16710, 24190, 9),
        "Shadowcraft Gloves" => (16712, 28166, 10),
        "Shadowblade" => (2163, 20291, 13),
        "Core Hound Tooth" => (18805, 33626, 13),
        "Death's Sting" => (21126, 34513, 13),
        "Devout Crown" => (16693, 31104, 1),
        "Devout Mantle" => (16695, 31103, 3),
        "Devout Robe" => (16690, 30422, 20),
        "Devout Belt" => (16696, 30425, 6),
        "Devout Skirt" => (16694, 30424, 7),
        "Devout Sandals" => (16691, 30430, 8),
        "Devout Bracers" => (16697, 30426, 9),
        "Devout Gloves" => (16692, 30427, 10),
        "Anathema" => (18609, 31346, 17),
        "Drake Fang Talisman" => (19406, 31936, 12),
        "Coif of Elements" => (16667, 31117, 1),
        "Pauldrons of Elements" => (16669, 30925, 3),
        "Vest of Elements" => (16666, 31416, 5),
        "Cord of Elements" => (16673, 31413, 6),
        "Kilt of Elements" => (16668, 31415, 7),
        "Boots of Elements" => (16670, 31412, 8),
        "Bindings of Elements" => (16671, 31411, 9),
        "Gauntlets of Elements" => (16672, 31414, 10),
        "Fist of Omokk" => (13167, 25180, 17),
        "Sulfuras, Hand of Ragnaros" => (17182, 29698, 17),
        "Magister's Crown" => (16686, 31087, 1),
        "Magister's Mantle" => (16689, 30211, 3),
        "Magister's Robes" => (16688, 29591, 20),
        "Magister's Belt" => (16685, 29596, 6),
        "Magister's Leggings" => (16687, 29273, 7),
        "Magister's Boots" => (16682, 29594, 8),
        "Magister's Bindings" => (16683, 29597, 9),
        "Magister's Gloves" => (16684, 29593, 10),
        "Staff of Dominance" => (18842, 34114, 17),
        "Deathmist Mask" => (22074, 34624, 1),
        "Deathmist Mantle" => (22073, 34623, 3),
        "Deathmist Robe" => (22075, 34625, 20),
        "Deathmist Belt" => (22070, 34620, 6),
        "Deathmist Leggings" => (22072, 34622, 7),
        "Deathmist Sandals" => (22076, 34626, 8),
        "Deathmist Bracers" => (22071, 34621, 9),
        "Deathmist Wraps" => (22077, 34627, 10),
        "Staff of the Shadow Flame" => (19356, 31960, 17),
        "Wildheart Cowl" => (16720, 31228, 1),
        "Wildheart Spaulders" => (16718, 30412, 3),
        "Wildheart Vest" => (16706, 29974, 5),
        "Wildheart Belt" => (16716, 29976, 6),
        "Wildheart Kilt" => (16719, 29975, 7),
        "Wildheart Boots" => (16715, 29981, 8),
        "Wildheart Bracers" => (16714, 29977, 9),
        "Wildheart Gloves" => (16717, 29979, 10),
        "Wushoolay's Poker" => (19965, 32439, 21),
        _ => return None,
    };
    Some(GearVisual {
        item_id,
        display_id,
        inventory_type,
    })
}

pub fn piece(class: usize, slot: usize, band: usize) -> Option<GearVisual> {
    named(super::campaign::piece_name(class, slot, band)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_and_ambiguous_items_never_borrow_a_visual() {
        assert_eq!(named("Atiesh, Greatstaff of the Guardian"), None);
        assert_eq!(piece(0, 0, 1), None);
        assert_eq!(piece(99, 0, 0), None);
        assert_eq!(piece(0, 9, 0), None);
    }

    #[test]
    fn named_armor_matches_its_slot_and_weapons_remain_distinct() {
        for class in 0..9 {
            for (slot, expected) in [1, 3, 5, 6, 7, 8, 9, 10].into_iter().enumerate() {
                let visual = piece(class, slot, 0).expect("named armor has a mapping");
                assert!(visual.item_id > 0 && visual.display_id > 0);
                assert!(
                    visual.inventory_type == expected || (slot == 2 && visual.inventory_type == 20)
                );
            }
        }
        assert_eq!(piece(0, 8, 0).unwrap().item_id, 6975);
        assert_eq!(piece(0, 8, 5).unwrap().item_id, 19019);
        assert_ne!(piece(0, 8, 0), piece(0, 8, 5));
    }
}
