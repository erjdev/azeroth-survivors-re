//! Stoneclaw's destructible lifetime, independent of the runtime forced-targeting policy.
//! Spell 5730 supplies the rank-one base health and lifetime. The caller supplies
//! the final health after level/rank/talent scaling; this module does not invent it.

pub const BASE_HEALTH: f32 = 50.0;
pub const LIFETIME: f32 = 15.0;
pub const RADIUS: f32 = 8.0;
pub const RECOVERY: f32 = 30.0;

#[derive(Clone, Debug)]
pub struct Stoneclaw {
    health: f32,
    maximum: f32,
    started: f32,
    expires: f32,
    observed: f32,
}

impl Stoneclaw {
    /// Create a fresh deployment. Replacements must be separate entities; this
    /// never revives a destroyed totem or adds health to a previous deployment.
    pub fn deploy(now: f32, health: f32) -> Option<Self> {
        let expires = now + LIFETIME;
        if !now.is_finite()
            || now < 0.0
            || !health.is_finite()
            || health <= 0.0
            || !expires.is_finite()
            || expires <= now
            // Allow ordinary floating-point rounding, but not a clock that
            // changes the authored lifetime by more than a millisecond.
            || (expires - now - LIFETIME).abs() > 0.001
        {
            return None;
        }
        Some(Self {
            health,
            maximum: health,
            started: now,
            expires,
            observed: now,
        })
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn maximum_health(&self) -> f32 {
        self.maximum
    }

    pub fn alive(&self, now: f32) -> bool {
        now.is_finite()
            && now >= self.started
            && now >= self.observed
            && now < self.expires
            && self.health > 0.0
    }

    /// Observe active-run time, including expiration. Once observed expired,
    /// a backwards timestamp cannot make the totem alive again.
    pub fn advance(&mut self, now: f32) -> bool {
        if now.is_finite() && now >= self.observed {
            self.observed = now;
        }
        self.alive(now)
    }

    /// Return only health actually removed. Excess damage is discarded, not
    /// redirected to the hero, and invalid/stale damage cannot heal or revive.
    pub fn hurt(&mut self, damage: f32, now: f32) -> f32 {
        if !damage.is_finite() || damage <= 0.0 || !self.advance(now) {
            return 0.0;
        }
        let removed = damage.min(self.health);
        self.health -= removed;
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_time_pauses_and_expiration_is_terminal_across_frame_sizes() {
        for step in [0.25, 1.0, 7.0, 20.0] {
            let mut totem = Stoneclaw::deploy(100.0, BASE_HEALTH).unwrap();
            // A paused run keeps its clock fixed, regardless of wall time.
            for _ in 0..60 {
                assert!(totem.advance(100.0));
            }
            let mut now = 100.0;
            while now < 120.0 {
                now += step;
                assert_eq!(totem.advance(now), now < 115.0);
            }
            assert!(!totem.advance(100.0));
            assert!(!totem.alive(114.0));
            assert_eq!(totem.hurt(1.0, 114.0), 0.0);
        }
    }

    #[test]
    fn fractional_cast_times_keep_the_lifetime_boundary() {
        for now in [0.1, 3.8, 123.456, 3600.25] {
            let mut totem = Stoneclaw::deploy(now, BASE_HEALTH).unwrap();
            assert!(totem.advance(now + LIFETIME - 0.01));
            assert!(!totem.advance(now + LIFETIME));
            assert!(!totem.advance(now));
        }
    }

    #[test]
    fn configured_health_absorbs_only_its_own_damage_and_replacement_is_fresh() {
        for health in [BASE_HEALTH, BASE_HEALTH * 1.25, BASE_HEALTH * 1.5] {
            let mut totem = Stoneclaw::deploy(0.0, health).unwrap();
            assert_eq!(totem.maximum_health(), health);
            assert_eq!(totem.hurt(20.0, 1.0), 20.0);
            assert_eq!(totem.health(), health - 20.0);
            assert_eq!(totem.hurt(1000.0, 2.0), health - 20.0);
            assert_eq!(totem.health(), 0.0);
            assert_eq!(totem.maximum_health(), health);
            assert!(!totem.advance(2.0));
            assert!(!totem.advance(1.0));
            assert_eq!(totem.hurt(1.0, 3.0), 0.0);
            let replacement = Stoneclaw::deploy(3.0, health).unwrap();
            assert!(replacement.alive(3.0));
            assert_eq!(replacement.health(), health);
        }
    }

    #[test]
    fn invalid_and_stale_inputs_cannot_create_or_change_health() {
        for now in [-1.0, f32::NAN, f32::INFINITY, f32::MAX, 1_000_000_000.0] {
            assert!(Stoneclaw::deploy(now, BASE_HEALTH).is_none());
        }
        for health in [-1.0, 0.0, f32::NAN, f32::INFINITY] {
            assert!(Stoneclaw::deploy(0.0, health).is_none());
        }
        let mut totem = Stoneclaw::deploy(5.0, BASE_HEALTH).unwrap();
        assert!(totem.advance(7.0));
        for now in [4.0, 6.0, f32::NAN, f32::INFINITY] {
            assert_eq!(totem.hurt(10.0, now), 0.0);
        }
        for damage in [-1.0, 0.0, f32::NAN, f32::INFINITY] {
            assert_eq!(totem.hurt(damage, 8.0), 0.0);
        }
        assert_eq!(totem.health(), BASE_HEALTH);
        assert_eq!(totem.hurt(10.0, 8.0), 10.0);
        assert_eq!(totem.hurt(1.0, 20.0), 0.0);
        assert!(!totem.alive(19.0));
    }
}
