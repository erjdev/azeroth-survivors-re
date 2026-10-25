//! The mode's tables: classes, abilities, passives, enemies and the wave clock. Spell ids name
//! `Spell.dbc` rows only for their icon, name and visual chain; every number here is the mode's.

/// A weapon on the hero's virtual item slots: `(slot, display, class, subclass, material, inv,
/// sheath)`, the creature form of `UNIT_VIRTUAL_ITEM_SLOT_DISPLAY` and `_INFO`.
pub type Weapon = (u16, u32, u8, u8, u8, u8, u8);

pub struct ClassDef {
    pub id: u8,
    pub name: &'static str,
    /// `RAID_CLASS_COLORS`, 0xRRGGBB.
    pub color: u32,
    /// `UNIT_FIELD_BYTES_0` byte 3.
    pub power: u8,
    pub start: &'static str,
    pub pool: &'static [&'static str],
    /// Class-trainer `CreatureDisplayInfo` ids, deduped by race and sex at the menu.
    pub looks: &'static [u32],
    pub weapons: &'static [Weapon],
    /// The sheath state the hero fights in: 1 melee, 2 ranged.
    pub sheath: u8,
}

impl ClassDef {
    /// The class's actual spell catalog, in offer/display order, with the starter once.
    pub fn ability_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for key in std::iter::once(self.start).chain(self.pool.iter().copied()) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    /// A missile at the nearest enemies (or instant if the spell has no speed).
    Bolt,
    /// An instant hit on the nearest enemies in range.
    Strike,
    /// A close hit on the nearest enemies in melee reach.
    Melee,
    /// Everything in a radius around the hero.
    Nova,
    /// Everything in a cone toward the nearest enemy.
    Cone,
    /// A ground area at the densest nearby enemy, ticking for `duration`.
    Ground,
    /// A ground area under the hero, ticking for `duration`.
    GroundSelf,
    /// Jumps from enemy to enemy.
    Chain,
    /// A damage-over-time on several enemies.
    Dot,
    /// Always on: pulses every `cooldown` around the hero, with the spell's state visual on him.
    Aura,
    /// Always on: zaps one enemy close to the hero every `cooldown`.
    Orbit,
    /// Absorbs `damage` for `duration`.
    Shield,
    /// Immune for `duration`.
    Immune,
    /// Returns `damage` to every attacker for `duration`.
    Thorns,
    /// Heals `damage` over `duration`.
    Heal,
    /// Drops a totem that casts `sub` for `duration`.
    Totem,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Effect {
    None,
    /// Speed multiplier for seconds.
    Slow(f32, f32),
    Root(f32),
    Stun(f32),
    Fear(f32),
    /// Outgoing damage multiplier for seconds.
    Weaken(f32, f32),
    /// Heal the hero by this fraction of damage dealt.
    Drain(f32),
    /// Kill outright below this health fraction.
    Execute(f32),
}

impl Effect {
    pub fn duration(self) -> f32 {
        match self {
            Self::Slow(_, t)
            | Self::Weaken(_, t)
            | Self::Root(t)
            | Self::Stun(t)
            | Self::Fear(t) => t,
            _ => 0.0,
        }
    }

    pub fn scaled_duration(self, multiplier: f32) -> Self {
        match self {
            Self::Slow(m, t) => Self::Slow(m, t * multiplier),
            Self::Weaken(m, t) => Self::Weaken(m, t * multiplier),
            Self::Root(t) => Self::Root(t * multiplier),
            Self::Stun(t) => Self::Stun(t * multiplier),
            Self::Fear(t) => Self::Fear(t * multiplier),
            other => other,
        }
    }
}

pub struct AbilityDef {
    pub key: &'static str,
    pub spell: u32,
    pub kind: Kind,
    /// Direct damage; per tick for areas and pulses; absorb, heal or reflect for the self kinds.
    pub damage: f32,
    /// Damage over time, per second, for `duration`.
    pub dot: f32,
    pub cooldown: f32,
    pub range: f32,
    /// Splash for single hits, area for the area kinds, the jump for a chain.
    pub radius: f32,
    pub count: u32,
    pub duration: f32,
    pub effect: Effect,
    /// The spell whose state kit marks an affected enemy, 0 for none.
    pub mark: u32,
    /// A totem's display, and the spell it casts.
    pub totem: (u32, u32),
    /// The area's own visual hangs overhead and would hide the field from the camera, so the
    /// area plays the spell's impact on whatever it hits instead.
    pub overhead: bool,
}

const fn ab(key: &'static str, spell: u32, kind: Kind) -> AbilityDef {
    AbilityDef {
        key,
        spell,
        kind,
        damage: 0.0,
        dot: 0.0,
        cooldown: 2.0,
        range: 20.0,
        radius: 0.0,
        count: 1,
        duration: 0.0,
        effect: Effect::None,
        mark: 0,
        totem: (0, 0),
        overhead: false,
    }
}

macro_rules! ability {
    ($key:literal, $spell:literal, $kind:ident { $($f:ident: $v:expr),* $(,)? }) => {
        AbilityDef { $($f: $v,)* ..ab($key, $spell, Kind::$kind) }
    };
}

pub static ABILITIES: &[AbilityDef] = &[
    // --- Mage ---
    ability!(
        "frostbolt",
        116,
        Bolt {
            damage: 12.0,
            cooldown: 1.3,
            range: 24.0,
            effect: Effect::Slow(0.5, 2.5),
            mark: 116
        }
    ),
    ability!(
        "fireball",
        133,
        Bolt {
            damage: 20.0,
            cooldown: 2.4,
            range: 26.0,
            radius: 3.5
        }
    ),
    ability!(
        "arcane_missiles",
        7268,
        Bolt {
            damage: 6.0,
            cooldown: 1.9,
            range: 22.0,
            count: 3
        }
    ),
    ability!(
        "frost_nova",
        122,
        Nova {
            damage: 8.0,
            cooldown: 7.0,
            radius: 9.0,
            effect: Effect::Root(2.5),
            mark: 122
        }
    ),
    ability!(
        "arcane_explosion",
        1449,
        Nova {
            damage: 10.0,
            cooldown: 2.8,
            radius: 7.0
        }
    ),
    ability!(
        "blizzard",
        10,
        Ground {
            damage: 7.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0,
            effect: Effect::Slow(0.6, 1.5)
        }
    ),
    ability!(
        "flamestrike",
        2120,
        Ground {
            damage: 14.0,
            cooldown: 8.0,
            range: 20.0,
            radius: 5.0,
            duration: 4.0
        }
    ),
    ability!(
        "fire_blast",
        2136,
        Strike {
            damage: 26.0,
            cooldown: 3.5,
            range: 18.0
        }
    ),
    ability!(
        "cone_of_cold",
        120,
        Cone {
            damage: 14.0,
            cooldown: 4.5,
            radius: 11.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 120
        }
    ),
    ability!(
        "blast_wave",
        11113,
        Nova {
            damage: 18.0,
            cooldown: 6.5,
            radius: 8.0,
            effect: Effect::Slow(0.6, 2.0)
        }
    ),
    ability!(
        "pyroblast",
        11366,
        Bolt {
            damage: 70.0,
            dot: 6.0,
            duration: 6.0,
            cooldown: 6.0,
            range: 30.0
        }
    ),
    // --- Warrior ---
    ability!(
        "cleave",
        845,
        Melee {
            damage: 14.0,
            cooldown: 1.1,
            range: 5.5,
            count: 3
        }
    ),
    ability!(
        "heroic_strike",
        78,
        Melee {
            damage: 28.0,
            cooldown: 1.6,
            range: 5.5
        }
    ),
    ability!(
        "whirlwind",
        1680,
        Nova {
            damage: 16.0,
            cooldown: 4.0,
            radius: 7.0
        }
    ),
    ability!(
        "thunder_clap",
        6343,
        Nova {
            damage: 10.0,
            cooldown: 5.0,
            radius: 8.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 6343
        }
    ),
    ability!(
        "sweeping_strikes",
        12292,
        Aura {
            damage: 6.0,
            cooldown: 0.6,
            radius: 5.5
        }
    ),
    ability!(
        "demoralizing_shout",
        1160,
        Nova {
            damage: 4.0,
            cooldown: 8.0,
            radius: 11.0,
            effect: Effect::Weaken(0.5, 5.0)
        }
    ),
    ability!(
        "execute",
        5308,
        Strike {
            damage: 40.0,
            cooldown: 2.0,
            range: 8.0,
            count: 2,
            effect: Effect::Execute(0.3)
        }
    ),
    ability!(
        "mortal_strike",
        12294,
        Melee {
            damage: 48.0,
            cooldown: 3.0,
            range: 6.0
        }
    ),
    ability!(
        "retaliation",
        20230,
        Thorns {
            damage: 14.0,
            cooldown: 18.0,
            duration: 8.0
        }
    ),
    // --- Rogue ---
    ability!(
        "sinister_strike",
        1752,
        Melee {
            damage: 15.0,
            cooldown: 0.85,
            range: 5.5
        }
    ),
    ability!(
        "eviscerate",
        2098,
        Melee {
            damage: 42.0,
            cooldown: 2.8,
            range: 5.5
        }
    ),
    ability!(
        "blade_flurry",
        13877,
        Nova {
            damage: 11.0,
            cooldown: 3.0,
            radius: 6.0
        }
    ),
    ability!(
        "rupture",
        1943,
        Dot {
            dot: 5.0,
            cooldown: 3.5,
            range: 10.0,
            count: 3,
            duration: 6.0
        }
    ),
    ability!(
        "gouge",
        1776,
        Melee {
            damage: 8.0,
            cooldown: 4.0,
            range: 6.0,
            count: 2,
            effect: Effect::Stun(2.0),
            mark: 1776
        }
    ),
    ability!(
        "kidney_shot",
        408,
        Melee {
            damage: 12.0,
            cooldown: 6.0,
            range: 6.0,
            effect: Effect::Stun(3.5),
            mark: 408
        }
    ),
    // --- Priest ---
    ability!(
        "smite",
        585,
        Strike {
            damage: 14.0,
            cooldown: 1.2,
            range: 20.0
        }
    ),
    ability!(
        "shadow_word_pain",
        589,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 18.0,
            count: 3,
            duration: 9.0
        }
    ),
    ability!(
        "holy_fire",
        14914,
        Strike {
            damage: 22.0,
            dot: 3.0,
            duration: 6.0,
            cooldown: 3.5,
            range: 20.0
        }
    ),
    ability!(
        "mind_blast",
        8092,
        Strike {
            damage: 32.0,
            cooldown: 3.5,
            range: 22.0
        }
    ),
    ability!(
        "holy_nova",
        15237,
        Nova {
            damage: 11.0,
            cooldown: 3.2,
            radius: 9.0,
            effect: Effect::Drain(0.1)
        }
    ),
    ability!(
        "psychic_scream",
        8122,
        Nova {
            damage: 2.0,
            cooldown: 12.0,
            radius: 8.0,
            effect: Effect::Fear(3.5),
            mark: 8122
        }
    ),
    ability!(
        "power_word_shield",
        17,
        Shield {
            damage: 30.0,
            cooldown: 14.0,
            duration: 10.0
        }
    ),
    // --- Warlock ---
    ability!(
        "shadow_bolt",
        686,
        Bolt {
            damage: 15.0,
            cooldown: 1.4,
            range: 24.0
        }
    ),
    ability!(
        "corruption",
        172,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 172
        }
    ),
    ability!(
        "curse_of_agony",
        980,
        Dot {
            dot: 3.5,
            cooldown: 2.5,
            range: 20.0,
            count: 4,
            duration: 12.0
        }
    ),
    ability!(
        "immolate",
        348,
        Strike {
            damage: 12.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.4,
            range: 20.0,
            count: 2,
            mark: 348
        }
    ),
    ability!(
        "rain_of_fire",
        5740,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0
        }
    ),
    ability!(
        "hellfire",
        5857,
        Aura {
            damage: 6.0,
            cooldown: 1.0,
            radius: 7.0
        }
    ),
    ability!(
        "death_coil",
        6789,
        Bolt {
            damage: 26.0,
            cooldown: 5.0,
            range: 22.0,
            effect: Effect::Drain(0.5)
        }
    ),
    ability!(
        "searing_pain",
        5676,
        Strike {
            damage: 18.0,
            cooldown: 1.8,
            range: 20.0
        }
    ),
    ability!(
        "shadowburn",
        17877,
        Strike {
            damage: 46.0,
            cooldown: 5.0,
            range: 20.0
        }
    ),
    // --- Hunter ---
    ability!(
        "arcane_shot",
        3044,
        Bolt {
            damage: 13.0,
            cooldown: 1.0,
            range: 28.0
        }
    ),
    ability!(
        "multi_shot",
        2643,
        Bolt {
            damage: 10.0,
            cooldown: 2.0,
            range: 26.0,
            count: 3
        }
    ),
    ability!(
        "serpent_sting",
        1978,
        Bolt {
            damage: 2.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.6,
            range: 26.0,
            count: 2
        }
    ),
    ability!(
        "volley",
        1510,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 24.0,
            radius: 7.0,
            duration: 5.0
        }
    ),
    ability!(
        "explosive_trap",
        13812,
        Ground {
            damage: 22.0,
            dot: 4.0,
            cooldown: 8.0,
            range: 14.0,
            radius: 5.0,
            duration: 0.0
        }
    ),
    ability!(
        "concussive_shot",
        5116,
        Bolt {
            damage: 8.0,
            cooldown: 3.0,
            range: 28.0,
            effect: Effect::Slow(0.5, 4.0),
            mark: 5116
        }
    ),
    ability!(
        "aimed_shot",
        19434,
        Bolt {
            damage: 48.0,
            cooldown: 4.0,
            range: 32.0
        }
    ),
    // --- Shaman ---
    ability!(
        "lightning_bolt",
        403,
        Bolt {
            damage: 15.0,
            cooldown: 1.3,
            range: 24.0
        }
    ),
    ability!(
        "chain_lightning",
        421,
        Chain {
            damage: 18.0,
            cooldown: 3.0,
            range: 20.0,
            radius: 9.0,
            count: 3
        }
    ),
    ability!(
        "earth_shock",
        8042,
        Strike {
            damage: 22.0,
            cooldown: 2.5,
            range: 20.0
        }
    ),
    ability!(
        "flame_shock",
        8050,
        Strike {
            damage: 10.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 3.0,
            range: 20.0,
            count: 2
        }
    ),
    ability!(
        "frost_shock",
        8056,
        Strike {
            damage: 16.0,
            cooldown: 3.0,
            range: 20.0,
            effect: Effect::Slow(0.5, 4.0)
        }
    ),
    ability!(
        "searing_totem",
        3599,
        Totem {
            damage: 9.0,
            cooldown: 14.0,
            range: 20.0,
            duration: 14.0,
            totem: (4589, 3606)
        }
    ),
    ability!(
        "magma_totem",
        8190,
        Totem {
            damage: 8.0,
            cooldown: 16.0,
            radius: 7.0,
            duration: 14.0,
            totem: (4683, 8349)
        }
    ),
    ability!(
        "lightning_shield",
        324,
        Orbit {
            damage: 12.0,
            cooldown: 0.9,
            radius: 4.0
        }
    ),
    // --- Paladin ---
    ability!(
        "hammer_of_wrath",
        24275,
        Bolt {
            damage: 16.0,
            cooldown: 1.5,
            range: 24.0
        }
    ),
    ability!(
        "consecration",
        26573,
        GroundSelf {
            damage: 7.0,
            cooldown: 9.0,
            radius: 7.0,
            duration: 8.0
        }
    ),
    ability!(
        "holy_shock",
        25912,
        Strike {
            damage: 24.0,
            cooldown: 2.5,
            range: 20.0
        }
    ),
    ability!(
        "exorcism",
        879,
        Strike {
            damage: 36.0,
            cooldown: 5.0,
            range: 22.0
        }
    ),
    ability!(
        "holy_wrath",
        2812,
        Nova {
            damage: 14.0,
            cooldown: 5.0,
            radius: 11.0
        }
    ),
    ability!(
        "hammer_of_justice",
        853,
        Strike {
            damage: 8.0,
            cooldown: 6.0,
            range: 12.0,
            count: 2,
            effect: Effect::Stun(3.0),
            mark: 853
        }
    ),
    ability!(
        "divine_shield",
        642,
        Immune {
            cooldown: 28.0,
            duration: 4.0
        }
    ),
    ability!(
        "judgement",
        20271,
        Strike {
            damage: 26.0,
            cooldown: 4.0,
            range: 14.0,
            count: 2
        }
    ),
    // --- Druid ---
    ability!(
        "wrath",
        5176,
        Bolt {
            damage: 13.0,
            cooldown: 1.2,
            range: 24.0
        }
    ),
    ability!(
        "moonfire",
        8921,
        Strike {
            damage: 10.0,
            dot: 3.0,
            duration: 9.0,
            cooldown: 2.0,
            range: 22.0,
            count: 2
        }
    ),
    ability!(
        "starfire",
        2912,
        Strike {
            damage: 40.0,
            cooldown: 4.0,
            range: 24.0
        }
    ),
    ability!(
        "hurricane",
        16914,
        Ground {
            damage: 7.0,
            cooldown: 10.0,
            range: 20.0,
            radius: 8.0,
            duration: 7.0,
            effect: Effect::Slow(0.6, 1.5),
            overhead: true
        }
    ),
    ability!(
        "insect_swarm",
        5570,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 5570
        }
    ),
    ability!(
        "entangling_roots",
        339,
        Dot {
            dot: 2.0,
            cooldown: 6.0,
            range: 16.0,
            count: 4,
            duration: 5.0,
            effect: Effect::Root(4.0),
            mark: 339
        }
    ),
    ability!(
        "thorns",
        467,
        Thorns {
            damage: 8.0,
            cooldown: 15.0,
            duration: 15.0
        }
    ),
    ability!(
        "rejuvenation",
        774,
        Heal {
            damage: 30.0,
            cooldown: 15.0,
            duration: 10.0
        }
    ),
];

pub fn ability(key: &str) -> &'static AbilityDef {
    ABILITIES
        .iter()
        .find(|a| a.key == key)
        .unwrap_or_else(|| panic!("survivors: no ability {key:?}"))
}

pub const MAX_ABILITY_LEVEL: u8 = 8;
pub const MAX_PASSIVE_LEVEL: u8 = 5;
pub const MAX_ABILITIES: usize = 6;
pub const MAX_PASSIVES: usize = 6;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stat {
    Might,
    MaxHp,
    Speed,
    Regen,
    Haste,
    Area,
    Magnet,
    Growth,
    Armor,
    Crit,
}

pub struct PassiveDef {
    pub key: &'static str,
    pub name: &'static str,
    /// Icon source, and a state visual on the hero where the spell has one.
    pub spell: u32,
    pub stat: Stat,
    pub per_level: f32,
}

pub static PASSIVES: &[PassiveDef] = &[
    PassiveDef {
        key: "might",
        name: "Blessing of Might",
        spell: 19740,
        stat: Stat::Might,
        per_level: 0.10,
    },
    PassiveDef {
        key: "fortitude",
        name: "Power Word: Fortitude",
        spell: 1243,
        stat: Stat::MaxHp,
        per_level: 0.20,
    },
    PassiveDef {
        key: "pack",
        name: "Aspect of the Pack",
        spell: 13159,
        stat: Stat::Speed,
        per_level: 0.10,
    },
    PassiveDef {
        key: "renew",
        name: "Renew",
        spell: 139,
        stat: Stat::Regen,
        per_level: 0.5,
    },
    PassiveDef {
        key: "intellect",
        name: "Arcane Intellect",
        spell: 1459,
        stat: Stat::Haste,
        per_level: 0.08,
    },
    PassiveDef {
        key: "amplify",
        name: "Amplify Magic",
        spell: 1008,
        stat: Stat::Area,
        per_level: 0.10,
    },
    PassiveDef {
        key: "detect",
        name: "Detect Lesser Invisibility",
        spell: 132,
        stat: Stat::Magnet,
        per_level: 0.40,
    },
    PassiveDef {
        key: "wisdom",
        name: "Blessing of Wisdom",
        spell: 19742,
        stat: Stat::Growth,
        per_level: 0.08,
    },
    PassiveDef {
        key: "wild",
        name: "Mark of the Wild",
        spell: 1126,
        stat: Stat::Armor,
        per_level: 1.0,
    },
    PassiveDef {
        key: "kings",
        name: "Blessing of Kings",
        spell: 20217,
        stat: Stat::Crit,
        per_level: 0.05,
    },
];

pub static CLASSES: &[ClassDef] = &[
    ClassDef {
        id: 1,
        name: "Warrior",
        color: 0xC79C6E,
        power: 1,
        start: "cleave",
        pool: &[
            "cleave",
            "heroic_strike",
            "whirlwind",
            "thunder_clap",
            "sweeping_strikes",
            "demoralizing_shout",
            "execute",
            "mortal_strike",
            "retaliation",
        ],
        looks: &[
            1300, 1374, 1375, 1504, 1578, 1599, 1707, 1721, 1880, 2096, 2103, 2113, 2196, 2198,
            2614, 2620, 2658, 3053, 3054, 3055, 3280, 3287, 3343, 3399, 3431, 3743, 3793, 3794,
            4242, 4556, 6071, 11037,
        ],
        weapons: &[(0, 7483, 2, 7, 1, 13, 3), (1, 1706, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassDef {
        id: 2,
        name: "Paladin",
        color: 0xF58CBA,
        power: 0,
        start: "hammer_of_wrath",
        pool: &[
            "hammer_of_wrath",
            "consecration",
            "holy_shock",
            "exorcism",
            "holy_wrath",
            "hammer_of_justice",
            "divine_shield",
            "judgement",
        ],
        looks: &[
            1299, 1499, 1622, 3087, 3088, 3089, 3284, 3289, 3346, 3393, 7356,
        ],
        weapons: &[(0, 8011, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassDef {
        id: 3,
        name: "Hunter",
        color: 0xABD473,
        power: 0,
        start: "arcane_shot",
        pool: &[
            "arcane_shot",
            "multi_shot",
            "serpent_sting",
            "volley",
            "explosive_trap",
            "concussive_shot",
            "aimed_shot",
        ],
        looks: &[
            1373, 1703, 1723, 1882, 2066, 2087, 2105, 2112, 2205, 2206, 2251, 3056, 3072, 3073,
            3299, 3309, 3310, 3312, 3395, 3558, 3744, 3810, 3811, 4239, 4241, 4372, 4560, 7538,
            10245,
        ],
        weapons: &[(0, 7480, 2, 6, 1, 17, 2), (2, 6235, 2, 2, 2, 15, 0)],
        sheath: 2,
    },
    ClassDef {
        id: 4,
        name: "Rogue",
        color: 0xFFF569,
        power: 3,
        start: "sinister_strike",
        pool: &[
            "sinister_strike",
            "eviscerate",
            "blade_flurry",
            "rupture",
            "gouge",
            "kidney_shot",
        ],
        looks: &[
            1297, 1327, 1328, 1507, 1580, 1603, 1704, 1725, 1886, 2231, 2243, 2252, 2631, 2639,
            2659, 3100, 3101, 3113, 3351, 3407, 3436, 3749, 4360, 5146, 5528, 13171,
        ],
        weapons: &[(0, 7492, 2, 7, 1, 13, 3), (1, 6443, 2, 15, 1, 13, 3)],
        sheath: 1,
    },
    ClassDef {
        id: 5,
        name: "Priest",
        color: 0xFFFFFF,
        power: 0,
        start: "smite",
        pool: &[
            "smite",
            "shadow_word_pain",
            "holy_fire",
            "mind_blast",
            "holy_nova",
            "psychic_scream",
            "power_word_shield",
        ],
        looks: &[
            1295, 1495, 1579, 1602, 1708, 1733, 1897, 2137, 2138, 2139, 2200, 2201, 2202, 2618,
            2626, 3066, 3085, 3086, 3282, 3283, 3344, 3401, 3429, 4068, 4690, 4711, 10473, 10723,
            11044, 11048, 11053,
        ],
        weapons: &[(0, 1926, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassDef {
        id: 7,
        name: "Shaman",
        color: 0x0070DE,
        power: 0,
        start: "lightning_bolt",
        pool: &[
            "lightning_bolt",
            "chain_lightning",
            "earth_shock",
            "flame_shock",
            "frost_shock",
            "searing_totem",
            "magma_totem",
            "lightning_shield",
        ],
        looks: &[
            1360, 1878, 2082, 2102, 2123, 3746, 3816, 4231, 4552, 10180, 13341,
        ],
        weapons: &[(0, 7477, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
    },
    ClassDef {
        id: 8,
        name: "Mage",
        color: 0x69CCF0,
        power: 0,
        start: "frostbolt",
        pool: &[
            "frostbolt",
            "fireball",
            "arcane_missiles",
            "frost_nova",
            "arcane_explosion",
            "blizzard",
            "flamestrike",
            "fire_blast",
            "cone_of_cold",
            "blast_wave",
            "pyroblast",
        ],
        looks: &[
            1294, 1470, 1484, 1592, 1600, 2134, 2135, 2644, 2657, 2810, 3108, 3109, 3292, 3293,
            4522, 4523, 4524, 4526, 4665, 5001, 6058, 6060, 6072, 7669, 10171, 10214, 10215, 10216,
            10548, 10733, 12849,
        ],
        weapons: &[(0, 1927, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassDef {
        id: 9,
        name: "Warlock",
        color: 0x9482C9,
        power: 0,
        start: "shadow_bolt",
        pool: &[
            "shadow_bolt",
            "corruption",
            "curse_of_agony",
            "immolate",
            "rain_of_fire",
            "hellfire",
            "death_coil",
            "searing_pain",
            "shadowburn",
        ],
        looks: &[
            1324, 1325, 1326, 1469, 1581, 1604, 1884, 1930, 2637, 2646, 2675, 3115, 3116, 3122,
            3271, 3286, 3291, 3345, 3607, 3745, 4567,
        ],
        weapons: &[(0, 5542, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
    ClassDef {
        id: 11,
        name: "Druid",
        color: 0xFF7D0A,
        power: 0,
        start: "wrath",
        pool: &[
            "wrath",
            "moonfire",
            "starfire",
            "hurricane",
            "insect_swarm",
            "entangling_roots",
            "thorns",
            "rejuvenation",
        ],
        looks: &[
            1706, 1732, 2106, 2115, 2121, 2250, 2255, 2261, 3300, 3301, 3302, 3819, 3820, 7357,
            10738, 12053,
        ],
        weapons: &[(0, 22391, 2, 10, 2, 17, 2)],
        sheath: 1,
    },
];

#[derive(Clone, Copy)]
pub struct EnemyDef {
    pub name: &'static str,
    pub display: u32,
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub scale: f32,
    pub xp: u32,
    pub level: u32,
    /// A great beast, which takes more room in the crowd.
    pub big: bool,
}

const fn en(
    name: &'static str,
    display: u32,
    hp: f32,
    speed: f32,
    damage: f32,
    xp: u32,
    level: u32,
) -> EnemyDef {
    EnemyDef {
        name,
        display,
        hp,
        speed,
        damage,
        scale: 1.0,
        xp,
        level,
        big: false,
    }
}

/// The Barrens, roughly in the order a character meets them.
pub static ENEMIES: &[EnemyDef] = &[
    en("Fleeting Plainstrider", 1284, 8.0, 3.6, 2.0, 1, 10),
    en("Zhevra Runner", 6087, 10.0, 4.4, 2.0, 1, 11),
    en("Razormane Quilboar", 1218, 16.0, 3.4, 3.0, 2, 12),
    en("Razormane Hunter", 6094, 18.0, 3.6, 4.0, 2, 12),
    en("Savannah Huntress", 1056, 22.0, 4.8, 8.0, 2, 13),
    en("Sunscale Lashtail", 1744, 26.0, 4.6, 8.0, 3, 13),
    en("Kolkar Wrangler", 9442, 36.0, 4.0, 10.0, 3, 14),
    en("Kolkar Stormer", 9443, 40.0, 4.0, 11.0, 3, 14),
    en("Witchwing Harpy", 3218, 34.0, 5.0, 10.0, 3, 15),
    en("Hecklefang Hyena", 2710, 44.0, 5.2, 12.0, 4, 15),
    en("Razormane Defender", 1253, 70.0, 3.3, 14.0, 5, 16),
    en("Razormane Mystic", 4643, 55.0, 3.8, 12.0, 4, 16),
    en("Sunscale Screecher", 1747, 60.0, 5.0, 13.0, 4, 17),
    en("Kolkar Bloodcharger", 9447, 85.0, 4.6, 16.0, 5, 18),
    en("Savannah Prowler", 1973, 75.0, 5.4, 15.0, 5, 18),
    en("Thunderhawk Hatchling", 1742, 90.0, 4.2, 16.0, 6, 19),
    en("Witchwing Slayer", 2163, 95.0, 5.0, 18.0, 6, 20),
    en("Bristleback Geomancer", 6091, 120.0, 3.6, 20.0, 7, 21),
    huge(en("Stormsnout", 1537, 150.0, 4.0, 22.0, 8, 22)),
    huge(en("Barrens Kodo", 1453, 220.0, 3.2, 28.0, 12, 23)),
    en("Sunscale Scytheclaw", 4442, 160.0, 5.6, 24.0, 8, 24),
    huge(en("Thunderhead", 1538, 180.0, 4.2, 26.0, 10, 25)),
    en("Silithid Swarmer", 2731, 90.0, 6.0, 18.0, 5, 25),
    huge(en("Greater Thunderhawk", 1974, 240.0, 4.4, 30.0, 12, 26)),
    en("Hecklefang Stalker", 2712, 200.0, 6.0, 30.0, 10, 27),
    en("Kolkar Destroyer", 9446, 280.0, 4.6, 34.0, 14, 28),
    en("Kolkar Mauler", 4874, 300.0, 4.6, 36.0, 14, 29),
    huge(en("Greater Barrens Kodo", 1232, 420.0, 3.4, 42.0, 24, 30)),
    huge(en("Elder Thunder Lizard", 2764, 380.0, 4.4, 44.0, 22, 31)),
    huge(en("Silithid Protector", 1307, 360.0, 5.0, 40.0, 20, 32)),
];

/// The great beasts at three quarters size, so a crowd of them still leaves the field readable.
const fn huge(def: EnemyDef) -> EnemyDef {
    EnemyDef {
        scale: 0.75,
        big: true,
        ..def
    }
}

pub static BOSSES: &[EnemyDef] = &[
    EnemyDef {
        scale: 1.6,
        ..en("Echeyakee", 1934, 700.0, 5.2, 20.0, 60, 16)
    },
    EnemyDef {
        scale: 1.6,
        ..en("Takk the Leaper", 1337, 1800.0, 5.4, 32.0, 120, 19)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Lakota'mani", 1241, 4200.0, 4.0, 45.0, 200, 22)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Humar the Pridelord", 4424, 7000.0, 5.6, 55.0, 300, 26)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Agathelos the Raging", 2450, 12000.0, 5.0, 70.0, 400, 33)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Hezrul Bloodmark", 9448, 20000.0, 4.8, 90.0, 500, 33)
    },
];

/// Survive this long to win.
pub const VICTORY_SECS: f32 = 900.0;
/// A boss every two and a quarter minutes, the last of six at 13:30, a fight before the bell.
pub const BOSS_EVERY_SECS: f32 = 135.0;
/// Experience per gem: every level of the curve comes at the run's pace.
pub const XP_RATE: f32 = 2.0;

/// How far through the run, 0 at the start and 1 at [`VICTORY_SECS`].
pub fn progress(elapsed: f32) -> f32 {
    (elapsed / VICTORY_SECS).max(0.0)
}

/// The pool of roster indices at a point in the run, from a roster `tiers` long.
pub fn wave_pool(elapsed: f32, tiers: usize) -> [usize; 3] {
    // The roster spread over thirty steps of the run, the last three from step 28: on The
    // Barrens' thirty tiers, one newer every step.
    let step = (progress(elapsed) * 30.0) as usize;
    let span = tiers.saturating_sub(3);
    let lo = (step.saturating_sub(1) * span / 27).min(span);
    [lo, lo + 1, lo + 2]
}

/// How many enemies the spawner keeps alive at a point in the run.
pub fn target_alive(elapsed: f32) -> usize {
    ((14.0 + progress(elapsed) * 225.0) as usize).min(220)
}

/// Enemy health grows on top of the tiers, so a late kill takes several spells.
pub fn hp_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.75
}

/// Enemy damage grows the same way.
pub fn damage_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.9
}

/// The experience to go from `level` to the next: Vampire Survivors' curve, cheap early.
pub fn xp_to_next(level: u32) -> f32 {
    let l = level as f32;
    if level < 20 {
        5.0 + (l - 1.0) * 10.0
    } else if level < 40 {
        195.0 + (l - 20.0) * 13.0
    } else {
        455.0 + (l - 40.0) * 16.0
    }
}

/// A creature on a battleground's roster: who it is, its pace and its size. Its health, damage
/// and experience come from its place on the roster, read off The Barrens' tuned ladder.
pub struct Foe {
    name: &'static str,
    display: u32,
    level: u32,
    speed: f32,
    /// The drawn scale, chosen against the model's Stand height so the field reads as The
    /// Barrens does: about 2-3.5 yd for the horde, 4-5 for the great beasts, 5-9 for a boss.
    scale: f32,
    big: bool,
}

const fn foe(name: &'static str, display: u32, level: u32, speed: f32, scale: f32) -> Foe {
    Foe {
        name,
        display,
        level,
        speed,
        scale,
        big: false,
    }
}

const fn giant(name: &'static str, display: u32, level: u32, speed: f32, scale: f32) -> Foe {
    Foe {
        big: true,
        ..foe(name, display, level, speed, scale)
    }
}

/// Where a run is fought.
pub struct MapDef {
    pub name: &'static str,
    /// The zone or continent under the name on the menu.
    pub zone: &'static str,
    /// The `Map.dbc` row.
    pub map: u32,
    /// Where the hero starts (WoW coordinates).
    pub center: [f32; 3],
    /// How far from the centre the hero may walk.
    pub leash: f32,
    /// The results screen's line on a win.
    pub victory: &'static str,
    /// Weakest first; empty for The Barrens, which keeps its hand-tuned [`ENEMIES`].
    foes: &'static [Foe],
    /// One every [`BOSS_EVERY_SECS`], in order; empty for The Barrens' [`BOSSES`].
    bosses: &'static [Foe],
}

pub static MAPS: &[MapDef] = &[
    MapDef {
        name: "The Barrens",
        zone: "Kalimdor",
        map: 1,
        // The plains south of the Crossroads.
        center: [-950.0, -2700.0, 92.0],
        leash: 150.0,
        victory: "The Barrens are yours.",
        foes: &[],
        bosses: &[],
    },
    MapDef {
        name: "The Dark Portal",
        zone: "Blasted Lands",
        map: 0,
        // The basin before the Portal, ringed by cliffs.
        center: [-11840.0, -3197.0, -30.0],
        leash: 75.0,
        victory: "The Dark Portal stands silent.",
        foes: &[
            foe("Starving Snickerfang", 8050, 45, 4.6, 1.0),
            foe("Snickerfang Hyena", 2714, 49, 5.0, 1.15),
            foe("Black Slayer", 10824, 47, 5.0, 0.6),
            foe("Redstone Crystalhide", 798, 51, 3.6, 1.3),
            foe("Shadowsworn Cultist", 7838, 51, 3.8, 1.3),
            foe("Felbeast", 7949, 50, 4.8, 0.85),
            foe("Dreadmaul Mauler", 14401, 53, 3.6, 1.1),
            foe("Felhound", 1913, 54, 5.2, 1.0),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9),
            foe("Manahound", 6173, 60, 5.6, 1.15),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1),
        ],
        bosses: &[
            foe("Akubar the Seer", 10920, 54, 4.4, 2.6),
            foe("Lord Azrethoc", 4426, 55, 4.6, 1.35),
            foe("Lord Hel'nurath", 14556, 62, 4.8, 1.65),
            giant("Teremus the Devourer", 6378, 60, 5.0, 0.55),
            foe("Lord Banehollow", 8611, 62, 4.8, 1.95),
            giant("Lord Kazzak", 12449, 63, 4.6, 0.75),
        ],
    },
    MapDef {
        name: "Gates of Ahn'Qiraj",
        zone: "Silithus",
        map: 1,
        // The sands before the Scarab Wall.
        center: [-8180.0, 1536.0, 5.0],
        leash: 80.0,
        victory: "The Qiraji retreat behind the Scarab Wall.",
        foes: &[
            foe("Sand Skitterer", 8014, 56, 5.0, 1.0),
            foe("Stonelash Scorpid", 15383, 55, 4.6, 1.6),
            foe("Dredge Striker", 15386, 56, 4.4, 1.0),
            foe("Twilight Avenger", 11811, 58, 4.0, 1.1),
            foe("Hive'Ashi Drone", 2303, 58, 5.4, 0.55),
            foe("Stonelash Flayer", 15385, 57, 4.8, 1.75),
            foe("Hive'Ashi Defender", 12153, 59, 4.2, 1.3),
            foe("Dust Stormer", 8715, 57, 4.6, 1.05),
            foe("Twilight Stonecaller", 11815, 59, 3.8, 1.1),
            foe("Hive'Zora Wasp", 11088, 59, 5.6, 0.6),
            foe("Hive'Zora Reaver", 11085, 60, 4.8, 1.0),
            foe("Hive'Regal Ambusher", 11106, 60, 5.2, 1.05),
            foe("Vekniss Wasp", 15335, 60, 5.6, 0.65),
            foe("Qiraji Gladiator", 15741, 61, 4.4, 0.75),
            foe("Vekniss Warrior", 15334, 61, 4.6, 1.15),
            giant("Obsidian Destroyer", 15343, 61, 3.8, 1.2),
            giant("Qiraji Champion", 15340, 63, 4.6, 0.95),
            giant("Anubisath Sentinel", 15347, 61, 4.0, 0.62),
        ],
        bosses: &[
            giant("Kurinnaxx", 15742, 62, 5.0, 1.15),
            foe("General Rajaxx", 15376, 63, 4.8, 1.15),
            foe("Moam", 15392, 63, 4.2, 1.6),
            giant("Buru the Gorger", 15654, 63, 4.4, 0.85),
            giant("Princess Huhuran", 15739, 63, 5.0, 0.55),
            foe("Ossirian the Unscarred", 15432, 63, 4.6, 0.82),
        ],
    },
    MapDef {
        name: "Fire Plume Ridge",
        zone: "Un'Goro Crater",
        map: 1,
        // The summit plateau, its lava lake on the west rim; the crater floor's canopy would
        // hide the fight from above.
        center: [-7136.0, -1309.0, -184.0],
        leash: 32.0,
        victory: "Even King Mosh has fallen.",
        foes: &[
            foe("Bloodpetal Lasher", 11634, 49, 3.6, 0.9),
            foe("Young Diemetradon", 8510, 48, 4.4, 0.55),
            foe("Ravasaur", 5242, 50, 5.4, 0.75),
            foe("Muculent Ooze", 11140, 50, 3.2, 1.4),
            foe("Ravasaur Hunter", 5292, 51, 5.6, 0.85),
            foe("Tar Beast", 1549, 51, 3.4, 1.0),
            foe("Bloodpetal Flayer", 11635, 52, 3.8, 1.05),
            foe("Primal Ooze", 4754, 52, 3.4, 1.6),
            foe("Gorishi Wasp", 11090, 53, 5.4, 0.55),
            foe("Pterrordax", 8411, 53, 5.2, 0.5),
            foe("Diemetradon", 8511, 52, 4.6, 0.7),
            foe("Glutinous Ooze", 1146, 54, 3.6, 1.8),
            foe("Tar Lurker", 9010, 54, 3.6, 1.15),
            foe("Scorching Elemental", 1070, 54, 4.2, 1.05),
            foe("Frenzied Pterrordax", 8412, 55, 5.6, 0.55),
            foe("Elder Diemetradon", 8512, 55, 4.6, 0.8),
            giant("Stegodon", 5241, 52, 3.8, 0.72),
            giant("Stone Guardian", 8395, 52, 3.6, 1.45),
            giant("Plated Stegodon", 5287, 54, 3.8, 0.78),
            giant("Devilsaur", 5239, 56, 5.0, 0.72),
        ],
        bosses: &[
            foe("Ravasaur Matriarch", 11319, 56, 5.6, 1.55),
            foe("Clutchmother Zavas", 11084, 56, 5.2, 1.4),
            foe("Uhk'loc", 8129, 59, 4.8, 2.4),
            foe("Gruff", 10932, 59, 4.4, 2.2),
            foe("Blazerunner", 1204, 57, 4.6, 1.8),
            foe("Baron Charr", 14517, 58, 4.6, 1.95),
            giant("Tyrant Devilsaur", 5240, 60, 5.0, 1.0),
            foe("U'cha", 8844, 60, 4.8, 2.6),
            giant("King Mosh", 5305, 61, 5.0, 1.2),
        ],
    },
    MapDef {
        name: "Gurubashi Arena",
        zone: "Stranglethorn Vale",
        map: 0,
        // The arena floor; the stands ring it.
        center: [-13205.0, 272.0, 22.0],
        leash: 30.0,
        victory: "The arena roars your name.",
        // Verified map-0 Stranglethorn identities; combat tuning and arena arrivals are authored.
        foes: &[
            foe("Shadowmaw Panther", 633, 37, 5.6, 1.4),
            foe("Cold Eye Basilisk", 8797, 39, 3.8, 1.0),
            foe("Jungle Stalker", 11317, 40, 5.2, 0.8),
            foe("Elder Mistvale Gorilla", 838, 40, 4.0, 1.1),
            foe("Zanzil Zombie", 1065, 43, 3.4, 1.1),
            foe("Thrashtail Basilisk", 8802, 41, 4.0, 1.15),
            foe("Bloodsail Sea Dog", 796, 43, 4.0, 1.05),
            foe("Skullsplitter Berserker", 4623, 43, 4.2, 1.1),
            foe("Bloodsail Raider", 793, 39, 4.2, 1.1),
            foe("Skullsplitter Hunter", 4628, 41, 4.2, 1.1),
            foe("Skullsplitter Mystic", 4616, 39, 4.2, 1.1),
            foe("Bloodsail Warlock", 4461, 41, 4.2, 1.1),
            foe("Zanzil Hunter", 828, 43, 4.2, 1.1),
            foe("Jungle Thunderer", 845, 37, 4.2, 1.1),
            foe("Stranglethorn Tigress", 614, 37, 4.2, 1.1),
        ],
        bosses: &[
            foe("Kurmokk", 3188, 42, 4.4, 2.0),
            foe("Verifonix", 7232, 42, 4.4, 2.0),
            foe("Scale Belly", 12342, 45, 4.4, 2.0),
            foe("Lord Sakrasis", 4910, 45, 4.4, 2.0),
            foe("Rippa", 14528, 44, 4.4, 2.0),
            foe("Tethis", 8472, 43, 4.4, 2.0),
            foe("Bhag'thera", 613, 40, 4.4, 2.0),
            giant("King Bangalash", 616, 43, 5.8, 3.0),
            foe("King Mukla", 792, 51, 4.4, 2.0),
        ],
    },
    MapDef {
        name: "Elwynn Forest",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Elwynn Forest's own creatures: no building or big prop near.
        center: [-9548.5, 115.0, 60.0],
        leash: 150.0,
        victory: "Elwynn Forest is safe once more.",
        foes: &[
            foe("Young Wolf", 447, 1, 5.4, 1.0),
            foe("Timber Wolf", 604, 2, 5.4, 1.0),
            foe("Defias Thug", 5035, 4, 4.2, 1.15),
            foe("Mangy Wolf", 903, 6, 5.4, 1.0),
            foe("Stonetusk Boar", 503, 6, 4.6, 1.0),
            foe("Kobold Tunneler", 139, 6, 4.2, 1.0),
            foe("Defias Cutpurse", 2361, 6, 4.2, 1.15),
            foe("Forest Spider", 382, 6, 4.2, 1.0),
            foe("Murloc", 617, 7, 4.2, 1.0),
            foe("Kobold Miner", 373, 7, 4.2, 1.0),
            foe("Murloc Streamrunner", 527, 7, 4.2, 1.0),
            foe("Rockhide Boar", 389, 8, 4.6, 1.0),
            foe("Gray Forest Wolf", 380, 8, 5.4, 1.0),
            foe("Defias Bandit", 2357, 9, 4.2, 1.15),
            giant("Young Forest Bear", 1006, 9, 4.6, 1.0),
            foe("Defias Rogue Wizard", 2359, 10, 3.8, 1.1),
        ],
        bosses: &[
            foe("Mother Fang", 2541, 10, 4.6, 1.9),
            foe("Narg the Taskmaster", 774, 10, 4.2, 1.9),
            foe("Hogger", 384, 11, 4.2, 1.9),
            foe("Thuros Lightfingers", 3341, 11, 4.2, 2.18),
            foe("Fedfennel", 175, 12, 4.2, 1.9),
            foe("Gruff Swiftbite", 175, 12, 4.2, 1.9),
        ],
    },
    MapDef {
        name: "Durotar",
        zone: "Kalimdor",
        map: 1,
        // Open, level ground among Durotar's own creatures: no building or big prop near.
        center: [587.5, -4311.4, 19.0],
        leash: 150.0,
        victory: "Durotar is quiet again.",
        foes: &[
            foe("Vile Familiar", 1185, 4, 4.2, 1.0),
            foe("Felstalker", 850, 4, 4.2, 1.0),
            foe("Clattering Scorpid", 2486, 6, 4.6, 1.5),
            foe("Pygmy Surf Crawler", 1307, 6, 4.6, 1.0),
            foe("Dire Mottled Boar", 381, 7, 4.6, 1.0),
            foe("Razormane Quilboar", 1218, 7, 4.6, 1.0),
            foe("Armored Scorpid", 2487, 8, 4.6, 1.5),
            foe("Bloodtalon Taillasher", 1960, 8, 5.4, 0.85),
            foe("Razormane Scout", 1252, 8, 4.6, 1.0),
            foe("Dustwind Harpy", 3012, 8, 4.2, 1.0),
            foe("Dustwind Pillager", 6814, 8, 4.2, 1.0),
            foe("Razormane Dustrunner", 6088, 9, 4.6, 1.0),
            foe("Elder Mottled Boar", 193, 9, 4.6, 1.0),
            foe("Razormane Battleguard", 1964, 10, 4.6, 1.0),
            foe("Burning Blade Neophyte", 4186, 10, 3.8, 1.0),
            foe("Burning Blade Cultist", 4194, 11, 3.8, 1.0),
        ],
        bosses: &[
            foe("Geolord Mottle", 6113, 9, 4.6, 1.9),
            foe("Razormane Dustrunner", 6088, 9, 4.6, 1.9),
            foe("Razormane Battleguard", 1964, 10, 4.6, 1.9),
            foe("Burning Blade Neophyte", 4186, 10, 3.8, 1.9),
            foe("Death Flayer", 2491, 11, 4.6, 2.85),
            foe("Burning Blade Cultist", 4194, 11, 3.8, 1.9),
        ],
    },
    MapDef {
        name: "Teldrassil",
        zone: "Kalimdor",
        map: 1,
        // Open, level ground among Teldrassil's own creatures: no building or big prop near.
        center: [9737.5, 649.8, 1298.5],
        leash: 150.0,
        victory: "Teldrassil blooms again.",
        foes: &[
            foe("Young Nightsaber", 11454, 1, 5.4, 1.5),
            foe("Young Thistle Boar", 8869, 2, 4.6, 1.0),
            foe("Grell", 6806, 3, 4.2, 1.0),
            foe("Grellkin", 3023, 4, 4.2, 1.0),
            foe("Nightsaber", 6805, 6, 5.4, 1.5),
            foe("Strigid Owl", 10832, 6, 5.2, 1.0),
            foe("Webwood Lurker", 760, 6, 4.2, 1.0),
            foe("Timberling", 3033, 6, 4.2, 1.15),
            foe("Gnarlpine Gardener", 6799, 6, 3.8, 1.0),
            foe("Gnarlpine Warrior", 6800, 7, 4.2, 1.0),
            foe("Gnarlpine Ambusher", 3024, 7, 4.2, 1.0),
            foe("Vicious Grell", 6811, 7, 4.2, 1.0),
            foe("Gnarlpine Defender", 6801, 8, 4.2, 1.0),
            foe("Gnarlpine Shaman", 897, 8, 3.8, 1.0),
            foe("Timberling Bark Ripper", 6817, 8, 4.2, 1.15),
            foe("Gnarlpine Augur", 3024, 9, 3.8, 1.0),
        ],
        bosses: &[
            foe("Threggil", 904, 6, 4.2, 1.9),
            foe("Uruson", 6818, 7, 4.2, 1.9),
            foe("Gnarlpine Defender", 6801, 8, 4.2, 1.9),
            foe("Duskstalker", 11453, 9, 5.4, 2.85),
            foe("Oakenscowl", 2567, 9, 4.2, 2.18),
            foe("Gnarlpine Augur", 3024, 9, 3.8, 1.9),
        ],
    },
    MapDef {
        name: "Dun Morogh",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Dun Morogh's own creatures: no building or big prop near.
        center: [-5400.8, -739.9, 396.7],
        leash: 150.0,
        victory: "Dun Morogh thaws in your wake.",
        foes: &[
            foe("Crag Boar", 607, 6, 4.6, 1.0),
            giant("Young Black Bear", 8843, 6, 4.6, 1.0),
            foe("Juvenile Snow Leopard", 748, 6, 5.4, 1.5),
            foe("Young Wendigo", 929, 6, 4.2, 1.0),
            foe("Wendigo", 950, 7, 4.2, 1.0),
            foe("Large Crag Boar", 381, 7, 4.6, 1.0),
            foe("Snow Tracker Wolf", 604, 7, 5.4, 1.0),
            foe("Snow Leopard", 954, 8, 5.4, 1.5),
            foe("Elder Crag Boar", 744, 8, 4.6, 1.0),
            foe("Winter Wolf", 785, 8, 5.4, 1.0),
            giant("Ice Claw Bear", 8840, 8, 4.6, 1.0),
            foe("Frostmane Seer", 5610, 9, 3.8, 1.2),
            foe("Frostmane Snowstrider", 5607, 9, 4.2, 1.2),
        ],
        bosses: &[
            foe("Edan the Howler", 931, 9, 4.2, 1.9),
            foe("Timber", 11422, 10, 5.4, 1.9),
            foe("Great Father Arctikus", 5625, 11, 4.2, 2.28),
            foe("Old Icebeard", 1050, 11, 4.2, 1.9),
            foe("Vagash", 1050, 11, 4.2, 1.9),
            giant("Bjarn", 913, 12, 4.6, 1.9),
        ],
    },
    MapDef {
        name: "Mulgore",
        zone: "Kalimdor",
        map: 1,
        // Open, level ground among Mulgore's own creatures: no building or big prop near.
        center: [-1809.9, -220.6, -8.4],
        leash: 150.0,
        victory: "Mulgore's herds roam free.",
        foes: &[
            foe("Plainstrider", 1219, 2, 5.4, 1.0),
            foe("Battleboar", 8869, 4, 4.6, 1.0),
            foe("Prairie Wolf", 1100, 6, 5.4, 1.0),
            foe("Palemane Tanner", 550, 6, 4.2, 1.0),
            foe("Venture Co. Hireling", 7243, 6, 4.2, 1.3),
            foe("Adult Plainstrider", 1220, 7, 5.4, 1.0),
            foe("Wiry Swoop", 1228, 7, 5.2, 0.8),
            foe("Venture Co. Laborer", 7244, 7, 4.2, 1.3),
            foe("Palemane Skinner", 1216, 7, 4.2, 1.0),
            foe("Prairie Stalker", 643, 8, 5.4, 1.0),
            foe("Flatland Cougar", 1059, 8, 5.4, 1.5),
            foe("Palemane Poacher", 1217, 8, 4.2, 1.0),
            foe("Venture Co. Taskmaster", 7246, 8, 4.2, 1.3),
            foe("Swoop", 1229, 9, 5.2, 0.8),
            foe("Elder Plainstrider", 1221, 9, 5.4, 1.0),
        ],
        bosses: &[
            foe("Prairie Stalker", 643, 8, 5.4, 1.9),
            foe("Mazzranache", 1961, 9, 5.4, 1.9),
            foe("Snagglespear", 275, 9, 4.2, 1.9),
            foe("Swoop", 1229, 9, 5.2, 1.52),
            foe("Elder Plainstrider", 1221, 9, 5.4, 1.9),
            foe("Ghost Howl", 720, 12, 5.4, 1.9),
        ],
    },
    MapDef {
        name: "Tirisfal Glades",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Tirisfal Glades's own creatures: no building or big prop near.
        center: [1993.0, -153.3, 35.4],
        leash: 150.0,
        victory: "Tirisfal is at peace.",
        foes: &[
            foe("Decrepit Darkhound", 9021, 6, 4.2, 1.4),
            foe("Rotting Dead", 1200, 6, 3.6, 1.0),
            foe("Greater Duskbat", 4734, 7, 5.2, 0.8),
            foe("Ravaged Corpse", 1201, 7, 3.6, 1.0),
            foe("Rot Hide Graverobber", 10853, 7, 4.2, 1.0),
            foe("Rot Hide Gnoll", 10852, 7, 4.2, 1.0),
            foe("Rattlecage Soldier", 200, 7, 4.2, 1.2),
            foe("Cursed Darkhound", 9020, 8, 4.2, 1.4),
            foe("Rot Hide Mongrel", 847, 8, 4.2, 1.0),
            foe("Hungering Dead", 1202, 8, 3.6, 1.0),
            foe("Scarlet Missionary", 2468, 8, 3.8, 1.15),
            foe("Darkeye Bonecaster", 11398, 8, 3.8, 1.0),
            foe("Vampiric Duskbat", 8808, 9, 5.2, 0.8),
            foe("Scarlet Zealot", 2483, 9, 4.2, 1.1),
            foe("Shambling Horror", 1197, 9, 3.6, 1.0),
        ],
        bosses: &[
            foe("Farmer Solliden", 3535, 8, 4.2, 2.18),
            foe("Tormented Spirit", 9534, 9, 4.2, 1.52),
            foe("Bayne", 7892, 10, 4.2, 2.66),
            foe("Ressan the Needler", 9750, 11, 5.2, 1.52),
            foe("Fellicent's Shade", 5430, 12, 4.2, 1.9),
            foe("Gorefang", 11413, 13, 5.4, 1.9),
        ],
    },
    MapDef {
        name: "Westfall",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Westfall's own creatures: no building or big prop near.
        center: [-10252.2, 1483.5, 41.6],
        leash: 150.0,
        victory: "Westfall's fields are safe.",
        foes: &[
            foe("Young Fleshripper", 410, 11, 5.2, 0.8),
            foe("Coyote", 643, 11, 5.4, 1.0),
            foe("Defias Smuggler", 4418, 12, 4.2, 1.15),
            giant("Harvest Golem", 367, 12, 3.6, 1.0),
            foe("Young Goretusk", 8871, 13, 4.6, 1.0),
            foe("Defias Trapper", 2331, 13, 4.2, 1.1),
            foe("Fleshripper", 2305, 14, 5.2, 0.8),
            foe("Goretusk", 3027, 15, 4.6, 1.0),
            giant("Harvest Watcher", 378, 15, 3.6, 1.0),
            foe("Riverpaw Brute", 384, 16, 4.2, 1.0),
            foe("Riverpaw Bandit", 383, 17, 4.2, 1.0),
            foe("Great Goretusk", 3035, 17, 4.6, 1.0),
            foe("Riverpaw Taskmaster", 376, 18, 4.2, 1.0),
            foe("Defias Highwayman", 2342, 18, 4.2, 1.15),
            foe("Pygmy Venom Web Spider", 958, 19, 4.6, 1.0),
            foe("Riverpaw Mystic", 502, 19, 3.8, 1.0),
        ],
        bosses: &[
            foe("Brainwashed Noble", 3267, 18, 4.2, 2.18),
            foe("Marisa du'Paige", 2355, 18, 4.2, 2.09),
            foe("Sergeant Brashclaw", 383, 18, 4.2, 1.9),
            foe("Klaven Mortwake", 5805, 23, 4.2, 2.18),
            foe("Lupos", 11412, 23, 5.4, 1.9),
            foe("Vultros", 507, 26, 5.2, 1.52),
        ],
    },
    MapDef {
        name: "Duskwood",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Duskwood's own creatures: no building or big prop near.
        center: [-10051.8, -792.5, 36.5],
        leash: 150.0,
        victory: "Duskwood's curse lifts.",
        foes: &[
            foe("Pygmy Venom Web Spider", 958, 19, 4.6, 1.0),
            foe("Venom Web Spider", 955, 20, 4.6, 1.0),
            foe("Skeletal Warrior", 200, 22, 4.2, 1.2),
            foe("Green Recluse", 2541, 22, 4.6, 1.0),
            foe("Skeletal Mage", 9783, 23, 4.2, 1.0),
            foe("Young Black Ravager", 246, 24, 5.4, 1.0),
            foe("Skeletal Horror", 9786, 24, 3.6, 1.0),
            foe("Black Widow Hatchling", 368, 25, 4.6, 1.0),
            foe("Black Ravager", 741, 25, 5.4, 1.0),
            foe("Black Ravager Mastiff", 9562, 26, 5.4, 1.0),
            foe("Nightbane Worgen", 657, 27, 5.4, 1.0),
            foe("Nightbane Shadow Weaver", 203, 28, 5.4, 1.0),
            foe("Nightbane Dark Runner", 736, 29, 5.4, 1.0),
            foe("Nightbane Vile Fang", 522, 30, 5.4, 1.0),
            foe("Fetid Corpse", 559, 30, 3.6, 1.3),
            foe("Nightbane Tainted One", 1098, 31, 5.4, 1.0),
        ],
        bosses: &[
            foe("Lupos", 11412, 23, 5.4, 1.9),
            foe("Naraxis", 963, 27, 4.6, 1.9),
            foe("Nightbane Vile Fang", 522, 30, 5.4, 1.9),
            foe("Nightbane Tainted One", 1098, 31, 5.4, 1.9),
            foe("Fenros", 11179, 32, 5.4, 1.9),
            foe("Nefaru", 11181, 34, 5.4, 1.9),
        ],
    },
    MapDef {
        name: "Redridge Mountains",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open, level ground among Redridge Mountains's own creatures: no building or big prop near.
        center: [-9199.8, -2767.5, 91.0],
        leash: 150.0,
        victory: "Redridge is safe once more.",
        foes: &[
            foe("Redridge Thrasher", 489, 15, 4.2, 1.0),
            foe("Redridge Mongrel", 487, 16, 4.2, 1.0),
            foe("Tarantula", 366, 16, 4.2, 1.0),
            foe("Great Goretusk", 3035, 17, 4.6, 1.0),
            foe("Redridge Poacher", 551, 17, 4.2, 1.0),
            foe("Murloc Shorestriker", 1305, 17, 4.2, 1.0),
            foe("Black Dragon Whelp", 387, 18, 4.2, 1.0),
            foe("Redridge Brute", 500, 18, 4.2, 1.0),
            foe("Redridge Mystic", 10789, 19, 3.8, 1.0),
            foe("Dire Condor", 490, 19, 5.2, 0.8),
            foe("Redridge Basher", 609, 20, 4.2, 1.0),
            foe("Greater Tarantula", 520, 20, 4.2, 1.0),
            foe("Redridge Drudger", 551, 21, 4.2, 1.0),
            foe("Blackrock Summoner", 6030, 23, 3.8, 1.0),
            foe("Blackrock Tracker", 6040, 24, 4.2, 1.0),
            foe("Blackrock Champion", 6044, 25, 4.2, 1.0),
        ],
        bosses: &[
            foe("Blackrock Hunter", 6031, 24, 4.2, 1.9),
            foe("Singe", 497, 24, 4.2, 1.9),
            foe("Tharil'zun", 6043, 24, 4.2, 1.9),
            foe("Blackrock Gladiator", 6042, 25, 4.2, 1.9),
            foe("Rohh the Silent", 10792, 26, 4.2, 1.9),
            foe("Kazon", 6041, 27, 4.2, 1.9),
        ],
    },
    MapDef {
        name: "Mount Hyjal",
        zone: "Kalimdor",
        map: 1,
        // A plateau below the World Tree.
        center: [5478.0, -3729.0, 1593.0],
        leash: 70.0,
        victory: "Nordrassil endures.",
        foes: &[
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3),
            foe("Jadefire Rogue", 8575, 52, 5.2, 1.0),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85),
            foe("Death Cultist", 10391, 56, 3.8, 1.1),
            foe("Felbeast", 7949, 50, 4.8, 0.85),
            foe("Torn Screamer", 984, 57, 5.0, 0.7),
            foe("Necromancer", 10427, 57, 3.6, 1.15),
            foe("Jadefire Felsworn", 11334, 54, 4.8, 1.1),
            foe("Felhound", 1913, 54, 5.2, 1.0),
            foe("Nerubian Overseer", 14698, 58, 4.4, 0.95),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8),
            foe("Hederine Manastalker", 6172, 58, 5.4, 1.15),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1),
        ],
        bosses: &[
            foe("Prince Xavalis", 11335, 57, 5.0, 1.85),
            foe("Lady Hederine", 10925, 61, 4.8, 2.7),
            giant("Taerar", 15363, 63, 4.8, 0.5),
            giant("Lethon", 15365, 63, 4.8, 0.5),
            giant("Emeriss", 15366, 63, 4.8, 0.53),
            giant("Ysondre", 15364, 63, 4.8, 0.58),
        ],
    },
    MapDef {
        name: "Naxxramas",
        zone: "Naxxramas",
        map: 533,
        // Stratholme Raid WMO group 19. Floor probe: all 2821 one-yard samples
        // within the 30-yard disk have a nearby walking surface. This is not
        // traversal evidence: the campaign content gate remains closed.
        center: [3271.151, -3328.7507, 292.59506],
        leash: 30.0,
        victory: "Kel'Thuzad's necropolis falls dark.",
        foes: &[
            foe("Scourge Soldier", 5231, 55, 4.4, 1.15),
            foe("Plagued Swine", 6121, 53, 5.0, 1.0),
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3),
            foe("Cursed Mage", 11397, 55, 3.6, 0.95),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85),
            foe("Scourge Warder", 612, 56, 4.4, 1.2),
            foe("Death Cultist", 10391, 56, 3.8, 1.1),
            foe("Frenzied Plaguehound", 9022, 57, 5.6, 1.4),
            foe("Torn Screamer", 984, 57, 5.0, 0.7),
            foe("Necromancer", 10427, 57, 3.6, 1.15),
            foe("Carrion Devourer", 7900, 57, 4.0, 1.25),
            foe("Crypt Walker", 11178, 58, 4.6, 0.85),
            foe("Putrid Shrieker", 7855, 58, 5.4, 1.0),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8),
            foe("Ziggurat Protector", 10627, 59, 4.4, 1.6),
            foe("Deathknight", 16508, 60, 4.6, 1.75),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25),
            giant("Slaughterhouse Protector", 7864, 60, 3.6, 1.3),
            giant("Death Lord", 10729, 61, 5.0, 1.05),
        ],
        bosses: &[
            giant("Anub'Rekhan", 15931, 63, 4.6, 0.55),
            foe("Grand Widow Faerlina", 15940, 63, 4.8, 2.9),
            giant("Patchwerk", 16174, 63, 4.0, 1.7),
            giant("Gluth", 16064, 63, 5.0, 0.62),
            giant("Thaddius", 16137, 63, 4.0, 0.98),
            foe("Kel'Thuzad", 15945, 63, 4.4, 2.1),
        ],
    },
    MapDef {
        name: "Arathi Highlands",
        zone: "Eastern Kingdoms",
        map: 0,
        // Open ground on the road toward Stromgarde.
        center: [-1816.0, -2574.0, 60.0],
        leash: 140.0,
        victory: "The highlands keep their peace.",
        foes: &[
            foe("Syndicate Highwayman", 2342, 32, 4.4, 1.0),
            foe("Drywhisker Kobold", 373, 33, 4.0, 0.9),
            foe("Plains Creeper", 958, 34, 4.2, 1.1),
            foe("Highland Strider", 3027, 35, 4.8, 1.0),
            foe("Dabyrie Militia", 4418, 36, 4.2, 1.0),
            foe("Witherbark Axe Thrower", 10706, 37, 4.4, 1.1),
            giant("Boulderfist Ogre", 1122, 38, 4.0, 1.4),
            giant("Rumbling Exile", 11548, 39, 3.6, 1.3),
        ],
        bosses: &[
            foe("Or'Kalar", 10617, 42, 4.6, 2.2),
            foe("Lord Falconcrest", 383, 43, 4.4, 2.0),
            giant("Boulderfist Lord", 11549, 45, 4.2, 1.6),
        ],
    },
    MapDef {
        name: "Western Plaguelands",
        zone: "Eastern Kingdoms",
        map: 0,
        // The fields west of Chillwind.
        center: [943.0, -1419.0, 64.0],
        leash: 140.0,
        victory: "The western fields fall quiet.",
        foes: &[
            foe("Skeletal Warrior", 5231, 50, 4.4, 1.1),
            foe("Skeletal Flayer", 10626, 51, 4.6, 1.2),
            foe("Plaguebat", 7854, 52, 5.2, 0.9),
            foe("Scarlet Knight", 4418, 53, 4.4, 1.15),
            foe("Rotting Cadaver", 1693, 54, 3.8, 1.2),
            foe("Blighted Zombie", 612, 55, 4.2, 1.1),
            giant("Flesh Golem", 7864, 56, 3.6, 1.35),
            giant("Crypt Fiend", 11178, 57, 4.6, 1.1),
        ],
        bosses: &[
            foe("The Hederine Slayer", 10729, 58, 4.8, 2.0),
            giant("Lord Maldazzar", 16508, 60, 4.4, 1.4),
            foe("Araj the Summoner", 10427, 61, 4.2, 2.1),
        ],
    },
    MapDef {
        name: "Feralas",
        zone: "Kalimdor",
        map: 1,
        // Provisional dry terrain disk at a verified Snarler placement; campaign stays locked.
        center: [-4200.0, -133.828, 55.827],
        leash: 50.0,
        victory: "The wilds of Feralas fall quiet.",
        foes: &[
            foe("Longtooth Runner", 165, 40, 4.4, 1.0),
            foe("Ironfur Bear", 3201, 41, 4.4, 1.0),
            foe("Woodpaw Mongrel", 8598, 40, 4.4, 1.0),
            foe("Gordunni Ogre", 597, 40, 4.4, 1.0),
            foe("Gordunni Brute", 11554, 42, 4.4, 1.0),
            foe("Woodpaw Trapper", 10796, 41, 4.4, 1.0),
            foe("Feral Scar Yeti", 3207, 43, 4.4, 1.0),
            foe("Gordunni Warlock", 11560, 43, 4.4, 1.0),
            foe("Frayfeather Hippogryph", 3210, 43, 4.4, 1.0),
            foe("Woodpaw Mystic", 10794, 42, 4.4, 1.0),
            foe("Woodpaw Reaver", 10795, 42, 4.4, 1.0),
            foe("Rage Scar Yeti", 3220, 46, 4.4, 1.0),
            foe("Gordunni Mauler", 11557, 43, 4.4, 1.0),
            foe("Woodpaw Alpha", 10793, 43, 4.4, 1.0),
            foe("Wandering Forest Walker", 9592, 44, 4.4, 1.0),
        ],
        bosses: &[
            foe("Snarler", 780, 42, 4.8, 1.8),
            foe("Old Grizzlegut", 706, 43, 4.8, 1.8),
            foe("Diamond Head", 1817, 45, 4.8, 1.8),
            foe("Qirot", 11142, 47, 4.8, 1.8),
            foe("Arash-ethis", 7569, 49, 4.8, 1.8),
            foe("Antilus the Soarer", 10889, 48, 4.8, 1.8),
            foe("Mushgog", 14382, 60, 4.8, 1.8),
            foe("The Razza", 10807, 60, 4.8, 1.8),
            foe("Skarr the Unbreakable", 10169, 58, 4.8, 1.8),
        ],
    },
];

/// A battleground's horde and bosses with their numbers.
pub struct Roster {
    pub enemies: Vec<EnemyDef>,
    pub bosses: Vec<EnemyDef>,
}

static ROSTERS: std::sync::LazyLock<Vec<Roster>> = std::sync::LazyLock::new(|| {
    MAPS.iter()
        .map(|m| {
            if m.foes.is_empty() {
                Roster {
                    enemies: ENEMIES.to_vec(),
                    bosses: BOSSES.to_vec(),
                }
            } else {
                Roster {
                    enemies: ladder(m.foes, ENEMIES),
                    bosses: ladder(m.bosses, BOSSES),
                }
            }
        })
        .collect()
});

pub fn roster(map: usize) -> &'static Roster {
    &ROSTERS[map.min(ROSTERS.len() - 1)]
}

/// Number a roster off a tuned ladder: each creature takes the health, damage and experience at
/// its own place along it, from the ladder's running peak so no later tier comes out softer.
fn ladder(foes: &[Foe], tuned: &[EnemyDef]) -> Vec<EnemyDef> {
    let mut peak = (0.0f32, 0.0f32, 0.0f32);
    let rungs: Vec<(f32, f32, f32)> = tuned
        .iter()
        .map(|d| {
            peak = (
                peak.0.max(d.hp),
                peak.1.max(d.damage),
                peak.2.max(d.xp as f32),
            );
            peak
        })
        .collect();
    let top = (rungs.len() - 1) as f32;
    foes.iter()
        .enumerate()
        .map(|(i, f)| {
            let t = if foes.len() > 1 {
                i as f32 / (foes.len() - 1) as f32 * top
            } else {
                0.0
            };
            let (lo, hi) = (rungs[t.floor() as usize], rungs[t.ceil() as usize]);
            let k = t.fract();
            let mix = |a: f32, b: f32| (a + (b - a) * k).round();
            EnemyDef {
                name: f.name,
                display: f.display,
                hp: mix(lo.0, hi.0),
                speed: f.speed,
                damage: mix(lo.1, hi.1),
                scale: f.scale,
                xp: mix(lo.2, hi.2) as u32,
                level: f.level,
                big: f.big,
            }
        })
        .collect()
}
