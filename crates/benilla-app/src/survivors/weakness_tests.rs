//! Curse replacement and potency use the real cast and hit settlement paths.
use super::poison_tests::{advance, fixture};
use super::*;
use crate::survivors::{curse::CurseState, Owned};
use survivors_core::talents::Milestones;

fn training(rank: u8) -> Milestones {
    Milestones {
        class: 7,
        ..Default::default()
    }
    .with_ranks(&[(0, 1006, rank), (0, 1081, 1), (0, 1082, 4)])
}

#[test]
fn weakness_cast_scales_rank_training_and_its_own_legendary_without_damage_or_crit() {
    let mut run = Run::default();
    run.bonus.might = 4.0;
    run.bonus.crit = 1.0;
    run.bonus.targets = 2;
    let plain = run.stats();
    for level in [1, 3, 8] {
        let owned = Owned {
            def: data::ability("curse_of_weakness"),
            level,
            cd: 0.,
        };
        for rank in 0..=3 {
            let cast = Params::new(&owned, &plain, training(rank));
            let bonus = [0., 0.06, 0.13, 0.20][usize::from(rank)];
            let expected = 3.0 * owned.damage_mult() * (1.0 + bonus);
            assert!(
                matches!(cast.effect,Effect::Weakness(n,t) if (n-expected).abs()<0.0001 && (t-owned.duration()).abs()<0.0001)
            );
            assert_eq!(
                (cast.dmg, cast.dot, cast.crit, cast.radius),
                (0., 0., 0., 0.)
            );
            assert_eq!(cast.count, owned.count(&plain) as usize);
            assert!(cast.count >= 3);
            let mut legendary = plain;
            legendary.legendary = Some((
                "curse_of_weakness",
                crate::survivors::campaign::Bonus {
                    might: 0.5,
                    ..Default::default()
                },
            ));
            let boosted = Params::new(&owned, &legendary, training(rank));
            assert!(
                matches!(boosted.effect,Effect::Weakness(n,_) if (n-expected*1.5).abs()<0.0001)
            );
            legendary.legendary = Some((
                "shadow_bolt",
                crate::survivors::campaign::Bonus {
                    might: 0.5,
                    ..Default::default()
                },
            ));
            assert_eq!(
                Params::new(&owned, &legendary, training(rank)).effect,
                cast.effect
            );
        }
    }
}

fn settle(app: &mut App, schedule: &mut Schedule, hit: Hit) {
    app.world_mut().resource_mut::<Messages<Hit>>().write(hit);
    advance(app, schedule, 0.);
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
}

#[test]
fn curses_replace_both_ways_refresh_without_stacking_and_preserve_other_effects() {
    let (mut app, target, mut schedule) = fixture("curse_of_agony", 1000.);
    app.world_mut().resource_mut::<Run>().milestones = training(3);
    {
        let mut e = app.world_mut().get_mut::<Enemy>(target).unwrap();
        e.slows.apply(
            survivors_core::slows::Source::Spell("fixture"),
            0.5,
            2.5,
            0.,
        );
        e.root_t = 2.;
        e.dots.push(enemies::Dot {
            dps: 1.,
            left: 10.,
            tick: 1.,
            source: "corruption",
        });
    }
    let agony = Hit {
        source: "curse_of_agony",
        dot: 3.5,
        dot_dur: 12.,
        mark: 980,
        mark_dur: 12.,
        ..Hit::direct(target, 0., false)
    };
    settle(&mut app, &mut schedule, agony.clone());
    settle(&mut app, &mut schedule, agony.clone());
    assert_eq!(app.world().get::<Enemy>(target).unwrap().dots.len(), 2);
    assert!(
        (app.world()
            .get::<CurseState>(target)
            .unwrap()
            .movement(1., 0.)
            - 0.7)
            .abs()
            < 0.0001
    );
    let weakness = Hit {
        source: "curse_of_weakness",
        effect: Effect::Weakness(3.6, 12.),
        mark: 702,
        mark_dur: 12.,
        ..Hit::direct(target, 0., false)
    };
    settle(&mut app, &mut schedule, weakness.clone());
    settle(&mut app, &mut schedule, weakness);
    let e = app.world().get::<Enemy>(target).unwrap();
    assert_eq!(
        (e.hp, e.slows.state(0.).0, e.slows.state(0.).1, e.root_t),
        (1000., 0.5, 2.5, 2.)
    );
    assert_eq!(e.dots.len(), 1);
    assert_eq!(e.dots[0].source, "corruption");
    let curse = app.world().get::<CurseState>(target).unwrap();
    assert_eq!(curse.movement(1., 0.), 1.);
    assert!((curse.outgoing(10., true, 0.) - 6.4).abs() < 0.0001);
    assert_eq!(curse.outgoing(10., false, 0.), 10.);
    assert_eq!(curse.outgoing(2., true, 0.), 0.);
    assert_eq!(curse.outgoing(10., true, 12.), 10.);
    settle(&mut app, &mut schedule, agony);
    let curse = app.world().get::<CurseState>(target).unwrap();
    assert_eq!(curse.outgoing(10., true, 0.), 10.);
    assert!((curse.movement(1., 0.) - 0.7).abs() < 0.0001);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().dots.len(), 2);
}

#[test]
fn weakness_invalid_and_old_applications_do_not_replace_live_curse() {
    let mut curse = CurseState::default();
    assert!(curse.weakness(3., 12., 5.));
    let live = curse;
    for (amount, seconds, now) in [
        (f32::NAN, 12., 6.),
        (-1., 12., 6.),
        (3., 0., 6.),
        (3., 12., 4.),
        (3., f32::INFINITY, 6.),
    ] {
        assert!(!curse.weakness(amount, seconds, now));
        assert_eq!(curse, live);
    }
    assert_eq!(curse.outgoing(10., true, 4.), 10.);
    assert_eq!(curse.outgoing(10., true, f32::NAN), 10.);
    assert!(curse.weakness(1., 12., 6.));
    assert_eq!(curse.outgoing(10., true, 6.), 9.);
}
