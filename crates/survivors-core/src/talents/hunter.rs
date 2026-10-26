//! Authored automatic Beast Mastery milestones for the Survivors combat wolf.
use super::Milestones;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PetBonus {
    pub damage: f32,
    pub cooldown: f32,
}

impl Default for PetBonus {
    fn default() -> Self {
        Self {
            damage: 1.0,
            cooldown: 1.0,
        }
    }
}

impl Milestones {
    /// Run time drives Wrath, so pause and selection screens cannot consume the buff.
    pub fn hunter_pet(self, elapsed: f32) -> PetBonus {
        let mut bonus = PetBonus::default();
        if self.class != 2 {
            return bonus;
        }
        if self.points[0] >= 10 {
            bonus.damage *= 1.2;
        }
        if self.points[0] >= 20 {
            bonus.cooldown *= 0.8;
        }
        if self.points[0] >= 30 && elapsed.is_finite() && elapsed >= 0.0 && elapsed % 30.0 < 10.0 {
            bonus.damage *= 1.5;
        }
        bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thresholds_and_periodic_wrath_are_pet_only() {
        let at = |points| Milestones {
            class: 2,
            points: [points, 0, 0],
            ..Default::default()
        };
        assert_eq!(at(9).hunter_pet(0.0), PetBonus::default());
        assert_eq!(at(10).hunter_pet(0.0).damage, 1.2);
        assert_eq!(at(19).hunter_pet(0.0).cooldown, 1.0);
        assert_eq!(at(20).hunter_pet(0.0).cooldown, 0.8);
        assert_eq!(at(29).hunter_pet(0.0).damage, 1.2);
        for elapsed in [0.0, 9.999, 30.0, 39.999, 60.0] {
            assert!((at(30).hunter_pet(elapsed).damage - 1.8).abs() < 0.00001);
        }
        for elapsed in [10.0, 29.999, 40.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(at(30).hunter_pet(elapsed).damage, 1.2);
        }
        for class in 0..9 {
            if class == 2 {
                continue;
            }
            assert_eq!(
                Milestones {
                    class,
                    points: [51; 3],
                    ..Default::default()
                }
                .hunter_pet(0.0),
                PetBonus::default()
            );
        }
        for tree in [1, 2] {
            let mut other = at(0);
            other.points[tree] = 51;
            assert_eq!(other.hunter_pet(0.0), PetBonus::default());
        }
        for key in ["arcane_shot", "multi_shot", "volley", "explosive_trap"] {
            assert_eq!(at(51).cast(key), super::super::CastEffect::default());
        }
        for threshold in [10, 20, 30] {
            assert!(super::super::description(2, 0, threshold).is_some());
        }
    }
}
