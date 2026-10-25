//! Placed Gurubashi WMO evidence, using explicit checkout data and runtime traces.
use super::{tests::fixture, Floor};
use avian3d::prelude::*;
use benilla_assets::coords::{placement_rotation, wow_to_bevy};
use benilla_world::collision::WorldCollision;
use bevy::{ecs::system::RunSystemOnce, prelude::*};

#[test]
fn gurubashi_placed_arena_body_and_camera_boundaries() {
    let Some(data) = std::env::var_os("WOW_DATA").filter(|p| !p.is_empty()) else {
        eprintln!("SKIP Gurubashi arena fixture: set WOW_DATA explicitly");
        return;
    };
    let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
    let point = [-13205., 272., 22.];
    let tiles = benilla_formats::MapTiles::load(&mut chain, "Azeroth").unwrap();
    let (x, y) = tiles.tile_at(point[0], point[1]);
    let tile = benilla_formats::load_tile_mesh(&mut chain, "Azeroth", x, y).unwrap();
    let arena = tile
        .wmos
        .iter()
        .find(|w| w.model.ends_with("stranglegladiatorarena.wmo"))
        .unwrap();
    let root = benilla_formats::parse_wmo_root(&chain.read_file(&arena.model).unwrap()).unwrap();
    assert_eq!(root.group_count(), 1);
    let path = format!("{}_000.wmo", arena.model.strip_suffix(".wmo").unwrap());
    let group = chain.read_file(&path).unwrap();
    let transform = Transform {
        translation: wow_to_bevy(arena.position),
        rotation: placement_rotation(arena.rotation),
        ..default()
    };
    let mut start = wow_to_bevy(point);
    start.y = benilla_formats::terrain_height_at(&tile.chunks, point).unwrap();
    for camera in [false, true] {
        let (mut vertices, mut indices) = (Vec::new(), Vec::new());
        if camera {
            benilla_formats::accumulate_wmo_group_camera_collision(
                &group,
                &mut vertices,
                &mut indices,
            );
        } else {
            benilla_formats::accumulate_wmo_group_collision(&group, &mut vertices, &mut indices);
            assert_eq!(indices.len() / 3, 1854);
        }
        // Separate fixtures avoid assigning private engine layers: each query receives
        // exactly its own parsed face set. Streaming/layer registration is not tested.
        let mut app = fixture(&[]);
        let wall = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::trimesh(
                    vertices
                        .into_iter()
                        .map(|v| transform.transform_point(wow_to_bevy(v)))
                        .collect(),
                    indices.as_chunks::<3>().0.to_vec(),
                ),
                Transform::default(),
            ))
            .id();
        app.update();
        let samples = app
            .world_mut()
            .run_system_once(move |floor: Floor, collision: WorldCollision| {
                (0..8)
                    .map(|i| {
                        let angle = i as f32 * std::f32::consts::TAU / 8.;
                        let direction = Vec3::new(angle.cos(), 0., angle.sin());
                        if camera {
                            (
                                i,
                                collision.cast_camera(
                                    start + Vec3::Y * 1.5,
                                    direction * 60.,
                                    false,
                                ),
                                super::camera_eye(
                                    &collision,
                                    start + Vec3::Y * 1.5,
                                    start + Vec3::Y * 1.5 + direction * 60.,
                                )
                                .distance(start + Vec3::Y * 1.5),
                            )
                        } else {
                            let at = floor.slide(
                                start,
                                start + direction * 60.,
                                super::super::HERO_RADIUS,
                            );
                            (
                                i,
                                collision
                                    .ray_body(start + Vec3::Y, Dir3::new(direction).unwrap(), 60.)
                                    .map(|h| h.distance),
                                at.distance(start),
                            )
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();
        eprintln!("Gurubashi camera={camera}: {samples:?}");
        for (direction, hit, stopped) in samples {
            let hit = hit.expect("all eight authored radial probes hit the arena wall");
            assert!((38.0..47.0).contains(&hit), "direction {direction}: {hit}");
            assert!(
                stopped > 30. && stopped < hit,
                "direction {direction}: stop {stopped}, hit {hit}"
            );
            if camera {
                assert!((stopped - (hit - 0.05)).abs() < 0.005);
            }
        }
        if !camera {
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
            app.world_mut()
                .run_system_once(move |floor: Floor, spatial: SpatialQuery| {
                    for radius in [super::super::HERO_RADIUS, 1.3] {
                        for ray in 0..64 {
                            let angle = ray as f32 * std::f32::consts::TAU / 64.;
                            let direction = Vec3::new(angle.cos(), 0., angle.sin());
                            let mut at = start;
                            for step in 0..120 {
                                let mut next = floor.slide(at, at + direction * 0.25, radius);
                                next.y = floor.height(next).expect("supported arena floor");
                                assert!((next.y - start.y).abs() < 0.03);
                                assert!(
                                    !spatial
                                        .shape_intersections(
                                            &Collider::capsule(radius - 0.02, 1.0),
                                            next + Vec3::Y * (radius + 0.5 + 0.03),
                                            Quat::IDENTITY,
                                            &WorldCollision::body_filter()
                                        )
                                        .contains(&wall),
                                    "arena wall penetration ray {ray} step {step}"
                                );
                                at = next;
                            }
                            assert!(
                                (29.7..30.2).contains(&at.distance(start)),
                                "arena traversal stalled: radius {radius}, ray {ray}, {at:?}"
                            );
                        }
                    }
                })
                .unwrap();
        }
        app.world_mut().despawn(wall);
        app.update();
        app.world_mut()
            .run_system_once(move |floor: Floor, collision: WorldCollision| {
                // Above the arena floor to isolate removal from terrain beyond its wall.
                let feet = start + Vec3::Y * 10.;
                let desired = feet + Vec3::X * 60.;
                if camera {
                    assert_eq!(super::camera_eye(&collision, feet, desired), desired);
                } else {
                    assert!(
                        floor
                            .slide(feet, desired, super::super::HERO_RADIUS)
                            .distance(desired)
                            < 0.02
                    );
                }
            })
            .unwrap();
    }
}
