//! Experimental completed-scene depth comparison; never affects non-Survivors views.
use super::{targets::SampledMask, ActorMaskView};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{binding_types::*, *},
        renderer::RenderContext,
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

#[derive(Component, Clone, Copy, ExtractComponent)]
pub(super) struct SilhouetteView(pub survivors_core::silhouette::Filters);

#[derive(Component)]
struct Inputs {
    mask: SampledMask,
    pipeline: CachedRenderPipelineId,
    layout: BindGroupLayoutDescriptor,
}

#[derive(Resource)]
struct Pipelines {
    shader: Handle<Shader>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct Composite;

pub(super) fn plugin(app: &mut App) {
    bevy::asset::embedded_asset!(app, "actor_silhouette.wgsl");
    app.add_plugins(ExtractComponentPlugin::<SilhouetteView>::default());
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .add_systems(RenderStartup, init)
        .add_systems(Render, prepare.in_set(RenderSystems::PrepareBindGroups))
        .add_render_graph_node::<ViewNodeRunner<CompositeNode>>(Core3d, Composite)
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::EndMainPass,
                Composite,
                Node3d::StartMainPassPostProcessing,
            ),
        );
}

fn init(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Pipelines {
        shader: assets.load("embedded://benilla_app/survivors/actor_silhouette.wgsl"),
    });
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn prepare(
    mut commands: Commands,
    pipelines: Res<Pipelines>,
    cache: Res<PipelineCache>,
    masks: Query<(&SampledMask, &ExtractedCamera, &ExtractedView), With<ActorMaskView>>,
    views: Query<
        (
            Entity,
            &ExtractedCamera,
            &ExtractedView,
            &ViewTarget,
            &Msaa,
            &SilhouetteView,
        ),
        With<SilhouetteView>,
    >,
    mut last_status: Local<Option<(usize, usize)>>,
    mut variants: Local<
        std::collections::HashMap<
            (TextureFormat, bool, bool, bool),
            (CachedRenderPipelineId, BindGroupLayoutDescriptor),
        >,
    >,
) {
    let status = (views.iter().count(), masks.iter().count());
    if *last_status != Some(status) {
        debug!(?status, "silhouette input availability");
        *last_status = Some(status);
    }
    for (entity, camera, view, target, msaa, filters) in &views {
        let Ok((mask, mask_camera, mask_view)) = masks.single() else {
            commands.entity(entity).remove::<Inputs>();
            continue;
        };
        if camera.physical_target_size != Some(mask.size)
            || mask.samples != msaa.samples()
            || !same_viewport(camera.viewport.as_ref(), mask_camera.viewport.as_ref())
            || view.clip_from_view != mask_view.clip_from_view
            || view.world_from_view != mask_view.world_from_view
            || view.clip_from_world != mask_view.clip_from_world
            || mask_camera.order >= camera.order
        {
            debug!(size = ?camera.physical_target_size, mask_size = ?mask.size,
                samples = msaa.samples(), mask_samples = mask.samples,
                viewport = same_viewport(camera.viewport.as_ref(), mask_camera.viewport.as_ref()),
                projection = view.clip_from_view == mask_view.clip_from_view,
                pose = view.world_from_view == mask_view.world_from_view,
                order = camera.order, mask_order = mask_camera.order,
                "silhouette inputs do not match");
            commands.entity(entity).remove::<Inputs>();
            continue;
        }
        let multi = mask.samples > 1;
        let (pipeline, layout) = variants
            .entry((
                target.main_texture_format(),
                multi,
                filters.0.enemies,
                filters.0.friendlies,
            ))
            .or_insert_with(|| {
                let float = TextureSampleType::Float { filterable: false };
                let layout = BindGroupLayoutDescriptor::new(
                    "survivors_silhouette",
                    &BindGroupLayoutEntries::sequential(
                        ShaderStages::FRAGMENT,
                        (
                            texture_2d(float),
                            if multi {
                                texture_depth_2d_multisampled()
                            } else {
                                texture_depth_2d()
                            },
                            if multi {
                                texture_2d_multisampled(float)
                            } else {
                                texture_2d(float)
                            },
                            if multi {
                                texture_depth_2d_multisampled()
                            } else {
                                texture_depth_2d()
                            },
                        ),
                    ),
                );
                let mut defs = if multi {
                    vec!["MULTISAMPLED".into()]
                } else {
                    vec![]
                };
                if filters.0.enemies {
                    defs.push("SHOW_ENEMIES".into());
                }
                if filters.0.friendlies {
                    defs.push("SHOW_FRIENDLIES".into());
                }
                let pipeline = cache.queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("survivors_silhouette".into()),
                    layout: vec![layout.clone()],
                    vertex: VertexState {
                        shader: pipelines.shader.clone(),
                        shader_defs: defs.clone(),
                        entry_point: Some("vertex".into()),
                        ..default()
                    },
                    fragment: Some(FragmentState {
                        shader: pipelines.shader.clone(),
                        shader_defs: defs,
                        entry_point: Some("fragment".into()),
                        targets: vec![Some(ColorTargetState {
                            format: target.main_texture_format(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    ..default()
                });
                (pipeline, layout)
            });
        commands.entity(entity).insert(Inputs {
            mask: mask.clone(),
            pipeline: *pipeline,
            layout: layout.clone(),
        });
    }
}

#[derive(Default)]
struct CompositeNode {
    reported: std::sync::atomic::AtomicBool,
}
impl ViewNode for CompositeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static Inputs,
        &'static SilhouetteView,
    );
    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        context: &mut RenderContext<'w>,
        (target, depth, inputs, _): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let cache = world.resource::<PipelineCache>();
        let Some(pipeline) = cache.get_render_pipeline(inputs.pipeline) else {
            return Ok(());
        };
        if !self
            .reported
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            debug!(samples = inputs.mask.samples, size = ?inputs.mask.size, "silhouette GPU pipeline ready; encoding depth composite");
        }
        // Flip only after all inputs and the pipeline are ready; always copy the full target.
        let output = target.post_process_write();
        let bind = context.render_device().create_bind_group(
            "survivors_silhouette",
            &cache.get_bind_group_layout(&inputs.layout),
            &BindGroupEntries::sequential((
                output.source,
                depth.view(),
                &inputs.mask.color.default_view,
                &inputs.mask.depth.default_view,
            )),
        );
        let mut pass = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("survivors_silhouette"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Default::default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}

fn same_viewport(a: Option<&bevy::camera::Viewport>, b: Option<&bevy::camera::Viewport>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.physical_position == b.physical_position
                && a.physical_size == b.physical_size
                && a.depth == b.depth
        }
        _ => false,
    }
}
