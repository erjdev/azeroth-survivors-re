//! Gameplay concealment is independent of scenery occlusion and camera cutaways.
use super::{hero::Hero, marking::Marked, Run};
use bevy::{ecs::system::SystemParam, prelude::*};

/// An authored effect may hide an enemy; ordinary enemies have no such component.
#[derive(Component)]
pub(super) struct Concealed;

#[derive(SystemParam)]
pub(super) struct Perception<'w, 's> {
    concealed: Query<'w, 's, (), With<Concealed>>,
    marks: Query<'w, 's, &'static Marked>,
}

impl Perception<'_, '_> {
    pub(super) fn has_concealed(&self) -> bool {
        !self.concealed.is_empty()
    }

    pub(super) fn visible(&self, observer: Option<Entity>, target: Entity, now: f32) -> bool {
        !self.concealed.contains(target)
            || observer.is_some_and(|observer| {
                self.marks
                    .get(target)
                    .is_ok_and(|mark| mark.0.reveals_to(observer, now))
            })
    }
}

/// Remember the root's original setting, including an independently hidden root.
#[derive(Component)]
pub(super) struct ConcealmentVisibility(Visibility);

pub(super) fn sync(
    mut commands: Commands,
    run: Res<Run>,
    hero: Query<Entity, With<Hero>>,
    perception: Perception,
    mut enemies: Query<
        (Entity, &mut Visibility, Option<&ConcealmentVisibility>),
        With<super::enemies::Enemy>,
    >,
) {
    let observer = run.alive().then(|| hero.single().ok()).flatten();
    for (entity, mut visibility, saved) in &mut enemies {
        if perception.visible(observer, entity, run.elapsed) {
            if let Some(saved) = saved {
                *visibility = saved.0;
                commands.entity(entity).remove::<ConcealmentVisibility>();
            }
        } else {
            if saved.is_none() {
                commands
                    .entity(entity)
                    .insert(ConcealmentVisibility(*visibility));
            }
            *visibility = Visibility::Hidden;
        }
    }
}
