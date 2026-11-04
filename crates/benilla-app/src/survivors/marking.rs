//! Gameplay marks have their own caster/expiry state, not the visual aura's slots.
use super::{enemies::Enemy, hero::Hero, Run};
use bevy::prelude::*;

#[derive(Component, Default)]
pub(super) struct Marked(pub survivors_core::marked_target::HunterMark<Entity>);

pub(super) fn prune(
    mut commands: Commands,
    run: Res<Run>,
    heroes: Query<(), With<Hero>>,
    targets: Query<(Entity, &Enemy, &Marked)>,
) {
    for (entity, enemy, mark) in &targets {
        if enemy.dead
            || !run.alive()
            || mark.0.remaining(run.elapsed) <= 0.0
            || !mark
                .0
                .caster()
                .is_some_and(|caster| heroes.contains(caster))
        {
            commands.entity(entity).remove::<Marked>();
        }
    }
}

/// These two authored attacks use the installed normalized ranged weapon effect.
/// Their 2.8-second normalization contributes 0.2 damage per point of attack power.
pub(super) fn coefficient(source: &str) -> f32 {
    if matches!(source, "aimed_shot" | "multi_shot") {
        0.2
    } else {
        0.0
    }
}
