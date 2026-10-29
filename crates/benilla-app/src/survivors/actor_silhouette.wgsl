// Compare each sample before resolving ownership; mixed actor edges are never classified.
@group(0) @binding(0) var scene_color: texture_2d<f32>;
#ifdef MULTISAMPLED
@group(0) @binding(1) var scene_depth: texture_depth_multisampled_2d;
@group(0) @binding(2) var actor_color: texture_multisampled_2d<f32>;
@group(0) @binding(3) var actor_depth: texture_depth_multisampled_2d;
#else
@group(0) @binding(1) var scene_depth: texture_depth_2d;
@group(0) @binding(2) var actor_color: texture_2d<f32>;
@group(0) @binding(3) var actor_depth: texture_depth_2d;
#endif

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(position.xy);
    let original = textureLoad(scene_color, pixel, 0);
    var tint = vec3<f32>(0.0);
    var coverage = 0.0;
#ifdef MULTISAMPLED
    let count = textureNumSamples(actor_depth);
#else
    let count = 1u;
#endif
    for (var sample = 0u; sample < count; sample += 1u) {
#ifdef MULTISAMPLED
        let actor = textureLoad(actor_color, pixel, i32(sample)).rgb;
        let front = textureLoad(actor_depth, pixel, i32(sample));
        let scene = textureLoad(scene_depth, pixel, i32(sample));
#else
        let actor = textureLoad(actor_color, pixel, 0).rgb;
        let front = textureLoad(actor_depth, pixel, 0);
        let scene = textureLoad(scene_depth, pixel, 0);
#endif
        // Depth32Float tolerance is provisional until the distance/backend matrix passes.
        if front > 0.0 && front <= 1.0 && scene <= 1.0 && scene - front > 0.00001 {
            var enabled = false;
#ifdef SHOW_ENEMIES
            enabled = enabled || actor.r > 0.5;
#endif
#ifdef SHOW_FRIENDLIES
            enabled = enabled || actor.g > 0.5;
#endif
            if enabled {
                tint += vec3<f32>(actor.r, actor.g, 0.0);
                coverage += 1.0;
            }
        }
    }
    let weight = 1.0 / f32(count);
    return vec4<f32>(original.rgb * (1.0 - coverage * weight) + tint * weight, original.a);
}
