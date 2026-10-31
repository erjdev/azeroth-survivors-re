//! Authored creature categories, independent of display, scale and encounter role.

/// Vanilla creature-type IDs. Unknown is deliberately outside every target mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CreatureType {
    #[default]
    Unknown = 0,
    Beast = 1,
    Dragonkin = 2,
    Demon = 3,
    Elemental = 4,
    Giant = 5,
    Undead = 6,
    Humanoid = 7,
    Critter = 8,
    Mechanical = 9,
    Unspecified = 10,
    Totem = 11,
}

impl CreatureType {
    pub const fn matches_mask(self, mask: u32) -> bool {
        let id = self as u8;
        id != 0 && mask & (1 << (id - 1)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::CreatureType::*;

    #[test]
    fn vanilla_masks_do_not_confuse_size_faction_or_unknown_types() {
        for kind in [
            Unknown,
            Beast,
            Dragonkin,
            Demon,
            Elemental,
            Giant,
            Undead,
            Humanoid,
            Critter,
            Mechanical,
            Unspecified,
            Totem,
        ] {
            assert_eq!(
                kind.matches_mask(19),
                matches!(kind, Beast | Dragonkin | Giant)
            );
            assert_eq!(kind.matches_mask(64), kind == Humanoid);
            assert_eq!(
                kind.matches_mask(211),
                matches!(kind, Beast | Dragonkin | Giant | Humanoid | Critter)
            );
        }
        assert!(!Unknown.matches_mask(u32::MAX));
    }

    #[test]
    fn every_generated_roster_preserves_an_explicit_identity() {
        use crate::data::{roster, MAPS};
        for index in 0..MAPS.len() {
            let roster = roster(index);
            for enemy in roster.enemies.iter().chain(&roster.bosses) {
                assert_ne!(enemy.template_entry, 0, "{}", enemy.name);
                assert_ne!(enemy.creature_type, Unknown, "{}", enemy.name);
                match enemy.name {
                    "Warsong Shredder Foreman" => assert_eq!(enemy.creature_type, Mechanical),
                    "Boulderfist Ogre" | "Boulderfist Lord" => {
                        assert_eq!(enemy.creature_type, Humanoid)
                    }
                    "Crypt Fiend" => assert_eq!(enemy.creature_type, Undead),
                    "The Hederine Slayer" => assert_eq!(enemy.creature_type, Demon),
                    _ => {}
                }
            }
        }
    }
}
