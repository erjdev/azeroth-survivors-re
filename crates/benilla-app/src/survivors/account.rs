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
    /// Fifteen nodes: five per tree.
    pub talents: [u8; 18],
}

#[derive(Clone, Debug, Default)]
pub(super) struct ClassProgress {
    pub gear: [bool; 10],
    pub legendary_weapon: bool,
    pub clears: [bool; 7],
    /// Legendary amulet names whose card has been dealt.
    pub seen: Vec<String>,
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
        let _ = local_state::write_atomic(&home.join(FILE), &encode(self));
        self.dirty = false;
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
            talents: [0; 18],
        });
        self.dirty = true;
        id
    }

    pub fn add_talent_xp(&mut self, id: u32, xp: u32) {
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return;
        };
        let cap_xp = TALENT_CAP * campaign::XP_PER_TALENT;
        c.talent_xp = (c.talent_xp + xp).min(cap_xp);
        self.dirty = true;
    }

    pub fn grant_zone(&mut self, class: usize, zone: usize) {
        let Some(progress) = self.classes.get_mut(class) else {
            return;
        };
        if let Some(flag) = progress.clears.get_mut(zone) {
            *flag = true;
        }
        for slot in 0..campaign::SLOTS.len() {
            if campaign::slot_zone(slot) == Some(zone) {
                if let Some(piece) = progress.gear.get_mut(slot) {
                    *piece = true;
                }
            }
        }
        if zone + 1 == campaign::ZONES.len() {
            progress.legendary_weapon = true;
        }
        self.dirty = true;
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
        let collection = progress.map(|p| p.gear.iter().all(|g| *g)).unwrap_or(false);
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
        let gear = progress.gear;
        let legendary_weapon = progress.legendary_weapon;
        let talents = self.character(id).map(|c| c.talents).unwrap_or([0; 18]);
        let cloak = campaign::cloak_quality(self.feats(class, id));
        let mut total = campaign::equipped_bonus(
            &gear,
            legendary_weapon,
            cloak,
            campaign::two_handed(class) || (legendary_weapon && matches!(class, 1 | 5)),
        );
        if self.legendary_ready() {
            let cloak = campaign::cloak_bonus(Quality::Legendary);
            total.might += cloak.might;
            total.reduction += cloak.reduction;
            total.multistrike += cloak.multistrike;
            total.leech += cloak.leech;
            total.crit_damage += cloak.crit_damage;
            total.targets += cloak.targets;
        }
        total.add(campaign::talent_bonus(&talents));
        total
    }

    /// Spend one talent point. A row stays shut until this tree holds 5 points for each step down.
    pub fn learn(&mut self, id: u32, node: usize) -> bool {
        let Some(c) = self.characters.iter().find(|c| c.id == id) else {
            return false;
        };
        let spent = c.talents.iter().map(|n| u32::from(*n)).sum();
        let unspent = campaign::unspent_talents(c.talent_xp, spent);
        let ranks = c.talents;
        if !campaign::can_learn(&ranks, node, unspent) {
            return false;
        }
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        c.talents[node] = c.talents[node].saturating_add(1);
        self.dirty = true;
        true
    }

    /// Respecs are free. Points return to the unspent pool.
    pub fn respec(&mut self, id: u32) {
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else {
            return;
        };
        c.talents = [0; 18];
        self.dirty = true;
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
            talents: [0; 18],
        });
        self.dirty = true;
        Some(id)
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
            if ids.iter().any(|id| {
                campaign::cloak_quality(self.feats(class, *id)) == Some(Quality::Epic)
            }) {
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
        let rank = if leash { self.pets.leash } else { self.pets.paws };
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
            let gear = class.gear.iter().filter(|g| **g).count() as u32;
            snap.pieces += gear;
            if class.gear.iter().all(|g| *g) {
                snap.full_sets += 1;
            }
            snap.legends += class.seen.len() as u32;
            let talent_max = self.characters.iter().any(|c| {
                c.class == i && campaign::talent_level(c.talent_xp) >= TALENT_CAP
            });
            snap.talent_cap[i] = talent_max;
            let feats = Feats {
                talent_max,
                raid: class.clears.last().copied().unwrap_or(false),
                collection: class.gear.iter().all(|g| *g),
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
        dirty: false,
    }
}

/// Experience toward the next talent point, from how far the run got.
pub(super) fn talent_xp_for(level: u32, kills: u32, won: bool) -> u32 {
    level.saturating_mul(2) + kills / 10 + if won { 40 } else { 8 }
}

fn encode(account: &Account) -> String {
    let mut out = String::from("# survivors account: characters, then one class block each\n");
    out.push_str(&format!("next {}\n", account.next_id));
    for c in &account.characters {
        let talents = c
            .talents
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "who {} {} {} {} {} {talents}\n",
            c.id,
            c.class,
            c.talent_xp,
            escape(&c.first),
            escape(&c.last),
        ));
    }
    for (i, class) in account.classes.iter().enumerate() {
        let gear = class
            .gear
            .iter()
            .map(|g| if *g { "1" } else { "0" })
            .collect::<Vec<_>>()
            .join("");
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
        "meta {} {} {} {} {}\n",
        account.runs, account.wins, account.spent, account.perk_spent, account.resets
    ));
    let perks = account.perks;
    out.push_str(&format!(
        "perk {} {} {} {} {} {}\n",
        perks.second_thoughts, perks.skip, perks.veterans, perks.banish, perks.opening_hand, perks.keen_eye
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

fn parse(text: &str) -> Result<Account, ()> {
    let mut account = empty();
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
                let mut talents = [0u8; 18];
                if let Some(list) = parts.next() {
                    for (i, n) in list.split(',').enumerate().take(18) {
                        talents[i] = n.parse().unwrap_or(0);
                    }
                }
                if class < data::CLASSES.len() {
                    account.characters.push(Character {
                        id,
                        first,
                        last,
                        class,
                        talent_xp,
                        talents,
                    });
                }
            }
            Some("class") => {
                let index: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
                let gear = parts.next().unwrap_or("");
                let legendary = parts.next().unwrap_or("0") == "1";
                let clears = parts.next().unwrap_or("");
                if let Some(progress) = account.classes.get_mut(index) {
                    for (i, ch) in gear.chars().enumerate().take(10) {
                        progress.gear[i] = ch == '1';
                    }
                    progress.legendary_weapon = legendary;
                    for (i, ch) in clears.chars().enumerate().take(7) {
                        progress.clears[i] = ch == '1';
                    }
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
    run: Res<super::Run>,
) {
    let Some(class) = selection.class else {
        return;
    };
    let id = account.ensure(class);
    account.runs = account.runs.saturating_add(1);
    if run.won {
        account.wins = account.wins.saturating_add(1);
    }
    let mine = account.character(id).map(|c| c.talent_xp).unwrap_or(0);
    let best = account
        .characters
        .iter()
        .map(|c| c.talent_xp)
        .max()
        .unwrap_or(0);
    let xp = talent_xp_for(run.level, run.kills, run.won);
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
    account.save();
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(account.classes[0].gear[0]);
        assert!(account.classes[0].clears[0]);
        assert!(!account.classes[0].clears[1]);
        assert_eq!(account.classes[0].seen, vec!["Amulet of Heroic Strike".to_string()]);
        let again = parse(&encode(&account)).unwrap();
        assert_eq!(account.pets.owned, 2);
        assert_eq!(account.pets.leash, 3);
        assert_eq!(account.pets.instinct, [1, 2]);
        assert_eq!(again.characters[0].last, "Wrynn");
        assert_eq!(again.pets.owned, 2);
        assert!(again.classes[0].gear[1]);
    }

    #[test]
    fn a_cleared_zone_keeps_its_pieces_and_opens_the_next() {
        let mut account = empty();
        let id = account.ensure(0);
        account.grant_zone(0, 0);
        account.see_amulet(0, "Amulet of Heroic Strike");
        assert_eq!(account.classes[0].seen.len(), 1);
        assert!(account.classes[0].gear[0] && account.classes[0].gear[1]);
        assert!(!account.classes[0].gear[2]);
        assert!(campaign::zone_open(
            1,
            campaign::talent_level(account.character(id).unwrap().talent_xp),
            &account.classes[0].clears,
        ) == false);
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
        assert!(account.classes[0].legendary_weapon);
        let id = account.ensure(0);
        account.add_talent_xp(id, 600);
        assert!(account.learn(id, 0));
        assert!(!account.learn(id, 1));
        for _ in 0..4 {
            assert!(account.learn(id, 0));
        }
        assert!(account.learn(id, 1));
        assert!(account.bonus(0, id).might > 0.0);
        account.respec(id);
        assert_eq!(account.character(id).unwrap().talents[0], 0);
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
        let text = encode(&account);
        let loaded = parse(&text).unwrap();
        assert_eq!(loaded.pets.leash, 1);
        assert_eq!(loaded.resets_left(), 1);
        assert!(!loaded.earned_ids().is_empty());
    }
}
