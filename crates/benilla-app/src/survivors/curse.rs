//! Curse of Exhaustion's clock is independent of unrelated spell and channel slows.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CurseSlow {
    speed: f32,
    applied: f32,
    expires: f32,
}

impl CurseSlow {
    pub(super) fn apply(&mut self, speed: f32, seconds: f32, now: f32) {
        let expires = now + seconds;
        if !speed.is_finite()
            || !(0.0..=1.0).contains(&speed)
            || !now.is_finite()
            || now < 0.0
            || now < self.applied
            || !expires.is_finite()
            || expires <= now
        {
            return;
        }
        *self = Self {
            speed,
            applied: now,
            expires,
        };
    }

    pub(super) fn movement(self, ordinary: f32, now: f32) -> f32 {
        if now.is_finite() && now >= self.applied && now < self.expires {
            ordinary.min(self.speed)
        } else {
            ordinary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivors::{
        abilities::{
            poison_tests::{advance, fixture},
            Hit,
        },
        enemies::Enemy,
        Run,
    };

    #[test]
    fn curse_does_not_prolong_a_stronger_short_spell_slow() {
        let (mut app, target, mut schedule) = fixture("curse_of_agony", 1000.);
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 7,
            ..Default::default()
        }
        .with_ranks(&[(0, 1081, 1), (0, 1082, 4)]);
        {
            let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
            enemy.slow = 0.5;
            enemy.slow_t = 2.5;
        }
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "curse_of_agony",
            dot: 3.5,
            dot_dur: 12.,
            ..Hit::direct(target, 0., false)
        });
        advance(&mut app, &mut schedule, 0.);
        let enemy = app.world().get::<Enemy>(target).unwrap();
        assert_eq!(
            (enemy.slow, enemy.slow_t),
            (0.5, 2.5),
            "curse must not extend Frostbolt's clock"
        );
        let curse = app.world().get::<CurseSlow>(target).unwrap();
        assert_eq!(curse.movement(0.5, 0.), 0.5);
        assert!((curse.movement(1., 2.5) - 0.7).abs() < 0.0001);
        assert_eq!(curse.movement(1., 12.), 1.);
    }

    #[test]
    fn delayed_applications_do_not_restart_expired_clocks_and_death_removes_state() {
        let (mut app, target, mut schedule) = fixture("curse_of_agony", 1000.);
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 7,
            ..Default::default()
        }
        .with_ranks(&[(0, 1081, 1)]);
        app.world_mut().resource_mut::<Run>().elapsed = 20.;
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "curse_of_agony",
            occurred: Some(1.),
            dot: 3.5,
            dot_dur: 12.,
            ..Hit::direct(target, 0., false)
        });
        advance(&mut app, &mut schedule, 0.);
        assert_eq!(
            app.world()
                .get::<CurseSlow>(target)
                .unwrap()
                .movement(1., 20.),
            1.
        );
        app.world_mut().resource_mut::<Messages<Hit>>().clear();
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "curse_of_agony",
            dot: 3.5,
            dot_dur: 12.,
            ..Hit::direct(target, 0., false)
        });
        advance(&mut app, &mut schedule, 0.);
        assert_eq!(
            app.world()
                .get::<CurseSlow>(target)
                .unwrap()
                .movement(1., 20.),
            0.9
        );
        app.world_mut().resource_mut::<Messages<Hit>>().clear();
        app.world_mut()
            .resource_mut::<Messages<Hit>>()
            .write(Hit::direct(target, 10000., false));
        advance(&mut app, &mut schedule, 0.);
        assert_eq!(
            app.world()
                .get::<CurseSlow>(target)
                .unwrap()
                .movement(1., 20.),
            1.
        );
        assert_eq!(app.world().resource::<Run>().kills, 1);
    }

    #[test]
    fn refresh_pause_expiry_and_invalid_clocks_keep_independent_potency() {
        let mut curse = CurseSlow::default();
        curse.apply(0.7, 12., 1.);
        let other = CurseSlow::default();
        assert_eq!(other.movement(1., 1.), 1.);
        assert_eq!(curse.movement(0.5, 1.), 0.5);
        assert_eq!(curse.movement(1., 12.999), 0.7);
        assert_eq!(curse.movement(1., 13.), 1.);
        curse.apply(0.9, 12., 10.);
        assert_eq!(
            curse.movement(1., 10.),
            0.9,
            "reapplication replaces its own potency"
        );
        for _ in 0..10 {
            assert_eq!(curse.movement(1., 10.), 0.9);
        }
        let saved = curse;
        for (speed, secs, now) in [
            (f32::NAN, 12., 11.),
            (0.7, -1., 11.),
            (0.7, 12., 9.),
            (0.7, 12., f32::NAN),
            (0.7, f32::INFINITY, 11.),
        ] {
            curse.apply(speed, secs, now);
            assert_eq!(curse, saved);
        }
        assert_eq!(curse.movement(1., 22.), 1.);
        assert_eq!(curse.movement(1., f32::NAN), 1.);
    }
}
