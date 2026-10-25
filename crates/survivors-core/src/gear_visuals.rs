//! Named gear visuals verified against a pinned vanilla item-template catalog.
//! Source and unresolved identities: docs/SURVIVORS-GEAR-VISUALS.md.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearVisual {
    pub item_id: u32,
    pub display_id: u32,
    pub inventory_type: u8,
    pub item_class: u8,
    pub subclass: u8,
    pub material: i32,
    pub sheath: u8,
}

/// Unknown and ambiguous names deliberately have no visual fallback.
pub fn named(name: &str) -> Option<GearVisual> {
    let (item_id, display_id, inventory_type, item_class, subclass, material, sheath) = match name {
        "Field Plate Helmet" => (9290, 25839, 1, 4, 4, 6, 0),
        "Field Plate Pauldrons" => (9292, 27361, 3, 4, 4, 6, 0),
        "Field Plate Armor" => (9286, 27356, 5, 4, 4, 6, 0),
        "Field Plate Girdle" => (9288, 27359, 6, 4, 4, 6, 0),
        "Field Plate Leggings" => (9291, 27360, 7, 4, 4, 6, 0),
        "Field Plate Boots" => (9289, 27357, 8, 4, 4, 6, 0),
        "Field Plate Vambraces" => (9285, 27362, 9, 4, 4, 6, 0),
        "Field Plate Gauntlets" => (9287, 27358, 10, 4, 4, 6, 0),
        "Whirlwind Axe" => (6975, 22734, 17, 2, 1, 1, 1),
        "Skullflame Shield" => (1168, 30993, 14, 4, 6, 1, 4),
        "Thunderfury, Blessed Blade of the Windseeker" => (19019, 30606, 13, 2, 7, 1, 1),
        "Lightforge Helm" => (16727, 31207, 1, 4, 4, 6, 0),
        "Lightforge Spaulders" => (16729, 29971, 3, 4, 4, 6, 0),
        "Lightforge Breastplate" => (16726, 29969, 5, 4, 4, 6, 0),
        "Lightforge Belt" => (16723, 29966, 6, 4, 4, 6, 0),
        "Lightforge Legplates" => (16728, 29972, 7, 4, 4, 6, 0),
        "Lightforge Boots" => (16725, 29967, 8, 4, 4, 6, 0),
        "Lightforge Bracers" => (16722, 29968, 9, 4, 4, 6, 0),
        "Lightforge Gauntlets" => (16724, 29970, 10, 4, 4, 6, 0),
        "Quel'Serrar" => (18348, 30994, 21, 2, 7, 1, 3),
        "Aegis of the Scarlet Commander" => (7726, 18751, 14, 4, 6, 1, 4),
        "Corrupted Ashbringer" => (22691, 35097, 17, 2, 8, 1, 1),
        "Beaststalker's Cap" => (16677, 31410, 1, 4, 3, 5, 0),
        "Beaststalker's Mantle" => (16679, 31409, 3, 4, 3, 5, 0),
        "Beaststalker's Tunic" => (16674, 31402, 5, 4, 3, 5, 0),
        "Beaststalker's Belt" => (16680, 31404, 6, 4, 3, 5, 0),
        "Beaststalker's Pants" => (16678, 31403, 7, 4, 3, 5, 0),
        "Beaststalker's Boots" => (16675, 31408, 8, 4, 3, 5, 0),
        "Beaststalker's Bindings" => (16681, 31405, 9, 4, 3, 5, 0),
        "Beaststalker's Gloves" => (16676, 31406, 10, 4, 3, 5, 0),
        "Blackcrow" => (12651, 22929, 26, 2, 18, 2, 0),
        "Ancient Sinew Wrapped Lamina" => (18714, 31162, 18, 11, 2, 8, 0),
        "Rhok'delar, Longbow of the Ancient Keepers" => (18713, 31338, 15, 2, 2, 2, 0),
        "Shadowcraft Cap" => (16707, 28180, 1, 4, 2, 8, 0),
        "Shadowcraft Spaulders" => (16708, 28179, 3, 4, 2, 8, 0),
        "Shadowcraft Tunic" => (16721, 28160, 5, 4, 2, 8, 0),
        "Shadowcraft Belt" => (16713, 28177, 6, 4, 2, 8, 0),
        "Shadowcraft Pants" => (16709, 28161, 7, 4, 2, 8, 0),
        "Shadowcraft Boots" => (16711, 28162, 8, 4, 2, 8, 0),
        "Shadowcraft Bracers" => (16710, 24190, 9, 4, 2, 8, 0),
        "Shadowcraft Gloves" => (16712, 28166, 10, 4, 2, 8, 0),
        "Shadowblade" => (2163, 20291, 13, 2, 15, 1, 3),
        "Core Hound Tooth" => (18805, 33626, 13, 2, 15, 1, 3),
        "Death's Sting" => (21126, 34513, 13, 2, 15, 1, 3),
        "Devout Crown" => (16693, 31104, 1, 4, 1, 7, 0),
        "Devout Mantle" => (16695, 31103, 3, 4, 1, 7, 0),
        "Devout Robe" => (16690, 30422, 20, 4, 1, 7, 0),
        "Devout Belt" => (16696, 30425, 6, 4, 1, 7, 0),
        "Devout Skirt" => (16694, 30424, 7, 4, 1, 7, 0),
        "Devout Sandals" => (16691, 30430, 8, 4, 1, 7, 0),
        "Devout Bracers" => (16697, 30426, 9, 4, 1, 7, 0),
        "Devout Gloves" => (16692, 30427, 10, 4, 1, 7, 0),
        "Anathema" => (18609, 31346, 17, 2, 10, 2, 2),
        "Drake Fang Talisman" => (19406, 31936, 12, 4, 0, 4, 0),
        "Coif of Elements" => (16667, 31117, 1, 4, 3, 5, 0),
        "Pauldrons of Elements" => (16669, 30925, 3, 4, 3, 5, 0),
        "Vest of Elements" => (16666, 31416, 5, 4, 3, 5, 0),
        "Cord of Elements" => (16673, 31413, 6, 4, 3, 5, 0),
        "Kilt of Elements" => (16668, 31415, 7, 4, 3, 5, 0),
        "Boots of Elements" => (16670, 31412, 8, 4, 3, 5, 0),
        "Bindings of Elements" => (16671, 31411, 9, 4, 3, 5, 0),
        "Gauntlets of Elements" => (16672, 31414, 10, 4, 3, 5, 0),
        "Fist of Omokk" => (13167, 25180, 17, 2, 5, 2, 1),
        "Sulfuras, Hand of Ragnaros" => (17182, 29698, 17, 2, 5, 1, 1),
        "Magister's Crown" => (16686, 31087, 1, 4, 1, 7, 0),
        "Magister's Mantle" => (16689, 30211, 3, 4, 1, 7, 0),
        "Magister's Robes" => (16688, 29591, 20, 4, 1, 7, 0),
        "Magister's Belt" => (16685, 29596, 6, 4, 1, 7, 0),
        "Magister's Leggings" => (16687, 29273, 7, 4, 1, 7, 0),
        "Magister's Boots" => (16682, 29594, 8, 4, 1, 7, 0),
        "Magister's Bindings" => (16683, 29597, 9, 4, 1, 7, 0),
        "Magister's Gloves" => (16684, 29593, 10, 4, 1, 7, 0),
        "Staff of Dominance" => (18842, 34114, 17, 2, 10, 2, 2),
        "Deathmist Mask" => (22074, 34624, 1, 4, 1, 7, 0),
        "Deathmist Mantle" => (22073, 34623, 3, 4, 1, 7, 0),
        "Deathmist Robe" => (22075, 34625, 20, 4, 1, 7, 0),
        "Deathmist Belt" => (22070, 34620, 6, 4, 1, 7, 0),
        "Deathmist Leggings" => (22072, 34622, 7, 4, 1, 7, 0),
        "Deathmist Sandals" => (22076, 34626, 8, 4, 1, 7, 0),
        "Deathmist Bracers" => (22071, 34621, 9, 4, 1, 7, 0),
        "Deathmist Wraps" => (22077, 34627, 10, 4, 1, 7, 0),
        "Staff of the Shadow Flame" => (19356, 31960, 17, 2, 10, 2, 2),
        "Wildheart Cowl" => (16720, 31228, 1, 4, 2, 8, 0),
        "Wildheart Spaulders" => (16718, 30412, 3, 4, 2, 8, 0),
        "Wildheart Vest" => (16706, 29974, 5, 4, 2, 8, 0),
        "Wildheart Belt" => (16716, 29976, 6, 4, 2, 8, 0),
        "Wildheart Kilt" => (16719, 29975, 7, 4, 2, 8, 0),
        "Wildheart Boots" => (16715, 29981, 8, 4, 2, 8, 0),
        "Wildheart Bracers" => (16714, 29977, 9, 4, 2, 8, 0),
        "Wildheart Gloves" => (16717, 29979, 10, 4, 2, 8, 0),
        "Wushoolay's Poker" => (19965, 32439, 21, 2, 15, 1, 3),
        _ => return None,
    };
    Some(GearVisual {
        item_id,
        display_id,
        inventory_type,
        item_class,
        subclass,
        material,
        sheath,
    })
}

pub fn piece(class: usize, slot: usize, band: usize) -> Option<GearVisual> {
    if band == 1 {
        return super::gear_upgrades::piece(class, slot).map(|(_, visual)| visual);
    }
    let name = super::campaign::piece_name(class, slot, band)?;
    if name == "Atiesh, Greatstaff of the Guardian" {
        // Pinned quest rewards disambiguate the four identically named staffs.
        let (item_id, display_id) = match super::data::CLASSES.get(class)?.id {
            5 => (22631, 35634),  // Priest: quest 9257.
            8 => (22589, 35632),  // Mage: quest 9270.
            9 => (22630, 35631),  // Warlock: quest 9271.
            11 => (22632, 35633), // Druid: quest 9269.
            _ => return None,
        };
        return Some(GearVisual {
            item_id,
            display_id,
            inventory_type: 17,
            item_class: 2,
            subclass: 10,
            material: 2,
            sheath: 2,
        });
    }
    named(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_and_ambiguous_items_never_borrow_a_visual() {
        assert_eq!(named("Atiesh, Greatstaff of the Guardian"), None);
        assert_eq!(piece(0, 8, 2), None);
        assert_eq!(piece(0, 0, 2), None);
        assert_eq!(piece(99, 0, 0), None);
        assert_eq!(piece(0, 9, 0), None);
    }

    #[test]
    fn atiesh_uses_the_class_quest_reward_without_a_name_only_fallback() {
        for (class, item, display) in [
            (4, 22631, 35634),
            (6, 22589, 35632),
            (7, 22630, 35631),
            (8, 22632, 35633),
        ] {
            let visual = piece(class, 8, 5).unwrap();
            assert_eq!((visual.item_id, visual.display_id), (item, display));
            assert_eq!(
                (visual.inventory_type, visual.item_class, visual.subclass),
                (17, 2, 10)
            );
            assert_eq!((visual.material, visual.sheath), (2, 2));
            assert_ne!(piece(class, 8, 0), Some(visual));
            assert_eq!(piece(class, 7, 5), None);
        }
        assert_eq!(named("Atiesh, Greatstaff of the Guardian"), None);
    }

    #[test]
    fn uncommon_weapons_have_class_appropriate_attachment_metadata() {
        for (class, item, inventory, subclass) in [
            (0, 12784, 17, 1),
            (1, 18717, 17, 5),
            (2, 18738, 26, 18),
            (3, 12783, 13, 15),
            (4, 18608, 17, 10),
            (5, 11923, 21, 4),
            (6, 13937, 17, 10),
            (7, 18534, 17, 10),
            (8, 18531, 17, 5),
        ] {
            let weapon = piece(class, 8, 1).unwrap();
            assert_eq!(
                (
                    weapon.item_id,
                    weapon.item_class,
                    weapon.inventory_type,
                    weapon.subclass
                ),
                (item, 2, inventory, subclass)
            );
            assert_ne!(Some(weapon), piece(class, 8, 0));
            assert_ne!(Some(weapon), piece(class, 8, 5));
            for slot in 0..9 {
                assert!(piece(class, slot, 1).is_some());
                assert!(crate::campaign::piece_name(class, slot, 1).is_some());
            }
        }
        assert_eq!(piece(9, 8, 1), None);
        assert_eq!(piece(0, 9, 1), None);
    }

    #[test]
    fn named_armor_matches_its_slot_and_weapons_remain_distinct() {
        for class in 0..9 {
            for (slot, expected) in [1, 3, 5, 6, 7, 8, 9, 10].into_iter().enumerate() {
                for band in [0, 1] {
                    let visual = piece(class, slot, band).expect("named armor has a mapping");
                    assert!(super::super::campaign::piece_name(class, slot, band).is_some());
                    assert!(visual.item_id > 0 && visual.display_id > 0);
                    assert!(
                        visual.inventory_type == expected
                            || (slot == 2 && visual.inventory_type == 20)
                    );
                }
            }
        }
        assert_eq!(piece(0, 8, 0).unwrap().item_id, 6975);
        assert_eq!(piece(0, 8, 5).unwrap().item_id, 19019);
        assert_ne!(piece(0, 8, 0), piece(0, 8, 5));
    }
}
