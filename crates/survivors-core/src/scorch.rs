//! Spell 22959: target-owned Fire vulnerability, 3% per stack for 30 active seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireVulnerability {
    stacks: u8,
    applied: f32,
    expires: f32,
}

pub fn chance_percent(rank: u8) -> u8 {
    [0, 33, 66, 100][usize::from(rank.min(3))]
}

impl FireVulnerability {
    pub fn stacks(self, now: f32) -> u8 {
        if now.is_finite() && now >= self.applied && now < self.expires {
            self.stacks
        } else {
            0
        }
    }

    pub fn apply(&mut self, now: f32) {
        if !now.is_finite() || now < 0.0 || now < self.applied || now + 30.0 <= now {
            return;
        }
        *self = Self {
            stacks: self.stacks(now).saturating_add(1).min(5),
            applied: now,
            expires: now + 30.0,
        };
    }

    pub fn multiplier(self, fire: bool, now: f32) -> f32 {
        if fire {
            1.0 + 0.03 * f32::from(self.stacks(now))
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_refresh_cap_and_expire_on_active_time_only() {
        let mut first = FireVulnerability::default();
        let other = first;
        for at in 0..8 {
            first.apply(at as f32);
            assert_eq!(first.stacks(at as f32), (at + 1).min(5));
        }
        assert_eq!(first.multiplier(true, 36.99), 1.15);
        assert_eq!(first.multiplier(false, 20.), 1.);
        assert_eq!(other.multiplier(true, 20.), 1.);
        for _ in 0..10 {
            assert_eq!(first.stacks(20.), 5);
        }
        assert_eq!(first.stacks(37.), 0);
        first.apply(37.);
        assert_eq!(first.stacks(37.), 1);
        let before = first;
        for at in [f32::NAN, f32::INFINITY, -1., 36., f32::MAX] {
            first.apply(at);
            assert_eq!(first, before);
        }
        assert_eq!(first.stacks(f32::NAN), 0);
    }
}

#[cfg(test)]
mod talent_tests {
    use crate::talents::{talent_definition, Milestones};
    #[test]
    fn exact_rank_chances_require_mage_original_positive_scorch_damage() {
        for class in 0..9 {
            for rank in [0, 1, 2, 3, 255] {
                let talents = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&[(1, 25, rank)]);
                let expected = if class == 6 {
                    super::chance_percent(rank) as f32 / 100.
                } else {
                    0.
                };
                assert_eq!(
                    talents.improved_scorch_chance("scorch", 10., true),
                    expected
                );
                assert_eq!(talents.improved_scorch_chance("scorch", 10., false), 0.);
                for damage in [0., -1., f32::NAN, f32::INFINITY] {
                    assert_eq!(talents.improved_scorch_chance("scorch", damage, true), 0.);
                }
                for source in ["fireball", "Ignite", "flamestrike", "frostbolt"] {
                    assert_eq!(talents.improved_scorch_chance(source, 10., true), 0.);
                }
            }
        }
        let node = talent_definition(6, 25).unwrap();
        for (rank, text) in [
            (0, "0 » 33% chance"),
            (1, "33 » 66% chance"),
            (2, "66 » 100% chance"),
            (3, "100% chance"),
        ] {
            let (description, spans) = node.inline_description_with_values(rank).unwrap();
            assert_eq!(&description[spans[0].clone()], text);
            assert_eq!(description.contains(" » "), rank < 3);
        }
        let mage = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(1, 35, 5)]);
        assert!(mage.cast("scorch").damage > 1.0);
        assert_eq!(mage.cast("scorch").damage, mage.cast("fireball").damage);
    }
}
