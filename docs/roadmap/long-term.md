# Physics-first Survivors roadmap

This plan incorporates the user's revised priorities. See [the index](../INDEX.md),
[current architecture](../architecture/overview.md) and [Box3D contract](../architecture/box3d.md).
It supersedes content-bundle-first sequencing, not the existing game's acceptance requirements.

## Priority and dependency order

| Milestone | Scope | Depends on | Acceptance |
| --- | --- | --- | --- |
| P0: baseline | Pin branch/library evidence; choose native hardware, elite rig, cosmetic policy and budgets | This research | Decisions recorded; latest failures distinguished from old ones; no premature green-gate claim |
| P1: Box3D adapter | Pinned C17 build, safe Rust ownership, fixed stepping, static geometry and headless fixture | P0 | Native drop/joint fixture works reproducibly; no renderer dependency; resources reclaimed |
| P2: elite showcase | One actual elite skeleton, animation-to-ragdoll transfer, impulse, attachments, ground/obstacle contact, bounded corpse lifecycle | P1 | Visible convincing death; unchanged rewards/living movement; fallbacks and stream/reset cleanup verified |
| P3: 15 Hz gameplay and replay | Fixed ordered gameplay ticks, separate 60 Hz physics, interpolation, tick-indexed inputs, controlled RNG and geometry order | P1; integrate P2 | Deadline and horde/corpse budget met; graphical gameplay replay and headless physics-fixture replay verified; gameplay timers retain behavior |
| P4: native expansion | More ragdoll profiles; optional dynamic props; measured collision/controller migration and higher-fidelity rendering | P2/P3 for gameplay-affecting physics | Each profile/interaction has collision, pose, lifetime and budget evidence; volumetrics have color/depth and performance acceptance |
| P5: authoring and distribution | Broader typed content, modular editor, live dev preview, versioned exports, optional PostgreSQL drafts/publishing, measured MPQ packaging | Stable enough native behavior boundaries; independent parts can overlap P3/P4 | Representative content edits without Rust rebuild; valid export/reload; production excludes privileged editor; package integrity and update behavior verified |
| P6: connected solo | Accounts/cloud saves; retain local simulation and explicit offline policy | Persistence/content identity contracts | Cross-device save/conflict/retry handling; no duplicated settlement; provenance policy established |
| P7: five-player co-op | Headless authoritative gameplay, parties, commands/snapshots, reconnect and version admission | P3 plus server extraction and P6 identity/persistence | Five clients finish runs; invalid commands rejected; loss/rejoin/retries cannot duplicate rewards; pinned versions enforced |
| P8: persistent world | Zone copies/shards, interest management, ownership/transfer, Survivors activity instances | P7 lifecycle and durable settlement | Crash/transfer cannot duplicate ownership; capacity routing and instance returns preserve state |
| P9: Kubernetes | Fleet allocation, health/drain, rollout, observability, autoscaling and restore procedures | Proven worker lifecycle and measured load | Node-loss/drain/restore exercises; scaling respects active sessions and tick/queue capacity |

The first compelling deliverable is P2, not a content editor or network service. P3 is
required before claiming the game meets the requested 15 Hz/reproduction contract.
P3 does not promise full headless gameplay: its headless fixture exercises the shared
physics adapter with recorded initial bodies, geometry and commands. Full gameplay
extraction remains a dependency of P7; do not infer server readiness from the fixture.
WASM implementation and parity tests are deferred until requested; keep an ordinary-death
fallback and a portable physics boundary, without making browser integration a native blocker.

## Why not Frostbolt first?

Frostbolt was proposed as a small edit/validate/preview/export exercise for a future editor.
It was never the intended size of the game's content or a permanent format commitment.
Physics now provides the more valuable first vertical slice. No Frostbolt export is a
prerequisite for Box3D, elite deaths, fixed ticks or replay.

When authoring becomes the priority, cover a representative set of abilities, enemies,
items, buffs/debuffs, progression and ragdoll profiles. Existing compiled behavior includes
class milestones, absorbs, control and reactive effects; serialization must preserve those
hooks rather than reduce everything to damage/cooldown fields. New behavior may still require
Rust; editing supported definitions should eventually avoid rebuilding the engine.

Keep standalone editor tooling separate, with an optional development-client preview plugin.
Distinguish live entity adjustment from persistent authored definitions. Published immutable
content loads into memory; database authoring/persistence stays outside the combat loop.

## Compatibility before and at 1.0

Pre-1.0 IDs, schemas and bundle layouts may churn. Do not build a permanent compatibility
framework now. Still give each build an explicit schema/content identity, reject unsupported
versions and preserve or explicitly migrate/reset saves rather than silently corrupt them.
Record reset/migration policy with each breaking experimental release. This does not require
backward compatibility across every prototype.

At 1.0, freeze documented stable-ID semantics, schema evolution, deprecation/remapping,
save migration, supported application/content combinations and release rollback policy.
An ID must identify content independently of which archive holds its bytes.

## Campaign version pinning

Accept the user's policy: a hosted campaign uses one specific app version. Make it precise
with an immutable session descriptor: application build, content and collision hashes,
simulation schema, physics revision/settings, ragdoll profile set, tick rates and RNG policy.
For different native binaries, a release manifest identifies each supported platform build
and their shared compatibility identity; byte-identical executables are not required.

Initially require exact compatible release/configuration matches. Keep active campaigns
on their pinned set; new campaigns can use the new release. Do not hot-migrate a running
campaign by default. Version pinning remains useful before 1.0 even while schemas change.
Server-approved commands/initial state remain authoritative; matching physics does not
make client-submitted outcomes trustworthy.

## MPQ packaging recommendation

Separate logical content identity from physical archive layout. MPQ is an asset container;
it does not by itself define content schemas, dependencies, deterministic physics inputs
or session compatibility. One MPQ does not imply loading all its contents into RAM.
The current reader/streamer is a foundation, not evidence of a production MPQ writer,
optimizer, patch generator or publisher. Treat those tools as future deliverables.

| Layout | Advantages | Costs |
| --- | --- | --- |
| One optimized MPQ | Simple installation and atomic selection; straightforward integrity manifest; can pack co-accessed assets together | Small changes may require larger distribution updates; less selective download/reuse unless a patch mechanism is built |
| Strict asset-type MPQs | Convenient separation of textures/models/audio and independent update cadence | One zone/model may span many archives; grouping by type alone may hurt loading locality |
| Shared base plus zone/activity packs | Reuses stable common content and supports targeted updates or later streaming | Requires dependency manifests, duplication control and coherent version selection |

Recommendation: retain the current development asset source, measure load traces and update
sizes, then trial one optimized archive for a bounded native release. Adopt shared base plus
zone/activity packs if selective updates and locality justify the complexity. Prefer those
measurements over committing now to one archive per asset type. Keep collision/ragdoll data
identifiable and loadable without textures so a future server need not ingest visual assets.

Packaging acceptance: reproducible manifest/file hashes, complete dependency closure,
equivalent runtime content, cold/warm read and seek measurements, peak memory, patch size,
interrupted-update recovery and atomic version switching. Runtime must not mix archives from
incompatible releases. Preserve the repository's read-only source-install and asset provenance
rules; packaging architecture is not permission to redistribute third-party assets.

## Decisions and deferred questions

Before P1/P2: select library pin, first native target/elite rig/scene, cosmetic-only policy,
units/gravity, corpse/body caps and budget. Before P3: define overload/pause policy, input
ordering and replay equality, hardware matrix and benchmark duration/load.

Defer permanent schema policy until approaching 1.0, final archive layout until measurements,
and browser physics until that target returns. Co-op still needs explicit pause/draft,
loot, reconnect and solo-economy trust decisions. Begin load-based sharding with whole-zone
copies; dynamic spatial repartitioning is a later design. Kubernetes manages processes;
session ownership, routing, durable settlement and transfers remain application concerns.

This documentation makes no implementation, performance, deployment or full-gate success
claim. All acceptance work is future work unless linked to specific measured evidence.
