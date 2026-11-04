//! Hemorrhage rank-one vulnerability: +3 physical damage, 30 charges, 15 active seconds.
//! Survivors settles the flat bonus after critical damage, once per positive damage event.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hemorrhage {
    applied: f32,
    expires: f32,
    charges: u8,
}
impl Hemorrhage {
    pub fn apply(&mut self, now: f32) -> bool {
        if !now.is_finite() || now < 0.0 || now + 15.0 <= now {
            return false;
        }
        *self = Self {
            applied: now,
            expires: now + 15.0,
            charges: 30,
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
    pub fn bonus(&mut self, physical: bool, damage: f32, now: f32) -> f32 {
        if !physical || !damage.is_finite() || damage <= 0.0 || self.charges(now) == 0 {
            return 0.0;
        }
        self.charges -= 1;
        3.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charges_expire_refresh_and_ignore_nonphysical_or_invalid_events() {
        let mut first = Hemorrhage::default();
        let other = Hemorrhage::default();
        assert!(first.apply(2.));
        for amount in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(first.bonus(true, amount, 2.), 0.);
        }
        assert_eq!(first.bonus(false, 100., 2.), 0.);
        assert_eq!(first.bonus(true, 100., 1.), 0.);
        for _ in 0..30 {
            assert_eq!(first.bonus(true, 100., 2.), 3.);
        }
        assert_eq!(first.bonus(true, 100., 2.), 0.);
        assert_eq!(other.charges(2.), 0);
        assert!(first.apply(3.));
        assert_eq!(first.charges(3.), 30);
        assert_eq!(first.charges(18.), 0);
        assert_eq!(first.bonus(true, 1., 18.), 0.);
        for now in [-1., f32::NAN, f32::INFINITY, f32::MAX] {
            assert!(!first.apply(now));
        }
    }
}
