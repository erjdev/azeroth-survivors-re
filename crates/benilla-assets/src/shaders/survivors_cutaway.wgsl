// Survivors-only camera-to-hero aperture; mirrors survivors_core::visibility::Cutaway.
// The final two shared-light rows are focus/radius and camera/edge feather.
fn cutaway_keep(world: vec3<f32>, pixel: vec2<f32>, focus: vec4<f32>, camera: vec4<f32>) -> bool {
    if (focus.w <= 0.0) { return true; }
    let axis = focus.xyz - camera.xyz;
    let distance = length(axis);
    if (distance <= 0.001) { return true; }
    let direction = axis / distance;
    let relative = world - camera.xyz;
    let depth = dot(relative, direction);
    if (depth <= 0.001 || depth >= distance - 0.5) { return true; }
    let radial = length(relative - direction * depth);
    let radius = focus.w * depth / distance;
    let inner = radius * (1.0 - clamp(camera.w, 0.0, 1.0));
    if (radial >= radius) { return true; }
    if (radial <= inner) { return false; }
    let visibility = smoothstep(inner, radius, radial);
    // Stable screen-space ordered coverage keeps opaque depth and alpha-key foliage consistent.
    let p = vec2<u32>(pixel) % vec2<u32>(4u);
    let bayer = array<f32, 16>(0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.);
    return visibility > (bayer[p.y * 4u + p.x] + 0.5) / 16.0;
}
