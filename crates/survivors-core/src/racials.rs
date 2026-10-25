//! Authored Survivors racials, independent of vanilla racial spell mechanics.

pub const HUMAN: u8 = 1;
pub const PERCEPTION_PERIOD: f32 = 30.0;
pub const PERCEPTION_DURATION: f32 = 10.0;
pub const PERCEPTION_DAMAGE: f32 = 0.15;
pub const HUMAN_EXPERIENCE: f32 = 0.10;
pub const HUMAN_GOLD: f32 = 0.20;

/// Run time stops with gameplay. The first proc starts at 30 seconds, not at spawn.
pub fn perception_active(race: u8, elapsed: f32) -> bool {
    race == HUMAN
        && elapsed.is_finite()
        && elapsed >= PERCEPTION_PERIOD
        && elapsed % PERCEPTION_PERIOD < PERCEPTION_DURATION
}

/// Bonuses are rebuilt from run time; timed effects never accumulate or use wall time.
#[derive(Clone, Copy, Debug, Default)]
pub struct Effects {
    pub bonus: crate::campaign::Bonus,
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub regen: f32,
    pub haste: f32,
}

fn periodic(elapsed: f32, duration: f32) -> bool {
    elapsed.is_finite() && elapsed >= 30.0 && elapsed % 30.0 < duration
}

pub fn effects(race: u8, elapsed: f32) -> Effects {
    let mut result = Effects {
        damage_multiplier: 1.0,
        health_multiplier: 1.0,
        ..Default::default()
    };
    let b = &mut result.bonus;
    match race {
        1 => {
            b.growth = HUMAN_EXPERIENCE;
            b.gold = HUMAN_GOLD;
            if perception_active(race, elapsed) {
                result.damage_multiplier = 1.0 + PERCEPTION_DAMAGE;
            }
        }
        2 => {
            if periodic(elapsed, 8.0) {
                result.damage_multiplier = 1.20;
            }
            b.reduction = 0.05;
            b.might = 0.05;
        }
        3 => {
            b.reduction = 0.03 + if periodic(elapsed, 8.0) { 0.15 } else { 0.0 };
            b.gold = 0.10;
        }
        4 => {
            b.speed = 0.05;
            b.reduction = if periodic(elapsed, 6.0) { 0.10 } else { 0.0 };
            result.regen = 0.3;
        }
        5 => {
            result.regen = 0.5;
            b.reduction = 0.05;
            b.crit_damage = 0.10;
        }
        6 => {
            result.health_multiplier = 1.10;
            b.armor = 5.0;
            b.pickup = 0.15;
        }
        7 => {
            b.growth = 0.10;
            b.speed = 0.05;
            b.area = 0.10;
        }
        8 => {
            result.haste = if periodic(elapsed, 10.0) { 0.15 } else { 0.0 };
            result.regen = 0.5;
            b.might = 0.05;
        }
        _ => {}
    }
    result
}

pub fn damage_multiplier(race: u8, elapsed: f32) -> f32 {
    effects(race, elapsed).damage_multiplier
}

/// Authored Survivors adaptations, not claims about vanilla racial spell behavior.
pub fn description(race: u8) -> Option<[String; 3]> {
    let lines = match race {
        1 => [
            "Perception: every 30 sec, +15% damage for 10 sec",
            "The Human Spirit: +10% experience",
            "Diplomacy: +20% gold",
        ],
        2 => [
            "Blood Fury: every 30 sec, +20% damage for 8 sec",
            "Hardiness: 5% reduced damage taken",
            "Warrior Spirit: +5% damage",
        ],
        3 => [
            "Stoneform: every 30 sec, 15% reduced damage taken for 8 sec",
            "Mountain Resilience: 3% reduced damage taken",
            "Treasure Sense: +10% gold",
        ],
        4 => [
            "Quickness: +5% movement speed",
            "Shadowmeld: every 30 sec, 10% reduced damage taken for 6 sec",
            "Elune's Grace: regenerate 0.3 health per sec",
        ],
        5 => [
            "Unending Hunger: regenerate 0.5 health per sec",
            "Will of the Forsaken: 5% reduced damage taken",
            "Shadow Affinity: +10% critical damage",
        ],
        6 => [
            "Endurance: +10% maximum health",
            "Thick Hide: +5 armor",
            "Earthmother's Reach: +15% pickup radius",
        ],
        7 => [
            "Expansive Mind: +10% experience",
            "Nimble: +5% movement speed",
            "Ingenious Design: +10% area radius",
        ],
        8 => [
            "Berserking: every 30 sec, +15% haste for 10 sec",
            "Regeneration: regenerate 0.5 health per sec",
            "Hunter's Instinct: +5% damage",
        ],
        _ => return None,
    };
    Some(lines.map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_playable_race_has_rules_and_timed_windows_expire() {
        for race in 1..=8 {
            assert!(description(race).is_some());
            for elapsed in [-1.0, 0.0, 29.999, 40.0, f32::NAN, f32::INFINITY] {
                let idle = effects(race, 0.0);
                let actual = effects(race, elapsed);
                assert_eq!(actual.damage_multiplier, idle.damage_multiplier);
                assert_eq!(actual.bonus.reduction, idle.bonus.reduction);
                assert_eq!(actual.haste, idle.haste);
            }
        }
        assert_eq!(effects(2, 30.0).damage_multiplier, 1.2);
        assert_eq!(effects(2, 38.0).damage_multiplier, 1.0);
        assert!((effects(3, 37.999).bonus.reduction - 0.18).abs() < 0.00001);
        assert_eq!(effects(3, 38.0).bonus.reduction, 0.03);
        assert_eq!(effects(4, 35.999).bonus.reduction, 0.10);
        assert_eq!(effects(4, 36.0).bonus.reduction, 0.0);
        assert_eq!(effects(8, 39.999).haste, 0.15);
        assert_eq!(effects(8, 40.0).haste, 0.0);
        assert_eq!(effects(8, 60.0).haste, 0.15);
    }

    #[test]
    fn perception_uses_half_open_windows_after_first_period() {
        for t in [
            0.0,
            10.0,
            29.999,
            40.0,
            59.999,
            70.0,
            f32::NAN,
            f32::INFINITY,
        ] {
            assert_eq!(damage_multiplier(HUMAN, t), 1.0, "time {t}");
        }
        for t in [30.0, 39.999, 60.0, 69.999, 1200.0] {
            assert_eq!(damage_multiplier(HUMAN, t), 1.15, "time {t}");
        }
        for race in [0, 9, 255] {
            assert_eq!(damage_multiplier(race, 30.0), 1.0);
            assert!(description(race).is_none());
        }
    }
}
