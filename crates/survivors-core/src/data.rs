//! The mode's tables: classes, abilities, passives, enemies and the wave clock. Spell ids name
//! `Spell.dbc` rows only for their icon, name and visual chain; every number here is the mode's.

use crate::creature::CreatureType;

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
    /// Run-long weapon coating; applications are driven by original landed melee hits.
    Coating(crate::talents::Coating),
    /// Timed self buff for original weapon casts; never emits a damage hit.
    WeaponBuff,
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
    pub required_weapon: Option<crate::gear_loadout::MeleeKind>,
    pub requires_rear: bool,
    /// Direct damage; per tick for areas and pulses; absorb, heal or reflect for the self kinds.
    pub damage: f32,
    /// Fractional weapon-damage bonus for WeaponBuff; independent of global damage.
    pub weapon_bonus: f32,
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

impl AbilityDef {
    /// Equipment identity is taken from the worn loadout, never its appearance.
    pub fn usable_with(&self, weapon: Option<crate::gear_loadout::MeleeWeapon>) -> bool {
        if matches!(self.kind, Kind::Coating(_)) && !crate::talents::Coating::weapon_allowed(weapon)
        {
            return false;
        }
        self.required_weapon
            .is_none_or(|required| weapon.is_some_and(|w| w.kind == required))
    }
}

const fn ab(key: &'static str, spell: u32, kind: Kind) -> AbilityDef {
    AbilityDef {
        key,
        spell,
        kind,
        required_weapon: None,
        requires_rear: false,
        damage: 0.0,
        weapon_bonus: 0.0,
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
        "mana_shield",
        1463,
        Shield {
            damage: 40.0,
            cooldown: 18.0,
            duration: 10.0
        }
    ),
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
    // Survivors tuning: self-only percentage damage replaces vanilla party attack power.
    ability!(
        "battle_shout",
        6673,
        WeaponBuff {
            weapon_bonus: 0.10,
            duration: 8.0,
            cooldown: 20.0,
            range: 0.0,
            count: 0
        }
    ),
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
    AbilityDef {
        damage: 22.0,
        ..ab(
            "instant_poison",
            8679,
            Kind::Coating(crate::talents::Coating::Instant),
        )
    },
    AbilityDef {
        damage: 9.0,
        duration: 12.0,
        ..ab(
            "deadly_poison",
            2823,
            Kind::Coating(crate::talents::Coating::Deadly),
        )
    },
    ability!(
        "backstab",
        53,
        Melee {
            damage: 30.0,
            cooldown: 1.3,
            range: 5.5,
            required_weapon: Some(crate::gear_loadout::MeleeKind::Dagger),
            requires_rear: true
        }
    ),
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
        "wing_clip",
        2974,
        Melee {
            damage: 5.0,
            cooldown: 4.0,
            range: 4.0,
            effect: Effect::Slow(0.5, 5.0),
            mark: 2974
        }
    ),
    ability!(
        "raptor_strike",
        2973,
        Melee {
            damage: 30.0,
            cooldown: 3.0,
            range: 4.0
        }
    ),
    // Automatic close attack: Survivors has no dodge-triggered action bar.
    ability!(
        "mongoose_bite",
        1495,
        Melee {
            damage: 22.0,
            cooldown: 2.5,
            range: 4.0
        }
    ),
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
        "scorpid_sting",
        3043,
        Bolt {
            damage: 4.0,
            cooldown: 4.0,
            range: 26.0,
            effect: Effect::Weaken(0.75, 6.0),
            mark: 3043
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
            "battle_shout",
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
            "scorpid_sting",
            "raptor_strike",
            "mongoose_bite",
            "wing_clip",
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
            "backstab",
            "instant_poison",
            "deadly_poison",
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
            "mana_shield",
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
    pub template_entry: u32,
    pub creature_type: CreatureType,
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

impl EnemyDef {
    const fn typed(mut self, entry: u32, kind: CreatureType) -> Self {
        self.template_entry = entry;
        self.creature_type = kind;
        self
    }
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
        template_entry: 0,
        creature_type: CreatureType::Unknown,
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
    en("Fleeting Plainstrider", 1284, 8.0, 3.6, 2.0, 1, 10).typed(3246, CreatureType::Beast),
    en("Zhevra Runner", 6087, 10.0, 4.4, 2.0, 1, 11).typed(3242, CreatureType::Beast),
    en("Razormane Quilboar", 1218, 16.0, 3.4, 3.0, 2, 12).typed(3111, CreatureType::Humanoid),
    en("Razormane Hunter", 6094, 18.0, 3.6, 4.0, 2, 12).typed(3265, CreatureType::Humanoid),
    en("Savannah Huntress", 1056, 22.0, 4.8, 8.0, 2, 13).typed(3415, CreatureType::Beast),
    en("Sunscale Lashtail", 1744, 26.0, 4.6, 8.0, 3, 13).typed(3254, CreatureType::Beast),
    en("Kolkar Wrangler", 9442, 36.0, 4.0, 10.0, 3, 14).typed(3272, CreatureType::Humanoid),
    en("Kolkar Stormer", 9443, 40.0, 4.0, 11.0, 3, 14).typed(3273, CreatureType::Humanoid),
    en("Witchwing Harpy", 3218, 34.0, 5.0, 10.0, 3, 15).typed(3276, CreatureType::Humanoid),
    en("Hecklefang Hyena", 2710, 44.0, 5.2, 12.0, 4, 15).typed(4127, CreatureType::Beast),
    en("Razormane Defender", 1253, 70.0, 3.3, 14.0, 5, 16).typed(3266, CreatureType::Humanoid),
    en("Razormane Mystic", 4643, 55.0, 3.8, 12.0, 4, 16).typed(3271, CreatureType::Humanoid),
    en("Sunscale Screecher", 1747, 60.0, 5.0, 13.0, 4, 17).typed(3255, CreatureType::Beast),
    en("Kolkar Bloodcharger", 9447, 85.0, 4.6, 16.0, 5, 18).typed(3397, CreatureType::Humanoid),
    en("Savannah Prowler", 1973, 75.0, 5.4, 15.0, 5, 18).typed(3425, CreatureType::Beast),
    en("Thunderhawk Hatchling", 1742, 90.0, 4.2, 16.0, 6, 19).typed(3247, CreatureType::Beast),
    en("Witchwing Slayer", 2163, 95.0, 5.0, 18.0, 6, 20).typed(3278, CreatureType::Humanoid),
    en("Bristleback Geomancer", 6091, 120.0, 3.6, 20.0, 7, 21).typed(3263, CreatureType::Humanoid),
    huge(en("Stormsnout", 1537, 150.0, 4.0, 22.0, 8, 22).typed(3240, CreatureType::Beast)),
    huge(en("Barrens Kodo", 1453, 220.0, 3.2, 28.0, 12, 23).typed(3236, CreatureType::Beast)),
    en("Sunscale Scytheclaw", 4442, 160.0, 5.6, 24.0, 8, 24).typed(3256, CreatureType::Beast),
    huge(en("Thunderhead", 1538, 180.0, 4.2, 26.0, 10, 25).typed(3239, CreatureType::Beast)),
    en("Silithid Swarmer", 2731, 90.0, 6.0, 18.0, 5, 25).typed(3252, CreatureType::Beast),
    huge(
        en("Greater Thunderhawk", 1974, 240.0, 4.4, 30.0, 12, 26).typed(3249, CreatureType::Beast),
    ),
    en("Hecklefang Stalker", 2712, 200.0, 6.0, 30.0, 10, 27).typed(4128, CreatureType::Beast),
    en("Kolkar Destroyer", 9446, 280.0, 4.6, 34.0, 14, 28).typed(4637, CreatureType::Humanoid),
    en("Kolkar Mauler", 4874, 300.0, 4.6, 36.0, 14, 29).typed(4634, CreatureType::Humanoid),
    huge(
        en("Greater Barrens Kodo", 1232, 420.0, 3.4, 42.0, 24, 30).typed(3235, CreatureType::Beast),
    ),
    huge(
        en("Elder Thunder Lizard", 2764, 380.0, 4.4, 44.0, 22, 31).typed(4727, CreatureType::Beast),
    ),
    huge(en("Silithid Protector", 1307, 360.0, 5.0, 40.0, 20, 32).typed(3503, CreatureType::Beast)),
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
        ..en("Echeyakee", 1934, 700.0, 5.2, 20.0, 60, 16).typed(3475, CreatureType::Beast)
    },
    EnemyDef {
        scale: 1.6,
        ..en("Takk the Leaper", 1337, 1800.0, 5.4, 32.0, 120, 19).typed(5842, CreatureType::Beast)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Lakota'mani", 1241, 4200.0, 4.0, 45.0, 200, 22).typed(3474, CreatureType::Beast)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Humar the Pridelord", 4424, 7000.0, 5.6, 55.0, 300, 26)
            .typed(5828, CreatureType::Beast)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Agathelos the Raging", 2450, 12000.0, 5.0, 70.0, 400, 33)
            .typed(4422, CreatureType::Beast)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Hezrul Bloodmark", 9448, 20000.0, 4.8, 90.0, 500, 33)
            .typed(3396, CreatureType::Humanoid)
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
    template_entry: u32,
    creature_type: CreatureType,
    name: &'static str,
    display: u32,
    level: u32,
    speed: f32,
    /// The drawn scale, chosen against the model's Stand height so the field reads as The
    /// Barrens does: about 2-3.5 yd for the horde, 4-5 for the great beasts, 5-9 for a boss.
    scale: f32,
    big: bool,
}

impl Foe {
    const fn typed(mut self, entry: u32, kind: CreatureType) -> Self {
        self.template_entry = entry;
        self.creature_type = kind;
        self
    }
}

const fn foe(name: &'static str, display: u32, level: u32, speed: f32, scale: f32) -> Foe {
    Foe {
        template_entry: 0,
        creature_type: CreatureType::Unknown,
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
            foe("Starving Snickerfang", 8050, 45, 4.6, 1.0).typed(5984, CreatureType::Beast),
            foe("Snickerfang Hyena", 2714, 49, 5.0, 1.15).typed(5985, CreatureType::Beast),
            foe("Black Slayer", 10824, 47, 5.0, 0.6).typed(5982, CreatureType::Beast),
            foe("Redstone Crystalhide", 798, 51, 3.6, 1.3).typed(5991, CreatureType::Beast),
            foe("Shadowsworn Cultist", 7838, 51, 3.8, 1.3).typed(6004, CreatureType::Humanoid),
            foe("Felbeast", 7949, 50, 4.8, 0.85).typed(8675, CreatureType::Demon),
            foe("Dreadmaul Mauler", 14401, 53, 3.6, 1.1).typed(5977, CreatureType::Humanoid),
            foe("Felhound", 1913, 54, 5.2, 1.0).typed(6010, CreatureType::Demon),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9).typed(6011, CreatureType::Demon),
            foe("Manahound", 6173, 60, 5.6, 1.15).typed(8718, CreatureType::Demon),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0).typed(4676, CreatureType::Demon),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0).typed(8717, CreatureType::Demon),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1).typed(12396, CreatureType::Demon),
        ],
        bosses: &[
            foe("Akubar the Seer", 10920, 54, 4.4, 2.6).typed(8298, CreatureType::Humanoid),
            foe("Lord Azrethoc", 4426, 55, 4.6, 1.35).typed(5760, CreatureType::Demon),
            foe("Lord Hel'nurath", 14556, 62, 4.8, 1.65).typed(14506, CreatureType::Demon),
            giant("Teremus the Devourer", 6378, 60, 5.0, 0.55).typed(7846, CreatureType::Dragonkin),
            foe("Lord Banehollow", 8611, 62, 4.8, 1.95).typed(9516, CreatureType::Demon),
            giant("Lord Kazzak", 12449, 63, 4.6, 0.75).typed(12397, CreatureType::Demon),
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
        // Outdoor Silithus identities; arrivals and tuning are authored Survivors encounters.
        foes: &[
            foe("Sand Skitterer", 8014, 55, 5.0, 1.0).typed(11738, CreatureType::Beast),
            foe("Stonelash Scorpid", 15383, 54, 4.6, 1.6).typed(11735, CreatureType::Beast),
            foe("Dredge Striker", 15386, 55, 4.4, 1.0).typed(11740, CreatureType::Beast),
            foe("Twilight Avenger", 11811, 57, 4.0, 1.1).typed(11880, CreatureType::Humanoid),
            foe("Hive'Ashi Worker", 12151, 57, 5.4, 0.55).typed(11721, CreatureType::Unspecified),
            foe("Stonelash Flayer", 15385, 57, 4.8, 1.75).typed(11737, CreatureType::Beast),
            foe("Hive'Ashi Defender", 12153, 58, 4.2, 1.3).typed(11722, CreatureType::Unspecified),
            foe("Dust Stormer", 8715, 55, 4.6, 1.05).typed(11744, CreatureType::Elemental),
            foe("Twilight Stonecaller", 11815, 59, 3.8, 1.1).typed(11882, CreatureType::Humanoid),
            foe("Hive'Zora Wasp", 11088, 57, 5.6, 0.6).typed(11727, CreatureType::Unspecified),
            foe("Hive'Zora Reaver", 11085, 59, 4.8, 1.0).typed(11728, CreatureType::Unspecified),
            foe("Hive'Regal Ambusher", 11106, 59, 5.2, 1.05)
                .typed(11730, CreatureType::Unspecified),
            foe("Hive'Ashi Stinger", 12149, 57, 5.6, 0.65).typed(11698, CreatureType::Unspecified),
            foe("Hive'Zora Hive Sister", 12160, 59, 4.4, 0.75)
                .typed(11729, CreatureType::Unspecified),
            foe("Hive'Regal Spitfire", 9229, 59, 4.6, 1.15).typed(11732, CreatureType::Unspecified),
            foe("Hive'Regal Slavemaker", 12154, 59, 3.8, 1.2)
                .typed(11733, CreatureType::Unspecified),
            foe("Hive'Regal Hive Lord", 12155, 59, 4.6, 0.95)
                .typed(11734, CreatureType::Unspecified),
            foe("Twilight Overlord", 15407, 59, 4.0, 1.2).typed(15213, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Gretheer", 1104, 57, 4.4, 2.0).typed(14472, CreatureType::Beast),
            foe("Krellack", 6068, 56, 4.4, 2.0).typed(14476, CreatureType::Beast),
            foe("Grubthor", 14523, 58, 4.4, 2.0).typed(14477, CreatureType::Beast),
            foe("Huricanian", 14525, 58, 4.4, 2.0).typed(14478, CreatureType::Elemental),
            foe("Lapress", 14521, 60, 4.4, 2.0).typed(14473, CreatureType::Unspecified),
            foe("Deathclasp", 15327, 59, 4.4, 2.0).typed(15196, CreatureType::Beast),
            foe("Twilight Lord Everun", 14526, 60, 4.4, 2.0).typed(14479, CreatureType::Humanoid),
            giant("Setis", 5965, 61, 4.4, 2.0).typed(14471, CreatureType::Humanoid),
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
            foe("Bloodpetal Lasher", 11634, 49, 3.6, 0.9).typed(6509, CreatureType::Elemental),
            foe("Young Diemetradon", 8510, 48, 4.4, 0.55).typed(9162, CreatureType::Beast),
            foe("Ravasaur", 5242, 50, 5.4, 0.75).typed(6505, CreatureType::Beast),
            foe("Muculent Ooze", 11140, 50, 3.2, 1.4).typed(6556, CreatureType::Unspecified),
            foe("Ravasaur Hunter", 5292, 51, 5.6, 0.85).typed(6507, CreatureType::Beast),
            foe("Tar Beast", 1549, 51, 3.4, 1.0).typed(6517, CreatureType::Elemental),
            foe("Bloodpetal Flayer", 11635, 52, 3.8, 1.05).typed(6510, CreatureType::Elemental),
            foe("Primal Ooze", 4754, 52, 3.4, 1.6).typed(6557, CreatureType::Unspecified),
            foe("Gorishi Wasp", 11090, 53, 5.4, 0.55).typed(6551, CreatureType::Unspecified),
            foe("Pterrordax", 8411, 53, 5.2, 0.5).typed(9166, CreatureType::Beast),
            foe("Diemetradon", 8511, 52, 4.6, 0.7).typed(9163, CreatureType::Beast),
            foe("Glutinous Ooze", 1146, 54, 3.6, 1.8).typed(6559, CreatureType::Unspecified),
            foe("Tar Lurker", 9010, 54, 3.6, 1.15).typed(6518, CreatureType::Elemental),
            foe("Scorching Elemental", 1070, 54, 4.2, 1.05).typed(6520, CreatureType::Elemental),
            foe("Frenzied Pterrordax", 8412, 55, 5.6, 0.55).typed(9167, CreatureType::Beast),
            foe("Elder Diemetradon", 8512, 55, 4.6, 0.8).typed(9164, CreatureType::Beast),
            giant("Stegodon", 5241, 52, 3.8, 0.72).typed(6501, CreatureType::Beast),
            giant("Stone Guardian", 8395, 52, 3.6, 1.45).typed(6560, CreatureType::Elemental),
            giant("Plated Stegodon", 5287, 54, 3.8, 0.78).typed(6502, CreatureType::Beast),
            giant("Devilsaur", 5239, 56, 5.0, 0.72).typed(6498, CreatureType::Beast),
        ],
        bosses: &[
            foe("Ravasaur Matriarch", 11319, 56, 5.6, 1.55).typed(6581, CreatureType::Beast),
            foe("Clutchmother Zavas", 11084, 56, 5.2, 1.4).typed(6582, CreatureType::Unspecified),
            foe("Uhk'loc", 8129, 59, 4.8, 2.4).typed(6585, CreatureType::Beast),
            foe("Gruff", 10932, 59, 4.4, 2.2).typed(6583, CreatureType::Beast),
            foe("Blazerunner", 1204, 57, 4.6, 1.8).typed(9376, CreatureType::Elemental),
            foe("Baron Charr", 14517, 58, 4.6, 1.95).typed(14461, CreatureType::Elemental),
            giant("Tyrant Devilsaur", 5240, 60, 5.0, 1.0).typed(6500, CreatureType::Beast),
            foe("U'cha", 8844, 60, 4.8, 2.6).typed(9622, CreatureType::Beast),
            giant("King Mosh", 5305, 61, 5.0, 1.2).typed(6584, CreatureType::Beast),
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
            foe("Shadowmaw Panther", 633, 37, 5.6, 1.4).typed(684, CreatureType::Beast),
            foe("Cold Eye Basilisk", 8797, 39, 3.8, 1.0).typed(690, CreatureType::Beast),
            foe("Jungle Stalker", 11317, 40, 5.2, 0.8).typed(687, CreatureType::Beast),
            foe("Elder Mistvale Gorilla", 838, 40, 4.0, 1.1).typed(1557, CreatureType::Beast),
            foe("Zanzil Zombie", 1065, 43, 3.4, 1.1).typed(1488, CreatureType::Undead),
            foe("Thrashtail Basilisk", 8802, 41, 4.0, 1.15).typed(1550, CreatureType::Beast),
            foe("Bloodsail Sea Dog", 796, 43, 4.0, 1.05).typed(1565, CreatureType::Humanoid),
            foe("Skullsplitter Berserker", 4623, 43, 4.2, 1.1).typed(783, CreatureType::Humanoid),
            foe("Bloodsail Raider", 793, 39, 4.2, 1.1).typed(1561, CreatureType::Humanoid),
            foe("Skullsplitter Hunter", 4628, 41, 4.2, 1.1).typed(669, CreatureType::Humanoid),
            foe("Skullsplitter Mystic", 4616, 39, 4.2, 1.1).typed(780, CreatureType::Humanoid),
            foe("Bloodsail Warlock", 4461, 41, 4.2, 1.1).typed(1564, CreatureType::Humanoid),
            foe("Zanzil Hunter", 828, 43, 4.2, 1.1).typed(1489, CreatureType::Undead),
            foe("Jungle Thunderer", 845, 37, 4.2, 1.1).typed(1114, CreatureType::Beast),
            foe("Stranglethorn Tigress", 614, 37, 4.2, 1.1).typed(772, CreatureType::Beast),
        ],
        bosses: &[
            foe("Kurmokk", 3188, 42, 4.4, 2.0).typed(14491, CreatureType::Beast),
            foe("Verifonix", 7232, 42, 4.4, 2.0).typed(14492, CreatureType::Humanoid),
            foe("Scale Belly", 12342, 45, 4.4, 2.0).typed(1552, CreatureType::Beast),
            foe("Lord Sakrasis", 4910, 45, 4.4, 2.0).typed(2541, CreatureType::Humanoid),
            foe("Rippa", 14528, 44, 4.4, 2.0).typed(14490, CreatureType::Beast),
            foe("Tethis", 8472, 43, 4.4, 2.0).typed(730, CreatureType::Beast),
            foe("Bhag'thera", 613, 40, 4.4, 2.0).typed(728, CreatureType::Beast),
            giant("King Bangalash", 616, 43, 5.8, 3.0).typed(731, CreatureType::Beast),
            foe("King Mukla", 792, 51, 4.4, 2.0).typed(1559, CreatureType::Beast),
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
            foe("Young Wolf", 447, 1, 5.4, 1.0).typed(299, CreatureType::Beast),
            foe("Timber Wolf", 604, 2, 5.4, 1.0).typed(69, CreatureType::Beast),
            foe("Defias Thug", 5035, 4, 4.2, 1.15).typed(38, CreatureType::Humanoid),
            foe("Mangy Wolf", 903, 6, 5.4, 1.0).typed(525, CreatureType::Beast),
            foe("Stonetusk Boar", 503, 6, 4.6, 1.0).typed(113, CreatureType::Beast),
            foe("Kobold Tunneler", 139, 6, 4.2, 1.0).typed(475, CreatureType::Humanoid),
            foe("Defias Cutpurse", 2361, 6, 4.2, 1.15).typed(94, CreatureType::Humanoid),
            foe("Forest Spider", 382, 6, 4.2, 1.0).typed(30, CreatureType::Beast),
            foe("Murloc", 617, 7, 4.2, 1.0).typed(285, CreatureType::Humanoid),
            foe("Kobold Miner", 373, 7, 4.2, 1.0).typed(40, CreatureType::Humanoid),
            foe("Murloc Streamrunner", 527, 7, 4.2, 1.0).typed(735, CreatureType::Humanoid),
            foe("Rockhide Boar", 389, 8, 4.6, 1.0).typed(524, CreatureType::Beast),
            foe("Gray Forest Wolf", 380, 8, 5.4, 1.0).typed(1922, CreatureType::Beast),
            foe("Defias Bandit", 2357, 9, 4.2, 1.15).typed(116, CreatureType::Humanoid),
            giant("Young Forest Bear", 1006, 9, 4.6, 1.0).typed(822, CreatureType::Beast),
            foe("Defias Rogue Wizard", 2359, 10, 3.8, 1.1).typed(474, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Mother Fang", 2541, 10, 4.6, 1.9).typed(471, CreatureType::Beast),
            foe("Narg the Taskmaster", 774, 10, 4.2, 1.9).typed(79, CreatureType::Humanoid),
            foe("Hogger", 384, 11, 4.2, 1.9).typed(448, CreatureType::Humanoid),
            foe("Thuros Lightfingers", 3341, 11, 4.2, 2.18).typed(61, CreatureType::Humanoid),
            foe("Fedfennel", 175, 12, 4.2, 1.9).typed(472, CreatureType::Humanoid),
            foe("Gruff Swiftbite", 175, 12, 4.2, 1.9).typed(100, CreatureType::Humanoid),
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
            foe("Vile Familiar", 1185, 4, 4.2, 1.0).typed(3101, CreatureType::Humanoid),
            foe("Felstalker", 850, 4, 4.2, 1.0).typed(3102, CreatureType::Demon),
            foe("Clattering Scorpid", 2486, 6, 4.6, 1.5).typed(3125, CreatureType::Beast),
            foe("Pygmy Surf Crawler", 1307, 6, 4.6, 1.0).typed(3106, CreatureType::Beast),
            foe("Dire Mottled Boar", 381, 7, 4.6, 1.0).typed(3099, CreatureType::Beast),
            foe("Razormane Quilboar", 1218, 7, 4.6, 1.0).typed(3111, CreatureType::Humanoid),
            foe("Armored Scorpid", 2487, 8, 4.6, 1.5).typed(3126, CreatureType::Beast),
            foe("Bloodtalon Taillasher", 1960, 8, 5.4, 0.85).typed(3122, CreatureType::Beast),
            foe("Razormane Scout", 1252, 8, 4.6, 1.0).typed(3112, CreatureType::Humanoid),
            foe("Dustwind Harpy", 3012, 8, 4.2, 1.0).typed(3115, CreatureType::Humanoid),
            foe("Dustwind Pillager", 6814, 8, 4.2, 1.0).typed(3116, CreatureType::Humanoid),
            foe("Razormane Dustrunner", 6088, 9, 4.6, 1.0).typed(3113, CreatureType::Humanoid),
            foe("Elder Mottled Boar", 193, 9, 4.6, 1.0).typed(3100, CreatureType::Beast),
            foe("Razormane Battleguard", 1964, 10, 4.6, 1.0).typed(3114, CreatureType::Humanoid),
            foe("Burning Blade Neophyte", 4186, 10, 3.8, 1.0).typed(3196, CreatureType::Humanoid),
            foe("Burning Blade Cultist", 4194, 11, 3.8, 1.0).typed(3199, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Geolord Mottle", 6113, 9, 4.6, 1.9).typed(5826, CreatureType::Humanoid),
            foe("Razormane Dustrunner", 6088, 9, 4.6, 1.9).typed(3113, CreatureType::Humanoid),
            foe("Razormane Battleguard", 1964, 10, 4.6, 1.9).typed(3114, CreatureType::Humanoid),
            foe("Burning Blade Neophyte", 4186, 10, 3.8, 1.9).typed(3196, CreatureType::Humanoid),
            foe("Death Flayer", 2491, 11, 4.6, 2.85).typed(5823, CreatureType::Beast),
            foe("Burning Blade Cultist", 4194, 11, 3.8, 1.9).typed(3199, CreatureType::Humanoid),
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
            foe("Young Nightsaber", 11454, 1, 5.4, 1.5).typed(2031, CreatureType::Beast),
            foe("Young Thistle Boar", 8869, 2, 4.6, 1.0).typed(1984, CreatureType::Beast),
            foe("Grell", 6806, 3, 4.2, 1.0).typed(1988, CreatureType::Demon),
            foe("Grellkin", 3023, 4, 4.2, 1.0).typed(1989, CreatureType::Demon),
            foe("Nightsaber", 6805, 6, 5.4, 1.5).typed(2042, CreatureType::Beast),
            foe("Strigid Owl", 10832, 6, 5.2, 1.0).typed(1995, CreatureType::Beast),
            foe("Webwood Lurker", 760, 6, 4.2, 1.0).typed(1998, CreatureType::Beast),
            foe("Timberling", 3033, 6, 4.2, 1.15).typed(2022, CreatureType::Elemental),
            foe("Gnarlpine Gardener", 6799, 6, 3.8, 1.0).typed(2007, CreatureType::Humanoid),
            foe("Gnarlpine Warrior", 6800, 7, 4.2, 1.0).typed(2008, CreatureType::Humanoid),
            foe("Gnarlpine Ambusher", 3024, 7, 4.2, 1.0).typed(2152, CreatureType::Humanoid),
            foe("Vicious Grell", 6811, 7, 4.2, 1.0).typed(2005, CreatureType::Demon),
            foe("Gnarlpine Defender", 6801, 8, 4.2, 1.0).typed(2010, CreatureType::Humanoid),
            foe("Gnarlpine Shaman", 897, 8, 3.8, 1.0).typed(2009, CreatureType::Humanoid),
            foe("Timberling Bark Ripper", 6817, 8, 4.2, 1.15).typed(2025, CreatureType::Elemental),
            foe("Gnarlpine Augur", 3024, 9, 3.8, 1.0).typed(2011, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Threggil", 904, 6, 4.2, 1.9).typed(14432, CreatureType::Demon),
            foe("Uruson", 6818, 7, 4.2, 1.9).typed(14428, CreatureType::Humanoid),
            foe("Gnarlpine Defender", 6801, 8, 4.2, 1.9).typed(2010, CreatureType::Humanoid),
            foe("Duskstalker", 11453, 9, 5.4, 2.85).typed(14430, CreatureType::Beast),
            foe("Oakenscowl", 2567, 9, 4.2, 2.18).typed(2166, CreatureType::Elemental),
            foe("Gnarlpine Augur", 3024, 9, 3.8, 1.9).typed(2011, CreatureType::Humanoid),
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
            foe("Crag Boar", 607, 6, 4.6, 1.0).typed(1125, CreatureType::Beast),
            giant("Young Black Bear", 8843, 6, 4.6, 1.0).typed(1128, CreatureType::Beast),
            foe("Juvenile Snow Leopard", 748, 6, 5.4, 1.5).typed(1199, CreatureType::Beast),
            foe("Young Wendigo", 929, 6, 4.2, 1.0).typed(1134, CreatureType::Humanoid),
            foe("Wendigo", 950, 7, 4.2, 1.0).typed(1135, CreatureType::Humanoid),
            foe("Large Crag Boar", 381, 7, 4.6, 1.0).typed(1126, CreatureType::Beast),
            foe("Snow Tracker Wolf", 604, 7, 5.4, 1.0).typed(1138, CreatureType::Beast),
            foe("Snow Leopard", 954, 8, 5.4, 1.5).typed(1201, CreatureType::Beast),
            foe("Elder Crag Boar", 744, 8, 4.6, 1.0).typed(1127, CreatureType::Beast),
            foe("Winter Wolf", 785, 8, 5.4, 1.0).typed(1131, CreatureType::Beast),
            giant("Ice Claw Bear", 8840, 8, 4.6, 1.0).typed(1196, CreatureType::Beast),
            foe("Frostmane Seer", 5610, 9, 3.8, 1.2).typed(1397, CreatureType::Humanoid),
            foe("Frostmane Snowstrider", 5607, 9, 4.2, 1.2).typed(1121, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Edan the Howler", 931, 9, 4.2, 1.9).typed(1137, CreatureType::Humanoid),
            foe("Timber", 11422, 10, 5.4, 1.9).typed(1132, CreatureType::Beast),
            foe("Great Father Arctikus", 5625, 11, 4.2, 2.28).typed(1260, CreatureType::Humanoid),
            foe("Old Icebeard", 1050, 11, 4.2, 1.9).typed(1271, CreatureType::Humanoid),
            foe("Vagash", 1050, 11, 4.2, 1.9).typed(1388, CreatureType::Humanoid),
            giant("Bjarn", 913, 12, 4.6, 1.9).typed(1130, CreatureType::Beast),
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
            foe("Plainstrider", 1219, 2, 5.4, 1.0).typed(2955, CreatureType::Beast),
            foe("Battleboar", 8869, 4, 4.6, 1.0).typed(2966, CreatureType::Beast),
            foe("Prairie Wolf", 1100, 6, 5.4, 1.0).typed(2958, CreatureType::Beast),
            foe("Palemane Tanner", 550, 6, 4.2, 1.0).typed(2949, CreatureType::Humanoid),
            foe("Venture Co. Hireling", 7243, 6, 4.2, 1.3).typed(2975, CreatureType::Humanoid),
            foe("Adult Plainstrider", 1220, 7, 5.4, 1.0).typed(2956, CreatureType::Beast),
            foe("Wiry Swoop", 1228, 7, 5.2, 0.8).typed(2969, CreatureType::Beast),
            foe("Venture Co. Laborer", 7244, 7, 4.2, 1.3).typed(2976, CreatureType::Humanoid),
            foe("Palemane Skinner", 1216, 7, 4.2, 1.0).typed(2950, CreatureType::Humanoid),
            foe("Prairie Stalker", 643, 8, 5.4, 1.0).typed(2959, CreatureType::Beast),
            foe("Flatland Cougar", 1059, 8, 5.4, 1.5).typed(3035, CreatureType::Beast),
            foe("Palemane Poacher", 1217, 8, 4.2, 1.0).typed(2951, CreatureType::Humanoid),
            foe("Venture Co. Taskmaster", 7246, 8, 4.2, 1.3).typed(2977, CreatureType::Humanoid),
            foe("Swoop", 1229, 9, 5.2, 0.8).typed(2970, CreatureType::Beast),
            foe("Elder Plainstrider", 1221, 9, 5.4, 1.0).typed(2957, CreatureType::Beast),
        ],
        bosses: &[
            foe("Prairie Stalker", 643, 8, 5.4, 1.9).typed(2959, CreatureType::Beast),
            foe("Mazzranache", 1961, 9, 5.4, 1.9).typed(3068, CreatureType::Beast),
            foe("Snagglespear", 275, 9, 4.2, 1.9).typed(5786, CreatureType::Humanoid),
            foe("Swoop", 1229, 9, 5.2, 1.52).typed(2970, CreatureType::Beast),
            foe("Elder Plainstrider", 1221, 9, 5.4, 1.9).typed(2957, CreatureType::Beast),
            foe("Ghost Howl", 720, 12, 5.4, 1.9).typed(3056, CreatureType::Beast),
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
            foe("Decrepit Darkhound", 9021, 6, 4.2, 1.4).typed(1547, CreatureType::Demon),
            foe("Rotting Dead", 1200, 6, 3.6, 1.0).typed(1525, CreatureType::Undead),
            foe("Greater Duskbat", 4734, 7, 5.2, 0.8).typed(1553, CreatureType::Beast),
            foe("Ravaged Corpse", 1201, 7, 3.6, 1.0).typed(1526, CreatureType::Undead),
            foe("Rot Hide Graverobber", 10853, 7, 4.2, 1.0).typed(1941, CreatureType::Undead),
            foe("Rot Hide Gnoll", 10852, 7, 4.2, 1.0).typed(1674, CreatureType::Undead),
            foe("Rattlecage Soldier", 200, 7, 4.2, 1.2).typed(1520, CreatureType::Undead),
            foe("Cursed Darkhound", 9020, 8, 4.2, 1.4).typed(1548, CreatureType::Demon),
            foe("Rot Hide Mongrel", 847, 8, 4.2, 1.0).typed(1675, CreatureType::Undead),
            foe("Hungering Dead", 1202, 8, 3.6, 1.0).typed(1527, CreatureType::Undead),
            foe("Scarlet Missionary", 2468, 8, 3.8, 1.15).typed(1536, CreatureType::Humanoid),
            foe("Darkeye Bonecaster", 11398, 8, 3.8, 1.0).typed(1522, CreatureType::Undead),
            foe("Vampiric Duskbat", 8808, 9, 5.2, 0.8).typed(1554, CreatureType::Beast),
            foe("Scarlet Zealot", 2483, 9, 4.2, 1.1).typed(1537, CreatureType::Humanoid),
            foe("Shambling Horror", 1197, 9, 3.6, 1.0).typed(1528, CreatureType::Undead),
        ],
        bosses: &[
            foe("Farmer Solliden", 3535, 8, 4.2, 2.18).typed(1936, CreatureType::Humanoid),
            foe("Tormented Spirit", 9534, 9, 4.2, 1.52).typed(1533, CreatureType::Undead),
            foe("Bayne", 7892, 10, 4.2, 2.66).typed(10356, CreatureType::Demon),
            foe("Ressan the Needler", 9750, 11, 5.2, 1.52).typed(10357, CreatureType::Beast),
            foe("Fellicent's Shade", 5430, 12, 4.2, 1.9).typed(10358, CreatureType::Undead),
            foe("Gorefang", 11413, 13, 5.4, 1.9).typed(12431, CreatureType::Beast),
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
            foe("Young Fleshripper", 410, 11, 5.2, 0.8).typed(199, CreatureType::Beast),
            foe("Coyote", 643, 11, 5.4, 1.0).typed(834, CreatureType::Beast),
            foe("Defias Smuggler", 4418, 12, 4.2, 1.15).typed(95, CreatureType::Humanoid),
            giant("Harvest Golem", 367, 12, 3.6, 1.0).typed(36, CreatureType::Mechanical),
            foe("Young Goretusk", 8871, 13, 4.6, 1.0).typed(454, CreatureType::Beast),
            foe("Defias Trapper", 2331, 13, 4.2, 1.1).typed(504, CreatureType::Humanoid),
            foe("Fleshripper", 2305, 14, 5.2, 0.8).typed(1109, CreatureType::Beast),
            foe("Goretusk", 3027, 15, 4.6, 1.0).typed(157, CreatureType::Beast),
            giant("Harvest Watcher", 378, 15, 3.6, 1.0).typed(114, CreatureType::Mechanical),
            foe("Riverpaw Brute", 384, 16, 4.2, 1.0).typed(124, CreatureType::Humanoid),
            foe("Riverpaw Bandit", 383, 17, 4.2, 1.0).typed(452, CreatureType::Humanoid),
            foe("Great Goretusk", 3035, 17, 4.6, 1.0).typed(547, CreatureType::Beast),
            foe("Riverpaw Taskmaster", 376, 18, 4.2, 1.0).typed(98, CreatureType::Humanoid),
            foe("Defias Highwayman", 2342, 18, 4.2, 1.15).typed(122, CreatureType::Humanoid),
            foe("Pygmy Venom Web Spider", 958, 19, 4.6, 1.0).typed(539, CreatureType::Beast),
            foe("Riverpaw Mystic", 502, 19, 3.8, 1.0).typed(453, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Brainwashed Noble", 3267, 18, 4.2, 2.18).typed(596, CreatureType::Humanoid),
            foe("Marisa du'Paige", 2355, 18, 4.2, 2.09).typed(599, CreatureType::Humanoid),
            foe("Sergeant Brashclaw", 383, 18, 4.2, 1.9).typed(506, CreatureType::Humanoid),
            foe("Klaven Mortwake", 5805, 23, 4.2, 2.18).typed(7053, CreatureType::Humanoid),
            foe("Lupos", 11412, 23, 5.4, 1.9).typed(521, CreatureType::Beast),
            foe("Vultros", 507, 26, 5.2, 1.52).typed(462, CreatureType::Beast),
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
            foe("Pygmy Venom Web Spider", 958, 19, 4.6, 1.0).typed(539, CreatureType::Beast),
            foe("Venom Web Spider", 955, 20, 4.6, 1.0).typed(217, CreatureType::Beast),
            foe("Skeletal Warrior", 200, 22, 4.2, 1.2).typed(48, CreatureType::Undead),
            foe("Green Recluse", 2541, 22, 4.6, 1.0).typed(569, CreatureType::Beast),
            foe("Skeletal Mage", 9783, 23, 4.2, 1.0).typed(203, CreatureType::Undead),
            foe("Young Black Ravager", 246, 24, 5.4, 1.0).typed(923, CreatureType::Beast),
            foe("Skeletal Horror", 9786, 24, 3.6, 1.0).typed(202, CreatureType::Undead),
            foe("Black Widow Hatchling", 368, 25, 4.6, 1.0).typed(930, CreatureType::Beast),
            foe("Black Ravager", 741, 25, 5.4, 1.0).typed(628, CreatureType::Beast),
            foe("Black Ravager Mastiff", 9562, 26, 5.4, 1.0).typed(1258, CreatureType::Beast),
            foe("Nightbane Worgen", 657, 27, 5.4, 1.0).typed(898, CreatureType::Humanoid),
            foe("Nightbane Shadow Weaver", 203, 28, 5.4, 1.0).typed(533, CreatureType::Humanoid),
            foe("Nightbane Dark Runner", 736, 29, 5.4, 1.0).typed(205, CreatureType::Humanoid),
            foe("Nightbane Vile Fang", 522, 30, 5.4, 1.0).typed(206, CreatureType::Humanoid),
            foe("Fetid Corpse", 559, 30, 3.6, 1.3).typed(1270, CreatureType::Undead),
            foe("Nightbane Tainted One", 1098, 31, 5.4, 1.0).typed(920, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Lupos", 11412, 23, 5.4, 1.9).typed(521, CreatureType::Beast),
            foe("Naraxis", 963, 27, 4.6, 1.9).typed(574, CreatureType::Beast),
            foe("Nightbane Vile Fang", 522, 30, 5.4, 1.9).typed(206, CreatureType::Humanoid),
            foe("Nightbane Tainted One", 1098, 31, 5.4, 1.9).typed(920, CreatureType::Humanoid),
            foe("Fenros", 11179, 32, 5.4, 1.9).typed(507, CreatureType::Humanoid),
            foe("Nefaru", 11181, 34, 5.4, 1.9).typed(534, CreatureType::Humanoid),
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
            foe("Redridge Thrasher", 489, 15, 4.2, 1.0).typed(712, CreatureType::Humanoid),
            foe("Redridge Mongrel", 487, 16, 4.2, 1.0).typed(423, CreatureType::Humanoid),
            foe("Tarantula", 366, 16, 4.2, 1.0).typed(442, CreatureType::Beast),
            foe("Great Goretusk", 3035, 17, 4.6, 1.0).typed(547, CreatureType::Beast),
            foe("Redridge Poacher", 551, 17, 4.2, 1.0).typed(424, CreatureType::Humanoid),
            foe("Murloc Shorestriker", 1305, 17, 4.2, 1.0).typed(1083, CreatureType::Humanoid),
            foe("Black Dragon Whelp", 387, 18, 4.2, 1.0).typed(441, CreatureType::Dragonkin),
            foe("Redridge Brute", 500, 18, 4.2, 1.0).typed(426, CreatureType::Humanoid),
            foe("Redridge Mystic", 10789, 19, 3.8, 1.0).typed(430, CreatureType::Humanoid),
            foe("Dire Condor", 490, 19, 5.2, 0.8).typed(428, CreatureType::Beast),
            foe("Redridge Basher", 609, 20, 4.2, 1.0).typed(446, CreatureType::Humanoid),
            foe("Greater Tarantula", 520, 20, 4.2, 1.0).typed(505, CreatureType::Beast),
            foe("Redridge Drudger", 551, 21, 4.2, 1.0).typed(580, CreatureType::Humanoid),
            foe("Blackrock Summoner", 6030, 23, 3.8, 1.0).typed(4463, CreatureType::Humanoid),
            foe("Blackrock Tracker", 6040, 24, 4.2, 1.0).typed(615, CreatureType::Humanoid),
            foe("Blackrock Champion", 6044, 25, 4.2, 1.0).typed(435, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Blackrock Hunter", 6031, 24, 4.2, 1.9).typed(4462, CreatureType::Humanoid),
            foe("Singe", 497, 24, 4.2, 1.9).typed(335, CreatureType::Dragonkin),
            foe("Tharil'zun", 6043, 24, 4.2, 1.9).typed(486, CreatureType::Humanoid),
            foe("Blackrock Gladiator", 6042, 25, 4.2, 1.9).typed(4464, CreatureType::Humanoid),
            foe("Rohh the Silent", 10792, 26, 4.2, 1.9).typed(947, CreatureType::Humanoid),
            foe("Kazon", 6041, 27, 4.2, 1.9).typed(584, CreatureType::Humanoid),
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
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3).typed(8530, CreatureType::Undead),
            foe("Jadefire Rogue", 8575, 52, 5.2, 1.0).typed(7106, CreatureType::Demon),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85).typed(8534, CreatureType::Undead),
            foe("Death Cultist", 10391, 56, 3.8, 1.1).typed(8547, CreatureType::Humanoid),
            foe("Felbeast", 7949, 50, 4.8, 0.85).typed(8675, CreatureType::Demon),
            foe("Torn Screamer", 984, 57, 5.0, 0.7).typed(8540, CreatureType::Undead),
            foe("Necromancer", 10427, 57, 3.6, 1.15).typed(8553, CreatureType::Humanoid),
            foe("Jadefire Felsworn", 11334, 54, 4.8, 1.1).typed(7109, CreatureType::Demon),
            foe("Felhound", 1913, 54, 5.2, 1.0).typed(6010, CreatureType::Demon),
            foe("Nerubian Overseer", 14698, 58, 4.4, 0.95).typed(16184, CreatureType::Undead),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9).typed(6011, CreatureType::Demon),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8).typed(8541, CreatureType::Undead),
            foe("Hederine Manastalker", 6172, 58, 5.4, 1.15).typed(7462, CreatureType::Demon),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0).typed(4676, CreatureType::Demon),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0).typed(8717, CreatureType::Demon),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25).typed(8543, CreatureType::Undead),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1).typed(12396, CreatureType::Demon),
        ],
        bosses: &[
            foe("Prince Xavalis", 11335, 57, 5.0, 1.85).typed(9877, CreatureType::Demon),
            foe("Lady Hederine", 10925, 61, 4.8, 2.7).typed(10201, CreatureType::Demon),
            giant("Taerar", 15363, 63, 4.8, 0.5).typed(14890, CreatureType::Dragonkin),
            giant("Lethon", 15365, 63, 4.8, 0.5).typed(14888, CreatureType::Dragonkin),
            giant("Emeriss", 15366, 63, 4.8, 0.53).typed(14889, CreatureType::Dragonkin),
            giant("Ysondre", 15364, 63, 4.8, 0.58).typed(14887, CreatureType::Dragonkin),
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
            foe("Scourge Soldier", 5231, 55, 4.4, 1.15).typed(8523, CreatureType::Undead),
            foe("Plagued Swine", 6121, 53, 5.0, 1.0).typed(16117, CreatureType::Beast),
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3).typed(8530, CreatureType::Undead),
            foe("Cursed Mage", 11397, 55, 3.6, 0.95).typed(8524, CreatureType::Undead),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85).typed(8534, CreatureType::Undead),
            foe("Scourge Warder", 612, 56, 4.4, 1.2).typed(8525, CreatureType::Undead),
            foe("Death Cultist", 10391, 56, 3.8, 1.1).typed(8547, CreatureType::Humanoid),
            foe("Frenzied Plaguehound", 9022, 57, 5.6, 1.4).typed(8598, CreatureType::Demon),
            foe("Torn Screamer", 984, 57, 5.0, 0.7).typed(8540, CreatureType::Undead),
            foe("Necromancer", 10427, 57, 3.6, 1.15).typed(8553, CreatureType::Humanoid),
            foe("Carrion Devourer", 7900, 57, 4.0, 1.25).typed(8605, CreatureType::Beast),
            foe("Crypt Walker", 11178, 58, 4.6, 0.85).typed(8556, CreatureType::Undead),
            foe("Putrid Shrieker", 7855, 58, 5.4, 1.0).typed(8535, CreatureType::Undead),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8).typed(8541, CreatureType::Undead),
            foe("Ziggurat Protector", 10627, 59, 4.4, 1.6).typed(12262, CreatureType::Undead),
            foe("Deathknight", 16508, 60, 4.6, 1.75).typed(16146, CreatureType::Undead),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25).typed(8543, CreatureType::Undead),
            giant("Slaughterhouse Protector", 7864, 60, 3.6, 1.3)
                .typed(12263, CreatureType::Undead),
            giant("Death Lord", 10729, 61, 5.0, 1.05).typed(16861, CreatureType::Undead),
        ],
        bosses: &[
            giant("Anub'Rekhan", 15931, 63, 4.6, 0.55).typed(15956, CreatureType::Undead),
            foe("Grand Widow Faerlina", 15940, 63, 4.8, 2.9).typed(15953, CreatureType::Humanoid),
            giant("Patchwerk", 16174, 63, 4.0, 1.7).typed(16028, CreatureType::Undead),
            giant("Gluth", 16064, 63, 5.0, 0.62).typed(15932, CreatureType::Undead),
            giant("Thaddius", 16137, 63, 4.0, 0.98).typed(15928, CreatureType::Undead),
            foe("Kel'Thuzad", 15945, 63, 4.4, 2.1).typed(15990, CreatureType::Undead),
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
            foe("Syndicate Highwayman", 2342, 32, 4.4, 1.0).typed(2586, CreatureType::Humanoid),
            foe("Drywhisker Kobold", 373, 33, 4.0, 0.9).typed(2572, CreatureType::Humanoid),
            foe("Plains Creeper", 958, 34, 4.2, 1.1).typed(2563, CreatureType::Beast),
            foe("Highland Strider", 3027, 35, 4.8, 1.0).typed(2559, CreatureType::Beast),
            foe("Dabyrie Militia", 4418, 36, 4.2, 1.0).typed(2581, CreatureType::Humanoid),
            foe("Witherbark Axe Thrower", 10706, 37, 4.4, 1.1).typed(2554, CreatureType::Humanoid),
            giant("Boulderfist Ogre", 1122, 38, 4.0, 1.4).typed(2562, CreatureType::Humanoid),
            giant("Rumbling Exile", 11548, 39, 3.6, 1.3).typed(2592, CreatureType::Elemental),
        ],
        bosses: &[
            foe("Or'Kalar", 10617, 42, 4.6, 2.2).typed(2773, CreatureType::Humanoid),
            foe("Lord Falconcrest", 383, 43, 4.4, 2.0).typed(2597, CreatureType::Humanoid),
            giant("Boulderfist Lord", 11549, 45, 4.2, 1.6).typed(2571, CreatureType::Humanoid),
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
            foe("Skeletal Warrior", 5231, 50, 4.4, 1.1).typed(48, CreatureType::Undead),
            foe("Skeletal Flayer", 10626, 51, 4.6, 1.2).typed(1783, CreatureType::Undead),
            foe("Plaguebat", 7854, 52, 5.2, 0.9).typed(8600, CreatureType::Beast),
            foe("Scarlet Knight", 4418, 53, 4.4, 1.15).typed(1833, CreatureType::Humanoid),
            foe("Rotting Cadaver", 1693, 54, 3.8, 1.2).typed(4474, CreatureType::Undead),
            foe("Blighted Zombie", 612, 55, 4.2, 1.1).typed(4475, CreatureType::Undead),
            giant("Flesh Golem", 7864, 56, 3.6, 1.35).typed(1805, CreatureType::Undead),
            giant("Crypt Fiend", 11178, 57, 4.6, 1.1).typed(8556, CreatureType::Undead),
        ],
        bosses: &[
            foe("The Hederine Slayer", 10729, 58, 4.8, 2.0).typed(7463, CreatureType::Demon),
            giant("Lord Maldazzar", 16508, 60, 4.4, 1.4).typed(1848, CreatureType::Humanoid),
            foe("Araj the Summoner", 10427, 61, 4.2, 2.1).typed(1852, CreatureType::Undead),
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
            foe("Longtooth Runner", 165, 40, 4.4, 1.0).typed(5286, CreatureType::Beast),
            foe("Ironfur Bear", 3201, 41, 4.4, 1.0).typed(5268, CreatureType::Beast),
            foe("Woodpaw Mongrel", 8598, 40, 4.4, 1.0).typed(5249, CreatureType::Humanoid),
            foe("Gordunni Ogre", 597, 40, 4.4, 1.0).typed(5229, CreatureType::Humanoid),
            foe("Gordunni Brute", 11554, 42, 4.4, 1.0).typed(5232, CreatureType::Humanoid),
            foe("Woodpaw Trapper", 10796, 41, 4.4, 1.0).typed(5251, CreatureType::Humanoid),
            foe("Feral Scar Yeti", 3207, 43, 4.4, 1.0).typed(5292, CreatureType::Humanoid),
            foe("Gordunni Warlock", 11560, 43, 4.4, 1.0).typed(5240, CreatureType::Humanoid),
            foe("Frayfeather Hippogryph", 3210, 43, 4.4, 1.0).typed(5300, CreatureType::Beast),
            foe("Woodpaw Mystic", 10794, 42, 4.4, 1.0).typed(5254, CreatureType::Humanoid),
            foe("Woodpaw Reaver", 10795, 42, 4.4, 1.0).typed(5255, CreatureType::Humanoid),
            foe("Rage Scar Yeti", 3220, 46, 4.4, 1.0).typed(5296, CreatureType::Humanoid),
            foe("Gordunni Mauler", 11557, 43, 4.4, 1.0).typed(5234, CreatureType::Humanoid),
            foe("Woodpaw Alpha", 10793, 43, 4.4, 1.0).typed(5258, CreatureType::Humanoid),
            foe("Wandering Forest Walker", 9592, 44, 4.4, 1.0).typed(7584, CreatureType::Elemental),
        ],
        bosses: &[
            foe("Snarler", 780, 42, 4.8, 1.8).typed(5356, CreatureType::Beast),
            foe("Old Grizzlegut", 706, 43, 4.8, 1.8).typed(5352, CreatureType::Beast),
            foe("Diamond Head", 1817, 45, 4.8, 1.8).typed(5345, CreatureType::Humanoid),
            foe("Qirot", 11142, 47, 4.8, 1.8).typed(5350, CreatureType::Unspecified),
            foe("Arash-ethis", 7569, 49, 4.8, 1.8).typed(5349, CreatureType::Beast),
            foe("Antilus the Soarer", 10889, 48, 4.8, 1.8).typed(5347, CreatureType::Beast),
            foe("Mushgog", 14382, 60, 4.8, 1.8).typed(11447, CreatureType::Elemental),
            foe("The Razza", 10807, 60, 4.8, 1.8).typed(11497, CreatureType::Beast),
            foe("Skarr the Unbreakable", 10169, 58, 4.8, 1.8).typed(11498, CreatureType::Humanoid),
        ],
    },
    MapDef {
        name: "Scarlet Monastery",
        zone: "Scarlet Monastery",
        map: 189,
        // Cathedral courtyard WMO floor at z=18.3377. Small candidate only;
        // traversal, camera and crowd acceptance remain gated in campaign.
        center: [950.0, 1399.0, 19.0],
        leash: 15.0,
        victory: "The Scarlet Crusade falls silent.",
        // Identities verified against pinned map-189 placements; order/tuning authored.
        foes: &[
            foe("Scarlet Sentry", 2520, 30, 4.4, 1.0).typed(4283, CreatureType::Humanoid),
            foe("Scarlet Torturer", 2607, 30, 4.4, 1.0).typed(4306, CreatureType::Humanoid),
            foe("Unfettered Spirit", 5430, 31, 4.4, 1.0).typed(4308, CreatureType::Undead),
            foe("Anguished Dead", 10256, 31, 4.4, 1.0).typed(6426, CreatureType::Undead),
            foe("Haunting Phantasm", 4629, 32, 4.4, 1.0).typed(6427, CreatureType::Undead),
            foe("Scarlet Gallant", 2463, 33, 4.4, 1.0).typed(4287, CreatureType::Humanoid),
            foe("Scarlet Adept", 5725, 33, 4.4, 1.0).typed(4296, CreatureType::Humanoid),
            foe("Scarlet Beastmaster", 2497, 34, 4.4, 1.0).typed(4288, CreatureType::Humanoid),
            foe("Scarlet Tracking Hound", 2709, 33, 4.4, 1.0).typed(4304, CreatureType::Beast),
            foe("Scarlet Diviner", 2508, 34, 4.4, 1.0).typed(4291, CreatureType::Humanoid),
            foe("Scarlet Soldier", 2522, 35, 4.4, 1.0).typed(4286, CreatureType::Humanoid),
            foe("Scarlet Conjuror", 2504, 35, 4.4, 1.0).typed(4297, CreatureType::Humanoid),
            foe("Scarlet Evoker", 2509, 36, 4.4, 1.0).typed(4289, CreatureType::Humanoid),
            foe("Scarlet Myrmidon", 2515, 37, 4.4, 1.0).typed(4295, CreatureType::Humanoid),
            foe("Scarlet Defender", 2462, 37, 4.4, 1.0).typed(4298, CreatureType::Humanoid),
            foe("Scarlet Centurion", 2499, 38, 4.4, 1.0).typed(4301, CreatureType::Humanoid),
            foe("Scarlet Champion", 2460, 39, 4.4, 1.0).typed(4302, CreatureType::Humanoid),
            foe("Scarlet Abbot", 2492, 39, 4.4, 1.0).typed(4303, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Interrogator Vishas", 2044, 32, 4.4, 1.6).typed(3983, CreatureType::Humanoid),
            foe("Bloodmage Thalnos", 11396, 34, 4.4, 1.6).typed(4543, CreatureType::Undead),
            foe("Houndmaster Loksey", 2040, 34, 4.4, 1.6).typed(3974, CreatureType::Humanoid),
            foe("Arcanist Doan", 5266, 37, 4.4, 1.6).typed(6487, CreatureType::Humanoid),
            foe("Herod", 2041, 40, 4.4, 1.6).typed(3975, CreatureType::Humanoid),
            foe("High Inquisitor Fairbanks", 2605, 40, 4.4, 1.6).typed(4542, CreatureType::Undead),
            foe("Scarlet Commander Mograine", 2042, 42, 4.4, 1.6)
                .typed(3976, CreatureType::Humanoid),
            foe("High Inquisitor Whitemane", 2043, 42, 4.4, 1.6)
                .typed(3977, CreatureType::Humanoid),
        ],
    },
    MapDef {
        name: "Warsong Gulch",
        zone: "Warsong Gulch",
        map: 489,
        // Surveyed 60-yard midpoint disk. Gameplay traversal/camera acceptance remains gated.
        center: [1244.792, 1468.75, 310.296],
        leash: 60.0,
        victory: "The battle for the gulch is won.",
        // Faction-themed Survivors encounters; not vanilla map-489 NPC spawns.
        foes: &[
            foe("Warsong Peon", 11857, 26, 3.6, 1.0).typed(11656, CreatureType::Humanoid),
            foe("Silverwing Warrior", 4841, 21, 4.0, 1.0).typed(12897, CreatureType::Humanoid),
            foe("Warsong Scout", 11869, 28, 4.6, 1.0).typed(12862, CreatureType::Humanoid),
            foe("Silverwing Sentinel", 4842, 22, 4.2, 1.0).typed(12896, CreatureType::Humanoid),
            foe("Warsong Runner", 11872, 29, 4.8, 1.0).typed(12863, CreatureType::Humanoid),
            foe("Warsong Shaman", 11865, 28, 4.0, 1.0).typed(11683, CreatureType::Humanoid),
            foe("Warsong Grunt", 11861, 29, 4.2, 1.0).typed(11682, CreatureType::Humanoid),
            foe("Warsong Outrider", 11871, 30, 4.8, 1.0).typed(12864, CreatureType::Humanoid),
            foe("Warsong Shredder", 1303, 27, 3.8, 1.0).typed(11684, CreatureType::Mechanical),
            foe("Silverwing Elite", 14712, 55, 4.6, 1.0).typed(14715, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Warsong Scout Captain", 11869, 28, 4.6, 1.4).typed(12862, CreatureType::Humanoid),
            foe("Silverwing Vanguard", 4841, 30, 4.2, 1.4).typed(12897, CreatureType::Humanoid),
            foe("Warsong Shaman Elder", 11865, 32, 4.0, 1.4).typed(11683, CreatureType::Humanoid),
            foe("Silverwing Sentinel Captain", 4842, 34, 4.4, 1.4)
                .typed(12896, CreatureType::Humanoid),
            foe("Warsong Shredder Foreman", 1303, 36, 3.8, 1.4)
                .typed(11684, CreatureType::Mechanical),
            foe("Warsong Grunt Champion", 11861, 38, 4.4, 1.4).typed(11682, CreatureType::Humanoid),
            foe("Silverwing Blade Champion", 14713, 40, 4.6, 1.4)
                .typed(14715, CreatureType::Humanoid),
            foe("Silverwing Field Marshal", 14712, 42, 4.6, 1.5)
                .typed(14715, CreatureType::Humanoid),
            foe("Warsong Gulch Warmaster", 11871, 44, 4.8, 1.5)
                .typed(12864, CreatureType::Humanoid),
        ],
    },
    MapDef {
        name: "Arathi Basin",
        zone: "Arathi Basin",
        map: 529,
        // Dry 40-yard terrain candidate near the stables; runtime acceptance stays gated.
        center: [1210.0, 1130.0, -54.705],
        leash: 40.0,
        victory: "The basin's last assault is broken.",
        // Reused faction archetypes, with authored survival waves rather than PvP objectives.
        foes: &[
            foe("Hammerfall Peon", 4030, 33, 3.6, 1.0).typed(2618, CreatureType::Humanoid),
            foe("Stromgarde Troll Hunter", 4139, 37, 4.0, 1.0).typed(2583, CreatureType::Humanoid),
            foe("Hammerfall Grunt", 4032, 34, 4.2, 1.0).typed(2619, CreatureType::Humanoid),
            foe("Stromgarde Defender", 4137, 38, 4.0, 1.0).typed(2584, CreatureType::Humanoid),
            foe("Stromgarde Cavalryman", 4145, 36, 4.8, 1.0).typed(2738, CreatureType::Humanoid),
            foe("Stromgarde Vindicator", 4143, 39, 4.4, 1.0).typed(2585, CreatureType::Humanoid),
            foe("Hammerfall Guardian", 4034, 50, 4.2, 1.0).typed(2621, CreatureType::Humanoid),
            foe("League of Arathor Elite", 15280, 40, 4.6, 1.0)
                .typed(15130, CreatureType::Humanoid),
            foe("Hammerfall Elite", 4034, 55, 4.4, 1.0).typed(15136, CreatureType::Humanoid),
            foe("Defiler Elite", 15284, 55, 4.6, 1.0).typed(15128, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Defiler Farm Raider", 4032, 36, 4.2, 1.4).typed(2619, CreatureType::Humanoid),
            foe("Arathor Stable Captain", 4137, 38, 4.2, 1.4).typed(2584, CreatureType::Humanoid),
            foe("Defiler Mine Overseer", 4034, 40, 4.0, 1.4).typed(2621, CreatureType::Humanoid),
            foe("Arathor Lumber Champion", 4143, 42, 4.4, 1.4).typed(2585, CreatureType::Humanoid),
            foe("Defiler Blacksmith Champion", 15284, 44, 4.2, 1.4)
                .typed(15128, CreatureType::Humanoid),
            foe("Arathor Cavalry Captain", 4145, 46, 4.8, 1.4).typed(2738, CreatureType::Humanoid),
            foe("Defiler Siege Commander", 15286, 48, 4.4, 1.4)
                .typed(15128, CreatureType::Humanoid),
            foe("Arathor Field Marshal", 15282, 50, 4.6, 1.5).typed(15130, CreatureType::Humanoid),
            foe("Defiler Basin Warmaster", 15284, 52, 4.6, 1.5)
                .typed(15128, CreatureType::Humanoid),
        ],
    },
    MapDef {
        name: "Alterac Valley",
        zone: "Alterac Valley",
        map: 30,
        // Surveyed central terrain; world acceptance remains gated separately.
        center: [-300.0, -300.0, 7.668],
        leash: 60.0,
        victory: "The valley's final assault is defeated.",
        // Vanilla faction identities in an authored survival sequence.
        foes: &[
            foe("Frostwolf", 10278, 50, 4.8, 1.0).typed(10981, CreatureType::Beast),
            foe("Alterac Ram", 13340, 50, 4.4, 1.0).typed(10990, CreatureType::Beast),
            foe("Whitewhisker Vermin", 371, 52, 4.0, 1.0).typed(10982, CreatureType::Humanoid),
            foe("Irondeep Miner", 556, 52, 4.0, 1.0).typed(13396, CreatureType::Humanoid),
            foe("Irondeep Peon", 13335, 52, 4.0, 1.0).typed(13397, CreatureType::Humanoid),
            foe("Stormpike Guardsman", 13253, 57, 4.2, 1.0).typed(12127, CreatureType::Humanoid),
            foe("Stormpike Defender", 13274, 58, 4.2, 1.0).typed(12050, CreatureType::Humanoid),
            foe("Frostwolf Guardian", 12165, 58, 4.2, 1.0).typed(12053, CreatureType::Humanoid),
            foe("Stormpike Bowman", 13390, 59, 4.4, 1.0).typed(13358, CreatureType::Humanoid),
            foe("Frostwolf Bowman", 13398, 59, 4.4, 1.0).typed(13359, CreatureType::Humanoid),
        ],
        bosses: &[
            foe("Lieutenant Stronghoof", 13468, 59, 4.2, 1.4).typed(13143, CreatureType::Humanoid),
            foe("Lieutenant Lewis", 13466, 59, 4.2, 1.4).typed(13147, CreatureType::Humanoid),
            foe("Lieutenant Grummus", 14692, 59, 4.3, 1.4).typed(13145, CreatureType::Undead),
            foe("Commander Malgor", 13463, 61, 4.3, 1.4).typed(13152, CreatureType::Humanoid),
            foe("Commander Louis Philips", 13465, 61, 4.3, 1.4)
                .typed(13154, CreatureType::Humanoid),
            foe("Captain Galvangar", 11895, 61, 4.5, 1.4).typed(11947, CreatureType::Humanoid),
            foe("Captain Balinda Stonehearth", 12823, 61, 4.2, 1.4)
                .typed(11949, CreatureType::Humanoid),
            foe("Vanndar Stormpike", 11896, 62, 4.4, 1.5).typed(11948, CreatureType::Humanoid),
            foe("Drek'Thar", 11894, 62, 4.4, 1.5).typed(11946, CreatureType::Humanoid),
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
                template_entry: f.template_entry,
                creature_type: f.creature_type,
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
