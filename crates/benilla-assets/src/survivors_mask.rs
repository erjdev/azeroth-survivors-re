//! Actor-mask material lane. Ordinary models never set this marker.
use crate::materials::WowModelMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{ColorWrites, CompareFunction, RenderPipelineDescriptor};

pub const MARKER: u32 = 0x8000;

/// Copy textures, animation slots and palette buffer unchanged. SIDN is unused
/// by M2 actors; the dedicated shader variant reads it as a categorical color.
pub fn material(source: &WowModelMaterial, allegiance: Vec3) -> WowModelMaterial {
    let mut mask = source.clone();
    mask.extension.clutter_fade.z = ((mask.extension.clutter_fade.z as u32) | MARKER) as f32;
    mask.extension.sidn = allegiance.extend(0.);
    mask.base.depth_bias = 0.;
    mask
}

pub(crate) fn specialize(descriptor: &mut RenderPipelineDescriptor) {
    if let Some(fragment) = &mut descriptor.fragment {
        fragment.shader_defs.push("SURVIVORS_ACTOR_MASK".into());
        if let Some(Some(target)) = fragment.targets.first_mut() {
            target.blend = None;
            target.write_mask = ColorWrites::ALL;
        }
    }
    if let Some(depth) = &mut descriptor.depth_stencil {
        depth.depth_write_enabled = true;
        depth.depth_compare = CompareFunction::GreaterEqual;
        depth.bias = default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::{
        BlendState, ColorTargetState, DepthBiasState, DepthStencilState, FragmentState,
        TextureFormat,
    };

    #[test]
    fn mask_overrides_source_blending_and_depth_flags_to_keep_nearest_surface() {
        let mut descriptor = RenderPipelineDescriptor {
            fragment: Some(FragmentState {
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::empty(),
                })],
                ..default()
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: default(),
                bias: DepthBiasState {
                    constant: 42,
                    slope_scale: 1.,
                    clamp: 1.,
                },
            }),
            ..default()
        };
        specialize(&mut descriptor);
        let depth = descriptor.depth_stencil.unwrap();
        assert!(depth.depth_write_enabled);
        assert_eq!(depth.depth_compare, CompareFunction::GreaterEqual);
        assert_eq!(depth.bias, DepthBiasState::default());
        let fragment = descriptor.fragment.unwrap();
        assert!(fragment
            .shader_defs
            .contains(&"SURVIVORS_ACTOR_MASK".into()));
        let color = fragment.targets[0].as_ref().unwrap();
        assert_eq!(color.blend, None);
        assert_eq!(color.write_mask, ColorWrites::ALL);
    }
}
