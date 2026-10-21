//! The account save: named characters, and the permanent progress shared by a class.

use bevy::prelude::*;

use super::campaign::{self, Bonus, Feats, Quality, TALENT_CAP};
use super::data;
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
        parse(&text).unwrap_or_else(|_| Self {
            next_id: 1,
            characters: Vec::new(),
            classes: vec![ClassProgress::default(); data::CLASSES.len()],
            pets: PetRanks::default(),
            dirty: false,
        })
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

    pub fn bonus_for(&self, class: usize) -> Bonus {
        let id = self
            .characters
            .iter()
            .find(|c| c.class == class)
            .map(|c| c.id)
            .unwrap_or(0);
        self.bonus(class, id)
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
    out
}

fn escape(s: &str) -> String {
    s.replace(' ', "_")
}

fn unescape(s: &str) -> String {
    s.replace('_', " ")
}

fn parse(text: &str) -> Result<Account, ()> {
    let mut account = Account {
        next_id: 1,
        characters: Vec::new(),
        classes: vec![ClassProgress::default(); data::CLASSES.len()],
        pets: PetRanks::default(),
        dirty: false,
    };
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
    account.add_talent_xp(id, talent_xp_for(run.level, run.kills, run.won));
    account.remember_amulets(class, &run.seen_legends);
    if run.won {
        if let Some(zone) = campaign::zone_index(selection.map_def().name) {
            account.grant_zone(class, zone);
        }
    }
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
        let mut account = Account {
            next_id: 1,
            characters: Vec::new(),
            classes: vec![ClassProgress::default(); data::CLASSES.len()],
            pets: PetRanks::default(),
            dirty: false,
        };
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
        let mut account = Account {
            next_id: 1,
            characters: Vec::new(),
            classes: vec![ClassProgress::default(); data::CLASSES.len()],
            pets: PetRanks::default(),
            dirty: false,
        };
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
}
