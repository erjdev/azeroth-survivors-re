//! Campaign rules that do not depend on a running world: zone order, gear quality, cloaks,
//! jewelry limits, and the talent skeleton. The menus read these; they do not restate them.

/// Talent experience for one point.
pub(super) const XP_PER_TALENT: u32 = 100;
/// Points a character can hold, matching a vanilla tree.
pub(super) const TALENT_CAP: u32 = 51;
/// Six nodes in a tree, five ranks each, so one tree can reach the 30-point milestone.
/// A character still cannot spend more than [`TALENT_CAP`] points across all three.
pub(super) const NODES_PER_TREE: usize = 6;
pub(super) const NODE_RANKS: [u8; NODES_PER_TREE] = [5, 5, 5, 5, 5, 5];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Quality {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

pub(super) fn quality_name(quality: Quality) -> &'static str {
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
pub(super) struct Zone {
    pub name: &'static str,
    pub map: &'static str,
    /// Talent level required, in addition to clearing the previous zone that can be entered.
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
pub(super) const CLEAR_COUNT: usize = 12;

pub(super) static ZONES: &[Zone] = &[
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
        elites: &[],
        boss: "",
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
        elites: &[],
        boss: "",
        ready: false,
        into_id: 72,
        champ_id: 92,
        points: 15,
    },
    Zone {
        name: "Feralas",
        map: "Feralas",
        talent: 20,
        elites: &[],
        boss: "",
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
        boss: "Hakkar",
        ready: true,
        into_id: 4,
        champ_id: 14,
        points: 20,
    },
    Zone {
        name: "Scarlet Monastery",
        map: "Scarlet Monastery",
        talent: 28,
        elites: &[],
        boss: "",
        ready: false,
        into_id: 74,
        champ_id: 94,
        points: 20,
    },
    Zone {
        name: "Alterac Valley",
        map: "Alterac Valley",
        talent: 32,
        elites: &[],
        boss: "",
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
        elites: &["Qiraji Gladiator", "Anubisath Sentinel"],
        boss: "Ossirian the Unscarred",
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
        ready: true,
        into_id: 7,
        champ_id: 17,
        points: 30,
    },
];

/// Gear slot order. A two-handed weapon occupies the off-hand slot.
pub(super) const SLOTS: [&str; 10] = [
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
pub(super) fn two_handed(class: usize) -> bool {
    matches!(class, 2 | 4 | 6 | 7 | 8)
}

pub(super) fn zone_index(map_name: &str) -> Option<usize> {
    ZONES.iter().position(|z| z.map == map_name)
}

/// Why a row cannot be entered yet, apart from the talent gate and the previous clear.
fn content_lock(zone: &Zone) -> Option<String> {
    let map = super::data::MAPS.iter().any(|m| m.name == zone.map);
    let boss = !zone.boss.is_empty();
    if zone.ready && map && boss {
        return None;
    }
    Some(match (map, boss) {
        (false, false) => format!("{} has no map and no boss yet", zone.name),
        (false, _) => format!("{} has no map yet", zone.name),
        (_, false) => format!("{} has no boss yet", zone.name),
        _ => format!("{} has no drop table yet", zone.name),
    })
}

/// The nearest earlier row that can actually be entered. A locked row does not block the path.
fn previous_ready(index: usize) -> Option<usize> {
    (0..index).rev().find(|&i| ZONES.get(i).is_some_and(|z| content_lock(z).is_none()))
}

/// A zone opens when it can be entered, the previous enterable zone is cleared on this class,
/// and this character's talent level meets the gate. Elwynn needs neither clear nor talent.
pub(super) fn zone_open(index: usize, talent_level: u32, clears: &[bool]) -> bool {
    let Some(zone) = ZONES.get(index) else {
        return false;
    };
    if content_lock(zone).is_some() || talent_level < zone.talent {
        return false;
    }
    match previous_ready(index) {
        None => true,
        Some(prev) => clears.get(prev).copied().unwrap_or(false),
    }
}

/// Maps that are not on the campaign path stay open. A campaign map uses [`zone_open`].
pub(super) fn map_allowed(map_name: &str, talent_level: u32, clears: &[bool]) -> bool {
    match zone_index(map_name) {
        Some(zone) => zone_open(zone, talent_level, clears),
        None => true,
    }
}

/// Why a campaign zone is shut, when it is.
pub(super) fn zone_lock_reason(index: usize, talent_level: u32, clears: &[bool]) -> Option<String> {
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
    let prev = previous_ready(index)
        .and_then(|i| ZONES.get(i))
        .map(|z| z.name)
        .unwrap_or("the previous zone");
    Some(format!("Clear {prev} on this class first"))
}

/// Flat and percent bonuses one piece of a quality contributes. Each step keeps the previous
/// lines and adds the next, at higher numbers.
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Bonus {
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
    pub(super) fn add(&mut self, other: Self) {
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

pub(super) fn quality_bonus(quality: Quality) -> Bonus {
    match quality {
        Quality::Poor => Bonus {
            armor: 4.0,
            ..Bonus::default()
        },
        Quality::Common => Bonus {
            armor: 6.0,
            might: 0.02,
            crit: 0.01,
            ..Bonus::default()
        },
        Quality::Uncommon => Bonus {
            armor: 8.0,
            might: 0.03,
            crit: 0.02,
            multistrike: 0.03,
            ..Bonus::default()
        },
        Quality::Rare => Bonus {
            armor: 10.0,
            might: 0.04,
            crit: 0.03,
            multistrike: 0.04,
            speed: 0.02,
            ..Bonus::default()
        },
        Quality::Epic => Bonus {
            armor: 14.0,
            might: 0.06,
            crit: 0.04,
            multistrike: 0.06,
            speed: 0.03,
            leech: 0.01,
            area: 0.04,
            ..Bonus::default()
        },
        Quality::Legendary => Bonus {
            armor: 18.0,
            might: 0.08,
            crit: 0.05,
            multistrike: 0.08,
            speed: 0.04,
            leech: 0.02,
            area: 0.06,
            crit_damage: 0.25,
            ..Bonus::default()
        },
    }
}

/// Ten base pieces together.
pub(super) fn set_bonus() -> Bonus {
    Bonus {
        might: 0.10,
        crit: 0.05,
        ..Bonus::default()
    }
}

/// Cloak power. A legendary cloak is the account reward for nine epic class cloaks.
pub(super) fn cloak_bonus(quality: Quality) -> Bonus {
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
pub(super) struct Feats {
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
pub(super) fn cloak_quality(feats: Feats) -> Option<Quality> {
    match feats.count() {
        0 => None,
        4 => Some(Quality::Epic),
        n if n >= 2 || feats.raid => Some(Quality::Rare),
        _ => Some(Quality::Uncommon),
    }
}

pub(super) fn legendary_cloak(epic_cloaks: usize) -> bool {
    epic_cloaks >= 9
}

/// Nine slots by six qualities. The tenth slot of the draft was the off-hand stand-in.
pub(super) const GRID_SLOTS: usize = 9;
pub(super) const BANDS: usize = 6;
pub(super) const PIECES: usize = GRID_SLOTS * BANDS;

pub(super) const BANDS_NAMED: [&str; BANDS] =
    ["Common", "Uncommon", "Rare", "Epic", "PvP", "Legendary"];

/// One creature's roll. The piece's name depends on the class; the creature has to be on the map.
pub(super) struct DropRow {
    pub zone: usize,
    pub slot: usize,
    pub band: usize,
    pub creature: &'static str,
    pub at_secs: u32,
    pub chance: u8,
}

pub(super) static DROPS: &[DropRow] = &[
    DropRow { zone: 0, slot: 0, band: 0, creature: "Hogger", at_secs: 180, chance: 60 },
    DropRow { zone: 0, slot: 1, band: 0, creature: "Mother Fang", at_secs: 360, chance: 60 },
    DropRow { zone: 0, slot: 2, band: 0, creature: "Narg the Taskmaster", at_secs: 540, chance: 60 },
    DropRow { zone: 0, slot: 3, band: 0, creature: "Thuros Lightfingers", at_secs: 720, chance: 60 },
    DropRow { zone: 0, slot: 8, band: 0, creature: "Fedfennel", at_secs: 960, chance: 60 },
    DropRow { zone: 1, slot: 7, band: 0, creature: "Lupos", at_secs: 180, chance: 60 },
    DropRow { zone: 1, slot: 6, band: 0, creature: "Naraxis", at_secs: 360, chance: 60 },
    DropRow { zone: 1, slot: 4, band: 0, creature: "Fenros", at_secs: 540, chance: 60 },
    DropRow { zone: 1, slot: 5, band: 0, creature: "Nefaru", at_secs: 960, chance: 60 },
    DropRow { zone: 11, slot: 8, band: 5, creature: "Kel'Thuzad", at_secs: 960, chance: 60 },
];

pub(super) fn piece_index(band: usize, slot: usize) -> Option<usize> {
    (band < BANDS && slot < GRID_SLOTS).then_some(band * GRID_SLOTS + slot)
}

/// A real item name, or nothing when that cell has no verified piece.
pub(super) fn piece_name(class: usize, slot: usize, band: usize) -> Option<&'static str> {
    let row = GEAR_NAMES.get(class)?;
    if band == 0 && slot < GRID_SLOTS {
        return Some(row[slot]);
    }
    if band == 5 && slot == 8 {
        return Some(row[10]);
    }
    None
}

pub(super) fn drop_for(zone: usize, creature: &str) -> Option<&'static DropRow> {
    DROPS.iter().find(|row| row.zone == zone && row.creature == creature)
}

/// The strongest owned copy in a slot: `(band, stacks)`.
pub(super) fn best_piece(pieces: &[u8], slot: usize) -> Option<(usize, u8)> {
    (0..BANDS).rev().find_map(|band| {
        let n = pieces.get(band * GRID_SLOTS + slot).copied().unwrap_or(0);
        (n > 0).then_some((band, n))
    })
}

pub(super) fn owned_pieces(pieces: &[u8]) -> usize {
    pieces.iter().filter(|n| **n > 0).count()
}

/// Nine worn pieces. A two-handed weapon does not add an off-hand cell.
pub(super) fn set_complete(pieces: &[u8]) -> bool {
    (0..GRID_SLOTS).all(|slot| best_piece(pieces, slot).is_some())
}

fn band_bonus(band: usize) -> Bonus {
    match band {
        0 => quality_bonus(Quality::Common),
        1 => quality_bonus(Quality::Uncommon),
        2 => quality_bonus(Quality::Rare),
        3 => quality_bonus(Quality::Epic),
        4 => Bonus {
            armor: 16.0,
            might: 0.07,
            crit: 0.045,
            multistrike: 0.07,
            speed: 0.035,
            leech: 0.015,
            area: 0.05,
            reduction: 0.03,
            ..Bonus::default()
        },
        _ => {
            let mut bonus = quality_bonus(Quality::Legendary);
            bonus.reduction = 0.04;
            bonus
        }
    }
}

fn scaled(bonus: Bonus, scale: f32) -> Bonus {
    Bonus {
        armor: bonus.armor * scale,
        might: bonus.might * scale,
        crit: bonus.crit * scale,
        multistrike: bonus.multistrike * scale,
        speed: bonus.speed * scale,
        leech: bonus.leech * scale,
        area: bonus.area * scale,
        crit_damage: bonus.crit_damage * scale,
        reduction: bonus.reduction * scale,
        cooldown: bonus.cooldown * scale,
        gold: bonus.gold * scale,
        growth: bonus.growth * scale,
        luck: bonus.luck * scale,
        targets: bonus.targets,
    }
}

/// The worn piece in each slot, plus a quarter of its lines for each extra copy, the set, and a cloak.
pub(super) fn equipped_bonus(pieces: &[u8], cloak: Option<Quality>) -> Bonus {
    let mut total = Bonus::default();
    for slot in 0..GRID_SLOTS {
        let Some((band, stacks)) = best_piece(pieces, slot) else {
            continue;
        };
        let extra = f32::from(stacks.saturating_sub(1).min(3)) * 0.25;
        total.add(scaled(band_bonus(band), 1.0 + extra));
    }
    if set_complete(pieces) {
        total.add(set_bonus());
    }
    if let Some(q) = cloak {
        total.add(cloak_bonus(q));
    }
    total
}

pub(super) const RINGS: usize = 9;
pub(super) const TRINKETS: usize = 13;
pub(super) const AMULETS: usize = 17;
pub(super) const RINGS_WORN: usize = 2;
pub(super) const TRINKETS_WORN: usize = 2;
pub(super) const AMULETS_WORN: usize = 1;

pub(super) static RING_NAMES: [&str; RINGS] = [
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

pub(super) static TRINKET_NAMES: [&str; TRINKETS] = [
    "Rune of the Guard Captain",
    "Smotts' Compass",
    "Defender of the Timbermaw",
    "Ramstein's Lightning Bolts",
    "Mark of the Champion",
    "Talisman of Ephemeral Power",
    "Zandalarian Hero Medallion",
    "Eye of the Beast",
    "Briarwood Reed",
    "Drakefang Talisman",
    "Royal Seal of Eldre'Thalas",
    "Mindtap Talisman",
    "Neltharion's Tear",
];

pub(super) static AMULET_NAMES: [&str; AMULETS] = [
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
pub(super) fn legendary_amulet(ability_key: &str) -> String {
    format!("Amulet of {ability_key}")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ChestTier {
    C,
    B,
    A,
}

/// `GameObjectDisplayInfo` 10, 11 and 12: the wooden chest ladder.
pub(super) fn chest_display(tier: ChestTier) -> u32 {
    match tier {
        ChestTier::C => super::stash::CHEST_DISPLAY,
        ChestTier::B => 11,
        ChestTier::A => 12,
    }
}

pub(super) fn chest_gold(tier: ChestTier, level: u32) -> u32 {
    let base = 18 + level / 3;
    match tier {
        ChestTier::C => base,
        ChestTier::B => base + 14,
        ChestTier::A => base + 30,
    }
}

pub(super) fn is_named_elite(map_name: &str, foe_name: &str) -> bool {
    zone_index(map_name)
        .and_then(|i| ZONES.get(i))
        .is_some_and(|z| z.elites.contains(&foe_name))
}

/// Chance a chest card is jewelry instead of an ability upgrade. Luck raises it.
pub(super) fn jewel_chance(luck_percent: f32) -> f32 {
    (0.08 + luck_percent.max(0.0) / 100.0 * 0.42).min(0.50)
}

/// Among jewelry cards, the share that is a legendary amulet for an owned ability.
pub(super) fn legend_share(luck_percent: f32) -> f32 {
    (luck_percent.max(0.0) / 100.0 * 0.25).min(0.25)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TradeCard {
    Upgrade(usize),
    Ring(usize),
    Trinket(usize),
    Amulet(usize),
    /// Index into the upgradeable-ability list, which is also the owned-ability index when that
    /// list is the upgradeable set.
    Legend(usize),
}

/// Three chest cards. `rolls[i]` below [`jewel_chance`] turns that card into jewelry. `picks`
/// chooses which upgrade or which catalog piece.
pub(super) fn trade_cards(
    upgradeable: &[usize],
    luck_percent: f32,
    rolls: [f32; 3],
    picks: [u32; 3],
    extra_jewel: f32,
) -> [TradeCard; 3] {
    let chance = (jewel_chance(luck_percent) + extra_jewel.max(0.0)).min(0.75);
    let legend = (legend_share(luck_percent) * 100.0) as u32;
    let catalog = RINGS + TRINKETS + AMULETS;
    std::array::from_fn(|i| {
        let jewel = rolls[i] < chance;
        if jewel || upgradeable.is_empty() {
            let n = picks[i];
            if !upgradeable.is_empty() && n % 100 < legend {
                TradeCard::Legend(upgradeable[n as usize % upgradeable.len()])
            } else {
                let slot = n as usize % catalog;
                if slot < RINGS {
                    TradeCard::Ring(slot)
                } else if slot < RINGS + TRINKETS {
                    TradeCard::Trinket(slot - RINGS)
                } else {
                    TradeCard::Amulet(slot - RINGS - TRINKETS)
                }
            }
        } else {
            TradeCard::Upgrade(upgradeable[picks[i] as usize % upgradeable.len()])
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JewelKind {
    Ring(u8),
    Trinket(u8),
    Amulet(u8),
    Legend(&'static str),
}

/// Jewelry worn for one run: two rings, two trinkets, one amulet, one legendary amulet.
#[derive(Clone, Copy, Default)]
pub(super) struct Worn {
    rings: [Option<u8>; RINGS_WORN],
    trinkets: [Option<u8>; TRINKETS_WORN],
    amulet: Option<u8>,
    legend: Option<&'static str>,
}

impl Worn {
    pub(super) fn equip(&mut self, kind: JewelKind) {
        match kind {
            JewelKind::Ring(i) => fill(&mut self.rings, i),
            JewelKind::Trinket(i) => fill(&mut self.trinkets, i),
            JewelKind::Amulet(i) => self.amulet = Some(i),
            JewelKind::Legend(key) => self.legend = Some(key),
        }
    }

    pub(super) fn bonus(&self) -> Bonus {
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
        if self.legend.is_some() {
            total.add(Bonus {
                might: 0.05,
                crit_damage: 0.15,
                targets: 1,
                ..Bonus::default()
            });
        }
        total
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
        crit: if index % 2 == 0 { 0.01 } else { 0.0 },
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

pub(super) fn talent_level(xp: u32) -> u32 {
    (xp / XP_PER_TALENT).min(TALENT_CAP)
}

/// Points still unspent. Spent points cannot exceed the level.
pub(super) fn unspent_talents(xp: u32, spent: u32) -> u32 {
    talent_level(xp).saturating_sub(spent)
}

/// Points already spent in one tree.
pub(super) fn points_in_tree(ranks: &[u8], tree: usize) -> u32 {
    let start = tree * NODES_PER_TREE;
    ranks
        .iter()
        .skip(start)
        .take(NODES_PER_TREE)
        .map(|n| u32::from(*n))
        .sum()
}

/// A node can be raised when its row is open, it is not full, and a point remains.
/// Row 0 is open. Each later row opens after 5 more points spent in that same tree.
pub(super) fn can_learn(ranks: &[u8], node: usize, unspent: u32) -> bool {
    if unspent == 0 || node >= NODES_PER_TREE * 3 {
        return false;
    }
    let row = node % NODES_PER_TREE;
    let max = NODE_RANKS[row];
    if ranks.get(node).copied().unwrap_or(0) >= max {
        return false;
    }
    points_in_tree(ranks, node / NODES_PER_TREE) >= (row as u32) * 5
}

/// One rank of a node. The three trees lean damage, spell, and survival.
fn node_rank_bonus(node: usize) -> Bonus {
    let row = node % NODES_PER_TREE;
    match node / NODES_PER_TREE {
        0 => match row {
            0 => Bonus { might: 0.02, ..Bonus::default() },
            1 => Bonus { crit: 0.01, ..Bonus::default() },
            2 => Bonus { multistrike: 0.03, ..Bonus::default() },
            3 => Bonus { reduction: 0.015, ..Bonus::default() },
            _ => Bonus { targets: 1, ..Bonus::default() },
        },
        1 => match row {
            0 => Bonus { cooldown: 0.02, ..Bonus::default() },
            1 => Bonus { crit_damage: 0.05, ..Bonus::default() },
            2 => Bonus { area: 0.04, ..Bonus::default() },
            3 => Bonus { armor: 2.0, ..Bonus::default() },
            _ => Bonus { might: 0.08, ..Bonus::default() },
        },
        _ => match row {
            0 => Bonus { leech: 0.008, ..Bonus::default() },
            1 => Bonus { speed: 0.02, ..Bonus::default() },
            2 => Bonus { gold: 0.04, ..Bonus::default() },
            3 => Bonus { growth: 0.04, ..Bonus::default() },
            _ => Bonus { luck: 4.0, ..Bonus::default() },
        },
    }
}

fn scale_bonus(bonus: Bonus, rank: u8) -> Bonus {
    let r = f32::from(rank);
    Bonus {
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

/// Spent talents plus the 10, 20, and 30 point milestones in each tree.
/// Those milestones are what start to change how spells play.
pub(super) fn talent_bonus(ranks: &[u8]) -> Bonus {
    let mut total = Bonus::default();
    for node in 0..NODES_PER_TREE * 3 {
        let rank = ranks.get(node).copied().unwrap_or(0);
        if rank > 0 {
            total.add(scale_bonus(node_rank_bonus(node), rank));
        }
    }
    for tree in 0..3 {
        let points = points_in_tree(ranks, tree);
        if points >= 10 {
            total.add(Bonus { might: 0.04, ..Bonus::default() });
        }
        if points >= 20 {
            total.add(Bonus { multistrike: 0.10, ..Bonus::default() });
        }
        if points >= 30 {
            total.add(Bonus { targets: 1, ..Bonus::default() });
        }
    }
    total
}

/// Milestone titles. Paladin uses the names from the talent tab; other classes use the tree.
pub(super) fn milestone_name(class: usize, tree: usize, at: u32) -> String {
    const PALADIN: [[&str; 3]; 3] = [
        ["Divine Favor", "Daybreak", "Light's Grace"],
        ["Hallowed Ground", "Ardent Defender", "Hammer of the Righteous"],
        ["Seal of Command", "The Art of War", "Divine Storm"],
    ];
    let tier = match at {
        10 => 0,
        20 => 1,
        _ => 2,
    };
    if class == 1 {
        return PALADIN[tree.min(2)][tier].to_string();
    }
    let tree_name = TREES.get(class).map(|t| t[tree.min(2)]).unwrap_or("Tree");
    format!("{tree_name} {at}")
}

/// Real tree names, three per class, in [`super::data::CLASSES`] order.
pub(super) static TREES: [[&str; 3]; 9] = [
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
pub(super) static GEAR_NAMES: [[&str; 11]; 9] = [
    [
        "Field Plate Helmet",
        "Field Plate Pauldrons",
        "Field Plate Armor",
        "Field Plate Girdle",
        "Field Plate Leggings",
        "Field Plate Boots",
        "Field Plate Bracers",
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
        "Drakefang Talisman",
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
        "Drakefang Talisman",
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
        "Drakefang Talisman",
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
        "Drakefang Talisman",
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
        "Drakefang Talisman",
        "Atiesh, Greatstaff of the Guardian",
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(zone_open(6, 24, &clears));
        assert!(!zone_open(6, 23, &clears));
        assert!(!zone_open(2, 51, &[true; CLEAR_COUNT]));
        assert!(!zone_open(11, 48, &clears));
        clears[6] = true;
        assert!(zone_open(11, 48, &clears));
        assert!(!zone_open(11, 47, &clears));
    }

    #[test]
    fn qualities_add_a_stat_line_at_each_step() {
        let poor = quality_bonus(Quality::Poor);
        assert!(poor.armor > 0.0 && poor.might == 0.0);
        let common = quality_bonus(Quality::Common);
        assert!(common.might > 0.0 && common.multistrike == 0.0);
        let uncommon = quality_bonus(Quality::Uncommon);
        assert!(uncommon.multistrike > 0.0 && uncommon.speed == 0.0);
        let rare = quality_bonus(Quality::Rare);
        assert!(rare.speed > 0.0 && rare.leech == 0.0);
        let epic = quality_bonus(Quality::Epic);
        assert!(epic.leech > 0.0 && epic.crit_damage == 0.0);
        let legend = quality_bonus(Quality::Legendary);
        assert!(legend.crit_damage > 0.0);
        assert!(legend.might > epic.might);
    }

    #[test]
    fn a_full_set_and_a_legendary_weapon_change_the_total() {
        let mut pieces = [0u8; PIECES];
        for slot in 0..GRID_SLOTS {
            pieces[slot] = 1;
        }
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
        assert!(equipped_bonus(&one, None).armor > single.armor);
    }

    #[test]
    fn drops_name_a_creature_on_that_map() {
        assert_eq!(PIECES, 54);
        assert_eq!(DROPS.len(), 10);
        for row in DROPS {
            let zone = &ZONES[row.zone];
            let map = super::super::data::MAPS
                .iter()
                .position(|m| m.name == zone.map)
                .unwrap();
            let roster = super::super::data::roster(map);
            assert!(
                roster.enemies.iter().chain(roster.bosses.iter()).any(|e| e.name == row.creature),
                "{} is not on {}",
                row.creature,
                zone.map
            );
            assert!(piece_name(1, row.slot, row.band).is_some());
        }
        assert!(DROPS.iter().all(|row| row.creature != "Rhahk'Zor" && row.creature != "Edwin VanCleef"));
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
    fn a_row_opens_after_five_points_and_milestones_change_the_sheet() {
        let mut ranks = vec![0u8; 15];
        assert!(can_learn(&ranks, 0, 1));
        assert!(!can_learn(&ranks, 1, 1));
        ranks[0] = 5;
        assert!(can_learn(&ranks, 1, 1));
        assert!(!can_learn(&ranks, 2, 1));
        assert!(!can_learn(&ranks, 0, 1));
        let trained = talent_bonus(&ranks);
        assert!(trained.might > 0.0);
        ranks[0] = 5;
        ranks[1] = 5;
        let ten = talent_bonus(&ranks);
        assert!(ten.might > trained.might);
        let mut deep = vec![0u8; NODES_PER_TREE * 3];
        for rank in &mut deep[..NODES_PER_TREE] {
            *rank = 5;
        }
        assert_eq!(points_in_tree(&deep, 0), 30);
        assert!(talent_bonus(&deep).targets >= 1);
        assert!(talent_bonus(&deep).multistrike > 0.0);
        assert_eq!(milestone_name(1, 0, 10), "Divine Favor");
        assert_eq!(milestone_name(1, 2, 30), "Divine Storm");
        assert_eq!(talent_level(5100), 51);
        assert_eq!(unspent_talents(250, 2), 0);
        assert_eq!(unspent_talents(250, 1), 1);
    }

    #[test]
    fn catalogs_have_the_agreed_counts() {
        assert_eq!(ZONES.len(), CLEAR_COUNT);
        assert_eq!(ZONES[0].name, "Elwynn Forest");
        assert_eq!(ZONES[0].boss, "Hogger");
        assert!(!ZONES[0].elites.is_empty());
        assert_eq!(ZONES[6].name, "Gurubashi Arena");
        assert_eq!(ZONES[6].boss, "Hakkar");
        assert_eq!(ZONES.last().unwrap().name, "Naxxramas");
        assert_eq!(ZONES.last().unwrap().boss, "Kel'Thuzad");
        assert!(ZONES.iter().all(|z| z.name != "Westfall" && z.name != "Western Plaguelands"));
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
            Some("Warsong Gulch has no map and no boss yet")
        );
        assert_eq!(
            zone_lock_reason(3, 60, &[true; CLEAR_COUNT]).as_deref(),
            Some("Un'Goro Crater has no drop table yet")
        );
    }

    #[test]
    fn chest_tiers_pay_more_and_use_the_chest_ladder() {
        assert!(chest_gold(ChestTier::C, 9) < chest_gold(ChestTier::B, 9));
        assert!(chest_gold(ChestTier::B, 9) < chest_gold(ChestTier::A, 9));
        assert_eq!(
            (chest_display(ChestTier::C), chest_display(ChestTier::B), chest_display(ChestTier::A)),
            (10, 11, 12)
        );
        assert!(is_named_elite("Elwynn Forest", "Defias Bandit"));
        assert!(!is_named_elite("Elwynn Forest", "Young Wolf"));
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
