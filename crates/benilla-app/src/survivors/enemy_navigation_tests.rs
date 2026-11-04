use super::*;
use avian3d::prelude::*;
use benilla_world::collision::MoverTraceExclusions;
use bevy::ecs::system::RunSystemOnce;

#[test]
fn visible_taunting_pet_beyond_low_fence_keeps_enemy_detour() {
    use super::super::hunter_pet::{HunterPet, PetTaunt};
    use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
    let mut app = App::new();
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
    app.init_resource::<Rng>();
    app.init_resource::<PlaySeq>();
    app.insert_resource(Run {
        hp: 100.,
        ..Default::default()
    });
    app.add_message::<CastEvent>();
    app.add_message::<SpellGoTargets>();
    app.add_message::<SwingMessage>();
    app.add_message::<crate::combat_text::CombatTextSpawn>();
    app.add_message::<Hit>();
    let center = hero::arena(app.world().resource::<Selection>().map_def());
    let start = center - Vec3::X * 4.;
    let goal = center + Vec3::X * 4.;
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(100., 2., 100.),
        Transform::from_translation(center - Vec3::Y),
    ));
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.4, 0.8, 6.),
        Transform::from_translation(center + Vec3::Y * 0.4),
    ));
    app.world_mut().spawn((
        Hero { vy: 0., air: 0. },
        Transform::from_translation(center + Vec3::X * 12.),
    ));
    let mut pet = HunterPet::default();
    pet.vitality = survivors_core::combat_pet::Vitality::new(1000., 10.);
    let pet = app
        .world_mut()
        .spawn((
            pet,
            Transform::from_translation(goal),
            ObjectStore::default(),
        ))
        .id();
    app.world_mut()
        .run_system_once(move |mut commands: Commands, mut run: ResMut<Run>| {
            spawn_enemy(
                &mut commands,
                &mut run,
                &data::roster(0).enemies[0],
                start,
                goal,
                false,
                false,
            );
        })
        .unwrap();
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .entity_mut(enemy)
        .insert(PetTaunt { pet, until: 60. });
    {
        let mut body = app.world_mut().get_mut::<Enemy>(enemy).unwrap();
        body.radius = 0.6;
        body.speed = 3.;
    }
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .run_system_once(move |floor: Floor| {
            assert!(
                floor.line_clear(start, goal),
                "pet is visible above the fence"
            );
            assert!(
                floor.travel(start, goal, 0.6).distance(goal) > 1.,
                "body cannot cross fence directly"
            );
        })
        .unwrap();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.05));
    app.insert_resource(time);
    let movement = app.world_mut().register_system(move_enemies);
    let mut detoured = false;
    for frame in 0..600 {
        app.world_mut().resource_mut::<Run>().elapsed = frame as f32 * 0.05;
        app.world_mut().run_system(movement).unwrap();
        let at = app.world().get::<Transform>(enemy).unwrap().translation;
        detoured |= (at.z - center.z).abs() > 3.;
        assert!(
            (at.y - center.y).abs() < 0.05,
            "enemy must not climb the fence: {at:?}"
        );
        if app.world().get::<HunterPet>(pet).unwrap().vitality.hp < 1000. {
            assert!(detoured, "contact must follow a detour around the fence");
            assert!(at.x > center.x);
            return;
        }
    }
    panic!("visible pet must not reset the enemy route every frame");
}
