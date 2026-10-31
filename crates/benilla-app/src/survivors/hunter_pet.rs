//! Hunter's combat companion, separate from the account's pickup companions.
//! Authored Survivors rules: one wolf, nearby enemy attention, and timed recovery.
use bevy::prelude::*;

use super::{
    abilities::Hit, enemies::Enemy, hero::Hero, navigation::Navigation, units, Floor, Run,
    Selection,
};
use crate::creature_anim::{move_flags, MovementState};

pub(super) const RADIUS: f32 = 0.4;
const LEASH: f32 = 18.0;
const DISPLAY: u32 = 604; // Timber Wolf, CreatureDisplayInfo.
pub(super) const SOURCE: &str = "Hunter Pet";

#[derive(Component, Default)]
pub(super) struct HunterPet {
    pub(super) vitality: survivors_core::combat_pet::Vitality,
    pub(super) incoming_slows: survivors_core::slows::Slows,
    bond_elapsed: f32,
    cooldown: f32,
    frenzy: survivors_core::talents::PetFrenzy,
}

impl HunterPet {
    fn new(run: &Run) -> Self {
        let max_hp = maximum_health(run);
        Self {
            vitality: survivors_core::combat_pet::Vitality::new(
                max_hp,
                run.milestones.hunter_pet(run.elapsed).recovery,
            ),
            ..Default::default()
        }
    }

    pub(super) fn hurt(&mut self, damage: f32, now: f32) -> f32 {
        let removed = self.vitality.hurt(damage, now);
        if removed > 0.0 && self.vitality.hp <= 0.0 {
            self.frenzy = Default::default();
            self.bond_elapsed = 0.0;
        }
        removed
    }
}

fn maximum_health(run: &Run) -> f32 {
    (60.0 + 6.0 * run.level.saturating_sub(1) as f32)
        * run.milestones.hunter_pet(run.elapsed).health
}

/// Forced attention expires in run time and never survives the wolf's defeat.
#[derive(Component)]
pub(super) struct PetTaunt {
    pub pet: Entity,
    pub until: f32,
}

#[derive(Component)]
pub(super) struct PetStatusText;

pub(super) fn update_status(
    run: Res<Run>,
    pets: Query<&HunterPet>,
    mut labels: Query<&mut Text, With<PetStatusText>>,
) {
    let value = if !run.alive() {
        String::new()
    } else {
        pets.single().map_or_else(
            |_| String::new(),
            |pet| {
                if let Some(at) = pet.vitality.revive_at {
                    format!(
                        "Wolf recovering · {:.0} sec",
                        (at - run.elapsed).max(0.0).ceil()
                    )
                } else {
                    format!(
                        "Wolf {:.0} / {:.0}",
                        pet.vitality.hp.ceil(),
                        pet.vitality.max_hp.ceil()
                    )
                }
            },
        )
    };
    for mut text in &mut labels {
        if **text != value {
            **text = value.clone();
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn fight(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    selection: Res<Selection>,
    world: Floor,
    hero: Query<(Entity, &Transform), (With<Hero>, Without<HunterPet>)>,
    enemies: Query<(Entity, &Transform, &Enemy), (Without<Hero>, Without<HunterPet>)>,
    mut pets: Query<(Entity, &mut Transform, &mut MovementState, &mut HunterPet), Without<Hero>>,
    perception: super::perception::Perception,
    mut navigation: Local<Navigation>,
    mut hits: MessageWriter<Hit>,
    mut rng: ResMut<super::Rng>,
    mut fx: units::Fx,
) {
    if !run.alive() || run.class.id != 3 {
        for (entity, ..) in &pets {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Ok((owner, hero)) = hero.single() else {
        return;
    };
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
        commands.entity(entity).insert(HunterPet::new(&run));
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
        if let Some(at) = pet.vitality.revive_at {
            navigation.forget(entity);
            if run.elapsed >= at && world.can_spawn(hpos, RADIUS) {
                commands.entity(entity).despawn();
            }
            continue;
        }
        let max_hp = maximum_health(&run);
        pet.vitality.resize(max_hp);
        let maximum = pet.vitality.max_hp;
        pet.vitality.heal(maximum * bonus.mend * dt);
        if bonus.spirit_bond > 0.0 {
            pet.bond_elapsed += dt;
            let pulses = (pet.bond_elapsed / 10.0).floor();
            if pulses > 0.0 {
                pet.bond_elapsed %= 10.0;
                pet.vitality.heal(maximum * bonus.spirit_bond * pulses);
                run.heal(stats.max_hp * bonus.spirit_bond * pulses, stats.max_hp);
            }
        }
        pet.cooldown = (pet.cooldown - dt).max(0.0);
        pet.frenzy.advance(dt);
        let at = tf.translation;
        // Full 3D distances prevent biting enemies on another floor.
        let target = (at.distance(hpos) <= LEASH)
            .then(|| {
                enemies
                    .iter()
                    .filter(|(entity, target, enemy)| {
                        perception.visible(Some(owner), *entity, run.elapsed)
                            && !enemy.dead
                            && target.translation.distance(hpos) <= LEASH
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
                    .filter(|(entity, tf, enemy)| {
                        perception.visible(Some(owner), *entity, run.elapsed)
                            && !enemy.dead
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
                for (index, (victim, _, enemy)) in victims
                    .into_iter()
                    .take(1 + stats.targets as usize)
                    .enumerate()
                {
                    let critical = rng.f32() < (stats.crit + bonus.crit).clamp(0.0, 1.0);
                    let damage = damage * if critical { stats.crit_damage } else { 1.0 };
                    if critical && damage > 0.0 {
                        pet.frenzy.critical(bonus.frenzy_chance, rng.f32());
                    }
                    let mut hit = Hit::direct(victim, damage, critical);
                    hit.source = SOURCE;
                    hit.origin = super::abilities::HitOrigin::CombatPet;
                    if index == 0
                        && bonus.intimidation
                        && run.elapsed >= run.pet_intimidation_ready_at
                    {
                        hit.effect = super::data::Effect::Stun(3.0);
                        if !enemy.boss {
                            commands.entity(victim).insert(PetTaunt {
                                pet: entity,
                                until: run.elapsed + 6.0,
                            });
                        }
                        run.pet_intimidation_ready_at = run.elapsed + 60.0;
                    }
                    hits.write(hit);
                    fx.swing(entity, victim, damage);
                }
                pet.cooldown = 2.0 * bonus.cooldown / (1.0 + pet.frenzy.haste());
            }
        } else if !in_reach || target.is_some() {
            let waypoint = navigation.waypoint(entity, at, goal).unwrap_or(goal);
            let dir = units::flat(waypoint - at);
            let speed =
                stats.speed.max(7.0) * 1.15 * bonus.speed * pet.incoming_slows.state(run.elapsed).0;
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
            .insert_resource(Rng(1))
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
                MovementState::default(),
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
                    slows: Default::default(),
                    root_t: 0.0,
                    frozen_t: 0.0,
                    winters_chill: Default::default(),
                    shadow_weaving: Default::default(),
                    kidney_t: 0.0,
                    stun_t: 0.0,
                    fear_t: 0.0,
                    incapacitate_t: 0.0,
                    weaken: 1.0,
                    weaken_t: 0.0,
                    poison: Default::default(),
                    dots: vec![],
                    dead: false,
                },
            ))
            .id()
    }

    #[test]
    fn movement_reveals_weaker_slow_after_stronger_source_expires() {
        let mut app = setup();
        let target = foe(&mut app, Vec3::X * 10.);
        {
            let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
            enemy.slows.apply(
                survivors_core::slows::Source::Spell("frostbolt"),
                0.5,
                2.5,
                0.,
            );
            enemy.slows.apply(
                survivors_core::slows::Source::Spell("thunder_clap"),
                0.8,
                10.,
                0.,
            );
        }
        for (now, expected) in [(0., 1.5), (2.5, 2.4), (10., 3.)] {
            app.world_mut().resource_mut::<Run>().elapsed = now;
            app.world_mut()
                .run_system_once(super::super::enemies::move_enemies)
                .unwrap();
            assert!(
                (app.world().get::<MovementState>(target).unwrap().speed - expected).abs() < 0.001
            );
        }
    }

    #[test]
    fn bites_only_acquire_concealed_targets_revealed_to_the_owner() {
        for (own_mark, elapsed, expected) in
            [(false, 0.0, false), (true, 0.0, true), (true, 120.0, false)]
        {
            let mut app = setup();
            let owner = app
                .world_mut()
                .query_filtered::<Entity, With<Hero>>()
                .single(app.world())
                .unwrap();
            let target = foe(&mut app, Vec3::X);
            let marker = if own_mark {
                owner
            } else {
                app.world_mut().spawn_empty().id()
            };
            let mut mark = super::super::marking::Marked::default();
            assert!(mark.0.apply(marker, 20.0, 120.0, 0.0));
            app.world_mut()
                .entity_mut(target)
                .insert((super::super::perception::Concealed, mark));
            app.world_mut().resource_mut::<Run>().elapsed = elapsed;
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(fight).unwrap();
            assert_eq!(
                !app.world().resource::<Messages<Hit>>().is_empty(),
                expected
            );
            for hit in app.world_mut().resource_mut::<Messages<Hit>>().drain() {
                assert_eq!(hit.origin, super::super::abilities::HitOrigin::CombatPet);
                assert_eq!(hit.source, SOURCE);
            }
        }
    }

    #[test]
    fn nearby_enemy_hurts_wolf_without_hero_damage_or_thorns_then_recovery_resummons() {
        let mut app = setup();
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut()
            .query_filtered::<&mut Transform, With<Hero>>()
            .single_mut(app.world_mut())
            .unwrap()
            .translation = Vec3::X * 3.0;
        let enemy = foe(&mut app, Vec3::X);
        app.world_mut().get_mut::<Enemy>(enemy).unwrap().damage = 100.0;
        let hero_hp = app.world().resource::<Run>().hp;
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.thorns_t = 10.0;
            run.thorns = 50.0;
        }
        app.world_mut()
            .run_system_once(super::super::enemies::move_enemies)
            .unwrap();
        assert_eq!(app.world().resource::<Run>().hp, hero_hp);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        let dead = app
            .world_mut()
            .query_filtered::<Entity, With<HunterPet>>()
            .single(app.world())
            .unwrap();
        assert_eq!(app.world().get::<HunterPet>(dead).unwrap().vitality.hp, 0.0);
        // Switching back to the hero must not grant another attack in the same cooldown.
        app.world_mut()
            .query_filtered::<&mut Transform, With<Hero>>()
            .single_mut(app.world_mut())
            .unwrap()
            .translation = Vec3::X * 1.2;
        app.world_mut()
            .run_system_once(super::super::enemies::move_enemies)
            .unwrap();
        assert_eq!(app.world().resource::<Run>().hp, hero_hp);
        app.world_mut().resource_mut::<Run>().elapsed = 9.99;
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().get::<HunterPet>(dead).is_some());
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        app.world_mut().resource_mut::<Run>().elapsed = 10.0;
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().get::<HunterPet>(dead).is_none());
        app.world_mut().run_system_once(fight).unwrap();
        let pet = app
            .world_mut()
            .query::<&HunterPet>()
            .single(app.world())
            .unwrap();
        assert_eq!(pet.vitality.hp, 60.0);
        assert_eq!(pet.vitality.revive_at, None);
    }

    #[test]
    fn bosses_keep_attacking_hero_and_endurance_scales_actual_pet_health() {
        let mut app = setup();
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(0, 1389, 5)]);
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut()
            .query_filtered::<&mut Transform, With<Hero>>()
            .single_mut(app.world_mut())
            .unwrap()
            .translation = Vec3::X * 3.0;
        let enemy = foe(&mut app, Vec3::X);
        {
            let mut e = app.world_mut().get_mut::<Enemy>(enemy).unwrap();
            e.damage = 10.0;
            e.boss = true;
            e.radius = 1.5;
        }
        let hero_hp = app.world().resource::<Run>().hp;
        app.world_mut()
            .run_system_once(super::super::enemies::move_enemies)
            .unwrap();
        assert!(app.world().resource::<Run>().hp < hero_hp);
        let pet = app
            .world_mut()
            .query::<&HunterPet>()
            .single(app.world())
            .unwrap();
        assert!((pet.vitality.hp - 69.0).abs() < 0.0001);
    }

    #[test]
    fn mend_regenerates_only_living_wolf_in_game_time_and_caps_at_maximum() {
        for rank in [0u8, 1, 2, 255] {
            let mut app = setup();
            app.world_mut().resource_mut::<Run>().milestones =
                survivors_core::talents::Milestones {
                    class: 2,
                    ..Default::default()
                }
                .with_ranks(&[(0, 1385, rank)]);
            app.world_mut().run_system_once(fight).unwrap();
            let pet = app
                .world_mut()
                .query_filtered::<Entity, With<HunterPet>>()
                .single(app.world())
                .unwrap();
            let max_hp = app.world().get::<HunterPet>(pet).unwrap().vitality.max_hp;
            app.world_mut()
                .get_mut::<HunterPet>(pet)
                .unwrap()
                .vitality
                .hp = max_hp / 2.0;
            let hero_hp = app.world().resource::<Run>().hp;
            // Ten 100 ms frames heal exactly one second of trained regeneration.
            for _ in 0..10 {
                app.world_mut().run_system_once(fight).unwrap();
            }
            let expected = max_hp * (0.5 + 0.01 * f32::from(rank.min(2)));
            assert!(
                (app.world().get::<HunterPet>(pet).unwrap().vitality.hp - expected).abs() < 0.0001
            );
            assert_eq!(app.world().resource::<Run>().hp, hero_hp);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::ZERO);
            app.world_mut().run_system_once(fight).unwrap();
            assert!(
                (app.world().get::<HunterPet>(pet).unwrap().vitality.hp - expected).abs() < 0.0001
            );
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs(1));
            app.world_mut()
                .get_mut::<HunterPet>(pet)
                .unwrap()
                .vitality
                .hp = max_hp;
            app.world_mut().run_system_once(fight).unwrap();
            assert_eq!(
                app.world().get::<HunterPet>(pet).unwrap().vitality.hp,
                max_hp
            );
            app.world_mut()
                .get_mut::<HunterPet>(pet)
                .unwrap()
                .hurt(max_hp, 0.0);
            app.world_mut().run_system_once(fight).unwrap();
            let defeated = app.world().get::<HunterPet>(pet).unwrap();
            assert_eq!(defeated.vitality.hp, 0.0);
            assert_eq!(defeated.vitality.revive_at, Some(10.0));
        }
    }

    #[test]
    fn spirit_bond_heals_both_on_interval_caps_health_and_stops_on_defeat() {
        let mut app = setup();
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(0, 1388, 2), (0, 1625, 2)]);
        app.world_mut().run_system_once(fight).unwrap();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<HunterPet>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .get_mut::<HunterPet>(entity)
            .unwrap()
            .hurt(30.0, 0.0);
        let max = app.world().resource::<Run>().stats().max_hp;
        app.world_mut().resource_mut::<Run>().hp = max * 0.5;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(9));
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world().get::<HunterPet>(entity).unwrap().vitality.hp,
            30.0
        );
        assert_eq!(app.world().resource::<Run>().hp, max * 0.5);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.world_mut().run_system_once(fight).unwrap();
        assert!((app.world().get::<HunterPet>(entity).unwrap().vitality.hp - 31.2).abs() < 0.0001);
        assert!((app.world().resource::<Run>().hp - max * 0.52).abs() < 0.0001);
        app.world_mut()
            .get_mut::<HunterPet>(entity)
            .unwrap()
            .vitality
            .hp = 59.9;
        app.world_mut().resource_mut::<Run>().hp = max - 0.1;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(20));
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world().get::<HunterPet>(entity).unwrap().vitality.hp,
            60.0
        );
        assert_eq!(app.world().resource::<Run>().hp, max);
        app.world_mut()
            .get_mut::<HunterPet>(entity)
            .unwrap()
            .hurt(100.0, 0.0);
        assert!(
            (app.world()
                .get::<HunterPet>(entity)
                .unwrap()
                .vitality
                .revive_at
                .unwrap()
                - 6.0)
                .abs()
                < 0.0001
        );
        app.world_mut().resource_mut::<Run>().hp = max * 0.5;
        app.world_mut().resource_mut::<Run>().elapsed = 5.99;
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(app.world().resource::<Run>().hp, max * 0.5);
        assert_eq!(
            app.world().get::<HunterPet>(entity).unwrap().bond_elapsed,
            0.0
        );
        app.world_mut().resource_mut::<Run>().elapsed = 6.0;
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().get::<HunterPet>(entity).is_none());
    }

    #[test]
    fn intimidation_stuns_only_nearest_bite_target_and_cooldown_survives_recovery() {
        let mut app = setup();
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(0, 1387, 1)]);
        app.world_mut().resource_mut::<Run>().bonus.targets = 1;
        let first = foe(&mut app, Vec3::X * 0.8);
        let second = foe(&mut app, Vec3::X);
        for entity in [first, second] {
            let mut enemy = app.world_mut().get_mut::<Enemy>(entity).unwrap();
            enemy.hp = 100.0;
            enemy.max_hp = 100.0;
        }
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(fight).unwrap();
        assert!(app.world().get::<PetTaunt>(first).is_some());
        assert!(app.world().get::<PetTaunt>(second).is_none());
        app.world_mut().run_system_once(apply_hits).unwrap();
        assert_eq!(app.world().get::<Enemy>(first).unwrap().stun_t, 3.0);
        assert_eq!(app.world().get::<Enemy>(second).unwrap().stun_t, 0.0);
        assert_eq!(
            app.world().resource::<Run>().pet_intimidation_ready_at,
            60.0
        );
        app.world_mut()
            .query::<&mut HunterPet>()
            .single_mut(app.world_mut())
            .unwrap()
            .hurt(100.0, 0.0);
        app.world_mut().resource_mut::<Run>().elapsed = 10.0;
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world().resource::<Run>().pet_intimidation_ready_at,
            60.0
        );
        app.world_mut().resource_mut::<Run>().elapsed = 60.0;
        app.world_mut()
            .query::<&mut HunterPet>()
            .single_mut(app.world_mut())
            .unwrap()
            .cooldown = 0.0;
        app.world_mut().run_system_once(fight).unwrap();
        assert_eq!(
            app.world().resource::<Run>().pet_intimidation_ready_at,
            120.0
        );
    }

    #[test]
    fn intimidation_boss_stun_is_reduced_without_forced_attention() {
        let mut app = setup();
        app.world_mut().resource_mut::<Run>().milestones = survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(0, 1387, 1)]);
        let boss = foe(&mut app, Vec3::X);
        {
            let mut enemy = app.world_mut().get_mut::<Enemy>(boss).unwrap();
            enemy.hp = 100.0;
            enemy.max_hp = 100.0;
            enemy.boss = true;
        }
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut().run_system_once(apply_hits).unwrap();
        assert!((app.world().get::<Enemy>(boss).unwrap().stun_t - 0.9).abs() < 0.0001);
        assert!(app.world().get::<PetTaunt>(boss).is_none());
    }

    #[test]
    fn forced_attention_overrides_proximity_expires_and_ignores_dead_wolf() {
        for (now, dead, boss, expected_pet_damage) in [
            (5.9, false, false, true),
            (6.0, false, false, false),
            (0.0, true, false, false),
            (0.0, false, true, false),
        ] {
            let mut app = setup();
            app.world_mut().run_system_once(fight).unwrap();
            let pet = app
                .world_mut()
                .query_filtered::<Entity, With<HunterPet>>()
                .single(app.world())
                .unwrap();
            app.world_mut()
                .get_mut::<Transform>(pet)
                .unwrap()
                .translation = Vec3::X;
            if dead {
                app.world_mut()
                    .get_mut::<HunterPet>(pet)
                    .unwrap()
                    .hurt(100.0, 0.0);
            }
            let enemy = foe(&mut app, Vec3::X * 0.2);
            app.world_mut().get_mut::<Enemy>(enemy).unwrap().boss = boss;
            app.world_mut()
                .entity_mut(enemy)
                .insert(PetTaunt { pet, until: 6.0 });
            app.world_mut().resource_mut::<Run>().elapsed = now;
            let hp = app.world().get::<HunterPet>(pet).unwrap().vitality.hp;
            let hero_hp = app.world().resource::<Run>().hp;
            app.world_mut()
                .run_system_once(super::super::enemies::move_enemies)
                .unwrap();
            assert_eq!(
                app.world().get::<HunterPet>(pet).unwrap().vitality.hp < hp,
                expected_pet_damage
            );
            assert_eq!(
                app.world().resource::<Run>().hp < hero_hp,
                !expected_pet_damage
            );
        }
    }

    #[test]
    fn trained_wrath_changes_real_bites_only_inside_extended_window() {
        for (elapsed, expected) in [(10.0, 19.2), (17.999, 19.2), (18.0, 9.6), (30.0, 19.2)] {
            let mut app = setup();
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.elapsed = elapsed;
                run.milestones = survivors_core::talents::Milestones {
                    class: 2,
                    points: [30, 0, 0],
                    ..Default::default()
                }
                .with_ranks(&[(0, 1386, 1)]);
            }
            let enemy = foe(&mut app, Vec3::X);
            app.world_mut().get_mut::<Enemy>(enemy).unwrap().hp = 100.0;
            app.world_mut().insert_resource(Rng(16));
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert!(
                (app.world().get::<Enemy>(enemy).unwrap().hp - (100.0 - expected)).abs() < 0.001
            );
        }
    }

    #[test]
    fn pet_level_growth_preserves_health_fraction() {
        let mut app = setup();
        app.world_mut().run_system_once(fight).unwrap();
        app.world_mut()
            .query::<&mut HunterPet>()
            .single_mut(app.world_mut())
            .unwrap()
            .hurt(30.0, 0.0);
        app.world_mut().resource_mut::<Run>().level = 2;
        app.world_mut().run_system_once(fight).unwrap();
        let pet = app
            .world_mut()
            .query::<&HunterPet>()
            .single(app.world())
            .unwrap();
        assert_eq!(pet.vitality.max_hp, 66.0);
        assert_eq!(pet.vitality.hp, 33.0);
    }

    #[test]
    fn pet_status_reports_health_and_recovery_without_reviving_or_healing() {
        let mut app = setup();
        app.world_mut().run_system_once(fight).unwrap();
        let label = app.world_mut().spawn((PetStatusText, Text::default())).id();
        app.world_mut().run_system_once(update_status).unwrap();
        assert_eq!(app.world().get::<Text>(label).unwrap().0, "Wolf 60 / 60");
        app.world_mut()
            .query::<&mut HunterPet>()
            .single_mut(app.world_mut())
            .unwrap()
            .hurt(80.0, 0.0);
        app.world_mut().resource_mut::<Run>().elapsed = 2.2;
        app.world_mut().run_system_once(update_status).unwrap();
        assert_eq!(
            app.world().get::<Text>(label).unwrap().0,
            "Wolf recovering · 8 sec"
        );
        assert_eq!(
            app.world_mut()
                .query::<&HunterPet>()
                .single(app.world())
                .unwrap()
                .vitality
                .hp,
            0.0
        );
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
    fn trained_bites_crit_and_proc_frenzy_without_buffing_untrained_pet() {
        for trained in [false, true] {
            let mut app = setup();
            app.insert_resource(Rng(16)); // First roll 0.0758: above base crit, below trained crit.
            app.world_mut().resource_mut::<Run>().milestones =
                survivors_core::talents::Milestones {
                    class: 2,
                    ..Default::default()
                }
                .with_ranks(if trained {
                    &[(0, 1396, 5), (0, 1393, 5), (0, 1397, 5)]
                } else {
                    &[]
                });
            let target = foe(&mut app, Vec3::X);
            app.world_mut().get_mut::<Enemy>(target).unwrap().hp = 100.0;
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(fight).unwrap();
            let pet = app
                .world_mut()
                .query::<&HunterPet>()
                .single(app.world())
                .unwrap();
            assert_eq!(pet.frenzy.haste(), if trained { 0.3 } else { 0.0 });
            assert!((pet.cooldown - if trained { 2.0 / 1.3 } else { 2.0 }).abs() < 0.0001);
            app.world_mut().run_system_once(apply_hits).unwrap();
            let damage = if trained { 19.2 } else { 8.0 };
            assert!(
                (app.world().get::<Enemy>(target).unwrap().hp - (100.0 - damage)).abs() < 0.0001
            );
        }
    }

    #[test]
    fn discipline_times_real_bites_and_zero_delta_preserves_cooldown() {
        for (rank, points, frenzy, interval) in [
            (0, 0, false, 2.0),
            (1, 0, false, 1.8),
            (2, 0, false, 1.6),
            (255, 0, false, 1.6),
            (2, 20, false, 1.28),
            (2, 20, true, 1.28 / 1.3),
        ] {
            let mut app = setup();
            app.world_mut().resource_mut::<Run>().milestones =
                survivors_core::talents::Milestones {
                    class: 2,
                    points: [points, 0, 0],
                    ..Default::default()
                }
                .with_ranks(&[(0, 1390, rank)]);
            let target = foe(&mut app, Vec3::X);
            app.world_mut().get_mut::<Enemy>(target).unwrap().hp = 1000.0;
            app.world_mut().run_system_once(fight).unwrap();
            let pet = app
                .world_mut()
                .query_filtered::<Entity, With<HunterPet>>()
                .single(app.world())
                .unwrap();
            if frenzy {
                app.world_mut()
                    .get_mut::<HunterPet>(pet)
                    .unwrap()
                    .frenzy
                    .critical(1.0, 0.0);
            }
            app.world_mut().run_system_once(fight).unwrap();
            assert_eq!(app.world().resource::<Messages<Hit>>().len(), 1);
            assert!(
                (app.world().get::<HunterPet>(pet).unwrap().cooldown - interval).abs() < 0.00001
            );
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert!(app.world().get::<Enemy>(target).unwrap().hp < 1000.0);
            app.world_mut().resource_mut::<Messages<Hit>>().clear();
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::ZERO);
            app.world_mut().run_system_once(fight).unwrap();
            assert!(
                (app.world().get::<HunterPet>(pet).unwrap().cooldown - interval).abs() < 0.00001
            );
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(interval - 0.01));
            app.world_mut().run_system_once(fight).unwrap();
            assert!(
                app.world().resource::<Messages<Hit>>().is_empty(),
                "no early bite at rank {rank}"
            );
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(20));
            app.world_mut().run_system_once(fight).unwrap();
            assert_eq!(
                app.world().resource::<Messages<Hit>>().len(),
                1,
                "bite after trained interval at rank {rank}"
            );
            assert_eq!(
                app.world().get::<HunterPet>(pet).unwrap().vitality.recovery,
                10.0
            );
        }
    }

    #[test]
    fn bestial_swiftness_increases_real_grounded_movement() {
        let mut distances = vec![];
        for rank in [0, 1] {
            let mut app = setup();
            app.world_mut().resource_mut::<Run>().milestones =
                survivors_core::talents::Milestones {
                    class: 2,
                    ..Default::default()
                }
                .with_ranks(&[(0, 1391, rank)]);
            foe(&mut app, Vec3::X * 4.0);
            app.world_mut().run_system_once(fight).unwrap();
            app.world_mut().run_system_once(fight).unwrap();
            let pos = app
                .world_mut()
                .query_filtered::<&Transform, With<HunterPet>>()
                .single(app.world())
                .unwrap()
                .translation;
            assert!(pos.y.abs() < 0.01);
            distances.push(pos.x);
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
        }
        assert!(distances[0] > 0.0);
        assert!((distances[1] / distances[0] - 1.3).abs() < 0.01);
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
    #[test]
    fn actual_parry_emits_counterattack_for_that_enemy_and_applies_root_once() {
        for boss in [false, true] {
            let mut app = setup();
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.milestones = survivors_core::talents::Milestones {
                    class: 2,
                    ..Default::default()
                }
                .with_ranks(&[(2, 1308, 1), (2, 1312, 1)]);
                run.shield = 10.;
                run.shield_t = 10.;
            }
            let target = foe(&mut app, Vec3::X);
            let other = foe(&mut app, Vec3::X * 20.);
            {
                let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
                enemy.hp = 1000.;
                enemy.max_hp = 1000.;
                enemy.boss = boss;
            }
            let hp = app.world().resource::<Run>().hp;
            app.insert_resource(Rng(1)); // First roll 0.280835: parry, above 25% dodge.
            app.world_mut()
                .run_system_once(super::super::enemies::move_enemies)
                .unwrap();
            assert_eq!(app.world().resource::<Run>().hp, hp);
            assert_eq!(app.world().resource::<Run>().shield, 10.);
            let swings: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<SwingMessage>>()
                .drain()
                .collect();
            assert_eq!(swings.len(), 1);
            assert_eq!(swings[0].attacker, target);
            assert_eq!(swings[0].victim_state, 3);
            let hits: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<Hit>>()
                .drain()
                .collect();
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].target, target);
            assert_eq!(hits[0].source, "Counterattack");
            let damage = hits[0].damage;
            app.world_mut().write_message(hits[0].clone());
            app.world_mut().run_system_once(apply_hits).unwrap();
            app.world_mut().resource_mut::<Messages<Hit>>().clear();
            let enemy = app.world().get::<Enemy>(target).unwrap();
            assert_eq!(enemy.hp, 1000. - damage);
            assert_eq!(enemy.root_t, if boss { 1.5 } else { 5. });
            assert_eq!(app.world().get::<Enemy>(other).unwrap().hp, 5.);
            app.world_mut().get_mut::<Enemy>(target).unwrap().attack_cd = 0.;
            app.insert_resource(Rng(1));
            app.world_mut()
                .run_system_once(super::super::enemies::move_enemies)
                .unwrap();
            assert!(
                app.world().resource::<Messages<Hit>>().is_empty(),
                "same-time parry cannot bypass recovery"
            );
            assert_eq!(app.world().resource::<Run>().hp, hp);
        }
    }

    #[test]
    fn curse_weakness_reduces_actual_enemy_contact_before_percentage_weakening() {
        let mut losses = Vec::new();
        for amount in [0.0, 3.0, 20.0] {
            let mut app = setup();
            app.world_mut().resource_mut::<Run>().class = &data::CLASSES[7];
            let target = foe(&mut app, Vec3::X);
            {
                let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
                enemy.hp = 1000.;
                enemy.max_hp = 1000.;
                enemy.damage = 10.;
                enemy.weaken = 0.5;
                enemy.weaken_t = 20.;
            }
            app.world_mut().write_message(Hit {
                source: "curse_of_weakness",
                effect: data::Effect::Weakness(amount, 12.),
                ..Hit::direct(target, 0., false)
            });
            app.world_mut().run_system_once(apply_hits).unwrap();
            let hp = app.world().resource::<Run>().hp;
            app.world_mut()
                .run_system_once(super::super::enemies::move_enemies)
                .unwrap();
            losses.push(hp - app.world().resource::<Run>().hp);
        }
        assert!(losses[0] > 0.0);
        assert!((losses[1] - losses[0] * 0.7).abs() < 0.0001, "{losses:?}");
        assert_eq!(losses[2], 0.0);
    }
}
