//! Target-owned Stormstrike vulnerability (spell 17364): two Nature hits, 20%, 12 seconds.
//! Application belongs to a successful Stormstrike; consumption belongs to Nature hit settlement.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stormstrike {
    applied: f32,
    expires: f32,
    charges: u8,
}

impl Stormstrike {
    /// Refreshing replaces the old charges and expiry; it never stacks potency or charges.
    pub fn apply(&mut self, now: f32) -> bool {
        if !now.is_finite() || now < 0.0 || now + 12.0 <= now {
            return false;
        }
        *self = Self {
            applied: now,
            expires: now + 12.0,
            charges: 2,
        };
        true
    }

    pub fn charges(self, now: f32) -> u8 {
        if now.is_finite() && now >= self.applied && now < self.expires {
            self.charges
        } else {
            0
        }
    }

    pub fn remaining(self, now: f32) -> f32 {
        if self.charges(now) > 0 {
            self.expires - now
        } else {
            0.0
        }
    }

    /// Call once per connected, positive Nature damage event, after avoidance/immunity.
    /// Other schools, missed attacks, and synthetic multistrike copies must not call this.
    /// Returns a multiplier so the caller can preserve its normal critical/mitigation order.
    pub fn consume_nature_hit(&mut self, damage: f32, now: f32) -> f32 {
        if !damage.is_finite() || damage <= 0.0 || self.charges(now) == 0 {
            return 1.0;
        }
        self.charges -= 1;
        1.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_positive_hits_consume_once_each_and_refresh_replaces() {
        let mut target = Stormstrike::default();
        assert_eq!(target.consume_nature_hit(20.0, 0.0), 1.0);
        assert!(target.apply(1.0));
        assert_eq!(target.consume_nature_hit(20.0, 1.0), 1.2);
        assert_eq!(target.charges(1.0), 1);
        assert!(target.apply(2.0));
        assert_eq!(target.charges(2.0), 2);
        assert_eq!(target.remaining(2.0), 12.0);
        assert_eq!(target.consume_nature_hit(40.0, 2.0), 1.2);
        assert_eq!(target.consume_nature_hit(10.0, 2.0), 1.2);
        assert_eq!(target.consume_nature_hit(10.0, 2.0), 1.0);
        assert_eq!(target.remaining(2.0), 0.0);
    }

    #[test]
    fn expiry_uses_active_time_and_targets_are_independent() {
        let mut first = Stormstrike::default();
        let second = Stormstrike::default();
        first.apply(10.0);
        for _ in 0..10 {
            assert_eq!(first.remaining(10.0), 12.0);
        }
        assert_eq!(first.charges(9.0), 0);
        assert_eq!(first.charges(21.999), 2);
        assert_eq!(first.consume_nature_hit(5.0, 22.0), 1.0);
        assert_eq!(second.charges(10.0), 0);
        assert_eq!(first.remaining(22.0), 0.0);
    }

    #[test]
    fn invalid_damage_and_time_preserve_live_charges() {
        let mut target = Stormstrike::default();
        target.apply(2.0);
        for damage in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(target.consume_nature_hit(damage, 3.0), 1.0);
        }
        for now in [-1.0, f32::NAN, f32::INFINITY, f32::MAX] {
            assert!(!target.apply(now));
            assert_eq!(target.consume_nature_hit(1.0, now), 1.0);
        }
        assert_eq!(target.charges(3.0), 2);
        assert_eq!(target.remaining(3.0), 11.0);
    }
}
