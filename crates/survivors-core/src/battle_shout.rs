//! A cast-owned weapon buff, measured against active run time.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BattleShout {
    strength: f32,
    started: f32,
    expires: f32,
}

impl BattleShout {
    /// Refresh one snapshot; recasting never stacks the same buff with itself.
    pub fn refresh(&mut self, strength: f32, duration: f32, elapsed: f32) -> bool {
        let expires = elapsed + duration;
        if !strength.is_finite()
            || strength <= 0.0
            || !duration.is_finite()
            || duration <= 0.0
            || !elapsed.is_finite()
            || elapsed < 0.0
            || !expires.is_finite()
        {
            return false;
        }
        *self = Self {
            strength,
            started: elapsed,
            expires,
        };
        true
    }

    pub fn remaining(self, elapsed: f32) -> f32 {
        if !elapsed.is_finite() || elapsed < self.started {
            return 0.0;
        }
        (self.expires - elapsed).max(0.0)
    }

    pub fn affects(source: &str) -> bool {
        matches!(
            source,
            "cleave"
                | "heroic_strike"
                | "whirlwind"
                | "sweeping_strikes"
                | "execute"
                | "mortal_strike"
                | "retaliation"
        )
    }

    /// Original weapon casts snapshot the buff; generated hits never multiply it again.
    pub fn bonus(self, source: &str, elapsed: f32) -> f32 {
        if self.remaining(elapsed) > 0.0 && Self::affects(source) {
            self.strength
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activates_refreshes_and_expires_on_run_time_without_stacking() {
        let mut buff = BattleShout::default();
        assert_eq!(buff.bonus("cleave", 0.0), 0.0);
        assert!(buff.refresh(0.1, 8.0, 20.0));
        assert_eq!(buff.remaining(19.0), 0.0);
        assert_eq!(buff.remaining(20.0), 8.0);
        // A paused run reads the same time and cannot consume duration.
        for _ in 0..100 {
            assert_eq!(buff.bonus("cleave", 20.0), 0.1);
            assert_eq!(buff.remaining(20.0), 8.0);
        }
        assert_eq!(buff.bonus("cleave", 27.99), 0.1);
        assert_eq!(buff.bonus("cleave", 28.0), 0.0);
        assert!(buff.refresh(0.125, 10.0, 25.0));
        assert_eq!(buff.bonus("cleave", 28.0), 0.125);
        assert_eq!(buff.remaining(28.0), 7.0);
        assert_eq!(buff.bonus("cleave", 35.0), 0.0);
        assert_eq!(BattleShout::default().bonus("cleave", 30.0), 0.0);
    }

    #[test]
    fn weapon_casts_qualify_but_shouts_ticks_and_generated_strikes_do_not() {
        let mut buff = BattleShout::default();
        assert!(buff.refresh(0.125, 8.0, 0.0));
        for source in [
            "cleave",
            "heroic_strike",
            "whirlwind",
            "sweeping_strikes",
            "execute",
            "mortal_strike",
            "retaliation",
        ] {
            assert_eq!(buff.bonus(source, 1.0), 0.125, "{source}");
        }
        for source in [
            "battle_shout",
            "demoralizing_shout",
            "thunder_clap",
            "deep_wounds",
            "sword_specialization",
            "ignite",
            "fireball",
            "",
        ] {
            assert_eq!(buff.bonus(source, 1.0), 0.0, "{source}");
        }
    }

    #[test]
    fn invalid_values_cannot_poison_or_replace_a_valid_buff() {
        let mut buff = BattleShout::default();
        assert!(buff.refresh(0.1, 8.0, 0.0));
        let saved = buff;
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0] {
            assert!(!buff.refresh(invalid, 8.0, 1.0));
            assert!(!buff.refresh(0.1, invalid, 1.0));
            assert_eq!(buff, saved);
        }
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
            assert!(!buff.refresh(0.1, 8.0, invalid));
            assert_eq!(buff.bonus("cleave", invalid), 0.0);
            assert_eq!(buff, saved);
        }
        assert!(!buff.refresh(0.1, f32::MAX, f32::MAX));
        assert_eq!(buff, saved);
    }
}
