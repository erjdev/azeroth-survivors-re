//! Manual mobility system acceptance without unattended combat.
use super::*;
use crate::survivors::{mobility, Phase};
use bevy::ecs::system::RunSystemOnce;
use survivors_core::slows::Source;
use survivors_core::talents::Milestones;

fn setup(rank: u8) -> App {
    let (mut app, _, _) = super::earthbind_tests::setup(true);
    app.insert_resource(State::new(Phase::Playing));
    app.init_resource::<crate::loading_screen::LoadingScreen>();
    app.init_resource::<hero::AimPoint>();
    let mut keys = ButtonInput::default();
    keys.press(mobility::KEY);
    app.insert_resource(keys);
    app.insert_resource(Time::<()>::default());
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[3];
    run.level = mobility::LEARN_LEVEL;
    run.elapsed = 4.;
    run.milestones = Milestones {
        class: 3,
        ..Default::default()
    }
    .with_ranks(&[(1, 222, rank)]);
    run.incoming_slows
        .apply(Source::Spell("enemy_frostbolt"), 0.5, 10., 1.);
    run.incoming_slows
        .apply(Source::Secondary("root"), 0., 10., 2.);
    app
}

#[test]
fn improved_sprint_cleanses_all_current_impairments_only_after_successful_activation() {
    for rank in [0, 2] {
        let mut app = setup(rank);
        app.world_mut()
            .run_system_once(mobility::cast_mobility)
            .unwrap();
        let mut run = app.world_mut().resource_mut::<Run>();
        assert_eq!(
            run.incoming_slows.state(4.).0,
            if rank == 2 { 1. } else { 0. }
        );
        assert_eq!(
            (run.mobile.cd, run.mobile.haste, run.mobile.haste_t),
            (15., 1.8, 4.)
        );
        if rank == 2 {
            assert!(!run
                .incoming_slows
                .apply(Source::Spell("delayed"), 0.2, 20., 3.));
            assert!(run.incoming_slows.apply(Source::Spell("new"), 0.4, 5., 4.));
            assert_eq!(run.incoming_slows.state(4.), (0.4, 5.));
        }
    }
}

#[test]
fn improved_sprint_never_cleanses_on_blocked_input_or_another_class_mobility() {
    for case in 0..7 {
        let mut app = setup(2);
        match case {
            0 => app.world_mut().resource_mut::<Run>().mobile.cd = 3.,
            1 => app.world_mut().resource_mut::<Run>().level = 4,
            2 => app.world_mut().resource_mut::<Run>().hp = 0.,
            3 => {
                app.insert_resource(State::new(Phase::Paused));
            }
            4 => {
                app.insert_resource(crate::loading_screen::LoadingScreen::test_covering());
            }
            5 => app
                .world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear(),
            _ => app.world_mut().resource_mut::<Run>().class = &data::CLASSES[8],
        }
        app.world_mut()
            .run_system_once(mobility::cast_mobility)
            .unwrap();
        let run = app.world().resource::<Run>();
        assert_eq!(run.incoming_slows.state(4.).0, 0., "case {case}");
        assert_eq!(
            run.mobile.haste_t,
            if case == 6 { 4. } else { 0. },
            "case {case}"
        );
    }
}
