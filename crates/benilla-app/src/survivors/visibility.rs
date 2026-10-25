//! Keeps the scenery aperture on the active hero without changing scenery collision.
use super::{
    camera::CameraSettings,
    hero::{Hero, CAMERA_FOCUS_HEIGHT},
    Phase,
};
use benilla_world::{survivors_visibility::SceneryCutaway, view::WorldCamera};
use bevy::prelude::*;

// Diameter as a fraction of the shorter viewport side, independent of UI/render scaling.
const DIAMETER_FRACTION: f32 = 0.35;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(benilla_world::survivors_visibility::plugin)
        .add_systems(
            PostUpdate,
            sync.after(bevy::transform::TransformSystems::Propagate)
                .after(bevy::camera::CameraUpdateSystems),
        );
}

fn sync(
    phase: Res<State<Phase>>,
    time: Res<Time<Real>>,
    spatial: avian3d::prelude::SpatialQuery,
    mut fade: Local<survivors_core::visibility::CutawayFade>,
    settings: Res<CameraSettings>,
    hero: Query<&GlobalTransform, With<Hero>>,
    camera: Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut cutaway: ResMut<SceneryCutaway>,
) {
    *cutaway = SceneryCutaway::default();
    if !settings.cutaway || matches!(phase.get(), Phase::Menu | Phase::GameOver) {
        *fade = default();
        return;
    }
    let (Ok(hero), Ok((camera, projection))) = (hero.single(), camera.single()) else {
        *fade = default();
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
        let occluded = benilla_world::survivors_visibility::hero_occluded(&spatial, eye, focus);
        cutaway.camera = eye.extend(fade.advance(occluded, time.delta_secs()));
    } else {
        *fade = default();
    }
}
