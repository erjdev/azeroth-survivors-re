//! The account save: named characters, and the permanent progress shared by a class.

use bevy::prelude::*;

use super::campaign::{self, Bonus, Feats, Quality, TALENT_CAP};
use super::data;
use super::rewards::{self, Perk, PerkRanks};
use crate::local_state;

const FILE: &str = "benilla-config/survivors-account.txt";

#[derive(Clone, Debug)]
pub(super) struct Character {
    pub id: u32,
    pub first: String,
    pub last: String,
    /// Index into [`data::CLASSES`].
    pub class: usize,
    pub talent_xp: u32,
    /// Which of the three saved specs the run uses.
    pub spec: u8,
    /// Three specs, each three trees of [`campaign::NODES_PER_TREE`] ranks.
    pub specs: [[u8; campaign::SPEC_LEN]; 3],
    /// Real `Talent.dbc` ranks for the three specs: tree index, talent id, rank.
    /// Empty means the skeleton above is what the run still uses.
    pub dbc: [Vec<(u8, u32, u8)>; 3],
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

    pub(super) fn dbc_rank(&self, talent: u32) -> u8 {
        let spec = usize::from(self.spec).min(2);
        self.dbc[spec]
            .iter()
            .find(|entry| entry.1 == talent)
            .map(|entry| entry.2)
            .unwrap_or(0)
    }
}

fn blank_dbc() -> [Vec<(u8, u32, u8)>; 3] {
    [Vec::new(), Vec::new(), Vec::new()]
}

fn blank_specs() -> [[u8; campaign::SPEC_LEN]; 3] {
    [[0; campaign::SPEC_LEN]; 3]
}

#[derive(Clone, Debug)]
pub(super) struct ClassProgress {
    /// Ownership in the nine-by-six grid: zero is unseen, one is collected.
    pub pieces: [u8; campaign::PIECES],
    pub equipment: super::equipment::Equipment,
    pub legendary_weapon: bool,
    pub clears: [bool; campaign::CLEAR_COUNT],
    /// Legendary amulet names whose card has been dealt.
    pub seen: Vec<String>,
}

impl Default for ClassProgress {
    fn default() -> Self {
        Self {
            pieces: [0; campaign::PIECES],
            equipment: super::equipment::Equipment::default(),
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
}

impl Default for PetRanks {
    fn default() -> Self {
        Self {
            owned: 1,
            leash: 0,
            paws: 0,
            instinct: [0, 0],
            look: [0, 0],
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub(super) struct Account {
    pub next_id: u32,
    pub characters: Vec<Character>,
    pub classes: Vec<ClassProgress>,
    pub pets: PetRanks,
    pub perks: PerkRanks,
    earned: Vec<u16>,
    spent: u32,
    perk_spent: u32,
    resets: u8,
    pub runs: u32,
    pub wins: u32,
    /// 0 is no tabard. 1 is the first checked cosmetic in [`campaign::TABARDS`].
    pub tabard: u8,
    dirty: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self::load()
    }
}

impl Account {
    fn load() -> Self {
        let text = local_state::home()
            .map(|h| h.join(FILE))
            .filter(|p| p.is_file())
            .and_then(|p| local_state::read_to_string(&p).ok())
            .unwrap_or_default();
        parse(&text).unwrap_or_else(|_| empty())
    }

    pub fn save(&mut self) {
        if !self.dirty {
            return;
        }
        let Some(home) = local_state::home() else {
            return;
        };
        if local_state::write_atomic(&home.join(FILE), &encode(self)).is_ok() {
            self.dirty = false;
        }
    }

    pub fn character(&self, id: u32) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    /// The first character of a class, creating "Adventurer" of that class when none exists.
    /// The character screen will add named ones beside this.
    pub fn ensure(&mut self, class: usize) -> u32 {
        if let Some(c) = self.characters.iter().find(|c| c.class == class) {
            return c.id;
        }
        let id = self.next_id;
        self.next_id += 1;
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
        });
        self.dirty = true;
        id
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
        if progress.equipment.worn[row.slot].is_none() {
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
        let Some(progress) = self.classes.get_mut(class) else {
            return;
        };
        if !progress.seen.iter().any(|s| s == name) {
            progress.seen.push(name.to_string());
            self.dirty = true;
        }
    }

    pub fn feats(&self, class: usize, id: u32) -> Feats {
        let progress = self.classes.get(class);
        let character = self.character(id);
        let talent_max = character
            .map(|c| campaign::talent_level(c.talent_xp) >= TALENT_CAP)
            .unwrap_or(false);
        let raid = progress
            .and_then(|p| p.clears.last().copied())
            .unwrap_or(false);
        let collection = progress
            .map(|p| campaign::set_complete(&p.pieces))
            .unwrap_or(false);
        let amulets = progress
            .map(|p| p.seen.len() >= data::ABILITIES.len())
            .unwrap_or(false);
        Feats {
            talent_max,
            raid,
            collection,
            amulets,
        }
    }

    /// Permanent bonuses for a character: class gear, set, cloak, and the legendary cloak once
    /// every class has its epic.
    pub fn bonus(&self, class: usize, id: u32) -> Bonus {
        let Some(progress) = self.classes.get(class) else {
            return Bonus::default();
        };
        let pieces = progress.equipment.worn_pieces(&progress.pieces);
        let hero = self.character(id);
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
        let mut total = campaign::equipped_bonus(&pieces, cloak);
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
        let id = self.next_id;
        self.next_id += 1;
        self.characters.push(Character {
            id,
            first: first.to_string(),
            last: last.to_string(),
            class,
            talent_xp: 0,
            spec: 0,
            specs: blank_specs(),
            dbc: blank_dbc(),
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

    pub fn bonus_for(&self, class: usize) -> Bonus {
        let id = self
            .characters
            .iter()
            .find(|c| c.class == class)
            .map(|c| c.id)
            .unwrap_or(0);
        self.bonus(class, id)
    }

    /// The picked character's sheet when one is selected, otherwise the class's first hero.
    pub fn bonus_picked(&self, class: usize, character: Option<u32>) -> Bonus {
        if let Some(id) = character {
            if let Some(hero_class) = self.character(id).map(|c| c.class) {
                return self.bonus(hero_class, id);
            }
        }
        self.bonus_for(class)
    }

    fn legendary_ready(&self) -> bool {
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
        epics >= 9
    }

    pub fn unspent(&self) -> u32 {
        rewards::points_of(&self.earned).saturating_sub(self.spent)
    }

    pub fn earned_ids(&self) -> &[u16] {
        &self.earned
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
        let found = rewards::fresh(&self.snapshot(), &self.earned);
        if found.is_empty() {
            return false;
        }
        for (id, _) in found {
            self.earned.push(id);
        }
        self.dirty = true;
        true
    }

    pub fn buy_perk(&mut self, perk: Perk) -> Result<(), &'static str> {
        let Some(cost) = rewards::perk_cost(perk, self.perks.rank(perk)) else {
            return Err("Max rank");
        };
        self.pay(cost)?;
        self.perks.raise(perk);
        self.perk_spent += cost;
        self.save();
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
        self.save();
        Ok(())
    }

    /// Puts this instinct on the first companion who does not have one, else the first companion.
    pub fn buy_instinct(&mut self, kind: u8) -> Result<(), &'static str> {
        let kind = kind.clamp(1, 3);
        let slot = if self.pets.instinct[0] == 0 {
            0
        } else if self.pets.owned > 1 && self.pets.instinct[1] == 0 {
            1
        } else {
            0
        };
        if self.pets.instinct[slot] == kind {
            return Err("Already learned");
        }
        self.pay(30)?;
        self.pets.instinct[slot] = kind;
        self.save();
        Ok(())
    }

    /// Wear one of the checked critter looks. An unknown index leaves the slot as it was.
    pub fn wear_look(&mut self, slot: u8, look: u8) {
        let slot = usize::from(slot);
        if slot > 1 || usize::from(look) >= super::companions::LOOKS.len() {
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
        self.save();
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
        self.save();
        Ok(())
    }

    fn snapshot(&self) -> rewards::Snapshot {
        let mut snap = rewards::Snapshot::default();
        let mut epics = 0u32;
        for (i, class) in self.classes.iter().enumerate().take(9) {
            let n = snap.clears[i].len().min(class.clears.len());
            snap.clears[i][..n].copy_from_slice(&class.clears[..n]);
            snap.pieces += campaign::owned_pieces(&class.pieces) as u32;
            if campaign::set_complete(&class.pieces) {
                snap.full_sets += 1;
            }
            snap.legends += class.seen.len() as u32;
            let talent_max = self
                .characters
                .iter()
                .any(|c| c.class == i && campaign::talent_level(c.talent_xp) >= TALENT_CAP);
            snap.talent_cap[i] = talent_max;
            let feats = Feats {
                talent_max,
                raid: class.clears.last().copied().unwrap_or(false),
                collection: campaign::set_complete(&class.pieces),
                amulets: class.seen.len() >= data::ABILITIES.len(),
            };
            if campaign::cloak_quality(feats).is_some() {
                snap.any_cloak = true;
            }
            if campaign::cloak_quality(feats) == Some(Quality::Epic) {
                epics += 1;
            }
        }
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

fn empty() -> Account {
    Account {
        next_id: 1,
        characters: Vec::new(),
        classes: vec![ClassProgress::default(); data::CLASSES.len()],
        pets: PetRanks::default(),
        perks: PerkRanks::default(),
        earned: Vec::new(),
        spent: 0,
        perk_spent: 0,
        resets: 2,
        runs: 0,
        wins: 0,
        tabard: 0,
        dirty: false,
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
        for name in &class.seen {
            out.push_str(&format!("seen {i} {}\n", escape(name)));
        }
    }
    out.push_str(&format!(
        "pets {} {} {} {} {} {} {}\n",
        account.pets.owned,
        account.pets.leash,
        account.pets.paws,
        account.pets.instinct[0],
        account.pets.instinct[1],
        account.pets.look[0],
        account.pets.look[1],
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
    out
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

fn encode_dbc(specs: &[Vec<(u8, u32, u8)>; 3]) -> String {
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

fn read_dbc(payload: &str) -> [Vec<(u8, u32, u8)>; 3] {
    let mut specs = blank_dbc();
    for (spec, group) in payload.split('|').enumerate().take(3) {
        for entry in group.split(',').filter(|s| !s.is_empty()) {
            let mut bits = entry.split(':');
            let tree = bits.next().unwrap_or("0").parse::<u8>().unwrap_or(0).min(2);
            let id = bits.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
            let rank = bits.next().unwrap_or("0").parse::<u8>().unwrap_or(0);
            if id != 0 && rank > 0 {
                specs[spec].push((tree, id, rank));
            }
        }
    }
    specs
}

fn parse(text: &str) -> Result<Account, ()> {
    let mut account = empty();
    let mut equipment_rows = Vec::new();
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
                if class < data::CLASSES.len() {
                    account.characters.push(Character {
                        id,
                        first,
                        last,
                        class,
                        talent_xp,
                        spec,
                        specs,
                        dbc: blank_dbc(),
                    });
                }
            }
            Some("dbc") => {
                let id: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
                let payload = parts.next().unwrap_or("");
                if let Some(hero) = account.characters.iter_mut().find(|c| c.id == id) {
                    hero.dbc = read_dbc(payload);
                }
            }
            Some("class") => {
                let index: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let gear = parts.next().unwrap_or("");
                let legendary = parts.next().unwrap_or("0") == "1";
                let clears = parts.next().unwrap_or("");
                if let Some(progress) = account.classes.get_mut(index) {
                    read_pieces(progress, gear, legendary);
                    progress.clears = read_clears(clears);
                }
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
                let name = unescape(&parts.collect::<Vec<_>>().join(" "));
                if let Some(progress) = account.classes.get_mut(index) {
                    if !name.is_empty() {
                        progress.seen.push(name);
                    }
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
                };
            }
            Some("meta") => {
                account.runs = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.wins = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.spent = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.perk_spent = parts.next().unwrap_or("0").parse().unwrap_or(0);
                account.resets = parts.next().unwrap_or("2").parse().unwrap_or(2);
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
            Some("earn") => {
                if let Some(list) = parts.next() {
                    for id in list.split(',') {
                        if let Ok(id) = id.parse() {
                            account.earned.push(id);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for (index, progress) in account.classes.iter_mut().enumerate() {
        progress.equipment = if let Some((_, worn, looks)) =
            equipment_rows.iter().rev().find(|row| row.0 == index)
        {
            super::equipment::Equipment::decode(worn, looks, &progress.pieces)
        } else {
            super::equipment::Equipment::legacy(&progress.pieces)
        };
    }
    Ok(account)
}

/// Keeps legendary amulets that were dealt, when a run is abandoned back to the menu.
pub(super) fn keep_seen(
    mut account: ResMut<Account>,
    selection: Res<super::Selection>,
    run: Res<super::Run>,
) {
    let Some(class) = selection.class else {
        return;
    };
    account.remember_amulets(class, &run.seen_legends);
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
    account.remember_amulets(class, &run.seen_legends);
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
    use super::*;

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
            vec!["Amulet of Heroic Strike".to_string()]
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
        assert!(
            campaign::zone_open(
                1,
                campaign::talent_level(account.character(id).unwrap().talent_xp),
                &account.classes[0].clears,
            ) == false
        );
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
