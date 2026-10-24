//! Survivors-specific milestone effects, independent of engine spell visuals and DBC ranks.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Milestones {
    pub class: usize,
    pub points: [u32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CastEffect {
    pub damage: f32,
    pub radius: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub extra_targets: usize,
    pub heal_fraction: f32,
    pub wrath_advance: f32,
    pub storm_fraction: f32,
}

impl Default for CastEffect {
    fn default() -> Self {
        Self {
            damage: 1.0,
            radius: 1.0,
            duration: 1.0,
            cooldown: 1.0,
            crit: 0.0,
            extra_targets: 0,
            heal_fraction: 0.0,
            wrath_advance: 0.0,
            storm_fraction: 0.0,
        }
    }
}

impl Milestones {
    fn paladin(self, tree: usize, threshold: u32) -> bool {
        self.class == 1 && self.points[tree] >= threshold
    }

    pub fn cast(self, key: &str) -> CastEffect {
        let mut effect = CastEffect::default();
        match key {
            "holy_shock" => {
                if self.paladin(0, 10) {
                    effect.crit = 0.20;
                }
                if self.paladin(0, 20) {
                    effect.heal_fraction = 0.03;
                }
                if self.paladin(0, 30) {
                    effect.cooldown = 0.80;
                }
            }
            "consecration" if self.paladin(1, 10) => {
                effect.radius = 1.25;
                effect.duration = 1.25;
            }
            "hammer_of_justice" if self.paladin(1, 30) => effect.extra_targets = 2,
            "judgement" => {
                if self.paladin(2, 10) {
                    effect.damage = 1.25;
                }
                if self.paladin(2, 20) {
                    effect.wrath_advance = 1.0;
                }
                if self.paladin(2, 30) {
                    effect.storm_fraction = 0.5;
                }
            }
            _ => {}
        }
        effect
    }

    /// Ardent Defender reduces the portion of a blow below 35% health, including a
    /// blow crossing that threshold. A large hit cannot skip the protection entirely.
    pub fn incoming(self, damage: f32, hp: f32, max_hp: f32) -> f32 {
        if !self.paladin(1, 20) {
            return damage;
        }
        let above = (hp - max_hp * 0.35).max(0.0).min(damage);
        above + (damage - above) * 0.8
    }
}

pub fn description(class: usize, tree: usize, threshold: u32) -> Option<&'static str> {
    if class != 1 {
        return None;
    }
    Some(match (tree, threshold) {
        (0, 10) => "Holy Shock: +20% critical chance",
        (0, 20) => "Holy Shock heals 3% maximum health once per cast",
        (0, 30) => "Holy Shock: 20% shorter cooldown",
        (1, 10) => "Consecration: +25% radius and duration",
        (1, 20) => "20% less damage below 35% health",
        (1, 30) => "Hammer of Justice: +2 targets",
        (2, 10) => "Judgement: +25% damage",
        (2, 20) => "Judgement advances Hammer of Wrath by 1 second",
        (2, 30) => "Judgement releases a 6-yard Divine Storm for 50% damage",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_paladin_milestone_opens_at_its_threshold() {
        for tree in 0..3 {
            for threshold in [10, 20, 30] {
                let mut before = Milestones {
                    class: 1,
                    points: [0; 3],
                };
                before.points[tree] = threshold - 1;
                let mut after = before;
                after.points[tree] = threshold;
                let changed = [
                    "holy_shock",
                    "consecration",
                    "hammer_of_justice",
                    "judgement",
                ]
                .iter()
                .any(|key| before.cast(key) != after.cast(key))
                    || before.incoming(100.0, 20.0, 100.0) != after.incoming(100.0, 20.0, 100.0);
                assert!(changed, "tree {tree}, threshold {threshold}");
                assert!(description(1, tree, threshold).is_some());
            }
        }
    }

    #[test]
    fn effects_do_not_leak_to_other_classes_or_spells() {
        for class in [0, 2, 3, 4, 5, 6, 7, 8] {
            let m = Milestones {
                class,
                points: [51; 3],
            };
            for key in [
                "holy_shock",
                "consecration",
                "hammer_of_justice",
                "judgement",
            ] {
                assert_eq!(m.cast(key), CastEffect::default());
            }
            assert_eq!(m.incoming(100.0, 20.0, 100.0), 100.0);
        }
        let m = Milestones {
            class: 1,
            points: [51; 3],
        };
        assert_eq!(m.cast("fireball"), CastEffect::default());
        assert_eq!(m.cast("consecration").extra_targets, 0);
    }

    #[test]
    fn ardent_defender_splits_threshold_crossing_hits() {
        let m = Milestones {
            class: 1,
            points: [0, 20, 0],
        };
        assert_eq!(m.incoming(10.0, 100.0, 100.0), 10.0);
        assert_eq!(m.incoming(20.0, 45.0, 100.0), 18.0);
        assert_eq!(m.incoming(20.0, 20.0, 100.0), 16.0);
    }
}
