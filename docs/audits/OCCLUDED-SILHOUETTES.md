# Occluded entity silhouettes

## Current evidence

The cutaway and silhouette requirements are separate. The current
`survivors/visibility.rs` probes rendered scenery and uploads a camera/hero aperture;
`survivors_cutaway.wgsl` removes scenery fragments inside that aperture. Neither
path renders red/green actor silhouettes. Graphics therefore correctly exposes no
silhouette switches yet. This audit does not establish rendered acceptance.

Actors use `WowModelMaterial`, with posed meshes, `MeshTag` palette/alpha payloads,
animated material slots and cutout textures. A generic unlit copy would lose parts
of that contract. `entities/attach/dress.rs` owns the source parts; the mask should
follow those parts, including equipment, instead of generating proxy capsules.
`survivors/perception.rs` hides gameplay-concealed enemies independently of scenery.
A reveal must not undo that visibility or expose a hidden/removed entity.

Retained static scenery renders between `StartMainPass` and `MainOpaquePass` in
`static_gx/render.rs`; ordinary terrain/models also write the main scene depth.
The FFX combine runs after `StartMainPassPostProcessing`, before Bloom. A depth copy
immediately after the retained static pass would miss ordinary scenery and terrain.
The compositor must use the completed relevant scene depth, with an explicit edge
before the FFX combine. Verify ordering rather than relying on node insertion order.

## Rendering contract and next implementation

1. Produce a per-world-view actor mask containing the nearest visible actor surface,
   its allegiance and its depth. Preserve posed mesh/instance tags, equipment,
   source alpha tests, clipping and inherited visibility. Include neutral actors
   and disabled filter groups in depth ownership: a disabled friendly must not
   make a deeper enemy appear through that friendly. Exclude gameplay-hidden actors
   and portrait/Armory views. Clear/reallocate the mask on view/size changes.
2. Composite only where the nearest actor depth is behind the completed scene
   depth, using the same projection, viewport and sample coordinates. Reverse-Z
   means the scene depth is larger. A clear mask, equal depth, scenery behind the
   actor and invalid inputs produce no reveal. Red is enemy, green is friendly;
   neutral/unclassified actors produce no color. Apply the two independent filters
   during compositing, not when constructing depth ownership.
3. Do not implement this as a reversed-depth duplicate mesh pass. A model's own
   back surfaces are behind its front surfaces and would produce false highlights.
   A center-ray gate also cannot establish which portions of a partially occluded
   body are hidden. The nearest actor mask is the required input to the comparison.
4. Select mask attachment format, depth encoding and comparison tolerance from
   measured GPU precision. Do not silently assume half-float alpha has sufficient
   depth precision across the near/far range. Verify Metal and the Windows backend,
   MSAA, render scaling, texture usage and resource cleanup. Reuse source geometry
   and palette data; avoid per-frame material allocation or per-enemy scene rays.
5. Connect persisted enemy/friendly switches and Graphics reset only once the
   renderer consumes them. Cutaway, camera push-in and both silhouette filters must
   remain independent. Disabling cutaway must not disable silhouettes.

`survivors_core::silhouette` now specifies the reverse-Z comparison and independent
filter/color policy. Tests cover visible actors, nearer/farther scenery, background,
invalid depth, tolerance and all filter combinations. It has no renderer consumer
and is not evidence of GPU precision, source-mask correctness or visual behavior.

## Required isolated visual acceptance

Use an isolated noncombat render fixture with an actual posed hero, enemy and
friendly behind a single wall, plus alpha-key foliage. Capture identical scenes
with no blocker, full obstruction, partial obstruction and scenery behind actors.
Include overlapping actors (including neutral and disabled groups), clear mask
pixels, concealment, spawn/despawn, equipment changes and animated poses. Verify
only occluded portions receive plain red/green and normal visible portions retain
normal rendering. Exercise all four filter combinations, save/restart and reset.

Repeat at compact/wide sizes, multiple render scales and camera distances, with
cutaway on/off and ordinary/merged/retained scenery. Check frame time and allocation
stability as actor count grows. Embed actual captures in CHANGELOG/README and PR;
do not use reference screenshots as completion evidence. Attended Elwynn/interior
and Windows acceptance remain necessary after isolated rendering succeeds.
