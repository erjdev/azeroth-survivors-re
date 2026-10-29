//! Sampleable per-sample allegiance/depth, separate from Bevy's resolve-only MSAA target.
use super::ActorMaskView;
use bevy::{
    core_pipeline::core_3d::{
        graph::{Core3d, Node3d},
        AlphaMask3d, Opaque3d, Transparent3d,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_phase::{ViewBinnedRenderPhases, ViewSortedRenderPhases},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        texture::{CachedTexture, TextureCache},
        view::ExtractedView,
        Render, RenderApp, RenderSystems,
    },
};

#[derive(Component)]
pub(super) struct SampledMask {
    pub color: CachedTexture,
    pub depth: CachedTexture,
    pub size: UVec2,
    pub samples: u32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MaskSamples;

pub(super) fn plugin(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .add_systems(Render, prepare.in_set(RenderSystems::PrepareResources))
        .add_render_graph_node::<ViewNodeRunner<MaskNode>>(Core3d, MaskSamples)
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::EndMainPass,
                MaskSamples,
                Node3d::StartMainPassPostProcessing,
            ),
        );
}

fn descriptor(size: UVec2, samples: u32, depth: bool) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: Some(if depth {
            "survivors_mask_sample_depth"
        } else {
            "survivors_mask_sample_color"
        }),
        size: Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: TextureDimension::D2,
        format: if depth {
            TextureFormat::Depth32Float
        } else {
            TextureFormat::Rgba16Float
        },
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    }
}

fn prepare(
    mut commands: Commands,
    device: Res<RenderDevice>,
    mut cache: ResMut<TextureCache>,
    views: Query<(Entity, &ExtractedCamera, &Msaa, Option<&SampledMask>), With<ActorMaskView>>,
) {
    for (entity, camera, msaa, previous) in &views {
        let Some(size) = camera
            .physical_target_size
            .filter(|size| size.x > 0 && size.y > 0)
        else {
            commands.entity(entity).remove::<SampledMask>();
            continue;
        };
        let samples = msaa.samples();
        if previous.is_some_and(|old| old.size == size && old.samples == samples) {
            continue;
        }
        commands.entity(entity).insert(SampledMask {
            color: cache.get(&device, descriptor(size, samples, false)),
            depth: cache.get(&device, descriptor(size, samples, true)),
            size,
            samples,
        });
    }
}

#[derive(Default)]
struct MaskNode;
impl ViewNode for MaskNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ExtractedView,
        &'static SampledMask,
        &'static ActorMaskView,
    );
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        context: &mut RenderContext<'w>,
        (camera, view, target, _): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let mut pass = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("survivors_actor_samples"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target.color.default_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Default::default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &target.depth.default_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(0.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if let Some(viewport) = &camera.viewport {
            pass.set_camera_viewport(viewport);
        }
        let entity = graph.view_entity();
        // Reuse the already-queued source-material pipelines, mesh tags and palettes.
        // Every mask pipeline writes nearest depth, including source transparent batches.
        if let Some(phases) = world.get_resource::<ViewBinnedRenderPhases<Opaque3d>>() {
            if let Some(phase) = phases.get(&view.retained_view_entity) {
                if let Err(error) = phase.render(&mut pass, world, entity) {
                    error!(?error, "actor mask opaque draw failed");
                }
            }
        }
        if let Some(phases) = world.get_resource::<ViewBinnedRenderPhases<AlphaMask3d>>() {
            if let Some(phase) = phases.get(&view.retained_view_entity) {
                if let Err(error) = phase.render(&mut pass, world, entity) {
                    error!(?error, "actor mask cutout draw failed");
                }
            }
        }
        if let Some(phases) = world.get_resource::<ViewSortedRenderPhases<Transparent3d>>() {
            if let Some(phase) = phases.get(&view.retained_view_entity) {
                if let Err(error) = phase.render(&mut pass, world, entity) {
                    error!(?error, "actor mask transparent draw failed");
                }
            }
        }
        Ok(())
    }
}
