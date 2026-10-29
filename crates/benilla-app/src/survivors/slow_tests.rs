//! Slow overlaps use actual hit settlement, without unattended gameplay.
use super::poison_tests::{advance, fixture};
use super::*;

#[test]
fn weaker_long_slow_does_not_extend_a_stronger_short_slow() {
    for reverse in [false, true] {
        let (mut app, target, mut schedule) = fixture("frostbolt", 1000.);
        let mut hits = [("frostbolt", 0.5, 2.5), ("thunder_clap", 0.8, 10.)];
        if reverse {
            hits.reverse();
        }
        for (source, speed, duration) in hits {
            app.world_mut().write_message(Hit {
                source,
                effect: Effect::Slow(speed, duration),
                ..Hit::direct(target, 0., false)
            });
        }
        advance(&mut app, &mut schedule, 0.);
        for (now, expected) in [(0., (0.5, 2.5)), (2.5, (0.8, 7.5)), (10., (1., 0.))] {
            app.world_mut().resource_mut::<Run>().elapsed = now;
            advance(&mut app, &mut schedule, 0.);
            assert_eq!(
                app.world().get::<Enemy>(target).unwrap().slows.state(now),
                expected
            );
        }
    }
}

#[test]
fn delayed_older_hit_cannot_restore_a_replaced_slow() {
    let (mut app, target, mut schedule) = fixture("frostbolt", 1000.);
    app.world_mut().resource_mut::<Run>().elapsed = 5.;
    for (occurred, speed, duration) in [(4., 0.5, 2.), (3., 0.2, 20.)] {
        app.world_mut().write_message(Hit {
            source: "frostbolt",
            effect: Effect::Slow(speed, duration),
            occurred: Some(occurred),
            ..Hit::direct(target, 0., false)
        });
        advance(&mut app, &mut schedule, 0.);
    }
    let enemy = app.world().get::<Enemy>(target).unwrap();
    assert_eq!(enemy.slows.state(5.), (0.5, 1.));
    assert_eq!(enemy.slows.state(6.), (1., 0.));
}
