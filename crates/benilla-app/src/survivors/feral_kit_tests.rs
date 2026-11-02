//! Cast and settlement coverage for permanent-form starting abilities.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::{forms::DruidForm, Milestones};

fn setup(form: Option<DruidForm>, key: &'static str) -> (App, Entity) {
    let (mut app, _, target) = super::earthbind_tests::setup(true);
    let ranks: Vec<_> = form
        .into_iter()
        .map(|f| (f.tree(), f.talent(), 1))
        .collect();
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[8];
        run.milestones = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&ranks);
        run.abilities = vec![super::super::Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.0,
        }];
        run.weapon_attack_cd = 100.0;
        run.bonus.targets = 0;
    }
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 3.0;
    (app, target)
}

#[test]
fn forms_gate_actual_casts_and_rake_settles_a_separate_noncritical_bleed() {
    for form in [
        None,
        Some(DruidForm::Cat),
        Some(DruidForm::Bear),
        Some(DruidForm::Moonkin),
        Some(DruidForm::TreeOfLife),
    ] {
        for (key, required) in [("rake", DruidForm::Cat), ("swipe", DruidForm::Bear)] {
            let (mut app, target) = setup(form, key);
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let hits: Vec<_> = app
                .world()
                .resource::<Messages<Hit>>()
                .iter_current_update_messages()
                .cloned()
                .collect();
            assert_eq!(hits.len(), usize::from(form == Some(required)));
            if hits.is_empty() {
                continue;
            }
            assert_eq!(hits[0].source, key);
            assert_eq!(hits[0].origin, HitOrigin::Hero);
            assert_eq!(hits[0].color, COLOR_MELEE);
            let expected_damage = hits[0].damage;
            let expected_dot = hits[0].dot;
            let before = app.world().get::<Enemy>(target).unwrap().hp;
            app.world_mut().run_system_once(apply_hits).unwrap();
            let enemy = app.world().get::<Enemy>(target).unwrap();
            assert!((enemy.hp - (before - expected_damage)).abs() < 0.001);
            if key == "rake" {
                assert_eq!(enemy.dots.len(), 1);
                assert_eq!(enemy.dots[0].source, "rake");
                assert_eq!(enemy.dots[0].origin, HitOrigin::Hero);
                assert!((enemy.dots[0].dps - expected_dot).abs() < 0.001);
            } else {
                assert!(enemy.dots.is_empty());
            }
        }
    }
}

#[test]
fn swipe_uses_area_radius_and_never_duplicates_hits_for_extra_targets() {
    for targets in [0, 10] {
        let (mut app, target) = setup(Some(DruidForm::Bear), "swipe");
        app.world_mut().resource_mut::<Run>().bonus.targets = targets;
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * 8.0;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.bonus.area = 1.0;
            // Isolate radius eligibility from the empty-target retry timer.
            run.abilities[0].cd = 0.0;
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(app.world().resource::<Messages<Hit>>().len(), 1);
    }
}
