//! Opt-in actor-only view for the silhouette compositor; no effect on the world color yet.
use super::{companions::Fetcher, enemies::Enemy, hero::Hero, hunter_pet::HunterPet, Phase};
use benilla_assets::materials::WowModelMaterial;
use benilla_world::{
    model_render::{ModelKind, ModelPart},
    view::WorldCamera,
};
use bevy::{
    camera::{
        visibility::{NoFrustumCulling, RenderLayers},
        RenderTarget,
    },
    core_pipeline::tonemapping::Tonemapping,
    mesh::MeshTag,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{TextureFormat, TextureUsages},
        view::Hdr,
    },
};
use std::collections::{HashMap, HashSet};

#[path = "actor_mask_preview.rs"]
mod preview;
#[path = "actor_mask_targets.rs"]
mod targets;

// Reserved above the portrait/UI-model layer ladder. This camera is never a WorldCamera.
const LAYER: usize = 128;

#[derive(Component, Clone, Copy, ExtractComponent)]
pub(crate) struct ActorMaskView;
#[derive(Component)]
struct Mirror;

#[derive(Resource, Default)]
struct Mask {
    enabled: bool,
    preview: bool,
    camera: Option<Entity>,
    image: Handle<Image>,
    size: UVec2,
    parts: HashMap<Entity, Entity>,
    materials: HashMap<(AssetId<WowModelMaterial>, u8), Handle<WowModelMaterial>>,
}

pub(super) fn plugin(app: &mut App) {
    let mut mask = Mask::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        mask.preview = crate::run_mode::dev_affordances()
            && std::env::var("WOW_SURVIVORS_ACTOR_MASK_PREVIEW").is_ok_and(|v| v == "1");
        mask.enabled =
            mask.preview || std::env::var("WOW_SURVIVORS_ACTOR_MASK").is_ok_and(|v| v == "1");
    }
    if mask.preview {
        preview::plugin(app);
    }
    targets::plugin(app);
    app.insert_resource(mask)
        .add_plugins(ExtractComponentPlugin::<ActorMaskView>::default())
        .add_systems(
            PostUpdate,
            sync.after(super::perception::sync)
                .after(bevy::transform::TransformSystems::Propagate)
                .after(bevy::camera::CameraUpdateSystems)
                .after(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate)
                .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
        );
}

fn side(hero: bool, enemy: bool, pet: bool, companion: bool) -> u8 {
    if hero || pet || companion {
        2
    } else if enemy {
        1
    } else {
        0
    }
}
fn color(side: u8) -> Vec3 {
    match side {
        1 => Vec3::X,
        2 => Vec3::Y,
        _ => Vec3::Z,
    }
}

type Owners<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static ChildOf>,
        Option<&'static benilla_world::billboard::BillboardCard>,
        Option<&'static crate::net::NetEntity>,
        Option<&'static InheritedVisibility>,
    ),
>;
type Allegiance<'w, 's> = Query<'w, 's, (Has<Hero>, Has<Enemy>, Has<HunterPet>, Has<Fetcher>)>;

fn owner_side(mut entity: Entity, owners: &Owners, allegiance: &Allegiance) -> Option<u8> {
    // Billboard batches are world roots; follow their explicit attachment before parents.
    // The bound also prevents malformed attachment cycles from hanging a frame.
    for _ in 0..64 {
        let (parent, card, actor, visible) = owners.get(entity).ok()?;
        if visible.is_some_and(|visible| !visible.get()) {
            return None;
        }
        if actor.is_some_and(|actor| {
            matches!(
                actor.kind,
                benilla_protocol::EntityKind::Player | benilla_protocol::EntityKind::Unit
            )
        }) {
            let (hero, enemy, pet, companion) = allegiance.get(entity).ok()?;
            return Some(side(hero, enemy, pet, companion));
        }
        entity = card
            .and_then(|card| card.follows())
            .or_else(|| parent.map(ChildOf::parent))?;
    }
    None
}

type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ModelPart,
        &'static Mesh3d,
        &'static MeshMaterial3d<WowModelMaterial>,
        &'static GlobalTransform,
        &'static InheritedVisibility,
        Option<&'static MeshTag>,
        Option<&'static RenderLayers>,
    ),
    (
        Without<Mirror>,
        Without<ActorMaskView>,
        Without<WorldCamera>,
    ),
>;

// Disjoint ECS queries keep source meshes, mirrors and cameras independently mutable.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync(
    mut commands: Commands,
    mut mask: ResMut<Mask>,
    phase: Res<State<Phase>>,
    view: Query<
        (&Camera, &Projection, &GlobalTransform, &Msaa),
        (With<WorldCamera>, Without<ActorMaskView>),
    >,
    mut cameras: Query<
        (
            &mut Camera,
            &mut Projection,
            &mut Transform,
            &mut GlobalTransform,
            &mut Msaa,
        ),
        (With<ActorMaskView>, Without<WorldCamera>, Without<Mirror>),
    >,
    parts: Parts,
    mut mirrors: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<WowModelMaterial>,
            &mut MeshTag,
            &mut Transform,
            &mut GlobalTransform,
        ),
        (With<Mirror>, Without<WorldCamera>, Without<ActorMaskView>),
    >,
    owners: Owners,
    allegiance: Allegiance,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WowModelMaterial>>,
    mut events: MessageReader<AssetEvent<WowModelMaterial>>,
) {
    let changed: HashSet<_> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();
    let active = mask.enabled
        && (!matches!(phase.get(), Phase::Menu | Phase::GameOver)
            || (mask.preview && *phase.get() == Phase::Menu));
    let source = view.single().ok().filter(|(camera, _, _, _)| {
        active
            && camera.is_active
            && camera
                .physical_target_size()
                .is_some_and(|size| size.x > 0 && size.y > 0)
    });
    let Some((camera, projection, transform, msaa)) = source else {
        if let Some(entity) = mask.camera {
            if let Ok((mut camera, _, _, _, _)) = cameras.get_mut(entity) {
                camera.is_active = false;
            }
        }
        for (_, entity) in mask.parts.drain() {
            commands.entity(entity).despawn();
        }
        mask.materials.clear();
        return;
    };
    let Some(size) = camera
        .physical_target_size()
        .filter(|size| size.x > 0 && size.y > 0)
    else {
        return;
    };
    if mask.camera.is_none() {
        mask.image = images.add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba16Float,
            None,
        ));
        mask.size = size;
        mask.camera = Some(
            commands
                .spawn((
                    ActorMaskView,
                    Camera3d {
                        depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT
                            | TextureUsages::TEXTURE_BINDING)
                            .into(),
                        ..default()
                    },
                    Camera {
                        order: camera.order.saturating_sub(1),
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                        viewport: camera.viewport.clone(),
                        ..default()
                    },
                    RenderTarget::Image(mask.image.clone().into()),
                    projection.clone(),
                    transform.compute_transform(),
                    *transform,
                    Hdr,
                    Tonemapping::None,
                    *msaa,
                    bevy::light::cluster::ClusterConfig::None,
                    RenderLayers::from_layers(&[LAYER]),
                ))
                .id(),
        );
    } else if let Some(entity) = mask.camera {
        if mask.size != size {
            if let Some(image) = images.get_mut(&mask.image) {
                image.resize(bevy::render::render_resource::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                });
                mask.size = size;
            }
        }
        if let Ok((mut target, mut optic, mut local, mut global, mut samples)) =
            cameras.get_mut(entity)
        {
            *samples = *msaa;
            target.is_active = true;
            target.order = camera.order.saturating_sub(1);
            target.viewport = camera.viewport.clone();
            *optic = projection.clone();
            *local = transform.compute_transform();
            *global = *transform;
        }
    }
    let mut live = HashSet::new();
    let mut used_materials = HashSet::new();
    for (entity, part, mesh, source, transform, visible, tag, layers) in &parts {
        if part.kind != ModelKind::Creature
            || !visible.get()
            || layers.is_some_and(|layers| !layers.intersects(&RenderLayers::default()))
        {
            continue;
        }
        let Some(group) = owner_side(entity, &owners, &allegiance) else {
            continue;
        };
        let key = (source.0.id(), group);
        if !materials.contains(key.0) {
            continue;
        }
        if !mask.materials.contains_key(&key)
            || (changed.contains(&key.0) && !used_materials.contains(&key))
        {
            let Some(source) = materials.get(key.0) else {
                continue;
            };
            let variant = benilla_assets::survivors_mask::material(source, color(group));
            if let Some(handle) = mask.materials.get(&key) {
                if let Some(target) = materials.get_mut(handle) {
                    *target = variant;
                }
            } else {
                mask.materials.insert(key, materials.add(variant));
            }
        }
        let material = mask.materials[&key].clone();
        let tag = MeshTag(tag.map_or(0, |tag| tag.0));
        if let Some(mirror) = mask.parts.get(&entity).copied() {
            if let Ok((
                mut target_mesh,
                mut target_material,
                mut target_tag,
                mut local,
                mut global,
            )) = mirrors.get_mut(mirror)
            {
                *target_mesh = mesh.clone();
                target_material.0 = material;
                *target_tag = tag;
                *local = transform.compute_transform();
                *global = *transform;
            }
        } else {
            let mirror = commands
                .spawn((
                    Mirror,
                    mesh.clone(),
                    MeshMaterial3d(material),
                    tag,
                    transform.compute_transform(),
                    *transform,
                    Visibility::Inherited,
                    NoFrustumCulling,
                    RenderLayers::from_layers(&[LAYER]),
                ))
                .id();
            mask.parts.insert(entity, mirror);
        }
        live.insert(entity);
        used_materials.insert(key);
    }
    mask.parts.retain(|source, mirror| {
        if live.contains(source) {
            true
        } else {
            commands.entity(*mirror).despawn();
            false
        }
    });
    mask.materials.retain(|key, _| used_materials.contains(key));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_or_missing_world_view_clears_mirrors_and_disables_the_mask_camera() {
        use bevy::ecs::system::RunSystemOnce;
        for phase in [Phase::Menu, Phase::GameOver, Phase::Playing] {
            let mut world = World::new();
            world.insert_resource(State::new(phase));
            world.init_resource::<Assets<Image>>();
            world.init_resource::<Assets<WowModelMaterial>>();
            world.init_resource::<Messages<AssetEvent<WowModelMaterial>>>();
            let source = world.spawn_empty().id();
            let mirror = world.spawn(Mirror).id();
            let camera = world
                .spawn((
                    ActorMaskView,
                    Msaa::Sample4,
                    Camera::default(),
                    Projection::default(),
                    Transform::default(),
                    GlobalTransform::default(),
                ))
                .id();
            let mut mask = Mask {
                enabled: true,
                camera: Some(camera),
                ..default()
            };
            mask.parts.insert(source, mirror);
            world.insert_resource(mask);
            world.run_system_once(sync).unwrap();
            assert!(!world.get::<Camera>(camera).unwrap().is_active);
            assert!(world.get_entity(mirror).is_err());
            assert!(world.resource::<Mask>().parts.is_empty());
        }
    }

    #[test]
    fn mask_view_tracks_world_target_size_and_pose_without_replacing_resources() {
        use bevy::{camera::RenderTargetInfo, ecs::system::RunSystemOnce};
        let mut world = World::new();
        world.insert_resource(State::new(Phase::Playing));
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<WowModelMaterial>>();
        world.init_resource::<Messages<AssetEvent<WowModelMaterial>>>();
        world.insert_resource(Mask {
            enabled: true,
            ..default()
        });
        let mut source = Camera::default();
        source.computed.target_info = Some(RenderTargetInfo {
            physical_size: UVec2::new(640, 480),
            scale_factor: 2.,
        });
        let view = world
            .spawn((
                WorldCamera,
                Msaa::Sample4,
                source,
                Projection::default(),
                GlobalTransform::IDENTITY,
            ))
            .id();
        world.run_system_once(sync).unwrap();
        let mask = world.resource::<Mask>();
        let camera = mask.camera.unwrap();
        let image = mask.image.clone();
        assert!(!world.entity(camera).contains::<WorldCamera>());
        assert_eq!(world.get::<Camera>(camera).unwrap().order, -1);
        assert_eq!(world.get::<Msaa>(camera), world.get::<Msaa>(view));
        *world.get_mut::<Msaa>(view).unwrap() = Msaa::Off;
        let usage = world.get::<Camera3d>(camera).unwrap().depth_texture_usages;
        assert!(TextureUsages::from(usage).contains(TextureUsages::TEXTURE_BINDING));
        world
            .get_mut::<Camera>(view)
            .unwrap()
            .computed
            .target_info
            .as_mut()
            .unwrap()
            .physical_size = UVec2::new(1280, 720);
        *world.get_mut::<GlobalTransform>(view).unwrap() =
            GlobalTransform::from_translation(Vec3::new(3., 4., 5.));
        world.run_system_once(sync).unwrap();
        assert_eq!(world.resource::<Mask>().camera, Some(camera));
        assert_eq!(world.get::<Msaa>(camera), Some(&Msaa::Off));
        assert_eq!(world.resource::<Mask>().image, image);
        let target = world.resource::<Assets<Image>>().get(&image).unwrap();
        assert_eq!(target.texture_descriptor.size.width, 1280);
        assert_eq!(target.texture_descriptor.size.height, 720);
        assert_eq!(
            world.get::<GlobalTransform>(camera).unwrap().translation(),
            Vec3::new(3., 4., 5.)
        );
        // A minimized or unresolved target must not leave a stale mask active.
        world.get_mut::<Camera>(view).unwrap().computed.target_info = None;
        world.run_system_once(sync).unwrap();
        assert!(!world.get::<Camera>(camera).unwrap().is_active);
        world.get_mut::<Camera>(view).unwrap().computed.target_info = Some(RenderTargetInfo {
            physical_size: UVec2::new(1280, 720),
            scale_factor: 2.,
        });
        world.run_system_once(sync).unwrap();
        assert!(world.get::<Camera>(camera).unwrap().is_active);
        world.insert_resource(State::new(Phase::Menu));
        world.run_system_once(sync).unwrap();
        assert!(!world.get::<Camera>(camera).unwrap().is_active);
    }

    #[test]
    fn detached_item_cards_follow_actor_ownership_and_concealment() {
        use benilla_world::billboard::BillboardCard;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let actor = |kind| crate::net::NetEntity {
            kind,
            display_id: Some(49),
            scale: 1.0,
        };
        let hero = world
            .spawn((
                actor(benilla_protocol::EntityKind::Unit),
                Hero { vy: 0.0, air: 0.0 },
                InheritedVisibility::VISIBLE,
            ))
            .id();
        let joint = world.spawn(ChildOf(hero)).id();
        let card = world
            .spawn((
                BillboardCard::frame_following(
                    benilla_formats::BillboardKind::Spherical,
                    Vec3::ZERO,
                    joint,
                ),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        let neutral = world
            .spawn((
                actor(benilla_protocol::EntityKind::Unit),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        let effect = world.spawn_empty().id();
        world
            .run_system_once(move |owners: Owners, allegiance: Allegiance| {
                assert_eq!(owner_side(card, &owners, &allegiance), Some(2));
                assert_eq!(owner_side(neutral, &owners, &allegiance), Some(0));
                assert_eq!(owner_side(effect, &owners, &allegiance), None);
            })
            .unwrap();
        *world.get_mut::<InheritedVisibility>(hero).unwrap() = InheritedVisibility::HIDDEN;
        world
            .run_system_once(move |owners: Owners, allegiance: Allegiance| {
                assert_eq!(owner_side(card, &owners, &allegiance), None);
            })
            .unwrap();
    }

    #[test]
    fn allegiance_is_gameplay_owned_and_neutral_surfaces_keep_depth_ownership() {
        assert_eq!(color(side(false, true, false, false)), Vec3::X);
        for flags in [
            (true, false, false, false),
            (false, false, true, false),
            (false, false, false, true),
        ] {
            assert_eq!(color(side(flags.0, flags.1, flags.2, flags.3)), Vec3::Y);
        }
        assert_eq!(color(side(false, false, false, false)), Vec3::Z);
        assert_ne!(
            color(0),
            Vec3::ZERO,
            "neutral surface differs from cleared mask"
        );
    }
}
