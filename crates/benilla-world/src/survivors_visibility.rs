//! Opt-in Survivors visibility aperture, uploaded once per frame for both scenery paths.
use crate::lighting::SharedLightBuffer;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::renderer::RenderQueue;
use bevy::render::{Render, RenderApp, RenderSystems};

/// Appended after the fixed-capacity palette; off-world light buffers leave this tail zeroed.
pub(crate) const TAIL_BYTES: u64 = 32;

/// Focus position/radius and camera position/edge feather. A zero radius disables the aperture.
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
