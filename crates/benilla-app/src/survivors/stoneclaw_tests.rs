use super::*;
use crate::survivors::{stoneclaw, Owned};
use bevy::ecs::system::RunSystemOnce;

fn setup(supported: bool) -> (App, Entity, Entity) {
    let (mut app, hero, enemy) = super::earthbind_tests::setup(supported);
    app.world_mut().resource_mut::<Run>().abilities[0] = Owned {
        def: data::ability("stoneclaw_totem"),
        level: 1,
        cd: 0.,
    };
    (app, hero, enemy)
}
fn deployed(app: &mut App) -> Entity {
    app.world_mut().run_system_once(cast_abilities).unwrap();
    app.world_mut()
        .query_filtered::<Entity, With<stoneclaw::Stoneclaw>>()
        .single(app.world())
        .unwrap()
}
fn move_once(app: &mut App) {
    app.world_mut()
        .run_system_once(enemies::move_enemies)
        .unwrap();
}

#[test]
fn stoneclaw_captures_even_when_hero_is_closer_and_retains_until_expiry() {
    let (mut app, hero, enemy) = setup(true);
    let totem = deployed(&mut app);
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 6.;
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x < 5.);
    assert!(app
        .world()
        .get::<stoneclaw::Stoneclaw>(totem)
        .unwrap()
        .captured
        .contains(&enemy));
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .unwrap()
        .translation = Vec3::X * 12.;
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 14.;
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x < 12.);
    app.world_mut().resource_mut::<Run>().elapsed = 15.;
    // Expired targets are ignored even before lifecycle cleanup runs.
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x >= 12.);
    app.world_mut().run_system_once(stoneclaw::tick).unwrap();
    assert!(app.world().get::<stoneclaw::Stoneclaw>(totem).is_none());
}

#[test]
fn stoneclaw_bosses_and_out_of_range_enemies_keep_hero_target() {
    for (boss, distance) in [(true, 5.), (false, 10.)] {
        let (mut app, hero, enemy) = setup(true);
        let totem = deployed(&mut app);
        app.world_mut().get_mut::<Enemy>(enemy).unwrap().boss = boss;
        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::X * distance;
        app.world_mut()
            .get_mut::<Transform>(hero)
            .unwrap()
            .translation = Vec3::X * 20.;
        move_once(&mut app);
        assert!(app.world().get::<Transform>(enemy).unwrap().translation.x > distance);
        assert!(app
            .world()
            .get::<stoneclaw::Stoneclaw>(totem)
            .unwrap()
            .captured
            .is_empty());
    }
}

#[test]
fn stoneclaw_absorbs_fatal_contact_without_hero_damage_or_reflection() {
    let (mut app, hero, enemy) = setup(true);
    let totem = deployed(&mut app);
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 3.;
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .unwrap()
        .translation = Vec3::X * 0.5;
    app.world_mut().get_mut::<Enemy>(enemy).unwrap().damage = 100.;
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.thorns = 20.;
        run.thorns_t = 100.;
        run.thorns_spell = 467;
    }
    let hp = app.world().resource::<Run>().hp;
    move_once(&mut app);
    assert_eq!(
        app.world()
            .get::<stoneclaw::Stoneclaw>(totem)
            .unwrap()
            .life
            .health(),
        0.
    );
    assert_eq!(app.world().resource::<Run>().hp, hp);
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x > 0.5);
}

#[test]
fn stoneclaw_grounding_replacement_and_hero_death() {
    let (mut app, _, _) = setup(false);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(
        app.world_mut()
            .query::<&stoneclaw::Stoneclaw>()
            .iter(app.world())
            .count(),
        0
    );
    let (mut app, _, _) = setup(true);
    let old = deployed(&mut app);
    app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
    let new = deployed(&mut app);
    assert_ne!(old, new);
    assert!(app.world().get_entity(old).is_err());
    app.world_mut().resource_mut::<Run>().hp = 0.;
    app.world_mut().run_system_once(stoneclaw::tick).unwrap();
    assert!(app.world().get::<stoneclaw::Stoneclaw>(new).is_none());
}

#[test]
fn stoneclaw_is_shaman_only_and_ranks_preserve_lifetime() {
    for class in data::CLASSES {
        assert_eq!(
            class.ability_keys().contains(&"stoneclaw_totem"),
            class.id == 7
        );
    }
    let run = Run::default();
    for level in 1..=8 {
        let p = Params::new(
            &Owned {
                def: data::ability("stoneclaw_totem"),
                level,
                cd: 0.,
            },
            &run.stats(),
            run.milestones,
        );
        assert_eq!(p.duration, 15.);
        assert_eq!(p.dmg, 0.);
    }
}

#[test]
fn stoneclaw_overrides_pet_taunt_and_hero_vanish() {
    use crate::survivors::hunter_pet::{HunterPet, PetTaunt};
    let (mut app, hero, enemy) = setup(true);
    deployed(&mut app);
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 12.;
    let mut pet = HunterPet::default();
    pet.vitality = survivors_core::combat_pet::Vitality::new(100., 10.);
    let pet = app
        .world_mut()
        .spawn((
            pet,
            Transform::from_xyz(10., 0., 0.),
            crate::net::ObjectStore::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(enemy)
        .insert(PetTaunt { pet, until: 60. });
    app.world_mut().resource_mut::<Run>().mobile.vanish_t = 10.;
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x < 5.);
}

#[test]
fn stoneclaw_requires_sight_to_acquire_but_not_to_retain() {
    use avian3d::prelude::*;
    let (mut app, hero, enemy) = setup(true);
    let totem = deployed(&mut app);
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 12.;
    let wall = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(0.4, 6., 10.),
            Transform::from_xyz(2.5, 3., 0.),
        ))
        .id();
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    move_once(&mut app);
    assert!(app
        .world()
        .get::<stoneclaw::Stoneclaw>(totem)
        .unwrap()
        .captured
        .is_empty());
    app.world_mut().entity_mut(wall).despawn();
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    move_once(&mut app);
    assert!(app
        .world()
        .get::<stoneclaw::Stoneclaw>(totem)
        .unwrap()
        .captured
        .contains(&enemy));
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.4, 6., 10.),
        Transform::from_xyz(2.5, 3., 0.),
    ));
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    let before = app.world().get::<Transform>(enemy).unwrap().translation.x;
    move_once(&mut app);
    assert!(app.world().get::<Transform>(enemy).unwrap().translation.x < before);
}

#[test]
fn earths_grasp_changes_deployed_health_and_damage_budget() {
    for (rank, health) in [(0, 50.), (1, 62.5), (2, 75.)] {
        let (mut app, hero, enemy) = setup(true);
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 5,
            ..Default::default()
        }
        .with_ranks(&[(0, 572, rank)]);
        let totem = deployed(&mut app);
        let body = app.world().get::<stoneclaw::Stoneclaw>(totem).unwrap();
        assert_eq!(body.life.health(), health);
        assert_eq!(body.life.maximum_health(), health);
        assert_eq!(body.radius, 8.);
        app.world_mut()
            .get_mut::<Transform>(hero)
            .unwrap()
            .translation = Vec3::X * 3.;
        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::X * 0.5;
        app.world_mut().get_mut::<Enemy>(enemy).unwrap().damage = 60.;
        move_once(&mut app);
        assert_eq!(
            app.world()
                .get::<stoneclaw::Stoneclaw>(totem)
                .unwrap()
                .life
                .health(),
            (health - 60.).max(0.)
        );
    }
}

#[test]
fn stoneclaw_retains_distant_captives_until_expiry_then_recycles() {
    let (mut app, hero, enemy) = setup(true);
    let totem = deployed(&mut app);
    move_once(&mut app);
    assert!(app
        .world()
        .get::<stoneclaw::Stoneclaw>(totem)
        .unwrap()
        .captured
        .contains(&enemy));
    app.world_mut()
        .get_mut::<Transform>(hero)
        .unwrap()
        .translation = Vec3::X * 100.;
    let before = app.world().get::<Transform>(enemy).unwrap().translation;
    move_once(&mut app);
    let retained = app.world().get::<Transform>(enemy).unwrap().translation;
    assert!(
        retained.x < before.x,
        "captured enemy must keep approaching the totem"
    );
    assert!(
        retained.distance(before) < 5.,
        "captured enemy must not teleport"
    );
    app.world_mut().resource_mut::<Run>().elapsed = 15.;
    move_once(&mut app);
    let released = app.world().get::<Transform>(enemy).unwrap().translation;
    // This fixture has no supported replacement near the distant hero: normal
    // recycling parks the enemy until streamed floor becomes available.
    assert_eq!(released, retained);
    assert_eq!(
        app.world()
            .get::<crate::creature_anim::MovementState>(enemy)
            .unwrap()
            .speed,
        0.
    );
}
