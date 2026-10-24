//! Gold and relics that outlive a run. Gold drops from kills and chests. Relics come from
//! chests and from the menu, and three of them can be worn into the next run.

use std::path::PathBuf;

use bevy::prelude::*;

use super::data::Stat;
use super::Stats;

pub(super) const MAX_RELIC_RANK: u8 = 3;
pub(super) const SEEK_COST: u32 = 75;
/// `GameObjectDisplayInfo` 10: the wooden chest worn chests use in 1.12.
pub(super) const CHEST_DISPLAY: u32 = survivors_core::CHEST_DISPLAY;

struct Relic {
    id: &'static str,
    name: &'static str,
    stat: Stat,
    per_rank: f32,
}

const RELICS: &[Relic] = &[
    Relic {
        id: "edge",
        name: "Keen Edge",
        stat: Stat::Might,
        per_rank: 0.04,
    },
    Relic {
        id: "heart",
        name: "Heartstone",
        stat: Stat::MaxHp,
        per_rank: 0.06,
    },
    Relic {
        id: "boots",
        name: "Swift Boots",
        stat: Stat::Speed,
        per_rank: 0.03,
    },
    Relic {
        id: "glass",
        name: "Hourglass",
        stat: Stat::Haste,
        per_rank: 0.02,
    },
    Relic {
        id: "lode",
        name: "Lodestone",
        stat: Stat::Magnet,
        per_rank: 0.15,
    },
    Relic {
        id: "plate",
        name: "Warplate",
        stat: Stat::Armor,
        per_rank: 1.5,
    },
    Relic {
        id: "coin",
        name: "Lucky Coin",
        stat: Stat::Crit,
        per_rank: 0.02,
    },
    Relic {
        id: "tome",
        name: "Scholar's Tome",
        stat: Stat::Growth,
        per_rank: 0.05,
    },
    Relic {
        id: "band",
        name: "Ember Band",
        stat: Stat::Area,
        per_rank: 0.05,
    },
    Relic {
        id: "wind",
        name: "Second Wind",
        stat: Stat::Regen,
        per_rank: 0.6,
    },
];

/// What a chest pays. `roll` in `[0, 1)`: under 0.45, a relic that can still grow.
pub(super) fn roll_chest(roll: f32, level: u32, ranks: &[u8]) -> (u32, Option<usize>) {
    let gold = 18 + level / 3;
    if roll >= 0.45 {
        return (gold + 12, None);
    }
    let open: Vec<usize> = ranks
        .iter()
        .enumerate()
        .filter(|(_, r)| **r < MAX_RELIC_RANK)
        .map(|(i, _)| i)
        .collect();
    if open.is_empty() {
        return (gold + 12, None);
    }
    let i = ((roll / 0.45) * open.len() as f32) as usize;
    (gold, Some(open[i.min(open.len() - 1)]))
}

/// Gold to raise a relic from `rank` to the next.
pub(super) fn upgrade_cost(rank: u8) -> u32 {
    40 * u32::from(rank.max(1))
}

#[derive(Resource)]
pub(super) struct Stash {
    pub gold: u32,
    ranks: Vec<u8>,
    equipped: [Option<usize>; 3],
    /// The relic slot Upgrade spends on.
    pub focus: usize,
    dirty: bool,
}

impl Default for Stash {
    fn default() -> Self {
        Self::load()
    }
}

impl Stash {
    fn load() -> Self {
        let text = path().and_then(|p| crate::local_state::read_to_string(&p).ok());
        text.map(|t| parse(&t)).unwrap_or_else(Self::blank)
    }

    fn blank() -> Self {
        Self {
            gold: 0,
            ranks: vec![0; RELICS.len()],
            equipped: [None; 3],
            focus: 0,
            dirty: false,
        }
    }

    #[allow(dead_code)]
    pub fn ranks(&self) -> &[u8] {
        &self.ranks
    }

    pub fn add_gold(&mut self, n: u32) {
        if n == 0 {
            return;
        }
        self.gold = self.gold.saturating_add(n);
        self.dirty = true;
    }

    /// Rank the relic up, or pay gold out if it is already finished. Equips it into a free slot.
    #[allow(dead_code)]
    pub fn grant_relic(&mut self, index: usize) -> String {
        let Some(relic) = RELICS.get(index) else {
            return "Gold".into();
        };
        let rank = self.ranks.get(index).copied().unwrap_or(MAX_RELIC_RANK);
        if rank >= MAX_RELIC_RANK {
            self.add_gold(25);
            return format!("{} is finished — 25 gold", relic.name);
        }
        self.ranks[index] = rank + 1;
        self.equip_free(index);
        self.dirty = true;
        format!("{} {}", relic.name, self.ranks[index])
    }

    fn equip_free(&mut self, index: usize) {
        if self.equipped.contains(&Some(index)) {
            return;
        }
        if let Some(slot) = self.equipped.iter_mut().find(|s| s.is_none()) {
            *slot = Some(index);
        }
    }

    pub fn cycle_slot(&mut self, slot: usize) {
        if slot >= self.equipped.len() {
            return;
        }
        let choices = cycle_choices(&self.ranks, &self.equipped, slot);
        let cur = self.equipped[slot];
        let pos = choices.iter().position(|c| *c == cur).unwrap_or(0);
        self.equipped[slot] = choices[(pos + 1) % choices.len()];
        self.focus = slot;
        self.dirty = true;
    }

    pub fn can_upgrade(&self) -> bool {
        let Some(i) = self.equipped.get(self.focus).copied().flatten() else {
            return false;
        };
        let rank = self.ranks.get(i).copied().unwrap_or(0);
        rank > 0 && rank < MAX_RELIC_RANK && self.gold >= upgrade_cost(rank)
    }

    pub fn upgrade(&mut self) -> bool {
        if !self.can_upgrade() {
            return false;
        }
        let i = self.equipped[self.focus].unwrap();
        let rank = self.ranks[i];
        self.gold -= upgrade_cost(rank);
        self.ranks[i] = rank + 1;
        self.dirty = true;
        true
    }

    pub fn can_seek(&self) -> bool {
        self.gold >= SEEK_COST && self.ranks.iter().any(|r| *r < MAX_RELIC_RANK)
    }

    /// Spend [`SEEK_COST`] on a random relic that can still grow. `roll` is in `[0, 1)`.
    pub fn seek(&mut self, roll: f32) -> Option<&'static str> {
        if !self.can_seek() {
            return None;
        }
        let open: Vec<usize> = self
            .ranks
            .iter()
            .enumerate()
            .filter(|(_, r)| **r < MAX_RELIC_RANK)
            .map(|(i, _)| i)
            .collect();
        let i = open[((roll * open.len() as f32) as usize).min(open.len() - 1)];
        self.gold -= SEEK_COST;
        self.ranks[i] = self.ranks[i].saturating_add(1).min(MAX_RELIC_RANK);
        self.equip_free(i);
        self.dirty = true;
        Some(RELICS[i].name)
    }

    /// Catalog index and rank for each worn slot, copied onto the run.
    pub fn gear(&self) -> [Option<(u8, u8)>; 3] {
        let mut g = [None; 3];
        for (s, equipped) in self.equipped.iter().enumerate() {
            if let Some(i) = equipped {
                let rank = self.ranks.get(*i).copied().unwrap_or(0);
                if rank > 0 {
                    g[s] = Some((*i as u8, rank));
                }
            }
        }
        g
    }

    pub fn line(&self) -> String {
        let slots: Vec<String> = self
            .equipped
            .iter()
            .enumerate()
            .map(|(s, e)| {
                let name = match e {
                    Some(i) => format!("{} {}", RELICS[*i].name, self.ranks[*i]),
                    None => "empty".into(),
                };
                if s == self.focus {
                    format!("[{name}]")
                } else {
                    name
                }
            })
            .collect();
        format!("Gold {}    {}", self.gold, slots.join("   "))
    }

    pub fn save(&mut self) {
        self.dirty = false;
        let Some(path) = path() else {
            return;
        };
        let mut body = String::from("# gold, then `rank <id> <n>` and `equip <id|- > ...`\n");
        body.push_str(&format!("gold {}\n", self.gold));
        for (i, r) in self.ranks.iter().enumerate() {
            if *r > 0 {
                body.push_str(&format!("rank {} {r}\n", RELICS[i].id));
            }
        }
        body.push_str("equip");
        for e in &self.equipped {
            match e.and_then(|i| RELICS.get(i).map(|r| r.id)) {
                Some(id) => body.push_str(&format!(" {id}")),
                None => body.push_str(" -"),
            }
        }
        body.push('\n');
        body.push_str(&format!("focus {}\n", self.focus));
        if let Err(e) = crate::local_state::write_atomic(&path, &body) {
            warn!("survivors: stash save failed: {e}");
        }
    }
}

fn cycle_choices(ranks: &[u8], equipped: &[Option<usize>; 3], slot: usize) -> Vec<Option<usize>> {
    let mut v = vec![None];
    for (i, rank) in ranks.iter().enumerate() {
        if *rank == 0 {
            continue;
        }
        if equipped
            .iter()
            .enumerate()
            .any(|(s, e)| s != slot && *e == Some(i))
        {
            continue;
        }
        v.push(Some(i));
    }
    v
}

fn path() -> Option<PathBuf> {
    crate::local_state::home().map(|h| h.join("survivors-stash.txt"))
}

fn parse(text: &str) -> Stash {
    let mut stash = Stash::blank();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut w = line.split_whitespace();
        match w.next() {
            Some("gold") => stash.gold = w.next().and_then(|n| n.parse().ok()).unwrap_or(0),
            Some("rank") => {
                let Some(id) = w.next() else { continue };
                let rank = w
                    .next()
                    .and_then(|n| n.parse::<u32>().ok())
                    .unwrap_or(0)
                    .min(u32::from(MAX_RELIC_RANK)) as u8;
                if let Some(i) = RELICS.iter().position(|r| r.id == id) {
                    stash.ranks[i] = rank;
                }
            }
            Some("equip") => {
                for (i, tok) in w.enumerate().take(3) {
                    stash.equipped[i] = RELICS.iter().position(|r| r.id == tok);
                }
            }
            Some("focus") => {
                stash.focus = w.next().and_then(|n| n.parse().ok()).unwrap_or(0).min(2);
            }
            _ => {}
        }
    }
    stash
}

pub(super) fn apply_gear(stats: &mut Stats, gear: [Option<(u8, u8)>; 3]) {
    for (index, rank) in gear.into_iter().flatten() {
        let Some(relic) = RELICS.get(index as usize) else {
            continue;
        };
        if rank == 0 {
            continue;
        }
        let v = relic.per_rank * f32::from(rank);
        match relic.stat {
            Stat::Might => stats.might += v,
            Stat::MaxHp => stats.max_hp *= 1.0 + v,
            Stat::Speed => stats.speed *= 1.0 + v,
            Stat::Regen => stats.regen += v,
            Stat::Haste => stats.haste += v,
            Stat::Area => stats.area += v,
            Stat::Magnet => stats.magnet *= 1.0 + v,
            Stat::Growth => stats.growth += v,
            Stat::Armor => stats.armor += v,
            Stat::Crit => stats.crit += v,
        }
    }
}

pub(super) fn blurb(gear: [Option<(u8, u8)>; 3]) -> String {
    let names: Vec<String> = gear
        .into_iter()
        .flatten()
        .filter_map(|(i, rank)| RELICS.get(i as usize).map(|r| format!("{} {rank}", r.name)))
        .collect();
    if names.is_empty() {
        String::new()
    } else {
        format!("Relics — {}", names.join(", "))
    }
}

/// Write a dirty purse. Called on a short timer and when a run ends.
pub(super) fn flush(mut stash: ResMut<Stash>, time: Res<Time<Real>>, mut acc: Local<f32>) {
    if !stash.dirty {
        *acc = 0.0;
        return;
    }
    *acc += time.delta_secs();
    if *acc < 2.0 {
        return;
    }
    stash.save();
}

pub(super) fn flush_now(mut stash: ResMut<Stash>) {
    if stash.dirty {
        stash.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chest_prefers_a_relic_that_can_still_grow() {
        let (gold, relic) = roll_chest(0.0, 30, &[0; 10]);
        assert_eq!(gold, 18 + 10);
        assert_eq!(relic, Some(0));
        let (gold, relic) = roll_chest(0.9, 3, &[MAX_RELIC_RANK; 10]);
        assert!(gold > 18);
        assert_eq!(relic, None);
    }

    #[test]
    fn a_comment_does_not_hide_the_gold() {
        let stash =
            parse("# gold\ngold 40\nrank edge 2\nrank boots 1\nequip edge - boots\nfocus 2\n");
        assert_eq!(stash.gold, 40);
        assert_eq!(stash.ranks[0], 2);
        assert_eq!(stash.ranks[2], 1);
        assert_eq!(stash.equipped, [Some(0), None, Some(2)]);
        assert_eq!(stash.focus, 2);
    }

    #[test]
    fn upgrading_spends_gold_and_stops_at_three() {
        let mut stash = Stash::blank();
        stash.gold = 200;
        stash.ranks[0] = 1;
        stash.equipped[0] = Some(0);
        assert!(stash.upgrade());
        assert_eq!(stash.ranks[0], 2);
        assert_eq!(stash.gold, 160);
        assert!(stash.upgrade());
        assert_eq!(stash.ranks[0], 3);
        assert!(!stash.upgrade());
    }

    #[test]
    fn seeking_equips_a_new_relic_into_an_empty_slot() {
        let mut stash = Stash::blank();
        stash.gold = SEEK_COST;
        let name = stash.seek(0.0).unwrap();
        assert_eq!(name, "Keen Edge");
        assert_eq!(stash.gold, 0);
        assert_eq!(stash.equipped[0], Some(0));
        assert_eq!(stash.ranks[0], 1);
        assert!(stash.seek(0.0).is_none());
    }
}
