#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> opacity: f32;

// Piecewise ridges stay crisp at every resolution. No world camera or elapsed time:
// the menu remains quiet while the player reads and navigates its panels.
fn ridge(x: f32, seed: f32, count: f32) -> f32 {
    let p = x * count;
    let i = floor(p);
    let a = fract(sin(i * 127.1 + seed) * 43758.5453);
    let b = fract(sin((i + 1.0) * 127.1 + seed) * 43758.5453);
    return mix(a, b, fract(p));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let glow = exp(-dot((uv - vec2<f32>(0.62, 0.03)) * vec2<f32>(1.8, 2.8),
                        (uv - vec2<f32>(0.62, 0.03)) * vec2<f32>(1.8, 2.8)));
    var color = mix(vec3<f32>(0.055, 0.018, 0.009), vec3<f32>(0.95, 0.50, 0.12), glow);
    let far = 0.07 + ridge(uv.x, 3.7, 43.0) * 0.075;
    let middle = 0.16 + ridge(uv.x, 7.3, 29.0) * 0.13;
    let near = 0.34 + ridge(uv.x, 12.1, 17.0) * 0.23;
    let edge = max(fwidth(uv.y), 0.0001);
    color = mix(color, vec3<f32>(0.20, 0.06, 0.04), smoothstep(far - edge, far + edge, uv.y));
    color = mix(color, vec3<f32>(0.10, 0.03, 0.02), smoothstep(middle - edge, middle + edge, uv.y));
    color = mix(color, vec3<f32>(0.035, 0.015, 0.01), smoothstep(near - edge, near + edge, uv.y));
    let vignette = 1.0 - 0.6 * smoothstep(0.25, 0.75, distance(uv, vec2<f32>(0.5, 0.4)));
    return vec4<f32>(color * vignette * opacity, opacity);
}
