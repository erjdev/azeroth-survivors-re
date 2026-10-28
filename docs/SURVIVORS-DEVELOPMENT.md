# Survivors development and distribution

The game must support quick iteration and a simple player installation. The user wants to
retain compiled engine work, reload as much game content as practical, and eventually offer
updates from the packaged executable. The existing installed Azeroth Survivors application
is a conceptual reference only: do not inspect, decompile, or use it as an implementation source.

## Compile once, reuse the engine

Both Survivors launchers (`warcraft-survivors` and browser `wenilla`) explicitly
enable `survivors-visibility`, `survivors-terrain` and `survivors-text` on
`benilla-app`. Keep these aligned: the browser build uses `--no-default-features`,
so gameplay carries must not depend on launcher defaults. `scripts/web-build.sh`
defaults to WebGPU; `WEB_BACKEND=webgl2` is the glue-screen-only fallback.
Use `WEB_PROFILE=play scripts/web-build.sh` for incremental browser iteration.
It retains `target/wasm32-unknown-unknown/play`, skips optimization/compression,
and deletes stale compressed siblings before the local host can serve them.
The first build still compiles this target/profile. Rebuild then reload the browser;
this is not Rust hot reload. Default release/ship builds retain distribution
postprocessing, and `WEB_DEBUG=1` preserves symbols in either mode.
The build script checks the `wasm-bindgen` CLI against `Cargo.lock` before compiling.
For a checkout-local CLI (current pin 0.2.122), use:

```sh
cargo install wasm-bindgen-cli --version 0.2.122 --locked --root tools/wasm-bindgen --target-dir target/wasm-bindgen-tools
PATH="$PWD/tools/wasm-bindgen/bin:$PATH" WEB_PROFILE=play scripts/web-build.sh
```

Keep the version aligned with `Cargo.lock` when updating dependencies. The local
CLI and its build cache are ignored; this does not alter a global CLI installation.


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
# Match the PR gate, including render-only foliage detection and reveal fading:
WOW_DATA="$PWD/wow-data/Data" cargo survivors-test
cargo survivors-clippy -- -D warnings
# Separate shipping configuration check (builds a different artifact variant):
cargo test -p benilla-app --lib survivors:: --no-default-features
cargo run -p survivors-audit -- "$PWD/wow-data/Data"
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --collection
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --achievement-art
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --floor 533 3005.87 -3435.01 293.88
```

### Keep the development cache warm

Use `cargo survivors-test` consistently for the integration loop; CI uses the same
alias. It selects `benilla-app`, library tests, the `survivors::` filter and the
`survivors-visibility` feature. `cargo survivors-clippy -- -D warnings` checks the
same library feature set. Both retain the normal `target/` cache and respect Cargo
configuration, including `CARGO_TARGET_DIR`. Extra test filters can follow `--`,
for example `cargo survivors-test -- --skip source_model`.

Run native builds, integration tests and Clippy sequentially: they share Cargo's
build-directory lock, so starting all three together queues work rather than
making it parallel. `cargo survivors-rules` has its own small persistent target
folder and can run alongside them. A previously built UI preview can also run
without Cargo using the fixture launcher's `--executable` option.

Keep the same profile, feature set, toolchain and compiler flags between ordinary
iterations. Cargo keeps distinct artifacts when those inputs change; switching to
`--release`, changing debug flags or testing without defaults is a separate build,
not a faster repeat of the current one. Do not clean `target/` between iterations.
These aliases prevent accidental variant switches; they do not remove recompilation
when Rust source changes, shorten first builds, or isolate the large app crate.
Shipping and browser configurations still need their own acceptance checks.

### Windows checkout launchers

Run `scripts\survivors.cmd` from cmd, or `./scripts/survivors.ps1` from PowerShell.
Both change to this checkout's root, use its `wow-data/Data` when `WOW_DATA` is
unset, and preserve an explicit `WOW_DATA` override. They invoke `cargo survivors`
on the incremental play profile and return Cargo/the game's exit status to the
caller. PowerShell callers can inspect `$LASTEXITCODE`; cmd callers can inspect
`%ERRORLEVEL%`. A failed build does not start an older executable.

The Windows CI job runs `python -m unittest discover -s scripts -p test_survivors_launchers.py -v` before compiling. Its temporary mock Cargo checks
both launchers from outside the checkout, paths containing spaces, absent/local/
explicit data paths, and successful/failing native exit codes without a game
install. This test deliberately skips on non-Windows hosts; a local skip is not
Windows validation.

### Windows build artifacts

The `check` workflow's `windows-survivors` job checks the development launcher,
runs the core rules, and links a player executable with:

```powershell
cargo build --locked --profile play -p warcraft-survivors --no-default-features
```

After a successful job, its run page offers a `survivors-windows-<sha>` artifact
containing only `warcraft-survivors.exe`, retained for seven days. The SHA identifies
the checked-out commit (for PR runs, normally GitHub's test merge). Extract the
archive into a writable playtest folder and point it at your own 1.12.1 data:

```powershell
$env:WOW_DATA = 'C:\path\to\WoW\Data'
.\warcraft-survivors.exe
```

The player build uses embedded layout defaults and does not watch source edits.
It contains no game data, credentials or account saves. It is a playtest artifact,
not a signed release or an automatic update. CI linking does not establish GPU,
foliage, navigation or live gameplay acceptance.

As of the 2026-10-27 audit, GitHub reports zero repository workflow runs. Workflow
registration is visible, but Actions enablement/approval settings could not be
verified with the available access. No Windows artifact or passing Windows build
is claimed until an actual run completes. Check the repository's Actions page for
an enablement or approval prompt when signed in as its owner.

### Diagnosing scenery reveal on Windows

For a manual playtest from this checkout, enable the dedicated visibility trace in
PowerShell (use the checkout's own data path):

```powershell
$env:WOW_DATA = Join-Path $PWD 'wow-data/Data'
$env:RUST_LOG = 'info,survivors_visibility=trace'
cargo survivors 2>&1 | Tee-Object -FilePath target/visibility-playtest.log
```

Enable **Scenery cutaway** in Options. Record `git rev-parse HEAD`, the zone,
and a screenshot where foliage hides the hero. Hold the camera still for a second, then move it until the hero is visible
and capture that adjacent state. Quit normally. Restore your previous `RUST_LOG`
value afterward (or remove it with `Remove-Item Env:RUST_LOG` if previously unset).

At each 100ms probe, the trace records camera/focus coordinates, render-hit count,
nearest qualifying blocker identity/distance, aperture radius and reveal strength
before that frame's fade step. `blocked=true` should approach strength 1 over
500ms; a clear sightline should approach 0. Debug level records only blocked/clear
transitions. Entity candidate counts exclude the retained static renderer, which
is still queried; zero candidates alone does not mean there is no scenery.

A hidden hero with `blocked=false` points to the sightline/geometry path. Sustained
`blocked=true`, positive radius and strength near 1 without a visible aperture
points to the rendering path. Render triangles are not texture-alpha samples, so
transparent leaf texels remain a separate accuracy limitation. These diagnostics
do not themselves establish that a reported Windows scene is fixed.

### Content audits

The content auditor requires an explicit data directory and never discovers another install.
`--talent-evidence <Talent.dbc ID>` prints each rank's raw spell effects, duration,
prerequisites and current Survivors definition. For example,
`cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1312` inspects
Counterattack. Unknown IDs fail. This is source evidence, not combat acceptance;
the [Hunter dependency audit](audits/HUNTER-TALENT-DEPENDENCIES.md) demonstrates
how to distinguish a missing underlying mechanic from a missing rank modifier.
`--achievement-art` checks every catalog icon file for presence/readability; rendered
appearance and visual suitability still require native screenshots.
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
The native polling, F12 screenshot capture and collision probe are registered through
`dev::DevProbesPlugin` from `capture/survivors_tools`; gameplay owns only the runtime
resources. There is no file polling in wasm or player builds. Packaged defaults are embedded so the
executable does not require a source checkout or loose tuning file.

Validation: boot the native development game using the repository-local downloaded data,
change one multiplier, confirm the reload log and actual stat/behavior change without a
build, then restore it. Exercise a partial file, unknown key and invalid numeric value;
the running configuration must remain intact. Start a new run after reloading to prove the
current configuration survives run reset. These live checks remain required even when the
parser tests pass.

The Armory layout is a second live surface, `crates/survivors-core/ui.txt`. Its seven
required keys control the left/right column proportions, column gap, panel padding, body
and title text sizes, and grid cell height. Values are bounded and finite; a complete valid
snapshot redraws the page without restarting or losing its selected item. Invalid, duplicate,
unknown or missing fields retain the prior layout. Columns scroll independently with the
mouse wheel. Talents also consumes body/title sizes, panel padding and gaps;
its painted trees and node positions retain their proportional geometry. Changes
redraw the selected talent/spec without a Rust rebuild. Packaged builds embed
the defaults and do not poll this file.

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

## README captures

For native development builds, set `WOW_SURVIVORS_UI_SHOTS` to an explicit output
directory and press F12 on the desired screen. Each press saves the rendered primary
window to a new `ui-NNNN.png`, including UI; existing numbered files are skipped. The log
records logical/physical viewport sizes and UI scale. Check that the file was actually
written and inspect it before using it in the README. This capture path does not enable
autoplay, change saves or tune gameplay. Use an isolated `BENILLA_HOME` for fixture accounts.

## Streamed collision probes

Native development builds accept `WOW_SURVIVORS_COLLISION_PROBE=/absolute/path/points.txt`.
The repeatable Elwynn points are in
`docs/probes/survivors-elwynn-trees.txt` (map 0).
Each non-comment line contains `label wow_x wow_y wow_z`; obtain positions from the source
placement audit, and load the matching map near those points. After the loading cover clears,
all geometry backlogs settle and sixty more frames pass, the probe logs four horizontal
12-yard queries per point using the same capsule radius and floor helpers as the hero.
It does not move the hero, change the save, add colliders or alter the world.

Compare `slide advance`, `travel advance` and `line clear` with a nearby open-space control.
A short advance alone does not identify the blocker: terrain, walls and neighboring trees
can also intersect the sweep. A clear path through a placement origin does not prove its
whole model lacks collision; the authored trunk can be offset from that origin. Use these
queries to distinguish source-hull tests from live streaming evidence, then reproduce actual
walking and mobility at the reported tree. Unloaded points or a mismatched map invalidate
that point's result. This is collision evidence, not a navigation/pathfinding test.

The source doodad audit also emits `SOURCE RAY` and `SOURCE SLICE` for tree origins within
50 yards of the requested point. Rays cross the origin from four directions at one yard
above ADT terrain; they test only that placement's transformed hull. Slices at one yard
above the placement origin expose offset geometry and provide a surface sample for the
live point file. Neither a slice bounding box nor a clear origin crossing identifies the
visible trunk by itself. Compare identical ray heights; runtime floors may include props
or buildings that the source terrain query excludes.

## Following feature milestones

Read [CHANGELOG.md](CHANGELOG.md) for grouped feature summaries and validation limits.
Keep new entries short; add screenshots under `docs/images/screenshots/<milestone>/`
without overwriting earlier milestones. **Embed actual captures inline in the changelog**
using `![description](images/screenshots/...)`; links alone do not satisfy this requirement.
For each major visual checkpoint, inspect the matching `docs/intentions/` image first,
capture the changed interface, and summarize remaining design gaps. Record viewport,
fixture/state and validation limits. Update the matching page in the README gallery
instead of prepending another milestone gallery. Keep representative captures for
each interface portion, date older gameplay evidence explicitly, and retain every
inline changelog image. Earlier README iterations live in [SCREENSHOT-HISTORY.md](SCREENSHOT-HISTORY.md).
If capture is
blocked, report the blocker and leave visual acceptance open; do not silently drop
the screenshots or accumulate further unreviewed visual milestones.
Commit and push coherent checkpoints, including screenshots and concise notes, to `origin`
when working on a feature branch.

To follow the current implementation on another checkout:

```bash
git fetch origin
git switch codex/survivors-long-term
git pull --ff-only
cargo survivors-build
```

A first build compiles dependencies; subsequent builds reuse the local `target/` cache.
Game data and build artifacts are not in Git. Use `scripts/fetch-data.sh` when needed,
then launch with `WOW_DATA="$PWD/wow-data/Data" cargo survivors` on macOS/Linux.

## Hold a chest hand for UI inspection

`WOW_SURVIVORS_AUTO=Warrior:chest WOW_SURVIVORS_MAP="Elwynn Forest" cargo survivors`
uses the existing accelerated late-run setup (600 simulated seconds and max-rank abilities)
and stops automatically choosing cards while a chest is open. Pick manually to continue;
subsequent chest hands also wait. Ordinary level-ups still auto-pick. Use an isolated
`BENILLA_HOME` and explicit `WOW_DATA` for validation: this mode grants accelerated
combat/rewards and is not evidence of normal progression or balance.

## Hold all upgrade cards for UI inspection

`WOW_SURVIVORS_AUTO=Mage:cards WOW_SURVIVORS_MAP=Barrens cargo survivors`
starts with the class's full rank-one ability pool and leaves level-up and chest
choices open until selected manually. It uses normal offers, so advance hands
until the ability being inspected appears. Use an isolated `BENILLA_HOME`:
the run still awards progress. This setup is for wrapping, resize and input checks,
not ordinary progression, balance or performance acceptance.

## Measure attended performance explicitly

On macOS, unattended probes normally open behind other windows. An occluded or background
window can be throttled by the OS; those FPS numbers do not establish attended gameplay
performance. Set `WOW_BG=0`, keep the game visible and focused, and finish compilation
before recording a comparison. Preserve resolution, display refresh/presentation settings
and scenario; report crowd counts and actual combat seconds. Accelerated runs are not
balance or normal-progression evidence.

For a foreground native probe from this checkout on macOS/Linux:

```bash
cargo survivors-build
mkdir -p target/survivors-profile
WOW_DATA="$PWD/wow-data/Data" \
BENILLA_HOME="$PWD/target/survivors-profile/state" \
WOW_BG=0 WOW_UNATTENDED=1 WOW_NOSOUND=1 WOW_WIN=1440x900 \
WOW_SURVIVORS_AUTO=Warrior:late WOW_SURVIVORS_MAP="Gurubashi Arena" \
WOW_FPS_JOURNAL="$PWD/target/survivors-profile/frames.csv" \
target/play/warcraft-survivors
```

Use a fresh isolated state directory for a new acquisition comparison. Replace `:late`
with `:chest` to hold a chest for paused-scene measurements. The auto log reports frame
count, unfocused frames and frames whose focus could not be read over the entire sample,
alongside combat-only FPS and its longest frame. A mixed/unknown focus sample must be
identified as such; focus alone is not proof that the window was visible or unthrottled.

Use `WOW_FRAME_PHASES=25` only in a separate diagnostic run. It reports frames over 25 ms,
splits main schedules and world stages, and groups the remaining time as render+present.
Its exclusive stamps change scheduling, so do not attribute differences between an
instrumented and uninstrumented run to a game change. Empty GPU columns in the FPS journal
mean unavailable measurements, not zero GPU cost.


## Starter UI iteration priority

The next development focus is the starter/menu and Armory pages. UI previews must
open chosen production pages against deterministic isolated fixtures, without a
full gameplay session. Add direct page selection, viewport presets and rapid
layout/content reloads while retaining the compiled engine cache. The existing
F12 capture and held-card modes are supporting tools, not the complete fast-preview
workflow. Real-time interface transitions must stay independent of the later
unlockable 1.1×/1.25×/1.5×/2× simulation-speed option.


Native development builds now accept direct page entry:

```sh
WOW_DATA="$PWD/wow-data/Data" BENILLA_HOME="$PWD/target/ui-preview-state" \
WOW_SURVIVORS_UI_PAGE=collection cargo survivors
```

Page names: `starter`, `run-options`, `heroes`, `talents`, `rewards`, `collection`, `codex`,
`achievements`, `notes`, `drops`, `look`. This selects the initial Armory page;
it does not start autoplay or a run, seed progression, or bypass unlock rules.
Use an isolated fixture account when a page needs earned content. Unknown names
warn and open the starter page. Player builds ignore this development switch.
Collection's existing `crates/survivors-core/ui.txt` controls can be edited while
this page remains open; broader page hot reload and deterministic fixture presets
still need implementation.

Codex previews can select a detail directly with `entry = legend:wrath`,
`entry = ring:0`, `entry = trinket:0`, or `entry = amulet:0`. Legendary keys come
from the actual class ability catalogs; shared jewelry uses zero-based catalog
indices. This key is valid only with `page = codex`. Omit it to preserve the manual
selection. Unknown keys, out-of-range indices and duplicate `entry` lines reject
the entire request, preserving the current view and size. Inspection does not
mark an item discovered, grant it, or alter the saved hero. Use the checked-in
`codex-wrath-1280.txt` and `codex-ring-640.txt` presets to compare descriptions
without navigating the catalog or playing a run.

For page and viewport changes without restarting, create a local control file:

```sh
mkdir -p target/ui-preview
cat > target/ui-preview/view.txt <<'EOF'
page = collection
width = 1280
height = 720
EOF
WOW_DATA="$PWD/wow-data/Data" BENILLA_HOME="$PWD/target/ui-preview-state" \
WOW_SURVIVORS_UI_PREVIEW="$PWD/target/ui-preview/view.txt" cargo survivors
```

Edit that file while the menu is open: for example use `page = talents` and
`width = 1024`, `height = 768` to inspect the production talent page at 4:3.
All three keys are required. Dimensions are logical window pixels, bounded to
640–7680 wide and 480–4320 high; the desktop may constrain the actual window size.
Comments beginning with `#` are allowed. Complete valid changes apply together
within the next 250 ms menu poll. Invalid, incomplete or missing files keep the
previous view. The control only applies new file contents, so ordinary navigation
still works; edit the file again to request another view. It releases text-entry
focus but preserves name drafts and account data. Changes wait while a run is
active and apply when returning to the menu. This opt-in native development tool
is excluded from player and wasm builds. It does not seed fixture accounts.

Character presets (`page = heroes`) are available as `character-640x480.txt`,
`character-640x720.txt` and `character-1024x768.txt` in `docs/ui-previews`.
They inspect the selected saved hero; `look` opens the separate legacy screen.

Talent viewport presets are available in `docs/ui-previews/talents-640.txt`,
`talents-1024.txt` and `talents-1280.txt` for compact, 4:3 and wide checks. Copy one
over the watched file to resize the same running menu. They use the selected hero
and spec without spending points; manual clicks still perform normal talent actions,
so use an isolated fixture account for interaction checks.

Add `hero = <saved character ID>` to select an existing fixture hero together with
any preview page. The request uses that hero's class and active spec, including for
`profile=selected` cards. Omit it to keep the current hero. Unknown IDs or talents
outside that hero's class reject the complete request without changing the hero,
page or dimensions. Preview selection does not write the saved menu pick, create
heroes, spend points, or load another map. Normal subsequent UI actions still save;
use an isolated `BENILLA_HOME` fixture. IDs are the first number on `who` rows in
that fixture's `survivors-account.txt`. Start with
`docs/ui-previews/hero-talents-1280.txt` and replace its example ID.

To pin a particular talent description, add `talent = <Talent.dbc ID>` on the
Talents page. For example, `docs/ui-previews/warlock-grim-reach-1024.txt` inspects
Grim Reach (1021) for an already-selected Warlock. The ID must belong to that hero's
loaded talent tree; missing heroes, unknown IDs, another class's nodes, duplicate
keys and use on another page reject the entire request without resizing or changing
pages. A loading catalog delays application until validation is possible.

The description stays pinned even if the pointer rests over another node. It shows
the actual saved rank and does not focus, train or refund a node. Removing the
`talent` key in a valid reload restores normal pointer/keyboard inspection. Actual
mouse/keyboard purchases still work, so use an isolated account when testing them.
Changing file contents is required to retry a rejected request after selecting a hero.

Collection and Drop Tables accept `piece = <band>:<slot>`, for example `piece = epic:hands`
or `piece = legendary:main hand`. Names are case-insensitive and use the current
catalog: Poor, Common, Uncommon, Rare, Epic, PvP, Legendary; Helm, Shoulders, Chest,
Waist, Legs, Feet, Wrist, Hands, Main Hand. This selects the inspected cell for the
current class, including unowned pieces, without granting or equipping it. Poor
resolves to the selected class’s supplemental catalog entry rather than a legacy
quality index. `collection-poor-hands-1280.txt` and `collection-poor-hands-640.txt`
provide direct Poor-quality previews. Collection also accepts `piece = raid epic:hands`
(or any of the eight armor slots). Raid previews are rejected on Drop Tables until
raid encounter sources are implemented; Main Hand is not a raid armor slot.
`collection-raid-hands-1280.txt` and `collection-raid-hands-640.txt` provide reusable
presets. Switching back to a legacy rarity clears the supplemental inspection. Normal
mouse/keyboard gear actions remain active; use an isolated account for interactions.
Omitting `piece` preserves the manual selection. Invalid names, Off Hand, repeated
keys or use outside Collection/Drop Tables reject the entire request, preserving page and size.
Wide/compact examples are `docs/ui-previews/collection-epic-hands-1280.txt` and
`collection-epic-hands-640.txt`; compact details remain reachable by scrolling.

Drop Tables accepts `zone = <zone name>`, such as `zone = Duskwood`, with
case-insensitive catalog names. This changes only the browsed drop table, never
the next run's map or stage. Omission preserves the currently browsed zone. Unknown
names, numeric indexes, duplicate keys or use on other pages reject the entire
request. Combine `zone`, `piece` and `hero` to inspect a fixture's loot directly;
inspection may show an item from another zone without inventing a source there.
Examples: `drops-duskwood-poor-1280.txt` and `drops-elwynn-common-640.txt`.

Character, Rewards and Achievements accept an optional `category` key. Category
changes apply with the page and dimensions in the same reload. Omission preserves
the last Rewards/Achievements category; Character defaults to the dressed view.
Unsupported categories, duplicate keys and categories on other pages reject the
whole request.

| Page | Category values |
| --- | --- |
| `heroes` (Character) | `appearance`, `manage` |
| `rewards` | `perks`, `companions`, `tabards` |
| `achievements` | `summary`, `zones`, `classes`, `collection`, `feats`, `lifetime` |

For example, open Feats directly without clicking through the menu:

```ini
page = achievements
category = feats
width = 1280
height = 720
```

Chest offers use `page = cards` with `jewelry = legend:rejuvenation,
legend:power_word_shield, ring:0, trinket:0` instead of `abilities`. Supply one to
four entries; `ring`, `trinket`, and `amulet` use zero-based catalog indices, while
`legend` uses an ability key. Unknown entries, out-of-range indices, and requests
containing both hand types are rejected atomically. `profile = selected` also
works with jewelry. These are preview offers only: clicking cannot grant items,
and leaving cards restores the original run and offer. Reusable wide and compact
requests are in `docs/ui-previews/jewelry-cards-1280.txt` and
`docs/ui-previews/jewelry-cards-640.txt`.

Reusable requests live in `docs/ui-previews/achievements-feats-1280.txt` and
`docs/ui-previews/rewards-tabards-640.txt`. Copy either over the watched file to
switch production screens while the same app remains open. These requests change
only the view; they do not award achievements, spend points or select cosmetics.
Existing category scroll positions and purchase drafts are retained.

To inspect production upgrade cards immediately, use the same file:

```ini
page = cards
width = 1280
height = 720
abilities = smite:1, holy_fire:3, holy_nova:0
```

One to four comma-separated ability keys are supported. Repeating a key at different
ranks allows side-by-side comparisons. A new spell deliberately shows its rank 1→2
preview, as required by the inline-upgrade design. Rank `0` shows a new spell;
`1`–`7` shows that rank's upgrade. Unknown keys, malformed ranks, and maximum-rank
upgrades are rejected. The preview defaults to baseline run stats. Add
`profile = selected` to snapshot the selected character using the production
run-start path: equipment, talents, renown, relics, race and account perks. The
selected profile waits for the talent catalog and a valid class/look selection;
it never enters gameplay or changes the menu's world selection. Edit the file
again to refresh the snapshot after changing a build. `profile = baseline` (or
omitting it) restores baseline stats. Remove `profile` when returning to an Armory
page. This option reads your account; use an isolated fixture for visual checks.

Passive cards use `passives = renew:0, renew:2, renew:4` in place of `abilities`.
Rank `0` previews acquisition and ranks `1`–`4` preview an upgrade; rank `5` is
already maximum and is rejected. Use one card source per request: `abilities`,
`passives`, or `jewelry`. The same one-to-four-card limit and selected/baseline
profiles apply. `profile = selected` includes Improved Renew when trained by the
selected Priest; it does not grant the talent. Requests at
`docs/ui-previews/passive-cards-1280.txt` and `passive-cards-640.txt` compare new,
intermediate and final Renew upgrades without entering gameplay. Leaving preview
restores the original passives along with the original run and offer.

It animates and scrolls the real cards but disables picks, gameplay starts,
and discovery/reward writes. Resize or replace the hand by editing the file; return
to `page = talents` (remove `abilities`) to restore the previous run and offer.
No combat session or level-up is needed. Normal Armory navigation remains available
outside the card overlay.

Repeat the checked long-description hand with the committed presets:

```sh
cp docs/ui-previews/long-cards-1280.txt target/ui-preview/view.txt
# While the preview remains open:
cp docs/ui-previews/long-cards-640.txt target/ui-preview/view.txt
```

These cover Blizzard 7→8, Chain Lightning 6→7 and Shadow Word: Pain 5→6.
At 640×720, scroll down to inspect the third card. They verify baseline descriptions;
additional talent-triggered text still needs separate coverage.

The Character page opens on the dressed preview. Use **Manage heroes** for the
roster, character creation, renown and relic controls, and **Back to character**
to return. Selecting or creating a hero returns to the preview; an account without
a selected valid hero shows management automatically. Direct page-preview requests
reset this local management view without clearing drafts or changing account data.

### Deterministic account fixtures

For the short-window equipment checkpoint, prepare `partial` with
`--preset collection-common-helm-800x600.txt`, then select **Options → Graphics →
Interface size → 125%**. Inspect the common helm, choose **This piece**, unequip it,
visit its source and return. Restart to verify the empty worn slot and retained
look. This preset selects a piece without granting or equipping it.

Prepare a synthetic account without compiling or launching the engine:

```sh
python3 scripts/survivors-ui-fixture.py partial --page collection
python3 scripts/survivors-ui-fixture.py collected --page heroes
```

To prepare and launch a watched preset in one command (Python 3 and Cargo on PATH):

```sh
python3 scripts/survivors-ui-fixture.py partial --preset talents-640.txt --run
```

For layout/content iteration with an already-built native development executable,
bypass Cargo and its build lock explicitly:

```sh
cargo survivors-build
python3 scripts/survivors-ui-fixture.py partial --preset talents-640.txt --run --executable target/play/warcraft-survivors
```

On Windows, use `py -3` and `--executable target/play/warcraft-survivors.exe`.
An explicit path also supports custom target directories or a copied development
build. This mode checks that the file exists and is executable before creating
fixture state, but does **not** check source freshness. Rebuild after Rust changes;
edit the isolated `view.txt` for supported live page/layout changes. Shipping
builds disable development preview switches and are not suitable here. Each launch
still creates a new isolated account and returns the executable’s exit code.

Open a particular saved spec without editing save records:

```sh
python3 scripts/survivors-ui-fixture.py trained --hero 2 --spec 2 --preset talents-640.txt --run
```

`--hero` and `--spec` are paired; specs are numbered 1–3. They require a talent
preset. Only the selected hero's active spec in the new disposable account changes;
all saved spreads, other heroes and the source fixture remain intact. The watched
request selects that hero, replacing any preset hero selection. A class-specific
talent inspection in the preset must still match the chosen hero. Preset validation
happens before creating the home, so a rejected non-talent preset can be corrected
and retried at the same output path.

On Windows use `py -3` instead of `python3`. By default, `--run` invokes `cargo survivors`
directly, using the incremental play profile and no shell. It clears inherited
autoplay/capture and old page/preview settings, then sets checkout `wow-data/Data`
and the new fixture home. Cargo's exit code is returned; a launch failure retains
the fixture for inspection. Without `--run`, only preparation occurs (macOS/Linux
also print a shell command for that exact prepared account).

`--preset` takes a filename from `docs/ui-previews` and overrides `--page`. The
request is copied to the new home's `view.txt`; edit that copy while the app runs
to change page, size or inspection. Repository presets stay unchanged. Presets
with `hero` or class-specific `talent` values must match heroes in the selected
fixture; invalid live requests retain the previous view. The script does not
invent talent ranks or unlock content.

The script uses the exact isolated `BENILLA_HOME` and checkout data path. Each invocation creates a new directory under
`target/ui-fixtures`; `--output PATH` also requires a new directory and refuses
existing directories or symlinks. Fixture edits are saved only in that copied
account. Create another copy to reset it. Use the printed home with the live
page/viewport control above when comparing screen sizes in one session.

| Fixture | Intended UI states |
| --- | --- |
| `fresh` | No named heroes, no collection, default account state. |
| `partial` | Warrior and Mage heroes, 18/54 pieces each, 12 and 6 unspent talent points, mixed worn/empty slots, hidden head and independent shoulder appearance. |
| `collected` | All 54 pieces for all nine classes, ten heroes including two Warriors, 51 unspent points for each class's first hero, varied worn rarities and independent appearance. |
| `trained` | Two Shaman heroes, each with 30 earned points: a 29-point Enhancement build, a 13-point Restoration build and an empty spec. Hero 1 starts Enhancement; hero 2 starts Restoration. No equipment or clears. |

Reproduce the populated talent captures directly:

```sh
python3 scripts/survivors-ui-fixture.py trained --preset shaman-enhancement-1280.txt --run
python3 scripts/survivors-ui-fixture.py trained --preset shaman-restoration-640.txt --run
```

Use `py -3` on Windows. In an existing preview, copy either preset over its printed
`view.txt` to switch hero, inspected talent and viewport without recompiling. Saved
spec edits remain local to that fixture; create a new copy for the original state.

These are explicit synthetic save fixtures, validated by the real Rust account
parser and round-trip tests. They do not add a cheat path to the player build.
They contain no zone clears or fabricated encounter history. Only `trained`
contains spent talent builds. The normal achievement rules may award collection achievements
when the menu loads. Full ownership is not proof that every item model, source,
talent or zone is ready. Granted/locked achievements and collection states can be
inspected; purchased reward fixtures and other classes' populated builds remain future
extensions. The fresh account may acquire normal defaults when the app initializes.


### Account recovery after a failed load

If `survivors-account.txt` cannot be read or parsed, or contains duplicate hero IDs,
the Armory displays a recovery
notice and disables saving and run entry for that session. The original file remains
untouched. Quit, preserve a copy of that file, then restore a known-good backup or
correct its access permissions and restart. To deliberately start over, move the
old file aside before restarting; a missing file follows the normal new-account flow.
The game does not automatically repair corrupt saves or create recovery backups.
This guard covers reported read/parse errors, not every semantically damaged row
accepted by the legacy-compatible parser.


Raid finale source previews: `docs/ui-previews/drops-naxx-raid-1280.txt` and
`docs/ui-previews/drops-naxx-raid-640.txt` select Naxxramas and Raid Epic hands.
These are read-only menu previews; they neither grant gear nor unlock the zone.

Run Options can be inspected without entering gameplay using `run-options-1280.txt`
or `run-options-640.txt` from `docs/ui-previews`. It displays the currently supported
Normal difficulty and 1× speed; fast-mode unlocks remain later work.

Talent spacing previews: `talent-spacing-800.txt` (Warrior) and
`talent-spacing-640.txt` (Paladin), using the collected fixture. Scroll to the
third tree and spec controls to check the entire layout.

`talent-inspection-640.txt` previews a long Hunter description above wrapped trees.
Use the collected fixture; scroll to the full details to verify the compact summary
hides once both the full title and effect are visible.

Character management previews: use `category=manage` with `page=heroes` to
open the real hero list and creation controls directly, including on an account
with a selected hero. `category=appearance` (or omission) opens the dressed view;
an account without a valid hero still shows creation automatically. Presets:
`character-manage-640.txt` and `character-manage-1600.txt`. These requests change
only the displayed page; creating, deleting or renaming still requires the normal
UI actions. Categories on incompatible pages and duplicate keys are rejected.
