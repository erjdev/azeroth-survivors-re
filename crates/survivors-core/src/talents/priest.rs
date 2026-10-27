//! Automatic Power Infusion follows active run time; no manual target or mana pool.
use super::{Milestones, TalentStat, PRIEST_HOLY, PRIEST_SHADOW};

#[derive(Clone, Copy, Debug, Default)]
pub struct PowerInfusionClock {
    next: f32,
    until: f32,
}
impl PowerInfusionClock {
    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.
            || talents.class != 4
            || !(PRIEST_HOLY.contains(&key) || PRIEST_SHADOW.contains(&key))
            || (elapsed >= self.until && elapsed < self.next)
        {
            return 0.;
        }
        talents.trained_value(TalentStat::PowerInfusion)
    }
    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if elapsed >= self.next && self.bonus(talents, key, elapsed) > 0. {
            self.until = elapsed + 15.;
            self.next = elapsed + 180.;
        }
    }
    pub fn remaining(self, talents: Milestones, elapsed: f32) -> f32 {
        if !elapsed.is_finite()
            || elapsed < 0.
            || talents.class != 4
            || talents.trained_value(TalentStat::PowerInfusion) <= 0.
        {
            return 0.;
        }
        (self.until - elapsed).max(0.)
    }
    pub fn active_bonus(self, talents: Milestones, elapsed: f32) -> f32 {
        if self.remaining(talents, elapsed) > 0. {
            talents.trained_value(TalentStat::PowerInfusion)
        } else {
            0.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn infusion_waits_for_success_and_cannot_refresh_or_bank_charges() {
        for class in [0, 4, 6] {
            for rank in [0, 1, 255] {
                let trained = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(0, 322, rank)]);
                let expected = if class == 4 && rank > 0 { 0.2 } else { 0. };
                let mut clock = PowerInfusionClock::default();
                for time in [f32::NAN, f32::INFINITY, -1.] {
                    clock.consume(trained, "smite", time);
                    assert_eq!(clock.bonus(trained, "smite", time), 0.);
                    assert_eq!(clock.active_bonus(trained, time), 0.);
                }
                for key in ["renew", "power_word_shield", "fireball"] {
                    clock.consume(trained, key, 0.);
                    assert_eq!(clock.bonus(trained, key, 0.), 0.);
                }
                assert_eq!(clock.active_bonus(trained, 1000.), 0.);
                clock.consume(trained, "smite", 1000.);
                for key in PRIEST_HOLY.iter().chain(PRIEST_SHADOW) {
                    assert_eq!(clock.bonus(trained, key, 1014.99), expected);
                }
                assert_eq!(clock.active_bonus(trained, 1014.99), expected);
                clock.consume(trained, "holy_nova", 1014.);
                assert_eq!(clock.active_bonus(trained, 1015.), 0.);
                assert_eq!(clock.bonus(trained, "smite", 1179.99), 0.);
                assert_eq!(clock.bonus(trained, "smite", 1180.), expected);
                assert_eq!(clock.active_bonus(trained.with_ranks(&[]), 1001.), 0.);
            }
        }
        let node = super::super::talent_definition(4, 322).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("+0 » 20% Priest"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}
