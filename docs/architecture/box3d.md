# Native Box3D and elite-death ragdolls

Status: recommended implementation contract, not implemented. See [the index](../INDEX.md)
for research revisions and [the roadmap](../roadmap/long-term.md) for sequencing.

## Decision and first visible result

Use [Erin Catto's Box3D](https://github.com/erincatto/box3d) for early native physics.
The first deliverable is one approved elite model transitioning from animation into an
articulated ragdoll when killed, settling against real world geometry, then fading cleanly.
This supersedes the earlier recommendation to begin with a Frostbolt content bundle.

Recommend cosmetic corpse physics first: gameplay resolves death, target removal and
rewards once; corpse motion cannot damage actors, block movement, move loot or change
progression. This is a real dynamics/skeletal integration, with a deliberately bounded
gameplay footprint. Physical gameplay interactions can follow after the boundary is proven.
This cosmetic policy is a recommendation, not a previously established user requirement.

Retain Avian's existing world queries and living controller during the first milestone.
Box3D owns a separate dynamics world populated with required static collision data.
Never let two backends independently integrate the same actor. Later replacement of
Avian is a separate movement/query migration, not a prerequisite for ragdolls.

## Upstream capability and limits

Research pin: `f36d061ba26998c87cb7dc593a4065aaa03fb7eb`; choose and freeze the actual
integration release/revision before building. The library is C17; samples are C++20.
The runtime library needs the C runtime and Unix libm, not the sample application stack.
Upstream advertises Windows/Linux/macOS, joints, collision filters, sleeping, recording
and replay. Its human sample is a 12-body rig, not an automatic converter for M2 skeletons.

The [determinism documentation](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/docs/simulation.md#L1995)
describes cross-thread and 64-bit cross-platform determinism with controlled creation
order and floating-point settings: precise MSVC math and disabled floating-point
contraction on GCC/Clang. Preserve the upstream build flags; do not enable fast-math.
The application's determinism remains its own responsibility. This is not a guarantee
for every device, every compiler setting or this game's deferred WASM build.

The [FAQ](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/docs/faq.md#L169)
explicitly disclaims rollback determinism because internal cached state matters. Current
[recording/replay APIs](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/include/box3d/box3d.h#L323)
are useful tools, not proof that restoring transforms/velocities supports multiplayer rollback.
Prove forward reproduction from an identical initial state first; validate any future
snapshot/rollback contract independently.

Documented limitations include static-only meshes/height fields, joint stretching under
extreme mass ratios and fast articulated motion, and an experimental character mover.
Bound impulses and mass ratios; do not assume CCD solves every ragdoll failure.
Sources: [README](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/README.md),
[human sample](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/shared/human.c),
[limitations](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/docs/loose_ends.md#L126).

## Integration boundaries

Proposed crate responsibilities, with names provisional:

- `survivors-physics-sys`: pinned C build and raw FFI; no sample dependencies.
- `survivors-physics`: safe owned world/body/joint handles, commands, stepping and snapshots;
  no renderer, window, audio or Bevy dependency required.
- Native engine adapter: scene geometry residency, model-to-ragdoll profiles, death-event
  consumption and custom skeleton pose output. Keep this separate from pure game rules.

Start with one physics worker. Step and world access must not overlap; respect callback
lifetimes and prevent stale handles after destruction. Add job-system integration only
after profiling and proving its completion semantics; nested blocking jobs can deadlock.
Keep native gating so deferred browser builds can retain ordinary death animation.

### Skeleton and death lifecycle

The engine uses a custom skinning palette, not Bevy SkinnedMesh. `RigPose` contains local
and model-space bones and parent relationships. `PosePost` is the existing writer window
after animation evaluation and before model composition/propagation. Use it to write
physics-driven local bone poses and mark the pose dirty; do not add a second skinning path.

Author a profile for one inspected elite skeleton: body shapes, mass distribution, joint
frames/limits, body-to-bone offsets, collision masks and nonphysics-bone policy. Establish
axis, handedness, scale and length-unit conversions explicitly. The existing game's gravity
is approximately 19.29 world units/s²; do not copy a metres/s² default without choosing units.

At death, retain the current pose and chosen velocities, create the bodies/joints in a
stable order, apply a bounded impulse, and transfer pose ownership from animation to physics.
Keep nonphysics bones and equipment attachments aligned. For future exact replay, record
the initialized body state or deterministically derive it; sampling a render-frame animation
pose is not automatically reproducible on a headless server.

Current `apply_hits` resolves death and loot; `mark_dying` starts a 2.5-second timer before
`reap_dead` requests `DespawnFade`. Replace that presentation timer only for supported
ragdolls. Preserve ordinary death fallback for unsupported rigs and exhausted budgets.
Give each death a once-only identity; repeated processing cannot create duplicate corpses
or rewards. The later fixed-tick milestone adds gameplay tick stamps to this event.

Sources: [RigPose](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/rig_anim/pose.rs#L16),
[PosePost](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-world/src/rig_anim/compose.rs#L274),
[death processing](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-app/src/survivors/abilities.rs#L827),
[corpse lifecycle](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/crates/benilla-app/src/survivors/enemies.rs#L483).

### Collision and residency

Convert source terrain/WMO/doodad collision geometry and placement transforms into Box3D
static shapes; Avian collider objects are not portable inputs. Use source collision, not
the entire visual mesh by default. Validate triangle winding, scale, origin and tile seams.
Share decoded immutable geometry where practical, while keeping backend handles separate.

Pin supporting collision regions while a ragdoll is active, or retire the corpse before
unloading them. Queue creation/destruction at controlled physics boundaries. For deterministic
replay, record/derive the same geometry and insertion order independent of asynchronous
asset completion. A future server loads collision data independently of textures/render distance.

Bound corpse/body/joint counts, lifetime and memory. Freeze/sleep settled bodies and fade
them predictably. Clean up on expiry, run reset, map transition, model failure and shutdown.
Unsupported geometry or models must fall back without affecting combat or progression.

## Timing contract

| Clock | Recommendation | Purpose |
| --- | --- | --- |
| Gameplay | Fixed 15 Hz, 1/15 s per tick | Movement/combat/timers and future authoritative campaign state |
| Box3D | Fixed 60 Hz, 1/60 s per full step | Stable articulated corpse motion |
| Solver | Start with four internal substeps per full Box3D step | Upstream recommended quality baseline, subject to profiling |
| Renderer | Independent frame rate with pose interpolation | Smooth motion without changing simulated outcomes |

There are four full physics steps per gameplay interval; internal solver substeps are
additional. `b3World_Step(world, 1/15, 4)` is not equivalent to four `1/60` calls: full-step
collision/event cadence differs. Upstream recommends full dt <= 1/30 s and normally 1/60 s.
See [simulation timing](https://github.com/erincatto/box3d/blob/f36d061ba26998c87cb7dc593a4065aaa03fb7eb/docs/simulation.md#L132).
If 15 Hz full physics becomes a hard requirement, treat it as an experiment needing explicit
joint/contact stability evidence. The current branch has not implemented fixed 15 Hz gameplay.

The first cosmetic prototype can coexist with current frame-driven gameplay. The subsequent
fixed-tick milestone moves movement, casts, hits, DoTs, cooldowns and buff expiry together
under an explicit ordered schedule. Preserve the latest DoT catch-up, Enrage and absorb
semantics. Input is tick-indexed; visual animation does not decide simulation time.

Use bounded catch-up without a variable large physics step. Local pause freezes both clocks
under a defined policy. Authoritative overload must not silently drop gameplay ticks: record
overload, bound work and apply an explicit slow/capacity/failure policy. Cosmetic quality can
degrade separately, but an exact replay uses its recorded fixed settings and corpse selection.

## Reproduction and future client/server agreement

Identify a run by application build, content/geometry manifest hash, Box3D revision/build
flags, precision and physics settings, ragdoll profiles, tick/step rates, initial state,
RNG algorithm/state and ordered tick-indexed commands. Fix creation/destruction/event order.
Exclude wall clock, pointer addresses, unordered-map iteration and render timing from inputs.
Keep cosmetic RNG separate from gameplay RNG.

Compare canonical gameplay hashes separately from canonical physics body/joint state.
Run identical tapes at 30/60/144 rendering FPS and on the selected native OS/CPU matrix.
Headless and graphical adapters must receive identical initial body states, geometry and
commands before claiming identical physics. Sharing the library alone is insufficient.
Do not use visual similarity or a tolerance-only comparison to claim bitwise determinism.
The server later owns accepted commands and rewards; determinism does not establish trust.

## Acceptance and decisions before implementation

1. Choose the first elite model/skeleton and representative collision scene from inspected
   game data, the Box3D pin, first native target, and subsequent Windows/Linux/macOS matrix.
2. Confirm cosmetic-only first scope; choose corpse cap, lifetime, body/joint budget, impulse
   bounds, units/gravity, and render-to-physics pose handoff policy.
3. Demonstrate the native C/Rust adapter and a headless repeated drop/constraint fixture;
   verify ownership, handles, cleanup and compiler settings before model integration.
4. Show a real elite death without snapping, detached attachments, invalid transforms or
   explosive joints. Cover slopes, stairs, static obstacles, seams, fast impulses and reset.
5. Confirm one death/loot award, no corpse targeting/damage/blocking, fallback behavior,
   bounded resources and correct geometry lifetime across stream-out.
6. Measure gameplay, all four physics steps, pose conversion and geometry work separately.
   At 15 Hz, the whole simulation has 66.67 ms per interval. Proposed engineering target:
   p99 total simulation cost <= 50 ms on declared minimum hardware, with max/overruns and
   catch-up backlog reported. This headroom is a recommendation, not a measured result.
   Throughput alone does not establish smooth rendering: separately target native 60 FPS
   (16.67 ms frame budget) on the selected hardware and agree the p99 frame threshold and
   maximum uninterrupted main-thread work before implementation. A provisional 4 ms physics
   adapter burst limit is a starting budget, not evidence that a 50 ms tick can run inline.
   Measure individual steps and tick spikes; distribute or offload work with explicit
   ownership if necessary rather than blocking a frame with the whole interval's work.
7. Benchmark sustained representative horde load plus maximum corpses and burst deaths;
   report duration, enemy/body/joint counts, cold collision work and frame pacing. No 15 Hz
   acceptance from average timing alone. Select workload/caps before claiming success.
8. Prove replay hashes on the declared native matrix and correct deadline/DoT/expiry behavior
   under fixed ticks, pauses and stalls. Rollback and browser parity remain separate work.

No tests/builds are authorized merely by this plan; execute validation when implementation
or validation is requested. Evidence belongs beside the milestone, with limits explicit.
