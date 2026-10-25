//! Opt-in Survivors visibility aperture, uploaded once per frame for both scenery paths.
use crate::lighting::SharedLightBuffer;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::renderer::RenderQueue;
use bevy::render::{Render, RenderApp, RenderSystems};

/// Appended after the fixed-capacity palette; off-world light buffers leave this tail zeroed.
pub(crate) const TAIL_BYTES: u64 = 32;

/// Focus position/radius and camera position/reveal strength. A zero radius disables the aperture.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct SceneryCutaway {
    pub focus: Vec4,
    pub camera: Vec4,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SceneryCutaway>()
        .add_plugins(ExtractResourcePlugin::<SceneryCutaway>::default());
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(Render, upload.in_set(RenderSystems::PrepareResources));
    }
}

fn upload(
    queue: Res<RenderQueue>,
    buffer: Option<Res<SharedLightBuffer>>,
    cutaway: Res<SceneryCutaway>,
) {
    let Some(buffer) = buffer else { return };
    let rows = [cutaway.focus.to_array(), cutaway.camera.to_array()];
    queue.write_buffer(
        &buffer.0,
        buffer.0.size() - TAIL_BYTES,
        bytemuck::cast_slice(&rows),
    );
}

/// Test only the segment in front of the hero, against scenery camera/LOS faces.
/// Geometry behind the player cannot activate the reveal.
pub fn hero_occluded(spatial: &avian3d::prelude::SpatialQuery, eye: Vec3, focus: Vec3) -> bool {
    let ray = focus - eye;
    let Ok(direction) = Dir3::new(ray) else {
        return false;
    };
    let distance = ray.length() - 0.5;
    distance > 0.0
        && spatial
            .cast_ray(
                eye,
                direction,
                distance,
                true,
                &crate::collision::WorldCollision::camera_filter(),
            )
            .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use avian3d::prelude::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn only_scenery_between_camera_and_hero_triggers_cutaway() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            PhysicsPlugins::new(PostUpdate),
        ));
        app.init_asset::<Mesh>();
        let wall = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::cuboid(4., 4., 1.),
                crate::collision::camera_layers(),
                Transform::from_xyz(0., 0., 12.),
            ))
            .id();
        app.finish();
        app.cleanup();
        app.update();
        let check = |spatial: SpatialQuery| hero_occluded(&spatial, Vec3::ZERO, Vec3::Z * 10.);
        assert!(
            !app.world_mut().run_system_once(check).unwrap(),
            "building behind hero"
        );
        app.world_mut()
            .get_mut::<Transform>(wall)
            .unwrap()
            .translation
            .z = 5.;
        app.update();
        assert!(
            app.world_mut().run_system_once(check).unwrap(),
            "building in front of hero"
        );
        app.world_mut()
            .get_mut::<Transform>(wall)
            .unwrap()
            .translation
            .x = 5.;
        app.update();
        assert!(
            !app.world_mut().run_system_once(check).unwrap(),
            "nearby building outside sightline"
        );
    }
}
