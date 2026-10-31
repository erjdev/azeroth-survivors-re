//! Shared combat-pet health. Species own attacks, recovery policy and respawning.
#[derive(Clone, Copy, Debug, Default)]
pub struct Vitality {
    pub hp: f32,
    pub max_hp: f32,
    pub revive_at: Option<f32>,
    pub recovery: f32,
}

impl Vitality {
    pub fn new(max_hp: f32, recovery: f32) -> Self {
        assert!(max_hp.is_finite() && max_hp > 0.0);
        assert!(recovery.is_finite() && recovery >= 0.0);
        Self {
            hp: max_hp,
            max_hp,
            recovery,
            revive_at: None,
        }
    }

    /// Returns actual health removed; repeated lethal hits do not postpone recovery.
    pub fn hurt(&mut self, damage: f32, now: f32) -> f32 {
        if self.hp <= 0.0 || !damage.is_finite() || damage <= 0.0 || !now.is_finite() {
            return 0.0;
        }
        let removed = damage.min(self.hp);
        self.hp -= removed;
        if self.hp <= 0.0 {
            self.revive_at = Some(now + self.recovery);
        }
        removed
    }

    /// Healing never resurrects a defeated pet.
    pub fn heal(&mut self, amount: f32) -> f32 {
        if self.hp <= 0.0 || !amount.is_finite() || amount <= 0.0 {
            return 0.0;
        }
        let restored = amount.min((self.max_hp - self.hp).max(0.0));
        self.hp += restored;
        restored
    }

    /// Level and talent changes preserve the current health fraction.
    pub fn resize(&mut self, maximum: f32) {
        if !maximum.is_finite() || maximum <= 0.0 || self.max_hp <= 0.0 || maximum == self.max_hp {
            return;
        }
        self.hp = (self.hp / self.max_hp * maximum).clamp(0.0, maximum);
        self.max_hp = maximum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defeat_is_once_and_healing_cannot_restart_a_pet() {
        let mut pet = Vitality::new(60., 10.);
        assert_eq!(pet.hurt(20., 1.), 20.);
        assert_eq!(pet.hurt(100., 3.), 40.);
        assert_eq!(pet.revive_at, Some(13.));
        assert_eq!(pet.hurt(100., 9.), 0.);
        assert_eq!(pet.heal(500.), 0.);
        pet.resize(90.);
        assert_eq!(pet.hp, 0.);
        assert_eq!(pet.revive_at, Some(13.));
    }

    #[test]
    fn growth_and_healing_preserve_fraction_and_cap() {
        let mut pet = Vitality::new(60., 10.);
        pet.hurt(30., 0.);
        pet.resize(66.);
        assert_eq!(pet.hp, 33.);
        assert_eq!(pet.heal(100.), 33.);
        pet.resize(30.);
        assert_eq!(pet.hp, 30.);
        for invalid in [f32::NAN, f32::INFINITY, -1., 0.] {
            assert_eq!(pet.hurt(invalid, 0.), 0.);
            assert_eq!(pet.heal(invalid), 0.);
            pet.resize(invalid);
            assert_eq!(pet.hp, 30.);
            assert_eq!(pet.max_hp, 30.);
        }
    }
}
