//! Exercise real targeting and hit settlement without unattended gameplay.
use super::poison_tests::{advance, fixture};
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::creature::CreatureType;
use survivors_core::talents::Milestones;

fn setup() -> (App, Entity, Schedule) {
    let (mut app, target, schedule) = fixture("repentance", 1000.0);
    let humanoid = data::roster(0)
        .enemies
        .iter()
        .find(|e| e.creature_type == CreatureType::Humanoid)
        .unwrap();
    app.world_mut().get_mut::<Enemy>(target).unwrap().def = humanoid;
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[1];
    run.milestones = Milestones {
        class: 1,
        ..Default::default()
    }
    .with_ranks(&[(2, 1441, 1)]);
    (app, target, schedule)
}

fn apply(app: &mut App, target: Entity, schedule: &mut Schedule) {
    let run = app.world().resource::<Run>();
    let owned = &run.abilities[0];
    let p = Params::new(owned, &run.stats(), run.milestones);
    let hit = hit_for(owned.def, &p, target, 0.0, false);
    app.world_mut().write_message(hit);
    advance(app, schedule, 0.0);
}

#[test]
fn repentance_damage_breaks_control_without_clearing_stuns_or_roots() {
    for origin in [HitOrigin::Hero, HitOrigin::CombatPet] {
        let (mut app, target, mut schedule) = setup();
        apply(&mut app, target, &mut schedule);
        assert_eq!(
            app.world().get::<Enemy>(target).unwrap().incapacitate_t,
            6.0
        );
        assert!(app
            .world()
            .get::<ObjectStore>(target)
            .unwrap()
            .0
            .unit_aura_ids()
            .any(|id| id == 20066));
        for (damage, expected) in [(0.0, 6.0), (1.0, 0.0)] {
            {
                let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
                enemy.root_t = 9.0;
                enemy.stun_t = 8.0;
            }
            app.world_mut().write_message(Hit {
                origin,
                source: "test",
                ..Hit::direct(target, damage, false)
            });
            advance(&mut app, &mut schedule, 0.0);
            let enemy = app.world().get::<Enemy>(target).unwrap();
            assert_eq!(
                (enemy.incapacitate_t, enemy.root_t, enemy.stun_t),
                (expected, 9.0, 8.0)
            );
            assert_eq!(
                app.world()
                    .get::<ObjectStore>(target)
                    .unwrap()
                    .0
                    .unit_aura_ids()
                    .any(|id| id == 20066),
                expected > 0.0
            );
        }
    }
}

#[test]
fn repentance_periodic_damage_and_poison_wake_targets() {
    for poison in [false, true] {
        let (mut app, target, mut schedule) = setup();
        apply(&mut app, target, &mut schedule);
        let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
        if poison {
            enemy.poison.apply(2.0);
        } else {
            enemy.dots.push(enemies::Dot {
                origin: HitOrigin::Hero,
                dps: 2.0,
                left: 6.0,
                tick: 1.0,
                source: "test",
            });
        }
        advance(&mut app, &mut schedule, 3.0);
        let enemy = app.world().get::<Enemy>(target).unwrap();
        assert!(enemy.hp < 1000.0);
        assert_eq!(enemy.incapacitate_t, 0.0);
    }
}

#[test]
fn repentance_rejects_bosses_nonhumanoids_and_corpses_at_impact() {
    for case in 0..3 {
        let (mut app, target, mut schedule) = setup();
        let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
        match case {
            0 => enemy.boss = true,
            1 => {
                enemy.def = data::roster(0)
                    .enemies
                    .iter()
                    .find(|e| e.creature_type != CreatureType::Humanoid)
                    .unwrap()
            }
            _ => {
                enemy.dead = true;
                enemy.hp = 0.0;
            }
        }
        apply(&mut app, target, &mut schedule);
        assert_eq!(
            app.world().get::<Enemy>(target).unwrap().incapacitate_t,
            0.0
        );
    }
}

#[test]
fn repentance_cast_preserves_readiness_without_eligible_targets_or_training() {
    let (mut app, target, _) = setup();
    app.world_mut()
        .spawn((Transform::default(), Hero { vy: 0.0, air: 0.0 }));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(100));
    for (trained, boss, distance, expected) in [
        (false, false, 10.0, false),
        (true, true, 10.0, false),
        (true, false, 25.0, false),
        (true, false, 10.0, true),
    ] {
        app.world_mut().get_mut::<Enemy>(target).unwrap().boss = boss;
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * distance;
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = run
                .milestones
                .with_ranks(if trained { &[(2, 1441, 1)] } else { &[] });
            run.abilities[0].cd = 0.0;
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let hits: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<Hit>>()
            .drain()
            .collect();
        assert_eq!(hits.len(), usize::from(expected));
        assert_eq!(
            app.world().resource::<Run>().abilities[0].cd > 0.0,
            expected
        );
    }
}

#[test]
fn repentance_stops_contact_and_movement_until_active_time_expires() {
    use avian3d::prelude::PhysicsPlugins;
    use benilla_world::collision::MoverTraceExclusions;
    let (mut app, target, _) = setup();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        PhysicsPlugins::new(PostUpdate),
    ));
    app.init_asset::<Mesh>();
    app.init_resource::<MoverTraceExclusions>();
    app.world_mut()
        .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
    app.world_mut()
        .entity_mut(target)
        .insert(crate::creature_anim::MovementState::default());
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 0.5;
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .get_mut::<Enemy>(target)
        .unwrap()
        .incapacitate_t = 6.0;
    app.world_mut().insert_resource(Time::<()>::default());
    app.world_mut()
        .run_system_once(enemies::move_enemies)
        .unwrap();
    assert_eq!(
        app.world().get::<Enemy>(target).unwrap().incapacitate_t,
        6.0
    );
    let hp = app.world().resource::<Run>().hp;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(5));
    app.world_mut()
        .run_system_once(enemies::move_enemies)
        .unwrap();
    assert_eq!(app.world().resource::<Run>().hp, hp);
    assert_eq!(
        app.world().get::<Enemy>(target).unwrap().incapacitate_t,
        1.0
    );
    assert_eq!(
        app.world().get::<Transform>(target).unwrap().translation,
        Vec3::X * 0.5
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.world_mut()
        .run_system_once(enemies::move_enemies)
        .unwrap();
    assert_eq!(
        app.world().get::<Enemy>(target).unwrap().incapacitate_t,
        0.0
    );
    assert!(app.world().resource::<Run>().hp < hp);
}
