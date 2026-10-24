# Survivors development and distribution

The game must support quick iteration and a simple player installation. The user wants to
retain compiled engine work, reload as much game content as practical, and eventually offer
updates from the packaged executable. The existing installed Azeroth Survivors application
is a conceptual reference only: do not inspect, decompile, or use it as an implementation source.

## Compile once, reuse the engine

Cargo already stores compiled dependencies under `target/`. The existing `play` profile
inherits release optimization, enables incremental compilation and disables LTO. Keep that
cache, the toolchain, feature selection and profile stable. Do not run `cargo clean` during
normal iteration. Switching among default/no-default features, test/dev/play/ship profiles,
target architectures or compiler flags creates different build artifacts; these are not
recompilations caused by every game edit.

`benilla-app` is still a large crate containing both client and Survivors systems. A one-line
Rust edit can trigger expensive code generation and linking there even when every engine
dependency is fresh. The build-id stamp is already isolated in the small launcher. Further
work must reduce this game/engine boundary, rather than repeatedly adjust global flags and
invalidate the entire cache.

`survivors-core` now owns the engine-independent catalogs, formulas, campaign progression,
equipment and reward rules. It has no Bevy dependency. `benilla-app` consumes its public
modules. `cargo survivors-rules` runs those tests using a small persistent cache under
`target/survivors-rules`, so an ongoing engine build does not block the rule-test loop.
The integration suite still verifies the adapter, persistence and gameplay wiring.

Commands from the repository root:

```bash
cargo survivors-rules
cargo survivors-build
WOW_DATA="$PWD/wow-data/Data" cargo survivors
cargo test -p benilla-app --lib survivors:: --no-default-features
cargo run -p survivors-audit -- "$PWD/wow-data/Data"
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --floor 533 3005.87 -3435.01 293.88
```

The content auditor requires an explicit data directory and never discovers another install.
It checks campaign map rows, terrain under start coordinates, roster model files, item names
and drop coverage. The optional `--floor <map> <x> <y> <z>` probe loads all walking collision
groups of WMO placements referenced by the point's ADT, applies the MODF transform, reports
floor heights and samples nearby rings plus a one-yard grid across a 30-yard disk using
the runtime grounding window. Coordinates use
raw WoW axes. Missing required groups are errors. The example is a Naxxramas entrance probe,
not a selected arena: a point on a floor does not establish a traversable combat area.
The probe excludes doodads, placements referenced only by neighbouring tiles, walls and
connectivity; ring and grid counts must never be treated as a navigation mesh or readiness flag. A nonzero exit reports unresolved content. Indoor WMO floor checks,
traversability, actual appearance identity and rendered UI validation remain separate work;
the auditor's output is not a claim that a whole zone or screen is finished.

Measure cold bootstrap, warm no-op build, launcher-only edit, rule edit and UI-system edit
separately. Use `cargo build --profile play -p warcraft-survivors --timings` and preserve
reports under `target/`. A rule-only test timing does not prove the native rebuild timing.
Do not promise a seconds-long native rebuild until measured on the actual tree.

Next boundary work: extract the remaining pure account/save model; make Survivors ECS/UI a
plugin crate with a narrow engine adapter. This requires explicit interfaces for models,
spell effects, catalogs, world queries and input instead of exposing all private client
modules. Shared engine dependencies should stay compiled when only that plugin changes.
Changing a Rust dependency can still require recompiling consumers; data reloads avoid this.
Evaluate development-only dynamic linking only after a timing baseline, keeping it out of
the distributable build. Compiler caches can complement Cargo for clean/CI builds but do
not replace retained incremental artifacts.

### Outdoor collision-source probe

Run `cargo run -p survivors-audit -- wow-data/Data --doodads 0 -9548.5 115 60`
to inspect outdoor placements around the Elwynn start. The read-only probe lists unique
placement IDs, model paths, positions, rotations, scales and source hull triangle counts,
then groups counts by model. It reads only the explicit data directory. Missing models or
unreadable existing tiles fail the command instead of silently reporting partial success.

Coverage is placement origins within 200 horizontal yards in the surrounding 3×3 existing
ADT tiles; it excludes WMO props and distant-origin geometry overlapping that area. A
nonempty hull establishes source geometry only. Verify runtime transforms, collider attachment,
body sweeps and tile reload separately before claiming a collision fix. Empty parsed hulls
must be investigated rather than automatically replacing the whole render mesh with a collider.

## Reload content without recompiling

The first live surface is `crates/survivors-core/tuning.txt`, a deliberately small key/value
format. Its four keys are `damage`, `movement`, `experience`, and `pickup`; all must be
finite multipliers from 0.1 to 10. Native builds with the `dev` feature poll it every half
second using real time, including while paused. Valid edits apply to the current run;
invalid or incomplete edits retain the last good settings and log the reason. Oversized or
unreadable files are ignored.
There is no file polling in wasm or player builds. Packaged defaults are embedded so the
executable does not require a source checkout or loose tuning file.

Validation: boot the native development game using the repository-local downloaded data,
change one multiplier, confirm the reload log and actual stat/behavior change without a
build, then restore it. Exercise a partial file, unknown key and invalid numeric value;
the running configuration must remain intact. Start a new run after reloading to prove the
current configuration survives run reset. These live checks remain required even when the
parser tests pass.

The Collection layout is a second live surface, `crates/survivors-core/ui.txt`. Its seven
required keys control the left/right column proportions, column gap, panel padding, body
and title text sizes, and grid cell height. Values are bounded and finite; a complete valid
snapshot redraws the page without restarting or losing its selected item. Invalid, duplicate,
unknown or missing fields retain the prior layout. Columns scroll independently with the
mouse wheel. Packaged builds embed the defaults and do not poll this file.

Validate a visible text-size or cell-height edit while the Collection page is open, then
try an incomplete file and an invalid value and confirm the prior layout survives. Restore
the defaults before committing. Check the page at both supported viewport sizes; scroll
long equipment, source and detail content instead of letting it cross the frame.

Expand this pattern to typed, versioned catalogs for item stats, quality progressions,
raid sets, talents, drop schedules, enemy curves and UI theme/layout values. Validate a
whole new snapshot before publishing it; show actionable errors; preserve the previous
snapshot. Stable content IDs must survive reloads and saves. Define which changes apply
live (colors, text, scalar tuning) and which apply at the next draft or run (catalog removal,
equipment identity, encounter ordering). Do not mutate a run halfway through a save or
settlement. Ship embedded known-good defaults, with development overrides kept separate.

Bevy's asset watcher can reload supported loose assets. MPQ-backed game assets and embedded
shaders need their source-specific reload paths; simply enabling `file_watcher` is not proof
that these sources reload. Rust systems/state-layout changes still need compilation and a
restart. Arbitrary Rust dynamic-code reload is not the initial design requirement.

## One player executable and prompted updates

The player receives a native release executable (an app bundle where the platform requires
one), embedded game defaults, and a separate read-only game-data location. The distributable
contains no Rust compiler, build cache or developer watcher. Player saves/settings remain in
the existing local-state folder and must survive an executable replacement.

The present Restart button starts the current binary; Restart local build uses the local
path in `survivors-update.txt`, falling back to the current binary when no path is configured.
A failed launch leaves the working game open and reports the error. A successful process
spawn does not prove the replacement reaches a healthy game window; health checks and rollback
belong to the updater work below. These controls do not check a release server, download
updates or verify them, and must not be advertised as a completed updater.

Proposed player flow: Check for updates → version and release notes → Update and restart.
The game finishes/saves safely before replacement, and permits declining or deferring the
update. It reports failure without destroying the working installation.

Implementation work:

1. Define a versioned manifest: release version/build, platform/architecture, download URL,
   size, cryptographic digest, minimum compatible save/schema version, release notes and a
   signature. Pin the trusted signing key in the executable; transport uses HTTPS. Validate
   the signed payload and downloaded digest before executing any replacement.
2. Publish release binaries and manifests through CI with protected signing material. Keep
   development and stable channels distinct. The channel URL and signing ownership must be
   chosen before a production updater is enabled; do not invent a live endpoint or key.
3. Download into a staged file beside the installation, verify it, then ask for the restart.
   On Windows, the running executable is locked: start a temporary helper copy of the same
   binary in `--apply-update` mode, wait for the parent to exit, replace the executable and
   restart it. On other platforms use their packaging/code-signing requirements and an
   atomic replacement where supported. This preserves a one-executable delivery experience
   without pretending one process can always overwrite itself while running.
4. Keep a recoverable previous binary until the new version completes a health check. Handle
   interrupted downloads, failed signatures, disk exhaustion, permissions, locked files,
   crash-before-health-check and rollback. Back up saves before any irreversible migration;
   never run an old executable against a newer incompatible save after a rollback.
5. Test the update state machine with local manifests/artifacts and injected failures, then
   perform an actual packaged update on each supported OS. A local restart test alone does
   not establish update safety or completeness.

This is the intended updater architecture, not a claim that remote updates exist yet.
Automatic unattended installation is not requested; the user-facing prompt is part of the
product. No third-party implementation or decompilation is needed.

## Technical references

- [Cargo profile and incremental settings](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Cargo build timing reports](https://doc.rust-lang.org/cargo/reference/timings.html)
- [Bevy 0.18 asset loading and hot reloading](https://docs.rs/bevy/0.18.1/bevy/asset/index.html)

## Native validation shutdown

The temporary `Survivors Checkout Validation.app` lives under
`target/survivors-validation/` and uses isolated checkout data and saves. When closing
it through native UI automation, obtain the live window controls, close the window,
and then check the running-app inventory or process exit. Do not request accessibility
state, screenshots, or focus through the old app handle after exit: contacting that
terminated app can produce Finder’s “application is not open anymore” dialog.
Reacquire the app handle only when intentionally launching the next validation run.

Treat the close action as the last operation on that handle, including in combined
action-and-observation calls. Clear the handle immediately. Poll
`cua.listApps({emit: false})` for bundle ID
`local.survivors.checkout-validation` until `isRunning` is false; an immediate
post-close inventory can still report true while shutdown finishes. Do not use
`getApp` as an exit check, since it can launch the app again. If checking for an
OS dialog, inspect Finder separately after confirming exit.
