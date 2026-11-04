//! Destructible, stationary Earthbind; its pulse clock lives in the shared core.
use super::{enemies::Enemy, hero::Hero, units, Floor, Run};
use crate::net::ObjectStore;
use bevy::prelude::*;
use survivors_core::{earthbind as rules, slows::Source};

pub(super) const BODY_RADIUS: f32 = 0.4;

#[derive(Component)]
pub(super) struct Earthbind {
    pub life: rules::Earthbind,
    pub radius: f32,
}

pub(super) fn deploy(
    commands: &mut Commands,
    run: &mut Run,
    floor: &Floor,
    feet: Vec3,
    radius: f32,
    previous: impl Iterator<Item = Entity>,
) -> bool {
    if !radius.is_finite() || radius <= 0.0 || !run.alive() {
        return false;
    }
    let Some(life) = rules::Earthbind::deploy(run.elapsed) else {
        return false;
    };
    let Some(y) = floor.height(feet) else {
        return false;
    };
    let at = Vec3::new(feet.x, y, feet.z);
    if !floor.can_spawn_from(feet, at, BODY_RADIUS) {
        return false;
    }
    for old in previous {
        commands.entity(old).despawn();
    }
    let entity = units::spawn_unit(
        commands,
        &units::UnitSpec {
            serial: run.guid(),
            display: 4588,
            scale: 1.0,
            level: 60,
            faction: units::FACTION_FRIENDLY,
            bytes0: 0,
            weapons: &[],
            sheath: 0,
        },
        at,
        0.0,
    );
    commands.entity(entity).insert(Earthbind { life, radius });
    true
}

pub(super) fn tick(
    mut commands: Commands,
    run: Res<Run>,
    mut totems: Query<(Entity, &Transform, &mut Earthbind, &mut ObjectStore)>,
    mut enemies: Query<(&Transform, &mut Enemy), Without<Hero>>,
) {
    for (entity, transform, mut totem, mut store) in &mut totems {
        let now = run.elapsed;
        if run.alive() {
            for at in totem.life.pulses(now) {
                // A delayed observation must not resurrect an already-expired slow.
                if at + rules::SLOW_SECONDS <= now {
                    continue;
                }
                for (target, mut enemy) in &mut enemies {
                    if !enemy.dead
                        && target.translation.distance_squared(transform.translation)
                            <= totem.radius * totem.radius
                    {
                        let speed = if enemy.boss { 0.75 } else { rules::SLOW_SPEED };
                        enemy.slows.apply(
                            Source::Spell("earthbind_totem"),
                            speed,
                            rules::SLOW_SECONDS,
                            at,
                        );
                    }
                }
            }
        }
        store
            .0
            .merge(benilla_protocol::messages::ObjectFields::from_pairs(&[
                (22, totem.life.health().ceil() as u32),
                (28, rules::HEALTH as u32),
            ]));
        if !run.alive() || !totem.life.alive(now) {
            units::kill_unit(&mut store);
            commands
                .entity(entity)
                .remove::<Earthbind>()
                .insert(benilla_world::model_fade::DespawnFade::default());
        }
    }
}
