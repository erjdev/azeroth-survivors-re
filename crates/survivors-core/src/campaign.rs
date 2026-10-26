//! Campaign rules that do not depend on a running world: zone order, gear quality, cloaks,
//! jewelry limits, and the talent skeleton. The menus read these; they do not restate them.

/// Talent experience for one point.
pub const XP_PER_TALENT: u32 = 100;
/// Initial campaign tuning: one XP per ten survived seconds, ten per defeated boss,
/// and ten percent more per later zone. The original level/kill/win award is retained.
pub fn talent_award(
    level: u32,
    kills: u32,
    elapsed: f32,
    bosses: u32,
    won: bool,
    zone: Option<usize>,
) -> u32 {
    let survival = if elapsed.is_finite() {
        (elapsed.max(0.0) / 10.0) as u32
    } else {
        0
    };
    let base = level
        .saturating_mul(2)
        .saturating_add(kills / 10)
        .saturating_add(if won { 40 } else { 8 })
        .saturating_add(survival)
        .saturating_add(bosses.saturating_mul(10));
    let scale = 100 + zone.unwrap_or(0).min(CLEAR_COUNT - 1) as u64 * 10;
    (u64::from(base) * scale / 100).min(u64::from(u32::MAX)) as u32
}

/// Points a character can hold, matching a vanilla tree.
pub const TALENT_CAP: u32 = 51;
/// Fourteen nodes in a tree. Ranks are the vanilla mix of 5, 3, and 1, and a full tree holds
/// more than 30 so the last row can open. A character still cannot spend more than
/// [`TALENT_CAP`] points across all three trees.
pub const NODES_PER_TREE: usize = 14;
pub const SPEC_COUNT: usize = 3;
pub const SPEC_LEN: usize = NODES_PER_TREE * SPEC_COUNT;
/// Row of each node inside one tree. Row `n` opens after `n * 5` points already spent there.
pub const NODE_ROW: [u8; NODES_PER_TREE] = [0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6];
pub const NODE_RANKS: [u8; NODES_PER_TREE] = [5, 5, 5, 3, 3, 1, 5, 5, 1, 3, 1, 1, 5, 1];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quality {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

pub fn quality_name(quality: Quality) -> &'static str {
    match quality {
        Quality::Poor => "Poor",
        Quality::Common => "Common",
        Quality::Uncommon => "Uncommon",
        Quality::Rare => "Rare",
        Quality::Epic => "Epic",
        Quality::Legendary => "Legendary",
    }
}

/// One zone on the path. `map` matches [`super::data::MapDef::name`] when the battleground exists.
pub struct Zone {
    pub name: &'static str,
    pub map: &'static str,
    /// Talent level required, in addition to clearing the preceding campaign zone.
    pub talent: u32,
    pub elites: &'static [&'static str],
    pub boss: &'static str,
    /// A carried row can be entered. A new row stays shut until it has a drop table.
    pub ready: bool,
    /// Stable achievement ids. Westfall, Arathi Highlands, and the Plaguelands kept theirs when
    /// they left the path, so a new row does not inherit an old earn.
    pub into_id: u16,
    pub champ_id: u16,
    pub points: u32,
}

/// How many clear flags a class stores, one per row of [`ZONES`].
pub const CLEAR_COUNT: usize = 12;

pub static ZONES: &[Zone] = &[
    Zone {
        name: "Elwynn Forest",
        map: "Elwynn Forest",
        talent: 0,
        elites: &["Defias Bandit", "Kobold Miner"],
        boss: "Hogger",
        ready: true,
        into_id: 1,
        champ_id: 11,
        points: 10,
    },
    Zone {
        name: "Duskwood",
        map: "Duskwood",
        talent: 5,
        elites: &["Skeletal Warrior", "Nightbane Worgen"],
        boss: "Nefaru",
        ready: true,
        into_id: 3,
        champ_id: 13,
        points: 10,
    },
    Zone {
        name: "Warsong Gulch",
        map: "Warsong Gulch",
        talent: 8,
        elites: &["Warsong Shredder Foreman", "Silverwing Field Marshal"],
        boss: "Warsong Gulch Warmaster",
        ready: false,
        into_id: 70,
        champ_id: 90,
        points: 15,
    },
    Zone {
        name: "Un'Goro Crater",
        map: "Fire Plume Ridge",
        talent: 12,
        elites: &["Devilsaur", "Stegodon"],
        boss: "King Mosh",
        ready: false,
        into_id: 71,
        champ_id: 91,
        points: 15,
    },
    Zone {
        name: "Arathi Basin",
        map: "Arathi Basin",
        talent: 16,
        elites: &["Defiler Blacksmith Champion", "Arathor Field Marshal"],
        boss: "Defiler Basin Warmaster",
        ready: false,
        into_id: 72,
        champ_id: 92,
        points: 15,
    },
    Zone {
        name: "Feralas",
        map: "Feralas",
        talent: 20,
        elites: &["Gordunni Mauler", "Wandering Forest Walker"],
        boss: "Skarr the Unbreakable",
        ready: false,
        into_id: 73,
        champ_id: 93,
        points: 15,
    },
    Zone {
        name: "Gurubashi Arena",
        map: "Gurubashi Arena",
        talent: 24,
        elites: &["Skullsplitter Berserker", "Bloodsail Raider"],
        boss: "King Mukla",
        ready: false,
        into_id: 4,
        champ_id: 14,
        points: 20,
    },
    Zone {
        name: "Scarlet Monastery",
        map: "Scarlet Monastery",
        talent: 28,
        elites: &["Scarlet Champion", "Scarlet Abbot"],
        boss: "High Inquisitor Whitemane",
        ready: false,
        into_id: 74,
        champ_id: 94,
        points: 20,
    },
    Zone {
        name: "Alterac Valley",
        map: "Alterac Valley",
        talent: 32,
        elites: &["Captain Galvangar", "Captain Balinda Stonehearth"],
        boss: "Drek'Thar",
        ready: false,
        into_id: 75,
        champ_id: 95,
        points: 20,
    },
    Zone {
        name: "Blackrock Mountain",
        map: "Blackrock Mountain",
        talent: 36,
        elites: &[],
        boss: "",
        ready: false,
        into_id: 76,
        champ_id: 96,
        points: 20,
    },
    Zone {
        name: "Gates of Ahn'Qiraj",
        map: "Gates of Ahn'Qiraj",
        talent: 42,
        elites: &["Hive'Regal Hive Lord", "Twilight Overlord"],
        boss: "Setis",
        ready: false,
        into_id: 77,
        champ_id: 97,
        points: 30,
    },
    Zone {
        name: "Naxxramas",
        map: "Naxxramas",
        talent: 48,
        elites: &["Necro Knight", "Plague Slime"],
        boss: "Kel'Thuzad",
        ready: false,
        into_id: 7,
        champ_id: 17,
        points: 30,
    },
];

/// Legacy slot labels, including off hand. The collection uses only the first `GRID_SLOTS`.
/// A two-handed weapon occupies main hand only.
pub const SLOTS: [&str; 10] = [
    "Helm",
    "Shoulders",
    "Chest",
    "Waist",
    "Legs",
    "Feet",
    "Wrist",
    "Hands",
    "Main Hand",
    "Off Hand",
];

/// True when the weapon in the main-hand slot is two-handed. It fills that slot only.
pub fn two_handed(class: usize) -> bool {
    matches!(class, 2 | 4 | 6 | 7 | 8)
}

pub fn zone_index(map_name: &str) -> Option<usize> {
    ZONES.iter().position(|z| z.map == map_name)
}

/// Instanced campaign destinations identified by Map.dbc in the 1.12.1 data.
/// A matching label on a different world is not the intended destination.
pub fn required_map_id(zone_name: &str) -> Option<u32> {
    match zone_name {
        "Alterac Valley" => Some(30),
        "Scarlet Monastery" => Some(189),
        "Warsong Gulch" => Some(489),
        "Arathi Basin" => Some(529),
        "Naxxramas" => Some(533),
        _ => None,
    }
}

/// Why a row cannot be entered yet, apart from the talent gate and the previous clear.
fn content_lock(zone: &Zone) -> Option<String> {
    let destination = super::data::MAPS.iter().find(|m| m.name == zone.map);
    if destination.is_some_and(|map| required_map_id(zone.name).is_some_and(|id| map.map != id)) {
        return Some(format!("{} world location is not ready", zone.name));
    }
    let map = destination.is_some();
    let boss = !zone.boss.is_empty();
    let scheduled = super::data::MAPS
        .iter()
        .position(|m| m.name == zone.map)
        .is_some_and(super::encounters::ready);
    if zone.ready && map && boss && scheduled {
        return None;
    }
    Some(match (map, boss) {
        (false, false) => format!("{} has no map and no boss yet", zone.name),
        (false, _) => format!("{} has no map yet", zone.name),
        (_, false) => format!("{} has no boss yet", zone.name),
        _ if scheduled && !zone.ready => format!("{} world content is not ready", zone.name),
        _ => format!("{} has no drop table yet", zone.name),
    })
}

/// A zone opens when it can be entered, the preceding campaign zone is cleared on this class,
/// and this character's talent level meets the gate. Elwynn needs neither clear nor talent.
pub fn zone_open(index: usize, talent_level: u32, clears: &[bool]) -> bool {
    let Some(zone) = ZONES.get(index) else {
        return false;
    };
    if content_lock(zone).is_some() || talent_level < zone.talent {
        return false;
    }
    match index.checked_sub(1) {
        None => true,
        Some(prev) => clears.get(prev).copied().unwrap_or(false),
    }
}

/// Maps that are not on the campaign path stay open. A campaign map uses [`zone_open`].
pub fn map_allowed(map_name: &str, talent_level: u32, clears: &[bool]) -> bool {
    match zone_index(map_name) {
        Some(zone) => zone_open(zone, talent_level, clears),
        None => true,
    }
}

/// Why a campaign zone is shut, when it is.
pub fn zone_lock_reason(index: usize, talent_level: u32, clears: &[bool]) -> Option<String> {
    if zone_open(index, talent_level, clears) {
        return None;
    }
    let zone = ZONES.get(index)?;
    if let Some(reason) = content_lock(zone) {
        return Some(reason);
    }
    if talent_level < zone.talent {
        return Some(format!("{} needs talent level {}", zone.name, zone.talent));
    }
    let prev = index
        .checked_sub(1)
        .and_then(|i| ZONES.get(i))
        .map(|z| z.name)
        .unwrap_or("the previous zone");
    Some(format!("Clear {prev} on this class first"))
}

/// Flat and percent bonuses one piece of a quality contributes. Each step keeps the previous
/// lines and adds the next, at higher numbers.
#[derive(Clone, Copy, Default, Debug)]
pub struct Bonus {
    /// Flat maximum health.
    pub health: f32,
    /// Fractional pickup-radius increase.
    pub pickup: f32,
    pub armor: f32,
    pub might: f32,
    pub crit: f32,
    pub multistrike: f32,
    pub speed: f32,
    pub leech: f32,
    pub area: f32,
    /// Added to the baseline double crit, so 0.25 makes a crit deal 2.25x.
    pub crit_damage: f32,
    pub reduction: f32,
    pub cooldown: f32,
    pub gold: f32,
    pub growth: f32,
    pub luck: f32,
    pub targets: u32,
}

impl Bonus {
    pub fn add(&mut self, other: Self) {
        self.health += other.health;
        self.pickup += other.pickup;
        self.armor += other.armor;
        self.might += other.might;
        self.crit += other.crit;
        self.multistrike += other.multistrike;
        self.speed += other.speed;
        self.leech += other.leech;
        self.area += other.area;
        self.crit_damage += other.crit_damage;
        self.reduction += other.reduction;
        self.cooldown += other.cooldown;
        self.gold += other.gold;
        self.growth += other.growth;
        self.luck += other.luck;
        self.targets += other.targets;
    }
}

pub fn quality_bonus(quality: Quality) -> Bonus {
    match quality {
        Quality::Poor => Bonus {
            armor: 4.0,
            ..Bonus::default()
        },
        Quality::Common => Bonus {
            armor: 6.0,
            might: 0.02,
            ..Bonus::default()
        },
        Quality::Uncommon => Bonus {
            armor: 8.0,
            might: 0.03,
            crit: 0.02,
            ..Bonus::default()
        },
        Quality::Rare => Bonus {
            armor: 10.0,
            might: 0.04,
            crit: 0.03,
            multistrike: 0.04,
            ..Bonus::default()
        },
        Quality::Epic => Bonus {
            armor: 14.0,
            might: 0.06,
            crit: 0.04,
            multistrike: 0.06,
            speed: 0.03,
            ..Bonus::default()
        },
        Quality::Legendary => Bonus {
            armor: 18.0,
            might: 0.08,
            crit: 0.05,
            multistrike: 0.08,
            speed: 0.04,
            crit_damage: 0.25,
            ..Bonus::default()
        },
    }
}

/// All nine worn slots together, independent of collection completion.
pub fn set_bonus() -> Bonus {
    Bonus {
        might: 0.10,
        crit: 0.05,
        ..Bonus::default()
    }
}

/// Cloak power. A legendary cloak is the account reward for nine epic class cloaks.
pub fn cloak_bonus(quality: Quality) -> Bonus {
    match quality {
        Quality::Uncommon => Bonus {
            might: 0.04,
            speed: 0.03,
            ..Bonus::default()
        },
        Quality::Rare => Bonus {
            might: 0.08,
            reduction: 0.05,
            multistrike: 0.05,
            ..Bonus::default()
        },
        Quality::Epic => Bonus {
            might: 0.12,
            reduction: 0.08,
            multistrike: 0.08,
            leech: 0.02,
            ..Bonus::default()
        },
        Quality::Legendary => Bonus {
            might: 0.20,
            reduction: 0.10,
            multistrike: 0.15,
            leech: 0.03,
            crit_damage: 0.50,
            targets: 1,
            ..Bonus::default()
        },
        Quality::Poor | Quality::Common => Bonus::default(),
    }
}

#[derive(Clone, Copy, Default)]
pub struct Feats {
    pub talent_max: bool,
    pub raid: bool,
    pub collection: bool,
    pub amulets: bool,
}

impl Feats {
    fn count(self) -> usize {
        [self.talent_max, self.raid, self.collection, self.amulets]
            .into_iter()
            .filter(|f| *f)
            .count()
    }
}

/// Cloak rarity from feats completed in any order. The raid feat never yields only an uncommon.
pub fn cloak_quality(feats: Feats) -> Option<Quality> {
    match feats.count() {
        0 => None,
        4 => Some(Quality::Epic),
        n if n >= 2 || feats.raid => Some(Quality::Rare),
        _ => Some(Quality::Uncommon),
    }
}

pub fn legendary_cloak(epic_cloaks: usize) -> bool {
    epic_cloaks >= 9
}

/// Nine slots by six qualities. The tenth slot of the draft was the off-hand stand-in.
pub const GRID_SLOTS: usize = 9;
pub const BANDS: usize = 6;
pub const PIECES: usize = GRID_SLOTS * BANDS;

pub const BANDS_NAMED: [&str; BANDS] = ["Common", "Uncommon", "Rare", "Epic", "PvP", "Legendary"];

/// One creature's roll. The piece's name depends on the class; the creature has to be on the map.
pub struct DropRow {
    pub zone: usize,
    pub slot: usize,
    pub band: usize,
    pub creature: &'static str,
    pub at_secs: u32,
    pub chance: u8,
}

pub static DROPS: &[DropRow] = &[
    // PvP names the equipment band; encounters remain horde survival.
    DropRow {
        zone: 8,
        slot: 0,
        band: 4,
        creature: "Lieutenant Stronghoof",
        at_secs: 120,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 1,
        band: 4,
        creature: "Lieutenant Lewis",
        at_secs: 240,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 2,
        band: 4,
        creature: "Lieutenant Grummus",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 3,
        band: 4,
        creature: "Commander Malgor",
        at_secs: 480,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 4,
        band: 4,
        creature: "Commander Louis Philips",
        at_secs: 600,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 5,
        band: 4,
        creature: "Captain Galvangar",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 6,
        band: 4,
        creature: "Captain Balinda Stonehearth",
        at_secs: 810,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 7,
        band: 4,
        creature: "Vanndar Stormpike",
        at_secs: 900,
        chance: 60,
    },
    DropRow {
        zone: 8,
        slot: 8,
        band: 4,
        creature: "Drek'Thar",
        at_secs: 960,
        chance: 60,
    },
    // Arathi is a Rare-tier survival source, independent of PvP objective rules.
    DropRow {
        zone: 4,
        slot: 0,
        band: 2,
        creature: "Defiler Farm Raider",
        at_secs: 120,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 1,
        band: 2,
        creature: "Arathor Stable Captain",
        at_secs: 240,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 2,
        band: 2,
        creature: "Defiler Mine Overseer",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 3,
        band: 2,
        creature: "Arathor Lumber Champion",
        at_secs: 480,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 4,
        band: 2,
        creature: "Defiler Blacksmith Champion",
        at_secs: 600,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 5,
        band: 2,
        creature: "Arathor Cavalry Captain",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 6,
        band: 2,
        creature: "Defiler Siege Commander",
        at_secs: 810,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 7,
        band: 2,
        creature: "Arathor Field Marshal",
        at_secs: 900,
        chance: 60,
    },
    DropRow {
        zone: 4,
        slot: 8,
        band: 2,
        creature: "Defiler Basin Warmaster",
        at_secs: 960,
        chance: 60,
    },
    // Authored faction-themed survival sources; Warsong rewards Uncommon gear.
    DropRow {
        zone: 2,
        slot: 0,
        band: 1,
        creature: "Warsong Scout Captain",
        at_secs: 120,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 1,
        band: 1,
        creature: "Silverwing Vanguard",
        at_secs: 240,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 2,
        band: 1,
        creature: "Warsong Shaman Elder",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 3,
        band: 1,
        creature: "Silverwing Sentinel Captain",
        at_secs: 480,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 4,
        band: 1,
        creature: "Warsong Shredder Foreman",
        at_secs: 600,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 5,
        band: 1,
        creature: "Warsong Grunt Champion",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 6,
        band: 1,
        creature: "Silverwing Blade Champion",
        at_secs: 810,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 7,
        band: 1,
        creature: "Silverwing Field Marshal",
        at_secs: 900,
        chance: 60,
    },
    DropRow {
        zone: 2,
        slot: 8,
        band: 1,
        creature: "Warsong Gulch Warmaster",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 0,
        slot: 0,
        band: 0,
        creature: "Hogger",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 0,
        slot: 1,
        band: 0,
        creature: "Mother Fang",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 0,
        slot: 2,
        band: 0,
        creature: "Narg the Taskmaster",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 0,
        slot: 3,
        band: 0,
        creature: "Thuros Lightfingers",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 0,
        slot: 8,
        band: 0,
        creature: "Fedfennel",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 1,
        slot: 7,
        band: 0,
        creature: "Lupos",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 1,
        slot: 6,
        band: 0,
        creature: "Naraxis",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 1,
        slot: 4,
        band: 0,
        creature: "Fenros",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 1,
        slot: 5,
        band: 0,
        creature: "Nefaru",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 0,
        band: 1,
        creature: "Ravasaur Matriarch",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 6,
        band: 1,
        creature: "Clutchmother Zavas",
        at_secs: 270,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 1,
        band: 1,
        creature: "Uhk'loc",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 7,
        band: 1,
        creature: "Gruff",
        at_secs: 450,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 2,
        band: 1,
        creature: "Blazerunner",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 3,
        band: 1,
        creature: "Baron Charr",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 4,
        band: 1,
        creature: "Tyrant Devilsaur",
        at_secs: 840,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 8,
        band: 1,
        creature: "U'cha",
        at_secs: 900,
        chance: 60,
    },
    DropRow {
        zone: 3,
        slot: 5,
        band: 1,
        creature: "King Mosh",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 11,
        slot: 8,
        band: 5,
        creature: "Kel'Thuzad",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 0,
        band: 2,
        creature: "Snarler",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 1,
        band: 2,
        creature: "Old Grizzlegut",
        at_secs: 270,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 2,
        band: 2,
        creature: "Diamond Head",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 3,
        band: 2,
        creature: "Qirot",
        at_secs: 450,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 4,
        band: 2,
        creature: "Arash-ethis",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 5,
        band: 2,
        creature: "Antilus the Soarer",
        at_secs: 630,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 6,
        band: 2,
        creature: "Mushgog",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 7,
        band: 2,
        creature: "The Razza",
        at_secs: 840,
        chance: 60,
    },
    DropRow {
        zone: 5,
        slot: 8,
        band: 2,
        creature: "Skarr the Unbreakable",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 0,
        band: 3,
        creature: "Kurmokk",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 1,
        band: 3,
        creature: "Verifonix",
        at_secs: 270,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 2,
        band: 3,
        creature: "Scale Belly",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 3,
        band: 3,
        creature: "Lord Sakrasis",
        at_secs: 450,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 4,
        band: 3,
        creature: "Rippa",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 5,
        band: 3,
        creature: "Tethis",
        at_secs: 630,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 6,
        band: 3,
        creature: "Bhag'thera",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 7,
        band: 3,
        creature: "King Bangalash",
        at_secs: 840,
        chance: 60,
    },
    DropRow {
        zone: 6,
        slot: 8,
        band: 3,
        creature: "King Mukla",
        at_secs: 960,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 0,
        band: 5,
        creature: "Gretheer",
        at_secs: 180,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 1,
        band: 5,
        creature: "Krellack",
        at_secs: 270,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 2,
        band: 5,
        creature: "Grubthor",
        at_secs: 360,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 3,
        band: 5,
        creature: "Huricanian",
        at_secs: 450,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 4,
        band: 5,
        creature: "Lapress",
        at_secs: 540,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 5,
        band: 5,
        creature: "Deathclasp",
        at_secs: 630,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 6,
        band: 5,
        creature: "Twilight Lord Everun",
        at_secs: 720,
        chance: 60,
    },
    DropRow {
        zone: 10,
        slot: 7,
        band: 5,
        creature: "Setis",
        at_secs: 960,
        chance: 60,
    },
];

pub fn piece_index(band: usize, slot: usize) -> Option<usize> {
    (band < BANDS && slot < GRID_SLOTS).then_some(band * GRID_SLOTS + slot)
}

/// A real item name, or nothing when that cell has no verified piece.
pub fn piece_name(class: usize, slot: usize, band: usize) -> Option<&'static str> {
    let row = GEAR_NAMES.get(class)?;
    if band == 0 && slot < GRID_SLOTS {
        return Some(row[slot]);
    }
    if matches!(band, 1..=4) || (band == 5 && slot < 8) {
        return super::gear_upgrades::piece(class, slot, band).map(|(name, _)| name);
    }
    if band == 5 && slot == 8 {
        return Some(row[10]);
    }
    None
}

/// Probability used by combat and source previews. Keen Eye only modifies boss rolls.
pub fn drop_chance(base: u8, boss: bool, keen_eye: u8) -> f32 {
    (f32::from(base) / 100.0
        + if boss {
            crate::rewards::keen_drop_bonus(keen_eye)
        } else {
            0.0
        })
    .clamp(0.0, 1.0)
}

pub fn drop_for(zone: usize, creature: &str) -> Option<&'static DropRow> {
    DROPS
        .iter()
        .find(|row| row.zone == zone && row.creature == creature)
}

/// The strongest owned copy in a slot: `(band, stacks)`.
pub fn best_piece(pieces: &[u8], slot: usize) -> Option<(usize, u8)> {
    (0..BANDS).rev().find_map(|band| {
        let n = pieces.get(band * GRID_SLOTS + slot).copied().unwrap_or(0);
        (n > 0).then_some((band, n))
    })
}

pub fn owned_pieces(pieces: &[u8]) -> usize {
    pieces.iter().take(PIECES).filter(|n| **n > 0).count()
}

/// Every slot at every collection quality, distinct from a complete worn set.
pub fn collection_complete(pieces: &[u8]) -> bool {
    pieces
        .get(..PIECES)
        .is_some_and(|grid| grid.iter().all(|&owned| owned > 0))
}

/// Nine worn pieces. A two-handed weapon does not add an off-hand cell.
pub fn set_complete(pieces: &[u8]) -> bool {
    (0..GRID_SLOTS).all(|slot| best_piece(pieces, slot).is_some())
}

/// Initial glove tuning follows the health/pickup/area/reduction/damage progression.
/// These coefficients are game balance data, not the original WoW item's stats.
pub fn glove_bonus(quality: Quality) -> Bonus {
    let (health, pickup, area, reduction, might, crit_damage) = match quality {
        Quality::Poor => (8.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        Quality::Common => (10.0, 0.12, 0.0, 0.0, 0.0, 0.0),
        Quality::Uncommon => (13.0, 0.14, 0.03, 0.0, 0.0, 0.0),
        Quality::Rare => (17.0, 0.195, 0.052, 0.012, 0.0, 0.0),
        Quality::Epic => (25.0, 0.521, 0.091, 0.029, 0.019, 0.0),
        Quality::Legendary => (35.0, 0.65, 0.12, 0.04, 0.03, 0.25),
    };
    Bonus {
        health,
        pickup,
        area,
        reduction,
        might,
        crit_damage,
        ..Bonus::default()
    }
}

/// Slot-specific stats shared by the item detail and the equipped combat sheet.
pub fn piece_bonus(band: usize, slot: usize) -> Bonus {
    if slot >= GRID_SLOTS || band >= BANDS {
        return Bonus::default();
    }
    if slot == 7 {
        let quality = crate::gear_tier::GearTier::from_legacy_band(band)
            .expect("collection band was checked above")
            .quality();
        let mut bonus = glove_bonus(quality);
        if band == 4 {
            bonus.health = 30.0;
            bonus.pickup = 0.58;
            bonus.area = 0.105;
            bonus.reduction = 0.035;
            bonus.might = 0.025;
            bonus.cooldown = 0.01;
        } else if band == 5 {
            bonus.cooldown = 0.02;
        }
        return bonus;
    }
    crate::gear_stats::piece(band, slot)
}

/// One selected piece per slot, its quality's stats, the worn set, and a cloak.
pub fn equipped_bonus(pieces: &[u8], cloak: Option<Quality>) -> Bonus {
    let mut total = Bonus::default();
    for slot in 0..GRID_SLOTS {
        let Some((band, _)) = best_piece(pieces, slot) else {
            continue;
        };
        total.add(piece_bonus(band, slot));
    }
    if set_complete(pieces) {
        total.add(set_bonus());
    }
    if let Some(q) = cloak {
        total.add(cloak_bonus(q));
    }
    total
}

pub const RINGS: usize = 9;
pub const TRINKETS: usize = 13;
pub const AMULETS: usize = 17;
pub const RINGS_WORN: usize = 2;
pub const TRINKETS_WORN: usize = 2;
pub const AMULETS_WORN: usize = 1;

pub static RING_NAMES: [&str; RINGS] = [
    "Band of the Unicorn",
    "Seal of Wrynn",
    "Ring of the Moon",
    "Magni's Will",
    "Ring of Protection",
    "Don Mauricio's Band",
    "Ring of Fortitude",
    "Band of Rumination",
    "Ring of the Fallen",
];

pub static TRINKET_NAMES: [&str; TRINKETS] = [
    "Rune of the Guard Captain",
    "Smotts' Compass",
    "Defender of the Timbermaw",
    "Ramstein's Lightning Bolts",
    "Mark of the Champion",
    "Talisman of Ephemeral Power",
    "Zandalarian Hero Medallion",
    "Eye of the Beast",
    "Briarwood Reed",
    "Drake Fang Talisman",
    "Royal Seal of Eldre'Thalas",
    "Mindtap Talisman",
    "Neltharion's Tear",
];

/// Account cosmetics. Display ids are `ItemDisplayInfo` rows the 1.12.1 client ships.
/// None of them adds a stat.
pub struct Tabard {
    pub name: &'static str,
    pub display: u32,
}

pub static TABARDS: &[Tabard] = &[Tabard {
    name: "Guild Tabard",
    display: 20621,
}];

pub static AMULET_NAMES: [&str; AMULETS] = [
    "Amulet of the Redeemed",
    "Medallion of Grand Marshal Morris",
    "Pendant of the Fallen Dragon",
    "Mark of Fordring",
    "Onyxia Tooth Pendant",
    "Prestor's Talisman",
    "The Eye of Hakkar",
    "Amulet of the Darkmoon",
    "Choker of the Fire Lord",
    "Gem of Nerubis",
    "Pendant of the Qiraji Guardian",
    "Amulet of Vek'nilash",
    "Sadist's Collar",
    "Stormrage's Talisman",
    "The Burrower's Shell",
    "Talisman of Ascendance",
    "Master Dragonslayer's Medallion",
];

/// One legendary amulet exists per primary ability. Seeing the card counts, whether or not it
/// was taken.
pub fn legendary_amulet(ability_key: &str) -> String {
    format!("Amulet of {ability_key}")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChestTier {
    C,
    B,
    A,
}

/// Verified `GameObjectDisplayInfo`: Chest01 (10), Chest02 (1), Chest03 (41).
pub fn chest_display(tier: ChestTier) -> u32 {
    match tier {
        ChestTier::C => crate::CHEST_DISPLAY,
        ChestTier::B => 1,
        ChestTier::A => 41,
    }
}

pub fn chest_gold(tier: ChestTier, level: u32) -> u32 {
    let base = 18 + level / 3;
    match tier {
        ChestTier::C => base,
        ChestTier::B => base + 14,
        ChestTier::A => base + 30,
    }
}

pub fn is_named_elite(map_name: &str, foe_name: &str) -> bool {
    zone_index(map_name)
        .and_then(|i| ZONES.get(i))
        .is_some_and(|z| z.elites.contains(&foe_name))
}

/// Chance a chest card is jewelry instead of an ability upgrade. Luck raises it.
pub fn jewel_chance(luck_percent: f32) -> f32 {
    (0.08 + luck_percent.max(0.0) / 100.0 * 0.42).min(0.50)
}

/// Among jewelry cards, the share that is a legendary amulet for an owned ability.
pub fn legend_share(luck_percent: f32) -> f32 {
    (luck_percent.max(0.0) / 100.0 * 0.25).min(0.25)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeCard {
    Upgrade(usize),
    Ring(usize),
    Trinket(usize),
    Amulet(usize),
    /// Owned-ability index supplied by the upgradeable list.
    Legend(usize),
}

/// Up to three distinct chest cards. Each roll chooses its reward category before sampling
/// unused rewards. An exhausted category produces no card, never an extra jewelry roll.
pub fn trade_cards(
    upgradeable: &[usize],
    luck_percent: f32,
    rolls: [f32; 3],
    picks: [u32; 3],
    extra_jewel: f32,
) -> Vec<TradeCard> {
    trade_cards_excluding(upgradeable, luck_percent, rolls, picks, extra_jewel, &[])
}

/// Draw with the same category rolls while excluding the current hand during replacement.
pub fn trade_cards_excluding(
    upgradeable: &[usize],
    luck_percent: f32,
    rolls: [f32; 3],
    picks: [u32; 3],
    extra_jewel: f32,
    excluded: &[TradeCard],
) -> Vec<TradeCard> {
    let chance = (jewel_chance(luck_percent) + extra_jewel.max(0.0)).min(0.75);
    let legend = (legend_share(luck_percent) * 100.0) as u32;
    let catalog = RINGS + TRINKETS + AMULETS;
    let mut out = Vec::with_capacity(3);
    for i in 0..3 {
        let n = picks[i];
        let mut pool: Vec<_> = if rolls[i] < chance || upgradeable.is_empty() {
            if !upgradeable.is_empty() && n % 100 < legend {
                upgradeable.iter().copied().map(TradeCard::Legend).collect()
            } else {
                (0..catalog)
                    .map(|slot| {
                        if slot < RINGS {
                            TradeCard::Ring(slot)
                        } else if slot < RINGS + TRINKETS {
                            TradeCard::Trinket(slot - RINGS)
                        } else {
                            TradeCard::Amulet(slot - RINGS - TRINKETS)
                        }
                    })
                    .collect()
            }
        } else {
            upgradeable
                .iter()
                .copied()
                .map(TradeCard::Upgrade)
                .collect()
        };
        pool.retain(|card| !out.contains(card) && !excluded.contains(card));
        if !pool.is_empty() {
            out.push(pool[n as usize % pool.len()]);
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JewelKind {
    Ring(u8),
    Trinket(u8),
    Amulet(u8),
    Legend(&'static str),
}

/// Jewelry worn for one run: two rings, two trinkets, one amulet, one legendary amulet.
#[derive(Clone, Copy, Default)]
pub struct Worn {
    rings: [Option<u8>; RINGS_WORN],
    trinkets: [Option<u8>; TRINKETS_WORN],
    amulet: Option<u8>,
    legend: Option<&'static str>,
}

impl Worn {
    pub fn equip(&mut self, kind: JewelKind) {
        match kind {
            JewelKind::Ring(i) => fill(&mut self.rings, i),
            JewelKind::Trinket(i) => fill(&mut self.trinkets, i),
            JewelKind::Amulet(i) => self.amulet = Some(i),
            JewelKind::Legend(key) => self.legend = Some(key),
        }
    }

    pub fn bonus(&self) -> Bonus {
        let mut total = Bonus::default();
        for i in self.rings.into_iter().flatten() {
            total.add(ring_bonus(i));
        }
        for i in self.trinkets.into_iter().flatten() {
            total.add(trinket_bonus(i));
        }
        if let Some(i) = self.amulet {
            total.add(amulet_bonus(i));
        }
        total
    }

    /// Cast-local bonuses; excluded from the hero's shared stat sheet.
    pub fn legendary_bonus(&self) -> Option<(&'static str, Bonus)> {
        self.legend.map(|key| {
            // Immunity consumes neither damage, critical damage nor extra targets.
            if key == "divine_shield" {
                return (
                    key,
                    Bonus {
                        cooldown: 0.15,
                        ..Bonus::default()
                    },
                );
            }
            (
                key,
                Bonus {
                    might: 0.05,
                    crit_damage: 0.15,
                    targets: 1,
                    ..Bonus::default()
                },
            )
        })
    }
}

fn fill(slots: &mut [Option<u8>], index: u8) {
    if let Some(empty) = slots.iter_mut().find(|s| s.is_none()) {
        *empty = Some(index);
    } else if let Some(first) = slots.first_mut() {
        *first = Some(index);
    }
}

fn ring_bonus(index: u8) -> Bonus {
    Bonus {
        might: 0.02 + 0.005 * f32::from(index % 3),
        crit: if index.is_multiple_of(2) { 0.01 } else { 0.0 },
        speed: if index % 2 == 1 { 0.02 } else { 0.0 },
        ..Bonus::default()
    }
}

fn trinket_bonus(index: u8) -> Bonus {
    match index % 3 {
        0 => Bonus {
            cooldown: 0.03,
            ..Bonus::default()
        },
        1 => Bonus {
            multistrike: 0.04,
            ..Bonus::default()
        },
        _ => Bonus {
            leech: 0.015,
            area: 0.03,
            ..Bonus::default()
        },
    }
}

fn amulet_bonus(index: u8) -> Bonus {
    match index % 4 {
        0 => Bonus {
            reduction: 0.03,
            ..Bonus::default()
        },
        1 => Bonus {
            gold: 0.10,
            ..Bonus::default()
        },
        2 => Bonus {
            growth: 0.08,
            ..Bonus::default()
        },
        _ => Bonus {
            luck: 4.0,
            ..Bonus::default()
        },
    }
}

pub fn talent_level(xp: u32) -> u32 {
    (xp / XP_PER_TALENT).min(TALENT_CAP)
}

/// Points still unspent. Spent points cannot exceed the level.
pub fn unspent_talents(xp: u32, spent: u32) -> u32 {
    talent_level(xp).saturating_sub(spent)
}

/// Points already spent in one tree.
pub fn points_in_tree(ranks: &[u8], tree: usize) -> u32 {
    let start = tree * NODES_PER_TREE;
    ranks
        .iter()
        .skip(start)
        .take(NODES_PER_TREE)
        .map(|n| u32::from(*n))
        .sum()
}

pub fn node_row(node: usize) -> u8 {
    NODE_ROW.get(node % NODES_PER_TREE).copied().unwrap_or(0)
}

pub fn node_max(node: usize) -> u8 {
    NODE_RANKS.get(node % NODES_PER_TREE).copied().unwrap_or(1)
}

fn points_before_row(ranks: &[u8], node: usize) -> u32 {
    let tree = node / NODES_PER_TREE;
    (tree * NODES_PER_TREE..(tree + 1) * NODES_PER_TREE)
        .filter(|&other| node_row(other) < node_row(node))
        .map(|other| u32::from(ranks.get(other).copied().unwrap_or(0)))
        .sum()
}

/// A node can be raised when its row is open, it is not full, and a point remains.
/// Row 0 is open. Each later row opens after 5 more points spent in that same tree.
pub fn can_learn(ranks: &[u8], node: usize, unspent: u32) -> bool {
    if unspent == 0 || node >= SPEC_LEN {
        return false;
    }
    if ranks.get(node).copied().unwrap_or(0) >= node_max(node) {
        return false;
    }
    points_before_row(ranks, node) >= u32::from(node_row(node)) * 5
}

/// One rank comes back when the node has one and every remaining rank still meets its row.
pub fn can_unlearn(ranks: &[u8], node: usize) -> bool {
    let rank = ranks.get(node).copied().unwrap_or(0);
    if rank == 0 || node >= SPEC_LEN {
        return false;
    }
    let mut next = ranks.to_vec();
    next[node] = rank - 1;
    (0..SPEC_LEN).all(|i| {
        next.get(i).copied().unwrap_or(0) == 0
            || points_before_row(&next, i) >= u32::from(node_row(i)) * 5
    })
}

/// One rank of a node. The three trees lean damage, spell, and survival.
/// Nodes past the sixth add a small amount of might; the 10, 20, and 30 milestones
/// are what change how spells play.
fn node_rank_bonus(node: usize) -> Bonus {
    let row = node % NODES_PER_TREE;
    if row >= 6 {
        return Bonus {
            might: 0.01,
            ..Bonus::default()
        };
    }
    match node / NODES_PER_TREE {
        0 => match row {
            0 => Bonus {
                might: 0.02,
                ..Bonus::default()
            },
            1 => Bonus {
                crit: 0.01,
                ..Bonus::default()
            },
            2 => Bonus {
                multistrike: 0.03,
                ..Bonus::default()
            },
            3 => Bonus {
                reduction: 0.015,
                ..Bonus::default()
            },
            _ => Bonus {
                targets: 1,
                ..Bonus::default()
            },
        },
        1 => match row {
            0 => Bonus {
                cooldown: 0.02,
                ..Bonus::default()
            },
            1 => Bonus {
                crit_damage: 0.05,
                ..Bonus::default()
            },
            2 => Bonus {
                area: 0.04,
                ..Bonus::default()
            },
            3 => Bonus {
                armor: 2.0,
                ..Bonus::default()
            },
            _ => Bonus {
                might: 0.08,
                ..Bonus::default()
            },
        },
        _ => match row {
            0 => Bonus {
                leech: 0.008,
                ..Bonus::default()
            },
            1 => Bonus {
                speed: 0.02,
                ..Bonus::default()
            },
            2 => Bonus {
                gold: 0.04,
                ..Bonus::default()
            },
            3 => Bonus {
                growth: 0.04,
                ..Bonus::default()
            },
            _ => Bonus {
                luck: 4.0,
                ..Bonus::default()
            },
        },
    }
}

fn scale_bonus(bonus: Bonus, rank: u8) -> Bonus {
    let r = f32::from(rank);
    Bonus {
        health: bonus.health * r,
        pickup: bonus.pickup * r,
        armor: bonus.armor * r,
        might: bonus.might * r,
        crit: bonus.crit * r,
        multistrike: bonus.multistrike * r,
        speed: bonus.speed * r,
        leech: bonus.leech * r,
        area: bonus.area * r,
        crit_damage: bonus.crit_damage * r,
        reduction: bonus.reduction * r,
        cooldown: bonus.cooldown * r,
        gold: bonus.gold * r,
        growth: bonus.growth * r,
        luck: bonus.luck * r,
        targets: bonus.targets * u32::from(rank),
    }
}

/// A real `Talent.dbc` node can take a point when its row is open, its prerequisite is met,
/// it is not full, and a point remains.
pub fn can_learn_talent(
    rank: u8,
    max_rank: u8,
    row: u8,
    tree_points: u32,
    prereq_met: bool,
    unspent: u32,
) -> bool {
    unspent > 0 && rank < max_rank && tree_points >= u32::from(row) * 5 && prereq_met
}

/// Points already spent in the three trees, without the skeleton's per-node lines.
/// The 10, 20, and 30 milestones are unchanged.
pub fn talent_bonus_spent(trees: [u32; 3]) -> Bonus {
    let mut total = Bonus::default();
    for points in trees {
        if points == 0 {
            continue;
        }
        total.add(Bonus {
            might: 0.01 * points as f32,
            ..Bonus::default()
        });
        if points >= 10 {
            total.add(Bonus {
                might: 0.04,
                ..Bonus::default()
            });
        }
        if points >= 20 {
            total.add(Bonus {
                multistrike: 0.10,
                ..Bonus::default()
            });
        }
        if points >= 30 {
            total.add(Bonus {
                targets: 1,
                ..Bonus::default()
            });
        }
    }
    total
}

/// Spent talents plus the 10, 20, and 30 point milestones in each tree.
/// Those milestones are what start to change how spells play.
pub fn talent_bonus(ranks: &[u8]) -> Bonus {
    let mut total = Bonus::default();
    for node in 0..SPEC_LEN {
        let rank = ranks.get(node).copied().unwrap_or(0);
        if rank > 0 {
            total.add(scale_bonus(node_rank_bonus(node), rank));
        }
    }
    for tree in 0..3 {
        let points = points_in_tree(ranks, tree);
        if points >= 10 {
            total.add(Bonus {
                might: 0.04,
                ..Bonus::default()
            });
        }
        if points >= 20 {
            total.add(Bonus {
                multistrike: 0.10,
                ..Bonus::default()
            });
        }
        if points >= 30 {
            total.add(Bonus {
                targets: 1,
                ..Bonus::default()
            });
        }
    }
    total
}

/// The 10, 20, and 30 point talents. Paladin uses the names from the talent-tab picture.
/// Every other class uses a talent that exists on that vanilla tree.
const MILESTONES: [[[&str; 3]; 3]; 9] = [
    [
        ["Deep Wounds", "Sweeping Strikes", "Mortal Strike"],
        ["Piercing Howl", "Death Wish", "Bloodthirst"],
        ["Last Stand", "Concussion Blow", "Shield Slam"],
    ],
    [
        ["Divine Favor", "Daybreak", "Light's Grace"],
        [
            "Hallowed Ground",
            "Ardent Defender",
            "Hammer of the Righteous",
        ],
        ["Seal of Command", "The Art of War", "Divine Storm"],
    ],
    [
        ["Unleashed Fury", "Frenzy", "Bestial Wrath"],
        ["Efficiency", "Mortal Shots", "Trueshot Aura"],
        ["Monster Slaying", "Surefooted", "Wyvern Sting"],
    ],
    [
        ["Malice", "Lethality", "Cold Blood"],
        [
            "Improved Sinister Strike",
            "Blade Flurry",
            "Adrenaline Rush",
        ],
        ["Master of Deception", "Preparation", "Premeditation"],
    ],
    [
        ["Unbreakable Will", "Mental Agility", "Power Infusion"],
        ["Holy Specialization", "Spiritual Guidance", "Lightwell"],
        ["Shadow Affinity", "Shadow Weaving", "Shadowform"],
    ],
    [
        ["Convection", "Elemental Focus", "Elemental Mastery"],
        ["Ancestral Knowledge", "Elemental Weapons", "Stormstrike"],
        [
            "Improved Healing Wave",
            "Nature's Swiftness",
            "Mana Tide Totem",
        ],
    ],
    [
        ["Arcane Focus", "Presence of Mind", "Arcane Power"],
        ["Improved Fireball", "Critical Mass", "Combustion"],
        ["Improved Frostbolt", "Ice Shards", "Ice Barrier"],
    ],
    [
        ["Improved Corruption", "Nightfall", "Siphon Life"],
        ["Improved Healthstone", "Demonic Sacrifice", "Soul Link"],
        ["Improved Shadow Bolt", "Ruin", "Conflagrate"],
    ],
    [
        ["Nature's Grasp", "Improved Moonfire", "Moonkin Form"],
        ["Ferocity", "Heart of the Wild", "Leader of the Pack"],
        ["Improved Healing Touch", "Nature's Swiftness", "Swiftmend"],
    ],
];

/// Milestone title for `at` points in one tree. `at` is 10, 20, or 30.
pub fn milestone_name(class: usize, tree: usize, at: u32) -> &'static str {
    let tier = match at {
        10 => 0,
        20 => 1,
        _ => 2,
    };
    MILESTONES
        .get(class)
        .and_then(|trees| trees.get(tree.min(2)))
        .map(|names| names[tier])
        .unwrap_or("Talent")
}

/// The node that stands for the 10, 20, or 30 point line: the first node of that row.
pub fn milestone_node(at: u32) -> Option<usize> {
    let row = match at {
        10 => 2,
        20 => 4,
        30 => 6,
        _ => return None,
    };
    NODE_ROW.iter().position(|r| *r == row)
}

/// Real tree names, three per class, in [`super::data::CLASSES`] order.
pub static TREES: [[&str; 3]; 9] = [
    ["Arms", "Fury", "Protection"],
    ["Holy", "Protection", "Retribution"],
    ["Beast Mastery", "Marksmanship", "Survival"],
    ["Assassination", "Combat", "Subtlety"],
    ["Discipline", "Holy", "Shadow"],
    ["Elemental", "Enhancement", "Restoration"],
    ["Arcane", "Fire", "Frost"],
    ["Affliction", "Demonology", "Destruction"],
    ["Balance", "Feral Combat", "Restoration"],
];

/// Real item names. Each row is the ten base pieces then the legendary weapon, in class order.
pub static GEAR_NAMES: [[&str; 11]; 9] = [
    [
        "Field Plate Helmet",
        "Field Plate Pauldrons",
        "Field Plate Armor",
        "Field Plate Girdle",
        "Field Plate Leggings",
        "Field Plate Boots",
        "Field Plate Vambraces",
        "Field Plate Gauntlets",
        "Whirlwind Axe",
        "Skullflame Shield",
        "Thunderfury, Blessed Blade of the Windseeker",
    ],
    [
        "Lightforge Helm",
        "Lightforge Spaulders",
        "Lightforge Breastplate",
        "Lightforge Belt",
        "Lightforge Legplates",
        "Lightforge Boots",
        "Lightforge Bracers",
        "Lightforge Gauntlets",
        "Quel'Serrar",
        "Aegis of the Scarlet Commander",
        "Corrupted Ashbringer",
    ],
    [
        "Beaststalker's Cap",
        "Beaststalker's Mantle",
        "Beaststalker's Tunic",
        "Beaststalker's Belt",
        "Beaststalker's Pants",
        "Beaststalker's Boots",
        "Beaststalker's Bindings",
        "Beaststalker's Gloves",
        "Blackcrow",
        "Ancient Sinew Wrapped Lamina",
        "Rhok'delar, Longbow of the Ancient Keepers",
    ],
    [
        "Shadowcraft Cap",
        "Shadowcraft Spaulders",
        "Shadowcraft Tunic",
        "Shadowcraft Belt",
        "Shadowcraft Pants",
        "Shadowcraft Boots",
        "Shadowcraft Bracers",
        "Shadowcraft Gloves",
        "Shadowblade",
        "Core Hound Tooth",
        "Death's Sting",
    ],
    [
        "Devout Crown",
        "Devout Mantle",
        "Devout Robe",
        "Devout Belt",
        "Devout Skirt",
        "Devout Sandals",
        "Devout Bracers",
        "Devout Gloves",
        "Anathema",
        "Drake Fang Talisman",
        "Atiesh, Greatstaff of the Guardian",
    ],
    [
        "Coif of Elements",
        "Pauldrons of Elements",
        "Vest of Elements",
        "Cord of Elements",
        "Kilt of Elements",
        "Boots of Elements",
        "Bindings of Elements",
        "Gauntlets of Elements",
        "Fist of Omokk",
        "Drake Fang Talisman",
        "Sulfuras, Hand of Ragnaros",
    ],
    [
        "Magister's Crown",
        "Magister's Mantle",
        "Magister's Robes",
        "Magister's Belt",
        "Magister's Leggings",
        "Magister's Boots",
        "Magister's Bindings",
        "Magister's Gloves",
        "Staff of Dominance",
        "Drake Fang Talisman",
        "Atiesh, Greatstaff of the Guardian",
    ],
    [
        "Deathmist Mask",
        "Deathmist Mantle",
        "Deathmist Robe",
        "Deathmist Belt",
        "Deathmist Leggings",
        "Deathmist Sandals",
        "Deathmist Bracers",
        "Deathmist Wraps",
        "Staff of the Shadow Flame",
        "Drake Fang Talisman",
        "Atiesh, Greatstaff of the Guardian",
    ],
    [
        "Wildheart Cowl",
        "Wildheart Spaulders",
        "Wildheart Vest",
        "Wildheart Belt",
        "Wildheart Kilt",
        "Wildheart Boots",
        "Wildheart Bracers",
        "Wildheart Gloves",
        "Wushoolay's Poker",
        "Drake Fang Talisman",
        "Atiesh, Greatstaff of the Guardian",
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn talent_experience_rewards_time_kills_defeated_bosses_and_later_zones() {
        let base = talent_award(1, 0, 0.0, 0, false, Some(0));
        assert_eq!(base, 10);
        assert_eq!(talent_award(1, 0, 60.0, 0, false, Some(0)), base + 6);
        assert_eq!(talent_award(1, 10, 0.0, 0, false, Some(0)), base + 1);
        assert_eq!(talent_award(1, 0, 0.0, 1, false, Some(0)), base + 10);
        assert_eq!(talent_award(1, 0, 0.0, 0, true, Some(0)), base + 32);
        assert_eq!(talent_award(1, 0, 0.0, 0, false, Some(1)), 11);
        assert_eq!(
            talent_award(
                u32::MAX,
                u32::MAX,
                f32::MAX,
                u32::MAX,
                true,
                Some(usize::MAX)
            ),
            u32::MAX
        );
        assert_eq!(talent_award(1, 0, f32::NAN, 0, false, None), base);
    }

    #[test]
    fn zones_open_in_order_and_honor_the_talent_gate() {
        let none = [false; CLEAR_COUNT];
        assert!(zone_open(0, 0, &none));
        assert!(!zone_open(1, 5, &none));
        let mut clears = none;
        clears[0] = true;
        assert!(!zone_open(1, 4, &clears));
        assert!(zone_open(1, 5, &clears));
        assert!(!zone_open(6, 24, &clears));
        clears[1] = true;
        assert!(!zone_open(6, 24, &clears));
        assert_eq!(
            zone_lock_reason(6, 24, &clears).as_deref(),
            Some("Gurubashi Arena world content is not ready")
        );
        clears[5] = true;
        assert!(!zone_open(6, 24, &clears));
        assert!(!zone_open(6, 23, &clears));
        assert!(!zone_open(2, 51, &[true; CLEAR_COUNT]));
        assert!(!zone_open(11, 48, &clears));
        clears[6] = true;
        assert!(!zone_open(11, 48, &clears));
        assert_eq!(
            zone_lock_reason(11, 48, &clears).as_deref(),
            Some("Naxxramas world content is not ready")
        );
        clears[10] = true;
        assert!(!zone_open(11, 48, &clears));
        assert!(!zone_open(11, 47, &clears));
    }

    #[test]
    fn qualities_add_exactly_one_stat_line_and_strengthen_previous_lines() {
        let qualities = [
            Quality::Poor,
            Quality::Common,
            Quality::Uncommon,
            Quality::Rare,
            Quality::Epic,
            Quality::Legendary,
        ];
        let mut previous = [0.0; 6];
        for (step, quality) in qualities.into_iter().enumerate() {
            let b = quality_bonus(quality);
            let lines = [
                b.armor,
                b.might,
                b.crit,
                b.multistrike,
                b.speed,
                b.crit_damage,
            ];
            assert_eq!(lines.iter().filter(|v| **v > 0.0).count(), step + 1);
            for index in 0..step {
                assert!(lines[index] > previous[index]);
            }
            assert_eq!(b.leech + b.area + b.reduction + b.health + b.pickup, 0.0);
            previous = lines;
        }
    }

    #[test]
    fn collection_completion_counts_catalog_cells_not_stacks_or_trailing_data() {
        assert!(!collection_complete(&[]));
        assert!(!collection_complete(&[1; PIECES - 1]));
        let mut pieces = vec![255; PIECES + 10];
        pieces[PIECES - 1] = 0;
        assert_eq!(owned_pieces(&pieces), PIECES - 1);
        assert!(!collection_complete(&pieces));
        assert!(set_complete(&pieces));
        pieces[PIECES - 1] = 1;
        assert_eq!(owned_pieces(&pieces), PIECES);
        assert!(collection_complete(&pieces));
        pieces[..PIECES].fill(1);
        assert!(collection_complete(&pieces));
    }

    #[test]
    fn collection_bands_keep_prior_stats_and_add_one_line_per_step() {
        for slot in 0..GRID_SLOTS {
            let mut previous = [0.0; 16];
            for band in 0..BANDS {
                let b = piece_bonus(band, slot);
                let lines = [
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
                ];
                assert_eq!(lines.iter().filter(|v| **v > 0.0).count(), band + 2);
                for (old, new) in previous.iter().zip(lines) {
                    if *old > 0.0 {
                        assert!(new > *old, "slot {slot}, band {band}");
                    }
                }
                previous = lines;
            }
        }
    }

    #[test]
    fn gloves_preserve_their_stat_identity_and_only_the_worn_copy_counts() {
        let qualities = [
            Quality::Poor,
            Quality::Common,
            Quality::Uncommon,
            Quality::Rare,
            Quality::Epic,
            Quality::Legendary,
        ];
        let mut previous = [0.0; 6];
        for (step, quality) in qualities.into_iter().enumerate() {
            let b = glove_bonus(quality);
            let lines = [
                b.health,
                b.pickup,
                b.area,
                b.reduction,
                b.might,
                b.crit_damage,
            ];
            assert_eq!(lines.iter().filter(|v| **v > 0.0).count(), step + 1);
            for index in 0..step {
                assert!(lines[index] > previous[index]);
            }
            assert_eq!(b.armor + b.crit + b.multistrike + b.speed + b.leech, 0.0);
            previous = lines;
        }
        let mut owned = [0; PIECES];
        owned[7] = 1;
        owned[3 * GRID_SLOTS + 7] = 99; // Legacy duplicate count cannot affect power.
        let mut gear = super::super::equipment::Equipment::legacy(&owned);
        let epic = equipped_bonus(&gear.worn_pieces(&owned), None);
        assert_eq!(epic.health, 25.0);
        assert_eq!(epic.pickup, 0.521);
        gear.wear(&owned, 7, Some(0));
        let common = equipped_bonus(&gear.worn_pieces(&owned), None);
        assert_eq!(common.health, 10.0);
        assert_eq!(common.pickup, 0.12);
        assert_eq!(common.might + common.area + common.reduction, 0.0);
        assert_eq!(piece_bonus(BANDS, 7).health, 0.0);
        assert_eq!(piece_bonus(0, GRID_SLOTS).armor, 0.0);
    }

    #[test]
    fn a_full_set_and_a_legendary_weapon_change_the_total() {
        let mut pieces = [0u8; PIECES];
        pieces[..GRID_SLOTS].fill(1);
        let plain = equipped_bonus(&pieces, None);
        assert!(plain.might > quality_bonus(Quality::Common).might * 4.0);
        assert!(set_complete(&pieces));
        pieces[0] = 0;
        let missing = equipped_bonus(&pieces, None);
        assert!(plain.might > missing.might);
        assert!(!set_complete(&pieces));
        pieces[0] = 1;
        pieces[8] = 0;
        pieces[5 * GRID_SLOTS + 8] = 1;
        let with_weapon = equipped_bonus(&pieces, None);
        assert!(with_weapon.crit_damage > plain.crit_damage);
        let mut one = [0u8; PIECES];
        one[0] = 1;
        let single = equipped_bonus(&one, None);
        one[0] = 2;
        assert_eq!(equipped_bonus(&one, None).armor, single.armor);
    }

    #[test]
    fn drops_name_a_creature_on_that_map() {
        assert_eq!(PIECES, 54);
        assert_eq!(DROPS.len(), 72);
        for row in DROPS {
            let zone = &ZONES[row.zone];
            let map = super::super::data::MAPS
                .iter()
                .position(|m| m.name == zone.map)
                .unwrap();
            let roster = super::super::data::roster(map);
            assert!(
                roster
                    .enemies
                    .iter()
                    .chain(roster.bosses.iter())
                    .any(|e| e.name == row.creature),
                "{} is not on {}",
                row.creature,
                zone.map
            );
            assert!(piece_name(1, row.slot, row.band).is_some());
        }
        assert!(DROPS
            .iter()
            .all(|row| row.creature != "Rhahk'Zor" && row.creature != "Edwin VanCleef"));
        assert!(drop_for(0, "Hogger").is_some());
        assert!(drop_for(0, "Rhahk'Zor").is_none());
    }

    #[test]
    fn cloaks_follow_feats_in_any_order() {
        assert_eq!(cloak_quality(Feats::default()), None);
        assert_eq!(
            cloak_quality(Feats {
                talent_max: true,
                ..Feats::default()
            }),
            Some(Quality::Uncommon)
        );
        assert_eq!(
            cloak_quality(Feats {
                raid: true,
                ..Feats::default()
            }),
            Some(Quality::Rare)
        );
        assert_eq!(
            cloak_quality(Feats {
                collection: true,
                amulets: true,
                ..Feats::default()
            }),
            Some(Quality::Rare)
        );
        assert_eq!(
            cloak_quality(Feats {
                talent_max: true,
                raid: true,
                collection: true,
                amulets: true,
            }),
            Some(Quality::Epic)
        );
        assert!(!legendary_cloak(8));
        assert!(legendary_cloak(9));
    }

    #[test]
    fn legacy_refunds_preserve_support_from_earlier_rows() {
        let mut ranks = [0; SPEC_LEN];
        ranks[0] = 5;
        ranks[2] = 5;
        ranks[4] = 1;
        assert!(!can_unlearn(&ranks, 0));
        assert!(!can_unlearn(&ranks, 2));
        assert!(can_unlearn(&ranks, 4));
        ranks[0] = 4;
        assert!(!can_learn(&ranks, 2, 1));
        assert!(!can_learn(&ranks, 4, 1));
        ranks[NODES_PER_TREE] = 5;
        assert!(
            !can_learn(&ranks, 4, 1),
            "another tree cannot supply row support"
        );
    }

    #[test]
    fn a_row_opens_after_five_points_and_milestones_change_the_sheet() {
        let mut ranks = vec![0u8; SPEC_LEN];
        assert!(can_learn(&ranks, 0, 1));
        assert!(can_learn(&ranks, 1, 1));
        assert!(!can_learn(&ranks, 2, 1));
        ranks[0] = 5;
        assert!(can_learn(&ranks, 2, 1));
        assert!(!can_learn(&ranks, 4, 1));
        assert!(!can_learn(&ranks, 0, 1));
        let trained = talent_bonus(&ranks);
        assert!(trained.might > 0.0);
        ranks[2] = 5;
        let ten = talent_bonus(&ranks);
        assert!(ten.might > trained.might);
        let mut deep = vec![0u8; SPEC_LEN];
        deep[..NODES_PER_TREE].copy_from_slice(&NODE_RANKS[..NODES_PER_TREE]);
        assert!(points_in_tree(&deep, 0) >= 30);
        assert!(talent_bonus(&deep).targets >= 1);
        assert!(talent_bonus(&deep).multistrike > 0.0);
        assert!(can_unlearn(&deep, 0));
        let mut tight = vec![0u8; SPEC_LEN];
        tight[0] = 4;
        tight[2] = 1;
        assert!(!can_unlearn(&tight, 0));
        assert!(can_unlearn(&tight, 2));
        assert_eq!(milestone_name(1, 0, 10), "Divine Favor");
        assert_eq!(milestone_name(1, 2, 30), "Divine Storm");
        assert_eq!(milestone_name(0, 0, 30), "Mortal Strike");
        assert!(can_learn_talent(0, 5, 0, 0, true, 1));
        assert!(!can_learn_talent(0, 5, 1, 4, true, 1));
        assert!(can_learn_talent(0, 5, 1, 5, true, 1));
        assert!(!can_learn_talent(0, 5, 0, 0, false, 1));
        let deep = talent_bonus_spent([30, 0, 0]);
        assert!(deep.targets >= 1);
        assert!(deep.might > 0.3);
        assert!(!milestone_name(0, 0, 10).ends_with(" 10"));
        assert_eq!(talent_level(5100), 51);
        assert_eq!(unspent_talents(250, 2), 0);
        assert_eq!(unspent_talents(250, 1), 1);
    }

    #[test]
    fn an_instanced_raid_requires_its_world_and_completed_content() {
        let wrong_world = Zone {
            map: "Elwynn Forest",
            ..ZONES[11]
        };
        assert_eq!(
            content_lock(&wrong_world).as_deref(),
            Some("Naxxramas world location is not ready")
        );
        assert_eq!(required_map_id("Naxxramas"), Some(533));
        assert!(!map_allowed("Naxxramas", 51, &[true; CLEAR_COUNT]));
        assert_eq!(
            zone_lock_reason(11, 51, &[true; CLEAR_COUNT]).as_deref(),
            Some("Naxxramas world content is not ready")
        );
        assert!(map_allowed("Elwynn Forest", 0, &[false; CLEAR_COUNT]));
    }

    #[test]
    fn catalogs_have_the_agreed_counts() {
        assert_eq!(ZONES.len(), CLEAR_COUNT);
        assert_eq!(ZONES[0].name, "Elwynn Forest");
        assert_eq!(ZONES[0].boss, "Hogger");
        assert!(!ZONES[0].elites.is_empty());
        assert_eq!(ZONES[6].name, "Gurubashi Arena");
        assert_eq!(ZONES[6].boss, "King Mukla");
        assert_eq!(ZONES.last().unwrap().name, "Naxxramas");
        assert_eq!(ZONES.last().unwrap().boss, "Kel'Thuzad");
        assert!(ZONES
            .iter()
            .all(|z| z.name != "Westfall" && z.name != "Western Plaguelands"));
        assert_eq!(RING_NAMES.len(), RINGS);
        assert_eq!(TRINKET_NAMES.len(), TRINKETS);
        assert_eq!(AMULET_NAMES.len(), AMULETS);
        assert_eq!((RINGS_WORN, TRINKETS_WORN, AMULETS_WORN), (2, 2, 1));
        assert_eq!(GEAR_NAMES.len(), 9);
        assert_eq!(TREES.len(), 9);
        assert!(GEAR_NAMES.iter().all(|row| row.len() == 11));
        assert!(two_handed(6));
        assert!(!two_handed(0));
        assert_eq!(legendary_amulet("Heroic Strike"), "Amulet of Heroic Strike");
        assert!(map_allowed("Elwynn Forest", 0, &[false; CLEAR_COUNT]));
        assert!(map_allowed("Westfall", 0, &[false; CLEAR_COUNT]));
        assert!(!map_allowed("Duskwood", 5, &[false; CLEAR_COUNT]));
        let mut cleared = [false; CLEAR_COUNT];
        cleared[0] = true;
        assert!(map_allowed("Duskwood", 5, &cleared));
        assert!(!map_allowed("Duskwood", 4, &cleared));
        assert!(map_allowed("The Barrens", 0, &[false; CLEAR_COUNT]));
        assert!(!map_allowed("Warsong Gulch", 60, &[true; CLEAR_COUNT]));
        assert!(!map_allowed("Fire Plume Ridge", 60, &[true; CLEAR_COUNT]));
        assert!(!map_allowed("Gates of Ahn'Qiraj", 60, &[true; CLEAR_COUNT]));
        assert_eq!(
            zone_lock_reason(1, 0, &[false; CLEAR_COUNT]).as_deref(),
            Some("Duskwood needs talent level 5")
        );
        assert_eq!(
            zone_lock_reason(2, 60, &[true; CLEAR_COUNT]).as_deref(),
            Some("Warsong Gulch world content is not ready")
        );
        assert_eq!(
            zone_lock_reason(3, 60, &[true; CLEAR_COUNT]).as_deref(),
            Some("Un'Goro Crater world content is not ready")
        );
    }

    #[test]
    fn chest_tiers_pay_more_and_use_the_chest_ladder() {
        assert!(chest_gold(ChestTier::C, 9) < chest_gold(ChestTier::B, 9));
        assert!(chest_gold(ChestTier::B, 9) < chest_gold(ChestTier::A, 9));
        assert_eq!(
            (
                chest_display(ChestTier::C),
                chest_display(ChestTier::B),
                chest_display(ChestTier::A)
            ),
            (10, 1, 41)
        );
        assert!(is_named_elite("Elwynn Forest", "Defias Bandit"));
        assert!(!is_named_elite("Elwynn Forest", "Young Wolf"));
    }

    #[test]
    fn chest_replacement_excludes_current_upgrades_jewelry_and_legends() {
        let excluded = [
            TradeCard::Upgrade(2),
            TradeCard::Ring(0),
            TradeCard::Legend(2),
        ];
        assert!(trade_cards_excluding(&[2], 0.0, [0.9; 3], [0; 3], 0.0, &excluded).is_empty());
        let jewels = trade_cards_excluding(&[2], 0.0, [0.0; 3], [0; 3], 0.0, &excluded);
        assert_eq!(jewels.len(), 3);
        assert!(jewels.iter().all(|card| !excluded.contains(card)));
        let legends = trade_cards_excluding(&[2, 5], 100.0, [0.0; 3], [0; 3], 0.0, &excluded);
        assert_eq!(legends, vec![TradeCard::Legend(5)]);
    }

    #[test]
    fn chest_hands_offer_distinct_eligible_rewards_without_promoting_exhausted_rolls() {
        // Reproduces the two-spell chest: three non-jewelry rolls must not repeat a spell.
        let hand = trade_cards(&[0, 2], 0.0, [0.9; 3], [1, 0, 2], 0.0);
        assert_eq!(hand, vec![TradeCard::Upgrade(2), TradeCard::Upgrade(0)]);
        assert_eq!(
            trade_cards(&[4], 0.0, [0.9; 3], [0; 3], 0.0),
            vec![TradeCard::Upgrade(4)]
        );
        for owned in [&[][..], &[0][..], &[1, 3][..], &[0, 2, 4, 5][..]] {
            for luck in [0.0, 100.0] {
                for roll in [0.0, 0.1, 0.9] {
                    for seed in 0..100 {
                        let hand = trade_cards(owned, luck, [roll; 3], [seed; 3], 0.0);
                        assert!(!hand.is_empty());
                        for (i, card) in hand.iter().enumerate() {
                            assert!(!hand[..i].contains(card), "{hand:?}");
                            if let TradeCard::Upgrade(index) | TradeCard::Legend(index) = card {
                                assert!(owned.contains(index));
                            }
                        }
                        if owned.is_empty() || (owned.len() >= 3 && roll == 0.9) {
                            assert_eq!(hand.len(), 3);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn luck_turns_a_chest_card_into_jewelry_and_can_offer_a_legend() {
        let owned = [0usize, 2];
        let plain = trade_cards(&owned, 0.0, [0.9, 0.9, 0.9], [1, 0, 2], 0.0);
        assert!(plain.iter().all(|c| matches!(c, TradeCard::Upgrade(_))));
        let jewel = trade_cards(&owned, 0.0, [0.0, 0.9, 0.9], [0, 0, 0], 0.0);
        assert_eq!(jewel[0], TradeCard::Ring(0));
        let legend = trade_cards(&owned, 100.0, [0.0, 0.9, 0.9], [0, 0, 0], 0.0);
        let missed = trade_cards(&owned, 0.0, [0.10, 0.9, 0.9], [0, 0, 0], 0.0);
        let eyed = trade_cards(&owned, 0.0, [0.10, 0.9, 0.9], [0, 0, 0], 0.05);
        assert!(matches!(missed[0], TradeCard::Upgrade(_)));
        assert!(!matches!(eyed[0], TradeCard::Upgrade(_)));
        assert_eq!(legend[0], TradeCard::Legend(0));
        let mut worn = Worn::default();
        worn.equip(JewelKind::Ring(0));
        worn.equip(JewelKind::Ring(1));
        worn.equip(JewelKind::Ring(4));
        assert!(worn.bonus().might > 0.0);
        assert_eq!(worn.rings[0], Some(4));
        assert_eq!(worn.rings[1], Some(1));
    }
}
