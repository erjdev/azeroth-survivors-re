#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> opacity: f32;
@group(1) @binding(1) var landscape: texture_2d<f32>;
@group(1) @binding(2) var landscape_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Centered cover crop preserves the landscape's proportions on any viewport.
    let dimensions = vec2<f32>(textureDimensions(landscape));
    let image_aspect = dimensions.x / dimensions.y;
    let view_aspect = in.size.x / max(in.size.y, 1.0);
    let crop = min(vec2<f32>(view_aspect / image_aspect, image_aspect / view_aspect), vec2<f32>(1.0));
    let uv = (in.uv - vec2<f32>(0.5)) * crop + vec2<f32>(0.5);
    let color = textureSample(landscape, landscape_sampler, uv).rgb;
    let vignette = 1.0 - 0.5 * smoothstep(0.25, 0.75, distance(in.uv, vec2<f32>(0.5, 0.4)));
    return vec4<f32>(color * vignette * 0.65 * opacity, opacity);
}
