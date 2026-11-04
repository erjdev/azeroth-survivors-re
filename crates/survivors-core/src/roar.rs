//! Demoralizing Roar's independent, melee-only reduction in active run time.

#[derive(Clone, Copy, Debug, PartialEq)]
struct Application {
    reduction: f32,
    applied: f32,
    expires: f32,
}

/// One solo-caster Roar per enemy. This never owns or replaces a Warlock curse.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Roar {
    active: Option<Application>,
}

impl Roar {
    /// Stronger applications replace; equal strength refreshes without shortening.
    /// Weaker applications cannot extend a stronger effect. Retain the application
    /// watermark after expiry to reject stale queued hits.
    pub fn apply(&mut self, reduction: f32, seconds: f32, now: f32) -> bool {
        let expires = now + seconds;
        if !reduction.is_finite()
            || !(0.0..=1.0).contains(&reduction)
            || reduction == 0.0
            || !now.is_finite()
            || now < 0.0
            || !seconds.is_finite()
            || seconds <= 0.0
            || !expires.is_finite()
            || expires <= now
        {
            return false;
        }
        let mut next = Application {
            reduction,
            applied: now,
            expires,
        };
        if let Some(old) = self.active {
            if now < old.applied {
                return false;
            }
            if now < old.expires {
                if reduction < old.reduction {
                    return false;
                }
                if reduction == old.reduction {
                    next.expires = next.expires.max(old.expires);
                }
            }
        }
        self.active = Some(next);
        true
    }

    /// Call only for a physical contact, before the recipient's mitigation.
    pub fn melee_multiplier(self, now: f32) -> f32 {
        match self.active {
            Some(active) if now.is_finite() && now >= active.applied && now < active.expires => {
                1.0 - active.reduction
            }
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strongest_refresh_owns_its_expiry_without_borrowing_weaker_duration() {
        let mut roar = Roar::default();
        assert!(roar.apply(0.2, 10., 1.));
        assert!(!roar.apply(0.1, 100., 2.));
        assert_eq!(roar.melee_multiplier(10.99), 0.8);
        assert_eq!(roar.melee_multiplier(11.), 1.);
        assert!(roar.apply(0.1, 5., 11.));
        assert_eq!(roar.melee_multiplier(12.), 0.9);
        assert!(roar.apply(0.3, 1., 12.));
        assert_eq!(roar.melee_multiplier(12.5), 0.7);
        assert_eq!(roar.melee_multiplier(13.), 1.);
        assert!(roar.apply(0.2, 10., 14.));
        assert!(roar.apply(0.2, 1., 15.));
        assert_eq!(roar.melee_multiplier(23.99), 0.8);
        assert!(roar.apply(0.2, 10., 20.));
        assert_eq!(roar.melee_multiplier(29.99), 0.8);
        assert_eq!(roar.melee_multiplier(30.), 1.);
    }

    #[test]
    fn invalid_and_stale_applications_cannot_change_a_target_or_revive_expired_roar() {
        let mut roar = Roar::default();
        assert!(roar.apply(0.2, 10., 10.));
        let valid = roar;
        for (reduction, seconds, now) in [
            (f32::NAN, 1., 11.),
            (f32::INFINITY, 1., 11.),
            (-0.1, 1., 11.),
            (1.1, 1., 11.),
            (0., 1., 11.),
            (0.5, 0., 11.),
            (0.5, -1., 11.),
            (0.5, f32::INFINITY, 11.),
            (0.5, f32::NAN, 11.),
            (0.5, 1., f32::NAN),
            (0.5, 1., f32::INFINITY),
            (0.5, 1., -1.),
            (0.5, 100., 9.),
        ] {
            assert!(!roar.apply(reduction, seconds, now));
            assert_eq!(roar, valid);
        }
        assert_eq!(roar.melee_multiplier(f32::NAN), 1.);
        assert_eq!(roar.melee_multiplier(9.), 1.);
        assert_eq!(roar.melee_multiplier(20.), 1.);
        assert!(!roar.apply(0.9, 100., 9.));
        assert_eq!(roar.melee_multiplier(21.), 1.);
        assert_eq!(Roar::default().melee_multiplier(11.), 1.);
    }
}

#[cfg(test)]
mod ability_tests {
    use crate::{
        data,
        talents::{forms::DruidForm, Milestones},
    };
    #[test]
    fn acquired_roar_is_non_damaging_bear_defense_without_a_starting_slot() {
        let roar = data::ability("demoralizing_roar");
        assert_eq!(roar.damage, 0.);
        assert_eq!(roar.dot, 0.);
        assert!(matches!(roar.effect, data::Effect::MeleeWeaken(n, t) if n == 0.2 && t == 10.));
        for class in 0..9 {
            for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
                let ranks: Vec<_> = form
                    .into_iter()
                    .map(|f| (f.tree(), f.talent(), 1))
                    .collect();
                let talents = Milestones {
                    class,
                    ..Default::default()
                }
                .with_ranks(&ranks);
                assert_eq!(
                    talents.ability_unlocked("demoralizing_roar"),
                    class == 8 && form == Some(DruidForm::Bear)
                );
                assert!(!talents
                    .starting_abilities(data::CLASSES[class].start)
                    .contains(&"demoralizing_roar"));
            }
        }
    }
}
