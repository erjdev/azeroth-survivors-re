//! Floors near a survivor's feet, including WMO interiors and bridges.
//!
//! A bounded probe preserves the authored/current height while geometry streams and avoids
//! selecting terrain far below an interior. This is grounding, not wall collision or pathfinding.

use benilla_world::collision::WorldCollision;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

const STEP_UP: f32 = 2.0;
const DROP: f32 = 8.0;

#[derive(SystemParam)]
pub(super) struct Floor<'w, 's> {
    collision: WorldCollision<'w, 's>,
}

impl Floor<'_, '_> {
    /// The nearest body-collidable floor within two yards above/eight below the feet.
    /// Callers keep their last/authored height when the geometry is not available.
    pub fn height(&self, feet: Vec3) -> Option<f32> {
        let origin = feet + Vec3::Y * STEP_UP;
        self.collision
            .ray_body(origin, Dir3::NEG_Y, STEP_UP + DROP)
            .filter(|hit| hit.normal.y > 0.5)
            .map(|hit| origin.y - hit.distance)
    }
}

/// Keep the camera's sphere inside the visible room, along its requested boom.
/// Geometry can shorten the boom without changing the player's preferred zoom.
pub(super) fn camera_eye(collision: &WorldCollision, look: Vec3, desired: Vec3) -> Vec3 {
    let arm = desired - look;
    let length = arm.length();
    if length <= f32::EPSILON {
        return desired;
    }
    match collision.cast_camera(look, arm, false) {
        Some(distance) => look + arm / length * (distance - 0.05).clamp(length.min(0.01), length),
        None => desired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avian3d::prelude::*;
    use benilla_world::collision::MoverTraceExclusions;
    use bevy::ecs::system::RunSystemOnce;

    fn fixture(heights: &[f32]) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            PhysicsPlugins::new(PostUpdate),
        ));
        app.init_asset::<Mesh>();
        app.init_resource::<MoverTraceExclusions>();
        for &y in heights {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::trimesh(
                    vec![
                        Vec3::new(-5., y, -5.),
                        Vec3::new(5., y, -5.),
                        Vec3::new(5., y, 5.),
                        Vec3::new(-5., y, 5.),
                    ],
                    vec![[0, 2, 1], [0, 3, 2]],
                ),
                Transform::default(),
            ));
        }
        app.finish();
        app.cleanup();
        app.update();
        app
    }

    fn height(app: &mut App, y: f32) -> Option<f32> {
        app.world_mut()
            .run_system_once(move |floor: Floor| floor.height(Vec3::Y * y))
            .unwrap()
    }

    #[test]
    fn camera_stays_below_a_ceiling_and_keeps_clear_booms_unchanged() {
        let mut app = fixture(&[0., 4.]);
        let eye = app
            .world_mut()
            .run_system_once(|collision: WorldCollision| {
                camera_eye(&collision, Vec3::Y * 1.4, Vec3::Y * 20.)
            })
            .unwrap();
        assert!(
            eye.y > 3.5 && eye.y < 3.75,
            "camera {eye:?} must fit below ceiling"
        );
        let desired = Vec3::new(2., 2., 0.);
        let clear = app
            .world_mut()
            .run_system_once(move |collision: WorldCollision| {
                camera_eye(&collision, Vec3::Y * 1.4, desired)
            })
            .unwrap();
        assert_eq!(clear, desired);
    }

    #[test]
    fn interior_floor_wins_over_terrain_and_ignores_the_storey_above() {
        let mut app = fixture(&[0., 30., 40.]);
        assert_eq!(height(&mut app, 30.), Some(30.));
        assert_eq!(height(&mut app, 31.), Some(30.));
        assert_eq!(height(&mut app, 40.), Some(40.));
    }

    #[test]
    fn missing_interior_does_not_snap_to_distant_terrain() {
        let mut app = fixture(&[0.]);
        assert_eq!(height(&mut app, 30.), None);
        assert_eq!(height(&mut app, 1.), Some(0.));
    }
}
