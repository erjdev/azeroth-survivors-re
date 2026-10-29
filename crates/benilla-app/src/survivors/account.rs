//! The account save: named characters, and the permanent progress shared by a class.

mod achievement_dates;

use bevy::prelude::*;
use std::collections::BTreeMap;

use super::campaign::{self, Bonus, Feats, Quality, TALENT_CAP};
use super::data;
use super::rewards::{self, Perk, PerkRanks};
use crate::local_state;

/// Purchase outcome independent of the language used by the Armory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PurchaseError {
    AlreadyLearned,
    Message(&'static str),
}

impl From<&'static str> for PurchaseError {
    fn from(message: &'static str) -> Self {
        Self::Message(message)
    }
}

const FILE: &str = "benilla-config/survivors-account.txt";

#[derive(Clone, Debug)]
pub(super) struct Character {
    pub id: u32,
    pub first: String,
    pub last: String,
    /// Index into [`data::CLASSES`].
    pub class: usize,
    pub talent_xp: u32,
    /// Selected race and sex; absent in legacy saves until a body is chosen.
    pub body: Option<(u8, u8)>,
    pub features: Option<[u8; 5]>,
    /// Which of the three saved specs the run uses.
    pub spec: u8,
    /// Three specs, each three trees of [`campaign::NODES_PER_TREE`] ranks.
    pub specs: [[u8; campaign::SPEC_LEN]; 3],
    /// Real `Talent.dbc` ranks for the three specs: tree index, talent id, rank.
    /// Empty means the skeleton above is what the run still uses.
    pub dbc: SavedTalentSpecs,
}

impl Character {
    pub(super) fn ranks(&self) -> &[u8] {
        &self.specs[usize::from(self.spec).min(2)]
    }

    pub(super) fn spent(&self) -> u32 {
        self.ranks().iter().map(|n| u32::from(*n)).sum()
    }

    /// Points in each tree of one spec. A spec that has spent on the real tree reports that.
    pub(super) fn tree_points(&self, spec: usize) -> [u32; 3] {
        let spec = spec.min(2);
        let dbc: u32 = self.dbc[spec].iter().map(|entry| u32::from(entry.2)).sum();
        if dbc > 0 {
            let mut points = [0u32; 3];
            for &(tree, _, rank) in &self.dbc[spec] {
                if tree < 3 {
                    points[usize::from(tree)] += u32::from(rank);
                }
            }
            points
        } else {
            let ranks = &self.specs[spec];
            [
                campaign::points_in_tree(ranks, 0),
                campaign::points_in_tree(ranks, 1),
                campaign::points_in_tree(ranks, 2),
            ]
        }
    }

    pub(super) fn active_points(&self) -> u32 {
        self.tree_points(usize::from(self.spec).min(2)).iter().sum()
    }

    /// Keep imported ranks available for repair without granting unearned effects.
    pub(super) fn talent_budget_valid(&self) -> bool {
        self.active_points() <= campaign::talent_level(self.talent_xp)
    }

    pub(super) fn dbc_rank(&self, talent: u32) -> u8 {
        let spec = usize::from(self.spec).min(2);
        self.dbc[spec]
            .iter()
            .find(|entry| entry.1 == talent)
            .map(|entry| entry.2)
            .unwrap_or(0)
    }
}

type SavedTalentSpecs = [Vec<(u8, u32, u8)>; 3];

fn blank_dbc() -> SavedTalentSpecs {
    [Vec::new(), Vec::new(), Vec::new()]
}

fn blank_specs() -> [[u8; campaign::SPEC_LEN]; 3] {
    [[0; campaign::SPEC_LEN]; 3]
}

#[derive(Clone, Debug)]
pub(super) struct ClassProgress {
    /// Ownership in the nine-by-six grid: zero is unseen, one is collected.
    pub pieces: [u8; campaign::PIECES],
    pub equipment: survivors_core::gear_loadout::Loadout,
    pub legendary_weapon: bool,
    pub clears: [bool; campaign::CLEAR_COUNT],
    /// Legendary amulet names whose card has been dealt.
    pub seen: Vec<String>,
}

impl Default for ClassProgress {
    fn default() -> Self {
        Self {
            pieces: [0; campaign::PIECES],
            equipment: Default::default(),
            legendary_weapon: false,
            clears: [false; campaign::CLEAR_COUNT],
            seen: Vec::new(),
        }
    }
}

/// Companion ranks bought from the rewards screen. One companion follows until a second is bought.
#[derive(Clone, Copy, Debug)]
pub(super) struct PetRanks {
    pub owned: u8,
    pub leash: u8,
    pub paws: u8,
    /// 0 none, 1 gold-sniffer, 2 wisp-herder, 3 rune-hound. One instinct per companion.
    pub instinct: [u8; 2],
    pub look: [u8; 2],
    /// Older accounts had unrestricted Chicken access; retain that entitlement.
    pub legacy_chicken: bool,
}

impl Default for PetRanks {
    fn default() -> Self {
        Self {
            owned: 1,
            leash: 0,
            paws: 0,
            instinct: [0, 0],
            look: [0, 0],
            legacy_chicken: false,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub(super) struct Account {
    pub next_id: u32,
    pub characters: Vec<Character>,
    pub classes: Vec<ClassProgress>,
    seen_jewels: Vec<String>,
    pub supplemental: survivors_core::supplemental_gear::Inventory,
    pub pets: PetRanks,
    pub perks: PerkRanks,
    earned: Vec<u16>,
    earned_dates: BTreeMap<u16, u32>,
    spent: u32,
    perk_spent: u32,
    resets: u8,
    pub runs: u32,
    pub wins: u32,
    /// 0 is no tabard. 1 is the first checked cosmetic in [`campaign::TABARDS`].
    pub tabard: u8,
    dirty: bool,
    load_failed: bool,
    pub(super) talent_rules: Option<std::sync::Arc<super::talent_validation::Rules>>,
}

impl Default for Account {
    fn default() -> Self {
        Self::load()
    }
}

impl Account {
    fn load() -> Self {
        local_state::home()
            .map(|home| Self::load_from(&home.join(FILE)))
            .unwrap_or_else(empty)
    }

    fn load_from(path: &std::path::Path) -> Self {
        let result = match local_state::read_to_string(path) {
            Ok(text) => parse(&text).map_err(|()| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid Survivors account")
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return empty(),
            Err(error) => Err(error),
        };
        result.unwrap_or_else(|error| {
            error!("Survivors account {} could not be loaded; original preserved and saving disabled: {error}", path.display());
            let mut account = empty();
            account.load_failed = true;
            account
        })
    }

    pub(super) fn recovery_notice(&self) -> Option<&'static str> {
        self.load_failed.then_some("Account could not be loaded. Saving and runs are disabled. Restore survivors-account.txt from a backup, or fix file access, then restart.")
    }

    pub fn save(&mut self) {
        if !self.dirty || self.load_failed {
            return;
        }
        let Some(home) = local_state::home() else {
            return;
        };
        if let Err(error) = self.save_to(&home.join(FILE)) {
            warn!("Survivors progress could not be saved; changes remain pending: {error}");
        }
    }

    fn save_to(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        if self.load_failed {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "saving disabled after account load failure",
            ));
        }
        if self.dirty {
            local_state::write_atomic(path, &encode(self))?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn character(&self, id: u32) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    /// Saved allocation cursors may be absent or stale; hero references must stay unique.
    fn allocate_character_id(&mut self) -> u32 {
        let start = self.next_id.max(1);
        let mut id = start;
        loop {
            let next = id.checked_add(1).unwrap_or(1);
            if self.character(id).is_none() {
                self.next_id = next;
                return id;
            }
            id = next;
            assert_ne!(id, start, "character ID space exhausted");
        }
    }

    /// The first character of a class, creating "Adventurer" of that class when none exists.
    /// The character screen will add named ones beside this.
    pub fn ensure(&mut self, class: usize) -> u32 {
        if let Some(c) = self.characters.iter().find(|c| c.class == class) {
            return c.id;
        }
        let id = self.allocate_character_id();
        let name = data::CLASSES.get(class).map(|c| c.name).unwrap_or("Hero");
        self.characters.push(Character {
            id,
            first: "Adventurer".into(),
            last: name.into(),
            class,
            talent_xp: 0,
            spec: 0,
            specs: blank_specs(),
            dbc: blank_dbc(),
            body: None,
            features: None,
        });
        self.dirty = true;
        id
    }

    pub fn set_body(&mut self, id: u32, body: (u8, u8)) -> bool {
        if !(1..=8).contains(&body.0) || body.1 > 1 {
            return false;
        }
        let Some(hero) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        if hero.body == Some(body) {
            return false;
        }
        hero.body = Some(body);
        hero.features = None;
        self.dirty = true;
        true
    }

    pub fn set_features(&mut self, id: u32, values: [u8; 5]) -> bool {
        let Some(hero) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        if hero.body.is_none() || hero.features == Some(values) {
            return false;
        }
        hero.features = Some(values);
        self.dirty = true;
        true
    }

    pub fn add_talent_xp(&mut self, id: u32, xp: u32) {
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return;
        };
        let cap_xp = TALENT_CAP * campaign::XP_PER_TALENT;
        c.talent_xp = c.talent_xp.saturating_add(xp).min(cap_xp);
        self.dirty = true;
    }

    pub fn grant_zone(&mut self, class: usize, zone: usize) {
        let Some(progress) = self.classes.get_mut(class) else {
            return;
        };
        if let Some(flag) = progress.clears.get_mut(zone) {
            *flag = true;
        }
        self.dirty = true;
    }

    /// A first drop unlocks the piece. Duplicates change neither equipment nor stats.
    pub fn collect_drop(
        &mut self,
        class: usize,
        zone: usize,
        creature: &str,
    ) -> Option<&'static str> {
        let row = campaign::drop_for(zone, creature)?;
        let name = campaign::piece_name(class, row.slot, row.band)?;
        let index = campaign::piece_index(row.band, row.slot)?;
        let progress = self.classes.get_mut(class)?;
        if progress.pieces[index] > 0 {
            return None;
        }
        progress.pieces[index] = 1;
        if progress
            .equipment
            .worn(class, &progress.pieces, &self.supplemental, row.slot)
            .is_none()
        {
            progress
                .equipment
                .wear(&progress.pieces, row.slot, Some(row.band as u8));
        }
        progress.legendary_weapon = progress
            .pieces
            .get(5 * campaign::GRID_SLOTS + 8)
            .copied()
            .unwrap_or(0)
            > 0;
        self.dirty = true;
        Some(name)
    }

    /// Poor drops fill empty slots only; duplicate ownership never re-equips or adds power.
    pub fn collect_poor_drop(&mut self, class: usize, slot: usize) -> Option<&'static str> {
        let piece = survivors_core::supplemental_gear::piece(
            class,
            slot,
            survivors_core::gear_tier::GearTier::Poor,
        )?;
        let progress = self.classes.get_mut(class)?;
        if !self.supplemental.collect(class, &piece.key) {
            return None;
        }
        if progress
            .equipment
            .worn(class, &progress.pieces, &self.supplemental, slot)
            .is_none()
        {
            progress
                .equipment
                .wear_supplemental(class, &self.supplemental, slot, &piece.key);
        }
        self.dirty = true;
        Some(piece.name)
    }

    /// Alternative weapon rewards grant ownership only, never automatic equipment or looks.
    pub fn collect_weapon_drop(
        &mut self,
        class: usize,
        zone: usize,
        creature: &str,
        boss: bool,
        roll: f32,
    ) -> Option<&'static str> {
        let key = survivors_core::weapon_alternatives::drop_key(class, zone, creature, boss, roll)?;
        self.classes.get(class)?;
        if !self.supplemental.collect(class, key) {
            return None;
        }
        self.dirty = true;
        survivors_core::weapon_alternatives::by_key(key).map(|piece| piece.name)
    }

    /// Raid loot grants ownership only; chosen equipment and appearances stay independent.
    pub fn collect_raid_drop(&mut self, class: usize, slot: usize) -> Option<&'static str> {
        let piece = survivors_core::supplemental_gear::piece(
            class,
            slot,
            survivors_core::gear_tier::GearTier::RaidEpic,
        )?;
        self.classes.get(class)?;
        if !self.supplemental.collect(class, &piece.key) {
            return None;
        }
        self.dirty = true;
        Some(piece.name)
    }

    pub fn wear_piece(&mut self, class: usize, slot: usize, band: Option<u8>) -> bool {
        let Some(progress) = self.classes.get_mut(class) else {
            return false;
        };
        if !progress.equipment.wear(&progress.pieces, slot, band) {
            return false;
        }
        self.dirty = true;
        true
    }

    pub fn select_supplemental(&mut self, class: usize, key: &str, cosmetic: bool) -> bool {
        let Some(piece) = survivors_core::supplemental_gear::by_key(key) else {
            return false;
        };
        let Some(progress) = self.classes.get_mut(class) else {
            return false;
        };
        let changed = if cosmetic {
            progress
                .equipment
                .show_supplemental(class, &self.supplemental, piece.slot, key)
        } else {
            progress
                .equipment
                .wear_supplemental(class, &self.supplemental, piece.slot, key)
        };
        self.dirty |= changed;
        changed
    }

    pub fn show_piece(&mut self, class: usize, slot: usize, look: super::equipment::Look) -> bool {
        let Some(progress) = self.classes.get_mut(class) else {
            return false;
        };
        if !progress.equipment.show(&progress.pieces, slot, look) {
            return false;
        }
        self.dirty = true;
        true
    }

    pub fn remember_amulets(&mut self, class: usize, keys: &[&str]) {
        for key in keys {
            self.see_amulet(class, &campaign::legendary_amulet(key));
        }
    }

    pub fn see_amulet(&mut self, class: usize, name: &str) {
        let name = canonical_amulet_name(class, name);
        let Some(progress) = self.classes.get_mut(class) else {
            return;
        };
        if !progress.seen.contains(&name) {
            progress.seen.push(name);
            self.dirty = true;
        }
    }

    /// Count catalog entries, never raw save rows: duplicates and foreign names earn nothing.
    pub fn legendary_progress(&self, class: usize) -> (usize, usize) {
        let Some(def) = data::CLASSES.get(class) else {
            return (0, 0);
        };
        let keys = def.ability_keys();
        let found = self.classes.get(class).map_or(0, |progress| {
            keys.iter()
                .filter(|key| progress.seen.contains(&campaign::legendary_amulet(key)))
                .count()
        });
        (found, keys.len())
    }

    fn legendary_complete(&self, class: usize) -> bool {
        let (found, total) = self.legendary_progress(class);
        total > 0 && found == total
    }

    /// Discovery belongs to the account; this never equips jewelry or changes run stats.
    pub fn remember_jewelry(
        &mut self,
        class: usize,
        items: impl IntoIterator<Item = campaign::JewelKind>,
    ) {
        for item in items {
            let (kind, index) = match item {
                campaign::JewelKind::Ring(i) => (2, i),
                campaign::JewelKind::Trinket(i) => (3, i),
                campaign::JewelKind::Amulet(i) => (4, i),
                campaign::JewelKind::Legend(key) => {
                    if data::CLASSES
                        .get(class)
                        .is_some_and(|c| c.ability_keys().contains(&key))
                    {
                        self.remember_amulets(class, &[key]);
                    }
                    continue;
                }
            };
            if let Some(key) = shared_jewel_key(kind, usize::from(index)) {
                if !self.seen_jewels.contains(&key) {
                    self.seen_jewels.push(key);
                    self.dirty = true;
                }
            }
        }
    }

    pub fn shared_jewel_seen(&self, kind: u8, index: usize) -> bool {
        shared_jewel_key(kind, index).is_some_and(|key| self.seen_jewels.contains(&key))
    }

    pub fn shared_jewel_progress(&self, kind: u8) -> (usize, usize) {
        let names = shared_jewel_names(kind);
        (
            (0..names.len())
                .filter(|&i| self.shared_jewel_seen(kind, i))
                .count(),
            names.len(),
        )
    }

    pub fn feats(&self, class: usize, id: u32) -> Feats {
        let progress = self.classes.get(class);
        let character = self.character(id).filter(|hero| hero.class == class);
        let talent_max = character
            .map(|c| campaign::talent_level(c.talent_xp) >= TALENT_CAP)
            .unwrap_or(false);
        let raid = progress
            .and_then(|p| p.clears.last().copied())
            .unwrap_or(false);
        let collection = progress
            .map(|p| campaign::collection_complete(&p.pieces))
            .unwrap_or(false);
        let amulets = self.legendary_complete(class);
        Feats {
            talent_max,
            raid,
            collection,
            amulets,
        }
    }

    pub(super) fn talents_valid(&self, hero: &Character) -> bool {
        hero.talent_budget_valid()
            && self
                .talent_rules
                .as_ref()
                .is_none_or(|rules| rules.valid(hero))
    }

    /// Permanent bonuses for a character: class gear, set, cloak, and the legendary cloak once
    /// every class has its epic.
    pub fn bonus(&self, class: usize, id: u32) -> Bonus {
        let Some(progress) = self.classes.get(class) else {
            return Bonus::default();
        };

        let hero = self
            .character(id)
            .filter(|hero| hero.class == class && self.talents_valid(hero));
        let using_dbc = hero.is_some_and(|c| {
            c.dbc[usize::from(c.spec).min(2)]
                .iter()
                .any(|entry| entry.2 > 0)
        });
        let talents = hero
            .map(|c| c.specs[usize::from(c.spec).min(2)])
            .unwrap_or([0; campaign::SPEC_LEN]);
        let dbc_points = hero
            .map(|c| c.tree_points(usize::from(c.spec).min(2)))
            .unwrap_or([0; 3]);
        let cloak = campaign::cloak_quality(self.feats(class, id));
        let mut total = progress
            .equipment
            .bonus(class, &progress.pieces, &self.supplemental);
        if let Some(quality) = cloak {
            total.add(campaign::cloak_bonus(quality));
        }
        if self.legendary_ready() {
            let cloak = campaign::cloak_bonus(Quality::Legendary);
            total.might += cloak.might;
            total.reduction += cloak.reduction;
            total.multistrike += cloak.multistrike;
            total.leech += cloak.leech;
            total.crit_damage += cloak.crit_damage;
            total.targets += cloak.targets;
        }
        if using_dbc {
            total.add(campaign::talent_bonus_spent(dbc_points));
            if let Some(hero) = hero.filter(|hero| hero.class == class) {
                total.add(survivors_core::talents::stat_adjustment(
                    class,
                    &hero.dbc[usize::from(hero.spec).min(2)],
                ));
            }
        } else {
            total.add(campaign::talent_bonus(&talents));
        }
        total
    }

    /// Spend one talent point on the selected spec. A row stays shut until this tree holds
    /// 5 points for each step down.
    pub fn learn(&mut self, id: u32, node: usize) -> bool {
        let Some(c) = self.characters.iter().find(|c| c.id == id) else {
            return false;
        };
        let spec = usize::from(c.spec).min(2);
        let ranks = c.specs[spec];
        let spent = ranks.iter().map(|n| u32::from(*n)).sum();
        let unspent = campaign::unspent_talents(c.talent_xp, spent);
        if !campaign::can_learn(&ranks, node, unspent) {
            return false;
        }
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        c.specs[spec][node] = c.specs[spec][node].saturating_add(1);
        self.dirty = true;
        true
    }

    /// Take one rank back. A point that is holding a deeper row open stays put.
    pub fn unlearn(&mut self, id: u32, node: usize) -> bool {
        let Some(c) = self.characters.iter().find(|c| c.id == id) else {
            return false;
        };
        let spec = usize::from(c.spec).min(2);
        if !campaign::can_unlearn(&c.specs[spec], node) {
            return false;
        }
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        c.specs[spec][node] = c.specs[spec][node].saturating_sub(1);
        self.dirty = true;
        true
    }

    /// The run uses this spec. The other two keep their points.
    pub fn select_spec(&mut self, id: u32, spec: u8) {
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return;
        };
        c.spec = spec.min(2);
        self.dirty = true;
    }

    /// Reset is free and clears only the selected spec. Points return to the unspent pool.
    pub fn respec(&mut self, id: u32) {
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return;
        };
        let spec = usize::from(c.spec).min(2);
        c.specs[spec] = [0; campaign::SPEC_LEN];
        c.dbc[spec].clear();
        self.dirty = true;
    }

    /// Spend one rank on a `Talent.dbc` talent. The skeleton spread on this spec returns to
    /// the unspent pool, because the run follows the real tree once it has a point.
    pub fn learn_talent(
        &mut self,
        id: u32,
        tree: u8,
        talent: u32,
        max_rank: u8,
        row: u8,
        prereq_met: bool,
    ) -> bool {
        let Some(c) = self.characters.iter().find(|c| c.id == id) else {
            return false;
        };
        let spec = usize::from(c.spec).min(2);
        let rank = c.dbc[spec]
            .iter()
            .find(|entry| entry.1 == talent)
            .map(|entry| entry.2)
            .unwrap_or(0);
        let points = c.dbc[spec]
            .iter()
            .filter(|entry| entry.0 == tree)
            .map(|entry| u32::from(entry.2))
            .sum();
        let spent = c.dbc[spec].iter().map(|entry| u32::from(entry.2)).sum();
        let unspent = campaign::unspent_talents(c.talent_xp, spent);
        if !campaign::can_learn_talent(rank, max_rank, row, points, prereq_met, unspent) {
            return false;
        }
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        c.specs[spec] = [0; campaign::SPEC_LEN];
        if let Some(entry) = c.dbc[spec].iter_mut().find(|entry| entry.1 == talent) {
            entry.2 = entry.2.saturating_add(1);
        } else {
            c.dbc[spec].push((tree, talent, 1));
        }
        self.dirty = true;
        true
    }

    /// Take one rank off a `Talent.dbc` talent. `allowed` is the row and prerequisite check.
    pub fn unlearn_talent(&mut self, id: u32, talent: u32, allowed: bool) -> bool {
        if !allowed {
            return false;
        }
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        let spec = usize::from(c.spec).min(2);
        let Some(pos) = c.dbc[spec].iter().position(|entry| entry.1 == talent) else {
            return false;
        };
        if c.dbc[spec][pos].2 == 0 {
            return false;
        }
        c.dbc[spec][pos].2 -= 1;
        if c.dbc[spec][pos].2 == 0 {
            c.dbc[spec].remove(pos);
        }
        self.dirty = true;
        true
    }

    /// A named hero. Empty names are refused.
    pub fn create(&mut self, first: &str, last: &str, class: usize) -> Option<u32> {
        let first = first.trim();
        let last = last.trim();
        if first.is_empty() || last.is_empty() || class >= data::CLASSES.len() {
            return None;
        }
        let id = self.allocate_character_id();
        self.characters.push(Character {
            id,
            first: first.to_string(),
            last: last.to_string(),
            class,
            talent_xp: 0,
            spec: 0,
            specs: blank_specs(),
            dbc: blank_dbc(),
            body: None,
            features: None,
        });
        self.dirty = true;
        Some(id)
    }

    /// Replaces a hero's name. Empty names are refused.
    pub fn rename(&mut self, id: u32, first: &str, last: &str) -> bool {
        let first = first.trim();
        let last = last.trim();
        if first.is_empty() || last.is_empty() {
            return false;
        }
        let Some(hero) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        hero.first = first.to_string();
        hero.last = last.to_string();
        self.dirty = true;
        self.save();
        true
    }

    /// Snapshot the selected hero's active spec for the next run, without sharing effects
    /// across heroes or changing an already-running game when the account is edited.
    pub fn milestones_picked(
        &self,
        class: usize,
        id: Option<u32>,
    ) -> survivors_core::talents::Milestones {
        let hero = match id {
            Some(id) => self.character(id).filter(|hero| hero.class == class),
            None => self.characters.iter().find(|hero| hero.class == class),
        };
        let hero = hero.filter(|hero| self.talents_valid(hero));
        survivors_core::talents::Milestones {
            class,
            points: hero
                .map(|hero| hero.tree_points(usize::from(hero.spec)))
                .unwrap_or([0; 3]),
            weapon: hero
                .and_then(|_| self.classes.get(class))
                .and_then(|progress| {
                    progress
                        .equipment
                        .melee_weapon(class, &progress.pieces, &self.supplemental)
                }),
            ..Default::default()
        }
        .with_ranks(
            hero.map(|hero| hero.dbc[usize::from(hero.spec)].as_slice())
                .unwrap_or(&[]),
        )
    }

    pub fn bonus_for(&self, class: usize) -> Bonus {
        let id = self
            .characters
            .iter()
            .find(|c| c.class == class)
            .map(|c| c.id)
            .unwrap_or(0);
        self.bonus(class, id)
    }

    /// The matching picked hero's sheet, or the class's first hero when no ID was selected.
    /// Explicit missing or wrong-class IDs grant no bonuses.
    pub fn bonus_picked(&self, class: usize, character: Option<u32>) -> Bonus {
        match character {
            Some(id) if self.character(id).is_some_and(|hero| hero.class == class) => {
                self.bonus(class, id)
            }
            // An explicit stale/mismatched selection must never borrow another hero's sheet.
            Some(_) => Bonus::default(),
            None => self.bonus_for(class),
        }
    }

    pub fn epic_cloaks(&self) -> usize {
        let mut epics = 0;
        for class in 0..self.classes.len().min(data::CLASSES.len()) {
            let ids: Vec<u32> = self
                .characters
                .iter()
                .filter(|c| c.class == class)
                .map(|c| c.id)
                .collect();
            if ids
                .iter()
                .any(|id| campaign::cloak_quality(self.feats(class, *id)) == Some(Quality::Epic))
            {
                epics += 1;
            }
        }
        epics
    }

    fn legendary_ready(&self) -> bool {
        campaign::legendary_cloak(self.epic_cloaks())
    }

    pub fn unspent(&self) -> u32 {
        rewards::points_of(&self.earned).saturating_sub(self.spent)
    }

    pub fn earned_ids(&self) -> &[u16] {
        &self.earned
    }

    pub fn earned_date(&self, id: u16) -> Option<String> {
        self.earned_dates
            .get(&id)
            .copied()
            .and_then(achievement_dates::label)
    }

    pub fn resets_left(&self) -> u8 {
        self.resets
    }

    pub fn next_locked(&self) -> Option<String> {
        rewards::next_locked(&self.snapshot(), &self.earned)
    }

    pub fn achievement_list(&self) -> Vec<rewards::Listed> {
        rewards::listed(&self.snapshot(), &self.earned)
    }

    /// Pays any achievement the account now meets. A second call pays nothing new.
    pub fn grant_new(&mut self) -> bool {
        self.grant_new_on(achievement_dates::today())
    }

    fn grant_new_on(&mut self, day: Option<u32>) -> bool {
        let day = day.filter(|day| *day <= achievement_dates::MAX_DAY);
        let found = rewards::fresh(&self.snapshot(), &self.earned);
        if found.is_empty() {
            return false;
        }
        for (id, _) in found {
            self.earned.push(id);
            if let Some(day) = day {
                self.earned_dates.insert(id, day);
            }
        }
        self.dirty = true;
        true
    }

    /// Reward mutations stay in memory so the Armory can preview a batch before saving.
    pub fn buy_perk(&mut self, perk: Perk) -> Result<(), &'static str> {
        let Some(cost) = rewards::perk_cost(perk, self.perks.rank(perk)) else {
            return Err("Max rank");
        };
        self.pay(cost)?;
        self.perks.raise(perk);
        self.perk_spent += cost;
        Ok(())
    }

    pub fn buy_leash(&mut self) -> Result<(), &'static str> {
        self.raise_pet(true)
    }

    pub fn buy_paws(&mut self) -> Result<(), &'static str> {
        self.raise_pet(false)
    }

    pub fn buy_second(&mut self) -> Result<(), &'static str> {
        if self.pets.owned >= 2 {
            return Err("Max rank");
        }
        self.pay(40)?;
        self.pets.owned = 2;
        Ok(())
    }

    /// Initial instincts fill the first free owned slot; replacements use the first companion.
    pub fn instinct_target(&self) -> Option<usize> {
        (0..usize::from(self.pets.owned.min(2)))
            .find(|&slot| self.pets.instinct[slot] == 0)
            .or_else(|| (self.pets.owned > 0).then_some(0))
    }

    pub fn buy_instinct(&mut self, kind: u8) -> Result<(), PurchaseError> {
        if !(1..=3).contains(&kind) {
            return Err("Unknown instinct".into());
        }
        if !super::companions::instinct_available(kind) {
            return Err(
                "Rune-hound is unavailable until battleground runes are implemented".into(),
            );
        }
        let slot = self.instinct_target().ok_or("Unlock a companion first")?;
        if self.pets.instinct[slot] == kind {
            return Err(PurchaseError::AlreadyLearned);
        }
        self.pay(30)?;
        self.pets.instinct[slot] = kind;
        Ok(())
    }

    pub fn look_unlocked(&self, look: u8) -> bool {
        let Some(entry) = super::companions::LOOKS.get(usize::from(look)) else {
            return false;
        };
        (look == 2 && self.pets.legacy_chicken)
            || entry
                .achievement
                .is_none_or(|id| rewards::available(id) && self.earned.contains(&id))
    }

    /// Wear one of the checked critter looks. An unknown index leaves the slot as it was.
    pub fn wear_look(&mut self, slot: u8, look: u8) {
        let slot = usize::from(slot);
        if slot >= usize::from(self.pets.owned.min(2)) || !self.look_unlocked(look) {
            return;
        }
        self.pets.look[slot] = look;
        self.dirty = true;
        self.save();
    }

    /// Wear an account tabard, or clear it with 0. A tabard adds nothing to the sheet.
    pub fn wear_tabard(&mut self, index: u8) {
        if index > 0 && campaign::TABARDS.get(usize::from(index - 1)).is_none() {
            return;
        }
        self.tabard = index;
        self.dirty = true;
        self.save();
    }

    /// Refunds perk points and leaves companion ranks. Two resets are kept on the account.
    pub fn reset_perks(&mut self) -> Result<(), &'static str> {
        if self.resets == 0 {
            return Err("No perk resets left");
        }
        if self.perk_spent == 0 {
            return Err("No perk points to take back");
        }
        self.spent = self.spent.saturating_sub(self.perk_spent);
        self.perk_spent = 0;
        self.perks = PerkRanks::default();
        self.resets -= 1;
        self.dirty = true;
        Ok(())
    }

    fn pay(&mut self, cost: u32) -> Result<(), &'static str> {
        if self.unspent() < cost {
            return Err("Not enough achievement points");
        }
        self.spent += cost;
        self.dirty = true;
        Ok(())
    }

    fn raise_pet(&mut self, leash: bool) -> Result<(), &'static str> {
        let rank = if leash {
            self.pets.leash
        } else {
            self.pets.paws
        };
        if rank >= 3 {
            return Err("Max rank");
        }
        let cost = [15, 25, 35][usize::from(rank)];
        self.pay(cost)?;
        if leash {
            self.pets.leash += 1;
        } else {
            self.pets.paws += 1;
        }
        Ok(())
    }

    fn snapshot(&self) -> rewards::Snapshot {
        let mut snap = rewards::Snapshot::default();
        let mut epics = 0u32;
        let mut supplemental_slots = [0u16; 9];
        for key in self.supplemental.keys() {
            if let Some(piece) = survivors_core::supplemental_gear::by_key(key) {
                snap.pieces += 1;
                supplemental_slots[piece.class] |= 1 << piece.slot;
            }
        }
        for (i, class) in self.classes.iter().enumerate().take(9) {
            let n = snap.clears[i].len().min(class.clears.len());
            snap.clears[i][..n].copy_from_slice(&class.clears[..n]);
            snap.pieces += campaign::owned_pieces(&class.pieces) as u32;
            if (0..campaign::GRID_SLOTS).all(|slot| {
                supplemental_slots[i] & (1 << slot) != 0
                    || campaign::best_piece(&class.pieces, slot).is_some()
            }) {
                snap.full_sets += 1;
            }
            let legendary = class.pieces[5 * campaign::GRID_SLOTS..6 * campaign::GRID_SLOTS]
                .iter()
                .filter(|&&owned| owned > 0)
                .count() as u32;
            snap.best_legendary_set = snap.best_legendary_set.max(legendary);
            snap.legends += self.legendary_progress(i).0 as u32;
            let talent_max = self
                .characters
                .iter()
                .any(|c| c.class == i && campaign::talent_level(c.talent_xp) >= TALENT_CAP);
            snap.talent_cap[i] = talent_max;
            let feats = Feats {
                talent_max,
                raid: class.clears.last().copied().unwrap_or(false),
                collection: campaign::collection_complete(&class.pieces),
                amulets: self.legendary_complete(i),
            };
            if campaign::cloak_quality(feats).is_some() {
                snap.any_cloak = true;
            }
            if campaign::cloak_quality(feats) == Some(Quality::Epic) {
                epics += 1;
            }
        }
        snap.jewelry = snap.legends
            + (2..=4)
                .map(|kind| self.shared_jewel_progress(kind).0 as u32)
                .sum::<u32>();
        snap.epic_cloaks = epics;
        snap.legendary_cloak = campaign::legendary_cloak(epics as usize);
        snap.runs = self.runs;
        snap.wins = self.wins;
        snap
    }
}

#[cfg(test)]
pub(super) fn blank() -> Account {
    empty()
}

pub(super) fn empty() -> Account {
    Account {
        next_id: 1,
        characters: Vec::new(),
        classes: vec![ClassProgress::default(); data::CLASSES.len()],
        seen_jewels: Vec::new(),
        supplemental: Default::default(),
        pets: PetRanks::default(),
        perks: PerkRanks::default(),
        earned: Vec::new(),
        earned_dates: BTreeMap::new(),
        spent: 0,
        perk_spent: 0,
        resets: 2,
        runs: 0,
        wins: 0,
        tabard: 0,
        dirty: false,
        load_failed: false,
        talent_rules: None,
    }
}

fn encode(account: &Account) -> String {
    let mut out = String::from("# survivors account: characters, then one class block each\n");
    out.push_str(&format!("next {}\n", account.next_id));
    for c in &account.characters {
        let talents = encode_specs(c);
        out.push_str(&format!(
            "who {} {} {} {} {} {talents}\n",
            c.id,
            c.class,
            c.talent_xp,
            escape(&c.first),
            escape(&c.last),
        ));
        if let Some((race, sex)) = c.body {
            out.push_str(&format!("body {} {race} {sex}\n", c.id));
            if let Some([skin, face, hair, color, facial]) = c.features {
                out.push_str(&format!(
                    "features {} {race} {sex} {skin} {face} {hair} {color} {facial}\n",
                    c.id
                ));
            }
        }
        if c.dbc
            .iter()
            .any(|spec| spec.iter().any(|entry| entry.2 > 0))
        {
            out.push_str(&format!("dbc {} {}\n", c.id, encode_dbc(&c.dbc)));
        }
    }
    for (i, class) in account.classes.iter().enumerate() {
        let gear = class
            .pieces
            .iter()
            .map(|n| char::from(b'0' + (*n).min(1)))
            .collect::<String>();
        let clears = class
            .clears
            .iter()
            .map(|g| if *g { "1" } else { "0" })
            .collect::<Vec<_>>()
            .join("");
        out.push_str(&format!(
            "class {i} {gear} {} {clears}\n",
            if class.legendary_weapon { 1 } else { 0 }
        ));
        let (worn, looks) = class.equipment.encode();
        out.push_str(&format!("equipment {i} {worn} {looks}\n"));
        for (kind, choices) in [
            ("supplemental_worn", class.equipment.supplemental_worn()),
            ("supplemental_look", class.equipment.supplemental_looks()),
        ] {
            for (slot, key) in choices.iter().enumerate() {
                if let Some(key) = key {
                    out.push_str(&format!("{kind} {i} {slot} {key}\n"));
                }
            }
        }
        for name in &class.seen {
            out.push_str(&format!("seen {i} {}\n", escape(name)));
        }
    }
    for key in account.supplemental.keys() {
        let piece = survivors_core::supplemental_gear::by_key(key)
            .expect("inventory admits only catalogued keys");
        out.push_str(&format!("supplemental {} {key}\n", piece.class));
    }
    for key in &account.seen_jewels {
        out.push_str(&format!("jewel_seen {key}\n"));
    }
    out.push_str(&format!(
        "pets {} {} {} {} {} {} {} {}\n",
        account.pets.owned,
        account.pets.leash,
        account.pets.paws,
        account.pets.instinct[0],
        account.pets.instinct[1],
        account.pets.look[0],
        account.pets.look[1],
        u8::from(account.pets.legacy_chicken),
    ));
    out.push_str(&format!(
        "meta {} {} {} {} {} {}\n",
        account.runs,
        account.wins,
        account.spent,
        account.perk_spent,
        account.resets,
        account.tabard
    ));
    let perks = account.perks;
    out.push_str(&format!(
        "perk {} {} {} {} {} {}\n",
        perks.second_thoughts,
        perks.skip,
        perks.veterans,
        perks.banish,
        perks.opening_hand,
        perks.keen_eye
    ));
    if !account.earned.is_empty() {
        let ids = account
            .earned
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!("earn {ids}\n"));
    }
    for (id, day) in &account.earned_dates {
        out.push_str(&format!("earned_date {id} {day}\n"));
    }
    out
}

fn shared_jewel_names(kind: u8) -> &'static [&'static str] {
    match kind {
        2 => &campaign::RING_NAMES,
        3 => &campaign::TRINKET_NAMES,
        4 => &campaign::AMULET_NAMES,
        _ => &[],
    }
}

fn shared_jewel_key(kind: u8, index: usize) -> Option<String> {
    let family = match kind {
        2 => "ring",
        3 => "trinket",
        4 => "amulet",
        _ => return None,
    };
    shared_jewel_names(kind)
        .get(index)
        .map(|name| format!("{family}:{name}"))
}

/// Legacy saves encode spaces as underscores, including the underscores inside ability keys.
/// Resolve only known class entries; retain unknown names for forward compatibility.
fn canonical_amulet_name(class: usize, name: &str) -> String {
    let normalized = name.replace('_', " ").to_ascii_lowercase();
    data::CLASSES
        .get(class)
        .into_iter()
        .flat_map(|class| class.ability_keys())
        .map(campaign::legendary_amulet)
        .find(|candidate| candidate.replace('_', " ").to_ascii_lowercase() == normalized)
        .unwrap_or_else(|| name.to_owned())
}

fn escape(s: &str) -> String {
    s.replace(' ', "_")
}

fn unescape(s: &str) -> String {
    s.replace('_', " ")
}

/// Seven bits are the draft path: Elwynn, Westfall, Duskwood, Stranglethorn, Arathi Highlands,
/// Western Plaguelands, Naxxramas. Westfall, the Highlands, and the Plaguelands left the path.
/// A longer string is the twelve-zone path, one bit per row.
/// Ten bits are the draft's slots. A longer string is the 54-piece grid, one ownership digit each (legacy stack counts become one).
/// The draft's off-hand bit is not a piece. Its legendary flag is the legendary main hand.
fn read_pieces(progress: &mut ClassProgress, bits: &str, legendary: bool) {
    if bits.len() <= 10 {
        for (slot, ch) in bits.chars().enumerate().take(campaign::GRID_SLOTS) {
            if ch == '1' {
                progress.pieces[slot] = 1;
            }
        }
        if legendary {
            if let Some(index) = campaign::piece_index(5, 8) {
                progress.pieces[index] = 1;
            }
        }
    } else {
        for (i, ch) in bits.chars().enumerate().take(progress.pieces.len()) {
            progress.pieces[i] = ch.to_digit(10).unwrap_or(0).min(1) as u8;
        }
    }
    progress.legendary_weapon = progress
        .pieces
        .get(5 * campaign::GRID_SLOTS + 8)
        .copied()
        .unwrap_or(0)
        > 0;
}

fn read_clears(bits: &str) -> [bool; campaign::CLEAR_COUNT] {
    let mut clears = [false; campaign::CLEAR_COUNT];
    if bits.len() == 7 {
        let on = |i: usize| bits.as_bytes().get(i).copied() == Some(b'1');
        for (from, to) in [(0usize, 0usize), (2, 1), (3, 6), (6, 11)] {
            clears[to] = on(from);
        }
    } else {
        for (i, ch) in bits.chars().enumerate().take(clears.len()) {
            clears[i] = ch == '1';
        }
    }
    clears
}

fn encode_specs(c: &Character) -> String {
    let groups = c
        .specs
        .iter()
        .map(|spec| {
            spec.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("|");
    format!("{}|{groups}", c.spec.min(2))
}

/// A save with `|` is three specs. A bare comma list is the old eighteen ranks, copied into
/// spec 0's first six nodes of each tree and clamped to the new rank caps.
fn read_specs(list: &str) -> (u8, [[u8; campaign::SPEC_LEN]; 3]) {
    let mut specs = blank_specs();
    if list.contains('|') {
        let mut parts = list.split('|');
        let spec = parts
            .next()
            .unwrap_or("0")
            .parse::<u8>()
            .unwrap_or(0)
            .min(2);
        for (i, group) in parts.enumerate().take(3) {
            for (n, rank) in group.split(',').enumerate().take(campaign::SPEC_LEN) {
                let parsed = rank.parse::<u8>().unwrap_or(0);
                specs[i][n] = parsed.min(campaign::node_max(n));
            }
        }
        return (spec, specs);
    }
    let mut old = [0u8; 18];
    for (i, n) in list.split(',').enumerate().take(18) {
        old[i] = n.parse().unwrap_or(0);
    }
    for tree in 0..3 {
        for node in 0..6 {
            let dest = tree * campaign::NODES_PER_TREE + node;
            specs[0][dest] = old[tree * 6 + node].min(campaign::node_max(dest));
        }
    }
    (0, specs)
}

fn encode_dbc(specs: &SavedTalentSpecs) -> String {
    specs
        .iter()
        .map(|spec| {
            spec.iter()
                .filter(|entry| entry.2 > 0)
                .map(|(tree, id, rank)| format!("{tree}:{id}:{rank}"))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn read_dbc(payload: &str) -> Result<SavedTalentSpecs, ()> {
    let mut specs = blank_dbc();
    for (spec, group) in payload.split('|').enumerate() {
        if spec >= specs.len() {
            return Err(());
        }
        for entry in group.split(',').filter(|s| !s.is_empty()) {
            let mut bits = entry.split(':');
            let (Some(tree), Some(id), Some(rank)) = (bits.next(), bits.next(), bits.next()) else {
                return Err(());
            };
            let (Ok(tree), Ok(id), Ok(rank)) =
                (tree.parse::<u8>(), id.parse::<u32>(), rank.parse::<u8>())
            else {
                return Err(());
            };
            if tree >= 3 || id == 0 || rank == 0 || bits.next().is_some() {
                return Err(());
            }
            // A talent owns one rank per spec; repeated rows never buy tree points.
            // Keep its first tree and highest rank, matching core spell training.
            if let Some(existing) = specs[spec].iter_mut().find(|entry| entry.1 == id) {
                existing.2 = existing.2.max(rank);
            } else {
                specs[spec].push((tree, id, rank));
            }
        }
    }
    Ok(specs)
}

fn parse(text: &str) -> Result<Account, ()> {
    let mut account = empty();
    let mut equipment_rows = Vec::new();
    let mut body_rows = Vec::new();
    let mut talent_rows = Vec::new();
    let mut supplemental_choices = Vec::new();
    let mut feature_rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("next") => account.next_id = parts.next().unwrap_or("1").parse().unwrap_or(1),
            Some("who") => {
                let id: u32 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let class: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let talent_xp: u32 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let first = unescape(parts.next().ok_or(())?);
                let last = unescape(parts.next().ok_or(())?);
                let (spec, specs) = read_specs(parts.next().unwrap_or(""));
                // A newer or damaged class identity cannot be dropped on the next save.
                // Preserve the whole file through the existing load-failure recovery path.
                if class >= data::CLASSES.len() || account.character(id).is_some() {
                    return Err(());
                }
                account.characters.push(Character {
                    id,
                    first,
                    last,
                    class,
                    talent_xp,
                    spec,
                    specs,
                    dbc: blank_dbc(),
                    body: None,
                    features: None,
                });
            }
            Some("features") => {
                let values: Vec<_> = parts.collect();
                if values.len() == 8 {
                    if let (Ok(id), Ok(values)) = (
                        values[0].parse::<u32>(),
                        values[1..]
                            .iter()
                            .map(|v| v.parse::<u8>())
                            .collect::<Result<Vec<_>, _>>(),
                    ) {
                        feature_rows.push((
                            id,
                            (values[0], values[1]),
                            [values[2], values[3], values[4], values[5], values[6]],
                        ));
                    }
                }
            }
            Some("body") => {
                let values: Vec<_> = parts.collect();
                if let [id, race, sex] = values.as_slice() {
                    if let (Ok(id), Ok(race), Ok(sex)) =
                        (id.parse::<u32>(), race.parse::<u8>(), sex.parse::<u8>())
                    {
                        if (1..=8).contains(&race) && sex <= 1 {
                            body_rows.push((id, (race, sex)));
                        }
                    }
                }
            }
            Some("dbc") => {
                let id = parts.next().ok_or(())?.parse::<u32>().map_err(|_| ())?;
                let specs = read_dbc(parts.next().ok_or(())?)?;
                if parts.next().is_some() {
                    return Err(());
                }
                talent_rows.push((id, specs));
            }
            Some("class") => {
                let index: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let gear = parts.next().unwrap_or("");
                let legendary = parts.next().unwrap_or("0") == "1";
                let clears = parts.next().unwrap_or("");
                let progress = account.classes.get_mut(index).ok_or(())?;
                read_pieces(progress, gear, legendary);
                progress.clears = read_clears(clears);
            }
            Some("equipment") => {
                if let Some(index) = parts.next().and_then(|value| value.parse::<usize>().ok()) {
                    equipment_rows.push((
                        index,
                        parts.next().unwrap_or("").to_string(),
                        parts.next().unwrap_or("").to_string(),
                    ));
                }
            }
            Some("seen") => {
                let index: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let name =
                    canonical_amulet_name(index, &unescape(&parts.collect::<Vec<_>>().join(" ")));
                if let Some(progress) = account.classes.get_mut(index) {
                    if !name.is_empty() && !progress.seen.contains(&name) {
                        progress.seen.push(name);
                    }
                }
            }
            Some("supplemental_worn" | "supplemental_look") => {
                let kind = line.split_whitespace().next().unwrap();
                if let (Some(class), Some(slot), Some(key), None) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                {
                    if let (Ok(class), Ok(slot)) = (class.parse::<usize>(), slot.parse::<usize>()) {
                        supplemental_choices.push((
                            kind == "supplemental_look",
                            class,
                            slot,
                            key.to_string(),
                        ));
                    }
                }
            }
            Some("supplemental") => {
                if let (Some(class), Some(key), None) = (parts.next(), parts.next(), parts.next()) {
                    if let Ok(class) = class.parse::<usize>() {
                        account.supplemental.collect(class, key);
                    }
                }
            }
            Some("jewel_seen") => {
                let key = parts.collect::<Vec<_>>().join(" ");
                if key
                    .split_once(':')
                    .is_some_and(|(family, name)| !family.is_empty() && !name.is_empty())
                    && !account.seen_jewels.contains(&key)
                {
                    account.seen_jewels.push(key);
                }
            }
            Some("pets") => {
                let mut n = [0u8; 7];
                for slot in &mut n {
                    *slot = parts.next().unwrap_or("0").parse().unwrap_or(0);
                }
                account.pets = PetRanks {
                    owned: n[0].min(2),
                    leash: n[1].min(3),
                    paws: n[2].min(3),
                    instinct: [n[3].min(3), n[4].min(3)],
                    look: [n[5], n[6]],
                    legacy_chicken: parts.next().is_none_or(|value| value == "1"),
                };
            }
            Some("meta") => {
                account.runs = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.wins = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.spent = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.perk_spent = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.resets = parts
                    .next()
                    .unwrap_or("2")
                    .parse::<u8>()
                    .unwrap_or(2)
                    .min(2);
                account.tabard = parts.next().unwrap_or("0").parse().unwrap_or(0);
            }
            Some("perk") => {
                let mut n = [0u8; 6];
                for slot in &mut n {
                    *slot = parts.next().unwrap_or("0").parse().unwrap_or(0);
                }
                account.perks = PerkRanks {
                    second_thoughts: n[0].min(3),
                    skip: n[1].min(1),
                    veterans: n[2].min(2),
                    banish: n[3].min(3),
                    opening_hand: n[4].min(1),
                    keen_eye: n[5].min(2),
                };
            }
            Some("earned_date") => {
                if let (Some(id), Some(day)) = (
                    parts.next().and_then(|s| s.parse::<u16>().ok()),
                    parts.next().and_then(|s| s.parse::<u32>().ok()),
                ) {
                    if day <= achievement_dates::MAX_DAY {
                        account.earned_dates.entry(id).or_insert(day);
                    }
                }
            }
            Some("earn") => {
                if let Some(list) = parts.next() {
                    for id in list.split(',') {
                        if let Ok(id) = id.parse() {
                            if !account.earned.contains(&id) {
                                account.earned.push(id);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // Refunds are a subset of total spending; never erase companion investment
    // by interpreting an impossible imported subtotal as refundable points.
    if account.perk_spent > account.spent {
        return Err(());
    }
    // Resolve references after all characters exist; later rows retain replacement semantics.
    for (id, specs) in talent_rows {
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == id)
            .ok_or(())?;
        hero.dbc = specs;
    }
    for (id, body) in body_rows {
        if let Some(hero) = account.characters.iter_mut().find(|hero| hero.id == id) {
            hero.body = Some(body);
        }
    }
    for (id, body, values) in feature_rows {
        if let Some(hero) = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == id && hero.body == Some(body))
        {
            hero.features = Some(values);
        }
    }
    for (index, progress) in account.classes.iter_mut().enumerate() {
        progress.equipment = survivors_core::gear_loadout::Loadout::from_legacy(
            if let Some((_, worn, looks)) = equipment_rows.iter().rev().find(|row| row.0 == index) {
                super::equipment::Equipment::decode(worn, looks, &progress.pieces)
            } else {
                super::equipment::Equipment::legacy(&progress.pieces)
            },
        );
    }
    for (look, class, slot, key) in supplemental_choices {
        if let Some(progress) = account.classes.get_mut(class) {
            if look {
                progress
                    .equipment
                    .show_supplemental(class, &account.supplemental, slot, &key);
            } else {
                progress
                    .equipment
                    .wear_supplemental(class, &account.supplemental, slot, &key);
            }
        }
    }
    account
        .earned_dates
        .retain(|id, _| account.earned.contains(id) && rewards::available(*id));
    // Achievement rows may follow pets; validate both slots after reading the entire save.
    for slot in 0..account.pets.look.len() {
        if !account.look_unlocked(account.pets.look[slot]) {
            account.pets.look[slot] = 0;
        }
    }
    Ok(account)
}

/// Retry any pending discovery save when returning to the menu.
pub(super) fn flush_discoveries(mut account: ResMut<Account>) {
    account.grant_new();
    account.save();
}

/// Writes the save when a run ends. A class with no character yet gets its first one here, so
/// progress is not thrown away before the character screen exists.
pub(super) fn settle(
    mut account: ResMut<Account>,
    selection: Res<super::Selection>,
    mut run: ResMut<super::Run>,
) {
    settle_run(&mut account, &selection, &mut run);
    account.save();
}

/// Apply a run once, independently of disk I/O so retries can flush without paying again.
fn settle_run(account: &mut Account, selection: &super::Selection, run: &mut super::Run) {
    if run.account_paid {
        return;
    }
    let Some(class) = selection.class.filter(|&class| class < data::CLASSES.len()) else {
        return;
    };
    let id = selection
        .character
        .filter(|&id| {
            account
                .character(id)
                .is_some_and(|hero| hero.class == class)
        })
        .unwrap_or_else(|| account.ensure(class));
    account.runs = account.runs.saturating_add(1);
    if run.won {
        account.wins = account.wins.saturating_add(1);
    }
    let mine = account
        .characters
        .iter()
        .filter(|c| c.class == class)
        .map(|c| c.talent_xp)
        .max()
        .unwrap_or(0);
    let best = account
        .characters
        .iter()
        .map(|c| c.talent_xp)
        .max()
        .unwrap_or(0);
    let xp = campaign::talent_award(
        run.level,
        run.kills,
        run.elapsed,
        run.bosses_defeated,
        run.won,
        campaign::zone_index(selection.map_def().name),
    );
    let xp = ((xp as f32) * rewards::veteran_multiplier(account.perks.veterans, mine, best)).round()
        as u32;
    account.add_talent_xp(id, xp);
    if run.won {
        if let Some(zone) = campaign::zone_index(selection.map_def().name) {
            account.grant_zone(class, zone);
        }
    }
    account.grant_new();
    run.account_paid = true;
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_load_never_overwrites_original_but_missing_save_is_writable() {
        let dir = std::env::temp_dir().join(format!("survivors-load-guard-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("account.txt");
        for bytes in [
            b"who broken\n".as_slice(),
            b"meta 0 0 40 41 2 0\n",
            &[0xff, 0xfe],
            b"who 1 0 100 First Hero\nwho 1 0 200 Second Hero\n",
            b"who 1 0 100 First Hero\nwho 1 1 200 Second Hero\n",
            b"who 1 0 100 First Hero\nwho 2 9 200 Future Hero\ndbc 2 0:74:2||\n",
            b"who 1 4294967295 200 Broken Hero\n",
            b"who 1 3 5100 Saved Rogue\ndbc 1 0:270:5,broken||\n",
            b"who 1 3 5100 Saved Rogue\ndbc bad 0:270:5||\n",
            b"who 1 3 5100 Saved Rogue\ndbc 99 0:270:5||\n",
            b"who 1 3 5100 Saved Rogue\ndbc 1\n",
            b"who 1 3 5100 Saved Rogue\ndbc 1 0:270:5|| extra\n",
            b"who 1 3 5100 Saved Rogue\ndbc 1 0:270:5|||1:187:2\n",
            b"who 1 0 100 First Hero\nclass 9 111111 1 111111111111\n",
        ] {
            std::fs::write(&path, bytes).unwrap();
            let mut account = Account::load_from(&path);
            assert!(account.recovery_notice().is_some());
            assert_eq!(
                super::super::armory::enter_refused(&account, &Default::default(), None).as_deref(),
                account.recovery_notice()
            );
            account.ensure(0);
            assert!(account.save_to(&path).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let mut account = Account::load_from(&path);
        account.ensure(0);
        assert!(account.save_to(&path).is_err());
        assert!(path.is_dir());
        std::fs::remove_dir(&path).unwrap();
        let mut account = Account::load_from(&path);
        assert!(account.recovery_notice().is_none());
        account.ensure(0);
        account.save_to(&path).unwrap();
        let restored = Account::load_from(&path);
        assert!(restored.recovery_notice().is_none());
        assert_eq!(encode(&account), encode(&restored));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn saved_rune_instinct_is_retained_without_allowing_new_purchases() {
        let mut account = parse("pets 2 0 0 3 1 0 0 0\n").unwrap();
        let before = encode(&account);
        assert!(account.buy_instinct(3).is_err());
        assert_eq!(encode(&account), before);
        assert_eq!(parse(&before).unwrap().pets.instinct, [3, 1]);
    }

    #[test]
    fn imported_reward_counters_bound_resets_and_preserve_nonperk_spending() {
        for resets in [0, 1, 2, 3, 255] {
            let mut account = parse(&format!(
                "meta 0 0 55 15 {resets} 0\nperk 1 0 0 0 0 0\npets 2 0 0 0 0 0 0 0\n"
            ))
            .unwrap();
            assert_eq!(account.resets_left(), resets.min(2));
            if resets == 0 {
                assert!(account.reset_perks().is_err());
                assert_eq!(account.spent, 55);
            } else {
                account.reset_perks().unwrap();
                assert_eq!(account.spent, 40);
                assert_eq!(account.perk_spent, 0);
                assert_eq!(account.pets.owned, 2);
                assert_eq!(account.resets_left(), resets.min(2) - 1);
                assert!(account.reset_perks().is_err());
            }
            let restored = parse(&encode(&account)).unwrap();
            assert_eq!(encode(&account), encode(&restored));
        }
        // Row ordering cannot conceal an impossible refund subtotal.
        for text in [
            "perk 1 0 0 0 0 0\nmeta 0 0 0 15 2 0\n",
            "meta 0 0 0 15 2 0\nperk 1 0 0 0 0 0\n",
        ] {
            assert!(parse(text).is_err());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_disk_write_preserves_previous_save_and_retries_pending_equipment() {
        use super::super::equipment::Look;
        let dir = std::env::temp_dir().join(format!("survivors-save-retry-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join(FILE);
        let mut account = parse(include_str!("../../testdata/survivors-ui/collected.txt")).unwrap();
        account.dirty = true;
        account.save_to(&path).unwrap();
        assert!(!account.dirty);
        let original = std::fs::read_to_string(&path).unwrap();
        assert!(account.wear_piece(0, 0, None));
        assert!(account.show_piece(0, 1, Look::Hidden));
        let pending = encode(&account);
        assert_ne!(pending, original);
        // A directory at the temporary-file location forces a real atomic-writer error.
        let obstacle = path.with_extension("tmp");
        std::fs::create_dir(&obstacle).unwrap();
        assert!(account.save_to(&path).is_err());
        assert!(account.dirty);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::remove_dir(&obstacle).unwrap();
        account.save_to(&path).unwrap();
        assert!(!account.dirty);
        let disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(disk, pending);
        let restored = parse(&disk).unwrap();
        assert_eq!(
            encode(&restored),
            pending,
            "all heroes, specs and rewards survive"
        );
        assert_eq!(restored.classes[0].equipment.worn[0], None);
        assert_eq!(restored.classes[0].equipment.looks[1], Look::Hidden);
        // A clean account must not touch storage, even when a write would fail.
        std::fs::create_dir(&obstacle).unwrap();
        account.save_to(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), pending);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ui_fixtures_parse_and_round_trip_without_claiming_zone_clears() {
        let fresh = parse(include_str!("../../testdata/survivors-ui/fresh.txt")).unwrap();
        assert!(fresh.characters.is_empty());
        assert!(fresh
            .classes
            .iter()
            .all(|class| class.pieces == [0; campaign::PIECES]));
        for (text, heroes, owned) in [
            (
                include_str!("../../testdata/survivors-ui/partial.txt"),
                2,
                36,
            ),
            (
                include_str!("../../testdata/survivors-ui/collected.txt"),
                10,
                486,
            ),
        ] {
            let account = parse(text).unwrap();
            assert_eq!(account.characters.len(), heroes);
            assert_eq!(
                account
                    .classes
                    .iter()
                    .flat_map(|c| c.pieces)
                    .filter(|&n| n > 0)
                    .count(),
                owned
            );
            assert!(account
                .classes
                .iter()
                .all(|c| c.clears.iter().all(|&clear| !clear)));
            assert!(account.characters.iter().all(|c| c.active_points() == 0));
            assert_eq!(
                account.classes[0].equipment.looks[0],
                super::super::equipment::Look::Hidden
            );
            assert_eq!(account.classes[0].equipment.worn[0], Some(0));
            let canonical = encode(&account);
            assert_eq!(encode(&parse(&canonical).unwrap()), canonical);
        }
    }

    #[test]
    fn trained_ui_fixture_has_independent_specs_without_earned_content() {
        let mut account = parse(include_str!("../../testdata/survivors-ui/trained.txt")).unwrap();
        assert_eq!(account.characters.len(), 2);
        assert!(account
            .classes
            .iter()
            .all(|c| c.pieces == [0; campaign::PIECES] && c.clears.iter().all(|&v| !v)));
        assert!(account.supplemental.keys().next().is_none());
        assert!(account.earned.is_empty());
        for (id, points) in [(1, [29, 13, 0]), (2, [13, 29, 0])] {
            for (spec, expected) in points.into_iter().enumerate() {
                account.select_spec(id, spec as u8);
                let hero = account.character(id).unwrap();
                assert_eq!(hero.active_points(), expected);
                assert!(hero.talent_budget_valid());
            }
            account.select_spec(id, 0);
        }
        let canonical = encode(&account);
        assert_eq!(encode(&parse(&canonical).unwrap()), canonical);
        account.respec(1);
        assert_eq!(account.character(1).unwrap().active_points(), 0);
        assert_eq!(account.character(2).unwrap().active_points(), 13);
        account.select_spec(1, 1);
        assert_eq!(account.character(1).unwrap().active_points(), 13);
    }

    #[test]
    fn trained_ui_fixture_specs_pass_the_checkout_talent_catalog() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let account = parse(include_str!("../../testdata/survivors-ui/trained.txt")).unwrap();
        let mut app = App::new();
        app.insert_resource(account)
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().resource_mut::<Account>();
        for id in [1, 2] {
            for spec in 0..3 {
                account.select_spec(id, spec);
                let hero = account.character(id).unwrap();
                assert!(account.talents_valid(hero), "hero {id} spec {spec}");
                assert_eq!(
                    account
                        .milestones_picked(5, Some(id))
                        .points
                        .iter()
                        .sum::<u32>(),
                    hero.active_points()
                );
            }
        }
    }

    #[test]
    fn inner_focus_reload_catalog_and_spec_selection_produce_isolated_run_snapshots() {
        use survivors_core::talents::CriticalCastClock;
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        // Spec 1 satisfies the real Discipline row gate; spec 2 lacks it; spec 3 is empty.
        let account = parse("who 1 4 5100 Saved Priest\ndbc 1 0:342:5,0:344:2,0:343:3,0:348:1|0:348:1|\nwho 2 4 5100 Other Priest\n").unwrap();
        let saved = encode(&account);
        let account = parse(&saved).unwrap();
        assert_eq!(encode(&account), saved);
        let bonus = |snapshot| CriticalCastClock::default().bonus(snapshot, "smite", 0.0);
        let mut app = App::new();
        app.insert_resource(account)
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        assert_eq!(
            bonus(
                app.world()
                    .resource::<Account>()
                    .milestones_picked(4, Some(1))
            ),
            0.0,
            "saved ranks must wait for catalog validation"
        );
        app.insert_resource(crate::ui_talent::Talents { catalog });
        app.update();
        let mut account = app.world_mut().resource_mut::<Account>();
        let active = account.milestones_picked(4, Some(1));
        assert_eq!(active.points, [11, 0, 0]);
        assert_eq!(bonus(active), 0.25);
        assert_eq!(
            bonus(account.milestones_picked(4, Some(2))),
            0.0,
            "other hero"
        );
        assert_eq!(
            bonus(account.milestones_picked(3, Some(1))),
            0.0,
            "wrong class"
        );
        for spec in [1, 2] {
            account.select_spec(1, spec);
            assert_eq!(bonus(account.milestones_picked(4, Some(1))), 0.0);
            assert_eq!(bonus(active), 0.25, "an existing run keeps its snapshot");
        }
        account.select_spec(1, 1);
        assert!(!account.talents_valid(account.character(1).unwrap()));
        assert_eq!(
            account.character(1).unwrap().dbc[1],
            vec![(0, 348, 1)],
            "invalid imported ranks stay available for repair"
        );
        account.select_spec(1, 0);
        assert_eq!(bonus(account.milestones_picked(4, Some(1))), 0.25);
        account.respec(1);
        assert_eq!(bonus(account.milestones_picked(4, Some(1))), 0.0);
        assert_eq!(bonus(active), 0.25);
        let reloaded = parse(&encode(&account)).unwrap();
        assert!(reloaded.character(1).unwrap().dbc[0].is_empty());
        assert_eq!(reloaded.character(1).unwrap().dbc[1], vec![(0, 348, 1)]);
    }

    #[test]
    fn custom_features_round_trip_by_hero_and_body_with_legacy_defaults() {
        let mut account = empty();
        let a = account.create("First", "Hero", 0).unwrap();
        let b = account.create("Second", "Hero", 0).unwrap();
        assert!(!account.set_features(a, [1; 5]));
        account.set_body(a, (1, 0));
        account.set_body(b, (1, 0));
        assert!(account.set_features(a, [1, 2, 3, 4, 5]));
        let text = encode(&account);
        let reordered = text.lines().rev().collect::<Vec<_>>().join("\n");
        let restored = parse(&reordered).unwrap();
        assert_eq!(
            restored.character(a).unwrap().features,
            Some([1, 2, 3, 4, 5])
        );
        assert_eq!(restored.character(b).unwrap().features, None);
        account.set_body(a, (1, 1));
        assert_eq!(account.character(a).unwrap().features, None);
        let invalid = parse("features 1 3 0 1 2 3 4 5\nfeatures 1 1 0 999 2 3 4 5\nbody 1 1 0\nwho 1 0 0 Old Hero\n").unwrap();
        assert_eq!(invalid.character(1).unwrap().features, None);
        let legacy = parse("who 1 0 0 Old Hero\n").unwrap();
        assert_eq!(legacy.character(1).unwrap().features, None);
    }

    #[test]
    fn character_bodies_round_trip_and_validate_independent_of_row_order() {
        let mut account = super::empty();
        let a = account.create("One", "Hero", 0).unwrap();
        let b = account.create("Two", "Hero", 0).unwrap();
        assert!(account.set_body(a, (1, 1)));
        assert!(account.set_body(b, (3, 0)));
        assert!(!account.set_body(a, (0, 1)));
        assert!(!account.set_body(a, (1, 2)));
        let text = super::encode(&account);
        let body_rows = text.lines().filter(|l| l.starts_with("body "));
        let other_rows = text.lines().filter(|l| !l.starts_with("body "));
        let reordered = body_rows.chain(other_rows).collect::<Vec<_>>().join("\n");
        let restored = super::parse(&reordered).unwrap();
        assert_eq!(restored.character(a).unwrap().body, Some((1, 1)));
        assert_eq!(restored.character(b).unwrap().body, Some((3, 0)));
        let old = super::parse("who 1 0 0 Old Hero\nbody 1 255 0\nbody 1 1 2\n").unwrap();
        assert_eq!(old.character(1).unwrap().body, None);
    }

    use super::*;

    #[test]
    fn raid_rewards_persist_without_equipping_and_duplicates_change_nothing() {
        for class in 0..data::CLASSES.len() {
            let mut account = blank();
            let equipment = account.classes[class].equipment.clone();
            for slot in 0..survivors_core::supplemental_gear::RAID_ARMOR_SLOTS {
                assert!(account.collect_raid_drop(class, slot).is_some());
            }
            assert_eq!(account.classes[class].equipment.worn, equipment.worn);
            assert_eq!(account.classes[class].equipment.looks, equipment.looks);
            assert_eq!(
                account.classes[class].equipment.supplemental_worn(),
                equipment.supplemental_worn()
            );
            assert_eq!(
                account.classes[class].equipment.supplemental_looks(),
                equipment.supplemental_looks()
            );
            let mut restored = parse(&encode(&account)).unwrap();
            assert_eq!(restored.supplemental.keys().count(), 8);
            assert_eq!(restored.classes[class].equipment.worn, equipment.worn);
            assert_eq!(restored.classes[class].equipment.looks, equipment.looks);
            assert_eq!(
                restored.classes[class].equipment.supplemental_worn(),
                equipment.supplemental_worn()
            );
            assert_eq!(
                restored.classes[class].equipment.supplemental_looks(),
                equipment.supplemental_looks()
            );
            restored.dirty = false;
            let before = encode(&restored);
            for slot in 0..8 {
                assert!(restored.collect_raid_drop(class, slot).is_none());
            }
            assert!(restored.collect_raid_drop(class, 8).is_none());
            assert!(restored.collect_raid_drop(99, 0).is_none());
            assert_eq!(encode(&restored), before);
            assert!(!restored.dirty);
        }
    }

    #[test]
    fn poor_drops_persist_fill_empty_slots_and_duplicates_do_not_reequip() {
        for class in 0..data::CLASSES.len() {
            let mut account = blank();
            for slot in 0..campaign::GRID_SLOTS {
                assert!(account.collect_poor_drop(class, slot).is_some());
                assert!(account.classes[class].equipment.supplemental_worn()[slot].is_some());
            }
            let mut account = parse(&encode(&account)).unwrap();
            assert_eq!(account.supplemental.keys().count(), campaign::GRID_SLOTS);
            assert_eq!(account.snapshot().pieces, 9);
            assert_eq!(account.snapshot().full_sets, 1);
            assert!(!account.feats(class, 0).collection);
            assert!(account.wear_piece(class, 7, None));
            account.dirty = false;
            let before = encode(&account);
            assert_eq!(account.collect_poor_drop(class, 7), None);
            assert_eq!(encode(&account), before);
            assert!(!account.dirty);
        }
    }

    #[test]
    fn alternative_rewards_preserve_legacy_progress_and_independent_saved_choices() {
        use survivors_core::{gear_loadout::MeleeKind, weapon_alternatives as weapons};
        for class in [0, 3] {
            let mut account = blank();
            let legacy = account.classes[class].pieces;
            let choices = weapons::choices(class);
            for (roll, weapon) in [(0.0, choices[0]), (0.5, choices[1])] {
                assert_eq!(
                    account.collect_weapon_drop(class, 0, weapons::BOSS, true, roll),
                    Some(weapon.name)
                );
                assert!(account.classes[class].equipment.supplemental_worn()[8].is_none());
                assert!(account.classes[class].equipment.supplemental_looks()[8].is_none());
            }
            assert_eq!(account.classes[class].pieces, legacy);
            assert!(account.select_supplemental(class, choices[0].key, false));
            assert!(account.select_supplemental(class, choices[1].key, true));
            let mut restored = parse(&encode(&account)).unwrap();
            assert_eq!(
                restored.classes[class].equipment.supplemental_worn()[8].as_deref(),
                Some(choices[0].key)
            );
            assert_eq!(
                restored.classes[class].equipment.supplemental_looks()[8].as_deref(),
                Some(choices[1].key)
            );
            assert_eq!(
                restored.classes[class]
                    .equipment
                    .melee_weapon(class, &legacy, &restored.supplemental)
                    .unwrap()
                    .kind,
                MeleeKind::Mace
            );
            let before = encode(&restored);
            for roll in [0.0, 0.5] {
                assert_eq!(
                    restored.collect_weapon_drop(class, 0, weapons::BOSS, true, roll),
                    None
                );
            }
            assert_eq!(encode(&restored), before);
            assert!(!restored.dirty);
            assert_eq!(
                restored.collect_weapon_drop(1, 0, weapons::BOSS, true, 0.0),
                None
            );
            assert!(!restored.select_supplemental(1, choices[0].key, false));
        }
    }

    #[test]
    fn new_poor_drop_preserves_existing_gear_and_cosmetic_selection() {
        let mut account = blank();
        assert!(account.collect_drop(0, 0, "Hogger").is_some());
        assert!(account.show_piece(0, 0, super::super::equipment::Look::Hidden));
        let before = account.bonus(0, 0);
        assert!(account.collect_poor_drop(0, 0).is_some());
        assert_eq!(account.classes[0].equipment.worn[0], Some(0));
        assert_eq!(
            account.classes[0].equipment.looks[0],
            super::super::equipment::Look::Hidden
        );
        assert_eq!(account.bonus(0, 0).armor, before.armor);
        assert_eq!(account.bonus(0, 0).might, before.might);
        assert_eq!(account.collect_poor_drop(99, 0), None);
        assert_eq!(account.collect_poor_drop(0, 99), None);
    }

    #[test]
    fn supplemental_loadout_restores_after_ownership_and_preserves_cosmetic_independence() {
        let base = parse("who 1 0 0 Test Warrior\n").unwrap();
        let choices =
            "supplemental_worn 0 7 poor:warrior:hands\nsupplemental_look 0 7 poor:warrior:hands\n";
        let ownership = "supplemental 0 poor:warrior:hands\n";
        for rows in [
            format!("{choices}{ownership}"),
            format!("{ownership}{choices}"),
        ] {
            let mut account = parse(&format!("who 1 0 0 Test Warrior\n{rows}")).unwrap();
            assert_eq!(account.bonus(0, 1).health, base.bonus(0, 1).health + 8.0);
            let saved = encode(&account);
            account = parse(&saved).unwrap();
            assert_eq!(
                account.classes[0].equipment.supplemental_worn()[7].as_deref(),
                Some("poor:warrior:hands")
            );
            assert_eq!(
                account.classes[0].equipment.supplemental_looks()[7].as_deref(),
                Some("poor:warrior:hands")
            );
            assert!(!account.dirty);
            assert!(account.wear_piece(0, 7, None));
            assert_eq!(account.bonus(0, 1).health, base.bonus(0, 1).health);
            assert!(account.classes[0]
                .equipment
                .visible(0, &account.classes[0].pieces, &account.supplemental, 7)
                .is_some());
            assert!(account.select_supplemental(0, "poor:warrior:hands", false));
            assert!(account.show_piece(0, 7, super::super::equipment::Look::Hidden));
            assert_eq!(account.bonus(0, 1).health, base.bonus(0, 1).health + 8.0);
            assert!(account.classes[0]
                .equipment
                .visible(0, &account.classes[0].pieces, &account.supplemental, 7)
                .is_none());
        }
    }

    #[test]
    fn raid_armor_round_trips_without_legacy_collection_or_cosmetic_set_power() {
        let mut account = empty();
        for slot in 0..6 {
            let piece = survivors_core::supplemental_gear::piece(
                0,
                slot,
                survivors_core::gear_tier::GearTier::RaidEpic,
            )
            .unwrap();
            assert!(account.supplemental.collect(0, &piece.key));
            assert!(!account.supplemental.collect(0, &piece.key));
            assert!(account.select_supplemental(0, &piece.key, false));
            assert!(account.select_supplemental(0, &piece.key, true));
        }
        let before = account.bonus(0, 0);
        let saved = encode(&account);
        let mut loaded = parse(&saved).unwrap();
        assert_eq!(loaded.supplemental.keys().count(), 6);
        assert!(loaded.classes[0].pieces.iter().all(|&count| count == 0));
        assert_eq!(loaded.bonus(0, 0).crit_damage, before.crit_damage);
        assert_eq!(loaded.bonus(0, 0).armor, before.armor);
        assert_eq!(encode(&loaded), saved);
        for slot in 0..6 {
            assert!(loaded.wear_piece(0, slot, None));
        }
        assert!(loaded.classes[0]
            .equipment
            .supplemental_looks()
            .iter()
            .take(6)
            .all(Option::is_some));
        assert_eq!(
            loaded.bonus(0, 0).crit_damage,
            empty().bonus(0, 0).crit_damage
        );
        assert_eq!(loaded.bonus(0, 0).armor, empty().bonus(0, 0).armor);
    }

    #[test]
    fn saved_raid_sets_reach_run_stats_for_every_class_without_cosmetic_power() {
        use super::super::{Run, Stats};
        use survivors_core::gear_tier::GearTier;
        let values = |s: Stats| {
            [
                s.max_hp,
                s.magnet,
                s.armor,
                s.might,
                s.crit,
                s.multistrike,
                s.speed,
                s.leech,
                s.area,
                s.crit_damage,
                s.reduction,
                s.cooldown,
            ]
        };
        let check = |actual: Stats, expected: Stats| {
            for (actual, expected) in values(actual).into_iter().zip(values(expected)) {
                assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
            }
            assert_eq!(actual.targets, expected.targets);
        };
        for class in 0..data::CLASSES.len() {
            let mut account = empty();
            let id = account.ensure(class);
            let make_run = |account: &Account| Run {
                class: &data::CLASSES[class],
                race: 0, // No racial multipliers obscure the equipment contract.
                bonus: account.bonus(class, id),
                ..Default::default()
            };
            let baseline = make_run(&account).stats();
            for slot in 0..6 {
                let piece =
                    survivors_core::supplemental_gear::piece(class, slot, GearTier::RaidEpic)
                        .unwrap();
                assert!(account.supplemental.collect(class, &piece.key));
                assert!(account.select_supplemental(class, &piece.key, true));
            }
            account = parse(&encode(&account)).unwrap();
            check(make_run(&account).stats(), baseline);
            let mut items = account.bonus(class, id);
            for slot in 0..6 {
                let piece =
                    survivors_core::supplemental_gear::piece(class, slot, GearTier::RaidEpic)
                        .unwrap();
                assert!(account.select_supplemental(class, &piece.key, false));
                items.add(piece.tier.piece_bonus(slot));
                // Spell out the build contract independently of raid_sets::bonus.
                let mut expected = items;
                if slot >= 1 {
                    match class {
                        0 | 1 => expected.armor += 24.,
                        2 | 3 => expected.speed += 0.04,
                        4 | 8 => expected.health += 30.,
                        _ => expected.pickup += 0.15,
                    }
                }
                if slot >= 3 {
                    match class {
                        0 | 3 => expected.crit += 0.04,
                        1 | 5 | 6 => expected.area += 0.10,
                        2 => expected.multistrike += 0.08,
                        4 | 8 => expected.cooldown += 0.06,
                        7 => expected.leech += 0.02,
                        _ => unreachable!(),
                    }
                }
                if slot >= 5 {
                    match class {
                        0 | 2 | 3 => expected.crit_damage += 0.25,
                        1 | 4 | 8 => expected.reduction += 0.04,
                        5 | 6 => expected.cooldown += 0.08,
                        7 => expected.might += 0.10,
                        _ => unreachable!(),
                    }
                }
                account = parse(&encode(&account)).unwrap();
                let expected_run = Run {
                    bonus: expected,
                    ..make_run(&account)
                };
                check(make_run(&account).stats(), expected_run.stats());
            }
            let snapshot = make_run(&account);
            let before = snapshot.stats();
            for slot in 0..6 {
                assert!(account.wear_piece(class, slot, None));
            }
            account = parse(&encode(&account)).unwrap();
            check(make_run(&account).stats(), baseline);
            check(snapshot.stats(), before);
            assert!(account.classes[class]
                .equipment
                .supplemental_looks()
                .iter()
                .take(6)
                .all(Option::is_some));
        }
    }

    #[test]
    fn supplemental_loadout_rejects_unowned_foreign_and_wrong_slot_save_references() {
        let rows = "supplemental 0 poor:warrior:hands\nsupplemental_worn 0 0 poor:warrior:hands\nsupplemental_worn 1 7 poor:warrior:hands\nsupplemental_worn 0 1 poor:warrior:shoulders\nsupplemental_look 0 7 poor:warrior:hands extra\n";
        let mut account = parse(rows).unwrap();
        for class in &account.classes {
            assert!(class
                .equipment
                .supplemental_worn()
                .iter()
                .all(Option::is_none));
            assert!(class
                .equipment
                .supplemental_looks()
                .iter()
                .all(Option::is_none));
        }
        assert!(!account.select_supplemental(1, "poor:warrior:hands", false));
        assert!(!account.select_supplemental(0, "poor:warrior:shoulders", true));
        assert!(!account.dirty);
    }

    #[test]
    fn supplemental_ownership_round_trips_without_changing_legacy_progress_or_power() {
        let legacy = "who 1 0 5100 Saved Warrior\nclass 0 1 0 1\nequipment 0 100000000 000000000\n";
        let base = parse(legacy).unwrap();
        let rows = "supplemental 0 poor:warrior:hands\nsupplemental 0 poor:warrior:hands\nsupplemental 6 poor:mage:main-hand\nsupplemental 0 poor:mage:hands\nsupplemental 6 raid-epic:mage:main-hand\nsupplemental bad poor:warrior:head\nsupplemental 0 poor:warrior:head extra\n";
        for text in [format!("{rows}{legacy}"), format!("{legacy}{rows}")] {
            let account = parse(&text).unwrap();
            assert_eq!(account.supplemental.keys().count(), 2);
            assert!(account.supplemental.owns(0, "poor:warrior:hands"));
            assert!(account.supplemental.owns(6, "poor:mage:main-hand"));
            assert_eq!(account.classes[0].pieces, base.classes[0].pieces);
            assert_eq!(
                account.classes[0].equipment.encode(),
                base.classes[0].equipment.encode()
            );
            assert_eq!(account.classes[0].clears, base.classes[0].clears);
            assert_eq!(account.bonus(0, 1).health, base.bonus(0, 1).health);
            assert_eq!(account.bonus(0, 1).might, base.bonus(0, 1).might);
            assert_eq!(account.feats(0, 1).collection, base.feats(0, 1).collection);
            assert_eq!(account.earned, base.earned);
            assert!(!account.dirty);
            let saved = encode(&account);
            assert_eq!(
                saved
                    .lines()
                    .filter(|line| line.starts_with("supplemental "))
                    .count(),
                2
            );
            assert_eq!(parse(&saved).unwrap().supplemental, account.supplemental);
        }
        assert_eq!(base.supplemental.keys().count(), 0);
    }

    #[test]
    fn talent_rows_before_characters_preserve_specs_and_run_effects() {
        let characters = "who 1 3 5100 First Rogue\nwho 2 3 5100 Second Rogue\n";
        let talents = "dbc 1 0:270:2,0:270:5|1:187:5|0:270:1\ndbc 2 0:270:3||1:187:2\n";
        let mut ordered = parse(&format!("{characters}{talents}")).unwrap();
        let mut reordered = parse(&format!("{talents}{characters}")).unwrap();
        for id in [1, 2] {
            for spec in 0..3 {
                ordered.select_spec(id, spec);
                reordered.select_spec(id, spec);
                assert_eq!(
                    ordered.character(id).unwrap().dbc,
                    reordered.character(id).unwrap().dbc
                );
                assert_eq!(
                    ordered.milestones_picked(3, Some(id)),
                    reordered.milestones_picked(3, Some(id))
                );
                assert_eq!(ordered.bonus(3, id).armor, reordered.bonus(3, id).armor);
            }
        }
        let round_trip = parse(&encode(&reordered)).unwrap();
        for id in [1, 2] {
            assert_eq!(
                round_trip.character(id).unwrap().dbc,
                reordered.character(id).unwrap().dbc
            );
        }
        let replacement = parse("dbc 0 0:270:1\nwho 0 3 5100 Legacy Rogue\nwho 1 3 5100 Saved Rogue\ndbc 1 0:270:3\ndbc 1 |1:187:2|\n").unwrap();
        assert_eq!(replacement.character(0).unwrap().dbc[0], vec![(0, 270, 1)]);
        assert_eq!(
            replacement.character(1).unwrap().dbc,
            [vec![], vec![(1, 187, 2)], vec![]]
        );
        assert_eq!(replacement.characters.len(), 2);
    }

    #[test]
    fn imported_duplicate_talents_do_not_inflate_tree_points_or_effects() {
        let mut account = parse(
            "who 1 3 5100 Saved Rogue\ndbc 1 0:270:2,0:270:5,0:270:5,1:270:3|1:187:5,1:187:5|0:270:1\n",
        )
        .unwrap();
        let mut untrained = parse(&encode(&account)).unwrap();
        untrained.respec(1);
        let baseline = untrained.bonus(3, 1);
        assert_eq!(account.character(1).unwrap().dbc[0], vec![(0, 270, 5)]);
        assert_eq!(account.character(1).unwrap().dbc_rank(270), 5);
        let active = account.milestones_picked(3, Some(1));
        assert_eq!(active.points, [5, 0, 0]);
        assert_eq!(active, active.with_ranks(&[(0, 270, 5)]));
        assert!((active.cast("sinister_strike").crit - 0.05).abs() < 0.00001);
        assert_eq!(account.bonus(3, 1).crit, baseline.crit);
        assert_eq!(account.bonus(3, 1).might, baseline.might);
        account.select_spec(1, 1);
        assert_eq!(account.milestones_picked(3, Some(1)).points, [0, 5, 0]);
        assert_eq!(account.bonus(3, 1).armor, baseline.armor);
        assert!((account.milestones_picked(3, Some(1)).melee_parry() - 0.05).abs() < 0.00001);
        account.select_spec(1, 2);
        assert_eq!(account.character(1).unwrap().dbc_rank(270), 1);
        assert_eq!(
            parse(&encode(&account)).unwrap().character(1).unwrap().dbc,
            account.character(1).unwrap().dbc
        );
    }

    #[test]
    fn malformed_saved_talent_entries_preserve_the_save_instead_of_dropping_ranks() {
        for entry in [
            "3:270:5",
            "bad:270:5",
            ":270:5",
            "0:bad:5",
            "0:270:bad",
            "0:270",
            "0:270:5:extra",
            "0:0:5",
            "0:270:0",
        ] {
            assert!(
                read_dbc(&format!("{entry},2:244:5||1:187:2")).is_err(),
                "{entry}"
            );
        }
        assert!(read_dbc("0:270:5|||1:187:2").is_err());
        assert_eq!(
            read_dbc("2:244:5||1:187:2").unwrap(),
            [vec![(2, 244, 5)], vec![], vec![(1, 187, 2)]]
        );
        assert_eq!(read_dbc("||").unwrap(), blank_dbc());
        // Unknown identities stay intact for catalog validation, rather than disappearing.
        assert_eq!(
            read_dbc("0:4294967295:255").unwrap()[0],
            vec![(0, u32::MAX, 255)]
        );
    }

    #[test]
    fn imported_overbudget_spec_preserves_ranks_but_grants_no_talent_effects() {
        let mut account = empty();
        let id = account.ensure(3);
        let hero = account.characters.iter_mut().find(|h| h.id == id).unwrap();
        hero.talent_xp = 1000;
        hero.dbc[0] = vec![(0, 276, 3), (0, 270, 5), (0, 272, 2), (0, 273, 1)];
        hero.dbc[1] = vec![(1, 187, 5)];
        let original = hero.dbc.clone();
        let mut loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.character(id).unwrap().dbc, original);
        assert_eq!(loaded.character(id).unwrap().active_points(), 11);
        assert!(!loaded.character(id).unwrap().talent_budget_valid());
        let inactive = loaded.milestones_picked(3, Some(id));
        assert_eq!(inactive.points, [0; 3]);
        assert_eq!(inactive.cast("eviscerate"), Default::default());
        assert_eq!(loaded.bonus(3, id).might, 0.0);
        assert_eq!(loaded.bonus(3, id).armor, 0.0);
        loaded.select_spec(id, 1);
        assert_eq!(loaded.bonus(3, id).armor, 0.0);
        assert!(
            (loaded.milestones_picked(3, Some(id)).melee_parry() - 0.05).abs() < 0.00001,
            "valid sibling spec retains its parry bonus"
        );
        assert_eq!(inactive.melee_parry(), 0.0);
        loaded.select_spec(id, 0);
        assert!(loaded.unlearn_talent(id, 273, true));
        assert!(loaded.character(id).unwrap().talent_budget_valid());
        assert_eq!(loaded.milestones_picked(3, Some(id)).points, [10, 0, 0]);
        assert!(
            loaded
                .milestones_picked(3, Some(id))
                .cast("eviscerate")
                .damage
                > 1.0
        );
        assert_eq!(inactive.points, [0; 3], "running snapshot is unchanged");
        assert_eq!(
            parse(&encode(&loaded)).unwrap().character(id).unwrap().dbc[0].len(),
            3
        );
        // Earning the missing point also activates the untouched original spec.
        account
            .characters
            .iter_mut()
            .find(|h| h.id == id)
            .unwrap()
            .talent_xp = 1100;
        assert_eq!(account.milestones_picked(3, Some(id)).points, [11, 0, 0]);
    }

    #[test]
    fn named_stat_talents_change_the_selected_sheet_and_survive_spec_switches() {
        let mut account = empty();
        let id = account.ensure(1);
        let other = account.create("Other", "Paladin", 1).unwrap();
        let baseline = account.bonus(1, id);
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == id)
            .unwrap();
        hero.talent_xp = 1000;
        hero.dbc[0] = vec![(1, 1423, 5)]; // Toughness: armor, not generic damage.
        hero.dbc[1] = vec![(0, 1450, 5)]; // Divine Strength: damage.
        let armor = account.bonus(1, id);
        assert_eq!(armor.armor - baseline.armor, 30.0);
        assert!((armor.might - baseline.might).abs() < 0.00001);
        assert_eq!(account.bonus(1, other).armor, baseline.armor);
        let mut run = super::super::Run::default();
        let base_sheet = run.stats();
        run.bonus = armor;
        assert_eq!(run.stats().armor - base_sheet.armor, armor.armor);
        account.select_spec(id, 1);
        let strength = account.bonus(1, id);
        assert_eq!(strength.armor, baseline.armor);
        assert!((strength.might - baseline.might - 0.10).abs() < 0.00001);
        let loaded = parse(&encode(&account)).unwrap();
        assert!((loaded.bonus(1, id).might - strength.might).abs() < 0.00001);
        account.respec(id);
        assert_eq!(account.bonus(1, id).might, baseline.might);
        account.select_spec(id, 0);
        assert_eq!(account.bonus(1, id).armor, armor.armor);
    }

    #[test]
    fn priest_defenses_reach_combat_and_remain_independent_per_saved_spec() {
        let mut account = empty();
        let id = account.ensure(4);
        let other = account.create("Other", "Priest", 4).unwrap();
        let baseline = account.bonus(4, id);
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == id)
            .unwrap();
        hero.talent_xp = 1000;
        hero.dbc[0] = vec![(0, 344, 2), (0, 346, 3), (1, 411, 5)];
        hero.dbc[1] = vec![(0, 344, 1)];
        let defenses = account.bonus(4, id);
        assert_eq!(defenses.health - baseline.health, 20.0);
        assert_eq!(defenses.armor - baseline.armor, 6.0);
        assert!((defenses.reduction - baseline.reduction - 0.10).abs() < 0.00001);
        assert!((defenses.might - baseline.might).abs() < 0.00001);
        let run = super::super::Run {
            bonus: defenses,
            ..Default::default()
        };
        let base = super::super::Run {
            bonus: baseline,
            ..Default::default()
        }
        .stats();
        let stats = run.stats();
        assert_eq!(stats.max_hp - base.max_hp, 20.0);
        assert_eq!(stats.armor - base.armor, 6.0);
        assert!(
            (survivors_core::sheet::mitigate(50.0, stats.armor, stats.reduction)
                - (50.0 - stats.armor) * (1.0 - stats.reduction))
                .abs()
                < 0.00001
        );
        assert_eq!(account.bonus(4, other).health, baseline.health);
        account.select_spec(id, 1);
        let alternate = account.bonus(4, id);
        assert_eq!(alternate.health - baseline.health, 10.0);
        assert_eq!(alternate.armor, baseline.armor);
        assert_eq!(alternate.reduction, baseline.reduction);
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.bonus(4, id).health, alternate.health);
        account.select_spec(id, 0);
        assert_eq!(account.bonus(4, id).armor, defenses.armor);
        account.respec(id);
        assert_eq!(account.bonus(4, id).health, baseline.health);
        assert_eq!(
            run.stats().max_hp,
            stats.max_hp,
            "active run keeps its snapshot"
        );
    }

    #[test]
    fn milestone_snapshot_uses_the_selected_hero_and_spec_and_survives_save() {
        let mut account = empty();
        let first = account.ensure(1);
        let second = account.create("Uther", "Lightbringer", 1).unwrap();
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap();
        hero.talent_xp = 2000;
        hero.dbc[0] = vec![(0, 99, 5), (0, 100, 5)];
        hero.dbc[1] = vec![(2, 101, 5), (2, 102, 5), (2, 103, 5), (2, 104, 5)];
        hero.spec = 1;
        let snapshot = account.milestones_picked(1, Some(second));
        assert_eq!(snapshot.points, [0, 0, 20]);
        assert_eq!(account.milestones_picked(1, Some(first)).points, [0; 3]);
        assert_eq!(account.milestones_picked(0, Some(second)).points, [0; 3]);
        let saved = encode(&account);
        let loaded = parse(&saved).unwrap();
        assert_eq!(loaded.milestones_picked(1, Some(second)), snapshot);
        account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap()
            .spec = 0;
        assert_eq!(
            account.milestones_picked(1, Some(second)).points,
            [10, 0, 0]
        );
        assert_eq!(snapshot.points, [0, 0, 20]);
    }

    #[test]
    fn spell_training_snapshot_is_selected_persistent_and_independent_of_later_edits() {
        let mut account = empty();
        let first = account.ensure(1);
        let second = account.create("Uther", "Lightbringer", 1).unwrap();
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap();
        hero.talent_xp = 300;
        hero.dbc[0] = vec![(2, 1631, 2)];
        hero.dbc[1] = vec![(1, 1521, 3)];
        hero.spec = 1;
        let snapshot = account.milestones_picked(1, Some(second));
        assert!((snapshot.cast("hammer_of_justice").cooldown - 0.7).abs() < 0.00001);
        assert_eq!(snapshot.cast("judgement").cooldown, 1.0);
        assert_eq!(
            account
                .milestones_picked(1, Some(first))
                .cast("hammer_of_justice")
                .cooldown,
            1.0
        );
        assert_eq!(
            account
                .milestones_picked(0, Some(second))
                .cast("hammer_of_justice")
                .cooldown,
            1.0
        );
        assert_eq!(
            parse(&encode(&account))
                .unwrap()
                .milestones_picked(1, Some(second)),
            snapshot
        );
        account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap()
            .spec = 0;
        let switched = account.milestones_picked(1, Some(second));
        assert!((switched.cast("judgement").cooldown - 0.8).abs() < 0.00001);
        assert_eq!(switched.cast("hammer_of_justice").cooldown, 1.0);
        account.respec(second);
        assert_eq!(
            account
                .milestones_picked(1, Some(second))
                .cast("judgement")
                .cooldown,
            1.0
        );
        account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap()
            .spec = 1;
        assert_eq!(account.milestones_picked(1, Some(second)), snapshot);
        assert!((snapshot.cast("hammer_of_justice").cooldown - 0.7).abs() < 0.00001);
    }

    #[test]
    fn shaman_enhancement_stats_survive_reload_spec_switch_and_reset_into_run_stats() {
        let mut account = empty();
        let first = account.ensure(5);
        let second = account.create("Mura", "Training", 5).unwrap();
        let baseline = vec![
            (1, 614, 5),
            (1, 613, 5),
            (1, 605, 2),
            (1, 607, 3),
            (1, 602, 5),
        ];
        let hero = account
            .characters
            .iter_mut()
            .find(|h| h.id == second)
            .unwrap();
        hero.talent_xp = 3000;
        hero.dbc[0] = baseline.clone();
        hero.dbc[1] = baseline;
        hero.dbc[1].extend([(1, 601, 5), (1, 1643, 4)]);
        let make_run = |account: &Account, id| super::super::Run {
            class: &data::CLASSES[5],
            bonus: account.bonus(5, id),
            milestones: account.milestones_picked(5, Some(id)),
            ..Default::default()
        };
        let base = make_run(&account, second).stats();
        account.select_spec(second, 1);
        let run = make_run(&account, second);
        let trained = run.stats();
        assert!((trained.might - base.might - 0.08).abs() < 0.00001);
        assert!((trained.reduction - base.reduction - 0.05).abs() < 0.00001);
        assert_eq!(trained.targets, base.targets);
        assert_eq!(trained.cooldown, base.cooldown);
        assert_eq!(trained.multistrike, base.multistrike);
        assert_eq!(make_run(&account, first).stats().reduction, 0.);
        assert_eq!(account.bonus(0, second).reduction, 0.);
        let mut restored = parse(&encode(&account)).unwrap();
        assert_eq!(make_run(&restored, second).stats().might, trained.might);
        assert_eq!(
            make_run(&restored, second).stats().reduction,
            trained.reduction
        );
        restored.select_spec(second, 0);
        assert_eq!(make_run(&restored, second).stats().might, base.might);
        assert_eq!(
            make_run(&restored, second).stats().reduction,
            base.reduction
        );
        restored.select_spec(second, 1);
        restored.respec(second);
        let reset = make_run(&restored, second).stats();
        let untrained = make_run(&restored, first).stats();
        assert_eq!(reset.might, untrained.might);
        assert_eq!(reset.reduction, untrained.reduction);
        assert_eq!(run.stats().might, trained.might);
        assert_eq!(run.stats().reduction, trained.reduction);
        let reloaded = parse(&encode(&restored)).unwrap();
        assert_eq!(make_run(&reloaded, second).stats().might, untrained.might);
        assert_eq!(
            make_run(&reloaded, second).stats().reduction,
            untrained.reduction
        );
    }

    #[test]
    fn restoration_fixture_reaches_cast_parameters_after_reload_and_spec_changes() {
        use super::super::{abilities::Params, Owned, Run};
        let mut account = parse(include_str!("../../testdata/survivors-ui/trained.txt")).unwrap();
        let make_run = |account: &Account, id| Run {
            class: &data::CLASSES[5],
            bonus: account.bonus(5, id),
            milestones: account.milestones_picked(5, Some(id)),
            ..Default::default()
        };
        let snapshot = make_run(&account, 2);
        assert_eq!(snapshot.milestones.points, [0, 0, 13]);
        let check = |run: &Run, trained: bool| {
            for key in ["searing_totem", "magma_totem", "lightning_bolt"] {
                let owned = Owned {
                    def: data::ability(key),
                    level: 3,
                    cd: 0.,
                };
                let stats = run.stats();
                let params = Params::new(&owned, &stats, run.milestones);
                let totem = key != "lightning_bolt";
                let expected = owned.cooldown(&stats)
                    * if trained && totem {
                        0.84 * 0.92 * 0.85
                    } else {
                        1.
                    };
                assert!((params.cooldown - expected).abs() < 0.00001, "{key}");
                assert_eq!(
                    params.heal_fraction,
                    if trained && totem { 0.03 } else { 0. }
                );
                assert_eq!(params.radius, owned.radius(&stats));
                assert_eq!(params.duration, owned.duration());
            }
        };
        check(&snapshot, true);
        account = parse(&encode(&account)).unwrap();
        check(&make_run(&account, 2), true);
        account.select_spec(2, 2);
        check(&make_run(&account, 2), false);
        account.select_spec(2, 0);
        check(&make_run(&account, 2), true);
        account.respec(2);
        check(&make_run(&account, 2), false);
        // The other hero keeps its own Restoration build, and the active run is immutable.
        account.select_spec(1, 1);
        check(&make_run(&account, 1), true);
        check(&snapshot, true);
        let restored = parse(&encode(&account)).unwrap();
        check(&make_run(&restored, 2), false);
        check(&make_run(&restored, 1), true);
    }

    #[test]
    fn caster_accuracy_purchases_persist_and_refund_without_global_damage() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let rules = app
            .world()
            .resource::<Account>()
            .talent_rules
            .clone()
            .unwrap();
        for (class, tree, id, cap, row, ranks, key, rate) in [
            (
                2,
                2,
                1322,
                2,
                3,
                vec![(2, 1311, 5), (2, 1304, 5), (2, 1622, 3), (2, 1306, 2)],
                "explosive_trap",
                0.05,
            ),
            (6, 0, 74, 2, 0, vec![], "arcane_missiles", 0.05),
            (
                4,
                2,
                463,
                5,
                1,
                vec![(2, 465, 5), (2, 464, 5)],
                "mind_blast",
                0.02,
            ),
            (
                5,
                2,
                583,
                3,
                2,
                vec![(2, 586, 5), (2, 593, 5)],
                "lightning_bolt",
                0.01,
            ),
            (7, 0, 1005, 5, 0, vec![], "corruption", 0.02),
        ] {
            let mut account = empty();
            account.talent_rules = Some(rules.clone());
            let hero = account.ensure(class);
            let character = account
                .characters
                .iter_mut()
                .find(|c| c.id == hero)
                .unwrap();
            character.talent_xp = 5100;
            character.dbc[0] = ranks;
            assert!(account.talents_valid(account.character(hero).unwrap()));
            let base = account
                .milestones_picked(class, Some(hero))
                .cast(key)
                .damage;
            let before = account.bonus(class, hero);
            for _ in 0..cap {
                assert!(account.learn_talent(hero, tree, id, cap, row, true));
            }
            assert!(!account.learn_talent(hero, tree, id, cap, row, true));
            assert!((account.bonus(class, hero).might - before.might).abs() < 0.00001);
            let damage = |account: &Account| {
                account
                    .milestones_picked(class, Some(hero))
                    .cast(key)
                    .damage
            };
            let expected = base * (1. + rate * f32::from(cap));
            assert!((damage(&account) - expected).abs() < 0.00001);
            let mut loaded = parse(&encode(&account)).unwrap();
            loaded.talent_rules = Some(rules.clone());
            assert!((damage(&loaded) - expected).abs() < 0.00001);
            loaded.select_spec(hero, 1);
            assert_eq!(damage(&loaded), 1.);
            loaded.select_spec(hero, 0);
            assert!((damage(&loaded) - expected).abs() < 0.00001);
            for _ in 0..cap {
                assert!(loaded.unlearn_talent(hero, id, true));
            }
            assert_eq!(damage(&loaded), base);
        }
    }

    #[test]
    fn repentance_unlock_tracks_catalog_rows_saved_specs_refunds_and_run_snapshots() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(1);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![];
        assert!(!account.learn_talent(hero, 2, 1441, 1, 6, true));
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.dbc[0] = vec![
            (2, 1401, 5),
            (2, 1407, 5),
            (2, 1403, 5),
            (2, 1411, 5),
            (2, 1631, 2),
            (2, 1634, 2),
            (2, 1409, 1),
            (2, 1402, 5),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert!(!account
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        assert!(account.learn_talent(hero, 2, 1441, 1, 6, true));
        assert!(!account.learn_talent(hero, 2, 1441, 1, 6, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let snapshot = account.milestones_picked(1, Some(hero));
        assert!(snapshot.ability_unlocked("repentance"));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!(loaded.talents_valid(loaded.character(hero).unwrap()));
        assert!(loaded
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        loaded.select_spec(hero, 1);
        assert!(!loaded
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        loaded.select_spec(hero, 0);
        assert!(loaded
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        let mut invalid = loaded.character(hero).unwrap().clone();
        invalid.dbc[0]
            .iter_mut()
            .find(|entry| entry.1 == 1402)
            .unwrap()
            .2 -= 1;
        assert!(
            !loaded.talents_valid(&invalid),
            "a trained capstone still needs 30 earlier points"
        );
        assert!(!loaded.unlearn_talent(hero, 1402, false));
        assert!(loaded.unlearn_talent(hero, 1441, true));
        assert!(!loaded
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        assert_eq!(
            loaded.character(hero).unwrap().dbc[0]
                .iter()
                .map(|entry| u32::from(entry.2))
                .sum::<u32>(),
            30
        );
        let refunded = parse(&encode(&loaded)).unwrap();
        assert!(!refunded
            .milestones_picked(1, Some(hero))
            .ability_unlocked("repentance"));
        assert!(
            snapshot.ability_unlocked("repentance"),
            "an existing run retains its captured spec"
        );
    }

    #[test]
    fn hawk_purchase_saved_specs_and_refund_match_runtime() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(2, hero);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 0, 1382, 5, 0, true));
        }
        assert!(!account.learn_talent(hero, 0, 1382, 5, 0, true));
        assert_eq!(account.bonus(2, hero).reduction, before.reduction);
        let factor = |account: &Account, elapsed| {
            let talents = account.milestones_picked(2, Some(hero));
            let mut clock = survivors_core::talents::HawkClock::default();
            clock.proc(talents, "arcane_shot", 0., 0.);
            clock.haste(talents, "arcane_shot", elapsed)
        };
        assert!((factor(&account, 0.) - 0.15).abs() < 0.00001);
        assert_eq!(factor(&account, 12.), 0.);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((factor(&loaded, 0.) - 0.15).abs() < 0.00001);
        loaded.select_spec(hero, 1);
        assert_eq!(factor(&loaded, 0.), 0.);
        loaded.select_spec(hero, 0);
        assert!((factor(&loaded, 0.) - 0.15).abs() < 0.00001);
        for _ in 0..5 {
            assert!(loaded.unlearn_talent(hero, 1382, true));
        }
        assert_eq!(factor(&loaded, 0.), 0.);
    }

    #[test]
    fn deterrence_purchase_saved_specs_and_refund_match_runtime() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(2, 1304, 5), (2, 1311, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(2, hero);
        assert!(account.learn_talent(hero, 2, 1308, 1, 2, true));
        assert!(!account.learn_talent(hero, 2, 1308, 1, 2, true));
        assert_eq!(account.bonus(2, hero).reduction, before.reduction);
        let factor = |account: &Account, elapsed| {
            account
                .milestones_picked(2, Some(hero))
                .deterrence_avoidance(elapsed)
        };
        assert_eq!(factor(&account, 0.), 0.25);
        assert_eq!(factor(&account, 10.), 0.);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(factor(&loaded, 180.), 0.25);
        loaded.select_spec(hero, 1);
        assert_eq!(factor(&loaded, 180.), 0.);
        loaded.select_spec(hero, 0);
        assert_eq!(factor(&loaded, 180.), 0.25);
        assert!(loaded.unlearn_talent(hero, 1308, true));
        assert_eq!(factor(&loaded, 180.), 0.);
    }

    #[test]
    fn counterattack_purchase_requires_deterrence_and_survives_saved_spec_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (2, 1311, 5),
            (2, 1304, 5),
            (2, 1622, 3),
            (2, 1306, 2),
            (2, 1322, 2),
            (2, 1305, 3),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert!(!account.learn_talent(hero, 2, 1312, 1, 4, false));
        assert!(account.learn_talent(hero, 2, 1308, 1, 2, true));
        let reduction = account.bonus(2, hero).reduction;
        assert!(account.learn_talent(hero, 2, 1312, 1, 4, true));
        assert_eq!(account.bonus(2, hero).reduction, reduction);
        let trained = account.milestones_picked(2, Some(hero));
        assert_eq!(trained.counterattack_damage(), 40.);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .counterattack_damage(),
            40.
        );
        loaded.select_spec(hero, 1);
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .counterattack_damage(),
            0.
        );
        loaded.select_spec(hero, 0);
        let mut invalid = loaded.character(hero).unwrap().clone();
        invalid.dbc[0].retain(|entry| entry.1 != 1308);
        let allowed = loaded.talents_valid(&invalid);
        assert!(!allowed, "catalog retains Counterattack prerequisite");
        assert!(!loaded.unlearn_talent(hero, 1308, allowed));
        assert!(loaded.unlearn_talent(hero, 1312, true));
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .counterattack_damage(),
            0.
        );
        assert!(loaded.unlearn_talent(hero, 1308, true));
        assert_eq!(
            trained.counterattack_damage(),
            40.,
            "captured run retains selected build"
        );
    }

    #[test]
    fn hunter_melee_metadata_and_savage_strikes_survive_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        for (id, name) in [(2973, "Raptor Strike"), (1495, "Mongoose Bite")] {
            assert_eq!(spells.get(id).unwrap().name, name);
        }
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(2, 1311, 5)];
        let before = account.bonus(2, hero);
        for _ in 0..2 {
            assert!(account.learn_talent(hero, 2, 1621, 2, 1, true));
        }
        assert!(!account.learn_talent(hero, 2, 1621, 2, 1, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert_eq!(account.bonus(2, hero).crit, before.crit);
        let crit = |account: &Account, key| account.milestones_picked(2, Some(hero)).cast(key).crit;
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        for key in ["raptor_strike", "mongoose_bite"] {
            assert_eq!(crit(&loaded, key), 0.2);
        }
        assert_eq!(crit(&loaded, "arcane_shot"), 0.0);
        loaded.select_spec(hero, 1);
        assert_eq!(crit(&loaded, "raptor_strike"), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 1621, true));
        assert_eq!(crit(&loaded, "mongoose_bite"), 0.1);
        assert!(loaded.unlearn_talent(hero, 1621, true));
        assert_eq!(crit(&loaded, "raptor_strike"), 0.0);
    }

    #[test]
    fn wing_clip_metadata_training_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        assert_eq!(spells.get(2974).unwrap().name, "Wing Clip");
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(2, 1311, 5)];
        let before = account.bonus(2, hero);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 2, 1305, 5, 1, true));
        }
        assert!(!account.learn_talent(hero, 2, 1305, 5, 1, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert_eq!(account.bonus(2, hero).crit, before.crit);
        let chance = |account: &Account, key| {
            account
                .milestones_picked(2, Some(hero))
                .wing_clip_root_chance(key, 5.)
        };
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((chance(&loaded, "wing_clip") - 0.2).abs() < 0.00001);
        assert_eq!(chance(&loaded, "arcane_shot"), 0.0);
        loaded.select_spec(hero, 1);
        assert_eq!(chance(&loaded, "wing_clip"), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 1305, true));
        assert_eq!(chance(&loaded, "wing_clip"), 0.16);
        assert!(loaded.unlearn_talent(hero, 1305, true));
        assert_eq!(chance(&loaded, "wing_clip"), 0.12);
    }

    #[test]
    fn natures_grace_metadata_training_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        assert_eq!(spells.get(16880).unwrap().name, "Nature's Grace");
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(8);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (0, 762, 5),
            (0, 761, 1),
            (0, 921, 4),
            (0, 763, 5),
            (0, 782, 3),
            (0, 764, 2),
        ];
        let before = account.bonus(8, hero);
        assert!(account.learn_talent(hero, 0, 789, 1, 4, true));
        assert!(!account.learn_talent(hero, 0, 789, 1, 4, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert_eq!(account.bonus(8, hero).crit, before.crit);

        let charged = |account: &Account| {
            let talents = account.milestones_picked(8, Some(hero));
            let mut grace = survivors_core::talents::NaturesGrace::default();
            grace.hit(talents, "moonfire", 10., true, 0.);
            grace.recovery(talents, "wrath", 0., 2.) == 1.5
        };
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!(charged(&loaded));
        loaded.select_spec(hero, 1);
        assert!(!charged(&loaded));
        loaded.select_spec(hero, 0);
        assert!(charged(&loaded));
        assert!(loaded.unlearn_talent(hero, 789, true));
        assert!(!charged(&loaded));
    }

    #[test]
    fn insect_swarm_metadata_training_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        assert_eq!(spells.get(5570).unwrap().name, "Insect Swarm");
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(8);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(2, 821, 5), (2, 824, 5)];
        let before = account.bonus(8, hero);
        assert!(account.learn_talent(hero, 2, 827, 1, 2, true));
        assert!(!account.learn_talent(hero, 2, 827, 1, 2, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert_eq!(account.bonus(8, hero).crit, before.crit);

        let weakened = |account: &Account| {
            matches!(
                account
                    .milestones_picked(8, Some(hero))
                    .control("insect_swarm", super::super::data::Effect::None),
                super::super::data::Effect::Weaken(0.98, 10.)
            )
        };
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!(weakened(&loaded));
        loaded.select_spec(hero, 1);
        assert!(!weakened(&loaded));
        loaded.select_spec(hero, 0);
        assert!(weakened(&loaded));
        assert!(loaded.unlearn_talent(hero, 827, true));
        assert!(!weakened(&loaded));
    }

    #[test]
    fn scorpid_metadata_purchase_saved_specs_and_refund_match_runtime() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        assert_eq!(spells.get(3043).unwrap().name, "Scorpid Sting");
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 1344, 5), (1, 1341, 5), (1, 1346, 5), (1, 1348, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(2, hero);
        for _ in 0..3 {
            assert!(account.learn_talent(hero, 1, 1351, 3, 4, true));
        }
        assert!(!account.learn_talent(hero, 1, 1351, 3, 4, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        assert_eq!(account.bonus(2, hero).reduction, before.reduction);
        let factor = |account: &Account| {
            account
                .milestones_picked(2, Some(hero))
                .cast("scorpid_sting")
                .duration
        };
        assert!((factor(&account) - 1.6).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((factor(&loaded) - 1.6).abs() < 0.00001);
        loaded.select_spec(hero, 1);
        assert_eq!(factor(&loaded), 1.);
        loaded.select_spec(hero, 0);
        assert!((factor(&loaded) - 1.6).abs() < 0.00001);
        for _ in 0..3 {
            assert!(loaded.unlearn_talent(hero, 1351, true));
        }
        assert_eq!(factor(&loaded), 1.);
    }

    #[test]
    fn mana_shield_metadata_purchase_and_saved_specs_match_the_runtime() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        assert_eq!(spells.get(1463).unwrap().name, "Mana Shield");
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(6);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(0, 76, 5), (0, 80, 5), (0, 1650, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(6, hero);
        for _ in 0..2 {
            assert!(account.learn_talent(hero, 0, 83, 2, 3, true));
        }
        assert!(!account.learn_talent(hero, 0, 83, 2, 3, true));
        assert!((account.bonus(6, hero).might - before.might).abs() < 0.00001);
        let bonus = |account: &Account| {
            account
                .milestones_picked(6, Some(hero))
                .cast("mana_shield")
                .cooldown
        };
        assert_eq!(bonus(&account), 0.8);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(bonus(&loaded), 0.8);
        loaded.select_spec(hero, 1);
        assert_eq!(bonus(&loaded), 1.0);
        loaded.select_spec(hero, 0);
        assert_eq!(bonus(&loaded), 0.8);
        for _ in 0..2 {
            assert!(loaded.unlearn_talent(hero, 83, true));
        }
        assert_eq!(bonus(&loaded), 1.0);
    }

    #[test]
    fn presence_purchase_persists_and_refund_retains_only_the_milestone() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(6);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(0, 76, 5), (0, 80, 5), (0, 1650, 5), (0, 75, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(6, hero);
        assert!(account.learn_talent(hero, 0, 86, 1, 4, true));
        assert!(!account.learn_talent(hero, 0, 86, 1, 4, true));
        assert!((account.bonus(6, hero).might - before.might).abs() < 0.00001);
        let bonus = |account: &Account| {
            account
                .milestones_picked(6, Some(hero))
                .cast("arcane_missiles")
                .cooldown_advance
        };
        assert_eq!(bonus(&account), Some(("arcane_explosion", 4.)));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(bonus(&loaded), Some(("arcane_explosion", 4.)));
        loaded.select_spec(hero, 1);
        assert_eq!(bonus(&loaded), None);
        loaded.select_spec(hero, 0);
        assert_eq!(bonus(&loaded), Some(("arcane_explosion", 4.)));
        assert!(loaded.unlearn_talent(hero, 86, true));
        assert_eq!(bonus(&loaded), Some(("arcane_explosion", 2.)));
    }

    #[test]
    fn combustion_purchase_persists_and_spec_refund_remove_the_burst() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(6);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 26, 5),
            (1, 30, 5),
            (1, 34, 5),
            (1, 28, 2),
            (1, 27, 3),
            (1, 31, 3),
            (1, 1141, 2),
            (1, 33, 3),
            (1, 35, 2),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(6, hero);
        assert!(account.learn_talent(hero, 1, 36, 1, 6, true));
        assert!(!account.learn_talent(hero, 1, 36, 1, 6, true));
        assert!((account.bonus(6, hero).might - before.might).abs() < 0.00001);
        let bonus = |account: &Account| {
            survivors_core::talents::CombustionClock::default().bonus(
                account.milestones_picked(6, Some(hero)),
                "fireball",
                0.,
            )
        };
        assert_eq!(bonus(&account), 0.1);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(bonus(&loaded), 0.1);
        loaded.select_spec(hero, 1);
        assert_eq!(bonus(&loaded), 0.);
        loaded.select_spec(hero, 0);
        assert_eq!(bonus(&loaded), 0.1);
        assert!(loaded.unlearn_talent(hero, 36, true));
        assert_eq!(bonus(&loaded), 0.);
    }

    #[test]
    fn inspiration_purchase_persists_and_spec_refund_restore_armor() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(4);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 401, 5), (1, 1181, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(4, hero);
        for _ in 0..3 {
            assert!(account.learn_talent(hero, 1, 361, 3, 2, true));
        }
        assert!(!account.learn_talent(hero, 1, 361, 3, 2, true));
        assert!((account.bonus(4, hero).might - before.might).abs() < 0.00001);
        let armor = |account: &Account| {
            account
                .milestones_picked(4, Some(hero))
                .inspiration_multiplier(true)
        };
        assert!((armor(&account) - 1.24).abs() < 0.0001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((armor(&loaded) - 1.24).abs() < 0.0001);
        loaded.select_spec(hero, 1);
        assert_eq!(armor(&loaded), 1.);
        loaded.select_spec(hero, 0);
        assert!((armor(&loaded) - 1.24).abs() < 0.0001);
        for _ in 0..3 {
            assert!(loaded.unlearn_talent(hero, 361, true));
        }
        assert_eq!(armor(&loaded), 1.);
    }

    #[test]
    fn blessed_recovery_purchase_persists_and_spec_refund_restore_healing() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(4);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 401, 5), (1, 1181, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(4, hero);
        for _ in 0..3 {
            assert!(account.learn_talent(hero, 1, 1636, 3, 2, true));
        }
        assert!(!account.learn_talent(hero, 1, 1636, 3, 2, true));
        assert!((account.bonus(4, hero).might - before.might).abs() < 0.00001);
        let healing = |account: &Account| {
            let talents = account.milestones_picked(4, Some(hero));
            let mut recovery = survivors_core::talents::BlessedRecovery::default();
            recovery.hit(talents, 100.);
            recovery.advance(talents, 6.)
        };
        assert!((healing(&account) - 21.).abs() < 0.0001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((healing(&loaded) - 21.).abs() < 0.0001);
        loaded.select_spec(hero, 1);
        assert_eq!(healing(&loaded), 0.);
        loaded.select_spec(hero, 0);
        assert!((healing(&loaded) - 21.).abs() < 0.0001);
        for _ in 0..3 {
            assert!(loaded.unlearn_talent(hero, 1636, true));
        }
        assert_eq!(healing(&loaded), 0.);
    }

    #[test]
    fn sanctuary_purchase_persists_and_spec_refund_restore_damage() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(1);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 1422, 5), (1, 1421, 5), (1, 1423, 5), (1, 1629, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(1, hero);
        assert!(account.learn_talent(hero, 1, 1431, 1, 4, true));
        assert!(!account.learn_talent(hero, 1, 1431, 1, 4, true));
        assert!((account.bonus(1, hero).might - before.might).abs() < 0.00001);
        let damage = |account: &Account| {
            account
                .milestones_picked(1, Some(hero))
                .after_flat_absorption(10.)
        };
        assert_eq!(damage(&account), 8.);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(damage(&loaded), 8.);
        loaded.select_spec(hero, 1);
        assert_eq!(damage(&loaded), 10.);
        loaded.select_spec(hero, 0);
        assert_eq!(damage(&loaded), 8.);
        assert!(loaded.unlearn_talent(hero, 1431, true));
        assert_eq!(damage(&loaded), 10.);
    }

    #[test]
    fn endurance_purchase_persists_and_spec_refund_restore_sprint() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 203, 3), (1, 201, 2), (1, 181, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(3, hero);
        for _ in 0..2 {
            assert!(account.learn_talent(hero, 1, 204, 2, 2, true));
        }
        assert!(!account.learn_talent(hero, 1, 204, 2, 2, true));
        assert!((account.bonus(3, hero).might - before.might).abs() < 0.00001);
        let cooldown = |account: &Account| {
            let run = super::super::Run {
                class: &data::CLASSES[3],
                level: 5,
                milestones: account.milestones_picked(3, Some(hero)),
                ..Default::default()
            };
            run.mobility().unwrap().cooldown
        };
        assert_eq!(cooldown(&account), 12.0);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(cooldown(&loaded), 12.0);
        loaded.select_spec(hero, 1);
        assert_eq!(cooldown(&loaded), 15.0);
        loaded.select_spec(hero, 0);
        assert_eq!(cooldown(&loaded), 12.0);
        assert!(loaded.unlearn_talent(hero, 204, true));
        assert_eq!(cooldown(&loaded), 13.5);
        assert!(loaded.unlearn_talent(hero, 204, true));
        assert_eq!(cooldown(&loaded), 15.0);
    }

    #[test]
    fn improved_fade_purchase_persists_and_spec_refund_restore_escape() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(4);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(2, 465, 5), (2, 464, 5), (2, 481, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(4, hero);
        for _ in 0..2 {
            assert!(account.learn_talent(hero, 2, 483, 2, 3, true));
        }
        assert!(!account.learn_talent(hero, 2, 483, 2, 3, true));
        assert!((account.bonus(4, hero).might - before.might).abs() < 0.00001);
        let cooldown = |account: &Account| {
            let run = super::super::Run {
                class: &data::CLASSES[4],
                level: 5,
                milestones: account.milestones_picked(4, Some(hero)),
                ..Default::default()
            };
            run.mobility().unwrap().cooldown
        };
        assert!((cooldown(&account) - 14.4).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert!((cooldown(&loaded) - 14.4).abs() < 0.00001);
        loaded.select_spec(hero, 1);
        assert!((cooldown(&loaded) - 18.0).abs() < 0.00001);
        loaded.select_spec(hero, 0);
        assert!((cooldown(&loaded) - 14.4).abs() < 0.00001);
        assert!(loaded.unlearn_talent(hero, 483, true));
        assert!((cooldown(&loaded) - 16.2).abs() < 0.00001);
        assert!(loaded.unlearn_talent(hero, 483, true));
        assert!((cooldown(&loaded) - 18.0).abs() < 0.00001);
    }

    #[test]
    fn rogue_sword_training_tracks_saved_worn_gear_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 201, 2),
            (1, 203, 3),
            (1, 186, 5),
            (1, 181, 5),
            (1, 182, 5),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        let chance = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .sword_extra_attack_chance("sinister_strike", 10.0)
        };
        let base = chance(&account);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 242, 5, 4, true));
        }
        assert!(!account.learn_talent(hero, 1, 242, 5, 4, true));
        assert!((chance(&account) - base - 0.05).abs() < 0.00001);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(0)
        ));
        assert!((chance(&account) - base - 0.05).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(chance(&loaded), chance(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(chance(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 242, true));
        assert!((chance(&loaded) - base - 0.04).abs() < 0.00001);
        let progress = &mut loaded.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(2)
        ));
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(0)));
        assert_eq!(chance(&loaded), base);
        assert_eq!(loaded.milestones_picked(3, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn backstab_training_persists_and_cosmetic_daggers_do_not_enable_casts() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;

        character.dbc[0] = vec![(1, 201, 2), (1, 203, 3)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(0)));
        for _ in 0..3 {
            assert!(account.learn_talent(hero, 1, 202, 3, 1, true));
        }
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 2, 261, 5, 0, true));
        }
        assert!(!account.learn_talent(hero, 1, 202, 3, 1, true));
        assert!(!account.learn_talent(hero, 2, 261, 5, 0, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let effect = |account: &Account| account.milestones_picked(3, Some(hero)).cast("backstab");
        assert!((effect(&account).crit - 0.3).abs() < 0.00001);
        assert!((effect(&account).damage - 1.2).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(effect(&loaded).crit, effect(&account).crit);
        assert_eq!(effect(&loaded).damage, effect(&account).damage);
        let usable = |account: &Account| {
            super::super::data::ability("backstab")
                .usable_with(account.milestones_picked(3, Some(hero)).weapon)
        };
        assert!(usable(&loaded));
        loaded.select_spec(hero, 1);
        assert_eq!(effect(&loaded).crit, 0.);
        assert_eq!(effect(&loaded).damage, 1.);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 202, true));
        assert!(loaded.unlearn_talent(hero, 261, true));
        assert!((effect(&loaded).crit - 0.2).abs() < 0.00001);
        assert!((effect(&loaded).damage - 1.16).abs() < 0.00001);
        let progress = &mut loaded.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(0)
        ));
        assert!(!usable(&loaded));
    }

    #[test]
    fn monkey_training_requires_legal_points_and_roundtrips_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap()
            .talent_xp = 5100;
        assert!(!account.learn_talent(hero, 0, 1381, 5, 1, true));
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 0, 1382, 5, 0, true));
        }
        for rank in 1..=5 {
            assert!(account.learn_talent(hero, 0, 1381, 5, 1, true));
            let t = account.milestones_picked(2, Some(hero));
            assert!((t.monkey_dodge(1, 0.) - (0.08 + 0.01 * rank as f32)).abs() < 0.00001);
            assert_eq!(
                t.monkey_dodge(0, 0.),
                0.,
                "talent does not grant the aspect"
            );
        }
        let snapshot = account.milestones_picked(2, Some(hero));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(
            loaded.milestones_picked(2, Some(hero)).monkey_dodge(1, 0.),
            snapshot.monkey_dodge(1, 0.)
        );
        loaded.select_spec(hero, 1);
        assert_eq!(
            loaded.milestones_picked(2, Some(hero)).monkey_dodge(1, 0.),
            0.08
        );
        loaded.select_spec(hero, 0);
        for _ in 0..5 {
            assert!(loaded.unlearn_talent(hero, 1381, true));
        }
        assert_eq!(
            loaded.milestones_picked(2, Some(hero)).monkey_dodge(1, 0.),
            0.08
        );
        assert!((snapshot.monkey_dodge(1, 0.) - 0.13).abs() < 0.00001);
    }

    #[test]
    fn mark_training_requires_legal_points_and_roundtrips_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(2);
        account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap()
            .talent_xp = 5100;
        assert!(!account.learn_talent(hero, 1, 1343, 5, 1, true));
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 1342, 5, 0, true));
        }
        for rank in 1..=5 {
            assert!(account.learn_talent(hero, 1, 1343, 5, 1, true));
            let t = account.milestones_picked(2, Some(hero));
            assert!(
                (t.hunters_mark_power(1, 0.) - (20.0 * (1.0 + 0.03 * rank as f32))).abs() < 0.00001
            );
            assert_eq!(
                t.hunters_mark_power(0, 0.),
                0.,
                "talent does not grant the mark"
            );
        }
        let snapshot = account.milestones_picked(2, Some(hero));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .hunters_mark_power(1, 0.),
            snapshot.hunters_mark_power(1, 0.)
        );
        loaded.select_spec(hero, 1);
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .hunters_mark_power(1, 0.),
            20.0
        );
        loaded.select_spec(hero, 0);
        for _ in 0..5 {
            assert!(loaded.unlearn_talent(hero, 1343, true));
        }
        assert_eq!(
            loaded
                .milestones_picked(2, Some(hero))
                .hunters_mark_power(1, 0.),
            20.0
        );
        assert!((snapshot.hunters_mark_power(1, 0.) - 23.0).abs() < 0.00001);
    }

    #[test]
    fn creature_training_roundtrips_purchases_specs_and_refunds() {
        use survivors_core::creature::CreatureType;
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let seed = app.world_mut().remove_resource::<Account>().unwrap();
        for (class, id, tree, row, max_rank, kind) in [
            (3, 274, 0, 1, 2, CreatureType::Beast),
            (2, 1623, 2, 0, 3, CreatureType::Beast),
            (2, 1301, 2, 0, 3, CreatureType::Humanoid),
        ] {
            let rules = seed.talent_rules.clone().unwrap();
            let mut account = parse(&encode(&seed)).unwrap();
            account.talent_rules = Some(rules.clone());
            let hero = account.ensure(class);
            let character = account
                .characters
                .iter_mut()
                .find(|c| c.id == hero)
                .unwrap();
            character.talent_xp = 5100;
            if class == 3 {
                character.dbc[0] = vec![(0, 270, 5)];
            }
            let value = |a: &Account| {
                a.milestones_picked(class, Some(hero))
                    .creature_damage_multiplier(kind, false)
            };
            for rank in 1..=max_rank {
                assert!(account.learn_talent(hero, tree, id, max_rank, row, true));
                assert!((value(&account) - (1.0 + 0.01 * rank as f32)).abs() < 0.00001);
            }
            let mut loaded = parse(&encode(&account)).unwrap();
            loaded.talent_rules = Some(rules);
            assert_eq!(value(&loaded), value(&account));
            loaded.select_spec(hero, 1);
            assert_eq!(value(&loaded), 1.0);
            loaded.select_spec(hero, 0);
            assert_eq!(value(&loaded), value(&account));
            for _ in 0..max_rank {
                assert!(loaded.unlearn_talent(hero, id, true));
            }
            assert_eq!(value(&loaded), 1.0);
        }
    }

    #[test]
    fn poison_training_roundtrips_specs_refunds_and_worn_weapon_identity() {
        use survivors_core::talents::Coating;
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (0, 276, 3),
            (0, 270, 5),
            (0, 272, 1),
            (0, 273, 3),
            (0, 277, 3),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(0)));
        let values = |a: &Account| {
            let t = a.milestones_picked(3, Some(hero));
            (Coating::Instant.chance(t), t.cast("deadly_poison").damage)
        };
        for rank in 1..=5 {
            for id in [268, 682] {
                assert!(account.learn_talent(hero, 0, id, 5, 3, true));
            }
            let (chance, damage) = values(&account);
            assert!((chance - (0.2 + 0.02 * rank as f32)).abs() < 0.00001);
            assert!((damage - (1.0 + 0.04 * rank as f32)).abs() < 0.00001);
        }
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(values(&loaded), values(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(values(&loaded), (0.2, 1.0));
        loaded.select_spec(hero, 0);
        for id in [268, 682] {
            assert!(loaded.unlearn_talent(hero, id, true));
        }
        assert!((values(&loaded).0 - 0.28).abs() < 0.00001);
        assert!((values(&loaded).1 - 1.16).abs() < 0.00001);
        let eligible = |a: &Account| {
            Coating::eligible(
                a.milestones_picked(3, Some(hero)),
                "sinister_strike",
                1.0,
                true,
            )
        };
        assert!(eligible(&loaded));
        let progress = &mut loaded.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, None));
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(0)
        ));
        assert!(!eligible(&loaded), "cosmetic dagger is not a worn weapon");
        let other = loaded.ensure(0);
        assert!(!Coating::eligible(
            loaded.milestones_picked(0, Some(other)),
            "sinister_strike",
            1.0,
            true
        ));
    }

    #[test]
    fn seal_fate_purchase_save_specs_and_refunds_preserve_proc_chance() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (0, 276, 3),
            (0, 270, 5),
            (0, 272, 1),
            (0, 273, 3),
            (0, 277, 3),
            (0, 281, 1),
            (0, 269, 5),
            (0, 279, 3),
            (0, 280, 1),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let chance = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .seal_fate_chance("sinister_strike", 10.0, true)
        };
        assert_eq!(chance(&account), 0.0);
        assert!(!account.learn_talent(hero, 0, 283, 5, 5, false));
        for rank in 1u8..=5 {
            assert!(account.learn_talent(hero, 0, 283, 5, 5, true));
            assert_eq!(chance(&account), 0.2 * f32::from(rank));
        }
        assert!(!account.learn_talent(hero, 0, 283, 5, 5, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(chance(&loaded), 1.0);
        loaded.select_spec(hero, 1);
        assert_eq!(chance(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 283, true));
        assert_eq!(chance(&loaded), 0.8);
        assert_eq!(
            loaded
                .milestones_picked(0, Some(hero))
                .seal_fate_chance("sinister_strike", 10.0, true),
            0.0
        );
        assert_eq!(
            loaded
                .milestones_picked(3, Some(u32::MAX))
                .seal_fate_chance("sinister_strike", 10.0, true),
            0.0
        );
    }

    #[test]
    fn rogue_weapon_expertise_tracks_saved_worn_gear_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 201, 2),
            (1, 203, 3),
            (1, 186, 5),
            (1, 181, 5),
            (1, 182, 5),
            (1, 183, 4),
            (1, 223, 1),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let weapon = "weapon:rogue:right-handed-brass-knuckles";
        assert!(account.supplemental.collect(3, weapon));
        assert!(account.classes[3].equipment.wear_supplemental(
            3,
            &account.supplemental,
            8,
            weapon
        ));
        let damage = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .cast("sinister_strike")
                .damage
        };
        let base = damage(&account);
        for _ in 0..2 {
            assert!(account.learn_talent(hero, 1, 1703, 2, 5, true));
        }
        assert!(!account.learn_talent(hero, 1, 1703, 2, 5, true));
        assert!((damage(&account) - base * 1.05).abs() < 0.00001);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(2)
        ));
        assert!((damage(&account) - base * 1.05).abs() < 0.00001);
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(damage(&loaded), damage(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(damage(&loaded), 1.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 1703, true));
        assert!((damage(&loaded) - base * 1.03).abs() < 0.00001);
        let progress = &mut loaded.classes[3];
        assert!(progress
            .equipment
            .show_supplemental(3, &loaded.supplemental, 8, weapon));
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        assert!((damage(&loaded) - base * 1.03).abs() < 0.00001);
        let mace = "weapon:rogue:worn-mace";
        assert!(loaded.supplemental.collect(3, mace));
        assert!(loaded.classes[3]
            .equipment
            .wear_supplemental(3, &loaded.supplemental, 8, mace));
        assert_eq!(damage(&loaded), base);
        assert_eq!(loaded.milestones_picked(3, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn rogue_mace_training_tracks_saved_worn_gear_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 201, 2),
            (1, 203, 3),
            (1, 186, 5),
            (1, 181, 5),
            (1, 182, 5),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let mace = "weapon:rogue:worn-mace";
        assert!(account.supplemental.collect(3, mace));
        assert!(account.classes[3]
            .equipment
            .wear_supplemental(3, &account.supplemental, 8, mace));
        let damage = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .cast("sinister_strike")
                .damage
        };
        let base = damage(&account);
        let stun = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .mace_stun_chance("sinister_strike", 20.0)
        };
        assert_eq!(stun(&account), 0.0);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 184, 5, 4, true));
        }
        assert!(!account.learn_talent(hero, 1, 184, 5, 4, true));
        assert!((damage(&account) - base * 1.05).abs() < 0.00001);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(2)
        ));
        assert!((damage(&account) - base * 1.05).abs() < 0.00001);
        assert_eq!(stun(&account), 0.06);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(damage(&loaded), damage(&account));
        assert_eq!(stun(&loaded), 0.06);
        loaded.select_spec(hero, 1);
        assert_eq!(damage(&loaded), 1.0);
        assert_eq!(stun(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 184, true));
        assert!((damage(&loaded) - base * 1.04).abs() < 0.00001);
        assert_eq!(stun(&loaded), 0.04);
        let progress = &mut loaded.classes[3];
        assert!(progress
            .equipment
            .show_supplemental(3, &loaded.supplemental, 8, mace));
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        assert_eq!(damage(&loaded), base);
        assert_eq!(stun(&loaded), 0.0);
        assert_eq!(loaded.milestones_picked(3, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn fist_training_tracks_saved_worn_gear_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 201, 2),
            (1, 203, 3),
            (1, 186, 5),
            (1, 181, 5),
            (1, 182, 5),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let fist = "weapon:rogue:right-handed-brass-knuckles";
        assert!(account.supplemental.collect(3, fist));
        assert!(account.classes[3]
            .equipment
            .wear_supplemental(3, &account.supplemental, 8, fist));
        let crit = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .cast("sinister_strike")
                .crit
        };
        let base = crit(&account);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 183, 5, 4, true));
        }
        assert!(!account.learn_talent(hero, 1, 183, 5, 4, true));
        assert!((crit(&account) - base - 0.05).abs() < 0.00001);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(2)
        ));
        assert!((crit(&account) - base - 0.05).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(crit(&loaded), crit(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(crit(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 183, true));
        assert!((crit(&loaded) - base - 0.04).abs() < 0.00001);
        let progress = &mut loaded.classes[3];
        assert!(progress
            .equipment
            .show_supplemental(3, &loaded.supplemental, 8, fist));
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        assert_eq!(crit(&loaded), base);
        assert_eq!(loaded.milestones_picked(3, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn dagger_training_tracks_saved_worn_gear_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 201, 2), (1, 203, 3), (1, 186, 5), (1, 181, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[3].pieces.fill(1);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(0)));
        let crit = |account: &Account| {
            account
                .milestones_picked(3, Some(hero))
                .cast("sinister_strike")
                .crit
        };
        let base = crit(&account);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 182, 5, 3, true));
        }
        assert!(!account.learn_talent(hero, 1, 182, 5, 3, true));
        assert!((crit(&account) - base - 0.05).abs() < 0.00001);
        let progress = &mut account.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(2)
        ));
        assert!((crit(&account) - base - 0.05).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(crit(&loaded), crit(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(crit(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 182, true));
        assert!((crit(&loaded) - base - 0.04).abs() < 0.00001);
        let progress = &mut loaded.classes[3];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(0)
        ));
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(2)));
        assert_eq!(crit(&loaded), base);
        assert_eq!(loaded.milestones_picked(3, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn two_handed_training_tracks_worn_gear_saved_specs_and_refunds() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(0);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (0, 124, 3),
            (0, 130, 5),
            (0, 127, 3),
            (0, 126, 2),
            (0, 128, 2),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        account.classes[0].pieces.fill(1);
        let progress = &mut account.classes[0];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(0)));
        let multiplier = |account: &Account| {
            account
                .milestones_picked(0, Some(hero))
                .cast("cleave")
                .damage
        };
        let base = multiplier(&account);
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 0, 136, 5, 3, true));
        }
        assert!(!account.learn_talent(hero, 0, 136, 5, 3, true));
        assert!((multiplier(&account) - base * 1.05).abs() < 0.00001);
        let progress = &mut account.classes[0];
        assert!(progress.equipment.show_legacy(
            &progress.pieces,
            8,
            survivors_core::equipment::Look::Piece(5)
        ));
        assert!((multiplier(&account) - base * 1.05).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(multiplier(&loaded), multiplier(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(multiplier(&loaded), 1.0);
        loaded.select_spec(hero, 0);
        assert!(loaded.unlearn_talent(hero, 136, true));
        assert!((multiplier(&loaded) - base * 1.04).abs() < 0.00001);
        let progress = &mut loaded.classes[0];
        assert!(progress.equipment.wear_legacy(&progress.pieces, 8, Some(5)));
        assert_eq!(multiplier(&loaded), base);
        assert_eq!(loaded.milestones_picked(0, Some(u32::MAX)).weapon, None);
    }

    #[test]
    fn weapon_training_follow_saved_worn_equipment() {
        for (id, tree, row, worn, cosmetic) in [
            (125, 0, 4, 0, 5),
            (134, 0, 5, 0, 5),
            (132, 0, 4, 0, 5),
            (702, 2, 5, 5, 0),
            (123, 0, 4, 3, 0),
        ] {
            let data = benilla_formats::wow_data_or_skip!();
            let mut chain = benilla_formats::Chain::open(&data).unwrap();
            let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
            let mut app = App::new();
            app.insert_resource(empty())
                .insert_resource(crate::ui_talent::Talents { catalog })
                .add_systems(PreUpdate, super::super::talent_validation::load_rules);
            app.update();
            let mut account = app.world_mut().remove_resource::<Account>().unwrap();
            let rules = account.talent_rules.clone().unwrap();
            let hero = account.ensure(0);
            let character = account
                .characters
                .iter_mut()
                .find(|c| c.id == hero)
                .unwrap();
            character.talent_xp = 5100;
            character.dbc[0] = if id != 702 {
                vec![
                    (0, 124, 3),
                    (0, 130, 5),
                    (0, 127, 3),
                    (0, 126, 2),
                    (0, 128, 2),
                    (0, 136, 5),
                ]
            } else {
                vec![
                    (2, 1601, 5),
                    (2, 138, 5),
                    (2, 140, 5),
                    (2, 145, 3),
                    (2, 147, 3),
                    (2, 151, 3),
                    (2, 152, 1),
                ]
            };
            if id == 134 {
                account
                    .characters
                    .iter_mut()
                    .find(|c| c.id == hero)
                    .unwrap()
                    .dbc[0]
                    .push((0, 132, 5));
            }
            assert!(account.talents_valid(account.character(hero).unwrap()));
            account.classes[0].pieces.fill(1);
            let progress = &mut account.classes[0];
            assert!(progress
                .equipment
                .wear_legacy(&progress.pieces, 8, Some(worn)));
            if matches!(id, 125 | 134) {
                let (roll, key, name) = if id == 125 {
                    (0.0, "weapon:warrior:worn-mace", "Worn Mace")
                } else {
                    (0.75, "weapon:warrior:short-spear", "Short Spear")
                };
                assert_eq!(
                    account.collect_weapon_drop(0, 0, "Hogger", true, roll),
                    Some(name)
                );
                assert!(account.select_supplemental(0, key, false));
            }
            let multiplier = |account: &Account| {
                let effect = account.milestones_picked(0, Some(hero)).cast("cleave");
                if id == 125 {
                    account
                        .milestones_picked(0, Some(hero))
                        .mace_stun_chance("cleave", 1.0)
                } else if id == 123 {
                    account
                        .milestones_picked(0, Some(hero))
                        .sword_extra_attack_chance("cleave", 1.0)
                } else if matches!(id, 132 | 134) {
                    effect.crit
                } else {
                    effect.damage
                }
            };
            let expected = |base: f32, rank: f32| {
                if id == 125 {
                    base + if rank == 5.0 { 0.06 } else { 0.01 * rank }
                } else if id != 702 {
                    base + 0.01 * rank
                } else {
                    base * (1.0 + 0.02 * rank)
                }
            };
            let base = multiplier(&account);
            for _ in 0..5 {
                assert!(account.learn_talent(hero, tree, id, 5, row, true));
            }
            assert!(!account.learn_talent(hero, tree, id, 5, row, true));
            assert!((multiplier(&account) - expected(base, 5.0)).abs() < 0.00001);
            let progress = &mut account.classes[0];
            assert!(progress.equipment.show_legacy(
                &progress.pieces,
                8,
                survivors_core::equipment::Look::Piece(cosmetic)
            ));
            assert!((multiplier(&account) - expected(base, 5.0)).abs() < 0.00001);
            let mut loaded = parse(&encode(&account)).unwrap();
            loaded.talent_rules = Some(rules);
            assert_eq!(multiplier(&loaded), multiplier(&account));
            loaded.select_spec(hero, 1);
            assert_eq!(multiplier(&loaded), if id != 702 { 0.0 } else { 1.0 });
            loaded.select_spec(hero, 0);
            assert!(loaded.unlearn_talent(hero, id, true));
            assert!((multiplier(&loaded) - expected(base, 4.0)).abs() < 0.00001);
            let progress = &mut loaded.classes[0];
            assert!(progress
                .equipment
                .wear_legacy(&progress.pieces, 8, Some(cosmetic)));
            assert_eq!(multiplier(&loaded), base);
            assert_eq!(loaded.milestones_picked(0, Some(u32::MAX)).weapon, None);
        }
    }

    #[test]
    fn battle_shout_purchase_save_spec_and_refund_reach_buff_strength() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(0);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![(1, 158, 5), (1, 157, 5)];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(0, hero);
        let strength = |account: &Account| {
            account
                .milestones_picked(0, Some(hero))
                .battle_shout_multiplier()
        };
        assert_eq!(strength(&account), 1.0);
        for rank in 1..=5 {
            assert!(account.learn_talent(hero, 1, 154, 5, 2, true));
            assert!((strength(&account) - (1.0 + 0.05 * rank as f32)).abs() < 0.00001);
        }
        assert!(!account.learn_talent(hero, 1, 154, 5, 2, true));
        assert!((account.bonus(0, hero).might - before.might).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(strength(&loaded), strength(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(strength(&loaded), 1.0);
        loaded.select_spec(hero, 0);
        for rank in (0..5).rev() {
            assert!(loaded.unlearn_talent(hero, 154, true));
            assert!((strength(&loaded) - (1.0 + 0.05 * rank as f32)).abs() < 0.00001);
        }
        assert!(!loaded.unlearn_talent(hero, 154, true));
        assert_eq!(
            loaded
                .milestones_picked(0, Some(u32::MAX))
                .battle_shout_multiplier(),
            1.0
        );
    }

    #[test]
    fn hamstring_purchase_save_spec_and_refund_reach_hit_rules() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(0);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (0, 124, 3),
            (0, 130, 5),
            (0, 127, 3),
            (0, 126, 2),
            (0, 128, 2),
            (0, 136, 5),
            (0, 132, 5),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(0, hero);
        let chance = |account: &Account| {
            account
                .milestones_picked(0, Some(hero))
                .hamstring_root_chance("thunder_clap", 10.0)
        };
        assert_eq!(chance(&account), 0.0);
        for rank in 1..=3 {
            assert!(account.learn_talent(hero, 0, 129, 3, 5, true));
            assert!((chance(&account) - 0.05 * rank as f32).abs() < 0.00001);
        }
        assert!(!account.learn_talent(hero, 0, 129, 3, 5, true));
        assert!((account.bonus(0, hero).might - before.might).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(chance(&loaded), chance(&account));
        loaded.select_spec(hero, 1);
        assert_eq!(chance(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        for rank in (0..3).rev() {
            assert!(loaded.unlearn_talent(hero, 129, true));
            assert!((chance(&loaded) - 0.05 * rank as f32).abs() < 0.00001);
        }
        assert!(!loaded.unlearn_talent(hero, 129, true));
        assert_eq!(
            loaded
                .milestones_picked(0, Some(u32::MAX))
                .hamstring_root_chance("thunder_clap", 10.0),
            0.0
        );
    }

    #[test]
    fn blade_flurry_purchase_save_and_refund_reach_clock() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 203, 3),
            (1, 201, 2),
            (1, 181, 5),
            (1, 187, 5),
            (1, 204, 2),
            (1, 221, 3),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(3, hero);
        for _ in 0..1 {
            assert!(account.learn_talent(hero, 1, 223, 1, 4, true));
        }
        assert!(!account.learn_talent(hero, 1, 223, 1, 4, true));
        assert!((account.bonus(3, hero).might - before.might).abs() < 0.00001);
        let bonus = |account: &Account| {
            survivors_core::talents::BladeFlurryClock::default().haste(
                account.milestones_picked(3, Some(hero)),
                "blade_flurry",
                0.,
            )
        };
        assert_eq!(bonus(&account), 0.2);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(bonus(&loaded), 0.2);
        loaded.select_spec(hero, 1);
        assert_eq!(bonus(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert_eq!(bonus(&loaded), 0.2);
        assert!(loaded.unlearn_talent(hero, 223, true));
        assert_eq!(bonus(&loaded), 0.0);
        assert!(!loaded.unlearn_talent(hero, 223, true));
    }

    #[test]
    fn adrenaline_rush_purchase_save_and_refund_reach_clock() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(3);
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 203, 3),
            (1, 201, 2),
            (1, 181, 5),
            (1, 187, 5),
            (1, 204, 2),
            (1, 221, 5),
            (1, 223, 1),
            (1, 242, 5),
            (1, 1703, 2),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(3, hero);
        for _ in 0..1 {
            assert!(account.learn_talent(hero, 1, 205, 1, 6, true));
        }
        assert!(!account.learn_talent(hero, 1, 205, 1, 6, true));
        assert!((account.bonus(3, hero).might - before.might).abs() < 0.00001);
        let bonus = |account: &Account| {
            survivors_core::talents::AdrenalineRushClock::default().bonus(
                account.milestones_picked(3, Some(hero)),
                "sinister_strike",
                0.,
            )
        };
        assert_eq!(bonus(&account), 0.5);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(bonus(&loaded), 0.5);
        loaded.select_spec(hero, 1);
        assert_eq!(bonus(&loaded), 0.0);
        loaded.select_spec(hero, 0);
        assert_eq!(bonus(&loaded), 0.5);
        assert!(loaded.unlearn_talent(hero, 205, true));
        assert_eq!(bonus(&loaded), 0.0);
        assert!(!loaded.unlearn_talent(hero, 205, true));
    }

    #[test]
    fn leader_of_the_pack_purchase_save_specs_and_refund_reach_run_stats() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(8);
        let other = account.create("Other", "Druid", 8).unwrap();
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 796, 5),
            (1, 795, 5),
            (1, 799, 5),
            (1, 794, 5),
            (1, 807, 2),
            (1, 798, 3),
            (1, 803, 3),
            (1, 802, 2),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let before = account.bonus(8, hero);
        let other_before = account.bonus(8, other);
        assert!(account.learn_talent(hero, 1, 809, 1, 6, true));
        assert!(!account.learn_talent(hero, 1, 809, 1, 6, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let after = account.bonus(8, hero);
        assert!((after.crit - before.crit - 0.03).abs() < 0.00001);
        assert!((after.might - before.might).abs() < 0.00001);
        assert_eq!(account.bonus(8, other).crit, other_before.crit);
        let snapshot = super::super::Run {
            class: &data::CLASSES[8],
            bonus: after,
            ..Default::default()
        };
        let baseline = super::super::Run {
            class: &data::CLASSES[8],
            bonus: before,
            ..Default::default()
        };
        assert!((snapshot.stats().crit - baseline.stats().crit - 0.03).abs() < 0.00001);
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(loaded.bonus(8, hero).crit, after.crit);
        for spec in [1, 2] {
            loaded.select_spec(hero, spec);
            assert_eq!(loaded.bonus(8, hero).crit, other_before.crit);
        }
        loaded.select_spec(hero, 0);
        assert_eq!(loaded.bonus(8, hero).crit, after.crit);
        assert!(loaded.unlearn_talent(hero, 809, true));
        assert_eq!(loaded.bonus(8, hero).crit, before.crit);
        assert!(!loaded.unlearn_talent(hero, 809, true));
        assert_eq!(snapshot.bonus.crit, after.crit);
    }

    #[test]
    fn heart_of_the_wild_purchase_save_specs_and_refund_reach_run_stats() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let hero = account.ensure(8);
        let other = account.create("Other", "Druid", 8).unwrap();
        let character = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        character.talent_xp = 5100;
        character.dbc[0] = vec![
            (1, 796, 5),
            (1, 795, 5),
            (1, 799, 5),
            (1, 794, 5),
            (1, 807, 2),
            (1, 798, 3),
            (1, 803, 3),
            (1, 802, 2),
        ];
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let baseline = account.milestones_picked(8, Some(hero));
        for _ in 0..5 {
            assert!(account.learn_talent(hero, 1, 808, 5, 5, true));
        }
        assert!(!account.learn_talent(hero, 1, 808, 5, 5, true));
        assert!(account.talents_valid(account.character(hero).unwrap()));
        let trained = account.milestones_picked(8, Some(hero));
        assert_eq!(trained.health_multiplier(), 1.2);
        assert_eq!(
            account
                .milestones_picked(8, Some(other))
                .health_multiplier(),
            1.0
        );
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules);
        assert_eq!(
            loaded.milestones_picked(8, Some(hero)).health_multiplier(),
            1.2
        );
        for spec in [1, 2] {
            loaded.select_spec(hero, spec);
            assert_eq!(
                loaded.milestones_picked(8, Some(hero)).health_multiplier(),
                1.0
            );
        }
        loaded.select_spec(hero, 0);
        for _ in 0..5 {
            assert!(loaded.unlearn_talent(hero, 808, true));
        }
        assert_eq!(
            loaded.milestones_picked(8, Some(hero)).health_multiplier(),
            baseline.health_multiplier()
        );
        assert!(!loaded.unlearn_talent(hero, 808, true));
        assert_eq!(trained.health_multiplier(), 1.2, "run snapshot stays fixed");
    }

    #[test]
    fn rogue_training_survives_save_and_stays_with_hero_and_active_spec() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let validate = |mut account: Account| {
            account.talent_rules = Some(rules.clone());
            account
        };
        let first = account.ensure(3);
        let second = account.create("Second", "Rogue", 3).unwrap();
        let baseline = account.bonus(3, second);
        let hero = account
            .characters
            .iter_mut()
            .find(|h| h.id == second)
            .unwrap();
        hero.talent_xp = 1200;
        // Combat meets Precision's real five-point earlier-row gate.
        hero.dbc[0] = vec![(0, 276, 3), (0, 270, 5)];
        hero.dbc[1] = vec![(1, 201, 2), (1, 186, 3), (1, 181, 5)];
        assert!(account.talents_valid(account.character(second).unwrap()));
        let assassination = account.milestones_picked(3, Some(second));
        assert!((assassination.cast("eviscerate").damage - 1.15).abs() < 0.00001);
        assert!((assassination.cast("sinister_strike").crit - 0.05).abs() < 0.00001);
        assert_eq!(
            account
                .milestones_picked(3, Some(first))
                .cast("eviscerate")
                .damage,
            1.0
        );
        assert_eq!(
            account
                .milestones_picked(0, Some(second))
                .cast("eviscerate")
                .damage,
            1.0
        );
        assert!((account.bonus(3, second).might - baseline.might).abs() < 0.00001);
        let run = super::super::Run {
            class: &data::CLASSES[3],
            milestones: assassination,
            bonus: account.bonus(3, second),
            ..Default::default()
        };
        account.select_spec(second, 1);
        let combat = account.milestones_picked(3, Some(second));
        assert!(account.talents_valid(account.character(second).unwrap()));
        assert_eq!(combat.points, [0, 10, 0]);
        assert!((combat.cast("eviscerate").damage - 1.05).abs() < 0.00001);
        assert!((combat.cast("rupture").damage - 1.05).abs() < 0.00001);
        assert_eq!(combat.cast("fireball").damage, 1.0);
        assert_eq!(combat.cast("eviscerate").crit, 0.0);
        assert!((combat.cast("sinister_strike").cooldown - 0.81).abs() < 0.00001);
        // Precision adds no global damage; reaching ten tree points retains the
        // separate generic +4% milestone shared by every class.
        assert!((account.bonus(3, second).might - baseline.might - 0.04).abs() < 0.00001);
        // Changing the account must not rewrite the snapshot of a run already in progress.
        assert_eq!(run.milestones, assassination);
        let mut loaded = validate(parse(&encode(&account)).unwrap());
        assert_eq!(loaded.milestones_picked(3, Some(second)), combat);
        loaded.select_spec(second, 0);
        assert_eq!(loaded.milestones_picked(3, Some(second)), assassination);
        loaded.select_spec(second, 2);
        assert_eq!(
            loaded
                .milestones_picked(3, Some(second))
                .cast("sinister_strike")
                .cooldown,
            1.0
        );
        loaded.select_spec(second, 1);
        loaded.respec(second);
        assert_eq!(
            loaded
                .milestones_picked(3, Some(second))
                .cast("sinister_strike")
                .cooldown,
            1.0
        );
        assert_eq!(
            loaded
                .milestones_picked(3, Some(second))
                .cast("rupture")
                .damage,
            1.0
        );
        assert!(
            (combat.cast("rupture").damage - 1.05).abs() < 0.00001,
            "a Combat run retains its saved snapshot after respec"
        );
        loaded.select_spec(second, 0);
        assert_eq!(loaded.milestones_picked(3, Some(second)), assassination);
        let reloaded = validate(parse(&encode(&loaded)).unwrap());
        assert_eq!(reloaded.milestones_picked(3, Some(second)), assassination);
        assert_eq!(reloaded.character(second).unwrap().dbc[1], Vec::new());
    }

    #[test]
    fn mage_training_follows_character_spec_save_and_refund() {
        let mut account = empty();
        let first = account.ensure(6);
        let second = account.create("Test", "Mage", 6).unwrap();
        let hero = account
            .characters
            .iter_mut()
            .find(|h| h.id == second)
            .unwrap();
        hero.talent_xp = 5100;
        hero.dbc[0] = vec![(1, 26, 5)];
        hero.dbc[1] = vec![(2, 37, 5)];
        let fire = account.milestones_picked(6, Some(second));
        assert!((fire.cast("fireball").cooldown - 0.8).abs() < 0.00001);
        assert_eq!(fire.cast("frostbolt").cooldown, 1.);
        assert_eq!(
            account
                .milestones_picked(6, Some(first))
                .cast("fireball")
                .cooldown,
            1.
        );
        account.select_spec(second, 1);
        let frost = account.milestones_picked(6, Some(second));
        assert!((frost.cast("frostbolt").cooldown - 0.8).abs() < 0.00001);
        assert_eq!(frost.cast("fireball").cooldown, 1.);
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.milestones_picked(6, Some(second)), frost);
        account.respec(second);
        assert_eq!(
            account
                .milestones_picked(6, Some(second))
                .cast("frostbolt")
                .cooldown,
            1.
        );
        account.select_spec(second, 0);
        assert_eq!(account.milestones_picked(6, Some(second)), fire);
    }

    #[test]
    fn hunter_milestones_follow_the_selected_spec_and_survive_save() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
        let mut app = App::new();
        app.insert_resource(empty())
            .insert_resource(crate::ui_talent::Talents { catalog })
            .add_systems(PreUpdate, super::super::talent_validation::load_rules);
        app.update();
        let mut account = app.world_mut().remove_resource::<Account>().unwrap();
        let rules = account.talent_rules.clone().unwrap();
        let restore = |account: &Account| {
            let mut loaded = parse(&encode(account)).unwrap();
            loaded.talent_rules = Some(rules.clone());
            loaded
        };
        let first = account.ensure(2);
        let second = account.create("Test", "Hunter", 2).unwrap();
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap();
        hero.talent_xp = 5100;
        hero.dbc[0] = vec![
            (1, 1342, 5),
            (1, 1344, 5),
            (1, 1346, 5),
            (1, 1348, 5),
            (1, 1349, 5),
            (1, 1347, 3),
            (1, 1362, 2),
        ];
        assert!(account.talents_valid(account.character(second).unwrap()));
        let baseline = account.milestones_picked(2, Some(second));
        assert_eq!(baseline.points, [0, 30, 0]);
        assert!(account.learn_talent(second, 1, 1361, 1, 6, true));
        assert!(!account.learn_talent(second, 1, 1361, 1, 6, true));
        assert!(account.talents_valid(account.character(second).unwrap()));
        let trained = account.milestones_picked(2, Some(second));
        assert_eq!(trained.points, [0, 31, 0]);
        let check = |actual: survivors_core::talents::Milestones| {
            for key in [
                "arcane_shot",
                "multi_shot",
                "serpent_sting",
                "concussive_shot",
                "aimed_shot",
                "volley",
                "explosive_trap",
                "fireball",
            ] {
                let factor = if matches!(key, "volley" | "explosive_trap" | "fireball") {
                    1.
                } else {
                    1.1
                };
                assert!(
                    (actual.cast(key).damage - baseline.cast(key).damage * factor).abs() < 0.00001,
                    "{key}"
                );
            }
            assert_eq!(actual.hunter_pet(0.), baseline.hunter_pet(0.));
            assert_eq!(actual.cast("multi_shot").extra_targets, 2);
        };
        check(trained);
        assert_eq!(account.milestones_picked(2, Some(first)).points, [0; 3]);
        assert_eq!(
            account
                .milestones_picked(0, Some(second))
                .cast("arcane_shot")
                .damage,
            1.
        );
        let run = super::super::Run {
            class: &data::CLASSES[2],
            milestones: trained,
            bonus: account.bonus(2, second),
            ..Default::default()
        };
        let mut loaded = restore(&account);
        assert_eq!(loaded.milestones_picked(2, Some(second)), trained);
        for spec in [1, 2] {
            loaded.select_spec(second, spec);
            loaded = restore(&loaded);
            assert_eq!(loaded.character(second).unwrap().spec, spec);
            assert_eq!(loaded.milestones_picked(2, Some(second)).points, [0; 3]);
            assert_eq!(
                loaded
                    .milestones_picked(2, Some(second))
                    .cast("arcane_shot")
                    .damage,
                1.
            );
        }
        loaded.select_spec(second, 0);
        check(loaded.milestones_picked(2, Some(second)));
        assert!(loaded.unlearn_talent(second, 1361, true));
        assert!(!loaded.unlearn_talent(second, 1361, true));
        assert!(loaded.talents_valid(loaded.character(second).unwrap()));
        loaded = restore(&loaded);
        assert_eq!(loaded.milestones_picked(2, Some(second)), baseline);
        // An active run retains its original snapshot through menu edits.
        check(run.milestones);
        loaded.respec(second);
        loaded = restore(&loaded);
        assert_eq!(loaded.milestones_picked(2, Some(second)).points, [0; 3]);
        assert_eq!(
            loaded
                .milestones_picked(2, Some(second))
                .cast("multi_shot")
                .extra_targets,
            0
        );
        check(run.milestones);
    }

    #[test]
    fn warrior_training_follows_selected_spec_and_persists() {
        let mut account = empty();
        let first = account.ensure(0);
        let second = account.create("Test", "Warrior", 0).unwrap();
        let baseline = account.bonus(0, second);
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == second)
            .unwrap();
        hero.talent_xp = 1000;
        hero.dbc[0] = vec![(0, 124, 3), (1, 166, 3)];
        hero.dbc[1] = vec![(1, 157, 5), (2, 140, 5)];
        let cast = account.milestones_picked(0, Some(second));
        assert!((cast.cast("heroic_strike").cooldown - 0.85).abs() < 0.00001);
        assert!((cast.cast("cleave").damage - 1.3).abs() < 0.00001);
        assert_eq!(
            account
                .milestones_picked(0, Some(first))
                .cast("cleave")
                .damage,
            1.0
        );
        assert!((account.bonus(0, second).might - baseline.might).abs() < 0.00001);
        account.select_spec(second, 1);
        let defense = account.bonus(0, second);
        assert!((defense.crit - baseline.crit - 0.05).abs() < 0.00001);
        assert_eq!(defense.armor - baseline.armor, 30.0);
        assert_eq!(
            account
                .milestones_picked(0, Some(second))
                .cast("cleave")
                .damage,
            1.0
        );
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.bonus(0, second).armor, defense.armor);
        account.respec(second);
        assert_eq!(account.bonus(0, second).armor, baseline.armor);
        account.select_spec(second, 0);
        assert_eq!(account.milestones_picked(0, Some(second)), cast);
    }

    #[test]
    fn alterac_rewards_preserve_equipment_and_reload_for_every_class() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(8, 4, 9);
    }

    #[test]
    fn arathi_rewards_preserve_equipment_and_share_rare_ownership() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(4, 2, 9);
        for class in 0..data::CLASSES.len() {
            let mut account = empty();
            account.ensure(class);
            for row in campaign::DROPS.iter().filter(|row| row.zone == 4) {
                assert!(account.collect_drop(class, 4, row.creature).is_some());
            }
            let saved = encode(&account);
            for row in campaign::DROPS.iter().filter(|row| row.zone == 5) {
                assert_eq!(account.collect_drop(class, 5, row.creature), None);
            }
            assert_eq!(
                encode(&account),
                saved,
                "alternate source must not stack gear"
            );
        }
    }

    #[test]
    fn warsong_rewards_preserve_equipment_and_share_uncommon_ownership() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(2, 1, 9);
        for class in 0..data::CLASSES.len() {
            let mut account = empty();
            account.ensure(class);
            for row in campaign::DROPS.iter().filter(|row| row.zone == 2) {
                assert!(account.collect_drop(class, 2, row.creature).is_some());
            }
            let saved = encode(&account);
            for row in campaign::DROPS.iter().filter(|row| row.zone == 3) {
                assert_eq!(account.collect_drop(class, 3, row.creature), None);
            }
            assert_eq!(
                encode(&account),
                saved,
                "alternate source must not stack gear"
            );
        }
    }

    #[test]
    fn uncommon_encounter_rewards_preserve_selected_gear_and_reload_for_every_class() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(3, 1, 9);
    }

    #[test]
    fn epic_encounter_rewards_preserve_selected_gear_and_reload_for_every_class() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(6, 3, 9);
    }

    #[test]
    fn rare_encounter_rewards_preserve_selected_gear_and_reload_for_every_class() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(5, 2, 9);
    }

    #[test]
    fn legendary_armor_rewards_preserve_selected_gear_and_reload_for_every_class() {
        encounter_rewards_preserve_selected_gear_and_reload_for_every_class(10, 5, 8);
    }

    fn encounter_rewards_preserve_selected_gear_and_reload_for_every_class(
        zone: usize,
        band: usize,
        slots: usize,
    ) {
        use super::super::equipment::Look;
        for class in 0..data::CLASSES.len() {
            let mut account = empty();
            let hero = account.ensure(class);
            // Start with an earned and deliberately selected common helmet.
            assert!(account.collect_drop(class, 0, "Hogger").is_some());
            assert!(account.show_piece(class, 0, Look::Piece(0)));
            for row in campaign::DROPS.iter().filter(|row| row.zone == zone) {
                assert_eq!(
                    account.collect_drop(class, zone, row.creature),
                    campaign::piece_name(class, row.slot, band)
                );
            }
            let progress = &account.classes[class];
            assert_eq!(
                progress.pieces.iter().filter(|&&n| n > 0).count(),
                slots + 1
            );
            assert_eq!(progress.equipment.worn[0], Some(0));
            assert!(progress.equipment.worn[1..slots]
                .iter()
                .all(|&b| b == Some(band as u8)));
            assert!(progress.equipment.worn[slots..].iter().all(Option::is_none));
            assert!(progress.clears.iter().all(|&cleared| !cleared));
            let might = account.bonus(class, hero).might;
            assert!(account.wear_piece(class, 0, Some(band as u8)));
            assert!(account.bonus(class, hero).might > might);
            assert_eq!(
                account.classes[class]
                    .equipment
                    .visible_bands(&account.classes[class].pieces)[0],
                Some(0)
            );
            // Repeated boss rewards must not undo an explicit empty slot or hidden look.
            assert!(account.wear_piece(class, slots - 1, None));
            assert!(account.show_piece(class, 7, Look::Hidden));
            let saved = encode(&account);
            let mut loaded = parse(&saved).unwrap();
            assert_eq!(encode(&loaded), saved);
            for row in campaign::DROPS.iter().filter(|row| row.zone == zone) {
                assert_eq!(loaded.collect_drop(class, zone, row.creature), None);
            }
            assert_eq!(encode(&loaded), saved);
            assert_eq!(loaded.classes[class].equipment.worn[slots - 1], None);
            assert_eq!(loaded.classes[class].equipment.looks[7], Look::Hidden);
            assert_eq!(
                loaded.bonus(class, hero).health,
                account.bonus(class, hero).health
            );
            assert!(loaded.classes.iter().enumerate().all(|(other, progress)| {
                other == class || progress.pieces.iter().all(|&n| n == 0)
            }));
        }
    }

    #[test]
    fn duplicate_drops_and_legacy_stack_counts_never_add_power() {
        let mut account = empty();
        let hero = account.ensure(0);
        assert!(account.collect_drop(0, 0, "Hogger").is_some());
        let saved = encode(&account);
        let armor = account.bonus(0, hero).armor;
        assert!(account.collect_drop(0, 0, "Hogger").is_none());
        assert_eq!(encode(&account), saved);
        assert_eq!(account.bonus(0, hero).armor, armor);
        let legacy = format!(
            "class 0 9{} 0 000000000000\n",
            "0".repeat(campaign::PIECES - 1)
        );
        let mut loaded = parse(&legacy).unwrap();
        let loaded_hero = loaded.ensure(0);
        assert_eq!(loaded.classes[0].pieces[0], 1);
        assert_eq!(loaded.bonus(0, loaded_hero).armor, armor);
    }

    #[test]
    fn equipment_rows_are_validated_after_ownership_regardless_of_order() {
        let account =
            parse("equipment 0 100000000 100000000\nclass 0 1000000000 0 0000000\n").unwrap();
        assert_eq!(account.classes[0].equipment.worn[0], Some(0));
        assert_eq!(
            account.classes[0].equipment.looks[0],
            super::super::equipment::Look::Hidden
        );
        let invalid =
            parse("equipment 0 600000000 700000000\nclass 0 1000000000 0 0000000\n").unwrap();
        assert_eq!(invalid.classes[0].equipment.worn[0], None);
        assert_eq!(
            invalid.classes[0].equipment.looks[0],
            super::super::equipment::Look::Worn
        );
    }

    #[test]
    fn legacy_equipment_migrates_once_and_explicit_empty_slots_survive_reload() {
        let mut account = parse("class 0 1111111110 0 0000000\n").unwrap();
        assert_eq!(
            account.classes[0].equipment.worn,
            [Some(0); campaign::GRID_SLOTS]
        );
        let id = account.ensure(0);
        let before = account.bonus(0, id).might;
        assert!(account.wear_piece(0, 0, None));
        assert!(account.bonus(0, id).might < before);
        assert!(account.show_piece(0, 0, super::super::equipment::Look::Hidden));
        let again = parse(&encode(&account)).unwrap();
        assert_eq!(again.classes[0].equipment.worn[0], None);
        assert_eq!(
            again.classes[0].equipment.looks[0],
            super::super::equipment::Look::Hidden
        );
        assert_eq!(
            again.classes[1].equipment.worn,
            [None; campaign::GRID_SLOTS]
        );
        assert_eq!(again.classes[0].pieces[0], 1);
        // A duplicate must not undo an explicit unequip.
        account.collect_drop(0, 0, "Hogger");
        assert_eq!(account.classes[0].equipment.worn[0], None);
    }

    #[test]
    fn settlement_rewards_the_selected_hero_once_and_survives_reload() {
        let mut account = empty();
        let first = account.create("First", "Warrior", 0).unwrap();
        let second = account.create("Second", "Warrior", 0).unwrap();
        let selection = super::super::Selection {
            class: Some(0),
            character: Some(second),
            map: data::MAPS
                .iter()
                .position(|map| map.name == "Elwynn Forest")
                .unwrap(),
            ..Default::default()
        };
        let mut run = super::super::Run {
            level: 8,
            kills: 100,
            won: true,
            ..Default::default()
        };
        settle_run(&mut account, &selection, &mut run);
        let xp = campaign::talent_award(8, 100, 0.0, 0, true, Some(0));
        assert_eq!(account.character(first).unwrap().talent_xp, 0);
        assert_eq!(account.character(second).unwrap().talent_xp, xp);
        assert!(account.classes[0].clears[0]);
        let saved = encode(&account);
        settle_run(&mut account, &selection, &mut run);
        assert_eq!(encode(&account), saved);
        let loaded = parse(&saved).unwrap();
        assert_eq!(loaded.character(second).unwrap().talent_xp, xp);
        assert_eq!((loaded.runs, loaded.wins), (1, 1));
    }

    #[test]
    fn settlement_rejects_foreign_and_stale_hero_references() {
        for selected in [Some(1), Some(u32::MAX), None] {
            let mut account = empty();
            let foreign = account.create("Other", "Class", 1).unwrap();
            let selection = super::super::Selection {
                class: Some(0),
                character: selected,
                ..Default::default()
            };
            let mut run = super::super::Run::default();
            settle_run(&mut account, &selection, &mut run);
            assert_eq!(account.character(foreign).unwrap().talent_xp, 0);
            assert_eq!(
                account
                    .characters
                    .iter()
                    .find(|c| c.class == 0)
                    .unwrap()
                    .talent_xp,
                campaign::talent_award(1, 0, 0.0, 0, false, Some(0))
            );
        }
    }

    #[test]
    fn settlement_without_a_valid_class_does_not_consume_the_award() {
        let mut account = empty();
        let mut run = super::super::Run::default();
        for class in [None, Some(usize::MAX)] {
            let selection = super::super::Selection {
                class,
                ..Default::default()
            };
            settle_run(&mut account, &selection, &mut run);
            assert!(!run.account_paid);
            assert_eq!(account.runs, 0);
            assert!(account.characters.is_empty());
        }
    }

    #[test]
    fn talent_awards_saturate_before_capping() {
        let mut account = empty();
        let id = account.ensure(0);
        account.add_talent_xp(id, 1);
        account.add_talent_xp(id, u32::MAX);
        assert_eq!(
            account.character(id).unwrap().talent_xp,
            TALENT_CAP * campaign::XP_PER_TALENT
        );
    }

    #[test]
    fn a_comment_does_not_drop_the_first_character() {
        let text = "\
# survivors account
next 3
who 1 0 250 Anduin Wrynn 1,0,0,0,0,0,0,0,0,0,0,0,0,0,0
class 0 1100000000 0 1000000
seen 0 Amulet_of_Heroic_Strike
pets 2 3 1 1 2 0 1
";
        let account = parse(text).unwrap();
        assert_eq!(account.characters.len(), 1);
        assert_eq!(account.characters[0].first, "Anduin");
        assert_eq!(account.characters[0].last, "Wrynn");
        assert_eq!(account.characters[0].talent_xp, 250);
        assert_eq!(account.characters[0].ranks()[0], 1);
        assert_eq!(account.characters[0].spec, 0);
        assert_eq!(account.classes[0].pieces[0], 1);
        assert!(account.classes[0].clears[0]);
        assert!(!account.classes[0].clears[1]);
        let moved = parse("class 0 0000000000 0 1010001\nclass 1 0000000000 0 1001000\n").unwrap();
        assert!(moved.classes[0].clears[0] && moved.classes[0].clears[1]);
        assert!(moved.classes[0].clears[11]);
        assert!(!moved.classes[0].clears[6]);
        assert!(moved.classes[1].clears[0] && moved.classes[1].clears[6]);
        assert!(!moved.classes[1].clears[1]);
        assert_eq!(
            account.classes[0].seen,
            vec!["Amulet of heroic_strike".to_string()]
        );
        let again = parse(&encode(&account)).unwrap();
        assert_eq!(again.characters[0].ranks()[0], 1);
        assert_eq!(again.characters[0].spec, 0);
        assert_eq!(account.pets.owned, 2);
        assert_eq!(account.pets.leash, 3);
        assert_eq!(account.pets.instinct, [1, 2]);
        assert_eq!(again.characters[0].last, "Wrynn");
        assert_eq!(again.pets.owned, 2);
        assert_eq!(again.classes[0].pieces[1], 1);
    }

    #[test]
    fn jewelry_achievement_counts_all_codex_families_once_across_classes_and_reload() {
        use campaign::JewelKind::{Amulet, Legend, Ring, Trinket};
        let mut account = empty();
        for class in 0..data::CLASSES.len() {
            account.remember_jewelry(class, [Ring(0), Trinket(0), Amulet(0)]);
        }
        assert_eq!(account.snapshot().jewelry, 3);
        assert!(!account.grant_new_on(Some(20_752)));
        let target = rewards::jewelry_half_target();
        let mut count = 3;
        'classes: for (class, def) in data::CLASSES.iter().enumerate() {
            for key in def.ability_keys() {
                account.remember_jewelry(class, [Legend(key), Legend(key)]);
                count += 1;
                assert_eq!(account.snapshot().jewelry, count);
                if count == target - 1 {
                    assert!(
                        !account
                            .achievement_list()
                            .iter()
                            .find(|row| row.id == 44)
                            .unwrap()
                            .earned
                    );
                }
                if count == target {
                    break 'classes;
                }
            }
        }
        assert_eq!(count, target);
        assert!(account.grant_new_on(Some(20_752)));
        assert!(account.earned_ids().contains(&44));
        let points = account.unspent();
        let mut restored = parse(&encode(&account)).unwrap();
        assert_eq!(restored.snapshot().jewelry, target);
        assert_eq!(restored.earned_date(44), account.earned_date(44));
        restored.remember_jewelry(8, [Ring(0), Trinket(0), Amulet(0), Ring(u8::MAX)]);
        assert!(!restored.grant_new_on(Some(20_753)));
        assert_eq!(restored.unspent(), points);
        assert_eq!(restored.snapshot().jewelry, target);
        let mut legacy = parse("earn 44\n").unwrap();
        assert!(
            legacy
                .achievement_list()
                .iter()
                .find(|row| row.id == 44)
                .unwrap()
                .earned
        );
        assert!(!legacy.grant_new_on(Some(20_752)));
        assert_eq!(legacy.unspent(), 10);
    }

    #[test]
    fn shared_jewelry_discovery_is_unique_account_wide_and_survives_reload() {
        use campaign::JewelKind::{Amulet, Ring, Trinket};
        let mut account = empty();
        assert_eq!(
            account.shared_jewel_progress(2),
            (0, campaign::RING_NAMES.len())
        );
        account.remember_jewelry(0, [Ring(0), Trinket(1), Amulet(2)]);
        account.remember_jewelry(1, [Ring(0), Ring(1), Trinket(1)]);
        account.remember_jewelry(0, [Ring(u8::MAX), Amulet(u8::MAX)]);
        assert_eq!(account.seen_jewels.len(), 4);
        let restored = parse(&encode(&account)).unwrap();
        assert_eq!(restored.shared_jewel_progress(2).0, 2);
        assert_eq!(restored.shared_jewel_progress(3).0, 1);
        assert_eq!(restored.shared_jewel_progress(4).0, 1);
        assert!(restored
            .classes
            .iter()
            .all(|class| class.pieces.iter().all(|&p| p == 0)));
        assert_eq!(restored.runs, 0, "offering a card does not settle a run");
        assert!(restored.classes.iter().all(|class| class.seen.is_empty()));
        let mut restored = restored;
        restored.remember_jewelry(8, [Ring(0)]);
        assert_eq!(restored.seen_jewels.len(), 4);
    }

    #[test]
    fn only_presented_legends_and_valid_shared_entries_advance_discovery() {
        use campaign::JewelKind::{Legend, Ring};
        let mut account = empty();
        let keys = data::CLASSES[0].ability_keys();
        // A banish may roll several candidates but presents only the retained card.
        let candidates = [Legend(keys[0]), Legend(keys[1]), Ring(0)];
        account.remember_jewelry(0, candidates.into_iter().take(1));
        assert_eq!(account.legendary_progress(0).0, 1);
        assert!(!account.classes[0]
            .seen
            .contains(&campaign::legendary_amulet(keys[1])));
        assert_eq!(account.shared_jewel_progress(2).0, 0);
        account.remember_jewelry(0, [Legend("not_a_spell")]);
        assert_eq!(account.legendary_progress(0).0, 1);
        let serialized = encode(&account);
        let noisy = format!(
            "{serialized}{serialized}jewel_seen ring:Unknown future item\njewel_seen malformed\n"
        );
        let reloaded = parse(&noisy).unwrap();
        assert_eq!(reloaded.legendary_progress(0).0, 1);
        assert_eq!(reloaded.shared_jewel_progress(2).0, 0);
    }

    #[test]
    fn talent_cap_cloak_credit_cannot_cross_heroes_or_classes() {
        let mut account = empty();
        let warrior = account.ensure(0);
        let paladin = account.ensure(1);
        account
            .characters
            .iter_mut()
            .find(|hero| hero.id == paladin)
            .unwrap()
            .talent_xp = campaign::TALENT_CAP * campaign::XP_PER_TALENT;
        assert!(account.feats(1, paladin).talent_max);
        assert!(!account.feats(0, warrior).talent_max);
        assert!(!account.feats(0, paladin).talent_max);
        assert!(!account.feats(1, u32::MAX).talent_max);
        assert_eq!(
            account.bonus(0, paladin).might,
            account.bonus(0, warrior).might
        );
        assert_eq!(account.epic_cloaks(), 0);
    }

    #[test]
    fn raid_collection_rewards_count_distinct_ownership_without_completing_legacy_feats() {
        use survivors_core::{gear_tier::GearTier, supplemental_gear};
        let mut account = empty();
        for class in 0..9 {
            let hero = account.ensure(class);
            for slot in 0..8 {
                let piece = supplemental_gear::piece(class, slot, GearTier::RaidEpic).unwrap();
                assert!(account.supplemental.collect(class, &piece.key));
                assert!(!account.supplemental.collect(class, &piece.key));
                assert!(account.select_supplemental(class, &piece.key, true));
                let count = (class * 8 + slot + 1) as u32;
                let snap = account.snapshot();
                assert_eq!(snap.pieces, count);
                assert_eq!(snap.full_sets, 0, "raid armor has no main hand");
                assert_eq!(snap.best_legendary_set, 0);
                assert!(!snap.any_cloak);
                assert!(!account.feats(class, hero).collection);
                account.grant_new_on(Some(20_752));
                assert_eq!(account.earned.contains(&41), count >= 10);
                assert_eq!(account.earned.contains(&42), count >= 40);
                for id in [43, 46, 51] {
                    assert!(!account.earned.contains(&id));
                }
                account = parse(&encode(&account)).unwrap();
                assert_eq!(account.snapshot().pieces, count);
            }
        }
        for id in [41, 42] {
            assert_eq!(
                account
                    .earned
                    .iter()
                    .filter(|&&earned| earned == id)
                    .count(),
                1
            );
        }
        assert!(account
            .classes
            .iter()
            .all(|class| class.pieces.iter().all(|&n| n == 0)));
    }

    #[test]
    fn raid_full_set_requires_same_class_weapon_and_not_equipped_or_visible_choices() {
        use survivors_core::{gear_tier::GearTier, supplemental_gear};
        for class in 0..9 {
            let mut account = empty();
            for slot in 0..8 {
                let piece = supplemental_gear::piece(class, slot, GearTier::RaidEpic).unwrap();
                assert!(account.supplemental.collect(class, &piece.key));
            }
            let other = (class + 1) % 9;
            account.classes[other].pieces[8] = 1;
            assert_eq!(account.snapshot().full_sets, 0);
            let weapon = supplemental_gear::piece(class, 8, GearTier::Poor).unwrap();
            assert!(account.supplemental.collect(class, &weapon.key));
            assert_eq!(account.snapshot().full_sets, 1);
            account.grant_new_on(Some(20_752));
            assert!(account.earned.contains(&43));
            assert!(!account.snapshot().any_cloak);
            let restored = parse(&encode(&account)).unwrap();
            assert_eq!(restored.snapshot().full_sets, 1);
            assert_eq!(restored.earned.iter().filter(|&&id| id == 43).count(), 1);
            assert_eq!(
                restored.classes[class]
                    .equipment
                    .raid_pieces(class, &restored.supplemental),
                0
            );
        }
    }

    #[test]
    fn cloak_collection_requires_every_quality_while_full_set_still_means_nine_slots() {
        let mut account = empty();
        let hero = account.ensure(0);
        account.classes[0].pieces[..campaign::GRID_SLOTS].fill(255);
        assert!(!account.feats(0, hero).collection);
        assert_eq!(account.snapshot().full_sets, 1);
        assert!(!account.snapshot().any_cloak);
        account.grant_new_on(Some(20_752));
        assert!(
            account.earned.contains(&43),
            "the nine-slot set achievement is separate"
        );
        assert!(!account.earned.contains(&51));
        let mut legacy = parse(&encode(&account)).unwrap();
        legacy.earned.push(51);
        let legacy = parse(&encode(&legacy)).unwrap();
        assert!(
            legacy.earned.contains(&51),
            "retain already-earned achievement points"
        );
        assert!(
            !legacy.snapshot().any_cloak,
            "legacy awards cannot supply cloak power"
        );
        account.classes[0].pieces.fill(1);
        account.classes[0].pieces[campaign::PIECES - 1] = 0;
        assert!(!account.feats(0, hero).collection);
        assert!(!account.snapshot().any_cloak);
        account.classes[1].pieces[campaign::PIECES - 1] = 1;
        assert!(
            !account.feats(0, hero).collection,
            "another class cannot fill the gap"
        );
        let before = account.bonus(0, hero);
        account.classes[0].pieces[campaign::PIECES - 1] = 1;
        let after = account.bonus(0, hero);
        let cloak = campaign::cloak_bonus(Quality::Uncommon);
        assert!((after.might - before.might - cloak.might).abs() < 0.00001);
        assert!((after.speed - before.speed - cloak.speed).abs() < 0.00001);
        assert!(account.feats(0, hero).collection);
        assert!(account.snapshot().any_cloak);
        assert_eq!(
            campaign::cloak_quality(account.feats(0, hero)),
            Some(Quality::Uncommon)
        );
        account.grant_new_on(Some(20_752));
        assert!(account.earned.contains(&51));
        let restored = parse(&encode(&account)).unwrap();
        assert_eq!(restored.earned.iter().filter(|&&id| id == 51).count(), 1);
        assert!(restored.feats(0, hero).collection);
        assert!(restored.snapshot().any_cloak);
        assert_eq!(restored.snapshot().full_sets, 1);
    }

    #[test]
    fn class_amulet_completion_matches_its_own_catalog_and_survives_reload() {
        for (class, def) in data::CLASSES.iter().enumerate() {
            let mut account = empty();
            let hero = account.ensure(class);
            let keys = def.ability_keys();
            assert!(!keys.is_empty());
            for key in &keys[..keys.len() - 1] {
                account.remember_amulets(class, &[*key]);
            }
            assert!(
                !account.feats(class, hero).amulets,
                "{} missing final amulet",
                def.name
            );
            account.remember_amulets(class, &keys);
            assert!(account.feats(class, hero).amulets, "{} complete", def.name);
            assert_eq!(account.legendary_progress(class), (keys.len(), keys.len()));
            assert_eq!(account.snapshot().legends, keys.len() as u32);
            let reloaded = parse(&encode(&account)).unwrap();
            assert!(reloaded.feats(class, hero).amulets);
            assert_eq!(reloaded.snapshot().legends, keys.len() as u32);
        }
    }

    #[test]
    fn duplicate_and_foreign_discoveries_cannot_inflate_rewards() {
        let name = campaign::legendary_amulet(data::CLASSES[0].start);
        let mut text = String::new();
        for _ in 0..data::ABILITIES.len() {
            text.push_str(&format!("seen 0 {}\n", escape(&name)));
        }
        text.push_str("seen 0 Amulet_of_a_nonexistent_spell\n");
        let account = parse(&text).unwrap();
        assert_eq!(
            account.classes[0].seen.len(),
            2,
            "retain unknown legacy names, deduplicate rows"
        );
        assert_eq!(account.legendary_progress(0).0, 1);
        assert_eq!(account.snapshot().legends, 1);
        assert!(!account.feats(0, 0).amulets);
        assert!(!account.legendary_complete(usize::MAX));
    }

    #[test]
    fn legendary_set_tracks_one_class_and_duplicate_drops_or_save_ids_cannot_inflate_it() {
        let mut account = empty();
        let drops: Vec<_> = (0..campaign::GRID_SLOTS)
            .map(|slot| {
                campaign::DROPS
                    .iter()
                    .find(|row| row.band == 5 && row.slot == slot)
                    .unwrap()
            })
            .collect();
        for drop in &drops[..8] {
            assert!(account.collect_drop(0, drop.zone, drop.creature).is_some());
            assert!(account.collect_drop(0, drop.zone, drop.creature).is_none());
        }
        let last = drops[8];
        account.collect_drop(1, last.zone, last.creature);
        // A full lower-rarity class and nine legendaries split between classes do not qualify.
        account.classes[2].pieces[..campaign::GRID_SLOTS].fill(1);
        account.grant_new_on(Some(20_752));
        assert_eq!(account.snapshot().best_legendary_set, 8);
        assert!(!account.earned_ids().contains(&46));
        account.collect_drop(0, last.zone, last.creature);
        assert!(account.grant_new_on(Some(20_752)));
        assert_eq!(account.snapshot().best_legendary_set, 9);
        assert!(account.earned_ids().contains(&46));
        let points = account.unspent();
        let text = format!("{}earn 46,46\n", encode(&account));
        let mut restored = parse(&text).unwrap();
        assert_eq!(
            restored.earned_ids().iter().filter(|&&id| id == 46).count(),
            1
        );
        assert_eq!(restored.unspent(), points);
        assert!(!restored.grant_new_on(Some(20_753)));
        assert_eq!(restored.earned_date(46), account.earned_date(46));
    }

    #[test]
    fn a_cleared_zone_keeps_its_pieces_and_opens_the_next() {
        let mut account = empty();
        let id = account.ensure(0);
        account.grant_zone(0, 0);
        account.see_amulet(0, "Amulet of Heroic Strike");
        assert_eq!(account.classes[0].seen.len(), 1);
        assert!(account.classes[0].pieces.iter().all(|n| *n == 0));
        assert_eq!(
            account.collect_drop(0, 0, "Hogger"),
            Some("Field Plate Helmet")
        );
        assert_eq!(account.classes[0].pieces[0], 1);
        account.collect_drop(0, 0, "Hogger");
        assert_eq!(account.classes[0].pieces[0], 1);
        assert!(!campaign::zone_open(
            1,
            campaign::talent_level(account.character(id).unwrap().talent_xp),
            &account.classes[0].clears,
        ));
        account.add_talent_xp(id, 500);
        assert!(campaign::zone_open(
            1,
            campaign::talent_level(account.character(id).unwrap().talent_xp),
            &account.classes[0].clears,
        ));
    }

    #[test]
    fn the_raid_grants_the_legendary_weapon() {
        let mut account = empty();
        account.grant_zone(0, campaign::ZONES.len() - 1);
        assert!(!account.classes[0].legendary_weapon);
        assert_eq!(
            account.collect_drop(0, campaign::ZONES.len() - 1, "Kel'Thuzad"),
            Some("Thunderfury, Blessed Blade of the Windseeker")
        );
        assert!(account.classes[0].legendary_weapon);
        let id = account.ensure(0);
        account.add_talent_xp(id, 600);
        assert!(account.learn(id, 0));
        assert!(account.learn(id, 1));
        assert!(!account.learn(id, 2));
        for _ in 0..3 {
            assert!(account.learn(id, 0));
        }
        assert!(account.learn(id, 2));
        assert!(account.bonus(0, id).might > 0.0);
        account.select_spec(id, 1);
        assert_eq!(account.character(id).unwrap().ranks()[0], 0);
        assert!(account.learn(id, 0));
        account.select_spec(id, 0);
        assert_eq!(account.character(id).unwrap().ranks()[0], 4);
        account.respec(id);
        assert_eq!(account.character(id).unwrap().ranks()[0], 0);
        account.select_spec(id, 1);
        assert_eq!(account.character(id).unwrap().ranks()[0], 1);
        assert!(account.unlearn(id, 0));
        assert_eq!(account.character(id).unwrap().ranks()[0], 0);
        let bonus = account.bonus(0, 1);
        assert!(bonus.crit_damage > 0.0);
    }

    #[test]
    fn stale_or_wrapped_character_cursors_never_alias_existing_heroes() {
        for cursor in ["", "next 0\n", "next 1\n", "next 4294967295\n"] {
            let mut account = parse(&format!(
                "{cursor}who 1 0 500 First Hero 0|||\nwho 4294967295 1 900 Last Hero 0|||\nbody 1 1 0\nbody 4294967295 3 1\n"
            )).unwrap();
            let created = account.create("New", "Hero", 0).unwrap();
            let starter = account.ensure(2);
            assert_ne!(created, 0);
            assert_ne!(starter, 0);
            assert_ne!(created, starter);
            assert!(![1, u32::MAX].contains(&created));
            assert!(![1, u32::MAX].contains(&starter));
            account.add_talent_xp(created, 100);
            account.set_body(starter, (3, 0));
            let loaded = parse(&encode(&account)).unwrap();
            assert_eq!(loaded.characters.len(), 4);
            assert_eq!(loaded.character(1).unwrap().talent_xp, 500);
            assert_eq!(loaded.character(1).unwrap().body, Some((1, 0)));
            assert_eq!(loaded.character(u32::MAX).unwrap().talent_xp, 900);
            assert_eq!(loaded.character(u32::MAX).unwrap().body, Some((3, 1)));
            assert_eq!(loaded.character(created).unwrap().talent_xp, 100);
            assert_eq!(loaded.character(starter).unwrap().body, Some((3, 0)));
            assert_eq!(loaded.next_id, account.next_id);
        }
    }

    #[test]
    fn explicit_invalid_hero_selection_cannot_borrow_another_sheet() {
        let mut account = empty();
        let id = account.create("Trained", "Paladin", 1).unwrap();
        account.add_talent_xp(id, 500);
        assert!(account.learn(id, 0));
        assert!(account.bonus_picked(1, Some(id)).might > 0.0);
        assert_eq!(
            account.bonus_picked(1, None).might,
            account.bonus(1, id).might
        );
        for (class, picked) in [(0, id), (1, u32::MAX)] {
            let bonus = account.bonus_picked(class, Some(picked));
            assert_eq!(bonus.might, 0.0);
            assert_eq!(bonus.health, 0.0);
            assert_eq!(
                account.milestones_picked(class, Some(picked)).points,
                [0; 3]
            );
        }
    }

    #[test]
    fn a_named_hero_keeps_their_own_talents() {
        let mut account = empty();
        assert!(account.create(" ", "Wrynn", 1).is_none());
        let anduin = account.create("Anduin", "Wrynn", 1).unwrap();
        let uther = account.create("Uther", "Lightbringer", 1).unwrap();
        account.add_talent_xp(anduin, 500);
        assert!(account.learn(anduin, 0));
        let anduin_might = account.bonus_picked(1, Some(anduin)).might;
        let uther_might = account.bonus_picked(1, Some(uther)).might;
        assert!(anduin_might > uther_might);
    }

    #[test]
    fn achievement_dates_record_once_roundtrip_and_never_backfill_or_grant() {
        let mut account = empty();
        account.runs = 10;
        assert!(account.grant_new_on(Some(20_695)));
        assert_eq!(account.earned_date(64).as_deref(), Some("30 Aug 2026 UTC"));
        let points = account.unspent();
        assert!(!account.grant_new_on(Some(20_696)));
        assert_eq!(account.unspent(), points);
        let mut loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.earned_dates, account.earned_dates);
        loaded.runs = 100;
        assert!(loaded.grant_new_on(Some(20_696)));
        assert_eq!(loaded.earned_date(64), account.earned_date(64));
        assert_eq!(loaded.earned_date(65).as_deref(), Some("31 Aug 2026 UTC"));

        let mut legacy = parse("earn 64\n").unwrap();
        legacy.runs = 10;
        assert!(!legacy.grant_new_on(Some(20_695)));
        assert!(legacy.earned_date(64).is_none());
        assert!(parse(&encode(&legacy)).unwrap().earned_date(64).is_none());

        let metadata = format!(
            "earned_date 64 20695\nearned_date 64 20696\nearned_date 65 20695\nearned_date {} 20695\nearned_date 63 4294967295\nearned_date 62 -1\nearned_date 61 bad\nearn 64,63,62,61,{}\n",
            rewards::DEATHLESS_HOUR, rewards::DEATHLESS_HOUR,
        );
        let loaded = parse(&metadata).unwrap();
        assert_eq!(loaded.earned_dates.len(), 1);
        assert_eq!(loaded.earned_date(64), account.earned_date(64));
        assert!(!loaded.earned_ids().contains(&65));
        assert_eq!(loaded.unspent(), rewards::points_of(loaded.earned_ids()));

        for missing in [None, Some(u32::MAX)] {
            let mut account = empty();
            account.runs = 10;
            assert!(account.grant_new_on(missing));
            assert!(account.earned_dates.is_empty());
            assert!(account.earned_ids().contains(&64));
            assert!(!account.grant_new_on(Some(20_695)));
            assert!(account.earned_dates.is_empty());
        }
    }

    #[test]
    fn achievement_points_buy_a_perk_and_a_reset_keeps_the_companion() {
        let mut account = empty();
        for zone in 0..campaign::ZONES.len() {
            account.grant_zone(0, zone);
        }
        assert!(account.grant_new());
        assert!(account.unspent() >= 30);
        assert!(account.buy_perk(Perk::SecondThoughts).is_ok());
        assert_eq!(account.perks.second_thoughts, 1);
        assert!(account.buy_leash().is_ok());
        assert_eq!(account.pets.leash, 1);
        let after = account.unspent();
        assert!(account.reset_perks().is_ok());
        assert_eq!(account.perks.second_thoughts, 0);
        assert_eq!(account.pets.leash, 1);
        assert_eq!(account.resets_left(), 1);
        assert!(account.unspent() > after);
        assert!(!account.grant_new());
        let list = account.achievement_list();
        assert!(list
            .iter()
            .any(|row| row.earned && row.name.contains("Elwynn")));
        assert!(list.iter().any(|row| !row.earned));
        let text = encode(&account);
        let loaded = parse(&text).unwrap();
        assert_eq!(loaded.pets.leash, 1);
        assert_eq!(loaded.resets_left(), 1);
        assert!(!loaded.earned_ids().is_empty());
    }

    #[test]
    fn companion_purchases_match_the_displayed_destination_and_preserve_refunds() {
        let mut account = empty();
        for class in 0..data::CLASSES.len() {
            for zone in 0..campaign::ZONES.len() {
                account.grant_zone(class, zone);
            }
        }
        account.grant_new();
        let before = encode(&account);
        for kind in [0, 3, 4, 255] {
            assert!(account.buy_instinct(kind).is_err());
            assert_eq!(encode(&account), before);
        }
        account.wear_look(1, 1);
        assert_eq!(encode(&account), before);
        assert_eq!(account.instinct_target(), Some(0));
        let points = account.unspent();
        account.buy_instinct(2).unwrap();
        assert_eq!(account.pets.instinct, [2, 0]);
        assert_eq!(account.unspent(), points - 30);
        let duplicate = encode(&account);
        assert!(account.buy_instinct(2).is_err());
        assert_eq!(encode(&account), duplicate);
        account.buy_second().unwrap();
        assert_eq!(account.instinct_target(), Some(1));
        account.buy_instinct(1).unwrap();
        assert_eq!(account.pets.instinct, [2, 1]);
        assert_eq!(account.instinct_target(), Some(0));
        account.buy_instinct(1).unwrap();
        assert_eq!(account.pets.instinct, [1, 1]);
        assert_eq!(account.unspent(), points - 130);
        account.buy_perk(Perk::SecondThoughts).unwrap();
        account.reset_perks().unwrap();
        assert_eq!(account.pets.instinct, [1, 1]);
        assert_eq!(account.unspent(), points - 130);
        let restored = parse(&encode(&account)).unwrap();
        assert_eq!(restored.pets.instinct, [1, 1]);
        assert_eq!(restored.pets.owned, 2);
        let mut poor = empty();
        assert!(poor.buy_instinct(1).is_err());
        assert_eq!(poor.pets.instinct, [0, 0]);
        assert_eq!(poor.unspent(), 0);
        poor.pets.owned = 0;
        assert_eq!(poor.instinct_target(), None);
    }

    #[test]
    fn placeholder_companion_looks_stay_locked_even_with_saved_achievement_ids() {
        let mut account = empty();
        account
            .earned
            .extend([rewards::DEATHLESS_HOUR, rewards::NO_U]);
        for look in [11, 12] {
            assert!(!account.look_unlocked(look));
            account.wear_look(0, look);
            assert_eq!(account.pets.look[0], 0);
        }
        let restored = parse(&encode(&account)).unwrap();
        assert!(!restored.look_unlocked(11));
        assert!(!restored.look_unlocked(12));
    }

    #[test]
    fn saved_companion_looks_respect_unlocks_in_both_slots_after_all_rows_load() {
        let mut account = empty();
        account.pets.owned = 2;
        // Placeholder achievement IDs must never unlock their associated appearances.
        account
            .earned
            .extend([rewards::DEATHLESS_HOUR, rewards::NO_U]);
        for look in [2, 11, 12, 13, 255] {
            account.pets.look = [look, look];
            let loaded = parse(&encode(&account)).unwrap();
            assert_eq!(loaded.pets.look, [0, 0], "unavailable look {look}");
            assert_eq!(parse(&encode(&loaded)).unwrap().pets.look, [0, 0]);
        }
        for look in [0, 1, 3, 4, 5, 6, 7, 8, 9, 10] {
            account.pets.look = [look, look];
            assert_eq!(parse(&encode(&account)).unwrap().pets.look, [look, look]);
        }
        account.pets.look = [2, 1];
        account.earned.push(65);
        let save = encode(&account);
        let rows: Vec<_> = save.lines().collect();
        let reversed = rows.into_iter().rev().collect::<Vec<_>>().join("\n");
        for save in [&save, &reversed] {
            let loaded = parse(save).unwrap();
            assert_eq!(loaded.pets.look, [2, 1]);
            assert_eq!(loaded.pets.instinct, account.pets.instinct);
            assert_eq!(loaded.unspent(), account.unspent());
        }
        account.earned.clear();
        account.pets.legacy_chicken = true;
        assert_eq!(parse(&encode(&account)).unwrap().pets.look, [2, 1]);
        // An unowned second slot may retain a legitimate cosmetic for a later purchase.
        account.pets.owned = 1;
        account.pets.look = [1, 10];
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.pets.owned, 1);
        assert_eq!(loaded.pets.look, [1, 10]);
    }

    #[test]
    fn chicken_unlock_requires_veteran_but_preserves_legacy_access() {
        let mut account = empty();
        assert!(!account.look_unlocked(2));
        account.wear_look(0, 2);
        assert_eq!(account.pets.look[0], 0);
        account.earned.push(65);
        account.wear_look(0, 2);
        assert_eq!(account.pets.look[0], 2);
        assert!(parse(&encode(&account)).unwrap().look_unlocked(2));
        let fresh = empty();
        assert!(!parse(&encode(&fresh)).unwrap().look_unlocked(2));
        let legacy = encode(&fresh).replace("pets 1 0 0 0 0 0 0 0", "pets 1 0 0 0 0 0 0");
        let migrated = parse(&legacy).unwrap();
        assert!(migrated.look_unlocked(2));
        assert!(parse(&encode(&migrated)).unwrap().look_unlocked(2));
        assert!(!migrated.look_unlocked(255));
    }

    #[test]
    fn a_tabard_and_a_look_do_not_change_the_sheet() {
        let mut account = empty();
        let id = account.ensure(0);
        let before = account.bonus(0, id).might;
        account.wear_tabard(1);
        assert_eq!(account.tabard, 1);
        assert_eq!(campaign::TABARDS[0].display, 20621);
        assert_eq!(account.bonus(0, id).might, before);
        account.wear_tabard(9);
        assert_eq!(account.tabard, 1);
        account.wear_look(0, 1);
        assert_eq!(account.pets.look[0], 1);
        account.wear_look(0, 99);
        assert_eq!(account.pets.look[0], 1);
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.tabard, 1);
        assert_eq!(loaded.pets.look[0], 1);
    }

    #[test]
    fn a_real_talent_replaces_the_skeleton_and_survives_a_save() {
        let mut account = empty();
        let id = account.ensure(0);
        account.add_talent_xp(id, 600);
        assert!(account.learn(id, 0));
        assert!(account.learn_talent(id, 0, 99, 5, 0, true));
        assert_eq!(account.character(id).unwrap().ranks()[0], 0);
        assert_eq!(account.character(id).unwrap().dbc_rank(99), 1);
        assert!(!account.learn_talent(id, 0, 100, 1, 2, true));
        assert!(account.bonus(0, id).might > 0.0);
        let loaded = parse(&encode(&account)).unwrap();
        assert_eq!(loaded.character(id).unwrap().dbc_rank(99), 1);
        assert!(account.unlearn_talent(id, 99, true));
        assert_eq!(account.character(id).unwrap().dbc_rank(99), 0);
    }
}

#[cfg(test)]
#[path = "account_recent_talents_tests.rs"]
mod recent_talents_tests;
