//! Ghostly Strike's refreshing seven-second, fifteen-percentage-point melee dodge window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GhostlyStrike {
    applied: f32,
    expires: f32,
}
impl GhostlyStrike {
    pub fn apply(&mut self, now: f32) -> bool {
        if !now.is_finite() || now < 0.0 || now + 7.0 <= now {
            return false;
        }
        self.applied = now;
        self.expires = now + 7.0;
        true
    }
    pub fn remaining(self, now: f32) -> f32 {
        if now.is_finite() && now >= self.applied {
            (self.expires - now).max(0.0)
        } else {
            0.0
        }
    }
    pub fn dodge(self, now: f32) -> f32 {
        if self.remaining(now) > 0.0 {
            0.15
        } else {
            0.0
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_time_window_refreshes_without_stacking_or_replaying() {
        let mut buff = GhostlyStrike::default();
        assert_eq!(buff.dodge(0.), 0.);
        assert!(buff.apply(2.));
        assert_eq!(buff.dodge(2.), 0.15);
        assert_eq!(buff.dodge(1.), 0.);
        assert_eq!(buff.remaining(5.), 4.);
        assert!(buff.apply(5.));
        assert_eq!(buff.dodge(9.), 0.15);
        assert_eq!(buff.dodge(12.), 0.);
        let before = buff;
        for invalid in [f32::NAN, f32::INFINITY, f32::MAX, -1.] {
            assert!(!buff.apply(invalid));
            assert_eq!(buff, before);
        }
    }
}
