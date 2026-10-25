# Current engine and intended boundaries

Evidence is pinned to the revisions in [the index](../INDEX.md). This describes code
and documented branch evidence; it does not certify runtime behavior on this checkout.

## Current composition

| Layer | Existing responsibility | Next boundary |
| --- | --- | --- |
| `warcraft-survivors`, `wenilla` | Native and WASM entry into the Survivors app | Keep platform boot separate from game rules |
| `survivors-core` | Bevy-free catalogs, formulas, campaign, equipment, rewards and talents | Extend shared rules; it is not yet a headless simulation |
| `benilla-app/src/survivors` | Frame-driven combat, actors, account, UI and presentation adapters | Fixed gameplay schedule, explicit death events, later headless simulation |
| `benilla-world` | World streaming, rendering, custom skeletons, collision queries | Box3D adapter and shared collision-data boundary |
| `benilla-assets`, format crates | MPQ asset readers, decoders, material/mesh resources | Preserve source identity independently of packaging |
| `local_state` | Native files; browser localStorage abstraction | Later authenticated persistence and durable settlement |
| `wenilla-host` | Local static/data server and legacy WoW traffic relay | Not a Survivors simulation server |

Source: [workspace manifest](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/Cargo.toml),
[shared rules](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/survivors-core/src/lib.rs),
[Survivors plugin](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-app/src/survivors/mod.rs).

## Rendering and platforms

The workspace pins Bevy 0.18.1 and wgpu 27. Native and browser share Rust gameplay;
target-specific adapters supply window/canvas, I/O, audio and storage behavior.
Browser world rendering requires WebGPU; WebGL2 is not a full world fallback.

The approximate flow is asset decode -> streamed world/placements -> lighting and
animation resources -> retained static geometry -> remaining opaque/transparent scene
rendering -> custom postprocessing -> UI/output. This is not an exhaustive pass list.
World scheduling exposes Net -> Input -> Stream -> Present; it is not an authoritative
fixed-step gameplay schedule. Survivors combat still runs through Bevy Update.

Custom WGSL reproduces WoW gamma-space lighting. StandardMaterial supplies some texture,
alpha and culling state, but this is not a conventional PBR pipeline. The retained StaticGx
pass is inserted before MainOpaquePass; FfxGlow is inserted before Bloom. Static scenery
and entity/material rendering must both participate in future lighting improvements.

Sources: [materials](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-assets/src/materials.rs),
[static graph insertion](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/static_gx/render.rs#L795),
[postprocessing](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/ffx_glow.rs#L1200).

Higher-fidelity rendering remains desirable after the physics showcase: first document
color space, depth ownership and transparency, then prototype a bounded optional volumetric
effect. Preserve water/UI composition and both retained/entity paths. Modern material maps
need authored metadata or explicit defaults; existing WoW assets do not supply them automatically.

## Streaming and collision

Terrain streaming manages ADT residency around the viewing focus, shared placement lifetimes,
terrain, clutter, liquids and colliders. Work is staged with a 4 ms terrain spawn budget.
Browser collider admission has a 2 ms budget, but individual mesh construction is indivisible.
These are scheduling mechanisms, not measured guarantees for every frame.

Native reads local MPQs. Browser Bevy asset reads use asynchronous HTTP, while direct Chain
reads can still use synchronous XHR. Boot manifests prefetch known assets; they do not remove
all synchronous decode or construction. Later streaming work should address memory limits,
eviction, priorities, cancellation, versioned cache identity, failure handling and teleport readiness.

Avian3D 0.6 is configured for spatial queries and shape-cast movement. Dynamics, contact
solvers, joints and related systems are disabled. Box3D will initially coexist with this
query-only world rather than immediately replace established actor movement.

Sources: [streamer](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/terrain_stream.rs),
[world plugins](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/world_plugins.rs#L21),
[browser reads](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-formats/src/web.rs).

## Latest branch comparison

The [31-commit comparison](https://github.com/erjdev/azeroth-survivors-re/compare/7ca166a1b4cfce88d938fc21f280c083509f28bc...18291595c70277a53717d67bf1fd1e7c1853315a)
adds meaningful gameplay and validation work, not Box3D or a multiplayer server:

- Twenty-five Warrior talent nodes have implemented effects, including reactive Enrage;
  Arms/Fury/Protection milestones, control durations, absorbs and bleed behavior expanded.
- DoTs now catch up across long frames and respect expiry. Preserve these semantics when
  moving to fixed ticks; do not assume a tick-schedule change is behavior-neutral.
- Codex discovery, jewelry artwork/descriptions and spell-specific legendary bonuses improved.
- Native dev tools moved behind the existing development entry point; cooldown material
  warmup and several earlier locale assumptions were addressed.
- Terrain/location surveys, a placed-stump capsule regression and a developer world preview
  provide fixtures for collision work. They do not establish general navigation or ragdoll support.

Recorded validation has advanced beyond the earlier 4,294-pass snapshot. The later complete
workspace report records 7,917 passed, five failed, 18 ignored across 143 suites, with failures
in reference strings, UI VM dependencies, web assets and the world API boundary. The latest
Enrage checkpoint records a native build, workspace test compilation, 54 core tests, 90
focused Survivors tests and the talent-data audit passing; manual combat remains pending.
Focused passes do not prove the full gate is green. These are branch reports, not new runs.
See the pinned [changelog](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/docs/CHANGELOG.md).

## Long-view separation

Content definitions, simulation, presentation and persistence remain separate design concerns.
An elite death should produce a single gameplay outcome plus a presentation event. The
Box3D corpse consumes that event without controlling rewards. A future headless process can
execute the same physics adapter with the same inputs when exact corpse reproduction matters.

Later server extraction must remove single-hero assumptions, frame-dependent execution,
global pause semantics and renderer-dependent collision lifetimes. Server authority still
owns accepted commands and rewards even if client/server physics reproduce identically.
WASM validation is explicitly deferred; shared Rust and an upstream Emscripten build do not
prove this repository's wasm32-unknown-unknown integration.
