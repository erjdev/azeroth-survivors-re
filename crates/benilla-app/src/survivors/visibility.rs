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
    fn elwynn_canopy_batches_occlude_from_both_sides_without_colliders() {
        use benilla_world::interact::PickMember;
        use bevy::camera::visibility::SetViewVisibility;
        let Some(data) = std::env::var_os("WOW_DATA").filter(|path| !path.is_empty()) else {
            eprintln!("SKIP authored canopy visibility: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        for model in [
            "elwynntreecanopy01",
            "elwynntreecanopy02",
            "elwynntreecanopy03",
            "elwynntreecanopy04",
        ] {
            let path = format!("World/azeroth/elwynn/passivedoodads/trees/{model}.m2");
            let meshes = benilla_formats::load_m2_mesh(&mut chain, &path).unwrap();
            let mut checked = 0;
            for mesh in meshes {
                let triangle = mesh.indices.as_chunks::<3>().0.iter().find_map(|indices| {
                    let a =
                        benilla_assets::coords::wow_to_bevy(mesh.positions[indices[0] as usize]);
                    let b =
                        benilla_assets::coords::wow_to_bevy(mesh.positions[indices[1] as usize]);
                    let c =
                        benilla_assets::coords::wow_to_bevy(mesh.positions[indices[2] as usize]);
                    let normal = (b - a).cross(c - a).try_normalize()?;
                    Some(((a + b + c) / 3.0, normal))
                });
                let Some((center, normal)) = triangle else {
                    continue;
                };
                checked += 1;
                let geometry = Arc::new(mesh);
                for merged in [false, true] {
                    let mut world = World::new();
                    let pose = Transform::from_xyz(13.0, 4.0, -7.0)
                        .with_rotation(Quat::from_rotation_y(0.7))
                        .with_scale(Vec3::splat(1.3));
                    let object = WorldObject {
                        kind: ModelKind::Doodad,
                        label: model.into(),
                        id: 1,
                        detail: String::new(),
                    };
                    let entity = world
                        .spawn((GlobalTransform::IDENTITY, ViewVisibility::HIDDEN))
                        .id();
                    if merged {
                        world
                            .entity_mut(entity)
                            .insert(PickBlob(Arc::from([PickMember {
                                object: Arc::new(object),
                                geometry: geometry.clone(),
                                transform: pose,
                            }])));
                    } else {
                        world.entity_mut(entity).insert((
                            PickMesh(geometry.clone()),
                            object,
                            GlobalTransform::from(pose),
                        ));
                    }
                    world
                        .get_mut::<ViewVisibility>(entity)
                        .unwrap()
                        .set_visible();
                    let center = pose.transform_point(center);
                    let normal = pose.rotation * normal;
                    for side in [-1.0, 1.0] {
                        let eye = center + normal * side * 2.0;
                        let focus = center - normal * side * 2.0;
                        assert!(
                            world
                                .run_system_once(move |pick: WorldPick| {
                                    visually_blocked(&pick, &HashSet::from([entity]), eye, focus)
                                })
                                .unwrap(),
                            "{model}: merged={merged}, side={side}"
                        );
                    }
                }
            }
            assert!(checked > 0, "{model}: no nondegenerate render batches");
        }
    }

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

    #[test]
    fn render_only_scenery_drives_real_time_fade_and_resets_across_modes() {
        use benilla_world::interact::PickMember;
        use bevy::camera::visibility::SetViewVisibility;
        use std::time::Duration;
        for merged in [false, true] {
            let mut world = World::new();
            world.insert_resource(State::new(Phase::Playing));
            world.insert_resource(Time::<Real>::default());
            let mut settings = CameraSettings::default();
            settings.cutaway = true;
            world.insert_resource(settings);
            world.init_resource::<SceneryCutaway>();
            // Put the camera focus at (0, 0, 10), beyond a render-only canopy at z=5.
            world.spawn((
                Hero { vy: 0., air: 0. },
                GlobalTransform::from_translation(Vec3::new(0., -CAMERA_FOCUS_HEIGHT, 10.)),
            ));
            world.spawn((
                WorldCamera,
                GlobalTransform::IDENTITY,
                Projection::Perspective(PerspectiveProjection::default()),
            ));
            let geometry = Arc::new(benilla_formats::RenderSubmesh {
                positions: vec![[-5., 2., -2.], [-5., -2., -2.], [-5., 0., 2.]],
                indices: vec![0, 1, 2],
                ..default()
            });
            let object = WorldObject {
                kind: ModelKind::Doodad,
                label: "Canopy without collision".into(),
                id: 1,
                detail: String::new(),
            };
            let canopy = world
                .spawn((GlobalTransform::IDENTITY, ViewVisibility::HIDDEN))
                .id();
            if merged {
                world
                    .entity_mut(canopy)
                    .insert(PickBlob(Arc::from([PickMember {
                        object: Arc::new(object),
                        geometry,
                        transform: Transform::IDENTITY,
                    }])));
            } else {
                world
                    .entity_mut(canopy)
                    .insert((PickMesh(geometry), object));
            }
            world
                .get_mut::<ViewVisibility>(canopy)
                .unwrap()
                .set_visible();
            let mut schedule = Schedule::default();
            schedule.add_systems(sync);
            let step = |world: &mut World, schedule: &mut Schedule| {
                world
                    .resource_mut::<Time<Real>>()
                    .advance_by(Duration::from_millis(100));
                schedule.run(world);
                world.resource::<SceneryCutaway>().camera.w
            };
            for frame in 1..=5 {
                let strength = step(&mut world, &mut schedule);
                assert!(
                    (strength - frame as f32 * 0.2).abs() < 0.00001,
                    "500 ms fade, merged={merged}"
                );
                let cutaway = world.resource::<SceneryCutaway>();
                assert_eq!(cutaway.focus.truncate(), Vec3::Z * 10.);
                assert_eq!(cutaway.camera.truncate(), Vec3::ZERO);
                assert!(cutaway.focus.w > 0.);
            }
            // Behind the hero, the same canopy must release the reveal over 500 ms.
            if merged {
                let mut blob = world.get_mut::<PickBlob>(canopy).unwrap();
                Arc::make_mut(&mut blob.0)[0].transform.translation = Vec3::Z * 10.;
            } else {
                *world.get_mut::<GlobalTransform>(canopy).unwrap() =
                    GlobalTransform::from_translation(Vec3::Z * 10.);
            }
            for frame in 1..=5 {
                assert!(
                    (step(&mut world, &mut schedule) - (1. - frame as f32 * 0.2)).abs() < 0.00001
                );
            }
            if merged {
                let mut blob = world.get_mut::<PickBlob>(canopy).unwrap();
                Arc::make_mut(&mut blob.0)[0].transform.translation = Vec3::ZERO;
            } else {
                *world.get_mut::<GlobalTransform>(canopy).unwrap() = GlobalTransform::IDENTITY;
            }
            // The fade remains on real time while gameplay is paused.
            world.insert_resource(State::new(Phase::Paused));
            assert!((step(&mut world, &mut schedule) - 0.2).abs() < 0.00001);
            world.resource_mut::<CameraSettings>().cutaway = false;
            assert_eq!(step(&mut world, &mut schedule), 0.);
            assert_eq!(world.resource::<SceneryCutaway>().focus, Vec4::ZERO);
            world.resource_mut::<CameraSettings>().cutaway = true;
            assert!((step(&mut world, &mut schedule) - 0.2).abs() < 0.00001);
            for phase in [Phase::Menu, Phase::GameOver] {
                world.insert_resource(State::new(phase));
                assert_eq!(step(&mut world, &mut schedule), 0.);
                assert_eq!(world.resource::<SceneryCutaway>().focus, Vec4::ZERO);
                world.insert_resource(State::new(Phase::Playing));
                assert!((step(&mut world, &mut schedule) - 0.2).abs() < 0.00001);
            }
        }
    }
}
