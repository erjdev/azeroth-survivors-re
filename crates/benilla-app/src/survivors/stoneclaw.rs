//! Destructible Stoneclaw with forced attention for captured ordinary enemies.
use super::{units, Floor, Run};
use crate::net::ObjectStore;
use bevy::prelude::*;
use std::collections::HashSet;
use survivors_core::stoneclaw as rules;

pub(super) const BODY_RADIUS: f32 = 0.4;

#[derive(Component)]
pub(super) struct Stoneclaw {
    pub life: rules::Stoneclaw,
    pub radius: f32,
    pub captured: HashSet<Entity>,
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
    let Some(life) = rules::Stoneclaw::deploy(run.elapsed, rules::BASE_HEALTH) else {
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
    commands.entity(entity).insert(Stoneclaw {
        life,
        radius,
        captured: HashSet::new(),
    });
    true
}

pub(super) fn tick(
    mut commands: Commands,
    run: Res<Run>,
    mut totems: Query<(Entity, &mut Stoneclaw, &mut ObjectStore)>,
) {
    for (entity, mut totem, mut store) in &mut totems {
        let alive = totem.life.advance(run.elapsed);
        store
            .0
            .merge(benilla_protocol::messages::ObjectFields::from_pairs(&[
                (22, totem.life.health().ceil() as u32),
                (28, totem.life.maximum_health().ceil() as u32),
            ]));
        if !run.alive() || !alive {
            units::kill_unit(&mut store);
            commands
                .entity(entity)
                .remove::<Stoneclaw>()
                .insert(benilla_world::model_fade::DespawnFade::default());
        }
    }
}
