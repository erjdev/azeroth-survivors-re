//! Resolve legacy collection choices and supplemental identities through one slot boundary.
use crate::{
    campaign::{self, Bonus, GRID_SLOTS},
    equipment::{Equipment, Look},
    gear_visuals::{self, GearVisual},
    supplemental_gear::{self, Inventory},
};

#[derive(Clone, Debug, Default)]
pub struct Loadout {
    legacy: Equipment,
    worn: [Option<String>; GRID_SLOTS],
    looks: [Option<String>; GRID_SLOTS],
}

#[derive(Clone, Debug)]
pub struct ResolvedPiece {
    pub name: &'static str,
    pub visual: GearVisual,
    pub bonus: Bonus,
}

// Read-only legacy access keeps collection-band inspection available during migration.
impl std::ops::Deref for Loadout {
    type Target = Equipment;
    fn deref(&self) -> &Equipment {
        &self.legacy
    }
}

impl Loadout {
    pub fn wear(&mut self, pieces: &[u8], slot: usize, band: Option<u8>) -> bool {
        self.wear_legacy(pieces, slot, band)
    }
    pub fn show(&mut self, pieces: &[u8], slot: usize, look: Look) -> bool {
        self.show_legacy(pieces, slot, look)
    }
    pub fn preview_wear_bonus(
        &self,
        class: usize,
        pieces: &[u8],
        owned: &Inventory,
        slot: usize,
        band: Option<u8>,
    ) -> Option<(Bonus, Bonus)> {
        let mut next = self.clone();
        if !next.wear_legacy(pieces, slot, band) {
            return None;
        }
        Some((
            self.bonus(class, pieces, owned),
            next.bonus(class, pieces, owned),
        ))
    }

    pub fn from_legacy(legacy: Equipment) -> Self {
        Self {
            legacy,
            ..Self::default()
        }
    }
    pub fn legacy(&self) -> &Equipment {
        &self.legacy
    }
    pub fn supplemental_worn(&self) -> &[Option<String>; GRID_SLOTS] {
        &self.worn
    }
    pub fn supplemental_looks(&self) -> &[Option<String>; GRID_SLOTS] {
        &self.looks
    }

    pub fn wear_legacy(&mut self, pieces: &[u8], slot: usize, band: Option<u8>) -> bool {
        if !self.legacy.wear(pieces, slot, band) {
            return false;
        }
        self.worn[slot] = None;
        true
    }

    pub fn wear_supplemental(
        &mut self,
        class: usize,
        owned: &Inventory,
        slot: usize,
        key: &str,
    ) -> bool {
        if supplemental(class, owned, slot, key).is_none() {
            return false;
        }
        self.legacy.worn[slot] = None;
        self.worn[slot] = Some(key.to_owned());
        true
    }

    pub fn show_legacy(&mut self, pieces: &[u8], slot: usize, look: Look) -> bool {
        if !self.legacy.show(pieces, slot, look) {
            return false;
        }
        self.looks[slot] = None;
        true
    }

    pub fn show_supplemental(
        &mut self,
        class: usize,
        owned: &Inventory,
        slot: usize,
        key: &str,
    ) -> bool {
        if supplemental(class, owned, slot, key).is_none() {
            return false;
        }
        self.legacy.looks[slot] = Look::Worn;
        self.looks[slot] = Some(key.to_owned());
        true
    }

    pub fn worn(
        &self,
        class: usize,
        pieces: &[u8],
        owned: &Inventory,
        slot: usize,
    ) -> Option<ResolvedPiece> {
        if let Some(key) = self.worn.get(slot)?.as_deref() {
            return supplemental(class, owned, slot, key);
        }
        legacy(class, pieces, slot, self.legacy.worn[slot]?)
    }

    pub fn visible(
        &self,
        class: usize,
        pieces: &[u8],
        owned: &Inventory,
        slot: usize,
    ) -> Option<ResolvedPiece> {
        if let Some(key) = self.looks.get(slot)?.as_deref() {
            return supplemental(class, owned, slot, key);
        }
        match self.legacy.looks[slot] {
            Look::Worn => self.worn(class, pieces, owned, slot),
            Look::Hidden => None,
            Look::Piece(band) => legacy(class, pieces, slot, band),
        }
    }

    /// Count owned raid armor in distinct equipped slots, independently of appearances.
    pub fn raid_pieces(&self, class: usize, owned: &Inventory) -> usize {
        self.worn
            .iter()
            .enumerate()
            .filter(|(slot, key)| {
                key.as_deref().is_some_and(|key| {
                    supplemental(class, owned, *slot, key).is_some()
                        && supplemental_gear::by_key(key)
                            .is_some_and(|piece| piece.tier == crate::gear_tier::GearTier::RaidEpic)
                })
            })
            .count()
    }

    /// Cosmetics never enter the power calculation. A full nine-slot loadout gets one set bonus.
    pub fn bonus(&self, class: usize, pieces: &[u8], owned: &Inventory) -> Bonus {
        let mut total = Bonus::default();
        let mut worn = 0;
        for slot in 0..GRID_SLOTS {
            if let Some(piece) = self.worn(class, pieces, owned, slot) {
                total.add(piece.bonus);
                worn += 1;
            }
        }
        total.add(crate::raid_sets::bonus(
            class,
            self.raid_pieces(class, owned),
        ));
        if worn == GRID_SLOTS {
            total.add(campaign::set_bonus());
        }
        total
    }
}

fn supplemental(class: usize, owned: &Inventory, slot: usize, key: &str) -> Option<ResolvedPiece> {
    if !owned.owns(class, key) {
        return None;
    }
    let piece = supplemental_gear::by_key(key).filter(|piece| piece.slot == slot)?;
    Some(ResolvedPiece {
        name: piece.name,
        visual: piece.visual,
        bonus: piece.tier.piece_bonus(slot),
    })
}

fn legacy(class: usize, pieces: &[u8], slot: usize, band: u8) -> Option<ResolvedPiece> {
    if !Equipment::owns(pieces, slot, band) {
        return None;
    }
    let band = usize::from(band);
    Some(ResolvedPiece {
        name: campaign::piece_name(class, slot, band)?,
        visual: gear_visuals::piece(class, slot, band)?,
        bonus: campaign::piece_bonus(band, slot),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gear_tier::GearTier;

    #[test]
    fn raid_sets_count_only_distinct_owned_worn_armor_and_reset_on_replacement() {
        fn values(b: Bonus) -> [f32; 16] {
            [
                b.health,
                b.pickup,
                b.armor,
                b.might,
                b.crit,
                b.multistrike,
                b.speed,
                b.leech,
                b.area,
                b.crit_damage,
                b.reduction,
                b.cooldown,
                b.gold,
                b.growth,
                b.luck,
                b.targets as f32,
            ]
        }
        for class in 0..9 {
            let pieces = [1; campaign::PIECES];
            let mut owned = Inventory::default();
            let mut loadout = Loadout::default();
            let mut item_total = Bonus::default();
            for slot in 0..8 {
                let raid = supplemental_gear::piece(class, slot, GearTier::RaidEpic).unwrap();
                assert!(!loadout.wear_supplemental(class, &owned, slot, &raid.key));
                assert!(owned.collect(class, &raid.key));
                assert!(!owned.collect(class, &raid.key));
                let before = values(loadout.bonus(class, &pieces, &owned));
                assert!(loadout.show_supplemental(class, &owned, slot, &raid.key));
                assert_eq!(values(loadout.bonus(class, &pieces, &owned)), before);
                assert!(loadout.wear_supplemental(class, &owned, slot, &raid.key));
                item_total.add(raid.tier.piece_bonus(slot));
                let mut expected = item_total;
                expected.add(crate::raid_sets::bonus(class, slot + 1));
                assert_eq!(
                    values(loadout.bonus(class, &pieces, &owned)),
                    values(expected)
                );
                assert!(loadout.show_legacy(&pieces, slot, Look::Hidden));
                assert_eq!(
                    values(loadout.bonus(class, &pieces, &owned)),
                    values(expected)
                );
            }
            // Identical ordinary Epic art does not retain raid membership.
            for slot in 0..8 {
                assert!(loadout.wear_legacy(&pieces, slot, Some(3)));
            }
            let mut expected = Bonus::default();
            for slot in 0..8 {
                expected.add(campaign::piece_bonus(3, slot));
            }
            assert_eq!(
                values(loadout.bonus(class, &pieces, &owned)),
                values(expected)
            );
        }
    }

    #[test]
    fn supplemental_replaces_power_but_looks_remain_independent_in_both_directions() {
        let class = 0;
        let slot = 7;
        let mut pieces = [0; campaign::PIECES];
        pieces[3 * GRID_SLOTS + slot] = 1;
        let mut loadout = Loadout::from_legacy(Equipment::legacy(&pieces));
        let poor = supplemental_gear::piece(class, slot, GearTier::Poor).unwrap();
        let mut owned = Inventory::default();
        assert!(!loadout.wear_supplemental(class, &owned, slot, &poor.key));
        owned.collect(class, &poor.key);
        let original = loadout.bonus(class, &pieces, &owned);
        assert!(loadout.show_supplemental(class, &owned, slot, &poor.key));
        assert_eq!(
            loadout.visible(class, &pieces, &owned, slot).unwrap().name,
            poor.name
        );
        assert_eq!(
            loadout.bonus(class, &pieces, &owned).health,
            original.health
        );
        assert!(loadout.wear_supplemental(class, &owned, slot, &poor.key));
        assert_eq!(loadout.legacy.worn[slot], None);
        assert_eq!(loadout.bonus(class, &pieces, &owned).health, 8.);
        assert_eq!(loadout.bonus(class, &pieces, &owned).might, 0.);
        assert!(loadout.show_legacy(&pieces, slot, Look::Piece(3)));
        assert_eq!(
            loadout
                .visible(class, &pieces, &owned, slot)
                .unwrap()
                .visual,
            gear_visuals::piece(class, slot, 3).unwrap()
        );
        assert_eq!(loadout.bonus(class, &pieces, &owned).health, 8.);
        assert!(loadout.show_legacy(&pieces, slot, Look::Hidden));
        assert!(loadout.visible(class, &pieces, &owned, slot).is_none());
        assert!(loadout.wear_legacy(&pieces, slot, Some(3)));
        assert!(loadout.supplemental_worn()[slot].is_none());
        assert_eq!(
            loadout.bonus(class, &pieces, &owned).health,
            original.health
        );
        assert!(loadout.wear_legacy(&pieces, slot, None));
        assert_eq!(loadout.bonus(class, &pieces, &owned).health, 0.);
        assert!(!loadout.wear_supplemental(class, &owned, 0, &poor.key));
        assert!(!loadout.wear_supplemental(1, &owned, slot, &poor.key));
        assert!(!loadout.show_supplemental(class, &owned, usize::MAX, &poor.key));
        assert!(loadout
            .visible(class, &pieces, &owned, usize::MAX)
            .is_none());
    }

    #[test]
    fn every_existing_class_and_band_retains_its_full_loadout_bonus() {
        for class in 0..9 {
            for band in 0..campaign::BANDS {
                let mut pieces = [0; campaign::PIECES];
                pieces[band * GRID_SLOTS..(band + 1) * GRID_SLOTS].fill(1);
                let equipment = Equipment::legacy(&pieces);
                let expected = campaign::equipped_bonus(&equipment.worn_pieces(&pieces), None);
                let actual =
                    Loadout::from_legacy(equipment).bonus(class, &pieces, &Inventory::default());
                macro_rules! same { ($($field:ident),*) => { $(assert_eq!(actual.$field, expected.$field, "{} class {class} band {band}", stringify!($field));)* }; }
                same!(
                    health,
                    pickup,
                    armor,
                    might,
                    crit,
                    multistrike,
                    speed,
                    leech,
                    area,
                    crit_damage,
                    reduction,
                    cooldown,
                    gold,
                    growth,
                    luck,
                    targets
                );
            }
        }
    }

    #[test]
    fn mixed_full_loadout_counts_each_slot_once_and_ownership_is_rechecked() {
        let mut pieces = [0; campaign::PIECES];
        pieces[..GRID_SLOTS].fill(1);
        let mut loadout = Loadout::from_legacy(Equipment::legacy(&pieces));
        let mut owned = Inventory::default();
        let poor = supplemental_gear::piece(0, 7, GearTier::Poor).unwrap();
        owned.collect(0, &poor.key);
        loadout.wear_supplemental(0, &owned, 7, &poor.key);
        let bonus = loadout.bonus(0, &pieces, &owned);
        loadout.wear_legacy(&pieces, 7, None);
        let empty = loadout.bonus(0, &pieces, &owned);
        assert_eq!(
            bonus.health - empty.health,
            8. + campaign::set_bonus().health
        );
        assert!((bonus.might - empty.might - campaign::set_bonus().might).abs() < 0.00001);
        loadout.wear_supplemental(0, &owned, 7, &poor.key);
        let no_ownership = loadout.bonus(0, &pieces, &Inventory::default());
        assert_eq!(no_ownership.health, empty.health);
        assert_eq!(no_ownership.might, empty.might);
        assert_eq!(loadout.bonus(99, &pieces, &owned).health, 0.);
    }
}
