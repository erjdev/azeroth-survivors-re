//! Opt-in Survivors visibility aperture, uploaded once per frame for both scenery paths.
use crate::{
    interact::{PickBlob, WorldObject, WorldPick},
    lighting::SharedLightBuffer,
    model_render::ModelKind,
};
use bevy::platform::collections::HashSet;
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

/// Render-geometry sightline query for the fork's scenery reveal. Renderer storage
/// (individual meshes, merged blobs, or retained static batches) stays engine-owned.
#[derive(bevy::ecs::system::SystemParam)]
pub struct SceneryProbe<'w, 's> {
    world: WorldPick<'w, 's>,
    #[allow(clippy::type_complexity)]
    objects: Query<
        'w,
        's,
        (Entity, Option<&'static WorldObject>),
        Or<(With<WorldObject>, With<PickBlob>)>,
    >,
}

impl SceneryProbe<'_, '_> {
    /// True when rendered scenery lies between the eye and focus, excluding the
    /// final half yard at the focus. Walking/camera collision is not consulted.
    pub fn blocked(&self, eye: Vec3, focus: Vec3) -> bool {
        let pickable = self
            .objects
            .iter()
            .filter_map(|(entity, object)| {
                object
                    .is_none_or(|object| scenery(object.kind))
                    .then_some(entity)
            })
            .collect();
        visually_blocked(&self.world, &pickable, eye, focus)
    }
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
    let hits = world.cast(Ray3d::new(eye, direction), pickable, true);
    let blocker = hits.iter().find(|hit| {
        hit.hit.distance < distance - 0.5
            && hit
                .object
                .as_ref()
                .is_some_and(|object| scenery(object.kind))
    });
    trace!(
        target: "survivors_visibility",
        ?eye, ?focus, distance,
        entity_candidates = pickable.len(),
        render_hits = hits.len(),
        blocked = blocker.is_some(),
        blocker = ?blocker.and_then(|hit| hit.object.as_ref()),
        hit_distance = ?blocker.map(|hit| hit.hit.distance),
        "camera-to-hero render probe (entity candidates exclude retained static sources)"
    );
    blocker.is_some()
}
