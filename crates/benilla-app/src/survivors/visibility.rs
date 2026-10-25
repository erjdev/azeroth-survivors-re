//! Keeps the scenery aperture on the active hero without changing scenery collision.
use super::{camera::CameraSettings, hero::Hero, Phase};
use benilla_world::{survivors_visibility::SceneryCutaway, view::WorldCamera};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(benilla_world::survivors_visibility::plugin)
        .add_systems(
            PostUpdate,
            sync.after(bevy::transform::TransformSystems::Propagate),
        );
}

fn sync(
    phase: Res<State<Phase>>,
    settings: Res<CameraSettings>,
    hero: Query<&GlobalTransform, With<Hero>>,
    camera: Query<&GlobalTransform, With<WorldCamera>>,
    mut cutaway: ResMut<SceneryCutaway>,
) {
    *cutaway = SceneryCutaway::default();
    if !settings.cutaway || matches!(phase.get(), Phase::Menu | Phase::GameOver) {
        return;
    }
    let (Ok(hero), Ok(camera)) = (hero.single(), camera.single()) else {
        return;
    };
    let focus = hero.translation() + Vec3::Y;
    let eye = camera.translation();
    if focus.is_finite() && eye.is_finite() {
        cutaway.focus = focus.extend(3.0);
        cutaway.camera = eye.extend(0.25);
    }
}
