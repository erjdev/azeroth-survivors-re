//! Connected channel victims and movement effects, separate from ordinary timed snares.
use super::{abilities::Hit, enemies::Enemy, hero::Hero, Run};
use bevy::prelude::*;
use survivors_core::channel::Channel;

pub(super) struct ActiveChannel {
    pub caster: Entity,
    pub clock: Channel<Entity>,
    pub damage: f32,
    pub range: f32,
    pub slow: f32,
    pub source: &'static str,
}

#[derive(Resource, Default)]
pub(super) struct Channels {
    pub active: Vec<ActiveChannel>,
    slowed: std::collections::HashMap<Entity, f32>,
}

impl Channels {
    /// The strongest snare wins. Ending a channel does not mutate another effect's timer.
    pub fn movement(&self, target: Entity, other: f32) -> f32 {
        if let Some(&slow) = self.slowed.get(&target) {
            other.min(slow)
        } else {
            other
        }
    }

    fn advance(
        &mut self,
        now: f32,
        alive: bool,
        mut connected: impl FnMut(Entity, Entity, f32) -> bool,
    ) -> Vec<Hit> {
        self.slowed.clear();
        let mut hits = Vec::new();
        if !alive {
            self.active.clear();
            return hits;
        }
        for channel in &mut self.active {
            let pulses = channel.clock.advance(now, |target| {
                connected(channel.caster, target, channel.range)
            });
            for (target, scheduled) in pulses {
                let mut hit = Hit::direct(target, channel.damage, false);
                hit.occurred = Some(scheduled);
                hit.source = channel.source;
                hit.color = super::units::COLOR_SPELL;
                hits.push(hit);
            }
            if !channel.clock.finished()
                && channel.slow.is_finite()
                && (0.0..=1.0).contains(&channel.slow)
            {
                for &target in channel.clock.targets() {
                    self.slowed
                        .entry(target)
                        .and_modify(|slow| *slow = slow.min(channel.slow))
                        .or_insert(channel.slow);
                }
            }
        }
        self.active.retain(|channel| !channel.clock.finished());
        hits
    }
}

/// Runs before enemy movement: disconnected channels stop slowing on the same frame.
pub(super) fn tick(
    run: Res<Run>,
    world: super::Floor,
    heroes: Query<&Transform, With<Hero>>,
    enemies: Query<(&Transform, &Enemy)>,
    perception: super::perception::Perception,
    mut channels: ResMut<Channels>,
    mut hits: MessageWriter<Hit>,
) {
    let pulses = channels.advance(run.elapsed, run.alive(), |caster, target, range| {
        let (Ok(hero), Ok((victim, enemy))) = (heroes.get(caster), enemies.get(target)) else {
            return false;
        };
        !enemy.dead
            && enemy.hp > 0.0
            && super::hero::reach(hero.translation, victim.translation) <= range + enemy.radius
            && perception.visible(Some(caster), target, run.elapsed)
            && world.line_clear(hero.translation, victim.translation)
    });
    hits.write_batch(pulses);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active(caster: Entity, target: Entity) -> ActiveChannel {
        ActiveChannel {
            caster,
            clock: Channel::new([target], 0.0, 1.0, 3).unwrap(),
            damage: 25.0,
            range: 20.0,
            slow: 0.5,
            source: "mind_flay",
        }
    }

    fn physics(app: &mut App) {
        use avian3d::prelude::*;
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            PhysicsPlugins::new(PostUpdate),
        ));
        app.init_asset::<Mesh>();
        app.init_resource::<benilla_world::collision::MoverTraceExclusions>();
        app.init_resource::<Channels>();
    }

    #[test]
    fn acquired_channel_requires_training_and_never_overlaps_or_hits_immediately() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, target, _) =
            super::super::abilities::poison_tests::fixture("mind_flay", 1000.0);
        physics(&mut app);
        app.finish();
        app.cleanup();
        app.update();
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * 4.0;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &super::super::data::CLASSES[4];
            run.milestones = survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            };
        }
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert!(app.world().resource::<Channels>().active.is_empty());
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = run.milestones.with_ranks(&[(2, 501, 1)]);
            run.abilities[0].cd = 0.0;
            run.bonus.crit = 1.0;
        }
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert_eq!(app.world().resource::<Channels>().active.len(), 1);
        assert_eq!(app.world().resource::<Messages<Hit>>().len(), 0);
        assert_eq!(
            app.world().resource::<Channels>().active[0].clock.targets(),
            &[target]
        );
        app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.0;
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert_eq!(app.world().resource::<Channels>().active.len(), 1);
    }

    #[test]
    fn acquisition_skips_blocked_victims_and_preserves_cooldown_until_clear() {
        use avian3d::prelude::*;
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, target, _) =
            super::super::abilities::poison_tests::fixture("mind_flay", 1000.0);
        physics(&mut app);
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * 4.0;
        let wall = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::cuboid(1.0, 4.0, 4.0),
                Transform::from_xyz(2.0, 1.0, 0.0),
            ))
            .id();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &super::super::data::CLASSES[4];
            run.milestones = survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(2, 501, 1)]);
        }
        app.finish();
        app.cleanup();
        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert!(app.world().resource::<Channels>().active.is_empty());
        // Use the ordinary short no-target retry, not the four-second spell recovery.
        assert_eq!(app.world().resource::<Run>().abilities[0].cd, 0.2);

        // The nearer blocked victim does not occupy the sole target slot.
        let (mut donor, donor_target, _) =
            super::super::abilities::poison_tests::fixture("mind_flay", 1000.0);
        let enemy = donor
            .world_mut()
            .entity_mut(donor_target)
            .take::<Enemy>()
            .unwrap();
        let farther = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 6.0), enemy))
            .id();
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert_eq!(
            app.world().resource::<Channels>().active[0].clock.targets(),
            &[farther]
        );
        assert!(app.world().resource::<Run>().abilities[0].cd > 0.0);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());

        app.world_mut().resource_mut::<Channels>().active.clear();
        app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.0;
        app.world_mut().despawn(wall);
        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert_eq!(
            app.world().resource::<Channels>().active[0].clock.targets(),
            &[target]
        );
    }

    #[test]
    fn runtime_checks_range_concealment_and_walls_before_paying_damage() {
        use avian3d::prelude::*;
        use bevy::ecs::system::RunSystemOnce;
        for interruption in ["none", "range", "concealment", "wall", "caster"] {
            let (mut app, target, mut settle) =
                super::super::abilities::poison_tests::fixture("instant_poison", 1000.0);
            app.add_plugins((
                MinimalPlugins,
                TransformPlugin,
                AssetPlugin::default(),
                bevy::scene::ScenePlugin,
                PhysicsPlugins::new(PostUpdate),
            ));
            app.init_asset::<Mesh>();
            app.init_resource::<benilla_world::collision::MoverTraceExclusions>();
            app.init_resource::<Channels>();
            let caster = app
                .world_mut()
                .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()))
                .id();
            app.world_mut()
                .get_mut::<Transform>(target)
                .unwrap()
                .translation = Vec3::X * 4.0;
            app.world_mut()
                .resource_mut::<Channels>()
                .active
                .push(active(caster, target));
            if interruption == "wall" {
                app.world_mut().spawn((
                    RigidBody::Static,
                    Collider::cuboid(1.0, 4.0, 4.0),
                    Transform::from_xyz(2.0, 1.0, 0.0),
                ));
            }
            app.finish();
            app.cleanup();
            app.update();
            app.world_mut().run_system_once(tick).unwrap();
            if interruption == "range" {
                app.world_mut()
                    .get_mut::<Transform>(target)
                    .unwrap()
                    .translation = Vec3::X * 30.0;
            } else if interruption == "concealment" {
                app.world_mut()
                    .entity_mut(target)
                    .insert(super::super::perception::Concealed);
            } else if interruption == "caster" {
                app.world_mut().despawn(caster);
            }
            app.world_mut().resource_mut::<Run>().elapsed = 1.0;
            app.world_mut().run_system_once(tick).unwrap();
            settle.run(app.world_mut());
            assert_eq!(
                app.world().get::<Enemy>(target).unwrap().hp,
                if interruption == "none" {
                    975.0
                } else {
                    1000.0
                },
                "{interruption}"
            );
            assert_eq!(
                app.world().resource::<Channels>().movement(target, 1.0),
                if interruption == "none" { 0.5 } else { 1.0 },
                "{interruption}"
            );
        }
    }

    #[test]
    fn paid_ticks_compose_shadow_training_and_heal_only_removed_health() {
        let (mut app, target, mut settle) =
            super::super::abilities::poison_tests::fixture("mind_flay", 100.0);
        let caster = app.world_mut().spawn_empty().id();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &super::super::data::CLASSES[4];
            run.bonus.leech = 0.0;
            run.bonus.multistrike = 1.0;
            run.milestones = survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(2, 501, 1), (2, 461, 5), (2, 484, 1), (2, 464, 5)]);
        }
        let mut channels = Channels::default();
        channels.active.push(active(caster, target));
        assert!(channels.advance(0.0, true, |_, _, _| true).is_empty());
        for time in [1.0, 2.0] {
            app.world_mut().resource_mut::<Run>().elapsed = time;
            let hits = channels.advance(time, true, |_, _, _| true);
            assert_eq!(hits.len(), 1);
            assert!(!hits[0].crit);
            app.world_mut()
                .resource_mut::<Messages<Hit>>()
                .write_batch(hits);
            settle.run(app.world_mut());
            if time == 1.0 {
                let enemy = app.world().get::<Enemy>(target).unwrap();
                assert_eq!(enemy.hp, 50.0); // One guaranteed multistrike copy.
                assert_eq!(enemy.stun_t, 3.0); // Seed 0 forces the Blackout roll.
                assert!((app.world().resource::<Run>().hp - 60.0).abs() < 0.001);
            }
        }
        let enemy = app.world().get::<Enemy>(target).unwrap();
        assert!(enemy.dead);
        assert_eq!(enemy.hp, 0.0); // Death clamps health; the meter retains raw damage.
        assert!((app.world().resource::<Run>().damage_done - 101.5).abs() < 0.001);
        assert!((app.world().resource::<Run>().hp - 70.0).abs() < 0.001);
        assert!(channels.advance(3.0, true, |_, _, _| false).is_empty());
        assert!(channels.active.is_empty());
        assert_eq!(channels.movement(target, 1.0), 1.0);
    }

    #[test]
    fn cast_snapshot_survives_infusion_expiry_and_paused_active_time() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, target, mut settle) =
            super::super::abilities::poison_tests::fixture("mind_flay", 1000.0);
        physics(&mut app);
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * 22.0;
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &super::super::data::CLASSES[4];
            run.bonus.leech = 0.0;
            run.elapsed = 14.5;
            run.milestones = survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(2, 501, 1), (2, 463, 5), (0, 322, 1)]);
            let talents = run.milestones;
            run.power_infusion.consume(talents, "smite", 0.0);
        }
        app.finish();
        app.cleanup();
        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        assert!(app.world().resource::<Channels>().active.is_empty()); // Outside base range.
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones =
                run.milestones
                    .with_ranks(&[(2, 501, 1), (2, 463, 5), (0, 322, 1), (2, 881, 3)]);
            run.abilities[0].cd = 0.0;
        }
        app.world_mut()
            .run_system_once(super::super::abilities::cast_abilities)
            .unwrap();
        let channel = &app.world().resource::<Channels>().active[0];
        assert!((channel.damage - 33.0).abs() < 0.001); // Focus 10%, then Infusion 20%.
        assert!((channel.range - 24.0).abs() < 0.001);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());

        let mut playing = Schedule::default();
        playing.add_systems(tick.run_if(in_state(super::super::Phase::Playing)));
        app.world_mut()
            .insert_resource(State::new(super::super::Phase::Paused));
        // Wall time may pass in a pause; active run time stays fixed and no pulses settle.
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs(10));
            playing.run(app.world_mut());
            settle.run(app.world_mut());
        }
        assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 1000.0);
        assert_eq!(app.world().resource::<Channels>().active.len(), 1);
        app.world_mut()
            .insert_resource(State::new(super::super::Phase::Playing));
        // A late frame catches all three ticks after the cast-time buff has expired.
        app.world_mut().resource_mut::<Run>().elapsed = 17.5;
        {
            let run = app.world().resource::<Run>();
            assert_eq!(
                run.power_infusion.active_bonus(run.milestones, run.elapsed),
                0.0
            );
        }
        playing.run(app.world_mut());
        assert_eq!(app.world().resource::<Messages<Hit>>().len(), 3);
        settle.run(app.world_mut());
        assert!((app.world().get::<Enemy>(target).unwrap().hp - 901.0).abs() < 0.001);
        assert!(app.world().resource::<Channels>().active.is_empty());
        playing.run(app.world_mut());
        settle.run(app.world_mut());
        assert!((app.world().get::<Enemy>(target).unwrap().hp - 901.0).abs() < 0.001);
    }

    #[test]
    fn slow_is_immediate_and_cancellation_preserves_unrelated_snares() {
        let mut world = World::new();
        let caster = world.spawn_empty().id();
        let target = world.spawn_empty().id();
        let mut channels = Channels::default();
        channels.active.push(active(caster, target));
        assert!(channels.advance(0.0, true, |_, _, _| true).is_empty());
        assert_eq!(channels.movement(target, 1.0), 0.5);
        assert_eq!(channels.movement(target, 0.2), 0.2);
        assert!(channels.advance(0.5, true, |_, _, _| false).is_empty());
        assert_eq!(channels.movement(target, 0.8), 0.8);
        assert!(channels.advance(3.0, true, |_, _, _| true).is_empty());
    }

    #[test]
    fn multiple_casters_keep_independent_channels_and_death_cancels_all() {
        let mut world = World::new();
        let first = world.spawn_empty().id();
        let second = world.spawn_empty().id();
        let target = world.spawn_empty().id();
        let mut channels = Channels::default();
        channels
            .active
            .extend([active(first, target), active(second, target)]);
        let hits = channels.advance(1.0, true, |caster, _, _| caster == second);
        assert_eq!(hits.len(), 1);
        assert_eq!(
            (hits[0].damage, hits[0].source, hits[0].target),
            (25.0, "mind_flay", target)
        );
        assert!(!hits[0].crit);
        assert_eq!(channels.movement(target, 1.0), 0.5);
        assert!(channels.advance(1.0, true, |_, _, _| true).is_empty());
        assert!(channels.advance(2.0, false, |_, _, _| true).is_empty());
        assert!(channels.active.is_empty());
        assert_eq!(channels.movement(target, 1.0), 1.0);
    }

    #[test]
    fn final_tick_releases_slow_without_repeating_damage() {
        let mut world = World::new();
        let caster = world.spawn_empty().id();
        let target = world.spawn_empty().id();
        let mut channels = Channels::default();
        channels.active.push(active(caster, target));
        assert_eq!(channels.advance(3.0, true, |_, _, _| true).len(), 3);
        assert_eq!(channels.movement(target, 1.0), 1.0);
        assert!(channels.advance(4.0, true, |_, _, _| true).is_empty());
    }
}
