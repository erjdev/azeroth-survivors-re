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
