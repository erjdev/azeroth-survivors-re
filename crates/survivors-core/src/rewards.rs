//! Achievement points and the perk tree. Points are earned once for the whole account.
//! Perks and companion ranks spend them. Two perk resets refund perk points and leave companions.

use super::campaign::{self, ZONES};
use super::data;

/// Ranks bought on the rewards screen. The number is the rank, and the effect is the rank.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PerkRanks {
    /// Free redraws of a level-up hand. Rank 1 is one, rank 3 is three.
    pub second_thoughts: u8,
    /// Pass a hand and take gold instead.
    pub skip: u8,
    /// A class behind your best talent experience gains 20% more per rank.
    pub veterans: u8,
    /// Times a run can throw out one card and draw another.
    pub banish: u8,
    /// The first level-up of a run deals four cards.
    pub opening_hand: u8,
    /// Boss chests are 5% more likely to offer jewelry per rank.
    pub keen_eye: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Perk {
    SecondThoughts,
    Skip,
    Veterans,
    Banish,
    OpeningHand,
    KeenEye,
}

impl PerkRanks {
    pub fn rank(self, perk: Perk) -> u8 {
        match perk {
            Perk::SecondThoughts => self.second_thoughts,
            Perk::Skip => self.skip,
            Perk::Veterans => self.veterans,
            Perk::Banish => self.banish,
            Perk::OpeningHand => self.opening_hand,
            Perk::KeenEye => self.keen_eye,
        }
    }

    pub fn raise(&mut self, perk: Perk) {
        let slot = match perk {
            Perk::SecondThoughts => &mut self.second_thoughts,
            Perk::Skip => &mut self.skip,
            Perk::Veterans => &mut self.veterans,
            Perk::Banish => &mut self.banish,
            Perk::OpeningHand => &mut self.opening_hand,
            Perk::KeenEye => &mut self.keen_eye,
        };
        *slot = slot.saturating_add(1);
    }
}

const THOUGHTS: [u32; 3] = [15, 25, 35];
const SKIP: [u32; 1] = [15];
const VETERANS: [u32; 2] = [20, 40];
const BANISH: [u32; 3] = [15, 25, 40];
const OPENING: [u32; 1] = [25];
const KEEN: [u32; 2] = [20, 35];

pub fn perk_cost(perk: Perk, rank: u8) -> Option<u32> {
    let table: &[u32] = match perk {
        Perk::SecondThoughts => &THOUGHTS,
        Perk::Skip => &SKIP,
        Perk::Veterans => &VETERANS,
        Perk::Banish => &BANISH,
        Perk::OpeningHand => &OPENING,
        Perk::KeenEye => &KEEN,
    };
    table.get(usize::from(rank)).copied()
}

pub fn perk_name(perk: Perk) -> &'static str {
    match perk {
        Perk::SecondThoughts => "Second Thoughts",
        Perk::Skip => "Skip",
        Perk::Veterans => "Veteran's Lessons",
        Perk::Banish => "Banish",
        Perk::OpeningHand => "Opening Hand",
        Perk::KeenEye => "Keen Eye",
    }
}

pub const PERKS: [Perk; 6] = [
    Perk::SecondThoughts,
    Perk::Skip,
    Perk::Veterans,
    Perk::Banish,
    Perk::OpeningHand,
    Perk::KeenEye,
];

/// Maximum purchasable rank, shared by reward pips and previews.
pub fn perk_max_rank(perk: Perk) -> u8 {
    (0..=u8::MAX)
        .find(|&rank| perk_cost(perk, rank).is_none())
        .unwrap_or(0)
}

/// One sentence with its changing value isolated for inline highlighting.
pub fn perk_preview(perk: Perk, rank: u8) -> (&'static str, String, &'static str) {
    let rank = rank.min(perk_max_rank(perk));
    let next = (rank + 1).min(perk_max_rank(perk));
    let compare = |current: String, future: String| {
        if rank == next {
            current
        } else {
            format!("{current} » {future}")
        }
    };
    match perk {
        Perk::SecondThoughts => (
            "Redraw a level-up hand ",
            format!("{} times", compare(rank.to_string(), next.to_string())),
            " per run.",
        ),
        Perk::Banish => (
            "Throw out one card and draw another ",
            format!("{} times", compare(rank.to_string(), next.to_string())),
            " per run.",
        ),
        Perk::Skip => (
            "Passing a hand for gold is ",
            compare(
                if rank == 0 {
                    "unavailable"
                } else {
                    "available"
                }
                .into(),
                "available".into(),
            ),
            ". The payout scales with your run level.",
        ),
        Perk::Veterans => {
            let bonus = |r| format!("{:.0}", (veteran_multiplier(r, 0, 1) - 1.0) * 100.0);
            (
                "A class behind the account's best earns ",
                format!(
                    "{}% more talent experience",
                    compare(bonus(rank), bonus(next))
                ),
                ".",
            )
        }
        Perk::OpeningHand => (
            "The first level-up of each run deals ",
            format!(
                "{} cards",
                compare(
                    hand_size(rank > 0, 0).to_string(),
                    hand_size(next > 0, 0).to_string()
                )
            ),
            ".",
        ),
        Perk::KeenEye => {
            let bonus = |r| format!("{:.0}", keen_drop_bonus(r) * 100.0);
            (
                "Add ",
                format!("{} percentage points", compare(bonus(rank), bonus(next))),
                " to each boss gear drop chance.",
            )
        }
    }
}

/// Gold offered for passing a hand, before gold find.
pub fn skip_gold(level: u32) -> u32 {
    8 + level
}

/// Talent experience multiplier. The account's best character is the measuring stick.
pub fn veteran_multiplier(rank: u8, mine: u32, best: u32) -> f32 {
    if rank == 0 || mine >= best {
        1.0
    } else {
        1.0 + 0.20 * f32::from(rank.min(2))
    }
}

/// Added to a boss's chance to drop each piece. Rank 2 is 10%.
pub fn keen_drop_bonus(rank: u8) -> f32 {
    0.05 * f32::from(rank.min(2))
}

pub fn perk_effect(perk: Perk) -> &'static str {
    match perk {
        Perk::SecondThoughts => {
            "Redraw a level-up hand. The first rank is one redraw, the third is three."
        }
        Perk::Skip => "Pass a hand and take gold instead.",
        Perk::Veterans => {
            "A class behind the account's best earns 20% more talent experience a rank."
        }
        Perk::Banish => "Throw out one card and draw another.",
        Perk::OpeningHand => "The first level-up of a run deals four cards.",
        Perk::KeenEye => "A boss is 5% more likely to drop each piece, a rank. Two ranks is 10%.",
    }
}

/// Cards in a level-up hand. The first hand of a run is four when Opening Hand is owned.
pub fn hand_size(opening: bool, drafts: u32) -> usize {
    if opening && drafts == 0 {
        4
    } else {
        3
    }
}

/// What the account has done, gathered by the save before it checks achievements.
pub struct Snapshot {
    pub clears: [[bool; campaign::CLEAR_COUNT]; 9],
    pub pieces: u32,
    pub full_sets: u32,
    pub legends: u32,
    pub talent_cap: [bool; 9],
    pub any_cloak: bool,
    pub epic_cloaks: u32,
    pub legendary_cloak: bool,
    pub runs: u32,
    pub wins: u32,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            clears: [[false; campaign::CLEAR_COUNT]; 9],
            pieces: 0,
            full_sets: 0,
            legends: 0,
            talent_cap: [false; 9],
            any_cloak: false,
            epic_cloaks: 0,
            legendary_cloak: false,
            runs: 0,
            wins: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    Zones,
    Classes,
    Collection,
    Feats,
    Lifetime,
}

pub const DEATHLESS_HOUR: u16 = 54;
pub const NO_U: u16 = 55;

enum Kind {
    Placeholder,
    ZoneAny(usize),
    ZoneAll(usize),
    ClassClear(usize),
    TalentCap(usize),
    Pieces(u32),
    FullSet,
    Legends(u32),
    AnyCloak,
    EpicCloak,
    LegendaryCloak,
    Wins(u32),
    Runs(u32),
}

struct Achievement {
    id: u16,
    name: String,
    points: u32,
    kind: Kind,
}

impl Kind {
    fn met(&self, snap: &Snapshot) -> bool {
        match *self {
            Kind::Placeholder => false,
            Kind::ZoneAny(z) => snap
                .clears
                .iter()
                .any(|c| c.get(z).copied().unwrap_or(false)),
            Kind::ZoneAll(z) => snap
                .clears
                .iter()
                .all(|c| c.get(z).copied().unwrap_or(false)),
            Kind::ClassClear(c) => snap
                .clears
                .get(c)
                .is_some_and(|row| row.iter().any(|on| *on)),
            Kind::TalentCap(c) => snap.talent_cap.get(c).copied().unwrap_or(false),
            Kind::Pieces(n) => snap.pieces >= n,
            Kind::FullSet => snap.full_sets > 0,
            Kind::Legends(n) => snap.legends >= n,
            Kind::AnyCloak => snap.any_cloak,
            Kind::EpicCloak => snap.epic_cloaks > 0,
            Kind::LegendaryCloak => snap.legendary_cloak,
            Kind::Wins(n) => snap.wins >= n,
            Kind::Runs(n) => snap.runs >= n,
        }
    }

    fn description(&self) -> String {
        match *self {
            Kind::Placeholder => "Criteria will be defined in a future update.".into(),
            Kind::ZoneAny(z) => format!("Clear {} with any class.", ZONES[z].name),
            Kind::ZoneAll(z) => format!("Clear {} with all nine classes.", ZONES[z].name),
            Kind::ClassClear(c) => format!("Clear a zone as {}.", data::CLASSES[c].name),
            Kind::TalentCap(c) => format!(
                "Reach talent level {} as {}.",
                campaign::TALENT_CAP,
                data::CLASSES[c].name
            ),
            Kind::Pieces(n) => {
                format!("Collect {n} distinct class gear pieces across the account.")
            }
            Kind::FullSet => "Collect gear for all nine slots of one class.".into(),
            Kind::Legends(n) => format!("Discover {n} legendary amulets in the Codex."),
            Kind::AnyCloak => "Collect a cloak of any quality.".into(),
            Kind::EpicCloak => "Collect an epic cloak.".into(),
            Kind::LegendaryCloak => "Collect a legendary cloak.".into(),
            Kind::Wins(1) => "Win a run.".into(),
            Kind::Wins(n) => format!("Win {n} runs."),
            Kind::Runs(n) => format!("Play {n} runs."),
        }
    }

    /// Authored icon choices from the vanilla icon set, shared by related achievement tiers.
    fn icon(&self, id: u16) -> &'static str {
        match *self {
            Kind::ZoneAny(z) | Kind::ZoneAll(z) => [
                "INV_Misc_Bandana_01",
                "INV_Misc_Bone_HumanSkull_01",
                "INV_BannerPVP_02",
                "Ability_Mount_Raptor",
                "INV_BannerPVP_01",
                "Spell_Nature_ProtectionformNature",
                "Ability_Warrior_BattleShout",
                "Spell_Holy_SealOfSalvation",
                "Spell_Frost_ChillingBlast",
                "Spell_Fire_Volcano",
                "INV_Misc_AhnQirajTrinket_01",
                "Spell_Shadow_DeathAndDecay",
            ][z],
            Kind::ClassClear(c) | Kind::TalentCap(c) => [
                "Ability_Warrior_SavageBlow",
                "Spell_Holy_HolyBolt",
                "Ability_Hunter_AimedShot",
                "Ability_BackStab",
                "Spell_Holy_PowerWordShield",
                "Spell_Nature_Lightning",
                "Spell_Frost_FrostBolt02",
                "Spell_Shadow_ShadowBolt",
                "Spell_Nature_Rejuvenation",
            ][c],
            Kind::Pieces(10) => "INV_Sword_04",
            Kind::Pieces(_) => "INV_Misc_Bag_10",
            Kind::FullSet => "INV_Chest_Plate04",
            Kind::Legends(8) => "INV_Jewelry_Ring_03",
            Kind::Legends(_) => "INV_Jewelry_Necklace_01",
            Kind::AnyCloak => "INV_Misc_Cape_01",
            Kind::EpicCloak => "INV_Misc_Cape_11",
            Kind::LegendaryCloak => "INV_Misc_Cape_18",
            Kind::Wins(1) => "Ability_BackStab",
            Kind::Wins(10) => "INV_BannerPVP_01",
            Kind::Wins(_) => "INV_BannerPVP_02",
            Kind::Runs(10) => "INV_Shield_05",
            Kind::Runs(_) => "INV_Shield_06",
            Kind::Placeholder if id == DEATHLESS_HOUR => "Spell_Holy_BlessingOfProtection",
            Kind::Placeholder => "Spell_Frost_FrostArmor02",
        }
    }

    fn category(&self) -> Category {
        match self {
            Kind::ZoneAny(_) | Kind::ZoneAll(_) => Category::Zones,
            Kind::ClassClear(_) | Kind::TalentCap(_) => Category::Classes,
            Kind::Pieces(_) | Kind::FullSet | Kind::Legends(_) => Category::Collection,
            Kind::AnyCloak | Kind::EpicCloak | Kind::LegendaryCloak | Kind::Placeholder => {
                Category::Feats
            }
            Kind::Wins(_) | Kind::Runs(_) => Category::Lifetime,
        }
    }

    fn progress(&self, snap: &Snapshot) -> Option<(u32, u32)> {
        match *self {
            Kind::ZoneAll(z) => {
                let have = snap
                    .clears
                    .iter()
                    .filter(|c| c.get(z).copied().unwrap_or(false))
                    .count() as u32;
                Some((have, 9))
            }
            Kind::Pieces(n) => Some((snap.pieces.min(n), n)),
            Kind::Legends(n) => Some((snap.legends.min(n), n)),
            Kind::Wins(n) => Some((snap.wins.min(n), n)),
            Kind::Runs(n) => Some((snap.runs.min(n), n)),
            _ => None,
        }
    }
}

fn catalog() -> Vec<Achievement> {
    let mut out = Vec::new();
    for (i, zone) in ZONES.iter().enumerate() {
        out.push(Achievement {
            id: zone.into_id,
            name: format!("Into {}", zone.name),
            points: zone.points,
            kind: Kind::ZoneAny(i),
        });
        out.push(Achievement {
            id: zone.champ_id,
            name: format!("Champion of {}", zone.name),
            points: 25,
            kind: Kind::ZoneAll(i),
        });
    }
    for (i, class) in data::CLASSES.iter().enumerate().take(9) {
        out.push(Achievement {
            id: 21 + i as u16,
            name: format!("Win as {}", class.name),
            points: 10,
            kind: Kind::ClassClear(i),
        });
        out.push(Achievement {
            id: 31 + i as u16,
            name: format!("{} mastered", class.name),
            points: 25,
            kind: Kind::TalentCap(i),
        });
    }
    out.push(Achievement {
        id: 41,
        name: "Armed".into(),
        points: 10,
        kind: Kind::Pieces(10),
    });
    out.push(Achievement {
        id: 42,
        name: "Hoarder".into(),
        points: 15,
        kind: Kind::Pieces(40),
    });
    out.push(Achievement {
        id: 43,
        name: "Full set".into(),
        points: 15,
        kind: Kind::FullSet,
    });
    out.push(Achievement {
        id: 44,
        name: "Jeweler's Eye".into(),
        points: 10,
        kind: Kind::Legends(8),
    });
    out.push(Achievement {
        id: 45,
        name: "Half the jewels".into(),
        points: 15,
        kind: Kind::Legends(36),
    });
    out.push(Achievement {
        id: 51,
        name: "Cloak bearer".into(),
        points: 10,
        kind: Kind::AnyCloak,
    });
    out.push(Achievement {
        id: 52,
        name: "Epic cloak".into(),
        points: 25,
        kind: Kind::EpicCloak,
    });
    out.push(Achievement {
        id: 53,
        name: "Legendary cloak".into(),
        points: 40,
        kind: Kind::LegendaryCloak,
    });
    out.push(Achievement {
        id: 61,
        name: "First Blood".into(),
        points: 10,
        kind: Kind::Wins(1),
    });
    out.push(Achievement {
        id: 62,
        name: "Campaigner".into(),
        points: 15,
        kind: Kind::Wins(10),
    });
    out.push(Achievement {
        id: 63,
        name: "Conqueror".into(),
        points: 25,
        kind: Kind::Wins(50),
    });
    out.push(Achievement {
        id: 64,
        name: "Recruit".into(),
        points: 10,
        kind: Kind::Runs(10),
    });
    out.push(Achievement {
        id: 65,
        name: "Veteran".into(),
        points: 15,
        kind: Kind::Runs(100),
    });
    for (id, name) in [(DEATHLESS_HOUR, "Deathless Hour"), (NO_U, "no u")] {
        out.push(Achievement {
            id,
            name: name.into(),
            points: 0,
            kind: Kind::Placeholder,
        });
    }
    out
}

/// Placeholder achievements cannot grant rewards, including from preexisting save IDs.
pub fn available(id: u16) -> bool {
    catalog()
        .iter()
        .any(|a| a.id == id && !matches!(a.kind, Kind::Placeholder))
}

/// Achievements newly met. Already-paid ids are skipped, so a second pass pays nothing.
pub fn fresh(snapshot: &Snapshot, earned: &[u16]) -> Vec<(u16, u32)> {
    catalog()
        .into_iter()
        .filter(|a| !earned.contains(&a.id) && a.kind.met(snapshot))
        .map(|a| (a.id, a.points))
        .collect()
}

pub fn points_of(earned: &[u16]) -> u32 {
    let cat = catalog();
    earned
        .iter()
        .filter_map(|id| cat.iter().find(|a| a.id == *id).map(|a| a.points))
        .sum()
}

pub fn earned_count(earned: &[u16]) -> usize {
    let cat = catalog();
    earned
        .iter()
        .filter(|id| {
            cat.iter()
                .any(|a| a.id == **id && !matches!(a.kind, Kind::Placeholder))
        })
        .count()
}

pub fn catalog_len() -> usize {
    catalog().len()
}

pub struct Listed {
    pub icon: &'static str,
    pub id: u16,
    pub name: String,
    pub description: String,
    pub points: u32,
    pub earned: bool,
    pub available: bool,
    pub category: Category,
    pub progress: Option<(u32, u32)>,
}

/// Every achievement, earned or still open.
pub fn listed(snapshot: &Snapshot, earned: &[u16]) -> Vec<Listed> {
    catalog()
        .into_iter()
        .map(|a| Listed {
            icon: a.kind.icon(a.id),
            id: a.id,
            earned: !matches!(a.kind, Kind::Placeholder)
                && (earned.contains(&a.id) || a.kind.met(snapshot)),
            available: !matches!(a.kind, Kind::Placeholder),
            points: a.points,
            description: a.kind.description(),
            name: a.name,
            category: a.kind.category(),
            progress: a.kind.progress(snapshot),
        })
        .collect()
}

/// The first achievement still open, for the rewards page.
pub fn next_locked(snapshot: &Snapshot, earned: &[u16]) -> Option<String> {
    catalog()
        .into_iter()
        .find(|a| {
            !matches!(a.kind, Kind::Placeholder) && !earned.contains(&a.id) && !a.kind.met(snapshot)
        })
        .map(|a| format!("{} — {} points", a.name, a.points))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_achievements_never_complete_or_award_points() {
        let snap = Snapshot {
            runs: u32::MAX,
            wins: u32::MAX,
            legendary_cloak: true,
            ..Snapshot::default()
        };
        let ids = [DEATHLESS_HOUR, NO_U];
        for id in ids {
            assert!(!available(id));
            assert!(!fresh(&snap, &[]).iter().any(|(found, _)| *found == id));
            let row = listed(&snap, &ids)
                .into_iter()
                .find(|row| row.id == id)
                .unwrap();
            assert!(!row.earned);
            assert!(!row.available);
            assert_eq!(row.progress, None);
        }
        assert_eq!(points_of(&ids), 0);
        assert_eq!(earned_count(&ids), 0);
        let catalog = catalog();
        let mut unique = std::collections::HashSet::new();
        assert!(catalog.iter().all(|entry| unique.insert(entry.id)));
    }

    #[test]
    fn previews_show_next_effect_once_and_stop_at_the_cap() {
        assert_eq!(perk_preview(Perk::OpeningHand, 0).1, "3 » 4 cards");
        assert_eq!(perk_preview(Perk::SecondThoughts, 1).1, "1 » 2 times");
        assert_eq!(
            perk_preview(Perk::Veterans, 1).1,
            "20 » 40% more talent experience"
        );
        assert_eq!(perk_preview(Perk::KeenEye, 1).1, "5 » 10 percentage points");
        assert_eq!(perk_preview(Perk::Skip, 0).1, "unavailable » available");
        for perk in PERKS {
            let max = perk_max_rank(perk);
            assert!(!perk_preview(perk, max).1.contains('»'));
            assert_eq!(perk_preview(perk, u8::MAX), perk_preview(perk, max));
            assert!(perk_preview(perk, max - 1).1.contains('»'));
        }
    }

    #[test]
    fn a_zone_pays_once_and_a_champion_needs_every_class() {
        let mut snap = Snapshot::default();
        snap.clears[0][0] = true;
        let first = fresh(&snap, &[]);
        assert!(first.iter().any(|(id, pts)| *id == 1 && *pts == 10));
        assert!(first.iter().any(|(id, _)| *id == 21));
        assert!(!first.iter().any(|(id, _)| *id == 11));
        let again = fresh(&snap, &[1, 21]);
        assert!(again.is_empty());
        for row in &mut snap.clears {
            row[0] = true;
        }
        assert!(fresh(&snap, &[1, 21])
            .iter()
            .any(|(id, pts)| *id == 11 && *pts == 25));
    }

    #[test]
    fn perks_price_the_next_rank_and_change_the_run() {
        assert_eq!(perk_cost(Perk::SecondThoughts, 0), Some(15));
        assert_eq!(perk_cost(Perk::SecondThoughts, 2), Some(35));
        assert_eq!(perk_cost(Perk::SecondThoughts, 3), None);
        assert_eq!(perk_cost(Perk::Skip, 0), Some(15));
        assert_eq!(veteran_multiplier(2, 100, 500), 1.4);
        assert_eq!(veteran_multiplier(2, 500, 500), 1.0);
        assert_eq!(veteran_multiplier(0, 0, 500), 1.0);
        assert!((keen_drop_bonus(2) - 0.10).abs() < 0.001);
        assert_eq!(hand_size(true, 0), 4);
        assert_eq!(hand_size(true, 1), 3);
        assert_eq!(skip_gold(3), 11);
        assert!(catalog_len() > 40);
    }

    #[test]
    fn a_champion_row_counts_classes_and_keeps_its_category() {
        let mut snap = Snapshot::default();
        snap.clears[0][0] = true;
        snap.clears[2][0] = true;
        let rows = listed(&snap, &[]);
        let champ = rows
            .iter()
            .find(|r| r.name == "Champion of Elwynn Forest")
            .expect("elwynn champion");
        assert_eq!(champ.category, Category::Zones);
        assert_eq!(champ.progress, Some((2, 9)));
        assert!(!champ.earned);
        assert!(rows.iter().any(|r| r.category == Category::Lifetime));
    }
}
