//! Survivors talent effects and dependency rules, independent of engine spell visuals.

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
}

impl Default for Milestones {
    fn default() -> Self {
        Self {
            class: 0,
            points: [0; 3],
            training: [0; 64],
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CastEffect {
    pub damage: f32,
    pub radius: f32,
    pub range: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub crit_damage: f32,
    pub extra_targets: usize,
    pub heal_fraction: f32,
    pub shield_fraction: f32,
    pub shield_seconds: f32,
    pub immune_seconds: f32,
    pub wrath_advance: f32,
    pub storm_fraction: f32,
}

impl Default for CastEffect {
    fn default() -> Self {
        Self {
            damage: 1.0,
            radius: 1.0,
            range: 1.0,
            duration: 1.0,
            cooldown: 1.0,
            crit: 0.0,
            crit_damage: 0.0,
            extra_targets: 0,
            heal_fraction: 0.0,
            shield_fraction: 0.0,
            shield_seconds: 0.0,
            immune_seconds: 0.0,
            wrath_advance: 0.0,
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
        if self.class == 3 && ROGUE_SPELLS.contains(&key) {
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
        if self.class == 6 {
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
                    effect.wrath_advance = 1.0;
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
                    TalentStat::ArcaneInstability
                        if MAGE_ARCANE.contains(&key)
                            || MAGE_FIRE.contains(&key)
                            || MAGE_FROST.contains(&key) =>
                    {
                        effect.damage *= 1.0 + value;
                        effect.crit += value;
                    }
                    TalentStat::ElementalPrecision
                        if MAGE_FIRE.contains(&key) || MAGE_FROST.contains(&key) =>
                    {
                        effect.damage *= 1.0 + value;
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
                    TalentStat::SpellDamage(spell) if spell == key => effect.damage *= 1.0 + value,
                    TalentStat::SpellCooldown(spell) if spell == key => {
                        effect.cooldown *= 1.0 - value
                    }
                    TalentStat::SpellCrit(spell) if spell == key => effect.crit += value,
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
        if effect.shield_fraction > 0.0 {
            // All talent-granted absorbs start with three seconds; duration ranks add to it.
            effect.shield_seconds += 3.0;
        } else {
            effect.shield_seconds = 0.0;
        }
        effect
    }

    fn trained_value(self, stat: TalentStat) -> f32 {
        definitions(self.class)
            .iter()
            .enumerate()
            .filter(|(_, node)| node.stat == stat)
            .map(|(index, node)| node.per_rank * f32::from(self.training[index].min(node.max_rank)))
            .sum()
    }

    /// Piercing Howl supplements, rather than replaces, the shout's damage weakening.
    pub fn secondary_slow(self, key: &str, boss: bool) -> Option<(f32, f32)> {
        if self.class != 0 || key != "demoralizing_shout" {
            return None;
        }
        let duration = if self.points[1] >= 10 { 3.0 } else { 0.0 }
            + self.trained_value(TalentStat::SecondarySlowDuration("demoralizing_shout"));
        (duration > 0.).then_some((if boss { 0.75 } else { 0.5 }, duration))
    }

    pub fn secondary_stun(self, key: &'static str, boss: bool) -> f32 {
        let milestone = if self.class == 0 && key == "heroic_strike" && self.points[2] >= 20 {
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

    pub fn flurry_haste(self, active: bool) -> f32 {
        if self.class == 0 && active {
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
            && matches!(
                key,
                "cleave" | "heroic_strike" | "execute" | "mortal_strike"
            )
    }

    /// Death Wish's risk applies only during the actual Retaliation buff.
    pub fn retaliation_risk(self, active: bool) -> f32 {
        if self.class == 0 && self.points[1] >= 20 && active {
            1.10
        } else {
            1.0
        }
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

    /// Fire hit proc; periodic Ignite ticks and non-Mage sources cannot trigger it.
    pub fn impact_chance(self, key: &str, damage: f32) -> f32 {
        if self.class != 6 || !damage.is_finite() || damage <= 0.0 || !MAGE_FIRE.contains(&key) {
            return 0.0;
        }
        self.trained_value(TalentStat::Impact)
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
    if class == 3 {
        return match (tree, threshold) {
            (0, 10) => Some("Rogue spells: +5 percentage points critical chance; periodic ticks do not crit"),
            (0, 20) => Some("Rogue direct attacks: +20 percentage points critical damage; excludes Rupture ticks"),
            (0, 30) => Some("Eviscerate always critically strikes (Survivors automatic Cold Blood)"),
            (1, 10) => Some("Sinister Strike: 10% shorter cooldown"),
            (1, 20) => Some("Blade Flurry: +30% radius; remains one area around the player"),
            (1, 30) => Some("Rogue spells: 15% shorter cooldown (Survivors automatic Adrenaline Rush)"),
            _ => None,
        };
    }
    if class == 6 {
        return match (tree, threshold) {
            (2, 10) => Some("Frostbolt: +1 target; area spells remain one area"),
            (2, 20) => Some("Frost spells: +20 percentage points critical damage"),
            (2, 30) => Some("Frost Nova grants a 10% maximum-health absorb for 5 seconds once per successful cast; does not stack"),
            _ => None,
        };
    }
    if class == 2 {
        return match (tree, threshold) {
            (1, 10) => Some("Shots: 10% shorter cooldown; excludes Volley and traps"),
            (1, 20) => {
                Some("Shots: +20 percentage points critical damage; excludes Volley and traps")
            }
            (1, 30) => Some("Multi-Shot: +2 targets; Aimed Shot: +25% damage"),
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
        let subtlety = Milestones {
            points: [0, 0, 51],
            ..base
        };
        for key in ROGUE_SPELLS {
            assert_eq!(subtlety.cast(key), CastEffect::default());
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
    Damage,
    Cooldown,
    Armor,
    Crit,
    CritDamage,
    CriticalBleed,
    CriticalBurn,
    Frostbite,
    Impact,
    WintersChill,
    Shatter,
    Permafrost,
    BlizzardSlow,
    FlameThrowing,
    ArcticReach,
    EnrageDamage,
    BloodCrazeHealing,
    FlurryHaste,
    SecondarySlowDuration(&'static str),
    SecondaryStunDuration(&'static str),
    SecondaryRootDuration(&'static str),
    Reduction,
    Speed,
    ArcaneInstability,
    ElementalPrecision,
    SpellGroupDamage(&'static [&'static str]),
    SpellGroupCrit(&'static [&'static str]),
    SpellGroupCritDamage(&'static [&'static str]),
    SpellDamage(&'static str),
    SpellCooldown(&'static str),
    SpellCrit(&'static str),
    SpellDuration(&'static str),
    SpellHeal(&'static str),
    SpellShield(&'static str),
    SpellShieldDuration(&'static str),
    SpellImmune(&'static str),
    SpellRadius(&'static str),
}

impl TalentDefinition {
    /// This node's contribution only, before other talents, equipment and milestones.
    pub fn rank_preview(&self, rank: u8) -> String {
        let rank = rank.min(self.max_rank);
        if self.stat == TalentStat::Permafrost {
            let strength = |r| (self.per_rank * f32::from(r) * 100.0).round() as u32;
            return if rank == self.max_rank {
                format!("This talent: {} percentage points stronger slows; {rank} sec added duration (maximum rank)", strength(rank))
            } else {
                format!("This talent: {} » {} percentage points stronger slows; {rank} » {} sec added duration next rank", strength(rank), strength(rank + 1), rank + 1)
            };
        }
        if self.stat == TalentStat::ArcaneInstability {
            let value = u32::from(rank);
            return if rank == self.max_rank {
                format!("This talent: {value}% damage; {value}% critical chance (maximum rank)")
            } else {
                format!(
                    "This talent: {value} » {}% damage; {value} » {}% critical chance next rank",
                    value + 1,
                    value + 1
                )
            };
        }
        let (scale, unit) = match self.stat {
            TalentStat::ArcaneInstability => unreachable!("handled above"),
            TalentStat::ElementalPrecision => (100.0, "% Fire and Frost damage"),
            TalentStat::Armor => (1.0, " armor"),
            TalentStat::SpellShield(_) => (100.0, "% maximum health absorbed per cast"),
            TalentStat::SpellImmune(_) => (1.0, " sec immunity per successful cast"),
            TalentStat::SpellShieldDuration(_) => (1.0, " s added shield duration"),
            TalentStat::SecondarySlowDuration(_) => (1.0, " s added slow duration"),
            TalentStat::SecondaryStunDuration(_) => (1.0, " s added stun duration"),
            TalentStat::SecondaryRootDuration(_) => (1.0, " s added root duration"),
            TalentStat::Damage | TalentStat::SpellDamage(_) | TalentStat::SpellGroupDamage(_) => {
                (100.0, "% damage")
            }
            TalentStat::Cooldown | TalentStat::SpellCooldown(_) => (100.0, "% cooldown reduction"),
            TalentStat::Crit | TalentStat::SpellCrit(_) | TalentStat::SpellGroupCrit(_) => {
                (100.0, "% critical chance")
            }
            TalentStat::CritDamage | TalentStat::SpellGroupCritDamage(_) => {
                (100.0, "% critical damage")
            }
            TalentStat::FlurryHaste => (100.0, "% haste while Flurry is active"),
            TalentStat::BloodCrazeHealing => {
                (100.0, "% maximum health over 6 seconds after health damage")
            }
            TalentStat::EnrageDamage => (100.0, "% damage while Enrage is active"),
            TalentStat::CriticalBleed => (100.0, "% of direct critical damage as bleed"),
            TalentStat::CriticalBurn => (100.0, "% of critical fire hit damage as Ignite"),
            TalentStat::WintersChill => (100.0, "% chance to apply Winter's Chill on frost damage"),
            TalentStat::Impact => (100.0, "% chance to stun on fire damage"),
            TalentStat::Frostbite => (100.0, "% chance to freeze on frost damage"),
            TalentStat::Shatter => (100.0, "% critical chance against frozen enemies"),
            TalentStat::Permafrost => (
                100.0,
                "% stronger frost slows; also +1 sec duration per rank",
            ),
            TalentStat::BlizzardSlow => (100.0, "% stronger Blizzard slow"),
            TalentStat::FlameThrowing => (100.0, "% fire spell targeting range"),
            TalentStat::ArcticReach => (100.0, "% frost targeting range / Nova and Cone radius"),
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
        description: "Demoralizing Shout: +10% radius per rank",
        stat: TalentStat::SpellRadius("demoralizing_shout"),
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

pub const PALADIN_TALENTS: &[TalentDefinition] = &[
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
];

static HUNTER_TALENTS: &[TalentDefinition] = &[
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
        id: 421, max_rank: 3, name: "Arcane Instability",
        description: "Mage spells: +1% damage and +1% critical chance per rank; excludes other-class and neutral jewelry spells",
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

pub fn definitions(class: usize) -> &'static [TalentDefinition] {
    match class {
        0 => WARRIOR_TALENTS,
        1 => PALADIN_TALENTS,
        2 => HUNTER_TALENTS,
        3 => ROGUE_TALENTS,
        6 => MAGE_TALENTS,
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
            TalentStat::Armor => bonus.armor += value,
            TalentStat::Crit => bonus.crit += value,
            TalentStat::CritDamage => bonus.crit_damage += value,
            TalentStat::Reduction => bonus.reduction += value,
            TalentStat::Speed => bonus.speed += value,
            TalentStat::SecondarySlowDuration(_)
            | TalentStat::SecondaryStunDuration(_)
            | TalentStat::SecondaryRootDuration(_)
            | TalentStat::EnrageDamage
            | TalentStat::BloodCrazeHealing
            | TalentStat::FlurryHaste
            | TalentStat::CriticalBleed
            | TalentStat::CriticalBurn
            | TalentStat::WintersChill
            | TalentStat::Impact
            | TalentStat::Frostbite
            | TalentStat::Shatter
            | TalentStat::Permafrost
            | TalentStat::BlizzardSlow
            | TalentStat::FlameThrowing
            | TalentStat::ArcticReach
            | TalentStat::ArcaneInstability
            | TalentStat::ElementalPrecision
            | TalentStat::SpellGroupDamage(_)
            | TalentStat::SpellGroupCrit(_)
            | TalentStat::SpellGroupCritDamage(_)
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
                    | TalentStat::FlurryHaste
                    | TalentStat::CriticalBleed
                    | TalentStat::CriticalBurn
                    | TalentStat::WintersChill
                    | TalentStat::Impact
                    | TalentStat::Frostbite
                    | TalentStat::Shatter
                    | TalentStat::Permafrost
                    | TalentStat::BlizzardSlow
                    | TalentStat::FlameThrowing
                    | TalentStat::ArcticReach
                    | TalentStat::ArcaneInstability
                    | TalentStat::ElementalPrecision
                    | TalentStat::SpellGroupDamage(_)
                    | TalentStat::SpellGroupCrit(_)
                    | TalentStat::SpellGroupCritDamage(_)
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
