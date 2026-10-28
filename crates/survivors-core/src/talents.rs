//! Survivors talent effects and dependency rules, independent of engine spell visuals.

mod critical_cast;
mod druid;
mod hunter;
mod mage;
mod natures_grace;
mod priest;
pub use priest::{BlessedRecovery, PowerInfusionClock};
mod rogue;
pub use critical_cast::CriticalCastClock;
pub use hunter::HawkClock;
pub use hunter::{PetBonus, PetFrenzy};
pub use mage::{ArcanePowerClock, ColdSnapClock, CombustionClock};
pub use natures_grace::NaturesGrace;
pub use rogue::{AdrenalineRushClock, BladeFlurryClock, PreparationClock};
mod shaman;
mod warlock;

/// One catalog node and its proposed rank, for validating a single talent tree.
#[derive(Clone, Copy, Debug)]
pub struct RankedTalent {
    pub id: u32,
    pub row: u32,
    pub max_rank: u8,
    pub rank: u8,
    /// Required rank is one-based; DBC's zero-based prerequisite rank is converted by the adapter.
    pub prerequisite: Option<(u32, u32)>,
}

/// Validate the result of a spend or refund. Only earlier rows support a row gate;
/// neither that row's own ranks nor points in deeper rows can hold it open.
pub fn valid_tree(nodes: &[RankedTalent]) -> bool {
    nodes.iter().enumerate().all(|(index, node)| {
        if nodes[..index].iter().any(|other| other.id == node.id) || node.rank > node.max_rank {
            return false;
        }
        if node.rank == 0 {
            return true;
        }
        let earlier: u32 = nodes
            .iter()
            .filter(|other| other.row < node.row)
            .map(|other| u32::from(other.rank))
            .sum();
        if earlier < node.row.saturating_mul(5) {
            return false;
        }
        node.prerequisite.is_none_or(|(id, required)| {
            required > 0
                && id != node.id
                && nodes
                    .iter()
                    .any(|other| other.id == id && u32::from(other.rank) >= required)
        })
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Milestones {
    pub class: usize,
    pub points: [u32; 3],
    /// Bounded selected-spec ranks in the class catalog order; not a save format.
    pub training: [u8; 64],
    pub weapon: Option<crate::gear_loadout::MeleeWeapon>,
}

impl Default for Milestones {
    fn default() -> Self {
        Self {
            class: 0,
            points: [0; 3],
            training: [0; 64],
            weapon: None,
        }
    }
}

/// Per-target Winter's Chill. A refresh preserves stacks; expiry removes them all.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WintersChill {
    stacks: u8,
    remaining: f32,
}

impl WintersChill {
    pub fn apply(&mut self) {
        self.stacks = (self.stacks + 1).min(5);
        self.remaining = 15.0;
    }

    pub fn advance(&mut self, seconds: f32) {
        if seconds.is_finite() && seconds > 0.0 {
            self.remaining = (self.remaining - seconds).max(0.0);
            if self.remaining == 0.0 {
                self.stacks = 0;
            }
        }
    }

    pub fn critical_bonus(self) -> f32 {
        if self.remaining > 0.0 {
            f32::from(self.stacks) * 0.02
        } else {
            0.0
        }
    }
}

/// Per-enemy Shadow vulnerability, refreshed by qualifying damage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowWeaving {
    stacks: u8,
    remaining: f32,
}

impl ShadowWeaving {
    pub fn apply(&mut self) {
        self.stacks = (self.stacks + 1).min(5);
        self.remaining = 15.0;
    }

    pub fn advance(&mut self, seconds: f32) {
        if seconds.is_finite() && seconds > 0.0 {
            self.remaining = (self.remaining - seconds).max(0.0);
            if self.remaining == 0.0 {
                self.stacks = 0;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CastEffect {
    pub damage: f32,
    pub radius: f32,
    pub range: f32,
    pub range_yards: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub crit_damage: f32,
    pub extra_targets: usize,
    pub heal_fraction: f32,
    pub shield_fraction: f32,
    pub shield_seconds: f32,
    pub immune_seconds: f32,
    pub cooldown_advance: Option<(&'static str, f32)>,
    pub storm_fraction: f32,
}

impl Default for CastEffect {
    fn default() -> Self {
        Self {
            damage: 1.0,
            radius: 1.0,
            range: 1.0,
            range_yards: 0.0,
            duration: 1.0,
            cooldown: 1.0,
            crit: 0.0,
            crit_damage: 0.0,
            extra_targets: 0,
            heal_fraction: 0.0,
            shield_fraction: 0.0,
            shield_seconds: 0.0,
            immune_seconds: 0.0,
            cooldown_advance: None,
            storm_fraction: 0.0,
        }
    }
}

impl Milestones {
    pub fn with_ranks(mut self, ranks: &[(u8, u32, u8)]) -> Self {
        self.training.fill(0);
        for (index, node) in definitions(self.class).iter().enumerate() {
            self.training[index] = bounded_rank(node, ranks);
        }
        self
    }

    fn paladin(self, tree: usize, threshold: u32) -> bool {
        self.class == 1 && self.points[tree] >= threshold
    }

    pub fn cast(self, key: &str) -> CastEffect {
        let mut effect = CastEffect::default();
        let mut healing_multiplier = 1.0;
        if self.class == 3 && ROGUE_SPELLS.contains(&key) {
            if self.points[2] >= 10 && key == "gouge" {
                effect.shield_fraction = 0.03;
            }
            if self.points[2] >= 20 && key == "kidney_shot" {
                effect.cooldown_advance = Some(("gouge", 2.0));
            }
            if self.points[2] >= 30 && key == "gouge" {
                effect.cooldown_advance = Some(("eviscerate", 1.0));
            }
            if self.points[0] >= 10 {
                effect.crit += 0.05;
            }
            if self.points[0] >= 20 && key != "rupture" {
                effect.crit_damage += 0.20;
            }
            if self.points[0] >= 30 && key == "eviscerate" {
                effect.crit = 1.0;
            }
            if self.points[1] >= 10 && key == "sinister_strike" {
                effect.cooldown *= 0.9;
            }
            if self.points[1] >= 20 && key == "blade_flurry" {
                effect.radius *= 1.3;
            }
            if self.points[1] >= 30 {
                effect.cooldown *= 0.85;
            }
        }
        if self.class == 4 {
            if key == "power_word_shield" {
                if self.points[0] >= 10 {
                    effect.immune_seconds = 1.0;
                }
                if self.points[0] >= 30 {
                    effect.cooldown_advance = Some(("mind_blast", 2.0));
                }
            }
            if self.points[0] >= 20 && matches!(key, "power_word_shield" | "psychic_scream") {
                effect.cooldown *= 0.85;
            }
            if self.points[1] >= 10 && PRIEST_HOLY.contains(&key) {
                effect.crit += 0.05;
            }
            if self.points[1] >= 20 && key == "holy_fire" {
                effect.heal_fraction += 0.03;
            }
            if self.points[1] >= 30 && key == "holy_nova" {
                effect.heal_fraction += 0.05;
            }
            if key == "mind_blast" {
                if self.points[2] >= 10 {
                    effect.extra_targets += 1;
                }
                if self.points[2] >= 20 {
                    effect.cooldown_advance = Some(("shadow_word_pain", 2.0));
                }
            }
            if self.points[2] >= 30 && key == "psychic_scream" {
                effect.shield_fraction = 0.05;
            }
        }
        if self.class == 5 {
            let lightning = shaman::LIGHTNING.contains(&key);
            let shock = shaman::SHOCKS.contains(&key);
            let totem = shaman::TOTEMS.contains(&key);
            if self.points[0] >= 10 && (lightning || shock) {
                effect.cooldown *= 0.9;
            }
            if self.points[0] >= 20 && lightning {
                effect.crit += 0.10;
            }
            if self.points[0] >= 30 && key == "chain_lightning" {
                effect.crit = 1.0;
            }
            if self.points[1] >= 10 && key == "earth_shock" {
                effect.shield_fraction = 0.03;
            }
            if self.points[1] >= 20 && (totem || key == "lightning_shield") {
                effect.damage *= 1.2;
            }
            if self.points[1] >= 30 && key == "earth_shock" {
                effect.damage *= 1.25;
                effect.extra_targets += 1;
            }
            if self.points[2] >= 10 && totem {
                effect.heal_fraction = 0.03;
            }
            if self.points[2] >= 20 && shock {
                effect.cooldown *= 0.85;
            }
            if self.points[2] >= 30 && totem {
                effect.cooldown_advance = Some((
                    if key == "searing_totem" {
                        "magma_totem"
                    } else {
                        "searing_totem"
                    },
                    2.0,
                ));
            }
        }
        if self.class == 6 {
            // Automatic Survivors adaptations of Arcane Focus, Presence of Mind and Arcane Power.
            if MAGE_ARCANE.contains(&key) {
                if self.points[0] >= 10 {
                    effect.crit += 0.05;
                }
                if self.points[0] >= 30 {
                    effect.damage *= 1.2;
                }
            }
            if self.points[0] >= 20 && key == "arcane_missiles" {
                effect.cooldown_advance = Some(("arcane_explosion", 2.0));
            }
            if self.points[1] >= 10 && key == "fireball" {
                effect.damage *= 1.25;
            }
            if MAGE_FIRE.contains(&key) {
                if self.points[1] >= 20 {
                    effect.crit += 0.10;
                }
                if self.points[1] >= 30 {
                    effect.crit_damage += 0.20;
                }
            }
            if self.points[2] >= 10 && key == "frostbolt" {
                effect.extra_targets += 1;
            }
            if self.points[2] >= 20 && MAGE_FROST.contains(&key) {
                effect.crit_damage += 0.2;
            }
            if self.points[2] >= 30 && key == "frost_nova" {
                effect.shield_fraction = 0.10;
                effect.shield_seconds = 2.0; // Added to the shared three-second absorb.
            }
        }
        if self.class == 7 {
            warlock::apply_milestones(self.points, key, &mut effect);
        }
        if self.class == 8 {
            druid::apply_milestones(self.points, key, &mut effect);
        }
        if self.class == 2 {
            if self.points[2] >= 10 && matches!(key, "serpent_sting" | "explosive_trap") {
                effect.damage *= 1.2;
            }
            if self.points[2] >= 20 && key == "concussive_shot" {
                effect.shield_fraction = 0.05;
            }
        }
        if self.class == 2 && HUNTER_SHOTS.contains(&key) {
            if self.points[1] >= 10 {
                effect.cooldown *= 0.9;
            }
            if self.points[1] >= 20 {
                effect.crit_damage += 0.2;
            }
            if self.points[1] >= 30 {
                if key == "multi_shot" {
                    effect.extra_targets += 2;
                }
                if key == "aimed_shot" {
                    effect.damage *= 1.25;
                }
            }
        }
        match key {
            "retaliation" if self.class == 0 => {
                if self.points[1] >= 20 {
                    effect.damage = 1.30;
                }
                if self.points[2] >= 10 {
                    effect.heal_fraction = 0.10;
                }
            }
            "heroic_strike" if self.class == 0 && self.points[2] >= 30 => {
                effect.shield_fraction = 0.03;
            }
            "execute" if self.class == 0 && self.points[1] >= 30 => {
                effect.heal_fraction = 0.03;
            }
            "sweeping_strikes" if self.class == 0 && self.points[0] >= 20 => {
                effect.radius = 1.30;
            }
            "mortal_strike" if self.class == 0 && self.points[0] >= 30 => {
                effect.damage = 1.25;
            }
            "holy_shock" => {
                if self.paladin(0, 10) {
                    effect.crit = 0.20;
                }
                if self.paladin(0, 20) {
                    effect.heal_fraction = 0.03;
                }
                if self.paladin(0, 30) {
                    effect.cooldown = 0.80;
                }
            }
            "consecration" if self.paladin(1, 10) => {
                effect.radius = 1.25;
                effect.duration = 1.25;
            }
            "hammer_of_justice" if self.paladin(1, 30) => effect.extra_targets = 2,
            "judgement" => {
                if self.paladin(2, 10) {
                    effect.damage = 1.25;
                }
                if self.paladin(2, 20) {
                    effect.cooldown_advance = Some(("hammer_of_wrath", 1.0));
                }
                if self.paladin(2, 30) {
                    effect.storm_fraction = 0.5;
                }
            }
            _ => {}
        }
        {
            for (index, node) in definitions(self.class).iter().enumerate() {
                let value = node.per_rank * f32::from(self.training[index].min(node.max_rank));
                match node.stat {
                    TalentStat::PresenceOfMind if key == "arcane_missiles" && value > 0. => {
                        let milestone = effect.cooldown_advance.map_or(0., |(_, seconds)| seconds);
                        effect.cooldown_advance = Some(("arcane_explosion", milestone + value));
                    }
                    TalentStat::Conflagrate if key == "immolate" && value > 0. => {
                        let milestone = effect.cooldown_advance.map_or(0., |(_, seconds)| seconds);
                        effect.cooldown_advance = Some(("shadowburn", milestone + value));
                    }
                    TalentStat::FlameThrowing if MAGE_FIRE.contains(&key) => {
                        effect.range *= 1.0 + value
                    }
                    TalentStat::ArcticReach => {
                        if matches!(key, "frostbolt" | "blizzard") {
                            effect.range *= 1.0 + value;
                        }
                        if matches!(key, "frost_nova" | "cone_of_cold") {
                            effect.radius *= 1.0 + value;
                        }
                    }
                    TalentStat::TotemicMastery => {
                        if key == "searing_totem" {
                            effect.range *= 1.0 + value;
                        } else if key == "magma_totem" {
                            effect.radius *= 1.0 + value;
                        }
                    }
                    TalentStat::HolyReach => {
                        if PRIEST_CAST_HOLY.contains(&key) {
                            effect.range *= 1.0 + value;
                        }
                        if key == "holy_nova" {
                            effect.radius *= 1.0 + value;
                        }
                    }
                    TalentStat::ShadowReach if matches!(key, "shadow_word_pain" | "mind_blast") => {
                        effect.range *= 1.0 + value;
                    }
                    TalentStat::ArcaneInstability
                        if MAGE_ARCANE.contains(&key)
                            || MAGE_FIRE.contains(&key)
                            || MAGE_FROST.contains(&key) =>
                    {
                        effect.damage *= 1.0 + value;
                        effect.crit += value;
                    }
                    TalentStat::ForceOfWill
                        if PRIEST_HOLY.contains(&key) || PRIEST_SHADOW.contains(&key) =>
                    {
                        effect.damage *= 1.0 + value;
                        if key != "shadow_word_pain" {
                            effect.crit += value;
                        }
                    }
                    TalentStat::ElementalPrecision
                        if MAGE_FIRE.contains(&key) || MAGE_FROST.contains(&key) =>
                    {
                        effect.damage *= 1.0 + value;
                    }
                    TalentStat::SpellGroupRange(spells) if spells.contains(&key) => {
                        effect.range *= 1.0 + value;
                    }
                    TalentStat::SpellGroupRangeYards(spells) if spells.contains(&key) => {
                        effect.range_yards += value;
                    }
                    TalentStat::DaggerCrit
                        if self.weapon.is_some_and(|weapon| {
                            weapon.kind == crate::gear_loadout::MeleeKind::Dagger
                        }) && matches!(
                            key,
                            "sinister_strike"
                                | "eviscerate"
                                | "blade_flurry"
                                | "gouge"
                                | "kidney_shot"
                        ) =>
                    {
                        effect.crit += value;
                    }
                    TalentStat::FistCrit
                        if self.weapon.is_some_and(|weapon| {
                            weapon.kind == crate::gear_loadout::MeleeKind::Fist
                        }) && matches!(
                            key,
                            "sinister_strike"
                                | "eviscerate"
                                | "blade_flurry"
                                | "gouge"
                                | "kidney_shot"
                        ) =>
                    {
                        effect.crit += value;
                    }
                    TalentStat::RogueMace
                        if self.weapon.is_some_and(|weapon| {
                            weapon.kind == crate::gear_loadout::MeleeKind::Mace
                        }) && matches!(
                            key,
                            "sinister_strike"
                                | "eviscerate"
                                | "blade_flurry"
                                | "gouge"
                                | "kidney_shot"
                        ) =>
                    {
                        effect.damage *= 1.0 + value;
                    }
                    stat @ (TalentStat::TwoHandedDamage
                    | TalentStat::OneHandedDamage
                    | TalentStat::AxeCrit
                    | TalentStat::PolearmCrit)
                        if self.weapon.is_some_and(|weapon| match stat {
                            TalentStat::TwoHandedDamage => weapon.two_handed,
                            TalentStat::OneHandedDamage => !weapon.two_handed,
                            TalentStat::AxeCrit => {
                                weapon.kind == crate::gear_loadout::MeleeKind::Axe
                            }
                            TalentStat::PolearmCrit => {
                                weapon.kind == crate::gear_loadout::MeleeKind::Polearm
                            }
                            _ => false,
                        }) && matches!(
                            key,
                            "cleave"
                                | "heroic_strike"
                                | "whirlwind"
                                | "sweeping_strikes"
                                | "execute"
                                | "mortal_strike"
                                | "retaliation"
                        ) =>
                    {
                        if matches!(stat, TalentStat::AxeCrit | TalentStat::PolearmCrit) {
                            effect.crit += value;
                        } else {
                            effect.damage *= 1.0 + value;
                        }
                    }
                    TalentStat::SpellGroupDamage(spells) if spells.contains(&key) => {
                        effect.damage *= 1.0 + value;
                    }
                    TalentStat::SpellGroupCrit(spells) if spells.contains(&key) => {
                        effect.crit += value
                    }
                    TalentStat::SpellGroupCritDamage(spells) if spells.contains(&key) => {
                        effect.crit_damage += value;
                    }
                    TalentStat::SpellDamageAndCrit(spell) if spell == key => {
                        effect.damage *= 1.0 + value;
                        effect.crit += value;
                    }
                    TalentStat::SpellHealing(spell) if spell == key => {
                        effect.damage *= 1.0 + value;
                        healing_multiplier *= 1.0 + value;
                    }
                    TalentStat::SpellDamage(spell) | TalentStat::SpellAbsorb(spell)
                        if spell == key =>
                    {
                        effect.damage *= 1.0 + value
                    }
                    TalentStat::SpellGroupCooldown(spells) if spells.contains(&key) => {
                        effect.cooldown *= 1.0 - value
                    }
                    TalentStat::SpellCooldown(spell) if spell == key => {
                        effect.cooldown *= 1.0 - value
                    }
                    TalentStat::SpellCrit(spell) if spell == key => effect.crit += value,
                    TalentStat::BoomingVoice
                        if matches!(key, "battle_shout" | "demoralizing_shout") =>
                    {
                        effect.duration *= 1.0 + value;
                        if key == "demoralizing_shout" {
                            effect.radius *= 1.0 + value;
                        }
                    }
                    TalentStat::SpellDuration(spell) if spell == key => {
                        effect.duration *= 1.0 + value
                    }
                    TalentStat::SpellHeal(spell) if spell == key => effect.heal_fraction += value,
                    TalentStat::SpellShield(spell) if spell == key => {
                        effect.shield_fraction += value
                    }
                    TalentStat::SpellImmune(spell) if spell == key => {
                        effect.immune_seconds += value
                    }
                    TalentStat::SpellShieldDuration(spell) if spell == key => {
                        effect.shield_seconds += value
                    }
                    TalentStat::SpellRadius(spell) if spell == key => effect.radius *= 1.0 + value,
                    _ => {}
                }
            }
        }
        // Apply after all flat heals so definition order cannot bypass healing training.
        effect.heal_fraction *= healing_multiplier;
        if self.class == 5 && shaman::TOTEMS.contains(&key) {
            effect.heal_fraction *= 1.0 + self.trained_value(TalentStat::TotemHealing);
        }
        if self.class == 4 && key == "holy_nova" {
            effect.heal_fraction *= 1.0 + self.trained_value(TalentStat::SpiritualHealing);
        }
        if self.class == 5 && key == "earth_shock" {
            effect.shield_fraction *= 1.0 + self.trained_value(TalentStat::ShamanShieldAbsorption);
        }
        if effect.shield_fraction > 0.0 {
            // All talent-granted absorbs start with three seconds; duration ranks add to it.
            effect.shield_seconds += 3.0;
        } else {
            effect.shield_seconds = 0.0;
        }
        effect
    }

    /// Renew is a passive in Survivors. Other regeneration sources are not multiplied.
    pub fn passive_multiplier(self, key: &str) -> f32 {
        if self.class == 4 && key == "renew" {
            (1.0 + self.trained_value(TalentStat::ImprovedRenew))
                * (1.0 + self.trained_value(TalentStat::SpiritualHealing))
        } else {
            1.0
        }
    }

    /// Prevent flat damage after mitigation and before temporary shields.
    pub fn after_flat_absorption(self, damage: f32) -> f32 {
        (damage - self.trained_value(TalentStat::FlatAbsorption)).max(0.0)
    }

    /// Applied after flat gear, passive, renown and racial health contributions.
    pub fn health_multiplier(self) -> f32 {
        1.0 + self.trained_value(TalentStat::MaximumHealth)
    }

    pub fn inspiration_multiplier(self, active: bool) -> f32 {
        if self.class == 4 && active {
            1.0 + self.trained_value(TalentStat::InspirationArmor)
        } else {
            1.0
        }
    }

    pub fn ancestral_healing_armor(self, active: bool) -> f32 {
        if self.class == 5 && active {
            self.trained_value(TalentStat::AncestralHealingArmor)
        } else {
            0.
        }
    }

    fn trained_value(self, stat: TalentStat) -> f32 {
        definitions(self.class)
            .iter()
            .enumerate()
            .filter(|(_, node)| node.stat == stat)
            .map(|(index, node)| node.per_rank * f32::from(self.training[index].min(node.max_rank)))
            .sum()
    }

    /// Supplemental slows preserve the source spell's damage and other control.
    pub fn secondary_slow(self, key: &str, boss: bool) -> Option<(f32, f32)> {
        if self.class == 7 && key == "curse_of_agony" {
            let base = self.trained_value(TalentStat::CurseExhaustion);
            if base <= 0. {
                return None;
            }
            let speed = 1. - base - self.trained_value(TalentStat::ImprovedCurseExhaustion);
            return Some((if boss { speed.max(0.75) } else { speed }, 12.));
        }
        if self.class != 0 || key != "demoralizing_shout" {
            return None;
        }
        let duration = if self.points[1] >= 10 { 3.0 } else { 0.0 }
            + self.trained_value(TalentStat::SecondarySlowDuration("demoralizing_shout"));
        (duration > 0.).then_some((if boss { 0.75 } else { 0.5 }, duration))
    }

    pub fn secondary_stun(self, key: &'static str, boss: bool) -> f32 {
        let milestone = if (self.class == 0 && key == "heroic_strike" && self.points[2] >= 20)
            || (self.class == 2 && key == "serpent_sting" && self.points[2] >= 30)
        {
            1.0
        } else {
            0.0
        };
        let duration = milestone + self.trained_value(TalentStat::SecondaryStunDuration(key));
        duration * if boss { 0.3 } else { 1.0 }
    }

    pub fn secondary_root(self, key: &'static str, boss: bool) -> f32 {
        self.trained_value(TalentStat::SecondaryRootDuration(key)) * if boss { 0.3 } else { 1.0 }
    }

    /// Survivors Enrage: health damage refreshes a four-second damage bonus.
    pub fn enrage_damage(self, active: bool) -> f32 {
        if self.class == 0 && active {
            1.0 + self.trained_value(TalentStat::EnrageDamage)
        } else {
            1.0
        }
    }

    /// Survivors Blood Craze heals over six seconds after a nonlethal health-damaging hit.
    pub fn blood_craze_fraction(self) -> f32 {
        if self.class == 0 {
            self.trained_value(TalentStat::BloodCrazeHealing)
        } else {
            0.0
        }
    }

    pub fn spirit_tap_chance(self) -> f32 {
        if self.class == 4 {
            self.trained_value(TalentStat::SpiritTap)
        } else {
            0.0
        }
    }

    pub fn spirit_tap_regen(self) -> f32 {
        if self.spirit_tap_chance() > 0.0 {
            0.01
        } else {
            0.0
        }
    }

    /// Kill-triggered critical chance for the next successful Rogue melee cast.
    pub fn remorseless_crit(self, key: &str) -> f32 {
        if self.class == 3
            && matches!(
                key,
                "sinister_strike" | "eviscerate" | "gouge" | "kidney_shot"
            )
        {
            self.trained_value(TalentStat::Remorseless)
        } else {
            0.0
        }
    }

    /// Exposure belongs to Kidney Shot's own timer, not any other stun on the target.
    pub fn kidney_exposure_multiplier(self, remaining: f32) -> f32 {
        1.0 + if self.class == 3 && remaining > 0.0 {
            self.trained_value(TalentStat::KidneyExposure)
        } else {
            0.0
        }
    }

    pub fn vengeance_bonus(self, active: bool) -> f32 {
        if self.class == 1 && active {
            self.trained_value(TalentStat::Vengeance)
        } else {
            0.0
        }
    }

    pub fn triggers_vengeance(self, key: &str, damage: f32, critical: bool) -> bool {
        self.vengeance_bonus(true) > 0.0
            && critical
            && damage.is_finite()
            && damage > 0.0
            && key != "consecration"
            && PALADIN_HOLY.contains(&key)
    }

    pub fn flurry_haste(self, active: bool) -> f32 {
        if matches!(self.class, 0 | 5) && active {
            self.trained_value(TalentStat::FlurryHaste)
        } else {
            0.0
        }
    }

    pub fn triggers_flurry(self, key: &str, damage: f32, critical: bool) -> bool {
        self.flurry_haste(true) > 0.0
            && critical
            && damage.is_finite()
            && damage > 0.0
            && match self.class {
                0 => matches!(
                    key,
                    "cleave" | "heroic_strike" | "execute" | "mortal_strike"
                ),
                5 => shaman::LIGHTNING.contains(&key) || shaman::SHOCKS.contains(&key),
                _ => false,
            }
    }

    /// Death Wish's risk applies only during the actual Retaliation buff.
    pub fn retaliation_risk(self, active: bool) -> f32 {
        if self.class == 0 && self.points[1] >= 20 && active {
            1.10
        } else {
            1.0
        }
    }

    /// Sword Specialization: one same-target echo of direct weapon damage, never a proc chain.
    pub fn sword_extra_attack_chance(self, key: &str, damage: f32) -> f32 {
        let direct_weapon_hit = match self.class {
            0 => matches!(
                key,
                "cleave"
                    | "heroic_strike"
                    | "whirlwind"
                    | "sweeping_strikes"
                    | "execute"
                    | "mortal_strike"
                    | "retaliation"
            ),
            3 => matches!(
                key,
                "sinister_strike" | "eviscerate" | "blade_flurry" | "gouge" | "kidney_shot"
            ),
            _ => false,
        };
        if !direct_weapon_hit
            || !damage.is_finite()
            || damage <= 0.0
            || !self
                .weapon
                .is_some_and(|weapon| weapon.kind == crate::gear_loadout::MeleeKind::Sword)
        {
            return 0.0;
        }
        self.trained_value(TalentStat::SwordExtraAttack)
    }

    /// Mace Specialization uses the original proc table, including its 6% final rank.
    pub fn mace_stun_chance(self, key: &str, damage: f32) -> f32 {
        let eligible = match self.class {
            0 => matches!(
                key,
                "cleave"
                    | "heroic_strike"
                    | "whirlwind"
                    | "sweeping_strikes"
                    | "execute"
                    | "mortal_strike"
                    | "retaliation"
            ),
            3 => matches!(
                key,
                "sinister_strike" | "eviscerate" | "blade_flurry" | "gouge" | "kidney_shot"
            ),
            _ => false,
        };
        if !eligible
            || !damage.is_finite()
            || damage <= 0.0
            || !self
                .weapon
                .is_some_and(|w| w.kind == crate::gear_loadout::MeleeKind::Mace)
        {
            return 0.0;
        }
        let rank = definitions(self.class)
            .iter()
            .position(|node| matches!(node.stat, TalentStat::MaceStun | TalentStat::RogueMace))
            .map_or(0, |index| self.training[index]);
        f32::from(mace_stun_percent(rank)) / 100.0
    }

    /// Deep Wounds snapshots the milestone plus trained fraction of a critical direct attack.
    /// Called before execute and echo expansion; ticks cannot trigger more bleeds.
    pub fn critical_bleed(self, key: &str, damage: f32, critical: bool) -> f32 {
        if self.class != 0 || !critical || !damage.is_finite() {
            return 0.0;
        }
        if matches!(
            key,
            "cleave" | "heroic_strike" | "execute" | "mortal_strike"
        ) {
            let trained: f32 = definitions(self.class)
                .iter()
                .enumerate()
                .filter(|(_, node)| matches!(node.stat, TalentStat::CriticalBleed))
                .map(|(index, node)| {
                    node.per_rank * f32::from(self.training[index].min(node.max_rank))
                })
                .sum();
            let fraction = if self.points[0] >= 10 { 0.12 } else { 0.0 } + trained;
            damage.max(0.0) * fraction * self.cast("deep_wounds").damage
        } else {
            0.0
        }
    }

    /// Critical Mage fire hits seed a four-tick Ignite before multistrike expansion.
    /// The burn's own source is excluded, so periodic burn damage cannot recurse.
    pub fn ignite(self, key: &str, damage: f32, critical: bool) -> f32 {
        if self.class != 6
            || !critical
            || !damage.is_finite()
            || damage <= 0.0
            || !MAGE_FIRE.contains(&key)
        {
            return 0.0;
        }
        damage * self.trained_value(TalentStat::CriticalBurn)
    }

    /// Survivors conversion: each damaging Mage frost hit can freeze for five seconds.
    /// Periodic burns and other classes never trigger this proc.
    pub fn frostbite_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 6 || !damage.is_finite() || damage <= 0.0 || !MAGE_FROST.contains(&key) {
            return 0.0;
        }
        self.trained_value(TalentStat::Frostbite)
    }

    /// Improves only the acquired shout's magnitude, not the hero's global damage.
    pub fn battle_shout_multiplier(self) -> f32 {
        if self.class == 0 {
            1.0 + self.trained_value(TalentStat::BattleShoutPower)
        } else {
            1.0
        }
    }

    /// Survivors maps Hamstring's root proc to the Warrior's movement-impairing nova.
    pub fn hamstring_root_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 0 || key != "thunder_clap" || !damage.is_finite() || damage <= 0.0 {
            return 0.0;
        }
        self.trained_value(TalentStat::HamstringRoot)
    }

    /// Wing Clip roots on direct damaging hits; attached periodic ticks never roll.
    pub fn wing_clip_root_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 2 || key != "wing_clip" || !damage.is_finite() || damage <= 0.0 {
            return 0.0;
        }
        self.trained_value(TalentStat::WingClipRoot)
    }

    /// Fire hit proc; periodic Ignite ticks and non-Mage sources cannot trigger it.
    pub fn impact_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 6 || !damage.is_finite() || damage <= 0.0 || !MAGE_FIRE.contains(&key) {
            return 0.0;
        }
        self.trained_value(TalentStat::Impact)
    }

    /// Includes direct Shadow hits and periodic Shadow Word: Pain ticks.
    pub fn blackout_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 4 || !damage.is_finite() || damage <= 0.0 || !PRIEST_SHADOW.contains(&key)
        {
            return 0.0;
        }
        self.trained_value(TalentStat::Blackout)
    }

    /// Individual Nightfall rolls only for Corruption ticks that remove health.
    /// The adapter owns the RNG and readies only an already-owned Shadow Bolt.
    pub fn nightfall_chance(self, key: &str, removed: f32) -> f32 {
        if self.class != 7 || key != "corruption" || !removed.is_finite() || removed <= 0.0 {
            return 0.0;
        }
        self.trained_value(TalentStat::Nightfall)
    }

    /// Healing from health actually removed: Priest or Warlock training and milestones.
    pub fn vampiric_healing(self, key: &str, damage: f32) -> f32 {
        if self.class == 7
            && matches!(key, "corruption" | "curse_of_agony")
            && damage.is_finite()
            && damage > 0.0
        {
            let milestone = if self.points[0] >= 30 { 0.05 } else { 0.0 };
            return damage * (milestone + self.trained_value(TalentStat::SiphonLife));
        }
        if self.class != 4 || !damage.is_finite() || damage <= 0.0 || !PRIEST_SHADOW.contains(&key)
        {
            return 0.0;
        }
        let base = self.trained_value(TalentStat::VampiricEmbrace);
        if base <= 0.0 {
            return 0.0;
        }
        damage * (base + self.trained_value(TalentStat::ImprovedVampiricEmbrace))
    }

    pub fn shadow_weaving_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 4 || !damage.is_finite() || damage <= 0.0 || !PRIEST_SHADOW.contains(&key)
        {
            return 0.0;
        }
        self.trained_value(TalentStat::ShadowWeaving)
    }

    pub fn shadow_damage_multiplier(self, key: &str, target: ShadowWeaving) -> f32 {
        if self.class == 4 && PRIEST_SHADOW.contains(&key) && target.remaining > 0.0 {
            1.0 + 0.03 * f32::from(target.stacks)
        } else {
            1.0
        }
    }

    /// A positive frost hit can add one stack after its damage is resolved.
    pub fn winters_chill_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 6 || !damage.is_finite() || damage <= 0.0 || !MAGE_FROST.contains(&key) {
            return 0.0;
        }
        self.trained_value(TalentStat::WintersChill)
    }

    /// Conditional target bonuses are shared by direct casts and ground-area ticks.
    pub fn target_crit(self, key: &str, frozen: bool, chill: WintersChill) -> f32 {
        self.shatter(key, frozen)
            + if self.class == 6 && MAGE_FROST.contains(&key) {
                chill.critical_bonus()
            } else {
                0.0
            }
    }

    /// Only Mage spells benefit; ordinary roots are not frozen targets.
    pub fn shatter(self, key: &str, frozen: bool) -> f32 {
        if self.class != 6
            || !frozen
            || !(MAGE_ARCANE.contains(&key)
                || MAGE_FIRE.contains(&key)
                || MAGE_FROST.contains(&key))
        {
            return 0.0;
        }
        self.trained_value(TalentStat::Shatter)
    }

    /// Apply frost slow talents after the ability rank's duration scaling.
    /// These percentage-point reductions preserve the existing baseline Blizzard slow.
    pub fn control(self, key: &str, effect: crate::data::Effect) -> crate::data::Effect {
        use crate::data::Effect;
        if self.class == 8 && key == "insect_swarm" && matches!(effect, Effect::None) {
            let reduction = self.trained_value(TalentStat::InsectSwarmWeakening);
            if reduction > 0.0 {
                return Effect::Weaken(1.0 - reduction, 10.0);
            }
        }
        if key == "death_coil" {
            if let Effect::Drain(fraction) = effect {
                return Effect::Drain(
                    fraction * (1.0 + self.trained_value(TalentStat::ImprovedDrainLife)),
                );
            }
        }
        if self.class != 6 || !MAGE_FROST.contains(&key) {
            return effect;
        }
        let Effect::Slow(speed, seconds) = effect else {
            return effect;
        };
        let permafrost = self.trained_value(TalentStat::Permafrost);
        let blizzard = if key == "blizzard" {
            self.trained_value(TalentStat::BlizzardSlow)
        } else {
            0.0
        };
        Effect::Slow(
            (speed - permafrost - blizzard).max(0.1),
            seconds + permafrost / 0.03,
        )
    }

    /// Ardent Defender reduces the portion of a blow below 35% health, including a
    /// blow crossing that threshold. A large hit cannot skip the protection entirely.
    pub fn incoming(self, damage: f32, hp: f32, max_hp: f32) -> f32 {
        if !self.paladin(1, 20) {
            return damage;
        }
        let above = (hp - max_hp * 0.35).max(0.0).min(damage);
        above + (damage - above) * 0.8
    }
}

pub fn description(class: usize, tree: usize, threshold: u32) -> Option<&'static str> {
    if class == 8 {
        return druid::description(tree, threshold);
    }
    if class == 7 {
        return warlock::description(tree, threshold);
    }
    if class == 3 {
        return match (tree, threshold) {
            (0, 10) => Some("Rogue spells: +5 percentage points critical chance; periodic ticks do not crit"),
            (0, 20) => Some("Rogue direct attacks: +20 percentage points critical damage; excludes Rupture ticks"),
            (0, 30) => Some("Eviscerate always critically strikes (Survivors automatic Cold Blood)"),
            (1, 10) => Some("Sinister Strike: 10% shorter cooldown"),
            (1, 20) => Some("Blade Flurry: +30% radius; remains one area around the player"),
            (1, 30) => Some("Rogue spells: 15% shorter cooldown (Survivors automatic Adrenaline Rush)"),
            (2, 10) => Some("Gouge grants a 3% maximum-health absorb for 3 seconds once per successful cast; does not stack"),
            (2, 20) => Some("Kidney Shot advances Gouge's cooldown by 2 seconds once per successful cast"),
            (2, 30) => Some("Gouge advances Eviscerate's cooldown by 1 second once per successful cast"),
            _ => None,
        };
    }
    if class == 4 {
        return match (tree, threshold) {
            (0, 10) => Some("Power Word: Shield grants 1 second of immunity once per cast (Survivors Unbreakable Will)"),
            (0, 20) => Some("Power Word: Shield and Psychic Scream: 15% shorter cooldown (Survivors Mental Agility)"),
            (0, 30) => Some("Power Word: Shield advances Mind Blast's cooldown by 2 seconds once per cast (Survivors automatic Power Infusion)"),
            (1, 10) => Some("Smite, Holy Fire and Holy Nova: +5 percentage points critical chance; adds to individual Holy Specialization ranks"),
            (1, 20) => Some("Holy Fire heals 3% maximum health once per successful cast, not per target or burn tick (Survivors Spiritual Guidance)"),
            (1, 30) => Some("Holy Nova heals 5% maximum health once per successful cast, not per enemy (Survivors Lightwell healing pulse)"),
            (2, 10) => Some("Mind Blast: +1 target; area spells remain one area (Survivors Shadow Affinity)"),
            (2, 20) => Some("Mind Blast advances Shadow Word: Pain's cooldown by 2 seconds once per successful cast (Survivors Shadow Weaving)"),
            (2, 30) => Some("Psychic Scream grants a 5% maximum-health absorb for 3 seconds once per successful cast; does not stack (Survivors defensive Shadowform)"),
            _ => None,
        };
    }
    if class == 5 {
        return match (tree, threshold) {
            (0, 10) => Some("Lightning Bolt, Chain Lightning and shocks: 10% shorter cooldown (Survivors Convection)"),
            (0, 20) => Some("Lightning Bolt and Chain Lightning: +10 percentage points critical chance (Survivors Elemental Focus)"),
            (0, 30) => Some("Chain Lightning always critically strikes (Survivors automatic Elemental Mastery)"),
            (1, 10) => Some("Earth Shock grants a 3% maximum-health absorb for 3 seconds once per successful cast; does not stack (Survivors Ancestral Knowledge)"),
            (1, 20) => Some("Searing Totem, Magma Totem and Lightning Shield: +20% damage (Survivors Elemental Weapons)"),
            (1, 30) => Some("Earth Shock: +25% damage and +1 direct target (Survivors Stormstrike)"),
            (2, 10) => Some("Deploying Searing Totem or Magma Totem heals 3% maximum health once per cast, not per pulse (Survivors Improved Healing Wave)"),
            (2, 20) => Some("Earth, Flame and Frost Shock: 15% shorter cooldown, multiplying Convection (Survivors automatic Nature's Swiftness)"),
            (2, 30) => Some("Deploying Searing Totem advances an owned Magma Totem cooldown by 2 seconds, and vice versa, once per cast (Survivors Mana Tide Totem)"),
            _ => None,
        };
    }
    if class == 6 {
        return match (tree, threshold) {
            (0, 10) => Some("Arcane Missiles and Arcane Explosion: +5 percentage points critical chance (Survivors Arcane Focus)"),
            (0, 20) => Some("Arcane Missiles advances an owned Arcane Explosion's cooldown by 2 seconds once per successful cast (Survivors automatic Presence of Mind)"),
            (0, 30) => Some("Arcane Missiles and Arcane Explosion: +20% damage (Survivors automatic Arcane Power)"),
            (1, 10) => Some("Fireball: +25% damage; includes its splash (Survivors Improved Fireball milestone)"),
            (1, 20) => Some("Fire spells: +10 percentage points critical chance; adds to individual Critical Mass ranks"),
            (1, 30) => Some("Fire spells: +20 percentage points critical damage (Survivors automatic Combustion)"),
            (2, 10) => Some("Frostbolt: +1 target; area spells remain one area"),
            (2, 20) => Some("Frost spells: +20 percentage points critical damage"),
            (2, 30) => Some("Frost Nova grants a 10% maximum-health absorb for 5 seconds once per successful cast; does not stack"),
            _ => None,
        };
    }
    if class == 2 {
        return match (tree, threshold) {
            (0, 10) => Some("Combat wolf: +20% bite damage; does not affect Hunter shots or pickup companions"),
            (0, 20) => Some("Combat wolf: 20% shorter bite cooldown (from 2 to 1.6 sec; Survivors automatic Frenzy)"),
            (0, 30) => Some("Combat wolf: +50% bite damage for the first 10 sec of every 30 sec of run time; multiplies Unleashed Fury (Survivors automatic Bestial Wrath)"),
            (1, 10) => Some("Shots: 10% shorter cooldown; excludes Volley and traps"),
            (1, 20) => {
                Some("Shots: +20 percentage points critical damage; excludes Volley and traps")
            }
            (1, 30) => Some("Multi-Shot: +2 targets; Aimed Shot: +25% damage"),
            (2, 10) => Some("Serpent Sting and Explosive Trap: +20% direct and periodic damage"),
            (2, 20) => Some("Concussive Shot grants a 5% maximum-health absorb for 3 seconds once per successful cast; does not stack"),
            (2, 30) => Some("Serpent Sting also stuns for 1 sec (bosses: 0.3 sec), added to individual Wyvern Sting training"),
            _ => None,
        };
    }
    if class == 0 {
        return match (tree, threshold) {
            (0, 10) => Some(
                "Direct attack criticals bleed for 12% damage over 3 seconds; refreshes, does not stack",
            ),
            (0, 20) => Some("Sweeping Strikes: +30% radius"),
            (0, 30) => Some("Mortal Strike: +25% damage"),
            (1, 10) => {
                Some("Demoralizing Shout also slows enemies by 50% for 3 seconds (bosses: 25%)")
            }
            (1, 20) => {
                Some("Retaliation: +30% reflected damage; take 10% more damage while active")
            }
            (1, 30) => Some("Execute heals 3% maximum health once per successful cast"),
            (2, 10) => Some("Retaliation heals 10% maximum health once per successful cast"),
            (2, 20) => Some("Heroic Strike stuns for 1 second (bosses: 0.3 seconds)"),
            (2, 30) => Some(
                "Heroic Strike grants a 3% maximum-health absorb for 3 seconds, once per cast; does not stack",
            ),
            _ => None,
        };
    }
    if class != 1 {
        return None;
    }
    Some(match (tree, threshold) {
        (0, 10) => "Holy Shock: +20% critical chance",
        (0, 20) => "Holy Shock heals 3% maximum health once per cast",
        (0, 30) => "Holy Shock: 20% shorter cooldown",
        (1, 10) => "Consecration: +25% radius and duration",
        (1, 20) => "20% less damage below 35% health",
        (1, 30) => "Hammer of Justice: +2 targets",
        (2, 10) => "Judgement: +25% damage",
        (2, 20) => "Judgement advances Hammer of Wrath by 1 second",
        (2, 30) => "Judgement releases a 6-yard Divine Storm for 50% damage",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u32, row: u32, rank: u8) -> RankedTalent {
        RankedTalent {
            id,
            row,
            rank,
            max_rank: 5,
            prerequisite: None,
        }
    }

    #[test]
    fn refund_cannot_use_deeper_points_to_support_their_own_row() {
        let mut nodes = [node(1, 0, 5), node(2, 1, 5), node(3, 2, 1)];
        assert!(valid_tree(&nodes));
        nodes[0].rank = 4;
        assert!(!valid_tree(&nodes));
        // Eleven total points still do not unlock row two with only five below it.
        let invalid = [node(1, 0, 5), node(2, 2, 5), node(3, 2, 1)];
        assert!(!valid_tree(&invalid));
        // Remove the deepest rank first, then refund surplus earlier-row points.
        nodes[0].rank = 5;
        nodes[2].rank = 0;
        nodes[1].rank = 4;
        assert!(valid_tree(&nodes));
    }

    #[test]
    fn prerequisite_rank_and_catalog_caps_survive_refunds() {
        let mut nodes = [node(1, 0, 5), node(2, 0, 5), node(3, 1, 1)];
        nodes[2].max_rank = 1;
        nodes[2].prerequisite = Some((1, 5));
        assert!(valid_tree(&nodes));
        nodes[0].rank = 4;
        assert!(
            !valid_tree(&nodes),
            "enough row points cannot replace a prerequisite"
        );
        nodes[0].rank = 5;
        nodes[2].rank = 2;
        assert!(!valid_tree(&nodes));
        nodes[2].rank = 0;
        nodes[0].rank = 4;
        assert!(valid_tree(&nodes));
    }

    #[test]
    fn protection_thresholds_and_foreign_classes_are_isolated() {
        let at = |points| Milestones {
            class: 0,
            points: [0, 0, points],
            ..Default::default()
        };
        assert_eq!(at(9).cast("retaliation").heal_fraction, 0.0);
        assert_eq!(at(10).cast("retaliation").heal_fraction, 0.1);
        assert_eq!(at(19).secondary_stun("heroic_strike", false), 0.0);
        assert_eq!(at(20).secondary_stun("heroic_strike", false), 1.0);
        assert_eq!(at(20).secondary_stun("heroic_strike", true), 0.3);
        assert_eq!(at(30).secondary_stun("cleave", false), 0.0);
        assert_eq!(at(29).cast("heroic_strike").shield_fraction, 0.0);
        assert_eq!(at(30).cast("heroic_strike").shield_fraction, 0.03);
        assert_eq!(at(30).cast("heroic_strike").shield_seconds, 3.0);
        assert_eq!(at(30).cast("cleave").shield_fraction, 0.0);
        let other = Milestones { class: 1, ..at(30) };
        assert_eq!(other.cast("retaliation"), CastEffect::default());
        assert_eq!(other.cast("heroic_strike"), CastEffect::default());
        assert_eq!(other.secondary_stun("heroic_strike", false), 0.0);
        let mixed = Milestones {
            points: [0, 20, 10],
            ..at(0)
        };
        assert_eq!(mixed.cast("retaliation").damage, 1.3);
        assert_eq!(mixed.cast("retaliation").heal_fraction, 0.1);
        for threshold in [10, 20, 30] {
            assert!(description(0, 2, threshold).is_some());
        }
    }

    #[test]
    fn fury_thresholds_preserve_shout_targeting_and_limit_retaliation_risk() {
        let at = |points| Milestones {
            class: 0,
            points: [0, points, 0],
            ..Default::default()
        };
        assert_eq!(at(9).secondary_slow("demoralizing_shout", false), None);
        assert_eq!(
            at(10).secondary_slow("demoralizing_shout", false),
            Some((0.5, 3.0))
        );
        assert_eq!(
            at(10).secondary_slow("demoralizing_shout", true),
            Some((0.75, 3.0))
        );
        assert_eq!(at(30).secondary_slow("thunder_clap", false), None);
        assert_eq!(at(30).cast("demoralizing_shout").extra_targets, 0);
        assert_eq!(at(19).cast("retaliation").damage, 1.0);
        assert_eq!(at(20).cast("retaliation").damage, 1.3);
        assert_eq!(at(19).retaliation_risk(true), 1.0);
        assert_eq!(at(20).retaliation_risk(true), 1.1);
        assert_eq!(at(30).retaliation_risk(false), 1.0);
        assert_eq!(at(29).cast("execute").heal_fraction, 0.0);
        assert_eq!(at(30).cast("execute").heal_fraction, 0.03);
        let other = Milestones { class: 1, ..at(30) };
        assert_eq!(other.secondary_slow("demoralizing_shout", false), None);
        assert_eq!(other.retaliation_risk(true), 1.0);
        assert_eq!(other.cast("execute"), CastEffect::default());
        for threshold in [10, 20, 30] {
            assert!(description(0, 1, threshold).is_some());
        }
    }

    #[test]
    fn arms_milestones_open_at_thresholds_and_keep_one_area() {
        let at = |points| Milestones {
            class: 0,
            points: [points, 0, 0],
            ..Default::default()
        };
        assert_eq!(at(9).critical_bleed("cleave", 100.0, true), 0.0);
        assert_eq!(at(10).critical_bleed("cleave", 100.0, true), 12.0);
        assert_eq!(at(19).cast("sweeping_strikes").radius, 1.0);
        assert_eq!(at(20).cast("sweeping_strikes").radius, 1.3);
        assert_eq!(at(29).cast("mortal_strike").damage, 1.0);
        assert_eq!(at(30).cast("mortal_strike").damage, 1.25);
        assert_eq!(at(30).cast("sweeping_strikes").extra_targets, 0);
        assert_eq!(at(30).cast("fireball"), CastEffect::default());
        for (key, damage, crit) in [
            ("cleave", 100.0, false),
            ("deep_wounds", 100.0, true),
            ("Thorns", 100.0, true),
            ("fireball", 100.0, true),
            ("cleave", -10.0, true),
            ("cleave", f32::NAN, true),
        ] {
            assert_eq!(at(30).critical_bleed(key, damage, crit), 0.0);
        }
        let outsider = Milestones { class: 1, ..at(30) };
        assert_eq!(outsider.critical_bleed("cleave", 100.0, true), 0.0);
        for threshold in [10, 20, 30] {
            assert!(description(0, 0, threshold).is_some());
        }
    }

    #[test]
    fn rogue_subtlety_milestones_have_scoped_cast_triggers() {
        let at = |points| Milestones {
            class: 3,
            points: [0, 0, points],
            ..Default::default()
        };
        assert_eq!(at(9).cast("gouge").shield_fraction, 0.0);
        assert_eq!(at(10).cast("gouge").shield_fraction, 0.03);
        assert_eq!(at(10).cast("gouge").shield_seconds, 3.0);
        assert_eq!(at(19).cast("kidney_shot").cooldown_advance, None);
        assert_eq!(
            at(20).cast("kidney_shot").cooldown_advance,
            Some(("gouge", 2.0))
        );
        assert_eq!(at(29).cast("gouge").cooldown_advance, None);
        assert_eq!(
            at(30).cast("gouge").cooldown_advance,
            Some(("eviscerate", 1.0))
        );
        for key in [
            "eviscerate",
            "sinister_strike",
            "blade_flurry",
            "rupture",
            "fireball",
        ] {
            assert_eq!(at(30).cast(key), CastEffect::default());
        }
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let other = Milestones { class, ..at(30) };
            assert_eq!(other.cast("gouge"), CastEffect::default());
            assert_eq!(other.cast("kidney_shot"), CastEffect::default());
        }
        for threshold in [10, 20, 30] {
            assert!(description(3, 2, threshold).is_some());
        }
    }

    #[test]
    fn rogue_assassination_and_combat_milestones_unlock_and_compose() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        for tree in 0..2 {
            for threshold in [10, 20, 30] {
                let mut before = base;
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                assert!(ROGUE_SPELLS
                    .iter()
                    .any(|key| before.cast(key) != after.cast(key)));
                assert!(description(3, tree, threshold).is_some());
                assert_eq!(after.cast("fireball"), CastEffect::default());
                assert_eq!(after.cast("holy_shock"), CastEffect::default());
            }
        }
        let assassination = Milestones {
            points: [30, 0, 0],
            ..base
        };
        assert_eq!(assassination.cast("eviscerate").crit, 1.0);
        assert_eq!(assassination.cast("sinister_strike").crit, 0.05);
        assert_eq!(assassination.cast("sinister_strike").crit_damage, 0.20);
        assert_eq!(assassination.cast("rupture").crit_damage, 0.0);
        let combat = Milestones {
            points: [0, 30, 0],
            ..base
        };
        let trained = combat.with_ranks(&[(1, 201, 2), (0, 269, 5)]);
        assert!((trained.cast("sinister_strike").cooldown - 0.6885).abs() < 0.00001);
        assert!((trained.cast("sinister_strike").crit_damage - 0.30).abs() < 0.00001);
        for key in ROGUE_SPELLS {
            let effect = trained.cast(key);
            assert_eq!(effect.extra_targets, 0);
            assert_eq!(
                effect.radius,
                if *key == "blade_flurry" { 1.3 } else { 1.0 }
            );
            if *key != "sinister_strike" {
                assert_eq!(effect.cooldown, 0.85);
            }
        }
        assert_eq!(trained.with_ranks(&[]), combat);
    }

    #[test]
    fn hunter_marksmanship_milestones_open_at_thresholds_and_compose_with_training() {
        let at = |points| Milestones {
            class: 2,
            points: [0, points, 0],
            ..Default::default()
        };
        assert_eq!(at(9).cast("arcane_shot").cooldown, 1.);
        assert_eq!(at(10).cast("arcane_shot").cooldown, 0.9);
        assert_eq!(at(19).cast("aimed_shot").crit_damage, 0.);
        assert_eq!(at(20).cast("aimed_shot").crit_damage, 0.2);
        assert_eq!(at(29).cast("multi_shot").extra_targets, 0);
        assert_eq!(at(30).cast("multi_shot").extra_targets, 2);
        assert_eq!(at(29).cast("aimed_shot").damage, 1.);
        assert_eq!(at(30).cast("aimed_shot").damage, 1.25);
        for key in ["volley", "explosive_trap", "fireball", "holy_shock"] {
            assert_eq!(at(51).cast(key), CastEffect::default());
        }
        let trained = at(30).with_ranks(&[(1, 1346, 5), (1, 1349, 5), (1, 1345, 1)]);
        assert!((trained.cast("arcane_shot").cooldown - 0.675).abs() < 0.00001);
        assert!((trained.cast("aimed_shot").crit_damage - 0.5).abs() < 0.00001);
        assert_eq!(trained.cast("aimed_shot").damage, 1.5625);
        assert_eq!(trained.with_ranks(&[]), at(30));
        for tree in [0, 2] {
            let mut other = at(0);
            other.points[tree] = 51;
            assert_eq!(other.cast("multi_shot"), CastEffect::default());
        }
        for threshold in [10, 20, 30] {
            assert!(description(2, 1, threshold).is_some());
        }
    }

    #[test]
    fn hunter_survival_thresholds_compose_without_changing_target_geometry() {
        let at = |points| Milestones {
            class: 2,
            points: [0, 0, points],
            ..Default::default()
        };
        for key in ["serpent_sting", "explosive_trap"] {
            assert_eq!(at(9).cast(key).damage, 1.);
            assert_eq!(at(10).cast(key).damage, 1.2);
            assert_eq!(at(30).cast(key).extra_targets, 0);
            assert_eq!(at(30).cast(key).radius, 1.);
            assert_eq!(at(30).cast(key).duration, 1.);
        }
        assert_eq!(at(19).cast("concussive_shot").shield_fraction, 0.);
        assert_eq!(at(20).cast("concussive_shot").shield_fraction, 0.05);
        assert_eq!(at(20).cast("concussive_shot").shield_seconds, 3.);
        for boss in [false, true] {
            assert_eq!(at(29).secondary_stun("serpent_sting", boss), 0.);
            assert_eq!(
                at(30).secondary_stun("serpent_sting", boss),
                if boss { 0.3 } else { 1. }
            );
            let trained = at(30).with_ranks(&[(2, 1325, 1)]);
            assert_eq!(
                trained.secondary_stun("serpent_sting", boss),
                if boss { 0.75 } else { 2.5 }
            );
            assert_eq!(trained.secondary_stun("explosive_trap", boss), 0.);
            assert_eq!(trained.with_ranks(&[]), at(30));
        }
        let trained = at(30).with_ranks(&[(1, 1348, 5), (2, 1306, 2)]);
        assert!((trained.cast("serpent_sting").damage - 1.32).abs() < 0.00001);
        assert!((trained.cast("explosive_trap").damage - 1.56).abs() < 0.00001);
        for key in [
            "arcane_shot",
            "multi_shot",
            "aimed_shot",
            "volley",
            "fireball",
        ] {
            assert_eq!(at(51).cast(key), CastEffect::default());
        }
        for threshold in [10, 20, 30] {
            assert!(description(2, 2, threshold).is_some());
        }
    }

    #[test]
    fn each_paladin_milestone_opens_at_its_threshold() {
        for tree in 0..3 {
            for threshold in [10, 20, 30] {
                let mut before = Milestones {
                    class: 1,
                    points: [0; 3],
                    ..Default::default()
                };
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                let changed = [
                    "holy_shock",
                    "consecration",
                    "hammer_of_justice",
                    "judgement",
                ]
                .iter()
                .any(|key| before.cast(key) != after.cast(key))
                    || before.incoming(100.0, 20.0, 100.0) != after.incoming(100.0, 20.0, 100.0);
                assert!(changed, "tree {tree}, threshold {threshold}");
                assert!(description(1, tree, threshold).is_some());
            }
        }
    }

    #[test]
    fn effects_do_not_leak_to_other_classes_or_spells() {
        for class in [0, 2, 3, 4, 5, 6, 7, 8] {
            let m = Milestones {
                class,
                points: [51; 3],
                ..Default::default()
            };
            for key in [
                "holy_shock",
                "consecration",
                "hammer_of_justice",
                "judgement",
            ] {
                assert_eq!(m.cast(key), CastEffect::default());
            }
            assert_eq!(m.incoming(100.0, 20.0, 100.0), 100.0);
        }
        let m = Milestones {
            class: 1,
            points: [51; 3],
            ..Default::default()
        };
        assert_eq!(m.cast("fireball"), CastEffect::default());
        assert_eq!(m.cast("consecration").extra_targets, 0);
    }

    #[test]
    fn ardent_defender_splits_threshold_crossing_hits() {
        let m = Milestones {
            class: 1,
            points: [0, 20, 0],
            ..Default::default()
        };
        assert_eq!(m.incoming(10.0, 100.0, 100.0), 10.0);
        assert_eq!(m.incoming(20.0, 45.0, 100.0), 18.0);
        assert_eq!(m.incoming(20.0, 20.0, 100.0), 16.0);
    }
}

/// A supported individual talent. The listed bonus replaces the temporary 1% damage
/// per point for this node; tree milestones continue to count all spent points.
#[derive(Clone, Copy, Debug)]
pub struct TalentDefinition {
    pub id: u32,
    pub max_rank: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub stat: TalentStat,
    pub per_rank: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TalentStat {
    Health,
    MaximumHealth,
    FlatAbsorption,
    Damage,
    Cooldown,
    Armor,
    Crit,
    CritDamage,
    CriticalBleed,
    CriticalBurn,
    Frostbite,
    WingClipRoot,
    HamstringRoot,
    BattleShoutPower,
    BoomingVoice,
    InsectSwarmWeakening,
    NaturesGrace,
    Impact,
    Blackout,
    ShadowWeaving,
    VampiricEmbrace,
    ImprovedVampiricEmbrace,
    WintersChill,
    Shatter,
    Permafrost,
    BlizzardSlow,
    FlameThrowing,
    ArcticReach,
    HolyReach,
    ShadowReach,
    EnrageDamage,
    BloodCrazeHealing,
    BlessedRecovery,
    InspirationArmor,
    FlurryHaste,
    Vengeance,
    Remorseless,
    ColdBlood,
    ElementalMastery,
    InnerFocus,
    DivineFavor,
    KidneyExposure,
    Aftermath,
    Pyroclasm,
    CurseExhaustion,
    ImprovedCurseExhaustion,
    SpiritTap,
    ImprovedRenew,
    SpiritualHealing,
    TotemHealing,
    TotemicMastery,
    ShamanShieldAbsorption,
    AncestralHealingArmor,
    PetHealth,
    PetRecovery,
    PetIntimidation,
    PetWrath,
    SpiritBond,
    PetDamage,
    PetCrit,
    PetFrenzy,
    PetSpeed,
    PetCooldown,
    PetMend,
    ColdSnap,
    ArcanePower,
    Combustion,
    AdrenalineRush,
    BladeFlurry,
    ImprovedHawk,
    Deterrence,
    PowerInfusion,
    Preparation,
    SiphonLife,
    Nightfall,
    SecondarySlowDuration(&'static str),
    SecondaryStunDuration(&'static str),
    SecondaryRootDuration(&'static str),
    Reduction,
    Speed,
    ArcaneInstability,
    ForceOfWill,
    ElementalPrecision,
    SpellGroupDamage(&'static [&'static str]),
    TwoHandedDamage,
    OneHandedDamage,
    AxeCrit,
    PolearmCrit,
    DaggerCrit,
    FistCrit,
    SwordExtraAttack,
    MaceStun,
    RogueMace,
    SpellGroupCrit(&'static [&'static str]),
    SpellGroupCritDamage(&'static [&'static str]),
    SpellGroupCooldown(&'static [&'static str]),
    SpellGroupRange(&'static [&'static str]),
    SpellGroupRangeYards(&'static [&'static str]),
    SpellDamage(&'static str),
    SpellAbsorb(&'static str),
    SpellHealing(&'static str),
    ImprovedDrainLife,
    Conflagrate,
    PresenceOfMind,
    SpellDamageAndCrit(&'static str),
    SpellCooldown(&'static str),
    SpellCrit(&'static str),
    SpellDuration(&'static str),
    SpellHeal(&'static str),
    SpellShield(&'static str),
    SpellShieldDuration(&'static str),
    SpellImmune(&'static str),
    SpellRadius(&'static str),
}

/// Match complete stat phrases, longest first by length, rather than coloring
/// everything up to punctuation (which can include unrelated fixed effects).
fn inline_stat_label(tail: &str) -> &str {
    const LABELS: &[&str] = &[
        " maximum health per sec",
        " reflected damage",
        " retaliation damage",
        " bleed damage",
        " weakening duration",
        " Paladin Holy damage",
        " damage-reduction duration",
        " shorter bite cooldown",
        " shorter recovery after defeat",
        " direct critical chance",
        " regeneration",
        " reduced damage taken",
        " reduced outgoing damage",
        " more damage",
        " drain healing",
        " additional slow",
        " active duration",
        " shield absorption",
        " buff strength",
        " shorter deployment cooldown",
        " hero damage",
        " initial targeting range",
        " chance",
        " maximum health",
        " hero health",
        " health",
        " hero armor",
        " armor",
        " hero critical chance",
        " critical chance",
        " critical damage",
        " hero movement speed",
        " movement speed",
        " hero damage reduction",
        " damage reduction",
        " direct and periodic damage",
        " direct damage",
        " periodic damage",
        " damage",
        " healing",
        " shorter cooldown",
        " cooldown reduction",
        " cooldown",
        " pickup range",
        " targeting range",
        " casting range",
        " radius",
        " area",
        " root duration",
        " stun duration",
        " slow duration",
        " duration",
        " bite damage",
        " bite critical chance",
        " bite haste",
        " haste",
        " bite speed",
    ];
    LABELS
        .iter()
        .copied()
        .filter(|label| {
            tail.strip_prefix(label).is_some_and(|rest| {
                !rest
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
            })
        })
        .max_by_key(|label| label.len())
        .unwrap_or("")
}

// Spell.dbc procChance for 12284/12701–12704; rank five is deliberately non-linear.
fn mace_stun_percent(rank: u8) -> u8 {
    [0, 1, 2, 3, 4, 6][usize::from(rank.min(5))]
}

impl TalentDefinition {
    /// Inline this node's linear contribution; retain authored scope and caveats.
    /// Supported coupled effects share their rank scale; other compounds/procs retain a breakdown.
    pub fn inline_description(&self, rank: u8) -> Option<String> {
        self.inline_description_with_values(rank)
            .map(|(text, _)| text)
    }

    /// Byte ranges of changing values and units, for rich-text highlighting.
    pub fn inline_description_with_values(
        &self,
        rank: u8,
    ) -> Option<(String, Vec<std::ops::Range<usize>>)> {
        if self.stat == TalentStat::RogueMace {
            let rank = rank.min(self.max_rank);
            let (prefix, rest) = self.description.split_once("1–6% chance")?;
            let (middle, suffix) = rest.split_once("+1% direct damage per rank")?;
            let stun = if rank == self.max_rank {
                format!("{}% chance", mace_stun_percent(rank))
            } else {
                format!(
                    "{} » {}% chance",
                    mace_stun_percent(rank),
                    mace_stun_percent(rank + 1)
                )
            };
            let damage = if rank == self.max_rank {
                format!("+{rank}% direct damage")
            } else {
                format!("+{rank} » {}% direct damage", rank + 1)
            };
            let first = prefix.len()..prefix.len() + stun.len();
            let second_start = first.end + middle.len();
            return Some((
                format!("{prefix}{stun}{middle}{damage}{suffix}"),
                vec![first, second_start..second_start + damage.len()],
            ));
        }
        if self.stat == TalentStat::MaceStun {
            let rank = rank.min(self.max_rank);
            let (prefix, suffix) = self.description.split_once("1–6% chance")?;
            let current = mace_stun_percent(rank);
            let next = mace_stun_percent(rank.saturating_add(1));
            let value = if rank == self.max_rank {
                format!("{current}% chance")
            } else {
                format!("{current} » {next}% chance")
            };
            let start = prefix.len();
            let end = start + value.len();
            return Some((
                format!("{prefix}{value}{suffix}"),
                std::iter::once(start..end).collect(),
            ));
        }
        if self.stat == TalentStat::BladeFlurry {
            let rank = rank.min(self.max_rank);
            let (haste, targets) = if rank == 0 {
                ("0 » 20%", "0 » 1 additional target")
            } else {
                ("20%", "1 additional target")
            };
            let (prefix, rest) = self.description.split_once("20%")?;
            let (middle, suffix) = rest.split_once("1 additional target")?;
            let mut text = prefix.to_owned();
            let start = text.len();
            text.push_str(haste);
            let first = start..text.len();
            text.push_str(middle);
            let start = text.len();
            text.push_str(targets);
            let second = start..text.len();
            text.push_str(suffix);
            return Some((text, vec![first, second]));
        }
        if self.stat == TalentStat::Permafrost {
            let (prefix, tail) = self.description.split_once("1 sec")?;
            let token = format!(
                "{} additional percentage points per rank",
                (self.per_rank * 100.).round() as u32
            );
            let (middle, suffix) = tail.split_once(&token)?;
            let rank = rank.min(self.max_rank);
            let next = rank.saturating_add(1).min(self.max_rank);
            let value = |current: u32, next: u32, unit: &str| {
                if rank == self.max_rank {
                    format!("{current}{unit}")
                } else {
                    format!("{current} » {next}{unit}")
                }
            };
            let mut text = prefix.to_owned();
            let start = text.len();
            text.push_str(&value(u32::from(rank), u32::from(next), " sec"));
            let duration = start..text.len();
            text.push_str(middle);
            let start = text.len();
            text.push_str(&value(
                (self.per_rank * f32::from(rank) * 100.).round() as u32,
                (self.per_rank * f32::from(next) * 100.).round() as u32,
                " additional percentage points",
            ));
            let strength = start..text.len();
            text.push_str(suffix);
            return Some((text, vec![duration, strength]));
        }
        let (scale, unit) = match self.stat {
            TalentStat::Health
            | TalentStat::Armor
            | TalentStat::AncestralHealingArmor
            | TalentStat::FlatAbsorption
            | TalentStat::Preparation
            | TalentStat::ColdSnap => (1.0, ""),
            TalentStat::Conflagrate
            | TalentStat::NaturesGrace
            | TalentStat::PresenceOfMind
            | TalentStat::SpellShieldDuration(_)
            | TalentStat::SecondaryRootDuration(_)
            | TalentStat::SecondaryStunDuration(_)
            | TalentStat::SecondarySlowDuration(_)
            | TalentStat::SpellImmune(_) => (1.0, "time"),
            TalentStat::SpellGroupRangeYards(_) => (1.0, "distance"),
            TalentStat::Damage
            | TalentStat::ColdBlood
            | TalentStat::ElementalMastery
            | TalentStat::InnerFocus
            | TalentStat::ImprovedHawk
            | TalentStat::Vengeance
            | TalentStat::AdrenalineRush
            | TalentStat::BladeFlurry
            | TalentStat::Deterrence
            | TalentStat::Combustion
            | TalentStat::ArcanePower
            | TalentStat::PowerInfusion
            | TalentStat::DivineFavor
            | TalentStat::Aftermath
            | TalentStat::BattleShoutPower
            | TalentStat::BoomingVoice
            | TalentStat::HamstringRoot
            | TalentStat::WingClipRoot
            | TalentStat::InsectSwarmWeakening
            | TalentStat::CurseExhaustion
            | TalentStat::ImprovedCurseExhaustion
            | TalentStat::Pyroclasm
            | TalentStat::KidneyExposure
            | TalentStat::BlessedRecovery
            | TalentStat::InspirationArmor
            | TalentStat::MaximumHealth
            | TalentStat::Cooldown
            | TalentStat::Crit
            | TalentStat::CritDamage
            | TalentStat::Reduction
            | TalentStat::Speed
            | TalentStat::FlurryHaste
            | TalentStat::ShamanShieldAbsorption
            | TalentStat::PetHealth
            | TalentStat::PetRecovery
            | TalentStat::PetMend
            | TalentStat::PetCooldown
            | TalentStat::SpiritBond
            | TalentStat::PetDamage
            | TalentStat::PetCrit
            | TalentStat::PetFrenzy
            | TalentStat::FlameThrowing
            | TalentStat::ArcticReach
            | TalentStat::HolyReach
            | TalentStat::ArcaneInstability
            | TalentStat::ForceOfWill
            | TalentStat::TotemHealing
            | TalentStat::SpiritualHealing
            | TalentStat::ImprovedRenew
            | TalentStat::SpellDamage(_)
            | TalentStat::SpellDamageAndCrit(_)
            | TalentStat::SpellGroupDamage(_)
            | TalentStat::TwoHandedDamage
            | TalentStat::OneHandedDamage
            | TalentStat::AxeCrit
            | TalentStat::PolearmCrit
            | TalentStat::DaggerCrit
            | TalentStat::FistCrit
            | TalentStat::SwordExtraAttack
            | TalentStat::MaceStun
            | TalentStat::RogueMace
            | TalentStat::SpellCooldown(_)
            | TalentStat::SpellGroupCooldown(_)
            | TalentStat::SpellCrit(_)
            | TalentStat::SpellGroupCrit(_)
            | TalentStat::SpellGroupCritDamage(_)
            | TalentStat::SpellDuration(_)
            | TalentStat::SpellHeal(_)
            | TalentStat::SpellShield(_)
            | TalentStat::SpellAbsorb(_)
            | TalentStat::ImprovedDrainLife
            | TalentStat::SpellHealing(_)
            | TalentStat::SpellRadius(_)
            | TalentStat::SpellGroupRange(_) => (100.0, "%"),
            _ => return None,
        };
        let paired_ranges = matches!(self.stat, TalentStat::ArcticReach | TalentStat::HolyReach);
        let expected_clauses = if paired_ranges { 2 } else { 1 };
        let clauses = self.description.matches(" per rank").count();
        // One-rank unlocks naturally omit "per rank". Their typed bonus still
        // identifies the changing value; the candidate check below rejects
        // ambiguous repeated numbers instead of coloring unrelated constants.
        let single_rank = self.max_rank == 1 && clauses == 0 && !paired_ranges;
        if clauses != expected_clauses && !single_rank {
            return None;
        }
        let source = self.description.replace(" per rank", "");
        let amount = |rank| {
            let value = format!("{:.1}", self.per_rank * f32::from(rank) * scale);
            value.strip_suffix(".0").unwrap_or(&value).to_owned()
        };
        let units: &[&str] = match unit {
            "%" => &["%", " percentage point", " percentage points"],
            "time" => &[" sec", " second", " seconds"],
            "distance" => &[" yd"],
            _ => &[""],
        };
        let mut candidates = Vec::new();
        for &unit in units {
            let token = format!("{}{unit}", amount(1));
            for (index, _) in source.match_indices(&token) {
                let before = source[..index].chars().next_back();
                let after = source[index + token.len()..].chars().next();
                if !before.is_some_and(|c| c.is_ascii_digit() || c == '.')
                    && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '.')
                {
                    candidates.push((index, token.len(), unit));
                }
            }
        }
        let compound = matches!(self.stat, TalentStat::SpellDamageAndCrit(_));
        if compound {
            if candidates.len() != 2
                || candidates
                    .iter()
                    .filter(|(_, _, unit)| *unit == "%")
                    .count()
                    != 1
                || candidates
                    .iter()
                    .filter(|(_, _, unit)| unit.starts_with(" percentage point"))
                    .count()
                    != 1
            {
                return None;
            }
        } else if matches!(
            self.stat,
            TalentStat::ArcaneInstability
                | TalentStat::ForceOfWill
                | TalentStat::ArcticReach
                | TalentStat::HolyReach
        ) {
            if candidates.len() != 2 || candidates.iter().any(|(_, _, unit)| *unit != "%") {
                return None;
            }
        } else if candidates.len() != 1 {
            return None;
        }
        candidates.sort_by_key(|(index, _, _)| *index);
        let rank = rank.min(self.max_rank);
        let next = rank.saturating_add(1).min(self.max_rank);
        let mut text = String::new();
        let mut ranges = Vec::new();
        let mut cursor = 0;
        for (index, token_len, unit) in candidates {
            let unit = if unit.starts_with(" percentage point") {
                if amount(next) == "1" {
                    " percentage point"
                } else {
                    " percentage points"
                }
            } else if matches!(unit, " second" | " seconds") {
                if amount(next) == "1" {
                    " second"
                } else {
                    " seconds"
                }
            } else {
                unit
            };
            let value = if rank == self.max_rank {
                format!("{}{unit}", amount(rank))
            } else {
                format!("{} » {}{unit}", amount(rank), amount(next))
            };
            text.push_str(&source[cursor..index]);
            let start = text.len() - usize::from(source[..index].ends_with('+'));
            text.push_str(&value);
            cursor = index + token_len;
            // Keep the stat label with its changing number, without consuming
            // connective prose, fixed secondary bonuses or punctuation.
            let label = inline_stat_label(&source[cursor..]);
            text.push_str(label);
            cursor += label.len();
            ranges.push(start..text.len());
        }
        text.push_str(&source[cursor..]);
        Some((text, ranges))
    }

    /// This node's contribution only, before other talents, equipment and milestones.
    pub fn rank_preview(&self, rank: u8) -> String {
        let rank = rank.min(self.max_rank);
        if self.stat == TalentStat::RogueMace {
            return if rank == self.max_rank {
                format!(
                    "This talent: {}% stun chance; +{rank}% direct damage (maximum rank)",
                    mace_stun_percent(rank)
                )
            } else {
                format!(
                    "This talent: {} » {}% stun chance; +{rank} » {}% direct damage next rank",
                    mace_stun_percent(rank),
                    mace_stun_percent(rank + 1),
                    rank + 1
                )
            };
        }
        if self.stat == TalentStat::MaceStun {
            let current = mace_stun_percent(rank);
            let next = mace_stun_percent(rank.saturating_add(1));
            return if rank == self.max_rank {
                format!("This talent: {current}% stun chance (maximum rank)")
            } else {
                format!("This talent: {current} to {next}% stun chance next rank")
            };
        }
        if self.stat == TalentStat::Permafrost {
            let strength = |r| (self.per_rank * f32::from(r) * 100.0).round() as u32;
            return if rank == self.max_rank {
                format!("This talent: {} percentage points stronger slows; {rank} sec added duration (maximum rank)", strength(rank))
            } else {
                format!("This talent: {} » {} percentage points stronger slows; {rank} » {} sec added duration next rank", strength(rank), strength(rank + 1), rank + 1)
            };
        }
        if matches!(
            self.stat,
            TalentStat::ArcaneInstability
                | TalentStat::ForceOfWill
                | TalentStat::SpellDamageAndCrit(_)
        ) {
            let value = (self.per_rank * f32::from(rank) * 100.0).round() as u32;
            let next = (self.per_rank * f32::from(rank + 1) * 100.0).round() as u32;
            return if rank == self.max_rank {
                format!("This talent: {value}% damage; {value}% critical chance (maximum rank)")
            } else {
                format!(
                    "This talent: {value} » {}% damage; {value} » {}% critical chance next rank",
                    next, next
                )
            };
        }
        let (scale, unit) = match self.stat {
            TalentStat::ArcaneInstability
            | TalentStat::ForceOfWill
            | TalentStat::SpellDamageAndCrit(_) => {
                unreachable!("handled above")
            }
            TalentStat::ElementalPrecision => (100.0, "% Fire and Frost damage"),
            TalentStat::Health => (1.0, " maximum health"),
            TalentStat::MaximumHealth => (100.0, "% maximum health"),
            TalentStat::FlatAbsorption => (1.0, " damage prevented per hit"),
            TalentStat::Armor => (1.0, " armor"),
            TalentStat::SpellShield(_) => (100.0, "% maximum health absorbed per cast"),
            TalentStat::SpellImmune(_) => (1.0, " sec immunity per successful cast"),
            TalentStat::SpellShieldDuration(_) => (1.0, " s added shield duration"),
            TalentStat::SecondarySlowDuration(_) => (1.0, " s added slow duration"),
            TalentStat::SecondaryStunDuration(_) => (1.0, " s added stun duration"),
            TalentStat::SecondaryRootDuration(_) => (1.0, " s added root duration"),
            TalentStat::SpellAbsorb(_) => (100.0, "% absorption"),
            TalentStat::SpellHealing(_) => (100.0, "% healing"),
            TalentStat::ImprovedDrainLife => (100.0, "% drain healing"),
            TalentStat::Conflagrate => (1.0, " sec Shadowburn cooldown advanced per Immolate cast"),
            TalentStat::PresenceOfMind => (
                1.0,
                " sec Arcane Explosion cooldown advanced per Arcane Missiles cast",
            ),
            TalentStat::Damage | TalentStat::SpellDamage(_) | TalentStat::SpellGroupDamage(_)
            | TalentStat::TwoHandedDamage
            | TalentStat::OneHandedDamage => {
                (100.0, "% damage")
            }
            TalentStat::Cooldown
            | TalentStat::SpellCooldown(_)
            | TalentStat::SpellGroupCooldown(_) => (100.0, "% cooldown reduction"),
            TalentStat::SwordExtraAttack => (100.0, "% chance of an extra strike"),
            TalentStat::MaceStun | TalentStat::RogueMace => (100.0, "% stun chance"),
            TalentStat::Crit | TalentStat::SpellCrit(_) | TalentStat::SpellGroupCrit(_) | TalentStat::AxeCrit | TalentStat::PolearmCrit | TalentStat::DaggerCrit | TalentStat::FistCrit => {
                (100.0, "% critical chance")
            }
            TalentStat::CritDamage | TalentStat::SpellGroupCritDamage(_) => {
                (100.0, "% critical damage")
            }
            TalentStat::SpellGroupRange(_) => (100.0, "% targeting range"),
            TalentStat::SpellGroupRangeYards(_) => (1.0, " yd targeting range"),
            TalentStat::SpiritTap => (100.0, "% chance on kill"),
            TalentStat::ImprovedRenew => (100.0, "% Renew regeneration"),
            TalentStat::AncestralHealingArmor => (1.0, " armor after effective totem healing"),
            TalentStat::ShamanShieldAbsorption => (100.0, "% Earth Shock shield absorption"),
            TalentStat::TotemicMastery => (100.0, "% Searing range and Magma radius"),
            TalentStat::TotemHealing => (100.0, "% totem deployment healing"),
            TalentStat::SpiritualHealing => (100.0, "% Renew and Holy Nova healing"),
            TalentStat::PetWrath => (1.0, " upgraded Bestial Wrath window"),
            TalentStat::PetIntimidation => (1.0, " empowered bite every 60 sec"),
            TalentStat::PetRecovery => (100.0, "% shorter combat wolf recovery"),
            TalentStat::SpiritBond => (100.0, "% maximum health restored every 10 sec"),
            TalentStat::PetHealth => (100.0, "% combat wolf maximum health"),
            TalentStat::PetDamage => (100.0, "% combat wolf bite damage"),
            TalentStat::PetCrit => (100.0, " percentage points combat wolf critical chance"),
            TalentStat::PetFrenzy => (
                100.0,
                "% chance per wolf critical for 30% bite haste for 8 sec",
            ),
            TalentStat::ColdSnap => (1.0, " automatic frost reset every 90 sec"),
            TalentStat::PowerInfusion => {
                (100.0, "% Priest damage and healing during Power Infusion")
            }
            TalentStat::BladeFlurry => (100.0, "% melee recovery haste and one additional melee target during the burst"),
            TalentStat::AdrenalineRush => (
                100.0,
                "% shorter Rogue cast cooldown during Adrenaline Rush",
            ),
            TalentStat::ImprovedHawk => (100.0, "% Hunter shot haste during Improved Hawk"),
            TalentStat::Deterrence => (100.0, "% less incoming damage during Deterrence"),
            TalentStat::Combustion => (100.0, "% Fire critical chance per direct hit"),
            TalentStat::ArcanePower => (100.0, "% Mage damage during Arcane Power"),
            TalentStat::Preparation => (1.0, " automatic Rogue reset every 180 sec"),
            TalentStat::PetMend => (100.0, "% combat wolf maximum health restored per sec"),
            TalentStat::PetCooldown => (100.0, "% shorter combat wolf bite cooldown"),
            TalentStat::PetSpeed => (100.0, "% combat wolf movement speed"),
            TalentStat::Nightfall => (100.0, "% chance per Corruption tick"),
            TalentStat::SiphonLife => (100.0, "% Corruption and Curse of Agony damage healed"),
            TalentStat::InsectSwarmWeakening => {
                (100.0, "% reduced outgoing enemy damage from Insect Swarm")
            }
            TalentStat::NaturesGrace => (1.0, " sec shorter next Wrath or Starfire recovery after a spell critical; one 15 sec charge"),
            TalentStat::BoomingVoice => (100.0, "% shout duration and Demoralizing Shout radius"),
            TalentStat::BattleShoutPower => (100.0, "% stronger Battle Shout weapon bonus"),
            TalentStat::HamstringRoot => (100.0, "% chance to root Thunder Clap targets"),
            TalentStat::WingClipRoot => (100.0, "% chance to root Wing Clip targets"),
            TalentStat::Aftermath => (100.0, "% chance to slow Destruction targets"),
            TalentStat::CurseExhaustion => (100.0, "% Curse of Agony slow"),
            TalentStat::ImprovedCurseExhaustion => (100.0, "% additional curse slow"),
            TalentStat::Pyroclasm => (100.0, "% chance to stun fire-area targets"),
            TalentStat::KidneyExposure => (100.0, "% damage taken during Kidney Shot"),
            TalentStat::InnerFocus => (
                100.0,
                "% additional next-cast critical chance; 180 sec recovery",
            ),
            TalentStat::DivineFavor => {
                (100.0, "% next Holy Shock critical chance; 120 sec recovery")
            }
            TalentStat::ColdBlood | TalentStat::ElementalMastery => {
                (100.0, "% next-cast critical chance; 180 sec recovery")
            }
            TalentStat::Remorseless => (100.0, "% next-attack critical chance after a kill"),
            TalentStat::FlurryHaste => (100.0, "% haste while Flurry is active"),
            TalentStat::Vengeance => (100.0, "% Paladin Holy damage while Vengeance is active"),
            TalentStat::InspirationArmor => (100.0, "% armor after Holy Nova healing"),
            TalentStat::BlessedRecovery => (100.0, "% health damage restored over 6 sec"),
            TalentStat::BloodCrazeHealing => {
                (100.0, "% maximum health over 6 seconds after health damage")
            }
            TalentStat::EnrageDamage => (100.0, "% damage while Enrage is active"),
            TalentStat::CriticalBleed => (100.0, "% of direct critical damage as bleed"),
            TalentStat::CriticalBurn => (100.0, "% of critical fire hit damage as Ignite"),
            TalentStat::WintersChill => (100.0, "% chance to apply Winter's Chill on frost damage"),
            TalentStat::Impact => (100.0, "% chance to stun on fire damage"),
            TalentStat::Blackout => (100.0, "% chance to stun on Shadow damage"),
            TalentStat::ShadowWeaving => (100.0, "% chance to add Shadow vulnerability"),
            TalentStat::VampiricEmbrace => (100.0, "% Shadow damage healed"),
            TalentStat::ImprovedVampiricEmbrace => (100.0, "% additional Shadow damage healed"),
            TalentStat::Frostbite => (100.0, "% chance to freeze on frost damage"),
            TalentStat::Shatter => (100.0, "% critical chance against frozen enemies"),
            TalentStat::Permafrost => (
                100.0,
                "% stronger frost slows; also +1 sec duration per rank",
            ),
            TalentStat::BlizzardSlow => (100.0, "% stronger Blizzard slow"),
            TalentStat::FlameThrowing => (100.0, "% fire spell targeting range"),
            TalentStat::ArcticReach => (100.0, "% frost targeting range / Nova and Cone radius"),
            TalentStat::HolyReach => (100.0, "% Smite/Holy Fire range and Holy Nova radius"),
            TalentStat::ShadowReach => (100.0, "% Shadow Word: Pain/Mind Blast range"),
            TalentStat::Reduction => (100.0, "% damage reduction"),
            TalentStat::Speed => (100.0, "% movement speed"),
            TalentStat::SpellDuration(_) => (100.0, "% duration"),
            TalentStat::SpellHeal(_) => (100.0, "% maximum health healed per cast"),
            TalentStat::SpellRadius(_) => (100.0, "% radius"),
        };
        let amount = |rank| {
            let value = format!("{:.1}", self.per_rank * f32::from(rank) * scale);
            value.strip_suffix(".0").unwrap_or(&value).to_owned()
        };
        if rank == self.max_rank {
            format!("This talent: {}{unit} (maximum rank)", amount(rank))
        } else {
            format!(
                "This talent: {} to {}{unit} next rank",
                amount(rank),
                amount(rank + 1)
            )
        }
    }
}

/// Audited 1.12.1 node identities; explicit Survivors conversions, without rage or parry.
pub const WARRIOR_TALENTS: &[TalentDefinition] = &[
    // Spell.dbc 12318/12857/12858/12860/12861: +5–25% to the shout's magnitude.
    TalentDefinition {
        id: 154, max_rank: 5, name: "Improved Battle Shout",
        description: "Battle Shout: +5% buff strength per rank. Improves its weapon-damage bonus, not global damage; requires acquiring and casting Battle Shout (Survivors attack-power conversion)",
        stat: TalentStat::BattleShoutPower, per_rank: 0.05,
    },
    // Spell.dbc 12289/12668/23695: 5/10/15%; trigger 23694 lasts 5 seconds.
    TalentDefinition {
        id: 129, max_rank: 3, name: "Improved Hamstring",
        description: "Thunder Clap: +5% chance per rank on each direct hit to root for 5 sec (bosses: 1.5 sec). Preserves its slow; excludes splash copies and does not freeze enemies or grant the ability (Survivors Hamstring conversion)",
        stat: TalentStat::HamstringRoot, per_rank: 0.05,
    },
    TalentDefinition {
        id: 125, max_rank: 5, name: "Mace Specialization",
        description: "With a worn mace: 1–6% chance after a direct weapon hit to stun its target for 3 sec (bosses: 0.9 sec). Cosmetic weapons do not qualify. Excludes Thunder Clap, shouts, bleed ticks, splash copies and generated extra strikes",
        stat: TalentStat::MaceStun, per_rank: 0.01,
    },
    // Spell.dbc 12165/12830–12833: +1–5 percentage points of polearm critical chance.
    TalentDefinition {
        id: 134, max_rank: 5, name: "Polearm Specialization",
        description: "With a worn polearm: +1% critical chance per rank for Cleave, Heroic Strike, Whirlwind, Sweeping Strikes, Execute, Mortal Strike and Retaliation. Cosmetic weapons do not qualify. Excludes Thunder Clap, shouts and bleed ticks",
        stat: TalentStat::PolearmCrit, per_rank: 0.01,
    },
    TalentDefinition {
        id: 123, max_rank: 5, name: "Sword Specialization",
        description: "With a worn sword: 1% chance per rank after a direct weapon hit to strike the same target again. Repeats the hit's direct damage, including its critical result, without copying area, control, bleed or execute effects. Extra strikes cannot trigger further talent procs. Cosmetic weapons do not qualify (Survivors conversion)",
        stat: TalentStat::SwordExtraAttack, per_rank: 0.01,
    },
    // Spell.dbc 12700/12781/12783–12785 and 16538–16542: die + base points.
    TalentDefinition {
        id: 132, max_rank: 5, name: "Axe Specialization",
        description: "With a worn axe: +1% critical chance per rank for Cleave, Heroic Strike, Whirlwind, Sweeping Strikes, Execute, Mortal Strike and Retaliation. Cosmetic weapons do not qualify. Excludes Thunder Clap, shouts and bleed ticks",
        stat: TalentStat::AxeCrit, per_rank: 0.01,
    },
    TalentDefinition {
        id: 702, max_rank: 5, name: "One-Handed Weapon Specialization",
        description: "With a worn one-handed melee weapon: +2% damage per rank for Cleave, Heroic Strike, Whirlwind, Sweeping Strikes, Execute, Mortal Strike and Retaliation. Cosmetic weapons do not qualify. Excludes Thunder Clap and shouts; does not separately multiply bleed ticks",
        stat: TalentStat::OneHandedDamage, per_rank: 0.02,
    },
    TalentDefinition {
        id: 136, max_rank: 5, name: "Two-Handed Weapon Specialization",
        description: "With a worn two-handed melee weapon: +1% damage per rank for Cleave, Heroic Strike, Whirlwind, Sweeping Strikes, Execute, Mortal Strike and Retaliation. Cosmetic weapons do not qualify. Excludes Thunder Clap and shouts; does not separately multiply bleed ticks",
        stat: TalentStat::TwoHandedDamage, per_rank: 0.01,
    },
    TalentDefinition {
        id: 124,
        max_rank: 3,
        name: "Improved Heroic Strike",
        description: "Heroic Strike: 5% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("heroic_strike"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 130,
        max_rank: 5,
        name: "Deflection",
        description: "+1% damage reduction per rank (Survivors conversion)",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 128,
        max_rank: 3,
        name: "Improved Thunder Clap",
        description: "Thunder Clap: 10% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("thunder_clap"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 662,
        max_rank: 2,
        name: "Impale",
        description: "+10% critical damage per rank",
        stat: TalentStat::CritDamage,
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 133,
        max_rank: 1,
        name: "Sweeping Strikes",
        description: "Sweeping Strikes: +20% damage",
        stat: TalentStat::SpellDamage("sweeping_strikes"),
        per_rank: 0.2,
    },
    TalentDefinition {
        id: 135,
        max_rank: 1,
        name: "Mortal Strike",
        description: "Mortal Strike: +25% damage",
        stat: TalentStat::SpellDamage("mortal_strike"),
        per_rank: 0.25,
    },
    TalentDefinition {
        id: 158,
        max_rank: 5,
        name: "Booming Voice",
        description: "Shouts: +10% duration and Demoralizing Shout radius per rank. Battle Shout remains self-only",
        stat: TalentStat::BoomingVoice,
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 157,
        max_rank: 5,
        name: "Cruelty",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 166,
        max_rank: 3,
        name: "Improved Cleave",
        description: "Cleave: +10% damage per rank",
        stat: TalentStat::SpellDamage("cleave"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 1542,
        max_rank: 2,
        name: "Improved Execute",
        description: "Execute: 10% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("execute"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 138,
        max_rank: 5,
        name: "Anticipation",
        description: "+1% damage reduction per rank (Survivors conversion)",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 140,
        max_rank: 5,
        name: "Toughness",
        description: "+6 armor per rank",
        stat: TalentStat::Armor,
        per_rank: 6.0,
    },
    TalentDefinition {
        id: 147,
        max_rank: 3,
        name: "Improved Revenge",
        description: "Retaliation: +10% reflected damage per rank (Survivors conversion)",
        stat: TalentStat::SpellDamage("retaliation"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 150,
        max_rank: 2,
        name: "Improved Shield Wall",
        description: "Retaliation: +15% duration per rank (Survivors conversion)",
        stat: TalentStat::SpellDuration("retaliation"),
        per_rank: 0.15,
    },
    TalentDefinition {
        id: 153,
        max_rank: 1,
        name: "Last Stand",
        description: "Retaliation heals an additional 5% maximum health once per cast",
        stat: TalentStat::SpellHeal("retaliation"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 167,
        max_rank: 1,
        name: "Bloodthirst",
        description: "Execute heals an additional 2% maximum health once per cast",
        stat: TalentStat::SpellHeal("execute"),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 161,
        max_rank: 5,
        name: "Improved Demoralizing Shout",
        description: "Demoralizing Shout: +10% weakening duration per rank (Survivors conversion)",
        stat: TalentStat::SpellDuration("demoralizing_shout"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 127,
        max_rank: 3,
        name: "Improved Rend",
        description: "Deep Wounds: +10% bleed damage per rank (Survivors conversion)",
        stat: TalentStat::SpellDamage("deep_wounds"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 121,
        max_rank: 3,
        name: "Deep Wounds",
        description: "Direct criticals bleed for an additional 4% damage per rank over 3 seconds; refreshes, does not stack",
        stat: TalentStat::CriticalBleed,
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 160,
        max_rank: 1,
        name: "Piercing Howl",
        description: "Demoralizing Shout adds 2 seconds of 50% slow; adds to the Fury milestone. Bosses are slowed by 25%",
        stat: TalentStat::SecondarySlowDuration("demoralizing_shout"),
        per_rank: 2.0,
    },
    TalentDefinition {
        id: 152,
        max_rank: 1,
        name: "Concussion Blow",
        description: "Heroic Strike adds 1 second of stun; adds to the Protection milestone. Boss stun duration is reduced by 70%",
        stat: TalentStat::SecondaryStunDuration("heroic_strike"),
        per_rank: 1.0,
    },
    TalentDefinition {
        id: 1601,
        max_rank: 5,
        name: "Shield Specialization",
        description: "Heroic Strike grants a 3-second absorb: +1% maximum health per rank; refreshes without stacking",
        stat: TalentStat::SpellShield("heroic_strike"),
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 145,
        max_rank: 3,
        name: "Improved Shield Block",
        description: "Heroic Strike's absorb lasts 0.5 seconds longer per rank; requires an absorb talent or milestone",
        stat: TalentStat::SpellShieldDuration("heroic_strike"),
        per_rank: 0.5,
    },
    TalentDefinition {
        id: 148,
        max_rank: 1,
        name: "Shield Slam",
        description: "Heroic Strike adds a 3% maximum-health absorb for 3 seconds; adds to other shield talents and milestone",
        stat: TalentStat::SpellShield("heroic_strike"),
        per_rank: 0.03,
    },
    TalentDefinition {
        id: 155,
        max_rank: 5,
        name: "Enrage",
        description: "After losing health to a hit, deal +4% damage per rank for 4 seconds; further hits refresh without stacking",
        stat: TalentStat::EnrageDamage,
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 165,
        max_rank: 1,
        name: "Death Wish",
        description: "Retaliation: +25% duration; also extends the Fury milestone's increased damage taken (Survivors conversion)",
        stat: TalentStat::SpellDuration("retaliation"),
        per_rank: 0.25,
    },
    TalentDefinition {
        id: 151,
        max_rank: 3,
        name: "Improved Disarm",
        description: "Demoralizing Shout: +10% weakening duration per rank (Survivors conversion)",
        stat: TalentStat::SpellDuration("demoralizing_shout"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 143,
        max_rank: 2,
        name: "Improved Taunt",
        description: "Demoralizing Shout: 10% shorter cooldown per rank (Survivors conversion)",
        stat: TalentStat::SpellCooldown("demoralizing_shout"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 661,
        max_rank: 3,
        name: "Blood Craze",
        description: "After losing health to a hit, heal 1% maximum health per rank over 6 seconds; further hits refresh without stacking (Survivors conversion)",
        stat: TalentStat::BloodCrazeHealing,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 156, max_rank: 5, name: "Flurry",
        description: "Direct attack criticals grant 5% haste per rank for 3 seconds; further criticals refresh without stacking (Survivors conversion)",
        stat: TalentStat::FlurryHaste, per_rank: 0.05,
    },

    TalentDefinition {
        id: 126, max_rank: 2, name: "Improved Charge",
        description: "Charge: +20% stun duration per rank (Survivors conversion)",
        stat: TalentStat::SpellDuration("charge"), per_rank: 0.2,
    },
    TalentDefinition {
        id: 1543, max_rank: 2, name: "Improved Intercept",
        description: "Charge: 10% shorter cooldown per rank (Survivors conversion)",
        stat: TalentStat::SpellCooldown("charge"), per_rank: 0.1,
    },

    TalentDefinition {
        id: 131, max_rank: 2, name: "Improved Overpower",
        description: "Heroic Strike: +10 percentage points critical chance per rank (Survivors conversion)",
        stat: TalentStat::SpellCrit("heroic_strike"), per_rank: 0.1,
    },
    TalentDefinition {
        id: 168, max_rank: 5, name: "Improved Slam",
        description: "Mortal Strike: 5% shorter cooldown per rank (Survivors conversion)",
        stat: TalentStat::SpellCooldown("mortal_strike"), per_rank: 0.05,
    },

];

/// Survivors Paladin attacks; excludes immunity and other classes' Holy spells.
pub const PALADIN_HOLY: &[&str] = &[
    "hammer_of_wrath",
    "consecration",
    "holy_shock",
    "exorcism",
    "holy_wrath",
    "hammer_of_justice",
    "judgement",
];

pub const PALADIN_TALENTS: &[TalentDefinition] = &[
    TalentDefinition {
        id: 1431, max_rank: 1, name: "Blessing of Sanctuary",
        description: "Prevents 2 damage per incoming hit after armor and percentage reduction, before shields. Cannot reduce damage below zero. Always active on you; grants no block chance or retaliation damage (Survivors passive blessing conversion)",
        stat: TalentStat::FlatAbsorption, per_rank: 2.0,
    },
    TalentDefinition {
        id: 1433, max_rank: 1, name: "Divine Favor",
        description: "Your next successful Holy Shock has 100% direct critical chance for every target, then recovers after 120 sec of active run time. Ready at run start; no stored charges. Requires acquiring Holy Shock; healing, other spells and periodic damage are unchanged",
        stat: TalentStat::DivineFavor, per_rank: 1.0,
    },
    TalentDefinition {
        id: 1407, max_rank: 5, name: "Benediction",
        description: "Judgement: 3% shorter cooldown per rank, multiplying Improved Judgement and milestone training (Survivors seal mana-cost conversion)",
        stat: TalentStat::SpellCooldown("judgement"), per_rank: 0.03,
    },
    TalentDefinition {
        id: 1463, max_rank: 5, name: "Improved Seal of Righteousness",
        description: "Judgement: +3% damage per rank, multiplying other training. Requires acquiring Judgement during the run (Survivors seal conversion)",
        stat: TalentStat::SpellDamage("judgement"), per_rank: 0.03,
    },
    TalentDefinition {
        id: 1409, max_rank: 1, name: "Sanctity Aura",
        description: "Paladin Holy attacks: +10% damage, including Consecration; always active (Survivors aura conversion). Excludes other-class and jewelry spells",
        stat: TalentStat::SpellGroupDamage(PALADIN_HOLY), per_rank: 0.10,
    },
    TalentDefinition {
        id: 1402, max_rank: 5, name: "Vengeance",
        description: "Direct Paladin Holy critical hits grant +3% Paladin Holy damage per rank for 8 sec; refreshes without stacking. New Consecration casts snapshot the bonus; its ticks cannot trigger it",
        stat: TalentStat::Vengeance, per_rank: 0.03,
    },
    TalentDefinition {
        id: 1450,
        max_rank: 5,
        name: "Divine Strength",
        description: "+2% damage per rank",
        stat: TalentStat::Damage,
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 1449,
        max_rank: 5,
        name: "Divine Intellect",
        description: "+1% cooldown reduction per rank",
        stat: TalentStat::Cooldown,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1422,
        max_rank: 5,
        name: "Improved Devotion Aura",
        description: "+8 armor per rank",
        stat: TalentStat::Armor,
        per_rank: 8.0,
    },
    TalentDefinition {
        id: 1630,
        max_rank: 3,
        name: "Precision",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1423,
        max_rank: 5,
        name: "Toughness",
        description: "+6 armor per rank",
        stat: TalentStat::Armor,
        per_rank: 6.0,
    },
    TalentDefinition {
        id: 1629,
        max_rank: 5,
        name: "Anticipation",
        description: "+1% damage reduction per rank",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1401,
        max_rank: 5,
        name: "Improved Blessing of Might",
        description: "+2% damage per rank",
        stat: TalentStat::Damage,
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 1403,
        max_rank: 5,
        name: "Deflection",
        description: "+1% damage reduction per rank",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1411,
        max_rank: 5,
        name: "Conviction",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1634,
        max_rank: 2,
        name: "Pursuit of Justice",
        description: "+4% movement speed per rank",
        stat: TalentStat::Speed,
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 1631,
        max_rank: 2,
        name: "Improved Judgement",
        description: "Judgement: 10% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("judgement"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 1521,
        max_rank: 3,
        name: "Improved Hammer of Justice",
        description: "Hammer of Justice: 10% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("hammer_of_justice"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 1627,
        max_rank: 5,
        name: "Holy Power",
        description: "Holy Shock: +2% critical chance per rank",
        stat: TalentStat::SpellCrit("holy_shock"),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 1435,
        max_rank: 1,
        name: "Consecration",
        description: "Consecration: +20% damage",
        stat: TalentStat::SpellDamage("consecration"),
        per_rank: 0.2,
    },
    TalentDefinition {
        id: 1502,
        max_rank: 1,
        name: "Holy Shock",
        description: "Holy Shock: +25% damage",
        stat: TalentStat::SpellDamage("holy_shock"),
        per_rank: 0.25,
    },
    TalentDefinition {
        id: 1444,
        max_rank: 3,
        name: "Healing Light",
        description: "Holy Shock heals 1% maximum health per rank once per cast",
        stat: TalentStat::SpellHeal("holy_shock"),
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1443,
        max_rank: 2,
        name: "Improved Lay on Hands",
        description: "Divine Shield heals 5% maximum health per rank once per cast",
        stat: TalentStat::SpellHeal("divine_shield"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 1425,
        max_rank: 2,
        name: "Guardian's Favor",
        description: "Divine Shield: +15% duration per rank",
        stat: TalentStat::SpellDuration("divine_shield"),
        per_rank: 0.15,
    },
    TalentDefinition {
        id: 1421,
        max_rank: 5,
        name: "Redoubt",
        description: "Divine Shield: 3% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("divine_shield"),
        per_rank: 0.03,
    },
];

const HUNTER_SHOTS: &[&str] = &[
    "arcane_shot",
    "multi_shot",
    "serpent_sting",
    "concussive_shot",
    "aimed_shot",
    "scorpid_sting",
];

static HUNTER_TALENTS: &[TalentDefinition] = &[
    TalentDefinition { id: 1305, max_rank: 5, name: "Improved Wing Clip",
        description: "Wing Clip: +4% chance per rank on each direct hit to root for 5 sec (bosses: 1.5 sec). Preserves its slow; does not freeze enemies or grant the ability",
        stat: TalentStat::WingClipRoot, per_rank: 0.04 },
    TalentDefinition { id: 1621, max_rank: 2, name: "Savage Strikes",
        description: "Raptor Strike and Mongoose Bite: +10 percentage points critical chance per rank. Excludes shots, traps, Volley and the combat wolf; requires acquiring either melee ability",
        stat: TalentStat::SpellGroupCrit(&["raptor_strike", "mongoose_bite"]), per_rank: 0.10 },
    TalentDefinition { id: 1382, max_rank: 5, name: "Improved Aspect of the Hawk",
        description: "Hunter shots: +3% haste per rank for 12 sec after a 10% chance on each successful shot cast. Refreshes without stacking; affects subsequent shot cooldowns. Excludes Volley, traps, pets and periodic ticks (Survivors automatic aspect conversion)",
        stat: TalentStat::ImprovedHawk, per_rank: 0.03 },
    TalentDefinition { id: 1322, max_rank: 2, name: "Trap Mastery",
        description: "Explosive Trap: +5% direct and periodic damage per rank, multiplying Clever Traps. Radius, cooldown and Entrapment root duration are unchanged (Survivors trap-resistance conversion)",
        stat: TalentStat::SpellDamage("explosive_trap"), per_rank: 0.05 },
    TalentDefinition { id: 1351, max_rank: 3, name: "Improved Scorpid Sting",
        description: "Scorpid Sting: +20% damage-reduction duration per rank. Its 25% enemy damage reduction, hit damage and target count are unchanged; requires acquiring the spell (Survivors stamina-reduction conversion)",
        stat: TalentStat::SpellDuration("scorpid_sting"), per_rank: 0.20 },
    TalentDefinition { id: 1361, max_rank: 1, name: "Trueshot Aura",
        description: "Your Hunter shots: +10% direct and periodic damage, multiplying other damage talents. Excludes Volley, traps, pets and other-class abilities (Survivors passive ranged-attack-power conversion; self only)",
        stat: TalentStat::SpellGroupDamage(HUNTER_SHOTS), per_rank: 0.10 },
    TalentDefinition { id: 1385, max_rank: 2, name: "Improved Mend Pet",
        description: "Living combat wolf: regenerate 1% maximum health per sec per rank. Does not heal the Hunter, revive the wolf or shorten recovery (Survivors automatic Mend Pet conversion; no dispel)",
        stat: TalentStat::PetMend, per_rank: 0.01 },
    TalentDefinition { id: 1390, max_rank: 2, name: "Bestial Discipline",
        description: "Combat wolf: 10% shorter bite cooldown per rank, multiplying the 20-point milestone and Frenzy haste. Does not affect Hunter spells, recovery or pickup companions (Survivors pet-focus regeneration conversion)",
        stat: TalentStat::PetCooldown, per_rank: 0.1 },
    TalentDefinition { id: 1342, max_rank: 5, name: "Efficiency",
        description: "Shots and Serpent Sting: 2% shorter cooldown per rank. Excludes Volley, traps and the combat wolf (Survivors mana-cost conversion)",
        stat: TalentStat::SpellGroupCooldown(HUNTER_SHOTS), per_rank: 0.02 },
    TalentDefinition { id: 1386, max_rank: 1, name: "Bestial Wrath",
        description: "Upgrades the 30-point milestone: combat wolf bite damage rises from +50 to +100%, and its window from 10 to 18 sec in each 30 sec cycle. Replaces the base Wrath bonus; no immunity or hero spell bonus",
        stat: TalentStat::PetWrath, per_rank: 1.0 },
    TalentDefinition { id: 1387, max_rank: 1, name: "Intimidation",
        description: "Every 60 sec, the wolf's next bite stuns its nearest victim for 3 sec and forces it to target the living wolf for 6 sec. Bosses: 0.9 sec stun, no forced targeting. Extra bite targets are unaffected",
        stat: TalentStat::PetIntimidation, per_rank: 1.0 },
    TalentDefinition { id: 1625, max_rank: 2, name: "Improved Revive Pet",
        description: "Combat wolf: 20% shorter recovery after defeat per rank, reducing the base 10 sec to 8 or 6 sec. Recovery still requires a safe position",
        stat: TalentStat::PetRecovery, per_rank: 0.2 },
    TalentDefinition { id: 1388, max_rank: 2, name: "Spirit Bond",
        description: "While the combat wolf is alive, every 10 sec restore 1% of your and its own maximum health per rank. Defeat resets the interval; pauses preserve it",
        stat: TalentStat::SpiritBond, per_rank: 0.01 },
    TalentDefinition { id: 1389, max_rank: 5, name: "Endurance Training",
        description: "Combat wolf: +3% maximum health per rank. Does not increase hero health; the wolf has a base recovery of 10 sec after defeat",
        stat: TalentStat::PetHealth, per_rank: 0.03 },
    TalentDefinition { id: 1396, max_rank: 5, name: "Unleashed Fury",
        description: "Combat wolf: +4% bite damage per rank, multiplying the Unleashed Fury milestone and Bestial Wrath. Does not affect Hunter spells or pickup companions",
        stat: TalentStat::PetDamage, per_rank: 0.04 },
    TalentDefinition { id: 1393, max_rank: 5, name: "Ferocity",
        description: "Combat wolf: +2 percentage points bite critical chance per rank, added to global critical chance. Critical bites use global critical damage",
        stat: TalentStat::PetCrit, per_rank: 0.02 },
    TalentDefinition { id: 1397, max_rank: 5, name: "Frenzy",
        description: "Wolf bite criticals: 20% chance per rank to grant 30% bite haste for 8 sec. Refreshes without stacking; combines with the automatic Frenzy milestone",
        stat: TalentStat::PetFrenzy, per_rank: 0.2 },
    TalentDefinition { id: 1391, max_rank: 1, name: "Bestial Swiftness",
        description: "Combat wolf: +30% movement speed; melee reach, leash and pickup companions are unchanged",
        stat: TalentStat::PetSpeed, per_rank: 0.3 },
    TalentDefinition {
        id: 1308, max_rank: 1, name: "Deterrence",
        description: "Automatically take 50% less incoming damage for 10 sec at run start and every 180 sec of gameplay time. Multiplies armor and other damage reduction before shields. Pauses stop the window; missed windows are not stored (Survivors dodge/parry conversion)",
        stat: TalentStat::Deterrence, per_rank: 0.50,
    },
    TalentDefinition {
        id: 1622, max_rank: 5, name: "Survivalist",
        description: "+2% maximum health per rank, including equipped health and multiplying passive, renown and racial health bonuses. Does not heal current health",
        stat: TalentStat::MaximumHealth, per_rank: 0.02,
    },
    TalentDefinition {
        id: 1352, max_rank: 3, name: "Hawk Eye",
        description: "Shots, stings and Volley: +2 yd targeting range per rank. Volley's radius and target counts are unchanged; excludes Explosive Trap",
        stat: TalentStat::SpellGroupRangeYards(&["arcane_shot", "multi_shot", "serpent_sting", "concussive_shot", "aimed_shot", "scorpid_sting", "volley"]), per_rank: 2.0,
    },
    TalentDefinition {
        id: 1353,
        max_rank: 1,
        name: "Scatter Shot",
        description: "Concussive Shot also stuns for 1 sec (bosses: 0.3 sec; Survivors conversion)",
        stat: TalentStat::SecondaryStunDuration("concussive_shot"),
        per_rank: 1.0,
    },
    TalentDefinition {
        id: 1304,
        max_rank: 5,
        name: "Entrapment",
        description: "Explosive Trap hits add 0.4 sec of root per rank (bosses: 30% duration; Survivors conversion)",
        stat: TalentStat::SecondaryRootDuration("explosive_trap"),
        per_rank: 0.4,
    },
    TalentDefinition {
        id: 1325,
        max_rank: 1,
        name: "Wyvern Sting",
        description: "Serpent Sting also stuns for 1.5 sec (bosses: 0.45 sec; Survivors conversion)",
        stat: TalentStat::SecondaryStunDuration("serpent_sting"),
        per_rank: 1.5,
    },
    TalentDefinition {
        id: 1344,
        max_rank: 5,
        name: "Lethal Shots",
        description: "Shots: +1 percentage point critical chance per rank; excludes Volley and traps",
        stat: TalentStat::SpellGroupCrit(HUNTER_SHOTS),
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1349,
        max_rank: 5,
        name: "Mortal Shots",
        description: "Shots: +6 percentage points critical damage per rank; excludes Volley and traps (Survivors conversion)",
        stat: TalentStat::SpellGroupCritDamage(HUNTER_SHOTS),
        per_rank: 0.06,
    },
    TalentDefinition {
        id: 1347,
        max_rank: 3,
        name: "Barrage",
        description: "Multi-Shot and Volley: +4% damage per rank",
        stat: TalentStat::SpellGroupDamage(&["multi_shot", "volley"]),
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 1362,
        max_rank: 5,
        name: "Ranged Weapon Specialization",
        description: "Shots: +1% direct and periodic damage per rank; excludes Volley and traps (Survivors conversion)",
        stat: TalentStat::SpellGroupDamage(HUNTER_SHOTS),
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1395,
        max_rank: 3,
        name: "Thick Hide",
        description: "+8 hero armor per rank (Survivors conversion)",
        stat: TalentStat::Armor,
        per_rank: 8.0,
    },
    TalentDefinition {
        id: 1384,
        max_rank: 2,
        name: "Pathfinding",
        description: "+4% hero movement speed per rank (Survivors conversion)",
        stat: TalentStat::Speed,
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 1341,
        max_rank: 5,
        name: "Improved Concussive Shot",
        description: "Concussive Shot: +10% slow duration per rank (Survivors conversion)",
        stat: TalentStat::SpellDuration("concussive_shot"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 1345,
        max_rank: 1,
        name: "Aimed Shot",
        description: "Aimed Shot: +25% damage; does not grant the ability (Survivors conversion)",
        stat: TalentStat::SpellDamage("aimed_shot"),
        per_rank: 0.25,
    },
    TalentDefinition {
        id: 1346,
        max_rank: 5,
        name: "Improved Arcane Shot",
        description: "Arcane Shot: 5% shorter cooldown per rank (Survivors conversion)",
        stat: TalentStat::SpellCooldown("arcane_shot"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 1348,
        max_rank: 5,
        name: "Improved Serpent Sting",
        description:
            "Serpent Sting: +2% direct and periodic damage per rank (Survivors conversion)",
        stat: TalentStat::SpellDamage("serpent_sting"),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 1311,
        max_rank: 5,
        name: "Deflection",
        description: "+1% damage reduction per rank (Survivors conversion)",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 1306,
        max_rank: 2,
        name: "Clever Traps",
        description:
            "Explosive Trap: +15% direct and periodic damage per rank (Survivors conversion)",
        stat: TalentStat::SpellDamage("explosive_trap"),
        per_rank: 0.15,
    },
    TalentDefinition {
        id: 1321,
        max_rank: 3,
        name: "Killer Instinct",
        description: "+1% critical chance per rank",
        stat: TalentStat::Crit,
        per_rank: 0.01,
    },
];

// Explicit school membership: neutral jewelry abilities and other classes do not inherit these.
const MAGE_ARCANE: &[&str] = &["arcane_missiles", "arcane_explosion"];
const MAGE_FIRE: &[&str] = &[
    "fireball",
    "fire_blast",
    "flamestrike",
    "pyroblast",
    "blast_wave",
];
const MAGE_FROST: &[&str] = &["frostbolt", "frost_nova", "blizzard", "cone_of_cold"];

/// DBC identities and rank caps; tuning is an explicit Survivors adaptation.
static MAGE_TALENTS: &[TalentDefinition] = &[
    TalentDefinition {
        id: 83, max_rank: 2, name: "Improved Mana Shield",
        description: "Mana Shield: 10% shorter cooldown per rank. Absorption and duration are unchanged; requires acquiring Mana Shield during the run (Survivors mana-cost conversion; no mana resource)",
        stat: TalentStat::SpellCooldown("mana_shield"), per_rank: 0.10,
    },
    TalentDefinition {
        id: 86, max_rank: 1, name: "Presence of Mind",
        description: "Arcane Missiles advances an owned Arcane Explosion cooldown by 2 sec once per successful cast; adds to the 20-point Arcane milestone for 4 sec total. Does not trigger per missile, grant Explosion or store a charge (Survivors automatic instant-cast conversion)",
        stat: TalentStat::PresenceOfMind, per_rank: 2.0,
    },
    TalentDefinition {
        id: 74, max_rank: 2, name: "Arcane Subtlety",
        description: "Arcane Missiles and Arcane Explosion: +5% direct and periodic damage per rank. Excludes Fire, Frost and other-class spells (Survivors resistance-penetration conversion; no resistance or threat roll)",
        stat: TalentStat::SpellGroupDamage(MAGE_ARCANE), per_rank: 0.05,
    },
    TalentDefinition { id: 36, max_rank: 1, name: "Combustion",
        description: "Mage Fire spells: +10% critical chance initially and per landed damaging direct hit, capped at 100%. Ends after three direct critical hits; recovers 180 sec after ending. Area targets count separately; burns and multistrike echoes do not count. Pauses stop recovery (Survivors automatic conversion)",
        stat: TalentStat::Combustion, per_rank: 0.10 },
    TalentDefinition { id: 87, max_rank: 1, name: "Arcane Power",
        description: "Mage spells: +30% direct and periodic damage for 15 sec, starting with the next successful damaging Mage cast. Ready at run start; recovers 180 sec after activation. Casts snapshot the bonus for their full duration. Pauses stop both timers. Excludes Mana Shield and foreign spells (Survivors automatic conversion; no mana surcharge)",
        stat: TalentStat::ArcanePower, per_rank: 0.30 },
    TalentDefinition { id: 69, max_rank: 1, name: "Cold Snap",
        description: "Grants 1 automatic reset of owned Frostbolt, Frost Nova, Blizzard and Cone of Cold cooldowns every 90 sec of run time; first at 90 sec. Does not grant spells or store charges (Survivors automatic conversion)",
        stat: TalentStat::ColdSnap, per_rank: 1.0 },
    TalentDefinition { id: 66, max_rank: 3, name: "Frost Channeling",
        description: "Frostbolt, Frost Nova, Blizzard and Cone of Cold: 5% shorter cooldown per rank. Does not change area duration or tick rate (Survivors mana-cost conversion)",
        stat: TalentStat::SpellGroupCooldown(MAGE_FROST), per_rank: 0.05 },
    TalentDefinition {
        id: 421, max_rank: 3, name: "Arcane Instability",
        description: "Mage spells: +1% damage and +1% critical chance per rank; excludes Mana Shield, other-class and neutral jewelry spells",
        stat: TalentStat::ArcaneInstability, per_rank: 0.01,
    },
    TalentDefinition {
        id: 1649, max_rank: 3, name: "Elemental Precision",
        description: "Fire and Frost spells: +1% damage per rank (Survivors hit conversion); excludes Arcane and other-class spells",
        stat: TalentStat::ElementalPrecision, per_rank: 0.01,
    },
    TalentDefinition {
        id: 72, max_rank: 1, name: "Ice Block",
        description: "Frost Nova grants 0.5 sec immunity once per successful cast; preserves longer immunity (Survivors conversion)",
        stat: TalentStat::SpellImmune("frost_nova"), per_rank: 0.5,
    },
    TalentDefinition {
        id: 71, max_rank: 1, name: "Ice Barrier",
        description: "Frost Nova grants an additional 5% maximum-health absorb for 3 sec, or 5 sec with the Frost 30-point milestone; does not stack (Survivors conversion)",
        stat: TalentStat::SpellShield("frost_nova"), per_rank: 0.05,
    },

    TalentDefinition {
        id: 68, max_rank: 5, name: "Winter's Chill",
        description: "Frost damage has a 20% chance per rank to add a 15 sec chill stack: +2% Frost critical chance against that enemy, up to five stacks (Survivors conversion)",
        stat: TalentStat::WintersChill, per_rank: 0.20,
    },

    TalentDefinition {
        id: 30, max_rank: 5, name: "Impact",
        description: "Fire damage has a 2% chance per rank to stun for 1 sec (bosses: 0.3 sec; Survivors conversion)",
        stat: TalentStat::Impact, per_rank: 0.02,
    },
    TalentDefinition {
        id: 1141, max_rank: 2, name: "Incinerate",
        description: "Fire Blast: +2% critical chance per rank",
        stat: TalentStat::SpellCrit("fire_blast"), per_rank: 0.02,
    },

    TalentDefinition {
        id: 28, max_rank: 2, name: "Flame Throwing",
        description: "Targeted fire spells: +10% targeting range per rank; does not enlarge their explosions (Survivors conversion)",
        stat: TalentStat::FlameThrowing, per_rank: 0.10,
    },
    TalentDefinition {
        id: 741, max_rank: 2, name: "Arctic Reach",
        description: "Frostbolt and Blizzard: +10% targeting range per rank. Frost Nova and Cone of Cold: +10% radius per rank; Blizzard stays one area",
        stat: TalentStat::ArcticReach, per_rank: 0.10,
    },

    TalentDefinition {
        id: 65, max_rank: 3, name: "Permafrost",
        description: "Frost slows last 1 sec longer and reduce movement by 3 additional percentage points per rank; does not extend freezes",
        stat: TalentStat::Permafrost, per_rank: 0.03,
    },
    TalentDefinition {
        id: 63, max_rank: 3, name: "Improved Blizzard",
        description: "Blizzard reduces movement by 10 additional percentage points per rank; adds to its existing slow (Survivors conversion)",
        stat: TalentStat::BlizzardSlow, per_rank: 0.10,
    },

    TalentDefinition {
        id: 67,
        max_rank: 5,
        name: "Shatter",
        description: "Mage spells gain 10% critical chance per rank against frozen enemies; ordinary roots do not qualify",
        stat: TalentStat::Shatter,
        per_rank: 0.10,
    },

    TalentDefinition {
        id: 38,
        max_rank: 3,
        name: "Frostbite",
        description: "Frost damage has a 5% chance per rank to freeze for 5 sec (bosses: 1.5 sec; Survivors conversion)",
        stat: TalentStat::Frostbite,
        per_rank: 0.05,
    },

    TalentDefinition {
        id: 34,
        max_rank: 5,
        name: "Ignite",
        description: "Critical fire hits add 8% of their damage per rank to a four-tick burn; remaining burn damage rolls into the refresh",
        stat: TalentStat::CriticalBurn,
        per_rank: 0.08,
    },

    TalentDefinition {
        id: 76,
        max_rank: 5,
        name: "Arcane Focus",
        description:
            "Arcane Missiles and Arcane Explosion: +2% damage per rank (Survivors hit conversion)",
        stat: TalentStat::SpellGroupDamage(MAGE_ARCANE),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 80,
        max_rank: 5,
        name: "Improved Arcane Missiles",
        description: "Arcane Missiles: +5% damage per rank (Survivors pushback conversion)",
        stat: TalentStat::SpellDamage("arcane_missiles"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 1650,
        max_rank: 5,
        name: "Magic Absorption",
        description: "+1% damage reduction per rank (Survivors resistance conversion)",
        stat: TalentStat::Reduction,
        per_rank: 0.01,
    },
    TalentDefinition {
        id: 81,
        max_rank: 3,
        name: "Improved Arcane Explosion",
        description: "Arcane Explosion: +2% critical chance per rank",
        stat: TalentStat::SpellCrit("arcane_explosion"),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 26,
        max_rank: 5,
        name: "Improved Fireball",
        description: "Fireball: 4% shorter cooldown per rank (Survivors cast-time conversion)",
        stat: TalentStat::SpellCooldown("fireball"),
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 27,
        max_rank: 3,
        name: "Improved Fire Blast",
        description: "Fire Blast: 5% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("fire_blast"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 31,
        max_rank: 3,
        name: "Improved Flamestrike",
        description: "Flamestrike: +5% critical chance per rank",
        stat: TalentStat::SpellCrit("flamestrike"),
        per_rank: 0.05,
    },
    TalentDefinition {
        id: 29,
        max_rank: 1,
        name: "Pyroblast",
        description: "Pyroblast: +20% damage (spell acquisition remains independent)",
        stat: TalentStat::SpellDamage("pyroblast"),
        per_rank: 0.2,
    },
    TalentDefinition {
        id: 33,
        max_rank: 3,
        name: "Critical Mass",
        description: "Fire spells: +2% critical chance per rank",
        stat: TalentStat::SpellGroupCrit(MAGE_FIRE),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 32,
        max_rank: 1,
        name: "Blast Wave",
        description: "Blast Wave: +20% damage (spell acquisition remains independent)",
        stat: TalentStat::SpellDamage("blast_wave"),
        per_rank: 0.2,
    },
    TalentDefinition {
        id: 35,
        max_rank: 5,
        name: "Fire Power",
        description: "Fire spells: +2% damage per rank",
        stat: TalentStat::SpellGroupDamage(MAGE_FIRE),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 37,
        max_rank: 5,
        name: "Improved Frostbolt",
        description: "Frostbolt: 4% shorter cooldown per rank (Survivors cast-time conversion)",
        stat: TalentStat::SpellCooldown("frostbolt"),
        per_rank: 0.04,
    },
    TalentDefinition {
        id: 73,
        max_rank: 5,
        name: "Ice Shards",
        description: "Frost spells: +10% critical damage per rank (Survivors conversion)",
        stat: TalentStat::SpellGroupCritDamage(MAGE_FROST),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 62,
        max_rank: 2,
        name: "Improved Frost Nova",
        description: "Frost Nova: 10% shorter cooldown per rank",
        stat: TalentStat::SpellCooldown("frost_nova"),
        per_rank: 0.1,
    },
    TalentDefinition {
        id: 61,
        max_rank: 3,
        name: "Piercing Ice",
        description: "Frost spells: +2% damage per rank",
        stat: TalentStat::SpellGroupDamage(MAGE_FROST),
        per_rank: 0.02,
    },
    TalentDefinition {
        id: 64,
        max_rank: 3,
        name: "Improved Cone of Cold",
        description: "Cone of Cold: +10% damage per rank",
        stat: TalentStat::SpellDamage("cone_of_cold"),
        per_rank: 0.1,
    },
];

// Explicit membership excludes neutral jewelry abilities and spells from other classes.
const ROGUE_SPELLS: &[&str] = &[
    "sinister_strike",
    "eviscerate",
    "blade_flurry",
    "rupture",
    "gouge",
    "kidney_shot",
];

const ROGUE_TALENTS: &[TalentDefinition] = &[
    // Spell.dbc 13709/13800–13803: proc 1/2/3/4/6%, plus 1–5 mace skill.
    TalentDefinition {
        id: 184, max_rank: 5, name: "Mace Specialization",
        description: "With a worn mace: 1–6% chance after a direct Rogue weapon hit to stun for 3 sec (bosses 0.9 sec), and +1% direct damage per rank for Sinister Strike, Eviscerate, Blade Flurry, Gouge and Kidney Shot. Cosmetic weapons, bleed ticks, splash copies and generated strikes cannot trigger the stun. Damage is a Survivors weapon-skill conversion; attacks have no skill or miss roll",
        stat: TalentStat::RogueMace, per_rank: 0.01,
    },
    // Spell.dbc 13707/13966–13969: +1–5 percentage points of fist critical chance.
    TalentDefinition {
        id: 183, max_rank: 5, name: "Fist Weapon Specialization",
        description: "With a worn fist weapon: +1% critical chance per rank for Sinister Strike, Eviscerate, Blade Flurry, Gouge and Kidney Shot. Cosmetic weapons do not qualify. Excludes Rupture bleed ticks and neutral item abilities",
        stat: TalentStat::FistCrit, per_rank: 0.01,
    },
    // Spell.dbc 13960–13964, proc-chance field: 1–5%.
    TalentDefinition {
        id: 242, max_rank: 5, name: "Sword Specialization",
        description: "With a worn sword: 1% chance per rank after a direct Rogue weapon hit to strike the same target again. Repeats direct damage, including its critical result, without copying area, control or bleed effects. Extra strikes cannot trigger further talent procs. Cosmetic weapons do not qualify (Survivors conversion)",
        stat: TalentStat::SwordExtraAttack, per_rank: 0.01,
    },
    // Spell.dbc 13706/13804–13807: +1–5 percentage points of dagger critical chance.
    TalentDefinition {
        id: 182, max_rank: 5, name: "Dagger Specialization",
        description: "With a worn dagger: +1% critical chance per rank for Sinister Strike, Eviscerate, Blade Flurry, Gouge and Kidney Shot. Cosmetic weapons do not qualify. Excludes Rupture bleed ticks and neutral item abilities",
        stat: TalentStat::DaggerCrit, per_rank: 0.01,
    },
    TalentDefinition {
        id: 223, max_rank: 1, name: "Blade Flurry",
        description: "A successful owned Blade Flurry starts a 15 sec burst: 20% faster recovery for Sinister Strike, Eviscerate, Gouge, Kidney Shot and Blade Flurry; single-target melee attacks hit 1 additional target within their existing reach. Ready at run start; recovers 120 sec after activation. Does not refresh on repeated casts or change area radius, Rupture, Sprint or existing cooldowns. Does not grant the spell (Survivors automatic conversion)",
        stat: TalentStat::BladeFlurry, per_rank: 0.20,
    },
    TalentDefinition {
        id: 205, max_rank: 1, name: "Adrenaline Rush",
        description: "Rogue abilities: 50% shorter cooldown for casts during a 15 sec burst, starting with the first successful cast. Ready at run start; recovers 300 sec after use. Multiplies other cooldown bonuses. Existing cooldowns, Sprint and Cold Blood recovery are unchanged; pauses stop timers (Survivors automatic energy-regeneration conversion)",
        stat: TalentStat::AdrenalineRush, per_rank: 0.50,
    },
    TalentDefinition {
        id: 204, max_rank: 2, name: "Endurance",
        description: "Sprint: 10% shorter cooldown per rank. Does not change its speed or duration; learned at hero level 5 and activated with F (Survivors cooldown conversion; Evasion is not an owned ability)",
        stat: TalentStat::SpellCooldown("sprint"), per_rank: 0.10,
    },
    TalentDefinition {
        id: 181, max_rank: 5, name: "Precision",
        description: "Rogue abilities: +1% direct and periodic damage per rank; excludes other-class and neutral jewelry abilities (Survivors hit-chance conversion; attacks have no miss roll)",
        stat: TalentStat::SpellGroupDamage(ROGUE_SPELLS), per_rank: 0.01,
    },
    TalentDefinition {
        id: 284, max_rank: 1, name: "Preparation",
        description: "Grants 1 automatic reset of owned Sinister Strike, Eviscerate, Blade Flurry, Rupture, Gouge, Kidney Shot and learned Sprint cooldowns every 180 sec of run time; first at 180 sec. Preserves active effects. Does not grant spells, store charges or reset Cold Blood or Adrenaline Rush recovery (Survivors automatic conversion)",
        stat: TalentStat::Preparation, per_rank: 1.0,
    },
    TalentDefinition {
        id: 280, max_rank: 1, name: "Cold Blood",
        description: "Automatically adds +100% critical chance to the next successful Sinister Strike, Eviscerate, Gouge or Kidney Shot cast, including extra targets. Ready at run start; recovers 180 sec after use. Pauses stop recovery. Excludes Blade Flurry and Rupture. Separate from the 30-point Eviscerate milestone.",
        stat: TalentStat::ColdBlood, per_rank: 1.0,
    },
    TalentDefinition {
        id: 279, max_rank: 3, name: "Improved Kidney Shot",
        description: "Kidney Shot: enemies take +3% damage per rank during its stun, starting after the triggering hit. Includes periodic damage; refreshes without stacking. Boss duration is reduced; other stuns do not extend this effect.",
        stat: TalentStat::KidneyExposure, per_rank: 0.03,
    },
    TalentDefinition {
        id: 273, max_rank: 3, name: "Ruthlessness",
        description: "Eviscerate, Rupture and Kidney Shot: 4% shorter cooldown per rank, multiplying other cooldown bonuses (Survivors finisher recovery conversion; no combo points)",
        stat: TalentStat::SpellGroupCooldown(&["eviscerate", "rupture", "kidney_shot"]), per_rank: 0.04,
    },
    TalentDefinition {
        id: 281, max_rank: 1, name: "Relentless Strikes",
        description: "Eviscerate, Rupture and Kidney Shot: 15% shorter cooldown per rank, multiplying other cooldown bonuses (Survivors finisher energy-return conversion; no energy resource)",
        stat: TalentStat::SpellGroupCooldown(&["eviscerate", "rupture", "kidney_shot"]), per_rank: 0.15,
    },
    TalentDefinition {
        id: 277, max_rank: 3, name: "Improved Slice and Dice",
        description: "Blade Flurry: 5% shorter cooldown per rank (Survivors automatic Slice and Dice conversion)",
        stat: TalentStat::SpellCooldown("blade_flurry"), per_rank: 0.05,
    },
    TalentDefinition {
        id: 247, max_rank: 2, name: "Elusiveness",
        description: "Gouge and Kidney Shot: 10% shorter cooldown per rank (Survivors defensive cooldown conversion; no stealth required)",
        stat: TalentStat::SpellGroupCooldown(&["gouge", "kidney_shot"]), per_rank: 0.10,
    },
    TalentDefinition {
        id: 382, max_rank: 1, name: "Vigor",
        description: "Rogue spells: 10% shorter cooldown (Survivors energy-capacity conversion); excludes other-class and neutral jewelry spells",
        stat: TalentStat::SpellGroupCooldown(ROGUE_SPELLS), per_rank: 0.10,
    },
    TalentDefinition {
        id: 272, max_rank: 2, name: "Remorseless Attacks",
        description: "After a kill, your next successful Sinister Strike, Eviscerate, Gouge or Kidney Shot cast gains +20% critical chance per rank; expires after 20 sec; refreshes without stacking",
        stat: TalentStat::Remorseless, per_rank: 0.20,
    },
    TalentDefinition {
        id: 186, max_rank: 5, name: "Lightning Reflexes",
        description: "+1% damage reduction per rank (Survivors dodge conversion)",
        stat: TalentStat::Reduction, per_rank: 0.01,
    },
    TalentDefinition {
        id: 187, max_rank: 5, name: "Deflection",
        description: "+3 armor per rank (Survivors parry conversion)",
        stat: TalentStat::Armor, per_rank: 3.0,
    },
    TalentDefinition {
        id: 244, max_rank: 5, name: "Camouflage",
        description: "+3% movement speed per rank; always active in Survivors, without stealth",
        stat: TalentStat::Speed, per_rank: 0.03,
    },
    TalentDefinition {
        id: 1700, max_rank: 2, name: "Sleight of Hand",
        description: "+2% damage reduction per rank (Survivors threat conversion)",
        stat: TalentStat::Reduction, per_rank: 0.02,
    },
    TalentDefinition {
        id: 203, max_rank: 3, name: "Improved Gouge",
        description: "Gouge: +25% stun duration per rank",
        stat: TalentStat::SpellDuration("gouge"), per_rank: 0.25,
    },
    TalentDefinition {
        id: 269, max_rank: 5, name: "Lethality",
        description: "Sinister Strike and Gouge: +6% critical damage per rank; excludes finishers and periodic damage",
        stat: TalentStat::SpellGroupCritDamage(&["sinister_strike", "gouge"]), per_rank: 0.06,
    },
    TalentDefinition {
        id: 1122, max_rank: 3, name: "Aggression",
        description: "Sinister Strike and Eviscerate: +2% damage per rank",
        stat: TalentStat::SpellGroupDamage(&["sinister_strike", "eviscerate"]), per_rank: 0.02,
    },
    TalentDefinition {
        id: 276, max_rank: 3, name: "Improved Eviscerate",
        description: "Eviscerate: +5% damage per rank",
        stat: TalentStat::SpellDamage("eviscerate"), per_rank: 0.05,
    },
    TalentDefinition {
        id: 270, max_rank: 5, name: "Malice",
        description: "Rogue spells: +1% critical chance per rank; excludes other-class and neutral jewelry spells",
        stat: TalentStat::SpellGroupCrit(ROGUE_SPELLS), per_rank: 0.01,
    },
    TalentDefinition {
        id: 201, max_rank: 2, name: "Improved Sinister Strike",
        description: "Sinister Strike: 5% shorter cooldown per rank (Survivors energy-cost conversion)",
        stat: TalentStat::SpellCooldown("sinister_strike"), per_rank: 0.05,
    },
    TalentDefinition {
        id: 1123, max_rank: 3, name: "Serrated Blades",
        description: "Rupture: +10% damage per rank; armor penetration is not simulated",
        stat: TalentStat::SpellDamage("rupture"), per_rank: 0.10,
    },
    TalentDefinition {
        id: 1702, max_rank: 5, name: "Deadliness",
        description: "Rogue spells: +2% damage per rank (Survivors attack-power conversion); excludes other-class and neutral jewelry spells",
        stat: TalentStat::SpellGroupDamage(ROGUE_SPELLS), per_rank: 0.02,
    },
];

const PRIEST_HOLY: &[&str] = &["smite", "holy_fire", "holy_nova"];
const PRIEST_CAST_HOLY: &[&str] = &["smite", "holy_fire"];
const PRIEST_SHADOW: &[&str] = &["shadow_word_pain", "mind_blast", "psychic_scream"];

// Vanilla identities and rank caps; automatic-cast conversions are labeled in descriptions.
#[rustfmt::skip]
const PRIEST_TALENTS: &[TalentDefinition] = &[
    TalentDefinition {
        id: 463, max_rank: 5, name: "Shadow Focus",
        description: "Mind Blast and Shadow Word: Pain: +2% direct and periodic damage per rank. Excludes Holy spells, Psychic Scream and other-class abilities (Survivors hit-chance conversion)",
        stat: TalentStat::SpellGroupDamage(&["mind_blast", "shadow_word_pain"]), per_rank: 0.02,
    },
    TalentDefinition {
        id: 361, max_rank: 3, name: "Inspiration",
        description: "Holy Nova restoring health grants +8% armor per rank for 15 sec. Refreshes without stacking; full-health casts, regeneration and other healing do not trigger it. Requires acquiring Holy Nova and unlocking its healing (Survivors conversion: no healing critical required)",
        stat: TalentStat::InspirationArmor, per_rank: 0.08,
    },
    TalentDefinition {
        id: 1636, max_rank: 3, name: "Blessed Recovery",
        description: "Restore 7% of health damage taken per rank over 6 sec after any nonlethal hit. Further hits add their healing to the unhealed amount and restart the duration. Shields and immunity do not trigger it; cannot revive you (Survivors conversion: incoming critical hits are not modeled)",
        stat: TalentStat::BlessedRecovery, per_rank: 0.07,
    },
    TalentDefinition { id: 483, max_rank: 2, name: "Improved Fade",
        description: "Fade: 10% shorter cooldown per rank. Preserves its 4 sec escape and 40% movement speed bonus; learned at hero level 5 and activated with F (Survivors scaled cooldown conversion)",
        stat: TalentStat::SpellCooldown("fade"), per_rank: 0.10 },
    TalentDefinition { id: 322, max_rank: 1, name: "Power Infusion",
        description: "Automatically grants +20% Priest direct and periodic damage, unlocked Holy Nova healing and owned Renew healing for 15 sec after a successful damaging Priest cast. Ready at run start; recovers 180 sec after use. Pauses stop timers. Cast damage keeps its bonus; Renew checks the active window. Excludes shields, lifesteal, other regeneration and other-class spells (Survivors self-only conversion)",
        stat: TalentStat::PowerInfusion, per_rank: 0.20 },
    TalentDefinition {
        id: 348, max_rank: 1, name: "Inner Focus",
        description: "The next successful Smite, Holy Fire, Holy Nova or Mind Blast gains 25 percentage points critical chance for all its direct hits, then recovers after 180 sec of active run time. Ready at run start; no stored charges. Excludes periodic ticks, shields, fear and other-class spells (Survivors automatic conversion; no mana costs)",
        stat: TalentStat::InnerFocus, per_rank: 0.25,
    },
    TalentDefinition { id: 341, max_rank: 5, name: "Mental Agility",
        description: "Shadow Word: Pain, Holy Nova, Psychic Scream and Power Word: Shield: 2% shorter cooldown per rank. Multiplies the Discipline 20-point milestone; does not change duration or tick rate (Survivors mana-cost conversion)",
        stat: TalentStat::SpellGroupCooldown(&["shadow_word_pain", "holy_nova", "psychic_scream", "power_word_shield"]), per_rank: 0.02 },
    TalentDefinition {
        id: 442, max_rank: 1, name: "Holy Nova",
        description: "Holy Nova heals 5% maximum health once per successful cast. Adds to the Holy 30-point milestone for 10% before Spiritual Healing. Requires acquiring Holy Nova during the run; damage and lifesteal are unchanged (Survivors conversion)",
        stat: TalentStat::SpellHeal("holy_nova"), per_rank: 0.05,
    },

    TalentDefinition {
        id: 404, max_rank: 5, name: "Spiritual Healing",
        description: "Renew and Holy Nova's unlocked healing: +2% healing per rank, multiplying Improved Renew. Does not unlock either effect; excludes damage, shields, Spirit Tap, lifesteal and other regeneration (Survivors healing scope)",
        stat: TalentStat::SpiritualHealing, per_rank: 0.02,
    },
    TalentDefinition {
        id: 406, max_rank: 3, name: "Improved Renew",
        description: "Renew: +5% regeneration per rank. Requires acquiring Renew during the run; excludes gear, racial regeneration, Spirit Tap and other healing (Survivors passive conversion)",
        stat: TalentStat::ImprovedRenew, per_rank: 0.05,
    },
    TalentDefinition {
        id: 461, max_rank: 5, name: "Shadow Weaving",
        description: "Priest Shadow damage, including Pain ticks: 20% chance per rank to add 3% Shadow vulnerability for 15 sec, up to 15%; refreshes all stacks after the triggering damage",
        stat: TalentStat::ShadowWeaving, per_rank: 0.20,
    },
    TalentDefinition {
        id: 484, max_rank: 1, name: "Vampiric Embrace",
        description: "Heal yourself for 20% of health removed by Priest Shadow damage, including Pain ticks; always active, excludes overkill (Survivors conversion)",
        stat: TalentStat::VampiricEmbrace, per_rank: 0.20,
    },
    TalentDefinition {
        id: 1638, max_rank: 2, name: "Improved Vampiric Embrace",
        description: "Vampiric Embrace heals an additional 5% of Shadow damage per rank (25% / 30% total); requires Vampiric Embrace",
        stat: TalentStat::ImprovedVampiricEmbrace, per_rank: 0.05,
    },
    TalentDefinition {
        id: 464, max_rank: 5, name: "Blackout",
        description: "Priest Shadow damage, including Pain ticks: 2% chance per rank to stun for 3 sec (bosses: 0.9 sec); refreshes without stacking",
        stat: TalentStat::Blackout, per_rank: 0.02,
    },
    TalentDefinition {
        id: 344, max_rank: 2, name: "Improved Power Word: Fortitude",
        description: "+10 maximum health per rank, always active (Survivors Fortitude conversion)",
        stat: TalentStat::Health, per_rank: 10.0,
    },
    TalentDefinition {
        id: 346, max_rank: 3, name: "Improved Inner Fire",
        description: "+2 armor per rank, always active; armor reduces each hit before percentage reduction (Survivors Inner Fire conversion)",
        stat: TalentStat::Armor, per_rank: 2.0,
    },
    TalentDefinition {
        id: 411, max_rank: 5, name: "Spell Warding",
        description: "2% reduced damage taken per rank; protects against all incoming damage (Survivors Spell Warding conversion)",
        stat: TalentStat::Reduction, per_rank: 0.02,
    },
    TalentDefinition {
        id: 1202, max_rank: 5, name: "Force of Will",
        description: "Priest Holy and Shadow spells: +1% damage and +1% direct critical chance per rank; periodic damage does not crit. Excludes shields, other-class and jewelry spells",
        stat: TalentStat::ForceOfWill, per_rank: 0.01,
    },
    TalentDefinition {
        id: 1635, max_rank: 2, name: "Holy Reach",
        description: "Smite and Holy Fire: +10% casting range per rank; Holy Nova: +10% radius per rank, still one player-centered area",
        stat: TalentStat::HolyReach, per_rank: 0.10,
    },
    TalentDefinition {
        id: 881, max_rank: 3, name: "Shadow Reach",
        description: "Shadow Word: Pain and Mind Blast: +6.67% casting range per rank (20% at rank 3); does not enlarge Psychic Scream or add targets",
        stat: TalentStat::ShadowReach, per_rank: 0.20 / 3.0,
    },
    TalentDefinition {
        id: 465, max_rank: 5, name: "Spirit Tap",
        description: "Kills have a 20% chance per rank to regenerate 1% maximum health per second for 15 sec; refreshes without stacking (Survivors spirit/mana conversion)",
        stat: TalentStat::SpiritTap, per_rank: 0.20,
    },
    TalentDefinition {
        id: 343, max_rank: 3, name: "Improved Power Word: Shield",
        description: "Power Word: Shield absorbs 5% more damage per rank; does not change its duration",
        stat: TalentStat::SpellAbsorb("power_word_shield"), per_rank: 0.05,
    },
    TalentDefinition {
        id: 401, max_rank: 5, name: "Holy Specialization",
        description: "Smite, Holy Fire and Holy Nova: +1% critical chance per rank",
        stat: TalentStat::SpellGroupCrit(PRIEST_HOLY), per_rank: 0.01,
    },
    TalentDefinition {
        id: 1181, max_rank: 5, name: "Divine Fury",
        description: "Smite and Holy Fire: 4% shorter cooldown per rank (Survivors cast-time conversion)",
        stat: TalentStat::SpellGroupCooldown(PRIEST_CAST_HOLY), per_rank: 0.04,
    },
    TalentDefinition {
        id: 403, max_rank: 2, name: "Searing Light",
        description: "Smite and Holy Fire: +5% damage per rank, including Holy Fire's burn",
        stat: TalentStat::SpellGroupDamage(PRIEST_CAST_HOLY), per_rank: 0.05,
    },
    TalentDefinition {
        id: 482, max_rank: 2, name: "Improved Shadow Word: Pain",
        description: "Shadow Word: Pain lasts 15% longer per rank; tick damage is unchanged (Survivors duration conversion)",
        stat: TalentStat::SpellDuration("shadow_word_pain"), per_rank: 0.15,
    },
    TalentDefinition {
        id: 481, max_rank: 5, name: "Improved Mind Blast",
        description: "Mind Blast: 5% shorter cooldown per rank (Survivors cooldown conversion)",
        stat: TalentStat::SpellCooldown("mind_blast"), per_rank: 0.05,
    },
    TalentDefinition {
        id: 542, max_rank: 2, name: "Improved Psychic Scream",
        description: "Psychic Scream: 10% shorter cooldown per rank; fear duration is unchanged (Survivors cooldown conversion)",
        stat: TalentStat::SpellCooldown("psychic_scream"), per_rank: 0.10,
    },
    TalentDefinition {
        id: 462, max_rank: 5, name: "Darkness",
        description: "Shadow Word: Pain, Mind Blast and Psychic Scream: +2% damage per rank; excludes other-class and jewelry spells",
        stat: TalentStat::SpellGroupDamage(PRIEST_SHADOW), per_rank: 0.02,
    },
];

pub fn definitions(class: usize) -> &'static [TalentDefinition] {
    match class {
        0 => WARRIOR_TALENTS,
        1 => PALADIN_TALENTS,
        2 => HUNTER_TALENTS,
        3 => ROGUE_TALENTS,
        4 => PRIEST_TALENTS,
        5 => shaman::DEFINITIONS,
        6 => MAGE_TALENTS,
        7 => warlock::DEFINITIONS,
        8 => druid::DEFINITIONS,
        _ => &[],
    }
}

pub fn talent_definition(class: usize, id: u32) -> Option<&'static TalentDefinition> {
    definitions(class).iter().find(|node| node.id == id)
}

/// Convert active saved DBC ranks into stat adjustments. Unknown nodes retain their
/// temporary baseline; never guess an effect from a spell name or localization.
pub fn stat_adjustment(class: usize, ranks: &[(u8, u32, u8)]) -> crate::campaign::Bonus {
    let mut bonus = crate::campaign::Bonus::default();
    for node in definitions(class) {
        let rank = bounded_rank(node, ranks);
        let value = node.per_rank * f32::from(rank);
        bonus.might -= 0.01 * f32::from(rank);
        match node.stat {
            TalentStat::Damage => bonus.might += value,
            TalentStat::Cooldown => bonus.cooldown += value,
            TalentStat::Health => bonus.health += value,
            TalentStat::Armor => bonus.armor += value,
            TalentStat::Crit => bonus.crit += value,
            TalentStat::CritDamage => bonus.crit_damage += value,
            TalentStat::Reduction => bonus.reduction += value,
            TalentStat::Speed => bonus.speed += value,
            TalentStat::SecondarySlowDuration(_)
            | TalentStat::SecondaryStunDuration(_)
            | TalentStat::SecondaryRootDuration(_)
            | TalentStat::FlatAbsorption
            | TalentStat::EnrageDamage
            | TalentStat::BloodCrazeHealing
            | TalentStat::BlessedRecovery
            | TalentStat::InspirationArmor
            | TalentStat::SpiritTap
            | TalentStat::ImprovedRenew
            | TalentStat::TotemHealing
            | TalentStat::ShamanShieldAbsorption
            | TalentStat::AncestralHealingArmor
            | TalentStat::TotemicMastery
            | TalentStat::PetWrath
            | TalentStat::PetIntimidation
            | TalentStat::PetRecovery
            | TalentStat::SpiritBond
            | TalentStat::PetHealth
            | TalentStat::PetDamage
            | TalentStat::PetCrit
            | TalentStat::PetFrenzy
            | TalentStat::Preparation
            | TalentStat::ColdSnap
            | TalentStat::PetMend
            | TalentStat::PetCooldown
            | TalentStat::PetSpeed
            | TalentStat::SpiritualHealing
            | TalentStat::MaximumHealth
            | TalentStat::SiphonLife
            | TalentStat::Nightfall
            | TalentStat::Aftermath
            | TalentStat::BattleShoutPower
            | TalentStat::BoomingVoice
            | TalentStat::HamstringRoot
            | TalentStat::WingClipRoot
            | TalentStat::InsectSwarmWeakening
            | TalentStat::CurseExhaustion
            | TalentStat::ImprovedCurseExhaustion
            | TalentStat::Pyroclasm
            | TalentStat::KidneyExposure
            | TalentStat::Remorseless
            | TalentStat::ColdBlood
            | TalentStat::ElementalMastery
            | TalentStat::InnerFocus
            | TalentStat::ImprovedHawk
            | TalentStat::AdrenalineRush
            | TalentStat::BladeFlurry
            | TalentStat::Deterrence
            | TalentStat::Combustion
            | TalentStat::ArcanePower
            | TalentStat::PowerInfusion
            | TalentStat::DivineFavor
            | TalentStat::FlurryHaste
            | TalentStat::Vengeance
            | TalentStat::CriticalBleed
            | TalentStat::CriticalBurn
            | TalentStat::WintersChill
            | TalentStat::Impact
            | TalentStat::Blackout
            | TalentStat::NaturesGrace
            | TalentStat::ShadowWeaving
            | TalentStat::VampiricEmbrace
            | TalentStat::ImprovedVampiricEmbrace
            | TalentStat::Frostbite
            | TalentStat::Shatter
            | TalentStat::Permafrost
            | TalentStat::BlizzardSlow
            | TalentStat::FlameThrowing
            | TalentStat::ArcticReach
            | TalentStat::HolyReach
            | TalentStat::ShadowReach
            | TalentStat::ArcaneInstability
            | TalentStat::ForceOfWill
            | TalentStat::ElementalPrecision
            | TalentStat::SpellGroupDamage(_)
            | TalentStat::TwoHandedDamage
            | TalentStat::OneHandedDamage
            | TalentStat::AxeCrit
            | TalentStat::PolearmCrit
            | TalentStat::DaggerCrit
            | TalentStat::FistCrit
            | TalentStat::SwordExtraAttack
            | TalentStat::MaceStun
            | TalentStat::RogueMace
            | TalentStat::SpellGroupCrit(_)
            | TalentStat::SpellGroupCritDamage(_)
            | TalentStat::SpellGroupCooldown(_)
            | TalentStat::SpellGroupRange(_)
            | TalentStat::SpellGroupRangeYards(_)
            | TalentStat::SpellAbsorb(_)
            | TalentStat::ImprovedDrainLife
            | TalentStat::Conflagrate
            | TalentStat::PresenceOfMind
            | TalentStat::SpellHealing(_)
            | TalentStat::SpellDamageAndCrit(_)
            | TalentStat::SpellDamage(_)
            | TalentStat::SpellCooldown(_)
            | TalentStat::SpellCrit(_)
            | TalentStat::SpellDuration(_)
            | TalentStat::SpellRadius(_)
            | TalentStat::SpellShield(_)
            | TalentStat::SpellImmune(_)
            | TalentStat::SpellShieldDuration(_)
            | TalentStat::SpellHeal(_) => {}
        }
    }
    bonus
}

#[cfg(test)]
mod stat_talent_tests {
    use super::*;

    #[test]
    fn kidney_exposure_requires_rogue_training_and_an_active_timer() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            for rank in [0, 1, 2, 3, 255] {
                let trained = base.with_ranks(&[(0, 279, rank)]);
                for time in [-1.0, 0.0, 0.001, 3.5] {
                    let bonus = if class == 3 && time > 0.0 {
                        0.03 * f32::from(rank.min(3))
                    } else {
                        0.0
                    };
                    assert_eq!(trained.kidney_exposure_multiplier(time), 1.0 + bonus);
                    assert_eq!(
                        trained.with_ranks(&[]).kidney_exposure_multiplier(time),
                        1.0
                    );
                }
            }
        }
        assert!(talent_definition(3, 279)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .starts_with("Kidney Shot: enemies take +3 » 6% damage"));
    }

    #[test]
    fn rogue_finisher_recovery_is_scoped_capped_and_composes() {
        assert!(talent_definition(3, 281)
            .unwrap()
            .inline_description(0)
            .unwrap()
            .starts_with("Eviscerate, Rupture and Kidney Shot: 0 » 15% shorter cooldown"));
        for class in 0..9 {
            for rank in [0, 1, 3, 255] {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                }
                .with_ranks(&[(0, 382, 1), (2, 247, 2)]);
                let trained =
                    base.with_ranks(&[(0, 382, 1), (2, 247, 2), (0, 273, rank), (0, 281, rank)]);
                for key in ROGUE_SPELLS
                    .iter()
                    .copied()
                    .chain(["fireball", "holy_nova"])
                {
                    let mut expected = base.cast(key);
                    if class == 3 && ["eviscerate", "rupture", "kidney_shot"].contains(&key) {
                        expected.cooldown *= (1. - 0.04 * f32::from(rank.min(3)))
                            * (1. - 0.15 * f32::from(rank.min(1)));
                    }
                    let actual = trained.cast(key);
                    assert!((actual.cooldown - expected.cooldown).abs() < 0.00001);
                    assert_eq!(
                        CastEffect {
                            cooldown: expected.cooldown,
                            ..actual
                        },
                        expected
                    );
                }
                assert_eq!(trained.with_ranks(&[(0, 382, 1), (2, 247, 2)]), base);
            }
        }
        assert!(talent_definition(3, 273)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .starts_with("Eviscerate, Rupture and Kidney Shot: 4 » 8% shorter cooldown"));
    }

    #[test]
    fn remorseless_chance_is_rank_bounded_and_excludes_areas_and_bleeds() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        for rank in 0..=2 {
            let trained = base.with_ranks(&[(0, 272, rank)]);
            for key in ["sinister_strike", "eviscerate", "gouge", "kidney_shot"] {
                assert_eq!(trained.remorseless_crit(key), 0.2 * f32::from(rank));
                assert_eq!(
                    trained.cast(key),
                    CastEffect::default(),
                    "not a permanent bonus"
                );
            }
            for key in ["rupture", "blade_flurry", "heroic_strike", "unknown"] {
                assert_eq!(trained.remorseless_crit(key), 0.0);
            }
        }
        assert_eq!(
            base.with_ranks(&[(0, 272, 255)]).remorseless_crit("gouge"),
            0.4
        );
        assert_eq!(
            base.with_ranks(&[(0, 272, 2)])
                .with_ranks(&[])
                .remorseless_crit("gouge"),
            0.0
        );
    }

    #[test]
    fn trueshot_is_self_scoped_composes_and_refunds() {
        for class in [0, 2, 6] {
            let base = Milestones {
                class,
                points: [30, 30, 30],
                ..Default::default()
            }
            .with_ranks(&[(1, 1362, 5), (1, 1347, 3)]);
            for rank in [0, 1, 255] {
                let trained = Milestones {
                    class,
                    points: [30, 30, 30],
                    ..Default::default()
                }
                .with_ranks(&[(1, 1362, 5), (1, 1347, 3), (1, 1361, rank)]);
                for key in HUNTER_SHOTS.iter().copied().chain([
                    "volley",
                    "explosive_trap",
                    "fireball",
                    "unknown",
                ]) {
                    let before = base.cast(key);
                    let after = trained.cast(key);
                    let factor = if class == 2 && rank > 0 && HUNTER_SHOTS.contains(&key) {
                        1.10
                    } else {
                        1.0
                    };
                    assert!(
                        (after.damage - before.damage * factor).abs() < 0.00001,
                        "{key}"
                    );
                    assert_eq!(
                        CastEffect {
                            damage: before.damage,
                            ..after
                        },
                        before
                    );
                }
                assert_eq!(trained.with_ranks(&[(1, 1362, 5), (1, 1347, 3)]), base);
            }
        }
        let node = talent_definition(2, 1361).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .starts_with("Your Hunter shots: +0 » 10% direct and periodic damage"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }

    #[test]
    fn caster_accuracy_conversions_are_capped_scoped_and_only_change_damage() {
        type AccuracyCase = (usize, u8, u32, u8, f32, &'static [&'static str]);
        let cases: &[AccuracyCase] = &[
            (2, 2, 1322, 2, 0.05, &["explosive_trap"]),
            (6, 0, 74, 2, 0.05, MAGE_ARCANE),
            (4, 2, 463, 5, 0.02, &["mind_blast", "shadow_word_pain"]),
            (
                5,
                2,
                583,
                3,
                0.01,
                &[
                    "lightning_bolt",
                    "chain_lightning",
                    "earth_shock",
                    "flame_shock",
                    "frost_shock",
                ],
            ),
            (
                7,
                0,
                1005,
                5,
                0.02,
                &["corruption", "curse_of_agony", "death_coil"],
            ),
        ];
        for &(owner, tree, id, cap, per_rank, affected) in cases {
            for class in 0..9 {
                let base = Milestones {
                    class,
                    points: [30, 30, 30],
                    ..Default::default()
                };
                for rank in [0, 1, cap, 255] {
                    let trained = base.with_ranks(&[(tree, id, rank)]);
                    for key in crate::data::CLASSES
                        .iter()
                        .flat_map(|class| class.pool.iter().copied())
                        .chain(["unknown"])
                    {
                        let before = base.cast(key);
                        let after = trained.cast(key);
                        let multiplier = if class == owner && affected.contains(&key) {
                            1. + per_rank * f32::from(rank.min(cap))
                        } else {
                            1.
                        };
                        assert!(
                            (after.damage - before.damage * multiplier).abs() < 0.00001,
                            "{id}/{class}/{key}"
                        );
                        assert_eq!(
                            CastEffect {
                                damage: before.damage,
                                ..after
                            },
                            before
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
            let node = talent_definition(owner, id).unwrap();
            assert!(node.inline_description(0).unwrap().contains(" » "));
            assert!(!node.inline_description(cap).unwrap().contains(" » "));
        }
    }

    #[test]
    fn rogue_precision_caps_ranks_scopes_damage_and_refunds_cleanly() {
        for class in [0, 3, 6] {
            let base = Milestones {
                class,
                points: [30, 30, 30],
                ..Default::default()
            };
            for rank in [0, 1, 5, 255] {
                let trained = base.with_ranks(&[(1, 181, rank)]);
                for key in ROGUE_SPELLS
                    .iter()
                    .copied()
                    .chain(["fireball", "holy_nova", "unknown"])
                {
                    let before = base.cast(key);
                    let after = trained.cast(key);
                    let multiplier = if class == 3 && ROGUE_SPELLS.contains(&key) {
                        1.0 + 0.01 * f32::from(rank.min(5))
                    } else {
                        1.0
                    };
                    assert!(
                        (after.damage - before.damage * multiplier).abs() < 0.00001,
                        "{key}"
                    );
                    assert_eq!(
                        CastEffect {
                            damage: before.damage,
                            ..after
                        },
                        before
                    );
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        assert!(talent_definition(3, 181)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .starts_with("Rogue abilities: +1 » 2% direct and periodic damage"));
    }

    #[test]
    fn rogue_cooldown_training_is_scoped_and_composes_with_milestones() {
        let base = Milestones {
            class: 3,
            points: [30, 30, 30],
            ..Default::default()
        };
        let ranks = [(0, 277, 3), (2, 247, 2), (0, 382, 1), (1, 201, 2)];
        let trained = base.with_ranks(&ranks);
        for key in ROGUE_SPELLS
            .iter()
            .copied()
            .chain(["fireball", "holy_nova", "unknown"])
        {
            let factor = match key {
                "blade_flurry" => 0.85 * 0.9,
                "gouge" | "kidney_shot" => 0.8 * 0.9,
                "sinister_strike" => 0.9 * 0.9,
                "rupture" | "eviscerate" => 0.9,
                _ => 1.0,
            };
            let mut expected = base.cast(key);
            expected.cooldown *= factor;
            assert!(
                (trained.cast(key).cooldown - expected.cooldown).abs() < 0.00001,
                "{key}"
            );
            let mut actual = trained.cast(key);
            actual.cooldown = expected.cooldown;
            assert_eq!(actual, expected, "only cooldown changes: {key}");
        }
        assert!((stat_adjustment(3, &ranks).might + 0.08).abs() < 0.00001);
        assert_eq!(trained.with_ranks(&[]), base);
    }

    #[test]
    fn rogue_defense_and_mobility_replace_damage_without_changing_spells() {
        let ranks = [(1, 186, 5), (1, 187, 5), (2, 244, 5), (2, 1700, 2)];
        let bonus = stat_adjustment(3, &ranks);
        assert!((bonus.reduction - 0.09).abs() < 0.00001);
        assert_eq!(bonus.armor, 15.0);
        assert!((bonus.speed - 0.15).abs() < 0.00001);
        assert!((bonus.might + 0.17).abs() < 0.00001);
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let trained = base.with_ranks(&ranks);
        for key in ROGUE_SPELLS {
            assert_eq!(trained.cast(key), base.cast(key));
        }
        assert_eq!(stat_adjustment(3, &[]).reduction, 0.0);
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let foreign = stat_adjustment(class, &ranks);
            assert_eq!(
                (
                    foreign.reduction,
                    foreign.armor,
                    foreign.speed,
                    foreign.might
                ),
                (0.0, 0.0, 0.0, 0.0)
            );
        }
    }

    #[test]
    fn rogue_control_and_strikes_compose_without_buffing_finishers_or_bleeds() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (1, 203, 3),
            (0, 269, 5),
            (1, 1122, 3),
            (0, 276, 3),
            (0, 270, 5),
            (2, 1702, 5),
        ]);
        for key in ROGUE_SPELLS {
            let effect = trained.cast(key);
            let expected_damage = 1.10
                * match *key {
                    "sinister_strike" => 1.06,
                    "eviscerate" => 1.06 * 1.15,
                    _ => 1.0,
                };
            assert!((effect.damage - expected_damage).abs() < 0.00001, "{key}");
            let expected_crit_damage = if matches!(*key, "sinister_strike" | "gouge") {
                0.30
            } else {
                0.0
            };
            assert!(
                (effect.crit_damage - expected_crit_damage).abs() < 0.00001,
                "{key}"
            );
            assert_eq!(effect.duration, if *key == "gouge" { 1.75 } else { 1.0 });
            assert!((effect.crit - 0.05).abs() < 0.00001);
            assert_eq!(effect.extra_targets, 0);
        }
        assert_eq!(trained.cast("heroic_strike"), CastEffect::default());
        assert_eq!(trained.with_ranks(&[]), base);
        assert_eq!(
            talent_definition(3, 203).unwrap().rank_preview(3),
            "This talent: 75% duration (maximum rank)"
        );
    }

    #[test]
    fn rogue_training_composes_only_for_its_spells_and_resets() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let ranks = [
            (0, 276, 3),
            (0, 270, 5),
            (1, 201, 2),
            (2, 1123, 3),
            (2, 1702, 5),
        ];
        let trained = base.with_ranks(&ranks);
        for key in ROGUE_SPELLS {
            let effect = trained.cast(key);
            let damage = 1.10
                * match *key {
                    "eviscerate" => 1.15,
                    "rupture" => 1.30,
                    _ => 1.0,
                };
            assert!((effect.damage - damage).abs() < 0.00001, "{key}");
            assert!((effect.crit - 0.05).abs() < 0.00001, "{key}");
            assert_eq!(
                effect.cooldown,
                if *key == "sinister_strike" { 0.9 } else { 1.0 }
            );
            assert_eq!(effect.extra_targets, 0);
        }
        for key in ["fireball", "heroic_strike", "arcane_shot", "unknown"] {
            assert_eq!(trained.cast(key), CastEffect::default());
        }
        assert_eq!(trained.with_ranks(&[]), base);
        // Supported ranks replace, rather than stack with, the provisional 1% per point.
        assert!((stat_adjustment(3, &ranks).might + 0.18).abs() < 0.00001);
        assert_eq!(stat_adjustment(3, &[]).might, 0.0);
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&ranks);
            assert_eq!(other.cast("eviscerate"), CastEffect::default());
        }
    }

    #[test]
    fn spirit_tap_chance_is_bounded_and_regeneration_is_not_rank_multiplied() {
        for class in 0..9 {
            for rank in 0..=6 {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(2, 465, rank)]);
                let chance = if class == 4 {
                    0.2 * f32::from(rank.min(5))
                } else {
                    0.0
                };
                assert_eq!(trained.spirit_tap_chance(), chance);
                assert_eq!(
                    trained.spirit_tap_regen(),
                    if chance > 0.0 { 0.01 } else { 0.0 }
                );
                assert_eq!(trained.with_ranks(&[]).spirit_tap_chance(), 0.0);
            }
        }
    }

    #[test]
    fn priest_milestones_unlock_at_exact_thresholds_and_compose_with_training() {
        for tree in 0..3 {
            for (threshold, key) in match tree {
                0 => [
                    (10, "power_word_shield"),
                    (20, "psychic_scream"),
                    (30, "power_word_shield"),
                ],
                1 => [(10, "smite"), (20, "holy_fire"), (30, "holy_nova")],
                _ => [
                    (10, "mind_blast"),
                    (20, "mind_blast"),
                    (30, "psychic_scream"),
                ],
            } {
                let mut below = Milestones {
                    class: 4,
                    ..Default::default()
                };
                below.points[tree] = threshold - 1;
                let mut at = below;
                at.points[tree] = threshold;
                assert_ne!(
                    below.cast(key),
                    at.cast(key),
                    "tree {tree} threshold {threshold}"
                );
                assert!(description(4, tree, threshold).is_some());
                assert_eq!(at.cast("shadow_bolt"), CastEffect::default());
                assert_eq!(at.cast("holy_shock"), CastEffect::default());
            }
        }
        let full = Milestones {
            class: 4,
            points: [30; 3],
            ..Default::default()
        }
        .with_ranks(&[(1, 401, 5), (0, 343, 3), (2, 542, 2)]);
        assert_eq!(full.cast("power_word_shield").immune_seconds, 1.0);
        assert_eq!(
            full.cast("power_word_shield").cooldown_advance,
            Some(("mind_blast", 2.0))
        );
        assert!((full.cast("power_word_shield").damage - 1.15).abs() < 0.00001);
        assert!((full.cast("psychic_scream").cooldown - 0.85 * 0.8).abs() < 0.00001);
        assert!((full.cast("holy_fire").crit - 0.10).abs() < 0.00001);
        assert_eq!(full.cast("holy_fire").heal_fraction, 0.03);
        assert_eq!(full.cast("holy_nova").heal_fraction, 0.05);
        assert_eq!(full.cast("holy_nova").extra_targets, 0);
        assert_eq!(full.cast("mind_blast").extra_targets, 1);
        assert_eq!(
            full.cast("mind_blast").cooldown_advance,
            Some(("shadow_word_pain", 2.0))
        );
        assert_eq!(full.cast("psychic_scream").shield_fraction, 0.05);
        assert_eq!(full.cast("psychic_scream").shield_seconds, 3.0);
    }

    #[test]
    fn priest_reach_separates_casting_range_from_area_radius() {
        let base = Milestones {
            class: 4,
            ..Default::default()
        };
        for (id, max) in [(1635, 2), (881, 3)] {
            for rank in 0..=max {
                let trained = base.with_ranks(&[(0, id, rank)]);
                for key in [
                    "smite",
                    "holy_fire",
                    "holy_nova",
                    "shadow_word_pain",
                    "mind_blast",
                    "psychic_scream",
                    "power_word_shield",
                    "shadow_bolt",
                    "holy_shock",
                ] {
                    let effect = trained.cast(key);
                    let amount = if id == 1635 { 0.1 } else { 0.2 / 3.0 } * f32::from(rank);
                    let range = if (id == 1635 && PRIEST_CAST_HOLY.contains(&key))
                        || (id == 881 && matches!(key, "shadow_word_pain" | "mind_blast"))
                    {
                        1.0 + amount
                    } else {
                        1.0
                    };
                    let radius = if id == 1635 && key == "holy_nova" {
                        1.0 + amount
                    } else {
                        1.0
                    };
                    assert!((effect.range - range).abs() < 0.00001, "{key}");
                    assert!((effect.radius - radius).abs() < 0.00001, "{key}");
                    assert_eq!(effect.extra_targets, 0);
                    assert_eq!(effect.damage, 1.0);
                }
            }
        }
    }

    #[test]
    fn force_of_will_scopes_damage_and_direct_critical_chance() {
        let base = Milestones {
            class: 4,
            ..Default::default()
        };
        for rank in 0..=5 {
            let trained = base.with_ranks(&[(0, 1202, rank)]);
            for key in crate::data::CLASSES[4].pool.iter().copied().chain([
                "shadow_bolt",
                "holy_shock",
                "fireball",
            ]) {
                let cast = trained.cast(key);
                let damaging = PRIEST_HOLY.contains(&key) || PRIEST_SHADOW.contains(&key);
                let value = f32::from(rank) * 0.01;
                assert!(
                    (cast.damage - if damaging { 1.0 + value } else { 1.0 }).abs() < 0.00001,
                    "{key}"
                );
                assert!(
                    (cast.crit
                        - if damaging && key != "shadow_word_pain" {
                            value
                        } else {
                            0.0
                        })
                    .abs()
                        < 0.00001,
                    "{key}"
                );
                assert_eq!(cast.cooldown, 1.0);
                assert_eq!(cast.duration, 1.0);
                assert_eq!(cast.extra_targets, 0);
            }
            assert_eq!(trained.with_ranks(&[]), base);
        }
        assert_eq!(
            talent_definition(4, 1202).unwrap().rank_preview(4),
            "This talent: 4 » 5% damage; 4 » 5% critical chance next rank"
        );
    }

    #[test]
    fn improved_renew_is_bounded_passive_only_and_class_specific() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            for rank in [0, 1, 2, 3, 255] {
                let trained = base.with_ranks(&[(1, 406, rank)]);
                let expected = if class == 4 {
                    1.0 + 0.05 * f32::from(rank.min(3))
                } else {
                    1.0
                };
                assert_eq!(trained.passive_multiplier("renew"), expected);
                assert_eq!(trained.passive_multiplier("fortitude"), 1.0);
                assert_eq!(trained.cast("renew"), CastEffect::default());
                assert_eq!(trained.with_ranks(&[]).passive_multiplier("renew"), 1.0);
            }
        }
    }

    #[test]
    fn priest_training_is_scoped_bounded_and_replaces_provisional_damage() {
        let base = Milestones {
            class: 4,
            ..Default::default()
        };
        let ranks: Vec<_> = PRIEST_TALENTS
            .iter()
            .map(|n| (0, n.id, n.max_rank))
            .collect();
        let trained = base.with_ranks(&ranks);
        for (key, damage, cooldown, duration, crit) in [
            ("smite", 1.155, 0.8, 1.0, 0.10),
            ("holy_fire", 1.155, 0.8, 1.0, 0.10),
            ("holy_nova", 1.05, 0.9, 1.0, 0.10),
            ("shadow_word_pain", 1.155 * 1.1, 0.9, 1.3, 0.0),
            ("mind_blast", 1.155 * 1.1, 0.75, 1.0, 0.05),
            ("psychic_scream", 1.155, 0.72, 1.0, 0.05),
            ("power_word_shield", 1.15, 0.9, 1.0, 0.0),
            ("shadow_bolt", 1.0, 1.0, 1.0, 0.0),
            ("holy_shock", 1.0, 1.0, 1.0, 0.0),
        ] {
            let cast = trained.cast(key);
            assert!((cast.damage - damage).abs() < 0.00001, "{key}");
            assert!((cast.cooldown - cooldown).abs() < 0.00001, "{key}");
            assert!((cast.duration - duration).abs() < 0.00001, "{key}");
            assert!((cast.crit - crit).abs() < 0.00001, "{key}");
            assert_eq!(cast.radius, if key == "holy_nova" { 1.2 } else { 1.0 });
            assert_eq!(cast.extra_targets, 0);
            assert_eq!(cast.shield_fraction, 0.0);
        }
        for node in PRIEST_TALENTS {
            assert_eq!(
                base.with_ranks(&[(0, node.id, 255), (0, node.id, 1)]),
                base.with_ranks(&[(0, node.id, node.max_rank)])
            );
            for rank in 0..=node.max_rank {
                assert!(
                    (stat_adjustment(4, &[(0, node.id, rank)]).might + 0.01 * f32::from(rank))
                        .abs()
                        < 0.00001
                );
            }
        }
        assert_eq!(trained.with_ranks(&[]), base);
        assert_eq!(
            talent_definition(4, 343).unwrap().rank_preview(3),
            "This talent: 15% absorption (maximum rank)"
        );
    }

    #[test]
    fn rogue_ranks_are_bounded_and_catalog_membership_is_complete() {
        assert_eq!(ROGUE_SPELLS, crate::data::CLASSES[3].pool);
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        for node in ROGUE_TALENTS {
            let capped = base.with_ranks(&[(0, node.id, node.max_rank)]);
            assert_eq!(
                base.with_ranks(&[(0, node.id, 255), (0, node.id, 1)]),
                capped
            );
            for rank in 0..=node.max_rank {
                let trained = base.with_ranks(&[(0, node.id, rank)]);
                assert_eq!(trained.with_ranks(&[]), base);
                assert!(
                    (stat_adjustment(3, &[(0, node.id, rank)]).might + 0.01 * f32::from(rank))
                        .abs()
                        < 0.00001
                );
            }
        }
    }

    #[test]
    fn class_catalogs_fit_snapshots_and_have_unique_ids() {
        for class in 0..9 {
            let nodes = definitions(class);
            assert!(nodes.len() <= Milestones::default().training.len());
            for (index, node) in nodes.iter().enumerate() {
                assert!(!nodes[..index].iter().any(|other| other.id == node.id));
            }
        }
    }

    #[test]
    fn linear_talent_descriptions_embed_values_without_repeating_units_or_rank_labels() {
        let cooldown = talent_definition(1, 1407).unwrap();
        assert_eq!(cooldown.inline_description(4).unwrap(), "Judgement: 12 » 15% shorter cooldown, multiplying Improved Judgement and milestone training (Survivors seal mana-cost conversion)");
        let armor = talent_definition(0, 140).unwrap();
        assert_eq!(armor.inline_description(0).unwrap(), "+0 » 6 armor");
        assert_eq!(armor.inline_description(255).unwrap(), "+30 armor");
        for node in (0..9).flat_map(definitions) {
            if let Some(text) = node.inline_description(0) {
                assert!(text.contains(" » "), "{}", node.name);
                assert!(!text.contains("per rank"));
                assert!(!text.contains("This talent:"));
                let (rich, range) = node.inline_description_with_values(0).unwrap();
                assert_eq!(rich, text);
                assert!(range.iter().all(|r| rich[r.clone()].contains(" » ")));
                assert!(!node.inline_description(255).unwrap().contains(" » "));
            }
        }
        let mut ambiguous = *cooldown;
        ambiguous.description = "Judgement: 13% damage and 3% cooldown per rank";
        assert!(ambiguous
            .inline_description(1)
            .unwrap()
            .contains("13% damage and 3 » 6%"));
        ambiguous.description = "3% damage and 3% cooldown per rank";
        assert!(ambiguous.inline_description(1).is_none());
    }

    #[test]
    fn percentage_point_upgrades_keep_the_unit_once_and_agree_with_the_next_value() {
        let crit = talent_definition(5, 613).unwrap();
        assert_eq!(crit.inline_description(0).unwrap(), "+0 » 1 percentage point hero critical chance, including spells and totems (Survivors conversion)");
        assert_eq!(crit.inline_description(1).unwrap(), "+1 » 2 percentage points hero critical chance, including spells and totems (Survivors conversion)");
        assert_eq!(crit.inline_description(255).unwrap(), "+5 percentage points hero critical chance, including spells and totems (Survivors conversion)");
        let claws = talent_definition(8, 798).unwrap();
        assert_eq!(claws.inline_description(1).unwrap(), "+2 » 4 percentage points hero critical chance (Survivors conversion; no form required)");
        let mut ambiguous = *crit;
        ambiguous.description = "1% damage and 1 percentage point critical chance per rank";
        assert!(ambiguous.inline_description(0).is_none());
        ambiguous.description =
            "11 percentage points damage and 1 percentage point critical chance per rank";
        assert_eq!(
            ambiguous.inline_description(1).unwrap(),
            "11 percentage points damage and 1 » 2 percentage points critical chance"
        );
        let moonfire = talent_definition(8, 763).unwrap();
        let (text, ranges) = moonfire.inline_description_with_values(1).unwrap();
        assert_eq!(text, "Moonfire: +2 » 4% direct and periodic damage and +2 » 4 percentage points critical chance");
        assert_eq!(
            ranges.iter().map(|r| &text[r.clone()]).collect::<Vec<_>>(),
            [
                "+2 » 4% direct and periodic damage",
                "+2 » 4 percentage points critical chance"
            ]
        );
        let (text, ranges) = moonfire.inline_description_with_values(255).unwrap();
        assert_eq!(
            text,
            "Moonfire: +10% direct and periodic damage and +10 percentage points critical chance"
        );
        assert_eq!(ranges.len(), 2);
        let mut ambiguous = *moonfire;
        ambiguous.description = "+2% damage and +2% chance per rank";
        assert!(ambiguous.inline_description(1).is_none());
    }

    #[test]
    fn talent_previews_show_node_contribution_and_stop_at_the_rank_cap() {
        let armor = talent_definition(0, 140).unwrap();
        assert_eq!(armor.rank_preview(0), "This talent: 0 to 6 armor next rank");
        assert_eq!(
            armor.rank_preview(4),
            "This talent: 24 to 30 armor next rank"
        );
        assert_eq!(
            armor.rank_preview(255),
            "This talent: 30 armor (maximum rank)"
        );
        assert_eq!(
            talent_definition(0, 150).unwrap().rank_preview(1),
            "This talent: 15 to 30% duration next rank"
        );
        assert_eq!(
            talent_definition(0, 153).unwrap().rank_preview(0),
            "This talent: 0 to 5% maximum health healed per cast next rank"
        );
        for node in (0..9).flat_map(definitions) {
            assert!(!node.rank_preview(node.max_rank).contains("next rank"));
            assert!(node.rank_preview(0).contains("next rank"));
        }
    }

    #[test]
    fn warrior_bleed_training_composes_without_recursive_or_cross_class_procs() {
        let base = Milestones {
            class: 0,
            points: [10, 0, 0],
            ..Default::default()
        };
        for rank in 0..=3 {
            let trained = base.with_ranks(&[(0, 121, rank), (0, 127, 3)]);
            let expected = 100.0 * (0.12 + 0.04 * f32::from(rank)) * 1.3;
            for key in ["cleave", "heroic_strike", "execute", "mortal_strike"] {
                assert!((trained.critical_bleed(key, 100., true) - expected).abs() < 0.00001);
                assert_eq!(trained.critical_bleed(key, 100., false), 0.);
                assert_eq!(trained.critical_bleed(key, f32::NAN, true), 0.);
            }
            for key in ["deep_wounds", "whirlwind", "thunder_clap", "retaliation"] {
                assert_eq!(trained.critical_bleed(key, 100., true), 0.);
            }
            assert_eq!(
                trained.with_ranks(&[]).critical_bleed("cleave", 100., true),
                12.
            );
        }
        // Improved Rend improves a bleed; it does not grant one by itself.
        let empty = Milestones::default();
        assert_eq!(
            empty
                .with_ranks(&[(0, 127, 3)])
                .critical_bleed("cleave", 100., true),
            0.
        );
        let outsider = Milestones {
            class: 1,
            points: [30; 3],
            ..Default::default()
        }
        .with_ranks(&[(0, 121, 3), (0, 127, 3)]);
        assert_eq!(outsider.critical_bleed("cleave", 100., true), 0.);
        let capped = base.with_ranks(&[(0, 121, 255), (0, 127, 255)]);
        assert!((capped.critical_bleed("cleave", 100., true) - 31.2).abs() < 0.00001);
        assert_eq!(
            talent_definition(0, 121).unwrap().rank_preview(1),
            "This talent: 4 to 8% of direct critical damage as bleed next rank"
        );
    }

    #[test]
    fn warrior_control_nodes_add_to_milestones_and_preserve_boss_resistance() {
        for points in [[0; 3], [30; 3]] {
            let base = Milestones {
                class: 0,
                points,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(1, 160, 1), (2, 152, 1)]);
            let slow = if points[1] >= 10 { 5. } else { 2. };
            let stun = if points[2] >= 20 { 2. } else { 1. };
            assert_eq!(
                trained.secondary_slow("demoralizing_shout", false),
                Some((0.5, slow))
            );
            assert_eq!(
                trained.secondary_slow("demoralizing_shout", true),
                Some((0.75, slow))
            );
            assert_eq!(trained.secondary_stun("heroic_strike", false), stun);
            assert!((trained.secondary_stun("heroic_strike", true) - stun * 0.3).abs() < 0.00001);
            assert_eq!(trained.secondary_slow("cleave", false), None);
            assert_eq!(trained.secondary_stun("cleave", false), 0.);
            assert_eq!(
                trained.cast("demoralizing_shout"),
                base.cast("demoralizing_shout")
            );
            assert_eq!(trained.with_ranks(&[]), base);
            assert_eq!(base.with_ranks(&[(1, 160, 255), (2, 152, 255)]), trained);
            let outsider = Milestones {
                class: 1,
                ..trained
            };
            assert_eq!(outsider.secondary_slow("demoralizing_shout", false), None);
            assert_eq!(outsider.secondary_stun("heroic_strike", false), 0.);
        }
        assert_eq!(
            talent_definition(0, 160).unwrap().rank_preview(0),
            "This talent: 0 to 2 s added slow duration next rank"
        );
        assert_eq!(
            talent_definition(0, 152).unwrap().rank_preview(1),
            "This talent: 1 s added stun duration (maximum rank)"
        );
    }

    #[test]
    fn shield_nodes_compose_and_duration_alone_grants_no_absorb() {
        for points in [0, 30] {
            let base = Milestones {
                class: 0,
                points: [0, 0, points],
                ..Default::default()
            };
            for rank in 0..=5 {
                let trained = base.with_ranks(&[(2, 1601, rank), (2, 145, 3), (2, 148, 1)]);
                let effect = trained.cast("heroic_strike");
                let expected = if points == 30 { 0.06 } else { 0.03 } + f32::from(rank) * 0.01;
                assert!((effect.shield_fraction - expected).abs() < 0.00001);
                assert_eq!(effect.shield_seconds, 4.5);
                assert_eq!(trained.cast("cleave"), base.cast("cleave"));
                assert_eq!(trained.with_ranks(&[]), base);
            }
        }
        let duration = Milestones::default()
            .with_ranks(&[(2, 145, 3)])
            .cast("heroic_strike");
        assert_eq!(
            (duration.shield_fraction, duration.shield_seconds),
            (0., 0.)
        );
        let shield = Milestones::default()
            .with_ranks(&[(2, 1601, 1)])
            .cast("heroic_strike");
        assert_eq!((shield.shield_fraction, shield.shield_seconds), (0.01, 3.));
        assert_eq!(
            talent_definition(0, 145).unwrap().rank_preview(1),
            "This talent: 0.5 to 1 s added shield duration next rank"
        );
    }

    #[test]
    fn ranged_talent_groups_change_only_targeting_distance() {
        let cases: &[(usize, u32, u8, &[&str], bool)] = &[
            (
                2,
                1352,
                3,
                &[
                    "arcane_shot",
                    "multi_shot",
                    "serpent_sting",
                    "concussive_shot",
                    "aimed_shot",
                    "scorpid_sting",
                    "volley",
                ],
                true,
            ),
            (5, 1641, 2, &["lightning_bolt", "chain_lightning"], false),
            (
                7,
                1021,
                2,
                &["corruption", "curse_of_agony", "death_coil"],
                false,
            ),
            (
                7,
                964,
                2,
                &[
                    "shadow_bolt",
                    "immolate",
                    "searing_pain",
                    "shadowburn",
                    "rain_of_fire",
                ],
                false,
            ),
            (
                8,
                764,
                2,
                &[
                    "wrath",
                    "moonfire",
                    "starfire",
                    "entangling_roots",
                    "hurricane",
                ],
                false,
            ),
        ];
        for &(class, id, max, spells, flat) in cases {
            for rank in [0, 1, 2, 3, 255] {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, id, rank)]);
                for def in crate::data::ABILITIES {
                    let mut expected = base.cast(def.key);
                    if spells.contains(&def.key) {
                        if flat {
                            expected.range_yards += 2. * f32::from(rank.min(max));
                        } else {
                            expected.range *= 1. + 0.10 * f32::from(rank.min(max));
                        }
                    }
                    assert_eq!(trained.cast(def.key), expected, "{id}: {}", def.key);
                }
                assert_eq!(trained.with_ranks(&[]), base);
                for other in (0..9).filter(|other| *other != class) {
                    let foreign = Milestones {
                        class: other,
                        ..Default::default()
                    }
                    .with_ranks(&[(0, id, rank)]);
                    assert_eq!(
                        foreign,
                        Milestones {
                            class: other,
                            ..Default::default()
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn paladin_seal_training_stacks_without_changing_other_cast_fields() {
        for class in 0..9 {
            for points in [[0; 3], [30; 3]] {
                for rank in [0, 1, 5, 255] {
                    let base = Milestones {
                        class,
                        points,
                        ..Default::default()
                    }
                    .with_ranks(&[(2, 1631, 2), (2, 1409, 1)]);
                    let trained = Milestones {
                        class,
                        points,
                        ..Default::default()
                    }
                    .with_ranks(&[
                        (2, 1631, 2),
                        (2, 1409, 1),
                        (2, 1407, rank),
                        (0, 1463, rank),
                    ]);
                    for ability in crate::data::ABILITIES {
                        let mut expected = base.cast(ability.key);
                        if class == 1 && ability.key == "judgement" {
                            let value = 0.03 * f32::from(rank.min(5));
                            expected.damage *= 1.0 + value;
                            expected.cooldown *= 1.0 - value;
                        }
                        let mut actual = trained.cast(ability.key);
                        assert!((actual.damage - expected.damage).abs() < 0.00001);
                        assert!((actual.cooldown - expected.cooldown).abs() < 0.00001);
                        actual.damage = expected.damage;
                        actual.cooldown = expected.cooldown;
                        assert_eq!(actual, expected, "{class} {}", ability.key);
                    }
                    assert_eq!(
                        trained.with_ranks(&[]),
                        Milestones {
                            class,
                            points,
                            ..Default::default()
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn paladin_retribution_damage_is_scoped_bounded_and_removable() {
        for rank in [0, 1, 5, 255] {
            let base = Milestones {
                class: 1,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(2, 1402, rank), (2, 1409, rank)]);
            let expected = 0.03 * f32::from(rank.min(5));
            assert!((trained.vengeance_bonus(true) - expected).abs() < 0.00001);
            assert_eq!(trained.vengeance_bonus(false), 0.);
            for key in PALADIN_HOLY {
                let effect = trained.cast(key);
                assert!((effect.damage - if rank > 0 { 1.1 } else { 1. }).abs() < 0.00001);
                assert_eq!(effect.radius, 1.);
                assert_eq!(effect.extra_targets, 0);
                assert_eq!(
                    trained.triggers_vengeance(key, 10., true),
                    rank > 0 && *key != "consecration"
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert!(!trained.triggers_vengeance(key, damage, true));
                }
                assert!(!trained.triggers_vengeance(key, 10., false));
            }
            for key in [
                "divine_shield",
                "smite",
                "holy_nova",
                "fireball",
                "rejuvenation",
                "unknown",
            ] {
                assert_eq!(trained.cast(key), CastEffect::default());
                assert!(!trained.triggers_vengeance(key, 10., true));
            }
            assert_eq!(trained.with_ranks(&[]), base);
            assert_eq!(
                Milestones::default()
                    .with_ranks(&[(2, 1402, rank)])
                    .vengeance_bonus(true),
                0.
            );
        }
    }

    #[test]
    fn flurry_requires_trained_direct_criticals() {
        for rank in 0..=5 {
            let trained = Milestones::default().with_ranks(&[(1, 156, rank)]);
            assert_eq!(trained.flurry_haste(false), 0.0);
            assert!((trained.flurry_haste(true) - 0.05 * f32::from(rank)).abs() < 0.00001);
            for key in ["cleave", "heroic_strike", "execute", "mortal_strike"] {
                assert_eq!(trained.triggers_flurry(key, 10., true), rank > 0);
                assert!(!trained.triggers_flurry(key, 10., false));
                assert!(!trained.triggers_flurry(key, 0., true));
                assert!(!trained.triggers_flurry(key, f32::NAN, true));
            }
            for key in [
                "deep_wounds",
                "whirlwind",
                "retaliation",
                "sweeping_strikes",
                "fireball",
            ] {
                assert!(!trained.triggers_flurry(key, 10., true));
            }
            assert_eq!(trained.with_ranks(&[]).flurry_haste(true), 0.);
            assert_eq!(
                Milestones {
                    class: 1,
                    ..trained
                }
                .flurry_haste(true),
                0.
            );
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(1, 156, 255)])
                .flurry_haste(true),
            0.25
        );
        for class in 0..9 {
            assert!(definitions(class).len() <= Milestones::default().training.len());
        }
    }

    #[test]
    fn blood_craze_training_is_bounded_and_class_specific() {
        for rank in 0..=3 {
            let trained = Milestones::default().with_ranks(&[(1, 661, rank)]);
            assert!((trained.blood_craze_fraction() - f32::from(rank) * 0.01).abs() < 0.00001);
            assert_eq!(trained.with_ranks(&[]).blood_craze_fraction(), 0.);
            assert_eq!(
                Milestones {
                    class: 1,
                    ..trained
                }
                .blood_craze_fraction(),
                0.
            );
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(1, 661, 255)])
                .blood_craze_fraction(),
            0.03
        );
    }

    #[test]
    fn overpower_and_slam_training_is_spell_specific_and_resettable() {
        let base = Milestones::default();
        let trained = base.with_ranks(&[(0, 131, 2), (1, 168, 5)]);
        assert!((trained.cast("heroic_strike").crit - 0.2).abs() < 0.00001);
        assert!((trained.cast("mortal_strike").cooldown - 0.75).abs() < 0.00001);
        assert_eq!(trained.cast("cleave"), base.cast("cleave"));
        assert_eq!(trained.cast("heroic_strike").cooldown, 1.0);
        assert_eq!(trained.cast("mortal_strike").crit, 0.0);
        assert_eq!(trained.with_ranks(&[]), base);
        let other = Milestones { class: 1, ..base }.with_ranks(&[(0, 131, 2), (1, 168, 5)]);
        assert_eq!(other.cast("heroic_strike"), CastEffect::default());
        assert_eq!(other.cast("mortal_strike"), CastEffect::default());
    }

    #[test]
    fn hunter_control_training_is_bounded_source_specific_and_boss_scaled() {
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(1, 1353, 1), (2, 1304, 5), (2, 1325, 1)]);
        assert_eq!(trained.secondary_stun("concussive_shot", false), 1.);
        assert_eq!(trained.secondary_stun("concussive_shot", true), 0.3);
        assert_eq!(trained.secondary_stun("serpent_sting", false), 1.5);
        assert!((trained.secondary_stun("serpent_sting", true) - 0.45).abs() < 0.00001);
        for rank in 0..=5 {
            let roots = base.with_ranks(&[(2, 1304, rank)]);
            assert_eq!(
                roots.secondary_root("explosive_trap", false),
                0.4 * f32::from(rank)
            );
            assert!(
                (roots.secondary_root("explosive_trap", true) - 0.12 * f32::from(rank)).abs()
                    < 0.00001
            );
        }
        for key in ["multi_shot", "volley", "heroic_strike", "fireball"] {
            assert_eq!(trained.secondary_stun(key, false), 0.);
            assert_eq!(trained.secondary_root(key, false), 0.);
        }
        assert_eq!(trained.with_ranks(&[]), base);
        assert_eq!(
            base.with_ranks(&[(1, 1353, 255), (2, 1304, 255), (2, 1325, 255)]),
            trained
        );
        let other = Milestones::default().with_ranks(&[(1, 1353, 1), (2, 1304, 5), (2, 1325, 1)]);
        assert_eq!(other.secondary_stun("serpent_sting", false), 0.);
        assert_eq!(other.secondary_root("explosive_trap", false), 0.);
    }

    #[test]
    fn hunter_shot_groups_compose_without_affecting_other_abilities() {
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        for (id, cap) in [(1344, 5), (1349, 5), (1347, 3), (1362, 5)] {
            for rank in 0..=cap {
                let trained = base.with_ranks(&[(1, id, rank)]);
                for key in [
                    "arcane_shot",
                    "multi_shot",
                    "serpent_sting",
                    "concussive_shot",
                    "aimed_shot",
                    "scorpid_sting",
                    "volley",
                    "explosive_trap",
                    "fireball",
                ] {
                    let shot = HUNTER_SHOTS.contains(&key);
                    let barrage = matches!(key, "multi_shot" | "volley");
                    let effect = trained.cast(key);
                    let value = f32::from(rank);
                    let expected = CastEffect {
                        damage: 1.
                            + if id == 1347 && barrage {
                                0.04 * value
                            } else if id == 1362 && shot {
                                0.01 * value
                            } else {
                                0.
                            },
                        crit: if id == 1344 && shot { 0.01 * value } else { 0. },
                        crit_damage: if id == 1349 && shot { 0.06 * value } else { 0. },
                        ..Default::default()
                    };
                    assert_eq!(effect, expected, "node {id} rank {rank}: {key}");
                }
                assert_eq!(trained.with_ranks(&[]), base);
            }
            assert_eq!(
                base.with_ranks(&[(1, id, 255), (1, id, 1)]),
                base.with_ranks(&[(1, id, cap)])
            );
        }
        let trained = base.with_ranks(&[(1, 1344, 5), (1, 1349, 5), (1, 1347, 3), (1, 1362, 5)]);
        assert!((trained.cast("multi_shot").damage - 1.176).abs() < 0.00001);
        for class in [0, 1, 3, 4, 5, 6, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(1, 1344, 5), (1, 1349, 5), (1, 1347, 3), (1, 1362, 5)]);
            assert_eq!(other.cast("multi_shot"), CastEffect::default());
        }
    }

    #[test]
    fn hunter_training_is_bounded_spell_specific_and_resettable() {
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let ranks = [
            (0, 1395, 3),
            (0, 1384, 2),
            (1, 1341, 5),
            (1, 1345, 1),
            (1, 1346, 5),
            (1, 1348, 5),
            (2, 1311, 5),
            (2, 1306, 2),
            (2, 1321, 3),
        ];
        let trained = base.with_ranks(&ranks);
        for (key, damage, cooldown, duration) in [
            ("arcane_shot", 1., 0.75, 1.),
            ("aimed_shot", 1.25, 1., 1.),
            ("serpent_sting", 1.1, 1., 1.),
            ("concussive_shot", 1., 1., 1.5),
            ("explosive_trap", 1.3, 1., 1.),
            ("multi_shot", 1., 1., 1.),
            ("volley", 1., 1., 1.),
        ] {
            let cast = trained.cast(key);
            assert!((cast.damage - damage).abs() < 0.00001, "{key}");
            assert!((cast.cooldown - cooldown).abs() < 0.00001, "{key}");
            assert!((cast.duration - duration).abs() < 0.00001, "{key}");
            assert_eq!(cast.extra_targets, 0);
        }
        let bonus = stat_adjustment(2, &ranks);
        assert_eq!(bonus.armor, 24.);
        assert!((bonus.speed - 0.08).abs() < 0.00001);
        assert!((bonus.reduction - 0.05).abs() < 0.00001);
        assert!((bonus.crit - 0.03).abs() < 0.00001);
        assert_eq!(trained.with_ranks(&[]), base);
        let excessive: Vec<_> = ranks
            .iter()
            .flat_map(|&(tree, id, _)| [(tree, id, 255), (tree, id, 1)])
            .collect();
        assert_eq!(base.with_ranks(&excessive), trained);
        for class in [0, 1, 3, 4, 5, 6, 7, 8] {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&ranks);
            assert_eq!(other.cast("arcane_shot"), CastEffect::default());
            assert_eq!(other.cast("explosive_trap"), CastEffect::default());
        }
    }

    #[test]
    fn warrior_rank_reset_removes_previous_training() {
        let base = Milestones {
            class: 0,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 124, 3), (1, 166, 3), (1, 158, 5)]);
        assert!((trained.cast("heroic_strike").cooldown - 0.85).abs() < 0.00001);
        assert!((trained.cast("cleave").damage - 1.3).abs() < 0.00001);
        assert_eq!(trained.cast("demoralizing_shout").radius, 1.5);
        assert_eq!(trained.cast("whirlwind"), CastEffect::default());
        assert_eq!(trained.with_ranks(&[]), base);
    }

    #[test]
    fn each_named_stat_talent_replaces_its_generic_damage_at_every_rank() {
        for (class, node) in
            (0..9).flat_map(|class| definitions(class).iter().map(move |node| (class, node)))
        {
            for rank in 0..=node.max_rank {
                let mut total = crate::campaign::Bonus {
                    might: 0.01 * f32::from(rank),
                    ..Default::default()
                };
                total.add(stat_adjustment(class, &[(0, node.id, rank)]));
                let value = node.per_rank * f32::from(rank);
                let actual = match node.stat {
                    TalentStat::Damage => total.might,
                    TalentStat::Cooldown => total.cooldown,
                    TalentStat::Health => total.health,
                    TalentStat::Armor => total.armor,
                    TalentStat::Crit => total.crit,
                    TalentStat::CritDamage => total.crit_damage,
                    TalentStat::Reduction => total.reduction,
                    TalentStat::Speed => total.speed,
                    TalentStat::SecondarySlowDuration(_)
                    | TalentStat::SecondaryStunDuration(_)
                    | TalentStat::SecondaryRootDuration(_)
                    | TalentStat::EnrageDamage
                    | TalentStat::BloodCrazeHealing
                    | TalentStat::BlessedRecovery
                    | TalentStat::InspirationArmor
                    | TalentStat::SpiritTap
                    | TalentStat::ImprovedRenew
                    | TalentStat::TotemHealing
                    | TalentStat::ShamanShieldAbsorption
                    | TalentStat::AncestralHealingArmor
                    | TalentStat::TotemicMastery
                    | TalentStat::PetWrath
                    | TalentStat::PetIntimidation
                    | TalentStat::PetRecovery
                    | TalentStat::SpiritBond
                    | TalentStat::PetHealth
                    | TalentStat::PetDamage
                    | TalentStat::PetCrit
                    | TalentStat::PetFrenzy
                    | TalentStat::Preparation
                    | TalentStat::ColdSnap
                    | TalentStat::PetMend
                    | TalentStat::PetCooldown
                    | TalentStat::PetSpeed
                    | TalentStat::SpiritualHealing
                    | TalentStat::MaximumHealth
                    | TalentStat::FlatAbsorption
                    | TalentStat::SiphonLife
                    | TalentStat::Nightfall
                    | TalentStat::Aftermath
                    | TalentStat::BattleShoutPower
                    | TalentStat::BoomingVoice
                    | TalentStat::HamstringRoot
                    | TalentStat::WingClipRoot
                    | TalentStat::InsectSwarmWeakening
                    | TalentStat::CurseExhaustion
                    | TalentStat::ImprovedCurseExhaustion
                    | TalentStat::Pyroclasm
                    | TalentStat::KidneyExposure
                    | TalentStat::Remorseless
                    | TalentStat::ColdBlood
                    | TalentStat::ElementalMastery
                    | TalentStat::InnerFocus
                    | TalentStat::ImprovedHawk
                    | TalentStat::AdrenalineRush
                    | TalentStat::BladeFlurry
                    | TalentStat::Deterrence
                    | TalentStat::Combustion
                    | TalentStat::ArcanePower
                    | TalentStat::PowerInfusion
                    | TalentStat::DivineFavor
                    | TalentStat::FlurryHaste
                    | TalentStat::Vengeance
                    | TalentStat::CriticalBleed
                    | TalentStat::CriticalBurn
                    | TalentStat::WintersChill
                    | TalentStat::Impact
                    | TalentStat::Blackout
                    | TalentStat::NaturesGrace
                    | TalentStat::ShadowWeaving
                    | TalentStat::VampiricEmbrace
                    | TalentStat::ImprovedVampiricEmbrace
                    | TalentStat::Frostbite
                    | TalentStat::Shatter
                    | TalentStat::Permafrost
                    | TalentStat::BlizzardSlow
                    | TalentStat::FlameThrowing
                    | TalentStat::ArcticReach
                    | TalentStat::HolyReach
                    | TalentStat::ShadowReach
                    | TalentStat::ArcaneInstability
                    | TalentStat::ForceOfWill
                    | TalentStat::ElementalPrecision
                    | TalentStat::SpellGroupDamage(_)
                    | TalentStat::TwoHandedDamage
                    | TalentStat::OneHandedDamage
                    | TalentStat::AxeCrit
                    | TalentStat::PolearmCrit
                    | TalentStat::DaggerCrit
                    | TalentStat::FistCrit
                    | TalentStat::SwordExtraAttack
                    | TalentStat::MaceStun
                    | TalentStat::RogueMace
                    | TalentStat::SpellGroupCrit(_)
                    | TalentStat::SpellGroupCritDamage(_)
                    | TalentStat::SpellGroupCooldown(_)
                    | TalentStat::SpellGroupRange(_)
                    | TalentStat::SpellGroupRangeYards(_)
                    | TalentStat::SpellAbsorb(_)
                    | TalentStat::ImprovedDrainLife
                    | TalentStat::Conflagrate
                    | TalentStat::PresenceOfMind
                    | TalentStat::SpellHealing(_)
                    | TalentStat::SpellDamageAndCrit(_)
                    | TalentStat::SpellDamage(_)
                    | TalentStat::SpellCooldown(_)
                    | TalentStat::SpellCrit(_)
                    | TalentStat::SpellDuration(_)
                    | TalentStat::SpellRadius(_)
                    | TalentStat::SpellShield(_)
                    | TalentStat::SpellImmune(_)
                    | TalentStat::SpellShieldDuration(_)
                    | TalentStat::SpellHeal(_) => {
                        assert!(total.might.abs() < 0.00001);
                        continue;
                    }
                };
                assert!(
                    (actual - value).abs() < 0.00001,
                    "{} rank {rank}",
                    node.name
                );
                if !matches!(node.stat, TalentStat::Damage) {
                    assert!(total.might.abs() < 0.00001);
                }
            }
        }
    }

    #[test]
    fn spell_training_changes_only_its_spell_and_composes_with_milestones() {
        for (class, node) in
            (0..9).flat_map(|class| definitions(class).iter().map(move |node| (class, node)))
        {
            let key = match node.stat {
                TalentStat::SpellDamage(key)
                | TalentStat::SpellCooldown(key)
                | TalentStat::SpellCrit(key)
                | TalentStat::SpellDuration(key)
                | TalentStat::SpellRadius(key)
                | TalentStat::SpellHeal(key) => key,
                _ => continue,
            };
            for rank in 0..=node.max_rank {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, node.id, rank)]);
                let before = base.cast(key);
                let after = trained.cast(key);
                let value = node.per_rank * f32::from(rank);
                match node.stat {
                    TalentStat::SpellDamage(_) => {
                        assert!((after.damage - before.damage * (1.0 + value)).abs() < 0.00001)
                    }
                    TalentStat::SpellCooldown(_) => {
                        assert!((after.cooldown - before.cooldown * (1.0 - value)).abs() < 0.00001)
                    }
                    TalentStat::SpellCrit(_) => {
                        assert!((after.crit - before.crit - value).abs() < 0.00001)
                    }
                    TalentStat::SpellDuration(_) => {
                        assert!((after.duration - before.duration * (1.0 + value)).abs() < 0.00001)
                    }
                    TalentStat::SpellRadius(_) => {
                        assert!((after.radius - before.radius * (1.0 + value)).abs() < 0.00001)
                    }
                    TalentStat::SpellHeal(_) => assert!(
                        (after.heal_fraction - before.heal_fraction - value).abs() < 0.00001
                    ),
                    _ => unreachable!(),
                }
                for other in crate::data::CLASSES
                    .iter()
                    .flat_map(|class| class.ability_keys())
                {
                    if other != key {
                        assert_eq!(trained.cast(other), base.cast(other));
                    }
                }
                assert_eq!(after.extra_targets, before.extra_targets);
                if !matches!(node.stat, TalentStat::SpellHeal(_)) {
                    assert_eq!(after.heal_fraction, before.heal_fraction);
                }
                let outsider = Milestones {
                    class: (class + 1) % 9,
                    ..Default::default()
                }
                .with_ranks(&[(0, node.id, rank)]);
                assert_eq!(outsider.cast(key), CastEffect::default());
            }
            let base = Milestones {
                class,
                ..Default::default()
            };
            assert_eq!(
                base.with_ranks(&[(0, node.id, 255), (0, node.id, 1)])
                    .cast(key),
                base.with_ranks(&[(0, node.id, node.max_rank)]).cast(key)
            );
        }
    }

    #[test]
    fn mage_reach_talents_distinguish_target_range_from_area_radius() {
        for rank in 0..=3 {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(1, 28, rank), (2, 741, rank)]);
            let mult = 1. + 0.1 * f32::from(rank.min(2));
            for key in [
                "fireball",
                "pyroblast",
                "fire_blast",
                "flamestrike",
                "frostbolt",
                "blizzard",
            ] {
                let cast = trained.cast(key);
                assert_eq!(cast.range, mult, "{key}");
                assert_eq!(cast.radius, 1., "{key}");
                assert_eq!(cast.extra_targets, 0);
            }
            for key in ["frost_nova", "cone_of_cold"] {
                assert_eq!(trained.cast(key).radius, mult);
                assert_eq!(trained.cast(key).range, 1.);
            }
            assert_eq!(trained.cast("arcane_explosion"), CastEffect::default());
            assert_eq!(
                trained.with_ranks(&[]).cast("fireball"),
                CastEffect::default()
            );
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(1, 28, 2), (2, 741, 2)])
                .cast("fireball"),
            CastEffect::default()
        );
    }

    #[test]
    fn permafrost_preview_compares_both_benefits_and_caps_the_rank() {
        let node = talent_definition(6, 65).unwrap();
        assert_eq!(node.rank_preview(2), "This talent: 6 » 9 percentage points stronger slows; 2 » 3 sec added duration next rank");
        assert_eq!(
            node.rank_preview(255),
            "This talent: 9 percentage points stronger slows; 3 sec added duration (maximum rank)"
        );
    }

    #[test]
    fn frost_slow_talents_compose_without_extending_freezes_or_other_schools() {
        use crate::data::Effect;
        for rank in 0..=4 {
            let mage = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(2, 65, rank), (2, 63, rank)]);
            let r = f32::from(rank.min(3));
            for (key, base, expected) in [
                ("blizzard", 0.6, 0.6 - 0.13 * r),
                ("frostbolt", 0.5, 0.5 - 0.03 * r),
                ("cone_of_cold", 0.5, 0.5 - 0.03 * r),
            ] {
                let Effect::Slow(speed, time) = mage.control(key, Effect::Slow(base, 2.)) else {
                    panic!("slow lost")
                };
                assert!((speed - expected).abs() < 0.00001);
                assert!((time - (2. + r)).abs() < 0.00001);
            }
            assert_eq!(
                mage.control("frost_nova", Effect::Root(2.5)),
                Effect::Root(2.5)
            );
            assert_eq!(
                mage.control("blast_wave", Effect::Slow(0.6, 2.)),
                Effect::Slow(0.6, 2.)
            );
        }
        let outsider = Milestones::default().with_ranks(&[(2, 65, 3), (2, 63, 3)]);
        assert_eq!(
            outsider.control("blizzard", Effect::Slow(0.6, 2.)),
            Effect::Slow(0.6, 2.)
        );
    }

    #[test]
    fn shatter_requires_frozen_targets_and_mage_spells() {
        for rank in 0..=6 {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(2, 67, rank)]);
            for key in MAGE_ARCANE.iter().chain(MAGE_FIRE).chain(MAGE_FROST) {
                assert!(
                    (trained.shatter(key, true) - 0.1 * f32::from(rank.min(5))).abs() < 0.00001
                );
                assert_eq!(trained.shatter(key, false), 0.);
            }
            for key in ["volley", "Ignite", "ice_barrier"] {
                assert_eq!(trained.shatter(key, true), 0.);
            }
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(2, 67, 5)])
                .shatter("frostbolt", true),
            0.
        );
    }

    #[test]
    fn shaman_milestones_unlock_at_exact_thresholds_without_extra_areas() {
        let spells = [
            "lightning_bolt",
            "chain_lightning",
            "earth_shock",
            "flame_shock",
            "frost_shock",
            "searing_totem",
            "magma_totem",
            "lightning_shield",
        ];
        for tree in 0..3 {
            for threshold in [10, 20, 30] {
                let mut before = Milestones {
                    class: 5,
                    ..Default::default()
                };
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                assert!(spells.iter().any(|key| before.cast(key) != after.cast(key)));
                assert!(description(5, tree, threshold).is_some());
                for key in ["fireball", "holy_shock", "arcane_explosion", "volley"] {
                    assert_eq!(after.cast(key), CastEffect::default());
                }
            }
        }
        let trained = Milestones {
            class: 5,
            points: [30; 3],
            ..Default::default()
        };
        assert_eq!(trained.cast("chain_lightning").crit, 1.);
        assert_eq!(trained.cast("lightning_bolt").crit, 0.1);
        assert!((trained.cast("earth_shock").cooldown - 0.765).abs() < 0.00001);
        assert_eq!(trained.cast("earth_shock").extra_targets, 1);
        assert_eq!(trained.cast("earth_shock").damage, 1.25);
        assert_eq!(trained.cast("earth_shock").shield_fraction, 0.03);
        assert_eq!(trained.cast("earth_shock").shield_seconds, 3.);
        for key in ["searing_totem", "magma_totem", "lightning_shield"] {
            let effect = trained.cast(key);
            assert_eq!(effect.damage, 1.2);
            assert_eq!(effect.extra_targets, 0);
            assert_eq!(effect.radius, 1.);
            assert_eq!(
                effect.heal_fraction,
                if key == "lightning_shield" { 0. } else { 0.03 }
            );
        }
        assert_eq!(
            trained.cast("searing_totem").cooldown_advance,
            Some(("magma_totem", 2.))
        );
        assert_eq!(
            trained.cast("magma_totem").cooldown_advance,
            Some(("searing_totem", 2.))
        );
        for class in [0, 1, 2, 3, 4, 6, 7, 8] {
            let other = Milestones { class, ..trained };
            for key in spells {
                assert_eq!(other.cast(key), CastEffect::default());
            }
        }
    }

    #[test]
    fn mage_arcane_and_fire_milestones_unlock_at_thresholds_and_stay_scoped() {
        for tree in 0..2 {
            for points in [0, 9, 10, 19, 20, 29, 30, 51] {
                let mut mage = Milestones {
                    class: 6,
                    ..Default::default()
                };
                mage.points[tree] = points;
                let missiles = mage.cast("arcane_missiles");
                let explosion = mage.cast("arcane_explosion");
                let fireball = mage.cast("fireball");
                assert_eq!(
                    missiles.crit,
                    if tree == 0 && points >= 10 { 0.05 } else { 0. }
                );
                assert_eq!(explosion.crit, missiles.crit);
                assert_eq!(
                    missiles.damage,
                    if tree == 0 && points >= 30 { 1.2 } else { 1. }
                );
                assert_eq!(explosion.damage, missiles.damage);
                assert_eq!(
                    missiles.cooldown_advance,
                    if tree == 0 && points >= 20 {
                        Some(("arcane_explosion", 2.))
                    } else {
                        None
                    }
                );
                assert_eq!(explosion.cooldown_advance, None);
                assert_eq!(
                    fireball.damage,
                    if tree == 1 && points >= 10 { 1.25 } else { 1. }
                );
                for key in MAGE_FIRE {
                    let effect = mage.cast(key);
                    assert_eq!(
                        effect.crit,
                        if tree == 1 && points >= 20 { 0.10 } else { 0. }
                    );
                    assert_eq!(
                        effect.crit_damage,
                        if tree == 1 && points >= 30 { 0.20 } else { 0. }
                    );
                    assert_eq!(effect.extra_targets, 0);
                    assert_eq!(effect.radius, 1.);
                    if *key != "fireball" {
                        assert_eq!(effect.damage, 1.);
                    }
                }
                for key in MAGE_FROST.iter().copied().chain([
                    "Ignite",
                    "holy_fire",
                    "smite",
                    "shadow_bolt",
                ]) {
                    assert_eq!(mage.cast(key), CastEffect::default());
                }
            }
            for threshold in [10, 20, 30] {
                assert!(description(6, tree, threshold).is_some());
            }
        }
        for class in [0, 1, 2, 3, 4, 5, 7, 8] {
            let other = Milestones {
                class,
                points: [30, 30, 0],
                ..Default::default()
            };
            for key in MAGE_ARCANE.iter().chain(MAGE_FIRE) {
                assert_eq!(other.cast(key), CastEffect::default());
            }
        }
    }

    #[test]
    fn mage_frost_milestones_and_defense_ranks_compose_without_extra_areas() {
        for points in [0, 9, 10, 19, 20, 29, 30, 51] {
            let base = Milestones {
                class: 6,
                points: [0, 0, points],
                ..Default::default()
            };
            let trained = base.with_ranks(&[(2, 72, 255), (2, 71, 255)]);
            let bolt = trained.cast("frostbolt");
            assert_eq!(bolt.extra_targets, usize::from(points >= 10));
            assert_eq!(bolt.crit_damage, if points >= 20 { 0.2 } else { 0. });
            let nova = trained.cast("frost_nova");
            assert_eq!(nova.immune_seconds, 0.5);
            assert!(
                (nova.shield_fraction - if points >= 30 { 0.15 } else { 0.05 }).abs() < 0.00001
            );
            assert_eq!(nova.shield_seconds, if points >= 30 { 5. } else { 3. });
            for key in MAGE_FROST {
                let effect = trained.cast(key);
                if *key != "frostbolt" {
                    assert_eq!(effect.extra_targets, 0);
                }
                assert_eq!(effect.radius, 1.);
                if *key != "frost_nova" {
                    assert_eq!((effect.shield_fraction, effect.immune_seconds), (0., 0.));
                }
            }
            assert_eq!(trained.cast("fireball"), CastEffect::default());
            assert_eq!(trained.cast("arcane_explosion"), CastEffect::default());
            assert_eq!(trained.with_ranks(&[]), base);
        }
        for threshold in [10, 20, 30] {
            assert!(description(6, 2, threshold).is_some());
        }
        for class in [0, 1, 2, 3, 4, 5, 7, 8] {
            let other = Milestones {
                class,
                points: [0, 0, 51],
                ..Default::default()
            }
            .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
            assert_eq!(other.cast("frost_nova"), CastEffect::default());
        }
    }

    #[test]
    fn winters_chill_refreshes_caps_expires_and_composes_with_shatter() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(2, 67, 5)]);
        let mut chill = WintersChill::default();
        for stacks in 1..=8 {
            chill.apply();
            assert!((chill.critical_bonus() - 0.02 * stacks.min(5) as f32).abs() < 0.00001);
        }
        chill.advance(14.9);
        chill.apply();
        chill.advance(14.9);
        assert!((mage.target_crit("frostbolt", true, chill) - 0.6).abs() < 0.00001);
        assert!((mage.target_crit("blizzard", false, chill) - 0.1).abs() < 0.00001);
        assert_eq!(mage.target_crit("fireball", true, chill), 0.5);
        assert_eq!(mage.target_crit("arcane_missiles", false, chill), 0.);
        assert_eq!(
            Milestones::default().target_crit("frostbolt", true, chill),
            0.
        );
        chill.advance(0.2);
        assert_eq!(chill.critical_bonus(), 0.);
        assert_eq!(mage.target_crit("frostbolt", true, chill), 0.5);
        chill.apply();
        assert_eq!(chill.critical_bonus(), 0.02);
        for dt in [-1., f32::NAN, f32::INFINITY] {
            chill.advance(dt);
        }
        assert_eq!(chill.critical_bonus(), 0.02);
        chill.advance(15.);
        assert_eq!(chill, WintersChill::default());
    }

    #[test]
    fn shadow_weaving_scopes_caps_refreshes_and_expires() {
        let priest = Milestones {
            class: 4,
            ..Default::default()
        };
        for rank in [0, 1, 3, 5, 255] {
            let trained = priest.with_ranks(&[(2, 461, rank)]);
            for key in PRIEST_SHADOW {
                assert!(
                    (trained.shadow_weaving_chance(key, 10.) - 0.20 * f32::from(rank.min(5))).abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.shadow_weaving_chance(key, damage), 0.);
                }
            }
            assert_eq!(trained.shadow_weaving_chance("smite", 10.), 0.);
            assert_eq!(trained.with_ranks(&[]), priest);
        }
        let mut target = ShadowWeaving::default();
        for n in 1..=8 {
            target.apply();
            assert!(
                (priest.shadow_damage_multiplier("mind_blast", target)
                    - (1. + 0.03 * n.min(5) as f32))
                    .abs()
                    < 0.00001
            );
        }
        target.advance(14.9);
        target.apply();
        target.advance(14.9);
        assert_eq!(
            priest.shadow_damage_multiplier("shadow_word_pain", target),
            1.15
        );
        assert_eq!(priest.shadow_damage_multiplier("holy_fire", target), 1.);
        for class in (0..9).filter(|class| *class != 4) {
            let other = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(2, 461, 5)]);
            assert_eq!(other.shadow_damage_multiplier("mind_blast", target), 1.);
            assert_eq!(other.shadow_weaving_chance("mind_blast", 10.), 0.);
        }
        target.advance(0.2);
        assert_eq!(target, ShadowWeaving::default());
        assert_eq!(priest.shadow_damage_multiplier("mind_blast", target), 1.);
    }

    #[test]
    fn vampiric_embrace_requires_base_talent_and_scoped_damage() {
        let base = Milestones {
            class: 4,
            ..Default::default()
        };
        for rank in [0, 1, 2, 255] {
            let trained = base.with_ranks(&[(2, 484, 1), (2, 1638, rank)]);
            for key in PRIEST_SHADOW {
                assert!(
                    (trained.vampiric_healing(key, 100.) - (20. + 5. * f32::from(rank.min(2))))
                        .abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.vampiric_healing(key, damage), 0.);
                }
            }
            for key in ["smite", "holy_fire", "drain_life", "fireball"] {
                assert_eq!(trained.vampiric_healing(key, 100.), 0.);
            }
            assert_eq!(
                base.with_ranks(&[(2, 1638, rank)])
                    .vampiric_healing("mind_blast", 100.),
                0.
            );
            assert_eq!(trained.with_ranks(&[]), base);
        }
        for class in (0..9).filter(|class| *class != 4) {
            assert_eq!(
                Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(2, 484, 1), (2, 1638, 2)])
                .vampiric_healing("mind_blast", 100.),
                0.
            );
        }
    }

    #[test]
    fn blackout_requires_priest_shadow_damage_and_bounded_training() {
        let priest = Milestones {
            class: 4,
            ..Default::default()
        };
        for rank in [0, 1, 3, 5, 255] {
            let trained = priest.with_ranks(&[(2, 464, rank)]);
            for key in PRIEST_SHADOW {
                assert!(
                    (trained.blackout_chance(key, 10.) - 0.02 * f32::from(rank.min(5))).abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.blackout_chance(key, damage), 0.);
                }
            }
            for key in [
                "smite",
                "holy_fire",
                "fireball",
                "Ignite",
                "power_word_shield",
            ] {
                assert_eq!(trained.blackout_chance(key, 10.), 0.);
            }
            assert_eq!(trained.with_ranks(&[]), priest);
        }
        for class in (0..9).filter(|class| *class != 4) {
            assert_eq!(
                Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(2, 464, 5)])
                .blackout_chance("mind_blast", 10.),
                0.
            );
        }
    }

    #[test]
    fn impact_requires_positive_mage_fire_damage_and_caps_ranks() {
        for rank in 0..=6 {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(1, 30, rank)]);
            for key in MAGE_FIRE {
                assert!(
                    (trained.impact_chance(key, 10.) - 0.02 * f32::from(rank.min(5))).abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.impact_chance(key, damage), 0.);
                }
            }
            for key in ["frostbolt", "Ignite", "impact", "volley"] {
                assert_eq!(trained.impact_chance(key, 10.), 0.);
            }
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(1, 30, 5)])
                .impact_chance("fireball", 10.),
            0.
        );
    }

    #[test]
    fn winters_chill_requires_positive_mage_frost_damage_and_caps_ranks() {
        for rank in 0..=6 {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(2, 68, rank)]);
            for key in MAGE_FROST {
                assert!(
                    (trained.winters_chill_chance(key, 10.) - 0.20 * f32::from(rank.min(5))).abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.winters_chill_chance(key, damage), 0.);
                }
            }
            for key in ["fireball", "Ignite", "winters_chill", "volley"] {
                assert_eq!(trained.winters_chill_chance(key, 10.), 0.);
            }
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(2, 68, 5)])
                .winters_chill_chance("frostbolt", 10.),
            0.
        );
    }

    #[test]
    fn frostbite_requires_positive_mage_frost_damage_and_caps_ranks() {
        for rank in 0..=4 {
            let trained = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(2, 38, rank)]);
            for key in MAGE_FROST {
                assert!(
                    (trained.frostbite_chance(key, 10.) - 0.05 * f32::from(rank.min(3))).abs()
                        < 0.00001
                );
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(trained.frostbite_chance(key, damage), 0.);
                }
            }
            for key in ["fireball", "Ignite", "frostbite", "volley"] {
                assert_eq!(trained.frostbite_chance(key, 10.), 0.);
            }
        }
        assert_eq!(
            Milestones::default()
                .with_ranks(&[(2, 38, 3)])
                .frostbite_chance("frostbolt", 10.),
            0.
        );
    }

    #[test]
    fn ignite_requires_a_trained_mage_fire_critical_and_cannot_recurse() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        for rank in 0..=5 {
            let trained = mage.with_ranks(&[(1, 34, rank)]);
            for key in MAGE_FIRE {
                assert!((trained.ignite(key, 100., true) - 8. * f32::from(rank)).abs() < 0.0001);
                assert_eq!(trained.ignite(key, 100., false), 0.);
            }
            for key in [
                "Ignite",
                "ignite",
                "frostbolt",
                "arcane_missiles",
                "consecration",
            ] {
                assert_eq!(trained.ignite(key, 100., true), 0.);
            }
            for damage in [0., -1., f32::INFINITY, f32::NAN] {
                assert_eq!(trained.ignite("fireball", damage, true), 0.);
            }
        }
        assert!(
            (mage
                .with_ranks(&[(1, 34, 255)])
                .ignite("fireball", 100., true)
                - 40.)
                .abs()
                < 0.0001
        );
        for class in [0, 1, 2, 3, 4, 5, 7, 8] {
            assert_eq!(
                Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 34, 5)])
                .ignite("fireball", 100., true),
                0.
            );
        }
    }

    #[test]
    fn mage_instability_and_precision_compose_across_schools_and_reset() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        for rank in 0..=4 {
            let trained = base.with_ranks(&[(0, 421, rank), (2, 1649, rank)]);
            let value = f32::from(rank.min(3)) * 0.01;
            for key in crate::data::CLASSES[6].pool {
                let effect = trained.cast(key);
                if *key == "mana_shield" {
                    assert_eq!(effect, CastEffect::default());
                    continue;
                }
                let multiplier = (1.0 + value)
                    * if MAGE_ARCANE.contains(key) {
                        1.0
                    } else {
                        1.0 + value
                    };
                assert!((effect.damage - multiplier).abs() < 0.00001, "{key}");
                assert!((effect.crit - value).abs() < 0.00001, "{key}");
                assert_eq!(effect.extra_targets, 0);
                assert_eq!(effect.radius, 1.0);
            }
            for key in ["holy_shock", "shadow_bolt", "consecration"] {
                assert_eq!(trained.cast(key), CastEffect::default());
            }
            assert_eq!(trained.with_ranks(&[]), base);
            for class in [0, 1, 2, 3, 4, 5, 7, 8] {
                let foreign = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, 421, rank), (2, 1649, rank)]);
                assert_eq!(foreign.cast("fireball"), CastEffect::default());
            }
        }
        let node = talent_definition(6, 421).unwrap();
        assert_eq!(
            node.rank_preview(2),
            "This talent: 2 » 3% damage; 2 » 3% critical chance next rank"
        );
        assert_eq!(
            node.rank_preview(255),
            "This talent: 3% damage; 3% critical chance (maximum rank)"
        );
    }

    #[test]
    fn mage_school_training_composes_caps_and_stays_class_specific() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        let ranks = [
            (0, 76, 5),
            (0, 80, 5),
            (0, 81, 3),
            (0, 1650, 5),
            (1, 26, 5),
            (1, 27, 3),
            (1, 31, 3),
            (1, 29, 1),
            (1, 33, 3),
            (1, 32, 1),
            (1, 35, 5),
            (2, 37, 5),
            (2, 73, 5),
            (2, 62, 2),
            (2, 61, 3),
            (2, 64, 3),
        ];
        let trained = base.with_ranks(&ranks);
        let near = |a: f32, b: f32| assert!((a - b).abs() < 0.0001, "{a} != {b}");
        near(trained.cast("arcane_missiles").damage, 1.1 * 1.25);
        near(trained.cast("arcane_explosion").crit, 0.06);
        near(trained.cast("fireball").cooldown, 0.8);
        near(trained.cast("fire_blast").cooldown, 0.85);
        near(trained.cast("flamestrike").crit, 0.21);
        near(trained.cast("pyroblast").damage, 1.2 * 1.1);
        near(trained.cast("blast_wave").damage, 1.2 * 1.1);
        near(trained.cast("frostbolt").cooldown, 0.8);
        near(trained.cast("frost_nova").cooldown, 0.8);
        near(trained.cast("cone_of_cold").damage, 1.06 * 1.3);
        near(trained.cast("blizzard").crit_damage, 0.5);
        near(stat_adjustment(6, &ranks).reduction, 0.05);
        for key in crate::data::CLASSES[6].pool {
            let effect = trained.cast(key);
            assert_eq!(effect.extra_targets, 0);
            assert_eq!(effect.radius, 1.0);
            assert_eq!(base.cast(key), CastEffect::default());
            let capped = base.with_ranks(&ranks.map(|(tree, id, _)| (tree, id, 255)));
            assert_eq!(capped.cast(key), effect);
            for class in [0, 1, 2, 3, 4, 5, 7, 8] {
                assert_eq!(
                    Milestones {
                        class,
                        ..Default::default()
                    }
                    .with_ranks(&ranks)
                    .cast(key),
                    CastEffect::default()
                );
            }
        }
        assert_eq!(trained.cast("holy_shock"), CastEffect::default());
        assert_eq!(trained.cast("shadow_bolt"), CastEffect::default());
        assert_eq!(trained.with_ranks(&[]), base);
    }

    #[test]
    fn unsupported_classes_and_nodes_receive_no_invented_adjustment() {
        assert_eq!(stat_adjustment(0, &[(0, 1422, 5)]).armor, 0.0);
        assert_eq!(stat_adjustment(1, &[(0, 999999, 5)]).might, 0.0);
        assert_eq!(
            stat_adjustment(1, &[(0, 1422, 255), (0, 1422, 5)]).armor,
            40.0
        );
    }
}

fn bounded_rank(node: &TalentDefinition, ranks: &[(u8, u32, u8)]) -> u8 {
    ranks
        .iter()
        .filter(|(_, id, _)| *id == node.id)
        .map(|(_, _, rank)| *rank)
        .max()
        .unwrap_or(0)
        .min(node.max_rank)
}

#[cfg(test)]
mod spiritual_healing_tests {
    use super::*;

    #[test]
    fn healing_training_scales_owned_effects_without_granting_or_buffing_damage() {
        for class in 0..9 {
            for points in [0, 29, 30, 40] {
                let base = Milestones {
                    class,
                    points: [0, points, 0],
                    ..Default::default()
                };
                for rank in [0, 1, 5, 255] {
                    let trained = base.with_ranks(&[(1, 404, rank), (1, 406, 3)]);
                    let multiplier = if class == 4 {
                        1. + 0.02 * f32::from(rank.min(5))
                    } else {
                        1.
                    };
                    let nova = trained.cast("holy_nova");
                    let mut expected = base.cast("holy_nova");
                    expected.heal_fraction *= multiplier;
                    assert_eq!(nova, expected);
                    let renew = if class == 4 { 1.15 * multiplier } else { 1. };
                    assert!((trained.passive_multiplier("renew") - renew).abs() < 0.00001);
                    for key in [
                        "regeneration",
                        "rejuvenation",
                        "holy_shock",
                        "power_word_shield",
                        "shadow_word_pain",
                        "smite",
                    ] {
                        assert_eq!(trained.cast(key), base.cast(key));
                        assert_eq!(trained.passive_multiplier(key), 1.);
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
        assert_eq!(
            talent_definition(4, 404).unwrap().rank_preview(1),
            "This talent: 2 to 4% Renew and Holy Nova healing next rank"
        );
    }
}

#[cfg(test)]
mod health_talent_tests {
    use super::*;

    #[test]
    fn sanctuary_is_capped_class_scoped_and_resettable() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            for rank in [0, 1, 255] {
                let trained = base.with_ranks(&[(1, 1431, rank), (1, 1431, rank)]);
                for damage in [0.0_f32, 1., 2., 10.] {
                    let prevented = if class == 1 && rank > 0 { 2. } else { 0. };
                    assert_eq!(
                        trained.after_flat_absorption(damage),
                        (damage - prevented).max(0.)
                    );
                    assert_eq!(
                        trained.with_ranks(&[]).after_flat_absorption(damage),
                        damage
                    );
                }
                assert_eq!(trained.health_multiplier(), 1.);
                assert_eq!(trained.cast("holy_shock"), base.cast("holy_shock"));
            }
        }
    }

    #[test]
    fn survivalist_is_capped_class_scoped_and_resettable() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            for rank in [0, 1, 5, 255] {
                let trained = base.with_ranks(&[(2, 1622, rank), (2, 1622, rank)]);
                let expected = if class == 2 {
                    1. + 0.02 * f32::from(rank.min(5))
                } else {
                    1.
                };
                assert_eq!(trained.health_multiplier(), expected);
                assert_eq!(trained.with_ranks(&[]).health_multiplier(), 1.);
                assert_eq!(trained.cast("arcane_shot"), base.cast("arcane_shot"));
                assert_eq!(trained.passive_multiplier("renew"), 1.);
            }
        }
        assert_eq!(
            talent_definition(2, 1622).unwrap().rank_preview(4),
            "This talent: 8 to 10% maximum health next rank"
        );
    }
}

#[cfg(test)]
mod holy_nova_training_tests {
    use super::*;

    #[test]
    fn trained_nova_adds_one_heal_and_scales_with_spiritual_healing() {
        for class in 0..9 {
            for points in [0, 29, 30, 40] {
                let base = Milestones {
                    class,
                    points: [0, points, 0],
                    ..Default::default()
                };
                for rank in [0, 1, 255] {
                    let trained = base.with_ranks(&[(1, 442, rank), (1, 404, 5)]);
                    let mut expected = base.cast("holy_nova");
                    if class == 4 {
                        expected.heal_fraction =
                            (expected.heal_fraction + if rank > 0 { 0.05 } else { 0. }) * 1.1;
                    }
                    assert_eq!(trained.cast("holy_nova"), expected);
                    for key in ["holy_fire", "smite", "power_word_shield", "rejuvenation"] {
                        assert_eq!(trained.cast(key), base.cast(key));
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }
}

#[cfg(test)]
mod efficiency_tests {
    use super::*;
    #[test]
    fn cost_conversions_are_scoped_capped_multiplicative_and_resettable() {
        for (owner, id, maximum, per_rank, spells) in [
            (2, 1342, 5, 0.02, HUNTER_SHOTS),
            (
                4,
                341,
                5,
                0.02,
                &[
                    "shadow_word_pain",
                    "holy_nova",
                    "psychic_scream",
                    "power_word_shield",
                ][..],
            ),
            (6, 66, 3, 0.05, MAGE_FROST),
        ] {
            for class in 0..9 {
                let base = Milestones {
                    class,
                    points: [30; 3],
                    ..Default::default()
                };
                for rank in [0, 1, 3, 5, 255] {
                    let trained = base.with_ranks(&[(0, id, rank)]);
                    for def in crate::data::ABILITIES {
                        let mut expected = base.cast(def.key);
                        if class == owner && spells.contains(&def.key) {
                            expected.cooldown *= 1.0 - per_rank * f32::from(rank.min(maximum));
                        }
                        assert_eq!(
                            trained.cast(def.key),
                            expected,
                            "class {class} talent {id} {}",
                            def.key
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }
}

#[cfg(test)]
mod additional_inline_tests {
    use super::*;

    #[test]
    fn single_rank_unlocks_inline_the_bonus_and_preserve_other_numbers() {
        for (class, id, value) in [
            (8, 761, "15% shorter cooldown"),
            (8, 831, "15% shorter cooldown"),
            (8, 844, "5% maximum health"),
            (4, 442, "5% maximum health"),
            (3, 382, "10% shorter cooldown"),
        ] {
            let node = talent_definition(class, id).unwrap();
            let (untrained, spans) = node.inline_description_with_values(0).unwrap();
            assert_eq!(spans.len(), 1);
            assert_eq!(&untrained[spans[0].clone()], format!("0 » {value}"));
            assert!(!untrained.contains("This talent"));
            for rank in [1, 255] {
                let (trained, spans) = node.inline_description_with_values(rank).unwrap();
                assert_eq!(trained, node.description);
                assert_eq!(&trained[spans[0].clone()], value);
            }
        }
        let node = talent_definition(8, 844).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("for 10% total"));
        let ambiguous = TalentDefinition {
            description: "Deal 5% damage and gain 5% armor",
            stat: TalentStat::Damage,
            per_rank: 0.05,
            ..*node
        };
        assert!(ambiguous.inline_description(0).is_none());
        let multiple_ranks = TalentDefinition {
            max_rank: 3,
            ..*node
        };
        assert!(multiple_ranks.inline_description(0).is_none());
    }

    #[test]
    fn healing_and_paired_damage_crit_values_are_inline_at_every_rank() {
        for (class, id, count) in [
            (6, 421, 2),
            (4, 1202, 2),
            (4, 404, 1),
            (4, 406, 1),
            (5, 588, 1),
            (5, 592, 1),
        ] {
            let node = talent_definition(class, id).unwrap();
            for rank in 0..=node.max_rank {
                let (text, spans) = node.inline_description_with_values(rank).unwrap();
                assert_eq!(spans.len(), count);
                assert!(!text.contains("per rank"));
                assert!(!text.contains("This talent"));
                assert_eq!(
                    text.matches(" » ").count(),
                    if rank == node.max_rank { 0 } else { count }
                );
                for span in spans {
                    let value = &text[span];
                    assert_eq!(value.matches('%').count(), 1);
                }
            }
            assert_eq!(
                node.inline_description(255),
                node.inline_description(node.max_rank)
            );
        }
        assert_eq!(talent_definition(6, 421).unwrap().inline_description(1).unwrap(),
            "Mage spells: +1 » 2% damage and +1 » 2% critical chance; excludes Mana Shield, other-class and neutral jewelry spells");
        assert!(talent_definition(5, 588).unwrap().inline_description(4).unwrap().starts_with("Searing Totem and Magma Totem deployment: +20 » 25% healing, adding to Purification."));
        let ambiguous = TalentDefinition {
            description: "+1% damage and +1% critical chance per rank; adds another 1%",
            ..*talent_definition(6, 421).unwrap()
        };
        assert!(ambiguous.inline_description(0).is_none());
    }
}

#[cfg(test)]
mod duration_pet_inline_tests {
    use super::*;

    #[test]
    fn pet_range_and_duration_previews_keep_fixed_numbers_and_single_units() {
        for (class, id) in [
            (2, 1625),
            (2, 1388),
            (2, 1389),
            (2, 1396),
            (2, 1393),
            (2, 1397),
            (2, 1352),
            (6, 28),
        ] {
            let node = talent_definition(class, id).unwrap();
            for rank in 0..=node.max_rank {
                let (text, ranges) = node.inline_description_with_values(rank).unwrap();
                assert_eq!(ranges.len(), 1);
                assert_eq!(
                    text.matches(" » ").count(),
                    usize::from(rank < node.max_rank)
                );
                assert!(!text.contains("per rank"));
                assert!(!text[ranges[0].clone()].contains("% »"));
            }
        }
        let find = |stat| definitions(0).iter().find(|d| d.stat == stat).unwrap();
        let shield = find(TalentStat::SpellShieldDuration("heroic_strike"));
        assert_eq!(shield.inline_description(1).unwrap(), "Heroic Strike's absorb lasts 0.5 » 1 second longer; requires an absorb talent or milestone");
        assert!(shield
            .inline_description(3)
            .unwrap()
            .contains("1.5 seconds longer"));
        let root = definitions(2)
            .iter()
            .find(|d| d.stat == TalentStat::SecondaryRootDuration("explosive_trap"))
            .unwrap();
        assert_eq!(root.inline_description(1).unwrap(), "Explosive Trap hits add 0.4 » 0.8 sec of root (bosses: 30% duration; Survivors conversion)");
        assert!(talent_definition(2, 1397)
            .unwrap()
            .inline_description(3)
            .unwrap()
            .contains("60 » 80% chance to grant 30% bite haste for 8 sec"));
        assert!(talent_definition(2, 1352)
            .unwrap()
            .inline_description(1)
            .unwrap()
            .contains("+2 » 4 yd targeting range"));
    }
}

#[cfg(test)]
mod compound_range_inline_tests {
    use super::*;

    #[test]
    fn compound_ranges_and_slow_dimensions_have_independent_inline_values() {
        for (class, id) in [(6, 741), (4, 1635), (6, 65)] {
            let node = talent_definition(class, id).unwrap();
            for rank in 0..=node.max_rank {
                let (text, spans) = node.inline_description_with_values(rank).unwrap();
                assert_eq!(spans.len(), 2);
                assert!(spans[0].end < spans[1].start);
                assert_eq!(
                    text.matches(" » ").count(),
                    if rank == node.max_rank { 0 } else { 2 }
                );
                assert!(!text.contains("per rank"));
                assert!(spans.iter().all(|r| !text[r.clone()].is_empty()));
            }
        }
        let frost = talent_definition(6, 65).unwrap();
        let (text, spans) = frost.inline_description_with_values(1).unwrap();
        assert_eq!(text, "Frost slows last 1 » 2 sec longer and reduce movement by 3 » 6 additional percentage points; does not extend freezes");
        assert_eq!(&text[spans[0].clone()], "1 » 2 sec");
        assert_eq!(
            &text[spans[1].clone()],
            "3 » 6 additional percentage points"
        );
        assert_eq!(frost.inline_description(255).unwrap(), "Frost slows last 3 sec longer and reduce movement by 9 additional percentage points; does not extend freezes");
        assert_eq!(talent_definition(4, 1635).unwrap().inline_description(1).unwrap(), "Smite and Holy Fire: +10 » 20% casting range; Holy Nova: +10 » 20% radius, still one player-centered area");
    }
}

#[cfg(test)]
mod inline_stat_label_tests {
    use super::*;
    #[test]
    fn stat_labels_share_green_ranges_without_coloring_fixed_effects() {
        for (class, id, expected) in [
            (8, 808, "+0 » 4% maximum health"),
            (8, 794, "+0 » 8 hero armor"),
            (8, 798, "+0 » 2 percentage points hero critical chance"),
            (8, 807, "+0 » 5% hero movement speed"),
            (2, 1352, "+0 » 2 yd targeting range"),
            (2, 1382, "+0 » 3% haste"),
            (0, 127, "+0 » 10% bleed damage"),
            (0, 147, "+0 » 10% reflected damage"),
            (0, 161, "+0 » 10% weakening duration"),
            (1, 1402, "+0 » 3% Paladin Holy damage"),
            (2, 1351, "+0 » 20% damage-reduction duration"),
            (5, 612, "+0 » 5% shield absorption"),
            (5, 586, "0 » 4% shorter deployment cooldown"),
            (5, 1641, "+0 » 10% initial targeting range"),
        ] {
            let node = talent_definition(class, id).unwrap();
            let (text, ranges) = node
                .inline_description_with_values(0)
                .unwrap_or_else(|| panic!("missing inline preview: class {class}, node {id}"));
            assert_eq!(&text[ranges[0].clone()], expected);
            assert!(!text.contains("per rank"));
            let (max, ranges) = node.inline_description_with_values(255).unwrap();
            assert!(!max.contains(" » "));
            assert!(!max[ranges[0].clone()].contains("Survivors"));
        }
        let node = talent_definition(2, 1397).unwrap();
        let (text, ranges) = node.inline_description_with_values(1).unwrap();
        assert_eq!(&text[ranges[0].clone()], "20 » 40% chance");
        assert!(text.contains("30% bite haste for 8 sec"));
        assert_eq!(
            inline_stat_label(" damage reduction, unchanged armor"),
            " damage reduction"
        );
        assert_eq!(inline_stat_label(" damageable objects"), "");
        assert_eq!(inline_stat_label(" healing_thing"), "");
        let hawk = talent_definition(2, 1382).unwrap();
        for (rank, expected) in [(1, "+3 » 6% haste"), (5, "+15% haste"), (255, "+15% haste")] {
            let (text, ranges) = hawk.inline_description_with_values(rank).unwrap();
            assert_eq!(ranges.len(), 1);
            assert_eq!(&text[ranges[0].clone()], expected);
            assert!(text.contains("12 sec"));
            assert!(text.contains("10% chance"));
        }
        assert_eq!(inline_stat_label(" hastened attacks"), "");
        for tail in [" bleed damageable", " shield absorptions", " chance_thing"] {
            assert_eq!(inline_stat_label(tail), "");
        }
        let scorpid = talent_definition(2, 1351).unwrap();
        for rank in [0, 1, 3, 255] {
            let (text, ranges) = scorpid.inline_description_with_values(rank).unwrap();
            assert_eq!(ranges.len(), 1);
            assert!(text[ranges[0].clone()].ends_with("% damage-reduction duration"));
            assert!(text[ranges[0].end..].contains("25% enemy damage reduction"));
        }
        let vengeance = talent_definition(1, 1402).unwrap();
        for rank in [0, 1, 5, 255] {
            let (text, ranges) = vengeance.inline_description_with_values(rank).unwrap();
            assert_eq!(ranges.len(), 1);
            assert!(text[ranges[0].clone()].ends_with("% Paladin Holy damage"));
            assert!(text[ranges[0].end..].contains("8 sec"));
        }
        let force = talent_definition(4, 1202).unwrap();
        let (text, ranges) = force.inline_description_with_values(1).unwrap();
        assert_eq!(
            ranges.iter().map(|r| &text[r.clone()]).collect::<Vec<_>>(),
            ["+1 » 2% damage", "+1 » 2% direct critical chance"]
        );
    }
}

#[cfg(test)]
mod savage_strikes_tests {
    use super::*;
    #[test]
    fn savage_strikes_is_capped_melee_only_and_does_not_inherit_shot_training() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 1621, rank)]);
                for key in crate::data::CLASSES[2]
                    .ability_keys()
                    .into_iter()
                    .chain(["heroic_strike", "sinister_strike"])
                {
                    let mut expected = base.cast(key);
                    if class == 2 && matches!(key, "raptor_strike" | "mongoose_bite") {
                        expected.crit += 0.1 * f32::from(rank.min(2));
                    }
                    assert_eq!(
                        trained.cast(key),
                        expected,
                        "class={class} rank={rank} key={key}"
                    );
                    assert_eq!(trained.with_ranks(&[]).cast(key), base.cast(key));
                }
            }
        }
        let hunter = Milestones {
            class: 2,
            points: [0, 30, 0],
            ..Default::default()
        }
        .with_ranks(&[
            (1, 1361, 1),
            (1, 1342, 5),
            (1, 1352, 3),
            (1, 1344, 5),
            (1, 1349, 5),
        ]);
        for key in ["raptor_strike", "mongoose_bite"] {
            assert_eq!(
                hunter.cast(key),
                Milestones {
                    class: 2,
                    ..Default::default()
                }
                .cast(key)
            );
        }
        let node = talent_definition(2, 1621).unwrap();
        let (text, ranges) = node.inline_description_with_values(1).unwrap();
        assert_eq!(
            &text[ranges[0].clone()],
            "+10 » 20 percentage points critical chance"
        );
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod wing_clip_tests {
    use super::*;
    #[test]
    fn root_chance_is_capped_class_and_hit_scoped_without_other_cast_changes() {
        for class in 0..9 {
            for rank in [0, 1, 4, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(2, 1305, rank)]);
                for key in crate::data::CLASSES[2].ability_keys() {
                    for damage in [0., -1., f32::NAN, f32::INFINITY, 5.] {
                        let expected = if class == 2 && key == "wing_clip" && damage == 5. {
                            0.04 * f32::from(rank.min(5))
                        } else {
                            0.
                        };
                        assert_eq!(trained.wing_clip_root_chance(key, damage), expected);
                    }
                    assert_eq!(trained.cast(key), base.cast(key));
                    assert_eq!(trained.with_ranks(&[]).wing_clip_root_chance(key, 5.), 0.);
                }
            }
        }
        let node = talent_definition(2, 1305).unwrap();
        let (text, ranges) = node.inline_description_with_values(1).unwrap();
        assert_eq!(&text[ranges[0].clone()], "+4 » 8% chance");
        assert!(text.contains("5 sec (bosses: 1.5 sec)"));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod weapon_training_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn weapon_training_requires_matching_worn_melee_and_only_changes_weapon_attacks() {
        for (id, tree, step) in [
            (136, 0, 0.01),
            (132, 0, 0.01),
            (134, 0, 0.01),
            (702, 2, 0.02),
        ] {
            for class in 0..9 {
                for rank in [0, 1, 5, 255] {
                    for weapon in [
                        None,
                        Some(MeleeWeapon {
                            kind: MeleeKind::Sword,
                            two_handed: false,
                        }),
                        Some(MeleeWeapon {
                            kind: MeleeKind::Axe,
                            two_handed: true,
                        }),
                        Some(MeleeWeapon {
                            kind: MeleeKind::Polearm,
                            two_handed: true,
                        }),
                    ] {
                        let base = Milestones {
                            class,
                            weapon,
                            ..Default::default()
                        };
                        let trained = base.with_ranks(&[(tree, id, rank)]);
                        for key in [
                            "cleave",
                            "heroic_strike",
                            "whirlwind",
                            "sweeping_strikes",
                            "execute",
                            "mortal_strike",
                            "retaliation",
                            "thunder_clap",
                            "demoralizing_shout",
                            "deep_wounds",
                            "fireball",
                        ] {
                            let mut expected = base.cast(key);
                            if class == 0
                                && weapon.is_some_and(|w| match id {
                                    136 => w.two_handed,
                                    132 => w.kind == MeleeKind::Axe,
                                    134 => w.kind == MeleeKind::Polearm,
                                    702 => !w.two_handed,
                                    _ => unreachable!(),
                                })
                                && !matches!(
                                    key,
                                    "thunder_clap"
                                        | "demoralizing_shout"
                                        | "deep_wounds"
                                        | "fireball"
                                )
                            {
                                if matches!(id, 132 | 134) {
                                    expected.crit += step * f32::from(rank.min(5));
                                } else {
                                    expected.damage *= 1.0 + step * f32::from(rank.min(5));
                                }
                            }
                            assert_eq!(trained.cast(key), expected, "{class} {rank} {key}");
                        }
                    }
                }
            }
        }
        let polearm = talent_definition(0, 134).unwrap();
        let (description, ranges) = polearm.inline_description_with_values(4).unwrap();
        assert!(ranges
            .iter()
            .any(|range| description[range.clone()].contains("4 » 5%")));
        assert!(!polearm.inline_description(255).unwrap().contains(" » "));
        let axe = talent_definition(0, 132).unwrap();
        assert!(axe.inline_description(4).unwrap().contains("4 » 5%"));
        let one = talent_definition(0, 702).unwrap();
        assert!(one.inline_description(4).unwrap().contains("8 » 10%"));
        let node = talent_definition(0, 136).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 1%"));
        assert!(node.inline_description(4).unwrap().contains("4 » 5%"));
    }
}

#[cfg(test)]
mod sword_training_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn sword_proc_requires_direct_damage_and_matching_worn_weapon() {
        for class in 0..9 {
            for rank in [0, 1, 5, 255] {
                for kind in [None, Some(MeleeKind::Sword), Some(MeleeKind::Axe)] {
                    for two_handed in [false, true] {
                        let base = Milestones {
                            class,
                            weapon: kind.map(|kind| MeleeWeapon { kind, two_handed }),
                            ..Default::default()
                        };
                        let trained = base.with_ranks(&[(0, 123, rank)]);
                        for key in [
                            "cleave",
                            "heroic_strike",
                            "whirlwind",
                            "sweeping_strikes",
                            "execute",
                            "mortal_strike",
                            "retaliation",
                            "thunder_clap",
                            "deep_wounds",
                            "sword_specialization",
                            "Thorns",
                            "fireball",
                        ] {
                            assert_eq!(trained.cast(key), base.cast(key));
                            for damage in [0.0, -1.0, f32::NAN, f32::INFINITY, 50.0] {
                                let eligible = class == 0
                                    && kind == Some(MeleeKind::Sword)
                                    && damage == 50.0
                                    && matches!(
                                        key,
                                        "cleave"
                                            | "heroic_strike"
                                            | "whirlwind"
                                            | "sweeping_strikes"
                                            | "execute"
                                            | "mortal_strike"
                                            | "retaliation"
                                    );
                                let expected = if eligible {
                                    0.01 * f32::from(rank.min(5))
                                } else {
                                    0.0
                                };
                                assert_eq!(
                                    trained.sword_extra_attack_chance(key, damage),
                                    expected
                                );
                            }
                        }
                    }
                }
            }
        }
        let node = talent_definition(0, 123).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 1%"));
        assert!(node.inline_description(4).unwrap().contains("4 » 5%"));
    }
}

#[cfg(test)]
mod mace_training_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn mace_proc_uses_original_non_linear_ranks_and_matching_weapon_attacks() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 4, 5, 255] {
                for kind in [
                    None,
                    Some(MeleeKind::Mace),
                    Some(MeleeKind::Sword),
                    Some(MeleeKind::Polearm),
                ] {
                    let base = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&[(0, 125, rank)]);
                    for key in [
                        "cleave",
                        "heroic_strike",
                        "whirlwind",
                        "sweeping_strikes",
                        "execute",
                        "mortal_strike",
                        "retaliation",
                        "thunder_clap",
                        "demoralizing_shout",
                        "deep_wounds",
                        "sword_specialization",
                        "fireball",
                    ] {
                        for damage in [0.0, -1.0, f32::NAN, f32::INFINITY, 20.0] {
                            let expected = if class == 0
                                && kind == Some(MeleeKind::Mace)
                                && damage == 20.0
                                && matches!(
                                    key,
                                    "cleave"
                                        | "heroic_strike"
                                        | "whirlwind"
                                        | "sweeping_strikes"
                                        | "execute"
                                        | "mortal_strike"
                                        | "retaliation"
                                ) {
                                [0.0, 0.01, 0.02, 0.03, 0.04, 0.06][usize::from(rank.min(5))]
                            } else {
                                0.0
                            };
                            assert_eq!(trained.mace_stun_chance(key, damage), expected);
                            assert_eq!(trained.cast(key), base.cast(key));
                        }
                    }
                }
            }
        }
        let node = talent_definition(0, 125).unwrap();
        for (rank, expected) in [
            (0, "0 » 1% chance"),
            (4, "4 » 6% chance"),
            (5, "6% chance"),
            (255, "6% chance"),
        ] {
            let (text, ranges) = node.inline_description_with_values(rank).unwrap();
            assert_eq!(&text[ranges[0].clone()], expected);
            assert!(text.contains("3 sec (bosses: 0.9 sec)"));
            assert!(node.rank_preview(rank).contains(if rank >= 5 {
                "maximum rank"
            } else {
                "next rank"
            }));
        }
    }
}

#[cfg(test)]
mod hamstring_tests {
    use super::*;
    #[test]
    fn root_chance_is_capped_class_and_hit_scoped_without_other_cast_changes() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(0, 129, rank)]);
                for key in crate::data::CLASSES[0].ability_keys() {
                    for damage in [0., -1., f32::NAN, f32::INFINITY, 5.] {
                        let expected = if class == 0 && key == "thunder_clap" && damage == 5. {
                            0.05 * f32::from(rank.min(3))
                        } else {
                            0.
                        };
                        assert_eq!(trained.hamstring_root_chance(key, damage), expected);
                    }
                    assert_eq!(trained.cast(key), base.cast(key));
                    assert_eq!(trained.with_ranks(&[]).hamstring_root_chance(key, 5.), 0.);
                }
            }
        }
        let node = talent_definition(0, 129).unwrap();
        let (text, ranges) = node.inline_description_with_values(1).unwrap();
        assert_eq!(&text[ranges[0].clone()], "+5 » 10% chance");
        assert!(text.contains("5 sec (bosses: 1.5 sec)"));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod battle_shout_tests {
    use super::*;
    #[test]
    fn shout_strength_is_class_scoped_capped_and_not_global_damage() {
        for class in 0..9 {
            for rank in [0, 1, 4, 5, 255] {
                let base = Milestones {
                    class,
                    ..Default::default()
                };
                let trained = base.with_ranks(&[(1, 154, rank)]);
                let expected = if class == 0 {
                    1.0 + 0.05 * f32::from(rank.min(5))
                } else {
                    1.0
                };
                assert_eq!(trained.battle_shout_multiplier(), expected);
                for key in crate::data::CLASSES[0].ability_keys() {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
                assert_eq!(trained.with_ranks(&[]).battle_shout_multiplier(), 1.0);
            }
        }
        let node = talent_definition(0, 154).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert_eq!(&text[ranges[0].clone()], "+20 » 25% buff strength");
        assert!(!node.inline_description(5).unwrap().contains(" » "));
    }
    #[test]
    fn shout_is_a_distinct_warrior_acquisition_without_damage_or_targets() {
        let def = crate::data::ability("battle_shout");
        assert_eq!(def.kind, crate::data::Kind::WeaponBuff);
        assert_eq!((def.damage, def.dot, def.count), (0.0, 0.0, 0));
        assert_eq!(def.weapon_bonus, 0.1);
        for (class, def) in crate::data::CLASSES.iter().enumerate() {
            assert_eq!(def.ability_keys().contains(&"battle_shout"), class == 0);
        }
    }
}

#[cfg(test)]
mod booming_voice_tests {
    use super::*;

    #[test]
    fn both_shouts_gain_duration_but_only_demoralizing_has_radius() {
        for rank in 0..=5 {
            let plain = Milestones::default();
            let trained = plain.with_ranks(&[(1, 158, rank)]);
            let factor = 1.0 + f32::from(rank) * 0.1;
            for key in [
                "battle_shout",
                "demoralizing_shout",
                "cleave",
                "thunder_clap",
            ] {
                let before = plain.cast(key);
                let after = trained.cast(key);
                let shout = matches!(key, "battle_shout" | "demoralizing_shout");
                assert!(
                    (after.duration - before.duration * if shout { factor } else { 1.0 }).abs()
                        < 0.00001
                );
                assert!(
                    (after.radius
                        - before.radius
                            * if key == "demoralizing_shout" {
                                factor
                            } else {
                                1.0
                            })
                    .abs()
                        < 0.00001
                );
                assert_eq!(after.damage, before.damage);
                assert_eq!(after.cooldown, before.cooldown);
                assert_eq!(after.extra_targets, before.extra_targets);
            }
            assert_eq!(trained.battle_shout_multiplier(), 1.0);
        }
    }
    #[test]
    fn shout_training_is_capped_class_scoped_and_composes_with_other_talents() {
        for class in 0..9 {
            let base = Milestones {
                class,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(1, 158, 255)]);
            for key in ["battle_shout", "demoralizing_shout"] {
                if class == 0 {
                    assert!(
                        (trained.cast(key).duration - base.cast(key).duration * 1.5).abs()
                            < 0.00001
                    );
                } else {
                    assert_eq!(trained.cast(key), base.cast(key));
                }
                assert_eq!(trained.with_ranks(&[]).cast(key), base.cast(key));
            }
        }
        let trained = Milestones::default().with_ranks(&[(1, 158, 5), (1, 161, 5), (1, 154, 5)]);
        assert!((trained.cast("demoralizing_shout").duration - 2.25).abs() < 0.00001);
        assert_eq!(trained.cast("battle_shout").duration, 1.5);
        assert_eq!(trained.battle_shout_multiplier(), 1.25);
        let node = talent_definition(0, 158).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert!(text.contains("+40 » 50% duration"));
        assert!(ranges
            .iter()
            .any(|range| text[range.clone()].contains("40 » 50%")));
        assert!(!node.inline_description(5).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod dagger_specialization_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};

    #[test]
    fn dagger_crit_requires_worn_dagger_and_only_changes_rogue_direct_attacks() {
        for class in 0..9 {
            for kind in [
                None,
                Some(MeleeKind::Dagger),
                Some(MeleeKind::Sword),
                Some(MeleeKind::Fist),
                Some(MeleeKind::Mace),
            ] {
                for rank in [0, 1, 4, 5, 255] {
                    let base = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&[(1, 182, rank)]);
                    for ability in crate::data::ABILITIES {
                        let mut expected = base.cast(ability.key);
                        if class == 3
                            && kind == Some(MeleeKind::Dagger)
                            && matches!(
                                ability.key,
                                "sinister_strike"
                                    | "eviscerate"
                                    | "blade_flurry"
                                    | "gouge"
                                    | "kidney_shot"
                            )
                        {
                            expected.crit += 0.01 * f32::from(rank.min(5));
                        }
                        assert_eq!(
                            trained.cast(ability.key),
                            expected,
                            "{class} {kind:?} {rank} {}",
                            ability.key
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }

    #[test]
    fn dagger_preview_colors_inline_critical_chance_and_caps_rank() {
        let node = talent_definition(3, 182).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert!(ranges
            .iter()
            .any(|range| text[range.clone()].contains("4 » 5%")));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod rogue_sword_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn rogue_sword_proc_is_scoped_to_worn_swords_direct_hits_and_capped_training() {
        for class in 0..9 {
            for rank in [0, 1, 4, 5, 255] {
                for kind in [
                    None,
                    Some(MeleeKind::Sword),
                    Some(MeleeKind::Dagger),
                    Some(MeleeKind::Mace),
                ] {
                    let base = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&[(1, 242, rank)]);
                    for key in crate::data::ABILITIES.iter().map(|a| a.key).chain([
                        "sword_specialization",
                        "Thorns",
                        "deep_wounds",
                    ]) {
                        assert_eq!(trained.cast(key), base.cast(key));
                        for damage in [0.0, -1.0, f32::NAN, f32::INFINITY, 12.0] {
                            let eligible = class == 3
                                && kind == Some(MeleeKind::Sword)
                                && damage == 12.0
                                && matches!(
                                    key,
                                    "sinister_strike"
                                        | "eviscerate"
                                        | "blade_flurry"
                                        | "gouge"
                                        | "kidney_shot"
                                );
                            let expected = if eligible {
                                0.01 * f32::from(rank.min(5))
                            } else {
                                0.0
                            };
                            assert_eq!(
                                trained.sword_extra_attack_chance(key, damage),
                                expected,
                                "{class} {rank} {kind:?} {key}"
                            );
                        }
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }
    #[test]
    fn rogue_sword_preview_colors_inline_proc_chance() {
        let node = talent_definition(3, 242).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert!(ranges
            .iter()
            .any(|range| text[range.clone()].contains("4 » 5%")));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod fist_specialization_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};

    #[test]
    fn fist_crit_requires_worn_fist_and_only_changes_rogue_direct_attacks() {
        for class in 0..9 {
            for kind in [
                None,
                Some(MeleeKind::Fist),
                Some(MeleeKind::Sword),
                Some(MeleeKind::Dagger),
                Some(MeleeKind::Mace),
            ] {
                for rank in [0, 1, 4, 5, 255] {
                    let base = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&[(1, 183, rank)]);
                    for ability in crate::data::ABILITIES {
                        let mut expected = base.cast(ability.key);
                        if class == 3
                            && kind == Some(MeleeKind::Fist)
                            && matches!(
                                ability.key,
                                "sinister_strike"
                                    | "eviscerate"
                                    | "blade_flurry"
                                    | "gouge"
                                    | "kidney_shot"
                            )
                        {
                            expected.crit += 0.01 * f32::from(rank.min(5));
                        }
                        assert_eq!(
                            trained.cast(ability.key),
                            expected,
                            "{class} {kind:?} {rank} {}",
                            ability.key
                        );
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }

    #[test]
    fn dagger_and_fist_training_never_stack_on_one_weapon() {
        for kind in [MeleeKind::Dagger, MeleeKind::Fist, MeleeKind::Sword] {
            let base = Milestones {
                class: 3,
                weapon: Some(MeleeWeapon {
                    kind,
                    two_handed: false,
                }),
                ..Default::default()
            };
            let trained = base.with_ranks(&[(1, 182, 5), (1, 183, 5)]);
            let expected = if kind == MeleeKind::Sword { 0.0 } else { 0.05 };
            assert!((trained.cast("sinister_strike").crit - expected).abs() < 0.000001);
            assert_eq!(trained.cast("rupture").crit, 0.0);
        }
    }

    #[test]
    fn fist_preview_colors_inline_critical_chance_and_caps_rank() {
        let node = talent_definition(3, 183).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert!(ranges
            .iter()
            .any(|range| text[range.clone()].contains("4 » 5%")));
        assert!(!node.inline_description(255).unwrap().contains(" » "));
    }
}

#[cfg(test)]
mod rogue_mace_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn mace_skill_conversion_and_stun_are_scoped_and_capped() {
        for class in 0..9 {
            for kind in [
                None,
                Some(MeleeKind::Mace),
                Some(MeleeKind::Fist),
                Some(MeleeKind::Sword),
            ] {
                for rank in [0, 1, 2, 3, 4, 5, 255] {
                    let base = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    };
                    let trained = base.with_ranks(&[(1, 184, rank)]);
                    for key in crate::data::ABILITIES
                        .iter()
                        .map(|a| a.key)
                        .chain(["sword_specialization", "deep_wounds"])
                    {
                        let eligible = class == 3
                            && kind == Some(MeleeKind::Mace)
                            && matches!(
                                key,
                                "sinister_strike"
                                    | "eviscerate"
                                    | "blade_flurry"
                                    | "gouge"
                                    | "kidney_shot"
                            );
                        let mut expected = base.cast(key);
                        if eligible {
                            expected.damage *= 1.0 + 0.01 * f32::from(rank.min(5));
                        }
                        assert_eq!(trained.cast(key), expected, "{class} {kind:?} {rank} {key}");
                        for damage in [0.0, -1.0, f32::NAN, f32::INFINITY, 20.0] {
                            let chance = if eligible && damage == 20.0 {
                                [0.0, 0.01, 0.02, 0.03, 0.04, 0.06][usize::from(rank.min(5))]
                            } else {
                                0.0
                            };
                            assert_eq!(
                                trained.mace_stun_chance(key, damage),
                                chance,
                                "{class} {kind:?} {rank} {key}"
                            );
                        }
                    }
                    assert_eq!(trained.with_ranks(&[]), base);
                }
            }
        }
    }
    #[test]
    fn both_mace_upgrade_values_are_inline_and_green() {
        let node = talent_definition(3, 184).unwrap();
        let (text, ranges) = node.inline_description_with_values(4).unwrap();
        assert_eq!(ranges.len(), 2);
        assert_eq!(&text[ranges[0].clone()], "4 » 6% chance");
        assert_eq!(&text[ranges[1].clone()], "+4 » 5% direct damage");
        let max = node.inline_description(255).unwrap();
        assert!(max.contains("6% chance"));
        assert!(max.contains("+5% direct damage"));
        assert!(!max.contains(" » "));
        assert!(node
            .rank_preview(4)
            .contains("4 » 6% stun chance; +4 » 5% direct damage"));
    }
}
