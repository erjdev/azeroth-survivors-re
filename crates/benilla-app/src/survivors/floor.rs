//! Floors near a survivor's feet, including WMO interiors and bridges.
//!
//! A bounded probe preserves the authored/current height while geometry streams and avoids
//! selecting terrain far below an interior. Body sweeps handle walls; route finding is separate.

use avian3d::character_controller::move_and_slide::MoveAndSlide;
use avian3d::prelude::{Collider, SpatialQuery};
use benilla_world::collision::WorldCollision;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

const STEP_UP: f32 = 2.0;
const DROP: f32 = 8.0;

#[derive(SystemParam)]
pub(crate) struct Floor<'w, 's> {
    collision: WorldCollision<'w, 's>,
    spatial: SpatialQuery<'w, 's>,
}

impl Floor<'_, '_> {
    /// Sweep the Survivors body horizontally, sliding along room geometry. Body radius is
    /// shared with horde separation; the capsule's cylindrical section is one yard tall.
    pub fn slide(&self, feet: Vec3, desired: Vec3, radius: f32) -> Vec3 {
        let mut movement = super::units::flat(desired - feet);
        if movement.length_squared() <= 1e-10 {
            return feet;
        }
        // Grounded steps follow gentle terrain before sweeping. A horizontal capsule
        // repeatedly grounded at its center can otherwise wedge against rising ground.
        let mut slope_lift = 0.0_f32;
        let grounded = self.ground(feet).filter(|(y, _)| (feet.y - y).abs() < 0.1);
        if let Some((_, normal_y)) = grounded {
            // The rounded base needs slope clearance; center floor height alone embeds
            // its uphill side. For a plane with normal n, the vertical offset is r/n.y.
            slope_lift = radius * (normal_y.recip() - 1.0);
            if let Some((next, next_normal_y)) = self.ground(feet + movement) {
                let rise = next - feet.y;
                if rise.abs() <= movement.length() * 0.75 {
                    movement.y = rise;
                    slope_lift = slope_lift.max(radius * (next_normal_y.recip() - 1.0));
                }
            }
        }
        let lift = Vec3::Y * (radius + 0.5 + 0.03 + slope_lift);
        let body = Collider::capsule(radius, 1.0);
        let at = self.sweep_slide(&body, feet + lift, movement) - lift;
        // Grounding changes both the foot height and the capsule's slope clearance.
        // Validate that final pose too: a clear sweep alone can end inside a widening
        // root when the actor subsequently snaps onto the floor beneath its center.
        if let Some((y, normal_y)) = self.ground(at).filter(|_| grounded.is_some()) {
            let center = Vec3::new(at.x, y + radius / normal_y + 0.5 + 0.03, at.z);
            if !self
                .spatial
                .shape_intersections(
                    &Collider::capsule((radius - 0.02).max(0.01), 1.0),
                    center,
                    Quat::IDENTITY,
                    &WorldCollision::body_filter(),
                )
                .is_empty()
            {
                return feet;
            }
        }
        at
    }

    /// Resolve the planes actually reached by successive sweeps. A second wall is
    /// found by sweeping the projected remainder, without gathering every nearby face.
    fn sweep_slide(&self, body: &Collider, mut at: Vec3, mut movement: Vec3) -> Vec3 {
        let mut planes = [Dir3::Y; 4];
        for i in 0..planes.len() {
            let Ok((direction, distance)) = Dir3::new_and_length(movement) else {
                break;
            };
            if distance < 0.0001 {
                break;
            }
            let Some(hit) = self.collision.cast_body(body, at, movement, 0.01) else {
                return at + movement;
            };
            let travel = hit.distance.clamp(0.0, distance);
            at += direction * travel;
            movement *= 1.0 - travel / distance;
            // Steep hulls remain walls; do not climb into a widening tree base.
            let normal = if hit.normal1.y < 0.5 {
                super::units::flat(hit.normal1)
            } else {
                hit.normal1
            };
            let Ok(normal) = Dir3::new(normal) else {
                break;
            };
            planes[i] = normal;
            movement = MoveAndSlide::project_velocity(movement, &planes[..=i]);
        }
        at
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
        // Preserve the airborne sweep: grounding its first segment would turn a jump
        // into an artificial steep descent and cancel otherwise valid mobility.
        if self
            .height(feet)
            .is_some_and(|ground| (feet.y - ground).abs() > 0.1)
        {
            let at = match self.collision.cast_body(&body, feet + lift, movement, 0.02) {
                Some(hit) => feet + movement / length * (hit.distance - 0.02).clamp(0.0, length),
                None => feet + movement,
            };
            return self.height(at).map_or(feet, |y| Vec3::new(at.x, y, at.z));
        }
        // Follow gentle terrain in short straight segments. A single horizontal sweep
        // intersects uphill ground long before the intended destination.
        let direction = movement / length;
        let steps = (length / 0.25).ceil() as usize;
        let mut at = feet;
        for step in 1..=steps {
            let horizontal = feet + direction * (length * step as f32 / steps as f32);
            let Some(y) = self.height(Vec3::new(horizontal.x, at.y, horizontal.z)) else {
                return feet;
            };
            let next = Vec3::new(horizontal.x, y, horizontal.z);
            let segment = next - at;
            let horizontal_length = super::units::flat(segment).length();
            // Do not turn a floor probe onto a low wall into a climb over that wall.
            if segment.y.abs() > horizontal_length * 0.75 {
                return at;
            }
            if let Some(hit) = self.collision.cast_body(&body, at + lift, segment, 0.02) {
                let distance = segment.length();
                let stopped = at + segment / distance * (hit.distance - 0.02).clamp(0.0, distance);
                return self
                    .height(stopped)
                    .map_or(at, |y| Vec3::new(stopped.x, y, stopped.z));
            }
            at = next;
        }
        at
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
        self.ground(feet).map(|(height, _)| height)
    }

    /// Spawn bodies need both a supported floor and unoccupied body space. Use the
    /// same slope clearance as movement, so ordinary uphill terrain is not an obstacle.
    pub fn can_spawn(&self, feet: Vec3, radius: f32) -> bool {
        let Some((y, normal_y)) = self.ground(feet) else {
            return false;
        };
        if (feet.y - y).abs() > 0.1 {
            return false;
        }
        let center = feet + Vec3::Y * (radius / normal_y + 0.5 + 0.03);
        self.spatial
            .shape_intersections(
                &Collider::capsule(radius, 1.0),
                center,
                Quat::IDENTITY,
                &WorldCollision::body_filter(),
            )
            .is_empty()
    }

    fn ground(&self, feet: Vec3) -> Option<(f32, f32)> {
        let origin = feet + Vec3::Y * STEP_UP;
        if let Some(hit) = self.collision.ray_body(origin, Dir3::NEG_Y, STEP_UP + DROP) {
            return (hit.normal.y > 0.5).then_some((origin.y - hit.distance, hit.normal.y));
        }
        // A ray exactly on a shared triangle edge can miss both faces. Probe one
        // centimetre to either side and project the hit plane back onto the feet.
        [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
            .into_iter()
            .find_map(|axis| {
                let offset = axis * 0.01;
                let hit = self
                    .collision
                    .ray_body(origin + offset, Dir3::NEG_Y, STEP_UP + DROP)?;
                if hit.normal.y <= 0.5 {
                    return None;
                }
                let height = origin.y - hit.distance + hit.normal.dot(offset) / hit.normal.y;
                (height >= feet.y - DROP && height <= origin.y).then_some((height, hit.normal.y))
            })
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

    pub(super) fn fixture(heights: &[f32]) -> App {
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
                let deflected = app
                    .world_mut()
                    .run_system_once(move |floor: Floor| {
                        let mut at = from;
                        let mut deflected = false;
                        for _ in 0..96 {
                            let wanted = at - axis * 0.25;
                            let mut next = floor.slide(at, wanted, 0.6);
                            deflected |= super::super::units::flat(next - wanted).length() > 0.001;
                            next.y = floor.height(next).unwrap_or(next.y);
                            assert!(
                                floor.line_clear(at, next),
                                "walking center crossed a hull face: {at:?} -> {next:?}"
                            );
                            at = next;
                        }
                        deflected
                    })
                    .unwrap();
                // Walking can slide around even a thin trunk. Check that it responds
                // to the hull, and independently ray-check every resulting segment.
                assert!(
                    deflected,
                    "{model}: repeated grounded walking ignored trunk from {from:?}"
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
    fn placed_elwynn_canopy_walks_without_crossing_hull_faces() {
        use benilla_assets::coords::{placement_rotation, wow_to_bevy};
        let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
            eprintln!("SKIP placed Elwynn test: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        let maps = benilla_formats::load_map_catalog(&mut chain).unwrap();
        let directory = maps.directory(0).unwrap();
        let tiles = benilla_formats::MapTiles::load(&mut chain, directory).unwrap();
        let (x, y) = tiles.tile_at(-9520.363, 100.507454);
        let tile = benilla_formats::load_tile_mesh(&mut chain, directory, x, y).unwrap();
        let d = tile.doodads.iter().find(|d| d.unique_id == 12580).unwrap();
        let hull = benilla_formats::load_m2_collision_hull(&mut chain, &d.model).unwrap();
        let transform = Transform {
            translation: wow_to_bevy(d.position),
            rotation: placement_rotation(d.rotation),
            scale: Vec3::splat(d.scale),
        };
        let mut app = fixture(&[]);
        for chunk in &tile.chunks {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::trimesh(
                    chunk.positions.iter().map(|p| wow_to_bevy(*p)).collect(),
                    chunk.indices.as_chunks::<3>().0.to_vec(),
                ),
                Transform::default(),
            ));
        }
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::trimesh(
                hull.positions
                    .iter()
                    .map(|p| transform.transform_point(wow_to_bevy(*p)))
                    .collect(),
                hull.indices.as_chunks::<3>().0.to_vec(),
            ),
            Transform::default(),
        ));
        app.update();
        app.update();
        for (start, direction) in [
            (Vec3::new(-106.507454, 59.379368, 9520.363), Vec3::X),
            (Vec3::new(-94.507454, 59.64867, 9520.363), Vec3::NEG_X),
        ] {
            app.world_mut()
                .run_system_once(move |floor: Floor| {
                    let mut at = start;
                    for step in 0..48 {
                        let mut next =
                            floor.slide(at, at + direction * 0.25, super::super::HERO_RADIUS);
                        next.y = floor.height(next).unwrap_or(next.y);
                        assert!(
                            floor.line_clear(at, next),
                            "placed canopy center crossed face at step {step}: {at:?} -> {next:?}"
                        );
                        at = next;
                    }
                    eprintln!("placed canopy {direction:?}: {start:?} -> {at:?}");
                })
                .unwrap();
        }
    }

    #[test]
    fn placed_warsong_stump_blocks_the_candidate_crossing_and_unregisters() {
        use benilla_assets::coords::{placement_rotation, wow_to_bevy};
        let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
            eprintln!("SKIP Warsong placement test: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        let maps = benilla_formats::load_map_catalog(&mut chain).unwrap();
        let directory = maps.directory(489).unwrap();
        let tiles = benilla_formats::MapTiles::load(&mut chain, directory).unwrap();
        let point = [1244.792, 1468.75, 310.296];
        let mut placement = None;
        for (x, y) in tiles.existing_in_radius(point[0], point[1], 1) {
            let tile = benilla_formats::load_tile_mesh(&mut chain, directory, x, y).unwrap();
            if let Some(found) = tile.doodads.into_iter().find(|d| d.unique_id == 596699) {
                placement = Some(found);
                break;
            }
        }
        let d = placement.expect("documented Warsong stump placement");
        assert!(d
            .model
            .to_ascii_lowercase()
            .contains("ashenvaletreestump02"));
        let hull = benilla_formats::load_m2_collision_hull(&mut chain, &d.model).unwrap();
        assert_eq!(hull.triangle_count(), 24);
        let transform = Transform {
            translation: wow_to_bevy(d.position),
            rotation: placement_rotation(d.rotation),
            scale: Vec3::splat(d.scale),
        };
        let vertices = hull
            .positions
            .iter()
            .map(|p| transform.transform_point(wow_to_bevy(*p)))
            .collect();
        let collider = Collider::trimesh(vertices, hull.indices.as_chunks::<3>().0.to_vec());
        let mut app = fixture(&[]);
        let stump = app
            .world_mut()
            .spawn((RigidBody::Static, collider.clone(), Transform::default()))
            .id();
        app.update();
        app.update();
        let start = wow_to_bevy(point);
        // Fixed-height capsule sweeps isolate this placement from terrain. The source ray
        // is one yard above the feet, at the documented 8.898-yard negative-WoW-Y hit.
        let feet = start - Vec3::Y;
        let desired = feet + Vec3::X * 24.;
        let check = |app: &mut App| {
            app.world_mut()
                .run_system_once(move |floor: Floor| {
                    let ray = floor.collision.ray_body(start, Dir3::X, 24.);
                    (
                        ray.map(|h| h.distance),
                        floor.slide(feet, desired, super::super::HERO_RADIUS),
                    )
                })
                .unwrap()
        };
        let (ray, stopped) = check(&mut app);
        assert!((ray.expect("registered hull ray") - 8.898).abs() < 0.02);
        assert!(
            stopped.distance(desired) > 1.,
            "capsule crossed stump: {stopped:?}"
        );
        assert!(stopped.x > feet.x, "capsule made no approach: {stopped:?}");
        app.world_mut().despawn(stump);
        app.update();
        let (ray, clear) = check(&mut app);
        assert!(ray.is_none());
        assert!(
            clear.distance(desired) < 0.02,
            "removed stump still blocks: {clear:?}"
        );
        app.world_mut()
            .spawn((RigidBody::Static, collider, Transform::default()));
        app.update();
        let (ray, reloaded) = check(&mut app);
        assert!(ray.is_some());
        assert!(
            reloaded.distance(stopped) < 0.02,
            "reloaded hull changed contact"
        );
    }

    #[test]
    fn placed_feralas_root_blocks_the_candidate_crossing_and_unregisters() {
        use benilla_assets::coords::{placement_rotation, wow_to_bevy};
        let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
            eprintln!("SKIP Feralas placement test: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        let maps = benilla_formats::load_map_catalog(&mut chain).unwrap();
        let directory = maps.directory(1).unwrap();
        let tiles = benilla_formats::MapTiles::load(&mut chain, directory).unwrap();
        let point = [-4200.0, -133.828, 55.827];
        let mut placement = None;
        for (x, y) in tiles.existing_in_radius(point[0], point[1], 1) {
            let tile = benilla_formats::load_tile_mesh(&mut chain, directory, x, y).unwrap();
            if let Some(found) = tile.doodads.into_iter().find(|d| d.unique_id == 245515) {
                placement = Some(found);
                break;
            }
        }
        let d = placement.expect("documented Feralas root placement");
        assert!(d.model.to_ascii_lowercase().contains("feralastreeroots02"));
        let hull = benilla_formats::load_m2_collision_hull(&mut chain, &d.model).unwrap();
        assert_eq!(hull.triangle_count(), 60);
        let transform = Transform {
            translation: wow_to_bevy(d.position),
            rotation: placement_rotation(d.rotation),
            scale: Vec3::splat(d.scale),
        };
        let vertices = hull
            .positions
            .iter()
            .map(|p| transform.transform_point(wow_to_bevy(*p)))
            .collect();
        let collider = Collider::trimesh(vertices, hull.indices.as_chunks::<3>().0.to_vec());
        let mut app = fixture(&[]);
        let root = app
            .world_mut()
            .spawn((RigidBody::Static, collider.clone(), Transform::default()))
            .id();
        app.update();
        app.update();
        let start = wow_to_bevy(point);
        // Fixed-height capsule sweeps isolate this placement from terrain. The source ray
        // is one yard above the feet, at the documented 38.137535-yard negative-WoW-X hit.
        let feet = start - Vec3::Y;
        let desired = feet + Vec3::Z * 55.;
        let check = |app: &mut App| {
            app.world_mut()
                .run_system_once(move |floor: Floor| {
                    let ray = floor.collision.ray_body(start, Dir3::Z, 55.);
                    (
                        ray.map(|h| h.distance),
                        floor.slide(feet, desired, super::super::HERO_RADIUS),
                    )
                })
                .unwrap()
        };
        let (ray, stopped) = check(&mut app);
        assert!((ray.expect("registered hull ray") - 38.137535).abs() < 0.02);
        assert!(
            stopped.distance(desired) > 1.,
            "capsule crossed root: {stopped:?}"
        );
        assert!(stopped.z > feet.z, "capsule made no approach: {stopped:?}");
        app.world_mut().despawn(root);
        app.update();
        let (ray, clear) = check(&mut app);
        assert!(ray.is_none());
        assert!(
            clear.distance(desired) < 0.02,
            "removed root still blocks: {clear:?}"
        );
        let root = app
            .world_mut()
            .spawn((RigidBody::Static, collider, Transform::default()))
            .id();
        app.update();
        let (ray, reloaded) = check(&mut app);
        assert!(ray.is_some());
        assert!(
            reloaded.distance(stopped) < 0.02,
            "reloaded hull changed contact"
        );
        let (x, y) = tiles.tile_at(point[0], point[1]);
        let tile = benilla_formats::load_tile_mesh(&mut chain, directory, x, y).unwrap();
        for chunk in &tile.chunks {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::trimesh(
                    chunk.positions.iter().map(|p| wow_to_bevy(*p)).collect(),
                    chunk.indices.as_chunks::<3>().0.to_vec(),
                ),
                Transform::default(),
            ));
        }
        app.update();
        let ground = benilla_formats::terrain_height_at(&tile.chunks, point).unwrap();
        let grounded = Vec3::new(start.x, ground, start.z);
        let walked = app
            .world_mut()
            .run_system_once(move |floor: Floor, spatial: SpatialQuery| {
                let radius = super::super::HERO_RADIUS;
                let mut at = grounded;
                for step in 0..220 {
                    let mut next = floor.slide(at, at + Vec3::Z * 0.25, radius);
                    // Match actors: a root face can be the nearest downward hit without
                    // being a walkable floor. Retain the swept height in that case.
                    next.y = floor.height(next).unwrap_or(next.y);
                    let wow = benilla_assets::coords::bevy_to_wow(next);
                    let terrain = tile
                        .chunks
                        .iter()
                        .find_map(|chunk| {
                            chunk.indices.as_chunks::<3>().0.iter().find_map(|tri| {
                                let face = tri.map(|i| chunk.positions[i as usize]);
                                benilla_formats::triangle_z_at(&face, wow[0], wow[1])
                            })
                        })
                        .expect("Feralas path stays on emitted terrain geometry");
                    assert!(
                        next.y >= terrain - 0.02,
                        "Feralas terrain penetration at step {step}: {next:?}, floor {terrain}"
                    );
                    assert!(
                        !spatial
                            .shape_intersections(
                                &Collider::capsule(radius - 0.02, 1.0),
                                next + Vec3::Y
                                    * (radius / floor.ground(next).map_or(1.0, |(_, ny)| ny)
                                        + 0.5
                                        + 0.03),
                                Quat::IDENTITY,
                                &WorldCollision::body_filter(),
                            )
                            .contains(&root),
                        "Feralas root penetration at step {step}: {at:?} -> {next:?}"
                    );
                    at = next;
                }
                at
            })
            .unwrap();
        assert!(
            walked.z > grounded.z + 30.,
            "grounded capsule stalled before reaching the root: {walked:?}"
        );
        eprintln!("Feralas grounded root walk: {grounded:?} -> {walked:?}");
        if std::env::var_os("WOW_SURVIVORS_FLOOR_BENCH").is_some() {
            benchmark_grounded_horde(&mut app, grounded);
        }
    }

    /// Opt-in timing only: no machine-dependent pass threshold. Uses the already loaded
    /// Feralas terrain/root fixture, excluding asset loading, rendering and actor separation.
    fn benchmark_grounded_horde(app: &mut App, origin: Vec3) {
        use bevy::platform::time::Instant;
        for count in [1, 60, super::super::data::target_alive(960.)] {
            app.world_mut()
                .run_system_once(move |floor: Floor| {
                    let mut actors: Vec<_> = (0..count)
                        .map(|i| {
                            let at = origin
                                + Vec3::new((i % 20) as f32 - 10., 0., (i / 20) as f32);
                            Vec3::new(at.x, floor.height(at).unwrap_or(at.y), at.z)
                        })
                        .collect();
                    let mut samples = Vec::with_capacity(600);
                    let mut overlap_samples = Vec::with_capacity(600);
                    let mut moving = 0;
                    for frame in 0..660 {
                        let start = Instant::now();
                        let mut advanced = 0;
                        for (i, at) in actors.iter_mut().enumerate() {
                            let radius = if i % 10 == 0 { 1.3 } else { 0.6 };
                            let mut next = floor.slide(*at, *at + Vec3::Z * 0.08, radius);
                            next.y = floor.height(next).unwrap_or(next.y);
                            advanced += usize::from((next.z - at.z).abs() > 0.001);
                            *at = next;
                        }
                        let elapsed = start.elapsed().as_secs_f64() * 1000.;
                        // Replay just final-pose validation on the same body positions to
                        // distinguish its cost from the pre-existing slide/contact solver.
                        let overlap_start = Instant::now();
                        for (i, at) in actors.iter().enumerate() {
                            let radius = if i % 10 == 0 { 1.3 } else { 0.6 };
                            if let Some((y, normal_y)) = floor.ground(*at) {
                                let center = Vec3::new(at.x, y + radius / normal_y + 0.53, at.z);
                                std::hint::black_box(floor.spatial.shape_intersections(
                                    &Collider::capsule(radius - 0.02, 1.0), center,
                                    Quat::IDENTITY, &WorldCollision::body_filter(),
                                ));
                            }
                        }
                        let overlap_elapsed = overlap_start.elapsed().as_secs_f64() * 1000.;
                        if frame >= 60 {
                            overlap_samples.push(overlap_elapsed);
                            samples.push(elapsed);
                            moving += advanced;
                        }
                    }
                    std::hint::black_box(&actors);
                    samples.sort_by(f64::total_cmp);
                    overlap_samples.sort_by(f64::total_cmp);
                    eprintln!(
                        "Feralas floor benchmark: actors={count}, frames={}, p50={:.3}ms, p95={:.3}ms, max={:.3}ms, moving_steps={moving}/{}",
                        samples.len(), samples[samples.len() / 2],
                        samples[samples.len() * 95 / 100], samples.last().unwrap(),
                        samples.len() * count,
                    );
                    eprintln!(
                        "Feralas final-pose replay: actors={count}, p50={:.3}ms, p95={:.3}ms",
                        overlap_samples[overlap_samples.len() / 2],
                        overlap_samples[overlap_samples.len() * 95 / 100],
                    );
                })
                .unwrap();
        }
    }

    #[test]
    fn placed_ungoro_tree_keeps_candidate_clear_and_blocks_the_hull_crossing() {
        use benilla_assets::coords::{placement_rotation, wow_to_bevy};
        let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
            eprintln!("SKIP Un'Goro placement test: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        let tiles = benilla_formats::MapTiles::load(&mut chain, "Kalimdor").unwrap();
        let point = [-7136., -1609., -268.];
        let (x, y) = tiles.tile_at(point[0], point[1]);
        let tile = benilla_formats::load_tile_mesh(&mut chain, "Kalimdor", x, y).unwrap();
        let d = tile.doodads.iter().find(|d| d.unique_id == 304715).unwrap();
        assert!(d.model.to_ascii_lowercase().ends_with("feralastree03.mdx"));
        let hull = benilla_formats::load_m2_collision_hull(&mut chain, &d.model).unwrap();
        assert_eq!(hull.triangle_count(), 30);
        let transform = Transform {
            translation: wow_to_bevy(d.position),
            rotation: placement_rotation(d.rotation),
            scale: Vec3::splat(d.scale),
        };
        let mut app = fixture(&[]);
        let tree = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::trimesh(
                    hull.positions
                        .iter()
                        .map(|p| transform.transform_point(wow_to_bevy(*p)))
                        .collect(),
                    hull.indices.as_chunks::<3>().0.to_vec(),
                ),
                Transform::default(),
            ))
            .id();
        app.update();
        app.update();
        let radius = super::super::HERO_RADIUS;
        let ground = benilla_formats::terrain_height_at(&tile.chunks, point).unwrap();
        let feet = wow_to_bevy([point[0], point[1], ground]);
        let lift = Vec3::Y * (radius + 0.5 + 0.03);
        let occupied = app
            .world_mut()
            .run_system_once(move |spatial: SpatialQuery| {
                spatial
                    .shape_intersections(
                        &Collider::capsule(radius, 1.0),
                        feet + lift,
                        Quat::IDENTITY,
                        &WorldCollision::body_filter(),
                    )
                    .contains(&tree)
            })
            .unwrap();
        assert!(
            !occupied,
            "candidate capsule overlaps nearby tree at terrain height"
        );
        let ray_start = wow_to_bevy(point);
        let probe_feet = ray_start - Vec3::Y;
        let desired = probe_feet + Vec3::NEG_X * 20.;
        let (ray, stopped) = app
            .world_mut()
            .run_system_once(move |floor: Floor| {
                (
                    floor
                        .collision
                        .ray_body(ray_start, Dir3::NEG_X, 20.)
                        .map(|h| h.distance),
                    floor.slide(probe_feet, desired, radius),
                )
            })
            .unwrap();
        assert!((ray.expect("placed tree must reproduce source ray") - 2.6105921).abs() < 0.02);
        assert!(
            stopped.distance(desired) > 1.,
            "capsule crossed trunk: {stopped:?}"
        );
        assert!(
            stopped.distance(probe_feet) > 0.01,
            "capsule could not approach trunk"
        );
        eprintln!("Un'Goro tree 304715: ground {ground}, ray {ray:?}, capsule {probe_feet:?} -> {stopped:?}");
        // Match walking: quarter-yard horizontal sweeps, then bounded floor grounding.
        let mut terrain = Vec::new();
        for chunk in &tile.chunks {
            terrain.push(
                app.world_mut()
                    .spawn((
                        RigidBody::Static,
                        Collider::trimesh(
                            chunk.positions.iter().map(|p| wow_to_bevy(*p)).collect(),
                            chunk.indices.as_chunks::<3>().0.to_vec(),
                        ),
                        Transform::default(),
                    ))
                    .id(),
            );
        }
        app.update();
        let walked = app
            .world_mut()
            .run_system_once(move |floor: Floor, spatial: SpatialQuery| {
                let mut at = feet;
                for step in 0..80 {
                    let mut next = floor.slide(at, at + Vec3::NEG_X * 0.25, radius);
                    next.y = floor.height(next).unwrap_or(next.y);
                    assert!(
                        !spatial
                            .shape_intersections(
                                &Collider::capsule(radius - 0.02, 1.0),
                                next + lift,
                                Quat::IDENTITY,
                                &WorldCollision::body_filter(),
                            )
                            .contains(&tree),
                        "tree penetration at step {step}: {at:?} -> {next:?}"
                    );
                    at = next;
                }
                at
            })
            .unwrap();
        eprintln!("Un'Goro grounded walk: {feet:?} -> {walked:?}");
        for entity in terrain {
            app.world_mut().despawn(entity);
        }
        app.world_mut().despawn(tree);
        app.update();
        let clear = app
            .world_mut()
            .run_system_once(move |floor: Floor| floor.slide(probe_feet, desired, radius))
            .unwrap();
        assert!(
            clear.distance(desired) < 0.02,
            "removed tree still blocks crossing"
        );
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
    fn spawn_clearance_rejects_wall_edges_low_ceilings_and_missing_floor() {
        let mut unsupported = fixture(&[]);
        assert!(!unsupported
            .world_mut()
            .run_system_once(|floor: Floor| floor.can_spawn(Vec3::ZERO, 0.6))
            .unwrap());
        let mut app = fixture(&[0.]);
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.2, 4., 10.),
            Transform::from_xyz(2.1, 2., 0.),
        ));
        app.update();
        app.world_mut()
            .run_system_once(|floor: Floor| {
                assert!(floor.can_spawn(Vec3::ZERO, 0.6));
                assert!(floor.can_spawn(Vec3::ZERO, 1.3));
                assert!(floor.can_spawn(Vec3::X * 1.2, 0.6));
                assert!(!floor.can_spawn(Vec3::X * 1.6, 0.6));
                assert!(!floor.can_spawn(Vec3::X * 1.2, 1.3));
                assert!(!floor.can_spawn(Vec3::Y, 0.6));
            })
            .unwrap();
        let ceiling = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::cuboid(3., 0.2, 3.),
                Transform::from_xyz(0., 2.2, 0.),
            ))
            .id();
        app.update();
        assert!(!app
            .world_mut()
            .run_system_once(|floor: Floor| floor.can_spawn(Vec3::ZERO, 0.6))
            .unwrap());
        app.world_mut().despawn(ceiling);
        app.update();
        assert!(app
            .world_mut()
            .run_system_once(|floor: Floor| floor.can_spawn(Vec3::ZERO, 0.6))
            .unwrap());
    }

    #[test]
    fn walking_slides_into_a_two_wall_corner_without_crossing_either_face() {
        let mut app = fixture(&[0.]);
        for (size, center) in [
            (Vec3::new(0.2, 4., 10.), Vec3::new(2.1, 2., 0.)),
            (Vec3::new(10., 4., 0.2), Vec3::new(0., 2., 3.1)),
        ] {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::cuboid(size.x, size.y, size.z),
                Transform::from_translation(center),
            ));
        }
        app.update();
        let at = app
            .world_mut()
            .run_system_once(|floor: Floor| {
                let mut at = Vec3::ZERO;
                for _ in 0..60 {
                    at = floor.slide(at, at + Vec3::new(0.3, 0., 0.3), 0.6);
                    at.y = floor.height(at).unwrap();
                    assert!(at.x < 1.41 && at.z < 2.41, "crossed corner: {at:?}");
                }
                at
            })
            .unwrap();
        assert!(
            at.x > 1.3 && at.z > 2.3,
            "failed to slide to corner: {at:?}"
        );
    }

    #[test]
    fn walking_and_mobility_follow_gentle_slopes_in_both_directions() {
        for slope in [-0.2_f32, -0.1, 0.1, 0.2] {
            let mut app = fixture(&[]);
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::trimesh(
                    vec![
                        Vec3::new(-20., -20. * slope, -5.),
                        Vec3::new(20., 20. * slope, -5.),
                        Vec3::new(20., 20. * slope, 5.),
                        Vec3::new(-20., -20. * slope, 5.),
                    ],
                    vec![[0, 2, 1], [0, 3, 2]],
                ),
                Transform::default(),
            ));
            app.update();
            for sign in [-1., 1.] {
                let expected = Vec3::new(sign * 12., sign * 12. * slope, 0.);
                let (at, walked) = app
                    .world_mut()
                    .run_system_once(move |floor: Floor| {
                        let mut walked = Vec3::ZERO;
                        for _ in 0..48 {
                            walked = floor.slide(
                                walked,
                                walked + Vec3::X * sign * 0.25,
                                super::super::HERO_RADIUS,
                            );
                            walked.y = floor.height(walked).unwrap();
                            assert!(
                                floor.can_spawn(walked, super::super::HERO_RADIUS),
                                "supported slope rejected"
                            );
                        }
                        (
                            floor.travel(Vec3::ZERO, expected, super::super::HERO_RADIUS),
                            walked,
                        )
                    })
                    .unwrap();
                assert!(
                    at.distance(expected) < 0.02,
                    "slope {slope}, expected {expected:?}, got {at:?}"
                );
                assert!(
                    walked.distance(expected) < 0.02,
                    "walk slope {slope}, expected {expected:?}, got {walked:?}"
                );
            }
        }
    }

    #[test]
    fn mobility_does_not_climb_a_low_wall() {
        let mut app = fixture(&[0.]);
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.5, 1.0, 8.0),
            Transform::from_xyz(2.0, 0.5, 0.0),
        ));
        app.update();
        let at = app
            .world_mut()
            .run_system_once(|floor: Floor| floor.travel(Vec3::ZERO, Vec3::new(4., 0., 0.), 0.7))
            .unwrap();
        assert!(at.x < 1.1 && at.y.abs() < 0.02, "{at:?}");
    }

    #[test]
    fn airborne_walking_does_not_validate_a_grounded_body_below_it() {
        let mut app = fixture(&[0.]);
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.5, 1.0, 8.0),
            Transform::from_xyz(2.0, 0.5, 0.0),
        ));
        app.update();
        let at = app
            .world_mut()
            .run_system_once(|floor: Floor| {
                floor.slide(Vec3::new(0., 1.5, 0.), Vec3::new(1.5, 1.5, 0.), 0.7)
            })
            .unwrap();
        assert!(at.distance(Vec3::new(1.5, 1.5, 0.)) < 0.02, "{at:?}");
    }

    #[test]
    fn airborne_mobility_keeps_its_supported_landing() {
        let mut app = fixture(&[0.]);
        let at = app
            .world_mut()
            .run_system_once(|floor: Floor| floor.travel(Vec3::Y, Vec3::new(4., 1., 0.), 0.7))
            .unwrap();
        assert!(at.distance(Vec3::new(4., 0., 0.)) < 0.02, "{at:?}");
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

#[cfg(test)]
#[path = "floor/arena_tests.rs"]
mod arena_tests;
