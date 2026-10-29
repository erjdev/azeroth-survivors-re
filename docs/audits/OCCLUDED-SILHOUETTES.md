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

## Actor-mask implementation checkpoint

`WOW_SURVIVORS_ACTOR_MASK=1` enables an experimental separate actor view in native
builds with `survivors-visibility`. It mirrors visible Creature model parts and their
mesh tags, source materials and live transforms; portrait layers are excluded.
Gameplay ancestry assigns enemy red, friendly green, or neutral blue ownership.
Neutral actors remain in depth ownership for the future compositor.

The dedicated material variant preserves source textures, animation and alpha tests,
then writes categorical color with blending disabled and nearest-surface reverse-Z
depth. Allegiance uses an HDR color target; depth uses a separate sampleable depth
attachment, rather than packing distance into half-float alpha. Camera projection,
viewport, pose and target size follow the world view. Missing/minimized views and
menu/game-over states disable the mask and clear mirrors/material references.

CPU evidence: all 552 Survivors tests pass, including active camera creation,
resize/pose tracking, missing-target disable/recovery and inactive cleanup. A separate
material pipeline test verifies blending overrides and nearest-depth writes. Feature
Clippy and the native incremental build pass. The active-view test caught the invalid
fixed-size layer constructor; the mask now uses the dynamic layer constructor.

This is renderer infrastructure, disabled by default. There is no compositor or
settings consumer yet. CPU lifecycle/pipeline tests do not establish shader compilation,
GPU precision, animated source fidelity, MSAA edges or Windows acceptance. The next
step is mixed-actor obstruction validation and the scene-depth compositor.

## First native mask evidence

With a development build, `WOW_SURVIVORS_ACTOR_MASK_PREVIEW=1` enables the mask in
the menu and displays its color attachment above the starter. Use an isolated
`BENILLA_HOME` and the existing UI preview file to choose a saved hero and size;
no run or combat is needed. Player builds ignore the preview flag. This is a
diagnostic overlay, not a new player-facing interface.

On Apple M1 Max / Metal, the shader compiled with `WOW_RIG_SKIN` and
`SURVIVORS_ACTOR_MASK`, no blending, depth writes, reverse-Z GreaterEqual and one
sample. Captures in `../images/screenshots/2026-10-28-actor-mask/` show a Warrior
at 1100×720 and a live switch to a Paladin at 640×720. The initial capture exposed
a detached weapon billboard classified as neutral; ownership now follows
`BillboardCard::follows()` before ordinary parent links and requires a Unit/Player
owner. Hidden owners reject their detached cards, and ownerless effects are excluded.

This proves the pictured player mask and live replacement/resize on Metal. It does
not prove depth comparison, obscured-only output, enemy/neutral overlap, all poses,
MSAA equivalence or Windows. The separate blocker fixture remains required below.

## Compositor integration constraints

`Node3d::EndMainPass → silhouette node → StartMainPassPostProcessing` provides
an explicit edge before the existing FFX combine while seeing completed scenery
and actor depth. A new extracted marker must identify the source world view;
`WorldCamera` itself is not an extracted component. Match target size, projection
and viewport before consuming the mask, and skip rather than use stale attachments.

The world defaults to 4× MSAA; the experimental mask now follows its sample count.
Bevy 0.18.1 exposes `ViewTarget::sampled_main_texture_view()`, but its multisampled
color attachment is created with **RENDER_ATTACHMENT only** in `prepare_view_targets`.
It cannot simply be bound as a sampled texture. Matching mask depth MSAA while
reading resolved allegiance color also loses per-sample ownership where different
actors overlap. Resolve this with appropriately owned sampleable attachments or an
equivalent verified pass before claiming MSAA edge/overlap acceptance. Do not force
the player's world MSAA off or silently treat mixed resolved colors as one actor.

## Sampleable attachment implementation

`actor_mask_targets.rs` allocates cached Rgba16Float allegiance and Depth32Float
textures with both RENDER_ATTACHMENT and TEXTURE_BINDING usage, matching the mask
camera's sample count and physical target size. Its render node runs after EndMainPass
and before StartMainPassPostProcessing, replaying opaque, alpha-mask and transparent
mask queues into these owned attachments. Viewport and posed source-material pipelines
are reused. Size/sample-count changes replace the cached pair; absent targets remove it.

The current debug preview still displays the ordinary resolved mask. The owned pass
adds a second actor draw while this experimental path is enabled; remove that duplicate
work or measure and justify it before enabling the feature for players. Native Metal
boots at WOW_MSAA=1 and 4 compiled matching mask variants and produced no validation
errors. The screenshots in `../images/screenshots/2026-10-28-actor-samples/` prove only
the pictured resolved preview and compatibility with the added pass. Sampling these
attachments, mixed-actor ownership and depth comparison remain unverified.

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
