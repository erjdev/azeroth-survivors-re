//! Keeps the scenery aperture on the active hero without changing scenery collision.
use super::{
    camera::CameraSettings,
    hero::{Hero, CAMERA_FOCUS_HEIGHT},
    Phase,
};
use benilla_world::{
    interact::{PickBlob, WorldObject, WorldPick},
    model_render::ModelKind,
    survivors_visibility::SceneryCutaway,
    view::WorldCamera,
};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

// Diameter as a fraction of the shorter viewport side, independent of UI/render scaling.
const DIAMETER_FRACTION: f32 = 0.35;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(benilla_world::survivors_visibility::plugin)
        .add_systems(
            PostUpdate,
            sync.after(bevy::transform::TransformSystems::Propagate)
                .after(bevy::camera::CameraUpdateSystems)
                .after(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
        );
}

#[derive(Default)]
struct VisualProbe {
    elapsed: f32,
    blocked: bool,
}

fn scenery(kind: ModelKind) -> bool {
    matches!(
        kind,
        ModelKind::Doodad | ModelKind::Wmo | ModelKind::GameObject
    )
}

/// Render triangles include leaf canopies omitted from walking/camera collision hulls.
/// WorldPick covers ordinary entities, merged blobs and the retained static renderer.
fn visually_blocked(world: &WorldPick, pickable: &HashSet<Entity>, eye: Vec3, focus: Vec3) -> bool {
    let Ok((direction, distance)) = Dir3::new_and_length(focus - eye) else {
        return false;
    };
    world
        .cast(Ray3d::new(eye, direction), pickable, true)
        .iter()
        .any(|hit| {
            hit.hit.distance < distance - 0.5
                && hit
                    .object
                    .as_ref()
                    .is_some_and(|object| scenery(object.kind))
        })
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync(
    phase: Res<State<Phase>>,
    time: Res<Time<Real>>,
    world: WorldPick,
    objects: Query<(Entity, Option<&WorldObject>), Or<(With<WorldObject>, With<PickBlob>)>>,
    mut probe: Local<VisualProbe>,
    mut fade: Local<survivors_core::visibility::CutawayFade>,
    settings: Res<CameraSettings>,
    hero: Query<&GlobalTransform, With<Hero>>,
    camera: Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut cutaway: ResMut<SceneryCutaway>,
) {
    *cutaway = SceneryCutaway::default();
    if !settings.cutaway || matches!(phase.get(), Phase::Menu | Phase::GameOver) {
        *fade = default();
        *probe = default();
        return;
    }
    let (Ok(hero), Ok((camera, projection))) = (hero.single(), camera.single()) else {
        *fade = default();
        *probe = default();
        return;
    };
    let focus = hero.translation() + Vec3::Y * CAMERA_FOCUS_HEIGHT;
    let eye = camera.translation();
    if focus.is_finite() && eye.is_finite() {
        let Projection::Perspective(projection) = projection else {
            *fade = default();
            return;
        };
        let radius = survivors_core::visibility::perspective_radius(
            eye.distance(focus),
            projection.fov,
            projection.aspect_ratio,
            DIAMETER_FRACTION,
        );
        cutaway.focus = focus.extend(radius);
        probe.elapsed -= time.delta_secs();
        if probe.elapsed <= 0.0 {
            let pickable = objects
                .iter()
                .filter_map(|(entity, object)| {
                    object
                        .is_none_or(|object| scenery(object.kind))
                        .then_some(entity)
                })
                .collect();
            probe.blocked = visually_blocked(&world, &pickable, eye, focus);
            probe.elapsed = 0.1;
        }
        cutaway.camera = eye.extend(fade.advance(probe.blocked, time.delta_secs()));
    } else {
        *fade = default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_world::interact::PickMesh;
    use bevy::ecs::system::RunSystemOnce;
    use std::sync::Arc;

    #[test]
    fn rendered_foliage_without_colliders_blocks_only_in_front_of_hero() {
        let mut world = World::new();
        use bevy::camera::visibility::SetViewVisibility;
        // Bevy triangle across the sightline at z=5, expressed in WoW model coordinates.
        let entity = world
            .spawn((
                PickMesh(Arc::new(benilla_formats::RenderSubmesh {
                    positions: vec![[-5., 2., -2.], [-5., -2., -2.], [-5., 0., 2.]],
                    indices: vec![0, 1, 2],
                    ..default()
                })),
                GlobalTransform::IDENTITY,
                ViewVisibility::HIDDEN,
                WorldObject {
                    kind: ModelKind::Doodad,
                    label: "Leaf canopy".into(),
                    id: 1,
                    detail: String::new(),
                },
            ))
            .id();
        world
            .get_mut::<ViewVisibility>(entity)
            .unwrap()
            .set_visible();
        let check = move |pick: WorldPick| {
            visually_blocked(&pick, &HashSet::from([entity]), Vec3::ZERO, Vec3::Z * 10.)
        };
        assert!(
            world.run_system_once(check).unwrap(),
            "render-only leaves activate aperture"
        );
        *world.get_mut::<GlobalTransform>(entity).unwrap() =
            GlobalTransform::from_translation(Vec3::Z * 10.);
        assert!(
            !world.run_system_once(check).unwrap(),
            "scenery behind hero does not activate aperture"
        );
        *world.get_mut::<GlobalTransform>(entity).unwrap() =
            GlobalTransform::from_translation(Vec3::X * 10.);
        assert!(
            !world.run_system_once(check).unwrap(),
            "scenery beside sightline does not activate aperture"
        );
        *world.get_mut::<GlobalTransform>(entity).unwrap() = GlobalTransform::IDENTITY;
        world.get_mut::<WorldObject>(entity).unwrap().kind = ModelKind::Creature;
        assert!(
            !world.run_system_once(check).unwrap(),
            "another unit does not cut away scenery"
        );
    }
}
