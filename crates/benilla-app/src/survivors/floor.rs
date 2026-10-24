//! Floors near a survivor's feet, including WMO interiors and bridges.
//!
//! A bounded probe preserves the authored/current height while geometry streams and avoids
//! selecting terrain far below an interior. Body sweeps handle walls; route finding is separate.

use avian3d::character_controller::move_and_slide::{MoveAndSlideConfig, MoveAndSlideHitResponse};
use avian3d::prelude::Collider;
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
    /// Sweep the Survivors body horizontally, sliding along room geometry. Body radius is
    /// shared with horde separation; the capsule's cylindrical section is one yard tall.
    pub fn slide(&self, feet: Vec3, desired: Vec3, radius: f32) -> Vec3 {
        let movement = super::units::flat(desired - feet);
        if movement.length_squared() <= 1e-10 {
            return feet;
        }
        let lift = Vec3::Y * (radius + 0.5 + 0.03);
        let body = Collider::capsule(radius, 1.0);
        let result = self.collision.slide_body(
            &body,
            feet + lift,
            movement,
            std::time::Duration::from_secs(1),
            &MoveAndSlideConfig::default(),
            |_| MoveAndSlideHitResponse::Accept,
        );
        result.position - lift
    }

    /// A straight mobility sweep stops at the first obstacle instead of sliding around it.
    pub fn travel(&self, feet: Vec3, desired: Vec3, radius: f32) -> Vec3 {
        let movement = super::units::flat(desired - feet);
        let length = movement.length();
        if length <= f32::EPSILON {
            return feet;
        }
        let body = Collider::capsule(radius, 1.0);
        let lift = Vec3::Y * (radius + 0.5 + 0.03);
        let at = match self.collision.cast_body(&body, feet + lift, movement, 0.02) {
            Some(hit) => feet + movement / length * (hit.distance - 0.02).clamp(0.0, length),
            None => feet + movement,
        };
        self.height(at).map_or(feet, |y| Vec3::new(at.x, y, at.z))
    }

    /// Solid-world line of sight at one yard above the feet, for contact effects.
    pub fn line_clear(&self, from: Vec3, to: Vec3) -> bool {
        let ray = to - from;
        let Ok(direction) = Dir3::new(ray) else {
            return true;
        };
        self.collision
            .ray_body(from + Vec3::Y, direction, ray.length())
            .is_none()
    }

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
    fn authored_elwynn_tree_hulls_block_survivors_from_both_sides() {
        // Explicit data only: never discover an installed game for this regression.
        let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
            eprintln!("SKIP authored Elwynn hull test: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        for model in [
            "canopylesstree01",
            "elwynntreecanopy01",
            "elwynntreecanopy02",
            "elwynntreecanopy03",
            "elwynntreecanopy04",
            "elwynntreemid01",
        ] {
            let path = format!("World/azeroth/elwynn/passivedoodads/trees/{model}.m2");
            let hull = benilla_formats::load_m2_collision_hull(&mut chain, &path).unwrap();
            assert!(!hull.is_empty(), "{model} has no usable source hull");
            // Bake the same coordinate basis as the streamer's welded static hulls.
            let vertices = hull
                .positions
                .iter()
                .map(|p| benilla_assets::coords::wow_to_bevy(*p))
                .collect();
            let indices = hull.indices.as_chunks::<3>().0.to_vec();
            let mut app = fixture(&[0.]);
            // Mobility requires supported destinations even after the tree unloads.
            for mut transform in app
                .world_mut()
                .query::<&mut Transform>()
                .iter_mut(app.world_mut())
            {
                transform.scale = Vec3::splat(10.0);
            }
            let tree = app
                .world_mut()
                .spawn((
                    RigidBody::Static,
                    Collider::trimesh(vertices, indices),
                    Transform::default(),
                ))
                .id();
            app.update();
            app.update();
            for axis in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
                let from = axis * 12.0;
                let desired = -from;
                let (walk, dash) = app
                    .world_mut()
                    .run_system_once(move |floor: Floor| {
                        (
                            floor.slide(from, desired, 0.6),
                            floor.travel(from, desired, 0.6),
                        )
                    })
                    .unwrap();
                assert!(
                    walk.distance(desired) > 0.5,
                    "{model}: walk crossed trunk from {from:?}: {walk:?}"
                );
                assert!(
                    dash.dot(axis) > 0.0,
                    "{model}: mobility crossed trunk from {from:?}: {dash:?}"
                );
            }
            app.world_mut().despawn(tree);
            app.update();
            let clear = app
                .world_mut()
                .run_system_once(|floor: Floor| {
                    floor.travel(Vec3::X * 12.0, Vec3::NEG_X * 12.0, 0.6)
                })
                .unwrap();
            assert!(
                (clear.x + 12.0).abs() < 0.01,
                "removed hull must stop blocking: {clear:?}"
            );
        }
    }

    #[test]
    fn walking_slides_along_a_wall_and_mobility_cannot_cross_it() {
        let mut app = fixture(&[0.]);
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::trimesh(
                vec![
                    Vec3::new(2., 0., -5.),
                    Vec3::new(2., 4., -5.),
                    Vec3::new(2., 4., 5.),
                    Vec3::new(2., 0., 5.),
                ],
                vec![[0, 2, 1], [0, 3, 2]],
            ),
            Transform::default(),
        ));
        app.update();
        let (slide, travel) = app
            .world_mut()
            .run_system_once(|floor: Floor| {
                (
                    floor.slide(Vec3::ZERO, Vec3::new(10., 0., 4.), 0.6),
                    floor.travel(Vec3::ZERO, Vec3::new(10., 0., 0.), 0.6),
                )
            })
            .unwrap();
        assert!(slide.x < 1.41 && slide.x > 1.0, "{slide:?}");
        assert!(
            slide.z > 3.5,
            "wall must permit tangential movement: {slide:?}"
        );
        assert!(travel.x < 1.41 && travel.x > 1.0, "{travel:?}");
        assert!(travel.z.abs() < 0.001);
        let clear = app
            .world_mut()
            .run_system_once(|floor: Floor| {
                floor.line_clear(Vec3::new(1.4, 0., 0.), Vec3::new(3., 0., 0.))
            })
            .unwrap();
        assert!(
            !clear,
            "a close charge target behind the wall must not receive its stun"
        );
    }

    #[test]
    fn mobility_requires_a_supported_destination() {
        let mut app = fixture(&[0.]);
        let at = app
            .world_mut()
            .run_system_once(|floor: Floor| floor.travel(Vec3::ZERO, Vec3::new(20., 0., 0.), 0.6))
            .unwrap();
        assert_eq!(at, Vec3::ZERO);
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
