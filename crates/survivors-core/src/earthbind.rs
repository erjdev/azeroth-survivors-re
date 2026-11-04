//! Earthbind's destructible lifetime and passive pulse clock, in active run seconds.
//! Installed spells 2484, 6474 and 3600 supply the base values below.

pub const HEALTH: f32 = 5.0;
pub const LIFETIME: f32 = 45.0;
pub const RADIUS: f32 = 10.0;
pub const INTERVAL: f32 = 3.0;
pub const SLOW_SPEED: f32 = 0.5;
pub const SLOW_SECONDS: f32 = 5.0;

#[derive(Clone, Debug)]
pub struct Earthbind {
    health: f32,
    started: f32,
    expires: f32,
    observed: f32,
    next_pulse: u8,
}

impl Earthbind {
    /// A new deployment replaces the caller's previous Earthbind, never revives it.
    pub fn deploy(now: f32) -> Option<Self> {
        let expires = now + LIFETIME;
        if !now.is_finite()
            || now < 0.0
            || !expires.is_finite()
            || expires <= now
            || expires + INTERVAL <= expires
        {
            return None;
        }
        Some(Self {
            health: HEALTH,
            started: now,
            expires,
            observed: now,
            next_pulse: 1,
        })
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn alive(&self, now: f32) -> bool {
        now.is_finite()
            && now >= self.started
            && now >= self.observed
            && now < self.expires
            && self.health > 0.0
    }

    /// Destruction is immediate; excess damage is discarded, never redirected.
    pub fn hurt(&mut self, damage: f32, now: f32) -> f32 {
        if !damage.is_finite() || damage <= 0.0 || !self.alive(now) {
            return 0.0;
        }
        self.observed = now;
        let removed = damage.min(self.health);
        self.health -= removed;
        removed
    }

    /// Return due pulse times, not the observation time, so old slows can expire.
    /// The terminal lifetime boundary is exclusive: an expired totem cannot pulse.
    /// No target information is retained; an empty pulse is spent by this call.
    pub fn pulses(&mut self, now: f32) -> Vec<f32> {
        if !now.is_finite() || now < self.observed || self.health <= 0.0 {
            return Vec::new();
        }
        self.observed = now;
        let mut due = Vec::new();
        while f32::from(self.next_pulse) * INTERVAL < LIFETIME {
            let at = self.started + f32::from(self.next_pulse) * INTERVAL;
            if at > now || at >= self.expires {
                break;
            }
            due.push(at);
            self.next_pulse += 1;
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slows::{Slows, Source};

    #[test]
    fn fine_and_coarse_updates_deliver_the_same_bounded_pulses_once() {
        let expected: Vec<_> = (1..15).map(|n| n as f32 * INTERVAL).collect();
        for step in [0.25, 1.0, 7.0, 60.0] {
            let mut totem = Earthbind::deploy(0.0).unwrap();
            assert!(totem.pulses(0.0).is_empty());
            let mut times = Vec::new();
            let mut now = 0.0;
            while now < 60.0 {
                now += step;
                times.extend(totem.pulses(now));
                assert!(totem.pulses(now).is_empty());
            }
            assert_eq!(times, expected);
            assert!(!totem.alive(now));
            assert!(totem.pulses(100.0).is_empty());
        }
    }

    #[test]
    fn destruction_stops_pulses_and_does_not_reanimate_after_clock_changes() {
        let mut totem = Earthbind::deploy(10.0).unwrap();
        assert_eq!(totem.pulses(13.0), [13.0]);
        assert_eq!(totem.hurt(2.0, 14.0), 2.0);
        assert_eq!(totem.health(), 3.0);
        assert!(totem.alive(14.0));
        assert_eq!(totem.hurt(100.0, 15.0), 3.0);
        assert_eq!(totem.health(), 0.0);
        for now in [0.0, 10.0, 16.0, 100.0] {
            assert!(!totem.alive(now));
            assert!(totem.pulses(now).is_empty());
            assert_eq!(totem.hurt(1.0, now), 0.0);
        }
        let replacement = Earthbind::deploy(16.0).unwrap();
        assert_eq!(replacement.health(), HEALTH);
        assert!(replacement.alive(16.0));
    }

    #[test]
    fn stale_or_invalid_updates_cannot_damage_or_duplicate_pulses() {
        for now in [-1.0, f32::NAN, f32::INFINITY, f32::MAX, 1_000_000_000.0] {
            assert!(Earthbind::deploy(now).is_none());
        }
        let mut totem = Earthbind::deploy(0.0).unwrap();
        assert_eq!(totem.pulses(6.0), [3.0, 6.0]);
        for now in [-1.0, 3.0, f32::NAN, f32::INFINITY] {
            assert!(totem.pulses(now).is_empty());
            assert_eq!(totem.hurt(1.0, now), 0.0);
        }
        for damage in [-1.0, 0.0, f32::NAN, f32::INFINITY] {
            assert_eq!(totem.hurt(damage, 7.0), 0.0);
        }
        assert_eq!(totem.health(), HEALTH);
        assert_eq!(totem.pulses(9.0), [9.0]);
        assert_eq!(totem.hurt(1.0, LIFETIME), 0.0);
        assert!(!totem.alive(LIFETIME));
    }

    #[test]
    fn slow_uses_pulse_time_and_survives_departure_only_for_its_own_duration() {
        let mut totem = Earthbind::deploy(0.0).unwrap();
        let mut slows = Slows::default();
        let source = Source::Spell("earthbind_totem");
        for time in totem.pulses(7.0) {
            assert!(slows.apply(source, SLOW_SPEED, SLOW_SECONDS, time));
        }
        assert_eq!(slows.state(7.0), (0.5, 4.0));
        // The enemy has left the radius; consume empty pulses without refreshing.
        assert_eq!(totem.pulses(9.0), [9.0]);
        assert_eq!(slows.state(11.0), (1.0, 0.0));
        for time in totem.pulses(60.0) {
            slows.apply(source, SLOW_SPEED, SLOW_SECONDS, time);
        }
        assert_eq!(slows.state(60.0), (1.0, 0.0));
    }
}
