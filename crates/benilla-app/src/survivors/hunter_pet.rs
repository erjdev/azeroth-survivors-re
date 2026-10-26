//! Hunter's combat companion, separate from the account's pickup companions.
//! Authored Survivors rules: one permanent wolf, no threat or incoming pet damage.
use bevy::prelude::*;

use super::{
    abilities::Hit, enemies::Enemy, hero::Hero, navigation::Navigation, units, Floor, Run,
    Selection,
};
use crate::creature_anim::{move_flags, MovementState};

const RADIUS: f32 = 0.4;
const LEASH: f32 = 18.0;
const DISPLAY: u32 = 604; // Timber Wolf, CreatureDisplayInfo.
const SOURCE: &str = "Hunter Pet";

#[derive(Component, Default)]
pub(super) struct HunterPet {
    cooldown: f32,
}

#[allow(clippy::type_complexity)]
pub(super) fn fight(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    selection: Res<Selection>,
    world: Floor,
    hero: Query<&Transform, (With<Hero>, Without<HunterPet>)>,
    enemies: Query<(Entity, &Transform, &Enemy), (Without<Hero>, Without<HunterPet>)>,
    mut pets: Query<(Entity, &mut Transform, &mut MovementState, &mut HunterPet), Without<Hero>>,
    mut navigation: Local<Navigation>,
    mut hits: MessageWriter<Hit>,
    mut fx: units::Fx,
) {
    if !run.alive() || run.class.id != 3 {
        for (entity, ..) in &pets {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Ok(hero) = hero.single() else { return };
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let hpos = hero.translation;
    if pets.is_empty() {
        // Start at the supported hero position; do not guess an offset inside a wall.
        if !world.can_spawn(hpos, RADIUS) {
            return;
        }
        let serial = run.guid();
        let entity = units::spawn_unit(
            &mut commands,
            &units::UnitSpec {
                serial,
                display: DISPLAY,
                scale: 1.0,
                level: run.level,
                faction: units::FACTION_FRIENDLY,
                bytes0: 1 << 8,
                weapons: &[],
                sheath: 0,
            },
            hpos,
            0.0,
        );
        commands.entity(entity).insert(HunterPet::default());
        return;
    }
    let bodies: Vec<_> = pets
        .iter()
        .map(|(e, tf, ..)| (e, tf.translation, RADIUS))
        .collect();
    navigation.advance(dt, selection.map, &bodies, |from, to, radius| {
        let end = world.travel(from, to, radius);
        (units::flat(end - to).length() < 0.1 && world.can_spawn(end, radius)).then_some(end)
    });
    let stats = run.stats();
    let bonus = run.milestones.hunter_pet(run.elapsed);
    for (entity, mut tf, mut movement, mut pet) in &mut pets {
        movement.flags = 0;
        movement.speed = 0.0;
        pet.cooldown = (pet.cooldown - dt).max(0.0);
        let at = tf.translation;
        // Full 3D distances prevent biting enemies on another floor.
        let target = (at.distance(hpos) <= LEASH)
            .then(|| {
                enemies
                    .iter()
                    .filter(|(_, target, enemy)| {
                        !enemy.dead && target.translation.distance(hpos) <= LEASH
                    })
                    .min_by(|(_, a, _), (_, b, _)| {
                        a.translation
                            .distance_squared(at)
                            .total_cmp(&b.translation.distance_squared(at))
                    })
            })
            .flatten();
        let (goal, stop) = target.map_or((hpos, 2.0), |(_, target, enemy)| {
            (target.translation, enemy.radius + RADIUS + 0.6)
        });
        let delta = goal - at;
        let in_reach = delta.length() <= stop;
        if target
            .filter(|_| in_reach && world.line_clear(at, goal))
            .is_some()
        {
            navigation.forget(entity);
            if units::flat(delta).length_squared() > 0.0001 {
                tf.rotation = Quat::from_rotation_y(units::yaw_toward(delta));
            }
            if pet.cooldown <= 0.0 {
                let damage =
                    (8.0 + 2.0 * run.level.saturating_sub(1) as f32) * stats.might * bonus.damage;
                let mut victims: Vec<_> = enemies
                    .iter()
                    .filter(|(_, tf, enemy)| {
                        !enemy.dead
                            && tf.translation.distance(hpos) <= LEASH
                            && tf.translation.distance(at) <= enemy.radius + RADIUS + 0.6
                            && world.line_clear(at, tf.translation)
                    })
                    .collect();
                victims.sort_by(|(_, a, _), (_, b, _)| {
                    a.translation
                        .distance_squared(at)
                        .total_cmp(&b.translation.distance_squared(at))
                });
                for (victim, _, _) in victims.into_iter().take(1 + stats.targets as usize) {
                    let mut hit = Hit::direct(victim, damage, false);
                    hit.source = SOURCE;
                    hits.write(hit);
                    fx.swing(entity, victim, damage);
                }
                pet.cooldown = 2.0 * bonus.cooldown;
            }
        } else if !in_reach || target.is_some() {
            let waypoint = navigation.waypoint(entity, at, goal).unwrap_or(goal);
            let dir = units::flat(waypoint - at);
            let speed = stats.speed.max(7.0) * 1.15;
            let step = (speed * dt).min(dir.length());
            let desired = at + dir.normalize_or_zero() * step;
            let end = world.travel(at, desired, RADIUS);
            let blocked = units::flat(end - desired).length() > 0.05;
            navigation.observe(entity, at, goal, RADIUS, blocked, dt);
            tf.translation = end;
            if units::flat(end - at).length_squared() > 0.0001 {
                tf.rotation = Quat::from_rotation_y(units::yaw_toward(end - at));
                movement.flags = move_flags::FORWARD;
                movement.speed = speed;
            }
        } else {
            navigation.forget(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        abilities::{apply_hits, PendingHits},
        data,
        enemies::Loot,
        units::Marks,
        Rng,
    };
    use super::*;
    use crate::{
        creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage},
        net::{FieldChanged, Guid, ObjectStore},
    };
    use bevy::ecs::system::RunSystemOnce;

    fn setup() -> App {
        let mut app = super::super::floor::tests::fixture(&[0.0]);
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.1));
        app.insert_resource(time)
            .insert_resource(Run {
                class: &data::CLASSES[2],
                ..Default::default()
            })
            .insert_resource(super::super::account::empty())
            .init_resource::<Selection>()
            .init_resource::<Rng>()
            .init_resource::<PlaySeq>()
            .init_resource::<PendingHits>()
            .add_message::<Hit>()
            .add_message::<Loot>()
            .add_message::<FieldChanged>()
            .add_message::<CastEvent>()
            .add_message::<SpellGoTargets>()
            .add_message::<SwingMessage>()
            .add_message::<crate::combat_text::CombatTextSpawn>();
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        app
    }

    fn foe(app: &mut App, at: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                Guid(123),
                Transform::from_translation(at),
                ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                Marks::debuffs(),
                Enemy {
                    def: &data::roster(0).enemies[0],
                    hp: 5.0,
                    max_hp: 5.0,
                    speed: 3.0,
                    damage: 1.0,
                    radius: 0.6,
                    xp: 1,
                    boss: false,
                    elite: false,
                    attack_cd: 0.0,
                    slow: 1.0,
                    slow_t: 0.0,
                    root_t: 0.0,
                    frozen_t: 0.0,
                    winters_chill: Default::default(),
                    shadow_weaving: Default::default(),
                    stun_t: 0.0,
                    fear_t: 0.0,
                    weaken: 1.0,
                    weaken_t: 0.0,
                    dots: vec![],
                    dead: false,
                },
            ))
            .id()
    }

    #[test]
    fn hunter_gets_one_pet_and_other_classes_do_not() {
        let mut app = setup();
        for _ in 0..3 {
            app.world_mut().run_system_once(fight).unwrap();
        }
        assert_eq!(
            app.world_mut()
                .query::<&HunterPet>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut().resource_mut::<Run>().class = &data::CLASSES[0];
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&HunterPet>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn bite_uses_normal_damage_kill_loot_and_cooldown_pipeline() {
        let mut app = setup();
        let target = foe(&mut app, Vec3::X);
        app.world_mut().run_system_once(fight).unwrap(); // summon
        app.world_mut().run_system_once(fight).unwrap(); // bite
        app.world_mut().run_system_once(apply_hits).unwrap();
        assert!(app.world().get::<Enemy>(target).unwrap().dead);
        assert_eq!(app.world().resource::<Run>().kills, 1);
        assert_eq!(app.world().resource::<Messages<Loot>>().len(), 1);
        app.world_mut().resource_mut::<Messages<Hit>>().clear();
        let next = foe(&mut app, Vec3::X);
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        assert!(!app.world().get::<Enemy>(next).unwrap().dead);
        // Neither another floor nor a distant enemy can receive a melee bite.
        app.world_mut()
            .query::<&mut HunterPet>()
            .single_mut(app.world_mut())
            .unwrap()
            .cooldown = 0.0;
        app.world_mut()
            .get_mut::<Transform>(next)
            .unwrap()
            .translation = Vec3::Y * 4.0;
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        app.world_mut().resource_mut::<Run>().hp = 0.0;
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world_mut()
                .query::<&HunterPet>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn beast_mastery_changes_real_bites_and_cooldowns() {
        for (points, elapsed, damage, cooldown) in [
            (9, 0.0, 8.0, 2.0),
            (10, 0.0, 9.6, 2.0),
            (20, 0.0, 9.6, 1.6),
            (30, 0.0, 14.4, 1.6),
            (30, 10.0, 9.6, 1.6),
        ] {
            let mut app = setup();
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.elapsed = elapsed;
                run.milestones = survivors_core::talents::Milestones {
                    class: 2,
                    points: [points, 0, 0],
                    ..Default::default()
                };
            }
            let target = foe(&mut app, Vec3::X);
            app.world_mut().get_mut::<Enemy>(target).unwrap().hp = 100.0;
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(fight).unwrap();
            let bite_cd = app
                .world_mut()
                .query::<&HunterPet>()
                .single(app.world())
                .unwrap()
                .cooldown;
            assert!((bite_cd - cooldown).abs() < 0.00001);
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert!(
                (app.world().get::<Enemy>(target).unwrap().hp - (100.0 - damage)).abs() < 0.0001
            );
        }
    }

    #[test]
    fn extra_targets_add_melee_victims_without_extending_reach() {
        let mut app = setup();
        app.world_mut().resource_mut::<Run>().bonus.targets = 1;
        for at in [Vec3::X, Vec3::Z, Vec3::NEG_X, Vec3::X * 4.0] {
            foe(&mut app, at);
        }
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(app.world().resource::<Messages<Hit>>().len(), 2);
        app.world_mut().run_system_once(apply_hits).unwrap();
        assert_eq!(app.world().resource::<Run>().kills, 2);
    }

    #[test]
    fn pet_cannot_bite_through_solid_scenery() {
        use avian3d::prelude::{Collider, RigidBody};
        let mut app = setup();
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.15, 4.0, 4.0),
            Transform::from_xyz(0.8, 2.0, 0.0),
        ));
        app.update();
        app.update();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        foe(&mut app, Vec3::X * 1.5);
        app.world_mut().run_system_once(fight).unwrap();
        let fight_id = app.world_mut().register_system(fight);
        // Reuse the same system state so the navigation queue advances between frames.
        for _ in 0..10 {
            app.world_mut().run_system(fight_id).unwrap();
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            let position = app
                .world_mut()
                .query_filtered::<&Transform, With<HunterPet>>()
                .single(app.world())
                .unwrap()
                .translation;
            assert!(position.x < 0.8, "pet crossed the wall: {position:?}");
        }
    }

    #[test]
    fn wolf_display_resolves_to_readable_model() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_creature_catalog(&mut chain).unwrap();
        let model = catalog.model(DISPLAY).expect("Hunter wolf");
        let url = benilla_assets::m2_url(&model.model_path);
        assert!(!chain
            .read(url.strip_prefix("mpq://").unwrap())
            .unwrap()
            .is_empty());
    }
}
