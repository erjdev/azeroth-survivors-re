use super::poison_tests::fixture;
use super::*;
use crate::survivors::{earthbind, Owned, Selection};
use avian3d::prelude::*;
use benilla_world::collision::MoverTraceExclusions;
use bevy::ecs::system::RunSystemOnce;

pub(super) fn setup(supported: bool) -> (App, Entity, Entity) {
    let (mut app, target, _) = fixture("earthbind_totem", 1000.0);
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        PhysicsPlugins::new(PostUpdate),
    ));
    app.init_asset::<Mesh>();
    app.init_resource::<MoverTraceExclusions>();
    app.init_resource::<Selection>();
    if supported {
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(100., 2., 100.),
            Transform::from_xyz(0., -1., 0.),
        ));
    }
    let hero = app
        .world_mut()
        .spawn((Hero { vy: 0., air: 0. }, Transform::default()))
        .id();
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 5.;
    app.world_mut().entity_mut(target).insert((
        crate::creature_anim::MovementState::default(),
        crate::survivors::curse::CurseState::default(),
    ));
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[5];
        run.milestones = survivors_core::talents::Milestones {
            class: 5,
            ..Default::default()
        };
        run.bonus.targets = 5;
    }
    app.finish();
    app.cleanup();
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    (app, hero, target)
}

fn deployed(app: &mut App) -> Entity {
    app.world_mut().run_system_once(cast_abilities).unwrap();
    app.world_mut()
        .query_filtered::<Entity, With<earthbind::Earthbind>>()
        .single(app.world())
        .unwrap()
}

#[test]
fn earthbind_cast_requires_floor_and_replaces_only_its_previous_entity() {
    let (mut app, _, _) = setup(false);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(
        app.world_mut()
            .query::<&earthbind::Earthbind>()
            .iter(app.world())
            .count(),
        0
    );
    let (mut app, _, _) = setup(true);
    let old = deployed(&mut app);
    let position = app.world().get::<Transform>(old).unwrap().translation;
    assert!(position.length() < 0.01);
    let fire = app
        .world_mut()
        .spawn((
            Transform::default(),
            Totem {
                cast: 3606,
                damage: 9.,
                range: 20.,
                radius: 0.,
                left: 14.,
                tick: 0.3,
                pulse: false,
                source: "searing_totem",
            },
        ))
        .id();
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        assert_eq!(
            run.abilities[0].cd,
            Params::new(&run.abilities[0], &run.stats(), run.milestones).cooldown
        );
        run.abilities[0].cd = 0.;
        run.elapsed = 1.;
    }
    let new = deployed(&mut app);
    assert_ne!(new, old);
    assert!(app.world().get_entity(old).is_err());
    assert!(app.world().get::<Totem>(fire).is_some());
    assert_eq!(
        app.world()
            .get::<earthbind::Earthbind>(new)
            .unwrap()
            .life
            .health(),
        5.
    );
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
}

#[test]
fn earthbind_pulses_slow_without_damage_and_expire_after_leaving_radius() {
    let (mut app, _, target) = setup(true);
    let totem = deployed(&mut app);
    for now in [0., 2.9] {
        app.world_mut().resource_mut::<Run>().elapsed = now;
        app.world_mut().run_system_once(earthbind::tick).unwrap();
        assert_eq!(
            app.world().get::<Enemy>(target).unwrap().slows.state(now),
            (1., 0.)
        );
    }
    app.world_mut().resource_mut::<Run>().elapsed = 3.;
    app.world_mut().run_system_once(earthbind::tick).unwrap();
    let enemy = app.world().get::<Enemy>(target).unwrap();
    assert_eq!(enemy.slows.state(3.), (0.5, 5.));
    assert_eq!(enemy.hp, 1000.);
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 11.;
    app.world_mut().resource_mut::<Run>().elapsed = 6.;
    app.world_mut().run_system_once(earthbind::tick).unwrap();
    assert_eq!(
        app.world().get::<Enemy>(target).unwrap().slows.state(8.),
        (1., 0.)
    );
    app.world_mut().resource_mut::<Run>().elapsed = 60.;
    app.world_mut().run_system_once(earthbind::tick).unwrap();
    assert!(app.world().get::<earthbind::Earthbind>(totem).is_none());
}

#[test]
fn ordinary_enemy_destroys_earthbind_without_hurting_hero_or_triggering_reflection() {
    let (mut app, hero, target) = setup(true);
    let totem = deployed(&mut app);
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 3.;
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 0.5;
    app.world_mut().get_mut::<Enemy>(target).unwrap().damage = 100.;
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.thorns = 20.;
        run.thorns_t = 100.;
        run.thorns_spell = 467;
    }
    let hp = app.world().resource::<Run>().hp;
    app.world_mut()
        .run_system_once(enemies::move_enemies)
        .unwrap();
    assert_eq!(
        app.world()
            .get::<earthbind::Earthbind>(totem)
            .unwrap()
            .life
            .health(),
        0.
    );
    assert_eq!(app.world().resource::<Run>().hp, hp);
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    app.world_mut().run_system_once(earthbind::tick).unwrap();
    assert!(app.world().get::<earthbind::Earthbind>(totem).is_none());
    assert!(app
        .world()
        .get::<benilla_world::model_fade::DespawnFade>(totem)
        .is_some());
}

#[test]
fn earthbind_rank_growth_keeps_health_lifetime_and_slow_fixed() {
    let run = Run::default();
    let mut previous = 0.;
    for level in 1..=8 {
        let params = Params::new(
            &Owned {
                def: data::ability("earthbind_totem"),
                level,
                cd: 0.,
            },
            &run.stats(),
            run.milestones,
        );
        assert_eq!(params.duration, 45.);
        assert_eq!(params.dmg, 0.);
        assert!(params.radius >= previous);
        previous = params.radius;
    }
}

#[test]
fn earthbind_respects_boss_control_and_does_not_extend_stronger_slows() {
    use survivors_core::slows::Source;
    let (mut app, _, target) = setup(true);
    deployed(&mut app);
    {
        let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
        enemy.boss = true;
        enemy.slows.apply(Source::Spell("frostbolt"), 0.5, 4., 0.);
    }
    app.world_mut().resource_mut::<Run>().elapsed = 3.;
    app.world_mut().run_system_once(earthbind::tick).unwrap();
    let enemy = app.world().get::<Enemy>(target).unwrap();
    assert_eq!(enemy.slows.state(3.), (0.5, 1.));
    assert_eq!(enemy.slows.state(4.), (0.75, 4.));
    assert_eq!(enemy.slows.state(8.), (1., 0.));
}

#[test]
fn earthbind_is_offered_only_to_shamans() {
    for class in data::CLASSES {
        assert_eq!(
            class.ability_keys().contains(&"earthbind_totem"),
            class.id == 7
        );
    }
}
