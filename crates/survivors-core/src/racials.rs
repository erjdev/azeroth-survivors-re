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

pub fn damage_multiplier(race: u8, elapsed: f32) -> f32 {
    if perception_active(race, elapsed) {
        1.0 + PERCEPTION_DAMAGE
    } else {
        1.0
    }
}

/// None means the race's Survivors rules have not been authored yet.
pub fn description(race: u8) -> Option<[String; 3]> {
    (race == HUMAN).then(|| {
        [
            format!("Perception: every {PERCEPTION_PERIOD:.0} sec, +{:.0}% damage for {PERCEPTION_DURATION:.0} sec", PERCEPTION_DAMAGE * 100.0),
            format!("The Human Spirit: +{:.0}% experience", HUMAN_EXPERIENCE * 100.0),
            format!("Diplomacy: +{:.0}% gold", HUMAN_GOLD * 100.0),
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for race in [0, 2, 3, 4, 5, 6, 7, 8] {
            assert_eq!(damage_multiplier(race, 30.0), 1.0);
            assert!(description(race).is_none());
        }
    }
}
