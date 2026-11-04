//! Target-owned Hunter's Mark snapshot, independent of visual aura slots.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HunterMark<Caster> {
    caster: Option<Caster>,
    power: f32,
    started: f32,
    expires: f32,
}

impl<Caster> Default for HunterMark<Caster> {
    fn default() -> Self {
        Self {
            caster: None,
            power: 0.0,
            started: 0.0,
            expires: 0.0,
        }
    }
}

impl<Caster: Copy + Eq> HunterMark<Caster> {
    pub fn caster(self) -> Option<Caster> {
        self.caster
    }

    /// Replace one snapshot; repeated applications never add attack power together.
    pub fn apply(&mut self, caster: Caster, power: f32, duration: f32, elapsed: f32) -> bool {
        let expires = elapsed + duration;
        if !power.is_finite()
            || power <= 0.0
            || !duration.is_finite()
            || duration <= 0.0
            || !elapsed.is_finite()
            || elapsed < 0.0
            || !expires.is_finite()
            || expires <= elapsed
        {
            return false;
        }
        *self = Self {
            caster: Some(caster),
            power,
            started: elapsed,
            expires,
        };
        true
    }

    /// Absolute active-run time means pause frames cannot consume mark duration.
    pub fn remaining(self, elapsed: f32) -> f32 {
        if self.caster.is_none() || !elapsed.is_finite() || elapsed < self.started {
            return 0.0;
        }
        (self.expires - elapsed).max(0.0)
    }

    /// The target grants the bonus to qualifying attacks; reveal belongs to its caster.
    pub fn attack_power(self, elapsed: f32) -> f32 {
        if self.remaining(elapsed) > 0.0 {
            self.power
        } else {
            0.0
        }
    }

    pub fn reveals_to(self, caster: Caster, elapsed: f32) -> bool {
        self.caster == Some(caster) && self.remaining(elapsed) > 0.0
    }

    /// Caster loss must not clear a later replacement applied by someone else.
    pub fn clear_caster(&mut self, caster: Caster) {
        if self.caster == Some(caster) {
            *self = Self::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_and_casters_stay_independent_through_refresh_and_expiry() {
        let mut target = HunterMark::default();
        let other = HunterMark::<u64>::default();
        assert_eq!(target.attack_power(0.0), 0.0);
        assert!(target.apply(1, 20.0, 120.0, 10.0));
        assert_eq!(target.attack_power(10.0), 20.0);
        assert_eq!(other.attack_power(10.0), 0.0);
        assert!(target.reveals_to(1, 10.0));
        assert!(!target.reveals_to(2, 10.0));
        assert_eq!(target.remaining(10.0), 120.0);
        assert_eq!(
            target.remaining(10.0),
            120.0,
            "paused run time is unchanged"
        );
        assert!(target.apply(1, 23.0, 120.0, 20.0));
        assert_eq!(target.attack_power(20.0), 23.0, "refresh does not stack");
        assert_eq!(target.remaining(130.0), 10.0);
        assert_eq!(target.attack_power(140.0), 0.0);
        assert!(!target.reveals_to(1, 140.0));
        assert!(target.apply(2, 45.0, 120.0, 150.0));
        target.clear_caster(1);
        assert_eq!(target.attack_power(150.0), 45.0);
        assert!(!target.reveals_to(1, 150.0));
        assert!(target.reveals_to(2, 150.0));
        target.clear_caster(2);
        assert_eq!(target.attack_power(150.0), 0.0);
    }

    #[test]
    fn invalid_applications_preserve_the_live_mark() {
        let mut target = HunterMark::default();
        assert!(target.apply(1, 20.0, 120.0, 10.0));
        let original = target;
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0] {
            assert!(!target.apply(2, invalid, 120.0, 20.0));
            assert!(!target.apply(2, 45.0, invalid, 20.0));
            assert_eq!(target, original);
        }
        for elapsed in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
            assert!(!target.apply(2, 45.0, 120.0, elapsed));
            assert_eq!(target.attack_power(elapsed), 0.0);
            assert!(!target.reveals_to(1, elapsed));
            assert_eq!(target, original);
        }
        assert!(!target.apply(2, 45.0, f32::MAX, f32::MAX));
        assert!(!target.apply(2, 45.0, 1.0, f32::MAX));
        assert_eq!(target.attack_power(9.0), 0.0);
        assert_eq!(target, original);
    }
}
