# Survivors architecture and roadmap

Start here for the engine's current boundaries and intended evolution. This is a
planning/documentation change, not an implemented physics or networking milestone.

## Reading order

1. [Engine overview](architecture/overview.md): current crates, rendering, platform,
   streaming, collision and simulation boundaries, with source references.
2. [Native Box3D and elite ragdolls](architecture/box3d.md): integration seams,
   determinism contract, 15 Hz gameplay / physics stepping, and acceptance criteria.
3. [Long-term roadmap](roadmap/long-term.md): phased scope, dependencies, content and
   MPQ packaging policy, version pinning, and deferred multiplayer/infrastructure.
4. Existing [game specification](SURVIVORS.md), [engine method](METHOD.md),
   [generated map](MAP.md), and [contribution boundaries](CONTRIBUTING.md).

The latest branch also contains its
[gameplay acceptance contract](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/docs/SURVIVORS-IMPLEMENTATION.md),
[development guide](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/docs/SURVIVORS-DEVELOPMENT.md),
and [location evidence](https://github.com/erjdev/azeroth-survivors-re/blob/18291595c70277a53717d67bf1fd1e7c1853315a/docs/SURVIVORS-LOCATIONS.md).
These links are pinned because the local checkout used to prepare this documentation
is older than the reviewed branch. Do not replace newer gameplay documents with older ones.

## Current direction

- Native physics is the next engine investment: Erin Catto's Box3D, demonstrated with
  an elite's animated model transitioning into a physical ragdoll on death.
- Gameplay targets a fixed 15 Hz cadence. Physics uses an independently specified fixed
  step; 15 Hz gameplay is not a requirement to integrate ragdoll joints at 15 Hz.
- WASM is a later target, not an acceptance blocker for the first native showcase.
- Pre-1.0 content schemas and IDs may change. Freeze the compatibility policy for 1.0;
  identify every experimental build/content set now so runs can be reproduced.
- Cooperative campaigns pin the application and its simulation/content configuration.
- Connected solo, five-player authoritative co-op, persistent zones/Survivors instances,
  and Kubernetes remain the long-term direction, after the physics milestones.

## Evidence and maintenance

Research compares `7ca166a1b4cfce88d938fc21f280c083509f28bc` with
`18291595c70277a53717d67bf1fd1e7c1853315a` on `codex/survivors-long-term`: 31 commits.
Box3D source is pinned to `f36d061ba26998c87cb7dc593a4065aaa03fb7eb` for this review.
Review date: 2026-10-24 America/New_York. Remote files were inspected; no application,
build, test suite, service or deployment was run for this documentation task.

Every architecture update should retain four distinctions: code observed at a named
revision, upstream claims, recommended design, and measured acceptance. Record hardware,
compiler flags, library revision and workload with performance/determinism evidence.
Do not infer current validation from historical checkpoints. Do not hand-edit MAP.md;
regenerate it through the repository's generator when the crate map changes.

The root AGENTS.md contains inherited upstream realm/release instructions. The reviewed
fork does not contain the referenced `crates/wenilla-realm` manifest. Treat external
realm deployment instructions as upstream context, not this game's server architecture.
