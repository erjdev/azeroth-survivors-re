# Changelog

Feature milestones, newest first. These are development checkpoints, not released versions.
Detailed changes remain in Git; unfinished work remains in the [implementation plan](SURVIVORS-IMPLEMENTATION.md).

## 2026-10-27 — Worn weapon identity for specialization rules

Added a shared equipment query for melee type and handedness. It uses owned,
worn gear, so cosmetic overrides cannot activate weapon-specific bonuses.
229 core tests pass, including replacement, ownership loss and catalog checks.
Talent effects are still pending; this has no visible UI change.

## 2026-10-27 — Direct character-management previews

The watched UI request can now open Manage heroes directly, without navigating
from the dressed view. Wide and compact presets expose the real creation controls
against isolated fixtures; live switching back to appearance is verified.

![Direct management preview at 1600×900](images/screenshots/2026-10-27-character-preview/wide.png)

![Management controls at 640×720](images/screenshots/2026-10-27-character-preview/compact.png)

Verified: 454 app tests, Clippy and native wide/compact reload. Build: 15 seconds.
This adds development preview controls, not a new character creation flow.

## 2026-10-27 — Blade Flurry talent burst

A successful owned Blade Flurry starts a 15-second burst: 20% faster melee
recovery and one extra single-target melee opponent, with a two-minute recovery.
Area attacks retain one center. Both bonuses appear green inline.

![Blade Flurry upgrade and taller Rogue trees](images/screenshots/2026-10-27-blade-flurry/talents.png)

![Purchased talent and updated point balance](images/screenshots/2026-10-27-blade-flurry/trained.png)

![Compact two-column trees and complete inspection text](images/screenshots/2026-10-27-blade-flurry/compact.png)

Verified: 227 core and 453 app tests, Clippy, catalog audit and native purchase.
Incremental build: 1m30s. Definitions: 273/432. Windows visuals and live balance
remain open.

## 2026-10-27 — Compact talent columns and saved-view validation

640×720 now fits two talent columns instead of one oversized column, preserving
the taller grid and proportional art. 1600×900 retains all four panels. Native
checks confirm purchase/refund continuity, separate spec scroll positions, a
Collection round trip, hero isolation and saved spec selection after restart.

| Before, 640×720 | After, 640×720 |
| --- | --- |
| ![Before: one oversized tree](images/screenshots/2026-10-27-talent-continuity/before.png) | ![After: two readable trees](images/screenshots/2026-10-27-talent-continuity/compact.png) |
| ![Spec one restores its deeper position](images/screenshots/2026-10-27-talent-continuity/spec-one.png) | ![Spec two restores its separate position](images/screenshots/2026-10-27-talent-continuity/spec-two.png) |

![Spending retains the visible section and updates totals](images/screenshots/2026-10-27-talent-continuity/purchase.png)

![Widescreen retains three trees and adjacent specs](images/screenshots/2026-10-27-talent-continuity/wide.png)

Native build: 23 seconds; formatting and diff checks pass. This is a layout-only
adjustment. Windows and physical-controller acceptance remain open.

## 2026-10-27 — Nature's Grace and Druid talent preview

Druid spell criticals grant one refreshing 15-second charge. The next successful
Wrath or Starfire recovers 0.5 seconds sooner; failed casts preserve the charge.
The talent explains this automatic-casting conversion with a green inline bonus.

![Taller Druid trees and inline Nature’s Grace bonus](images/screenshots/2026-10-27-natures-grace/talents.png)

![Purchased talent and updated point balance](images/screenshots/2026-10-27-natures-grace/trained.png)

![Wrath and Starfire upgrade cards; uncharged values](images/screenshots/2026-10-27-natures-grace/cards.png)

Verified: 226 core and 451 app tests, Clippy, catalog audit, native purchase and
save, plus card preview. Incremental native build: 1m28s. Definitions: 272/432;
Windows visuals and live combat balance remain open.

## 2026-10-27 — Keep talent specs beside widescreen trees

A rounding-sensitive panel width made the spec column wrap at standard 16:9.
The four-column reference layout now fits, retaining taller rows and proportional
art. Compact windows keep their inspection summary and scroll access to saved specs.

![Before: specs below the widescreen trees](images/screenshots/2026-10-27-talent-sidebar/before.png)

![1600×900: trees and specs together](images/screenshots/2026-10-27-talent-sidebar/wide.png)

![800×600: readable compact trees](images/screenshots/2026-10-27-talent-sidebar/compact.png)

![800×600: saved specs remain reachable](images/screenshots/2026-10-27-talent-sidebar/specs.png)

Native build passes (2m56s); both sizes, talent spending and scrolling checked on
macOS. The preceding [full integration baseline](validation/2026-10-27-integration.md)
passes all stages, with 31 addon-corpus skips per data/player stage. This layout
edit now also passes the full chain (8m30s), with the same addon-corpus skips.
Windows visual confirmation remains open.

## 2026-10-27 — Reject incomplete documentation validation

The documentation-link gate now fails if Cargo cannot run or rustdoc exits with
an error, instead of treating missing link warnings as success. Three regression
tests pass and are included in CI. This changes validation only; no new screenshot
is needed. The full workspace test run remains active.

## 2026-10-27 — Encapsulate scenery visibility queries

The engine now owns render-candidate selection for foliage reveal; gameplay no
longer inspects merged mesh storage. The fork's intentional visibility/start-map
APIs have an [explicit contract](SURVIVORS-ENGINE-API.md). The boundary audit passes
at its unchanged 107-leak ceiling.

449 feature-enabled Survivors tests and all-target app Clippy pass, including
regressions for real Elwynn canopy geometry,
behind-player exclusion and the 500 ms fade/reset lifecycle. This preserves the
existing effect; no new visual milestone or Windows scene acceptance is claimed.
Workspace Clippy, render-pass and documentation-link checks also pass; the full
workspace test chain is running. Windows replay notes now show the current taller
talent layout with the existing wide/compact captures.

## 2026-10-27 — Browser packaging checks

The asset checks now follow this checkout's standalone launcher and reject missing
source modules. Generated wasm glue remains explicitly checked; optional storage
setup cannot block startup. Two Rust packaging checks, six build-script tests and
seven browser tests pass. No visible UI change.

The full chain passes formatting, workspace Clippy and the repaired VM/browser
checks, then stops at the world-API audit: 111 unpublished items against a ceiling
of 107. That boundary review and the later integration stages remain open.

## 2026-10-27 — Restore the UI VM boundary

VM-backed tests now live in test files; runtime localization and the browser
bridge live under UI-feed paths. Test bodies and the public bridge API are
unchanged. The VM-boundary audit passes at its original 40-gameplay-file ceiling.

446 Survivors tests, app Clippy and WebGPU compilation pass (five existing browser
warnings). The subsequent full chain passed formatting/workspace Clippy; its current
world-API blocker is recorded above. No visible UI change; existing milestone captures apply.

## 2026-10-27 — Equipment and rank localization

Health, armor and damage summaries now use shared runtime templates throughout
the Armory, including cloaks and set-bonus previews. Signs, percentage units and
translated rank argument order are preserved.

![Collection totals and set-bonus preview](images/screenshots/2026-10-27-equipment-localization/collection.png)

446 Survivors app tests, app Clippy and the reference-string audit pass. Native
English Collection checked at 1280×720; build: 1m07s. The full integration chain
has been restarted and remains pending; live translated-layout acceptance is open.

## 2026-10-27 — Localized regeneration card values

Renew's inline health values use the shared runtime health template, preserving
fractional upgrades, green highlighting and a single unit after the arrow.

![New and upgraded Renew cards](images/screenshots/2026-10-27-regen-localization/cards.png)

444 Survivors app tests and all-target app Clippy pass. Native English preview
checked at 1280×720; translated fractional-value/color behavior has regression
coverage. Native build: 1m15s. Four reference-string findings remain; full-gate
and live translated-layout acceptance are still open.

## 2026-10-27 — More room between talent ranks

Talent grids are about 27% taller at the same width, with larger icons and a
minimum readable height. Artwork keeps its proportions with a centered side crop;
narrow windows wrap and scroll. This follows the intention reference's taller rows.

![Before, 1600×900](images/screenshots/2026-10-27-talent-room/before.png)

![Taller trees, 1600×900](images/screenshots/2026-10-27-talent-room/wide.png)

![More row spacing at 800×600](images/screenshots/2026-10-27-talent-room/compact.png)

![Saved specs remain reachable by scrolling](images/screenshots/2026-10-27-talent-room/specs.png)

Native macOS checks at both sizes confirm art alignment and scroll access to the
last talents and saved specs. 443 app tests and all-target app Clippy pass; native
incremental build: 2m31s. Windows confirmation remains open.

## 2026-10-27 — Reward message localization

Duplicate companion instincts now return a typed purchase result, translated by
the Armory without changing costs or draft behavior. Starter progress counts use
compact ratios. 443 Survivors app tests and all-target app Clippy pass; the
reference-string audit is down to five remaining literals, so the full gate stays open.

## 2026-10-27 — Localized settings and level labels

Audio switches and Character rotation buttons now resolve their text after startup,
including controls created before the string table is ready. HUD levels and zone
level ranges share the localized level template.

![Audio settings at 800×720](images/screenshots/2026-10-27-label-bindings/audio.png)

![Localized rotation labels](images/screenshots/2026-10-27-label-bindings/rotation.png)

442 Survivors app tests with game data, all-target app Clippy, and the native build
pass. Native build: 1m54s. Screenshots use the English install; translated binding
and missing-key behavior have regression coverage. Eight reference-string literals
remain, down from 15; the full integration gate is still open.

## 2026-10-27 — Shared tooltip localization

Ability and passive tooltip rank headings, ranges, durations and cooldowns now
resolve the engine's boot-VM string keys. The shared formatter preserves fractional
Survivors values, translated template order, and green upgrade lines. Missing or
empty strings retain readable English fallbacks.

All-target app Clippy and 441 Survivors app tests pass with game data, including startup loading and
translated tooltip/color regressions. The reference-string gate has 15 remaining
literals, down from 24. Live translated-tooltip layout acceptance remains open;
this checkpoint uses regression evidence, not new gameplay screenshots.

## 2026-10-27 — Appearance label localization

Character feature labels and the appearance screen's Enter World button now use
the install's glue-string keys. Race-specific horn, hair and facial-feature labels
retain readable fallbacks when a key is missing.

![Human feature labels at 800×720](images/screenshots/2026-10-27-appearance-localization/human.png)

![Tauren horn labels in the same layout](images/screenshots/2026-10-27-appearance-localization/tauren.png)

439 Survivors app tests (default-feature library), all-target app Clippy, and the native
build pass, including translation/fallback and dial-boundary coverage. Native build: 67 seconds. Visual evidence uses the local English install;
non-English text is covered by injected-table tests, not a translated live client.
The reference-string gate has 24 remaining literals, down from 31.

## 2026-10-27 — Preview activation and pipeline coverage

UI preview activation now follows the engine's development-mode boundary. Player
builds ignore preview environment variables and never install the watcher; the
player-build regression test passes. The starter-only backdrop has a documented
menu-material exemption, backed by its gameplay/pause lifecycle test.

Full-workspace Clippy and app unit tests pass. The full gate now reaches a later
failure: Survivors labels must resolve existing localization keys. No-install and
full player-suite acceptance remain pending. This checkpoint has no visual changes.

## 2026-10-27 — Broader integration audit

Fixed four test-target Clippy errors missed by library-only checks. Full-workspace
Clippy and formatting pass. Browser tests (7), launcher fixtures (12; one Windows
check skipped), build-script tests (6), and WebGPU compilation pass.

The workspace test gate remains **failing**: 4,672 app tests passed, three failed
(account selection, material warm-up coverage, and development-preview boundaries),
and ten were ignored. The account-selection test passes outside the sandbox; its
local mock-server socket was denied. Backdrop warm-up and preview-boundary failures
remain. Addon-corpus coverage and Windows acceptance remain open.
This checkpoint changes test setup only; the visual milestones below remain current.

## 2026-10-27 — Insect Swarm talent

The Restoration node now makes Insect Swarm reduce affected enemies' damage by
2% for 10 seconds. It preserves stronger reductions, including Scorpid Sting;
periodic ticks do not refresh it. Trained card descriptions show the fixed debuff
alongside green ability-rank changes.

![Restoration talent purchase at 800×600](images/screenshots/2026-10-27-insect-swarm/talent.png)

![Trained Insect Swarm cards at 1280×720](images/screenshots/2026-10-27-insect-swarm/cards.png)

![Compact card at 640×480](images/screenshots/2026-10-27-insect-swarm/compact.png)

224 core tests, 440 app tests and core/app Clippy pass. Native incremental build:
17 seconds. Definition coverage is 271/432; live combat balance and Windows
acceptance remain open.

## 2026-10-27 — Wing Clip and its root talent

Hunter offers now include Wing Clip, a close-range slow. Improved Wing Clip adds
4% root chance per rank (five ranks), with shorter roots on bosses. Cards show
trained root chance alongside green damage, slow-duration and cooldown upgrades.

![Purchased Improved Wing Clip at 800×600](images/screenshots/2026-10-27-wing-clip/talent.png)

![Trained new-spell and rank-up cards at 1280×720](images/screenshots/2026-10-27-wing-clip/cards.png)

![Complete description at 640×480](images/screenshots/2026-10-27-wing-clip/compact.png)

223 core tests, 438 app tests, core/app Clippy and formatting pass. Native
incremental build: 32 seconds. Local catalog coverage is 270/432 definitions; balance, combat visuals and Windows
acceptance remain open.

## 2026-10-27 — Hunter melee and Savage Strikes

Raptor Strike and Mongoose Bite join normal Hunter ability offers, inline upgrades
and the Codex. Savage Strikes adds 10 percentage points of critical chance per
rank to these two attacks only. Mongoose Bite uses an automatic cooldown conversion.

![Savage Strikes purchase in the taller Survival tree](images/screenshots/2026-10-27-hunter-melee/talent.png)

![Hunter melee inline upgrades](images/screenshots/2026-10-27-hunter-melee/cards.png)

![Raptor Strike Codex entry](images/screenshots/2026-10-27-hunter-melee/codex.png)

222 core tests, 435 app tests, core/app Clippy and formatting pass. Native previews
checked at 800×600 and 1280×720; incremental build: 26 seconds. Coverage reaches
269/432 talent definitions. Live balance, combat animation and Windows acceptance
remain open.

## 2026-10-27 — Complete talent stat highlighting

Inline upgrades now keep full stat phrases green, including bleed/reflected damage,
shield absorption, deployment cooldown and damage-reduction duration. Vengeance
now shows its damage increase inline; fixed duration and secondary effects stay white.

![Bleed damage highlighted inline](images/screenshots/2026-10-27-talent-stat-labels/rend.png)

![Vengeance inline damage and fixed duration](images/screenshots/2026-10-27-talent-stat-labels/vengeance.png)

221 core tests and core Clippy pass. Native macOS previews checked at 800×600 and
640×480; incremental build: 16 seconds. Formatting only; no balance changes.

## 2026-10-27 — Talent inspection stays in view

Wrapped talent trees now keep an inspection summary above them when full details
are off-screen. Keyboard focus shows the upgrade before purchase; spending a point
updates the green comparison. The summary hides when full details are visible.

![Inspect before spending](images/screenshots/2026-10-27-talent-inspection/focus.png)

![Purchase updates the comparison](images/screenshots/2026-10-27-talent-inspection/purchased.png)

![Full details take over after scrolling](images/screenshots/2026-10-27-talent-inspection/full-details.png)

![Long description at 640×480](images/screenshots/2026-10-27-talent-inspection/compact.png)

433 app tests, app Clippy and formatting pass. Native macOS keyboard/purchase/scroll
checks at 800×600 and compact description check at 640×480. Incremental build:
2m27s. Windows and full resolution-matrix acceptance remain open.

## 2026-10-27 — Taller, less crowded talent trees

Talent columns now wrap before squeezing the artwork and icons. Trees keep their
300:331 proportions, with uniform column widths and more vertical space between
ranks. The third tree and saved specs remain reachable by scrolling.

![Before: compressed trees at 800×600](images/screenshots/2026-10-27-talent-spacing/before.png)

![After: taller trees at 800×600](images/screenshots/2026-10-27-talent-spacing/800.png)

![Third tree and saved specs](images/screenshots/2026-10-27-talent-spacing/specs.png)

![Compact Paladin trees](images/screenshots/2026-10-27-talent-spacing/640.png)

Native macOS layouts checked at 640×480, 800×600 and 1280×720 against the intention
reference. App Clippy and formatting pass; incremental build: 30 seconds. Layout
only; no new combat tests. Windows confirmation remains open.

## 2026-10-27 — Run Options page

Run Options now shows the selected zone, Normal difficulty and normal game speed,
with working Settings and return actions. Enter cannot start a run from this page.
Native macOS navigation checked at 1280×720 and 640×480; 432 app tests, 12 preview
fixture tests and app Clippy pass. Incremental build: 1m18s. Fast modes remain planned.

![Run Options](images/screenshots/2026-10-27-run-options/wide.png)

![Compact Run Options](images/screenshots/2026-10-27-run-options/compact.png)

## 2026-10-27 — Starter panel bounds and 16:10 validation

Starter scroll content now stays inside the visible panel borders. Checked a
synthetic full collection at 1280×800, then live-resized to 640×720: final zones
remain reachable and the bottom actions remain visible.

![Populated 16:10 starter](images/screenshots/2026-10-27-starter-16-10/starter.png)

![Last zone within the frame](images/screenshots/2026-10-27-starter-16-10/zones.png)

![Compact final zones and actions](images/screenshots/2026-10-27-starter-16-10/compact-zones.png)

Native macOS visual/scroll checks, app Clippy and formatting pass. Incremental
build: 23 seconds. Run Options still only reports Normal difficulty; Windows,
other resolutions and zone readiness remain open. No gameplay changes or new
combat-test run in this checkpoint.

## 2026-10-27 — Complete haste upgrade highlighting

Talent descriptions now color the full changing haste stat green, including
“haste,” while fixed proc chance and duration remain white. Verified on the
native Hunter page at 1280×720; 221 core tests and core Clippy pass.
Incremental native build: 14 seconds. No gameplay mechanics changed.

![Full haste upgrade highlighted](images/screenshots/2026-10-27-haste-label/talent.png)

## 2026-10-27 — Improved Aspect of the Hawk

Hunter shots can now trigger 12 seconds of shot haste, scaling from 3% to 15%
with talent ranks. The buff refreshes without stacking and shows its remaining
time in the HUD. Volley, traps and pet attacks are excluded.

![Improved Hawk preview](images/screenshots/2026-10-27-improved-hawk/preview.png)

![Improved Hawk purchased](images/screenshots/2026-10-27-improved-hawk/purchased.png)

221 core tests, 431 app tests and core/app Clippy pass. Native macOS purchase
checked at 1280×720; incremental build: 1m45s. Definitions: 268/432;
164 remain. Live combat balance and Windows acceptance remain open.

## 2026-10-27 — Hunter Trap Mastery

Trap Mastery now strengthens Explosive Trap by 5% per rank (two ranks),
multiplying Clever Traps without changing its area or Entrapment roots.
The talent shows green inline upgrades and works with saved specs and refunds.

![Trap Mastery preview](images/screenshots/2026-10-27-trap-mastery/preview.png)

![Purchased Trap Mastery](images/screenshots/2026-10-27-trap-mastery/purchased.png)

220 core tests, 428 app tests and core/app Clippy pass. Native macOS purchase
checked at 1280×720; incremental build: 16 seconds. Definitions: 267/432;
165 remain. Live balance and Windows acceptance remain open.

## 2026-10-27 — Enlarged compact Hunter UI checked

At 800×600 and 125% interface size, Hunter talent art stays proportional and the
full description and spec controls remain reachable by scrolling. Switching back
to a trained spec restores its scroll position and ranks. The Codex stacks its
panels and scrolls through all 114 entries independently of the selected details.
All three Scorpid Sting cards fit, including the hovered card.

![Compact Hunter talent trees](images/screenshots/2026-10-27-hunter-compact/trees.png)

![Scrolled talent description and specs](images/screenshots/2026-10-27-hunter-compact/details.png)

![Stacked compact Codex](images/screenshots/2026-10-27-hunter-compact/codex.png)

![Compact upgrade cards](images/screenshots/2026-10-27-hunter-compact/cards.png)

![Active 125% interface setting](images/screenshots/2026-10-27-hunter-compact/settings.png)

Native macOS validation of `a4d59f0` gameplay/UI source; no code change or new test
run in this checkpoint. Windows, controller input and the wider matrix remain open.

## 2026-10-27 — Scorpid Sting and duration training

Hunters can acquire and upgrade Scorpid Sting: a targeted hit that reduces enemy
damage by 25%. Improved Scorpid Sting extends that debuff by 20% per rank, with
three ranks. The spell has its own Codex amulet and green inline upgrades.
Weaker debuffs cannot overwrite or prolong stronger active damage reductions.

![Scorpid Sting acquisition and upgrades](images/screenshots/2026-10-27-scorpid-sting/cards.png)

![Scorpid Sting Codex entry](images/screenshots/2026-10-27-scorpid-sting/codex.png)

![Improved Scorpid Sting purchased](images/screenshots/2026-10-27-scorpid-sting/talent.png)

220 core tests, 428 app tests, 12 preview-launcher tests, core/app Clippy and
formatting checks pass. Native 1280×720 cards, Codex and purchase checked;
build: 24 seconds. Definitions: 266/432 (Hunter 33/46); 166 remain.
Live projectile visuals, balance and Windows acceptance remain open.

## 2026-10-27 — Hunter Deterrence

Deterrence now grants an automatic 10-second window of 50% damage reduction at
run start and every 180 seconds of gameplay time. It applies before shields;
pauses freeze the window. The HUD reports its active duration, and saved specs
and refunds control whether it applies.

![Deterrence inline upgrade](images/screenshots/2026-10-27-deterrence/preview.png)

![Deterrence purchased](images/screenshots/2026-10-27-deterrence/purchased.png)

219 core tests, 425 app tests, core/app Clippy and formatting checks pass.
Native 1280×720 preview and purchase verified; build: 31 seconds.
Definitions: 265/432 (Hunter 32/46); 167 remain. Live combat balance and Windows
acceptance remain open; these captures show the menu only.

## 2026-10-27 — Preserve damaged talent saves

Malformed talent rows, extra spec groups and references to missing heroes now
activate account recovery instead of silently losing saved ranks. The original
file stays untouched and runs/saving remain disabled until it is repaired.
Valid legacy specs, reordered rows and duplicate-rank normalization still load.
Unknown talent IDs remain preserved for catalog validation.

No interface appearance changed; this uses the existing recovery notice.
422 app tests pass with local data; Clippy, formatting and diff checks pass on macOS.

## 2026-10-27 — Shield refreshes preserve stronger defenses

Mana Shield and Power Word: Shield now wait when a stronger absorb is active,
instead of overwriting it. Equal absorbs no longer shorten a longer shield.
A declined refresh retries after 0.2 seconds without spending the full cooldown;
successful refreshes still use one shared, non-stacking pool.

The existing talent-shield path uses the same replacement rule. No menu appearance
changed; live overlapping-shield visuals and balance remain unverified.
All 422 app tests, Clippy, formatting and diff checks pass on macOS.

## 2026-10-27 — Mana Shield and defensive training

Mages can now acquire and upgrade Mana Shield. Its initial rank absorbs 40 damage
for 10 seconds and recharges in 18 seconds. Improved Mana Shield reduces its
cooldown by 10% per rank, with two ranks maximum; no mana resource is simulated.
The spell has its own Codex amulet entry and green inline upgrade values.

![Mana Shield acquisition and upgrade cards](images/screenshots/2026-10-27-mana-shield/cards.png)

![Mana Shield in the Codex](images/screenshots/2026-10-27-mana-shield/codex.png)

![Improved Mana Shield purchased](images/screenshots/2026-10-27-mana-shield/talent.png)

218 core tests, 422 app tests, 12 preview-launcher tests, core/app Clippy and
formatting checks pass. Definitions: 264/432 (Mage 37/49); 168 remain.
Native 1280×720 cards, Codex and talent purchase checked; build: 20 seconds.
These are menu captures; live shield visuals, balance and Windows acceptance remain open.

## 2026-10-27 — Presence of Mind

The individual Mage talent now advances an owned Arcane Explosion cooldown by
two seconds per successful Arcane Missiles cast. It adds to the Arcane milestone
for four seconds total, without granting spells or triggering per missile.

![Presence of Mind upgrade preview](images/screenshots/2026-10-27-presence-of-mind/preview.png)

![Purchased Presence of Mind](images/screenshots/2026-10-27-presence-of-mind/purchased.png)

217 core and 419 app tests, core/app Clippy, formatting and diff checks pass.
Definition coverage: 263/432 (Mage 36/49); 169 definitions remain.
Native 1280×720 preview, purchase and saved rank checked. Native build: 28 seconds.
Live balance and Windows acceptance remain open.

## 2026-10-27 — Preserve unsupported class saves

Loading an unsupported class in a hero or class-progress record now enters the
existing recovery mode instead of silently dropping that progress on the next
save. The original file stays untouched and saving/runs remain disabled.
Supported classes and missing-save creation keep their existing behavior.

Disk-level regressions cover valid heroes mixed with unsupported records, talent
references, extreme class indices and refused writes. No interface appearance
changed; the existing recovery notice is reused.
All 418 app tests and Clippy pass on macOS, including byte-preservation checks.

## 2026-10-27 — Consistent cached integration checks

`cargo survivors-test` now matches the PR gate, including visibility features;
`cargo survivors-clippy -- -D warnings` uses the same library configuration.
This avoids accidentally switching test feature variants during iteration.

On this macOS checkout, the warm test build took 0.47 seconds and all 418 tests
ran in 1.67 seconds; cached Clippy took 0.38 seconds. A launcher-only rebuild took
3.19 seconds. Source edits and first builds still cost more. No UI appearance
changed; the Arcane Subtlety screenshots below remain the latest visual checkpoint.

## 2026-10-27 — Arcane Subtlety

Arcane Subtlety now strengthens Arcane Missiles and Arcane Explosion by 5% per
rank instead of provisional global damage. Two ranks maximum; the green inline
preview names the affected spells and explains the Survivors resistance conversion.

![Arcane Subtlety preview](images/screenshots/2026-10-27-arcane-subtlety/preview.png)

![Purchased rank and next upgrade](images/screenshots/2026-10-27-arcane-subtlety/purchased.png)

216 core and 418 app tests, core/app Clippy, formatting and diff checks pass.
Talent definitions: 262/432 (Mage 35/49); 170 remain.
Native 1280×720 preview, purchase and saved rank checked; build took 19 seconds.
Combat balance and Windows acceptance remain open.

## 2026-10-27 — Browser talent layouts at 5:4 and ultrawide

At 1280×1024, three proportional trees sit above the scrollable details panel;
Combustion's full green preview and zero-point purchase gate were checked.
At 2560×1080, the trees and complete details fit side by side.

![5:4 talent trees](images/screenshots/2026-10-27-browser-resolutions/talents-1280x1024.png)

![Scrollable Combustion details](images/screenshots/2026-10-27-browser-resolutions/combustion-1280x1024.png)

![Ultrawide talent artwork and details](images/screenshots/2026-10-27-browser-resolutions/talents-2560x1080.png)

Checkout `7aa09ef`: WebGPU check (28 seconds), linked package (76 seconds),
seven browser tests and six build-script tests pass. Browser console was clear.
Ultrawide pointer interactions remain unverified; this checkpoint adds visual
coverage, not Windows gameplay acceptance. No layout code changed.

## 2026-10-27 — Preview without waiting for Cargo

The fixture launcher accepts `--run --executable <path>` to open a previously
built development app directly, bypassing Cargo’s build lock. Isolated saves,
watched presets and exit codes are preserved. Rust changes still require rebuilding.

Twelve fixture/launcher tests pass on macOS. Windows execution remains unverified.
No interface appearance changed; the existing screenshots remain current.

## 2026-10-27 — Mage Combustion

Combustion now builds Fire critical chance from landed direct hits, ends after
three critical hits and recovers after three minutes. Area targets count separately;
burns and multistrike echoes do not. The HUD reports burst progress.

![Combustion inline preview](images/screenshots/2026-10-27-combustion/preview.png)

![Combustion learned in the Fire spec](images/screenshots/2026-10-27-combustion/purchased.png)

216 core and 418 app tests, core/app Clippy, formatting and diff checks pass.
Definition coverage: 261/432 (Mage 34/49); 171 definitions remain.
Native 1280×720 purchase and description checked; native build: 108 seconds.
Live balance, combat HUD appearance and Windows acceptance remain open.

## 2026-10-27 — Scoped caster talents

Shadow Focus, Nature’s Guidance and Suppression now strengthen their named
spells instead of provisional global damage. Green inline previews identify the
bonus and exclusions. Purchases, saves, spec changes, caps and refunds are covered.

![Priest Shadow Focus inline upgrade](images/screenshots/2026-10-27-caster-accuracy/priest.png)

![Shaman Nature’s Guidance and continuous tree artwork](images/screenshots/2026-10-27-caster-accuracy/shaman.png)

![Warlock Suppression and aligned talent rows](images/screenshots/2026-10-27-caster-accuracy/warlock.png)

215 core and 415 app tests, core/app Clippy and formatting checks pass.
Definition coverage reaches 260/432; 172 talents remain. Native build: 19 seconds.
Three class menus checked at 1280×720, including a Priest purchase. Windows
foliage, routing, chest cadence and talent-layout acceptance remain open.

## 2026-10-27 — Browser Priest and compact-layout validation

The current WebGPU package renders Inspiration, Blessed Recovery and provisional
bonuses with green inline scaling. A saved Priest loads correctly; zero earned
points block purchases. At 640×720 the trees reflow and the complete recovery
description remains accessible by scrolling.

![Current browser Inspiration description and purchase gate](images/screenshots/2026-10-27-browser-priest/inspiration.png)

![Compact browser recovery description](images/screenshots/2026-10-27-browser-priest/compact.png)

![Browser provisional bonus remains explicitly labeled](images/screenshots/2026-10-27-browser-priest/provisional.png)

At `d9d010a`: WebGPU check (27 seconds), linked play package (94 seconds), seven
browser tests and six build-script tests pass. Five existing compile warnings
remain; no browser warnings/errors were returned during this UI check. These
captures do not verify in-map gameplay or Windows rendering.

## 2026-10-27 — Priest Inspiration

Inspiration now grants 8% more armor per rank for 15 seconds after Holy Nova
actually restores health. It refreshes without stacking, replaces provisional
damage, and does not grant Holy Nova or its healing. Other healing and full-health
casts do not trigger it. The HUD reports the active armor bonus and duration.

![Inspiration inline armor upgrade](images/screenshots/2026-10-27-inspiration/preview.png)

![Maximum Inspiration rank saved to the Holy spec](images/screenshots/2026-10-27-inspiration/purchased.png)

214 core and 413 app tests, core/app Clippy and formatting checks pass.
Definition coverage: 257/432 (Priest 28/47). Native menu and purchase checked
at 1280×720; native build: 61 seconds.
Combat HUD rendering, live balance and Windows acceptance remain open.

## 2026-10-27 — Consistent provisional talent previews

Talents awaiting individual mechanics now show their existing fallback bonus as
an inline green comparison, with the unit shown once. Their pending notice stays
visible. Maximum ranks show the current value without an upgrade arrow.

![Provisional bonus before the last rank](images/screenshots/2026-10-27-provisional-preview/preview.png)

![Maximum rank shows its current bonus](images/screenshots/2026-10-27-provisional-preview/max-rank.png)

409 app tests, app Clippy and formatting checks pass. Native 1280×720 purchase
flow checked; native build passes in 79 seconds.
Mechanics and definition coverage are unchanged. Broader platform acceptance
remains open.

## 2026-10-27 — Blessed Recovery

Priest Blessed Recovery now heals 7% of health damage taken per rank over six
seconds. Repeated hits carry unpaid healing forward; shields, immunity and lethal
hits cannot trigger recovery. The description identifies the all-hit Survivors
conversion and the combat HUD shows its remaining time.

![Recovery purchase and inline next-rank scaling](images/screenshots/2026-10-27-blessed-recovery/purchased.png)

![Full recovery description in a compact window](images/screenshots/2026-10-27-blessed-recovery/compact.png)

213 core and 409 app tests pass, along with core/app Clippy. Talent definitions
reach 256/432 (Priest 27/47). Native menus checked at 1280×720 and 640×720;
build passed in 96 seconds.
Combat HUD rendering, live balance and Windows acceptance remain open.

## 2026-10-27 — Blessing of Sanctuary

Paladins can train Sanctuary to prevent 2 damage per incoming hit after armor
and general reduction, before shields. This passive self-only conversion replaces
placeholder damage and grants no block chance or retaliation damage.

![Sanctuary inline upgrade preview](images/screenshots/2026-10-27-sanctuary/preview.png)

![Sanctuary purchased and saved in the Protection spec](images/screenshots/2026-10-27-sanctuary/purchased.png)

212 core and 407 app tests pass, including shield ordering, rank caps, saved specs
and refunds. Core/app Clippy pass. Definitions: 255/432 (Paladin 25/44).
Native build: 20 seconds. Native menu checked at 1280×720;
Windows scenes and live combat remain unverified.

## 2026-10-27 — Browser talent and saved-hero integration

Current WebGPU package verifies Priest creation from the empty talent page,
Improved Fade's green description, the no-points purchase gate, and hero
persistence after reload. Draft PR validation and remaining scope refreshed.

![Current browser talent description and purchase gate](images/screenshots/2026-10-27-browser-talents/fade.png)

![Created Priest restored after browser reload](images/screenshots/2026-10-27-browser-talents/reloaded.png)

At `9d6c0ba`: WebGPU check and linked play package pass; seven browser tests
and six build-script tests pass. Five existing wasm warnings remain. Captures
are 1280×720; Windows CI, gameplay and broader resolution acceptance remain open.

## 2026-10-27 — Priest Improved Fade

Improved Fade replaces placeholder damage with 10% shorter Fade cooldown per
rank: 18 / 16.2 / 14.4 seconds. Its four-second escape, movement bonus and
level-five unlock stay unchanged. Saved specs and refunds update the next run.

![Improved Fade purchase and inline cooldown preview](images/screenshots/2026-10-27-improved-fade/purchased.png)

211 core / 405 app tests, both Clippy checks and native build pass. Definitions:
254/432. Native 1280×720 purchase capture; Windows and live combat acceptance
remain open.

## 2026-10-27 — Preparation resets learned Sprint

Preparation's automatic reset now includes Sprint after its level-five unlock.
Active Sprint duration and Endurance's shorter recharge are preserved. It neither
banks resets nor resets Cold Blood or Adrenaline Rush recovery.

![Preparation and Endurance in the same saved spec](images/screenshots/2026-10-27-preparation-sprint/talents.png)

210 core / 403 app tests, both Clippy checks and the 25-second native build
pass. Native 1280×720 description capture; live combat and Windows acceptance
remain open.

## 2026-10-27 — Rogue Adrenaline Rush

Adrenaline Rush now grants a 15-second burst of 50% shorter Rogue cast
cooldowns, starting on a successful cast and recovering after five minutes.
It replaces placeholder damage and adds an active HUD timer. Failed targeting
does not spend it; Sprint and existing cooldowns are unchanged.

![Adrenaline Rush inline upgrade preview](images/screenshots/2026-10-27-adrenaline-rush/preview.png)

![Purchased talent in the Combat tree](images/screenshots/2026-10-27-adrenaline-rush/purchased.png)

210 core / 403 app tests, both Clippy checks and native build pass.
Definitions: 253/432. Native 1280×720 menu capture; live combat balance and
Windows acceptance remain open.

## 2026-10-27 — Rogue Endurance and Sprint recovery

Endurance now reduces Sprint's 15-second cooldown to 13.5 / 12 seconds,
replacing placeholder damage. Speed, duration and the level-five unlock stay
unchanged. Saved specs and refunds carry the correct cooldown into the next run.

![Purchased Endurance and inline next-rank preview](images/screenshots/2026-10-27-endurance/purchased.png)

209 core / 400 app tests, both Clippy checks and native build pass. Native
1280×720 capture verifies purchase and green preview. Definitions: 252/432.
Windows and live combat acceptance remain open.

## 2026-10-27 — Complete green talent stat phrases

Talent upgrade highlights now include their stat labels: “4 » 8% maximum health”
and Moonfire's damage/critical-chance phrases are green together. Fixed secondary
bonuses and connective prose stay white; numbers, mechanics and saves are unchanged.

![Complete health phrase highlighted](images/screenshots/2026-10-27-talent-stat-labels/health.png)

![Both Moonfire changes highlighted inline](images/screenshots/2026-10-27-talent-stat-labels/moonfire.png)

208 core / 398 app tests, both Clippy checks, formatting and native build pass.

## 2026-10-27 — Browser integration and accurate credit

Current WebGPU package verifies Naxxramas source selection, Raid Epic stats and
set details in the browser. The readiness gate stays visible. Loading now credits
Aethus's game as inspiration rather than presenting this checkout as its source.
PR description updated to current features, validation and outstanding work.

![Browser raid source selection](images/screenshots/2026-10-27-browser-raid/drops-1280.png)

![Corrected browser loading credit](images/screenshots/2026-10-27-browser-raid/loading-credit.png)

WebGPU target check and linked play package pass with five existing warnings;
seven browser tests pass. UI checked at 1280×720; broader browser and Windows
acceptance remain open. No PR-triggered Actions runs were returned for this head.

## 2026-10-27 — Druid Heart of the Wild

Heart of the Wild replaces placeholder damage with 4% maximum health per rank,
up to 20%. It includes equipped health, requires no form and does not heal current
health. Purchase, saved specs, refunds and inline green previews are verified.

![Heart of the Wild purchase and next rank](images/screenshots/2026-10-27-heart-of-the-wild/purchased.png)

![Saved rank and compact description after restart](images/screenshots/2026-10-27-heart-of-the-wild/compact.png)

207 core / 398 app tests, both Clippy checks and native build pass. Talent
definitions: 251/432. No Windows or live combat acceptance is claimed.

## 2026-10-27 — Raid Epic finale rewards and sources

Kel'Thuzad now settles one random Raid Epic armor piece for the active class:
eight equal 12.5% slots, independent of the legendary weapon roll. Duplicates
add no power; equipment and appearances stay selected. Collection and Drop
Tables expose the source and odds. **Naxxramas remains gated by world readiness.**

![Raid finale source and armor details](images/screenshots/2026-10-27-raid-source/drops-1280.png)

![Compact source rows and selected armor](images/screenshots/2026-10-27-raid-source/drops-640.png)

Native menu captures at 1280×720 and 640×720 verify layout and row selection;
these are not live raid or Windows acceptance evidence.
206 core / 396 app tests, both Clippy gates, formatting, native build and the
real-data Collection audit pass.

## 2026-10-27 — Windows report regression recheck

Foliage, routing, spawn clearance, chest cadence and talent-art checks pass.
The [playtest tracker](WINDOWS-PLAYTEST.md) now links the Windows visibility capture
workflow. Full app suite: 392 passed; two unfinished Raid Epic source-page tests
failed. Windows scene acceptance remains open. No UI change or new screenshots.

## 2026-10-27 — Collection content verification

Added `survivors-audit --collection`: all 486 primary cells resolve to decoded
icons and scheduled positive-chance sources. All 153 supplemental identities
also have valid icons (495 unique display icons total). Raid acquisition and
world traversal remain open; this audit does not unlock zones. The real-data
audit, eight existing audit tests, Clippy and formatting pass. No UI change.

## 2026-10-27 — Windows launcher failure reporting

PowerShell now forwards Cargo's exit status; the cmd wrapper explicitly returns
it too. Windows CI gains a no-game-data launcher smoke test covering paths with
spaces, checkout-relative execution and data overrides. Python syntax and diff
checks pass locally; execution skips on macOS. GitHub still reports no workflow
runs, so Windows acceptance remains open. No UI change.

## 2026-10-27 — Accurate periodic amulet descriptions

Rupture, Shadow Word: Pain, Corruption, Curse of Agony, Insect Swarm and Entangling
Roots amulets no longer advertise ineffective critical damage. Their 5% damage
bonus remains; replacement comparisons omit the same misleading loss. Direct-hit
amulets retain their critical bonuses. Combat and save rules are unchanged.

Native before/after Codex captures and a mixed choice-card preview verify the fix.
All 394 app tests, app-test Clippy, formatting and native build pass.

![Before: ineffective critical bonus](images/screenshots/2026-10-27-periodic-amulets/before.png)
![After: effective periodic bonus](images/screenshots/2026-10-27-periodic-amulets/after.png)
![Periodic and direct-hit amulet cards](images/screenshots/2026-10-27-periodic-amulets/cards.png)

## 2026-10-27 — Direct Codex UI previews

Fast menu previews can now select any catalog spell or shared jewelry detail.
Checked-in wide/compact presets switch entry and resolution without a restart;
invalid requests preserve the current view. Inspection leaves discovery and the
account save unchanged. Native captures below verify both layouts.
All 393 app tests, app-test Clippy, formatting and the native build pass.

![Direct Wrath Codex preview](images/screenshots/2026-10-27-codex-preview/wrath.png)
![Live compact ring preview](images/screenshots/2026-10-27-codex-preview/ring-compact.png)

## 2026-10-27 — Druid Leader of the Pack

Leader of the Pack now grants three percentage points of hero critical chance,
adding to Sharpened Claws and gear. Its description explicitly identifies the
Survivors self-only, no-form conversion. Purchase replaces placeholder damage;
saves, spec switching and refunds preserve the intended bonus.

205 core tests, 391 app tests, both Clippy gates and the native build pass.
Native purchase at 1280×720 and restart at 640×720 verify the green inline preview
and saved rank. Catalog coverage is 250/432 (Druid 24/47); Windows and live combat
acceptance remain open.

![Leader of the Pack preview](images/screenshots/2026-10-27-leader-of-the-pack/preview.png)
![Purchased Druid capstone](images/screenshots/2026-10-27-leader-of-the-pack/purchased.png)
![Saved compact Druid talent](images/screenshots/2026-10-27-leader-of-the-pack/compact.png)

## 2026-10-27 — Windows playtest replay checklist

Rechecked the four reported areas against `1ad6616`; all 390 Survivors app tests
pass on macOS. The [Windows tracker](WINDOWS-PLAYTEST.md) now includes concrete
replay steps for foliage, stuck enemies, chest cadence and talent proportions.
Windows scene acceptance remains open. No runtime or visual change in this checkpoint;
the existing talent captures remain inline in the tracker.

## 2026-10-27 — Priest Power Infusion

Power Infusion now automatically boosts Priest damage, unlocked Holy Nova healing
and owned Renew by 20% for 15 seconds, with three-minute recovery. Failed targeting
preserves readiness; pauses stop timers. HUD status shows the active countdown.
Shields and unrelated regeneration receive no bonus.

203 core tests, 390 app tests, both Clippy gates and native build pass. Native 1280×720/640×720
captures verify the inline preview, legal purchase and saved rank after restart.
Coverage is 249/432 (Priest 25/47). HUD rendering in combat, live balance and
Windows acceptance remain open.
Boundary follow-up verifies independent Inner Focus recovery, unchanged shields,
Holy Nova's maximum-health cap and no healing without its unlock. All 204 core
and 390 app tests and both Clippy gates pass. No UI change.

![Power Infusion inline preview](images/screenshots/2026-10-27-power-infusion/preview.png)
![Purchased Power Infusion](images/screenshots/2026-10-27-power-infusion/purchased.png)
![Compact saved Power Infusion](images/screenshots/2026-10-27-power-infusion/compact.png)

## 2026-10-27 — Current browser build acceptance

Rebuilt WebGPU from `5a1294f`: target check and linked incremental package pass
with five existing warnings (86-second Cargo build). Browser startup and real
clicks verify starter/Armory Options retain the landscape and Done returns to
the originating page at 1280×720. No gameplay or Windows acceptance is claimed;
GitHub still reports zero workflow runs, so no Windows artifact is verified.

![Browser Options landscape](images/screenshots/2026-10-27-browser-options/options.png)
![Returned to browser Character management](images/screenshots/2026-10-27-browser-options/returned.png)

## 2026-10-27 — Hunter Trueshot Aura

Trueshot Aura now grants 10% more direct and periodic Hunter shot damage, with an
inline green preview. The passive applies to the Hunter only; traps, Volley and
pets are excluded. It multiplies existing damage talents and the separate
30-point milestone. Purchase and restart verified at 1280×720 and 640×720.

202 core tests, 387 app tests, both Clippy gates and native build pass. Definition
coverage is 248/432 (Hunter 31/46). Live balance and Windows acceptance remain open.
Saved-spec follow-up: the real-catalog regression now verifies hero/spec isolation,
refund persistence, unchanged pet bonuses and immutable active-run training.
All 387 app tests and app/test Clippy pass; no visual behavior changed.

![Trueshot Aura inline preview](images/screenshots/2026-10-27-trueshot/preview.png)
![Purchased Trueshot Aura](images/screenshots/2026-10-27-trueshot/purchased.png)
![Compact saved Trueshot Aura](images/screenshots/2026-10-27-trueshot/compact.png)

## 2026-10-27 — Consistent starter Options backdrop

Opening Options from the starter or Armory now retains the landscape instead of
revealing the world camera. In-game pause keeps its translucent world overlay.
Native macOS before/after captures verify the Collection transition and return at
800×600 with 125% interface size. Gameplay transition coverage is headless.
All 386 Survivors app tests, app/test Clippy and native build pass.

![Before: Options exposed the world camera](images/screenshots/2026-10-27-menu-options/before.png)
![After: Options retains the starter landscape](images/screenshots/2026-10-27-menu-options/after.png)

## 2026-10-27 — Windows playtest review

Consolidated foliage reveal, enemy routing, elite chest cadence and talent art
into a [playtest acceptance tracker](WINDOWS-PLAYTEST.md). All 385 Survivors app
tests pass with checkout-local data. No runtime changes in this checkpoint;
reported Windows scenes remain open. Earlier talent screenshots are retained
inline below and in the tracker, not presented as new captures.

## 2026-10-27 — Short-window Collection acceptance

Verified Collection at 800×600 with 125% interface size: item actions remain
reachable, cosmetic selection does not change bonuses, and unequipping the helm
removes its stats while retaining its look. A full restart preserves that state.
Source navigation highlights Hogger and returns to the same Collection position.
The intention's three columns stack vertically here; this is native macOS evidence
using a synthetic account, not a new gameplay change or Windows acceptance.

![Scaled Collection actions](images/screenshots/2026-10-27-collection-800-125/equipped.png)
![Unequipped with an independent look](images/screenshots/2026-10-27-collection-800-125/unequipped.png)
![Selected source](images/screenshots/2026-10-27-collection-800-125/source.png)
![Look-only state after restart](images/screenshots/2026-10-27-collection-800-125/restarted.png)

## 2026-10-27 — Mage Arcane Power

Arcane Power now grants +30% Mage damage for 15 seconds after a successful cast,
with three-minute recovery and an active-buff countdown. Failed targeting preserves
readiness; pauses stop timers. Casts retain their damage bonus until they end.

201 core/385 app tests, both Clippy gates and native build pass. Native wide/compact
captures verify the green preview, purchase and saved rank. Definitions reach
247/432; combat HUD appearance, Windows playtesting and live balance remain open.

![Arcane Power preview](images/screenshots/2026-10-27-arcane-power/preview.png)
![Arcane Power purchase](images/screenshots/2026-10-27-arcane-power/purchased.png)
![Compact Arcane Power description](images/screenshots/2026-10-27-arcane-power/compact.png)

## 2026-10-27 — Rogue Precision

Precision now grants +1% Rogue ability damage per rank, up to 5%, including
periodic damage. It multiplies other talent bonuses and excludes foreign abilities.
The description identifies this as a hit-chance conversion; attacks have no miss roll.

200 core/383 app tests and both Clippy gates pass; native build took 17 seconds.
Native wide/compact captures verify the inline preview, purchase and saved rank.
Talent definitions reach 246/432. Live balance and Windows acceptance remain open.
Save integration additionally checks Precision against the real DBC row rules,
spec switching, reload, respec and unchanged active-run snapshots.

![Precision preview](images/screenshots/2026-10-27-rogue-precision/preview.png)
![Precision purchased](images/screenshots/2026-10-27-rogue-precision/purchased.png)
![Compact Precision description](images/screenshots/2026-10-27-rogue-precision/compact.png)

## 2026-10-27 — Consistent hero selection at run entry

An explicit missing or wrong-class hero now refuses run entry and cannot borrow
another hero's bonuses. The refusal preserves the previous run. Selecting a valid
hero restores entry; the existing class-only fallback remains available when no
hero was explicitly selected. This closes a state-consistency gap found in the
entry code, not a claimed reproduction of a normal UI selection failure.

382 app tests and the native build pass; app-test Clippy passes.

## 2026-10-27 — Windows executable gate

The Windows CI job now links the player executable after the development check
and core rules, and uploads that executable alone as a seven-day playtest artifact.
This catches linker failures that `cargo check` cannot catch. Artifact names identify
the checked-out SHA; game data and local saves are excluded.

Workflow YAML parses; the no-default-features player check and all 199 core tests
under the play profile pass locally on macOS (five non-dev compiler warnings).
GitHub currently reports zero workflow runs. This is configured coverage, not a
successful Windows build or available download; execution remains unverified.

## 2026-10-27 — Paladin Divine Favor

Divine Favor readies one guaranteed-critical Holy Shock, recovering 120 active
seconds after a successful cast. All direct targets benefit; other spells cannot
consume it. Healing remains separate. Other classes retain their 180-second timers.

199 core and 380 app tests pass, including actual multi-target casts and failed-target
retention. Core/app-test Clippy and native build pass (16-second build).
Native wide/compact captures verify
the inline value, purchase, saved rank and full description. Definition coverage
reaches 245/432 (Paladin 24/44); live balance remains open.

![Divine Favor inline preview](images/screenshots/2026-10-27-divine-favor/preview.png)
![Divine Favor purchase](images/screenshots/2026-10-27-divine-favor/purchased.png)
![Compact Divine Favor description](images/screenshots/2026-10-27-divine-favor/compact.png)

## 2026-10-27 — Priest Inner Focus

Inner Focus adds 25 percentage points critical chance to the next successful
Smite, Holy Fire, Holy Nova or Mind Blast, then recovers after 180 active seconds.
Failed targeting preserves the charge; periodic damage and defensive spells are
excluded. Native previews verify the green inline value, purchase and saved rank.

198 core and 379 app tests pass, including actual multi-target casts, recovery,
pause and unchanged attached burns. Core/app-test Clippy pass; the native
incremental build took 17 seconds.
Definition coverage is 244/432 (Priest 24/47); live balance remains open.
Save/catalog integration also verifies that effects wait for the real DBC rules,
invalid sibling specs remain inactive and repairable, hero/spec switches stay
isolated, and a running snapshot survives a later respec unchanged.

![Inner Focus inline preview](images/screenshots/2026-10-27-inner-focus/preview.png)
![Purchased Inner Focus and remaining points](images/screenshots/2026-10-27-inner-focus/purchased.png)

Compact acceptance: at 640×720 and 125% interface size, the complete description
is reachable by scrolling. Native refund/repurchase succeeds and saves rank 1;
the inline comparison becomes the trained value without repeating the unit.

![Compact Inner Focus preview at 125%](images/screenshots/2026-10-27-inner-focus/compact-640-125.png)
![Compact purchased Inner Focus at 125%](images/screenshots/2026-10-27-inner-focus/compact-purchased-640-125.png)

## 2026-10-27 — Foliage reveal diagnostics

An opt-in visibility trace records the rendered blocker, camera/hero positions,
probe hits, reveal radius and fade strength. A documented Windows capture workflow
helps separate missed sightlines from shader failures without changing gameplay.
The three visibility tests pass against checkout Elwynn canopy assets, including
both triangle sides, behind-hero exclusion and real-time fading. Native build and
app-test Clippy pass. The reported
Windows scene remains unverified; this checkpoint does not claim a visual fix.

## 2026-10-27 — Live talent layout tuning

Talents now uses the live Armory typography, padding and gap settings in
`crates/survivors-core/ui.txt`. Native development builds apply valid edits without
a Rust rebuild, preserving the selected talent and spec. Default appearance and
proportional tree geometry are unchanged; packaged builds retain embedded settings.

197 core tests, app-test Clippy and the native build pass (28 seconds).
Same-process previews verify enlarged text at 1280×720 and 640×720, scrolling to
the complete description, rejection of invalid settings and restoration of defaults.
The enlarged captures demonstrate development tuning, not new shipped defaults.

![Default talent layout](images/screenshots/2026-10-27-talent-live-layout/default-1280.png)
![Live enlarged talent typography](images/screenshots/2026-10-27-talent-live-layout/live-1280.png)
![Compact live typography with complete description](images/screenshots/2026-10-27-talent-live-layout/live-640.png)

## 2026-10-27 — Clear talent purchase feedback

Unavailable purchases now distinguish no points, maximum rank, missing
prerequisites and earlier-row requirements (including required/current totals).
Invalid saved trees receive repair guidance. Rejections preserve saved ranks;
the first real-tree purchase still returns legacy-spec points to the pool.

377 app tests, app-test Clippy and the native build pass (27-second incremental
build). Native 1280×720 captures verify the no-points and row-gate cases.

![No points available](images/screenshots/2026-10-27-talent-feedback/no-points-1280.png)
![Exact earlier-row requirement](images/screenshots/2026-10-27-talent-feedback/row-gate-1280.png)

## 2026-10-27 — Current browser UI checkpoint

The current WebGPU package includes the painted starter and latest talent rules.
It restores the previously created female Human Mage and renders the starter at
requested 1280×720 and 640×720 preview sizes. In the normal narrow browser viewport
(551×751 CSS pixels), Talents navigation, scrolling, purchase rejection without
points and the full green inline Cold Snap description were verified.

The incremental wasm build took 87 seconds; packaging passed with five existing
wasm warnings. Sampled browser error/warning logs were empty. The preview tool's
viewport override interfered with click coordinates after a display-scale change;
interaction was verified after clearing that override. These captures do not
establish gameplay, Windows behavior or interaction at the overridden sizes.

![Current browser starter](images/screenshots/2026-10-27-browser-current/starter-1280.png)
![Compact browser starter](images/screenshots/2026-10-27-browser-current/starter-640.png)
![Scrollable browser talent details](images/screenshots/2026-10-27-browser-current/talent-details-551.png)

## 2026-10-27 — Rogue Preparation

Preparation now resets owned Rogue ability cooldowns every 180 seconds of active
run time. Missed resets never become stored charges; Cold Blood recovery remains
separate. Native preview/purchase checks verify the green inline effect, 20 → 21
Subtlety points, 10 → 9 unspent points and saved rank 1.

197 core and 376 app tests pass, including real cast-system cooldown checks;
native build and core/app-test Clippy pass. Talent definitions cover 243/432
nodes (Rogue 21/51). Live balance and Windows gameplay remain open.

![Preparation inline preview](images/screenshots/2026-10-27-preparation/preview.png)
![Preparation purchased and saved](images/screenshots/2026-10-27-preparation/purchased.png)

## 2026-10-27 — Painted starter landscape

An original sunset valley replaces the starter's procedural mountain silhouette.
The embedded artwork crops proportionally at different window sizes, with a dark
vignette keeping the gold headings and controls readable. [Artwork provenance and prompt](../crates/benilla-app/src/survivors/backdrop/ARTWORK.md).

Native build and app-test Clippy pass. Native previews verify 1280×720 and
640×720 starter crops and the separate talent-tree artwork; no shader errors
appeared in the validation log. Windows and browser rendering remain unverified
for this change.

![Wide starter landscape](images/screenshots/2026-10-27-starter-landscape/starter-1280.png)
![Compact starter landscape](images/screenshots/2026-10-27-starter-landscape/starter-640.png)
![Talent artwork check](images/screenshots/2026-10-27-starter-landscape/talents-1280.png)

## 2026-10-27 — Starter hero identity

The starter now follows the main-menu reference with a prominent clickable name
field and a body-matched racial portrait beside the bonus list. Race, class and
sex appear together below the name; Random remains adjacent.

Native 1280×720/640×720 checks cover direct rename, Save/Cancel, saved names,
Random, male/female portrait changes and compact scrolling. Native build and
app-test Clippy pass. The existing split first/last-name editor is retained;
remaining starter artwork and wider resolution/controller acceptance stay open.

![Starter hero identity](images/screenshots/2026-10-27-starter-identity/starter-1280.png)
![Compact name control](images/screenshots/2026-10-27-starter-identity/starter-640.png)
![Compact racial summary](images/screenshots/2026-10-27-starter-identity/racials-640.png)

## 2026-10-27 — Shaman Elemental Mastery

Elemental Mastery guarantees the next successful lightning/shock cast's direct
critical hits, including delayed Chain Lightning bounces. Ready at run start,
180-second recovery after use; pauses and empty target selections preserve it.
Flame Shock's burn, totems and Lightning Shield receive no bonus.

196 core and 375 app tests, core/app-test Clippy, native build and WebGPU check
pass (five existing wasm warnings). Definition coverage: 242/432, Shaman 30/46.
Native 1280×720 preview/purchase checks confirm
the green inline bonus, 30 → 31 Elemental points, and saved rank 1. Live balance
remains open.

![Elemental Mastery inline preview](images/screenshots/2026-10-27-elemental-mastery/preview.png)
![Elemental Mastery purchased and saved](images/screenshots/2026-10-27-elemental-mastery/purchased.png)

## 2026-10-27 — Rogue Cold Blood

Cold Blood now guarantees the next eligible Rogue direct cast's critical hits,
including extra targets. Ready at run start, it recovers 180 seconds after use;
failed target selections and pauses do not spend it. The existing Eviscerate
milestone stays separate. Live balance remains open.

195 core and 374 app tests, core/app-test Clippy and native build pass. Talent
definitions now cover 241/432 nodes (Rogue 20/51). Native 1280×720 checks verify the green inline
preview, purchase (23 → 24 Assassination points) and saved rank 1.

![Cold Blood inline preview](images/screenshots/2026-10-27-cold-blood/preview.png)
![Cold Blood purchased and saved](images/screenshots/2026-10-27-cold-blood/purchased.png)

## 2026-10-27 — Browser starter and saved hero

The real WebGPU development package boots through the loopback host using this
checkout's data. At 1280×720, verified starter → Character → create Mage → choose
female Human → reload → restored hero, then Talents. The sampled browser error/
warning logs were empty. No combat run or broader browser acceptance is claimed.

![Browser starter](images/screenshots/2026-10-27-browser-starter/starter.png)
![Created browser hero](images/screenshots/2026-10-27-browser-starter/created.png)
![Hero restored after browser reload](images/screenshots/2026-10-27-browser-starter/reloaded.png)
![Browser talent trees](images/screenshots/2026-10-27-browser-starter/talents.png)

## 2026-10-27 — Browser build validation

Browser builds now reject a mismatched wasm-bindgen CLI before compiling. Six
script tests pass, including CRLF lockfiles and missing/mismatched pins. A local
0.2.122 CLI is documented and ignored alongside the existing build tools.

Current WebGPU check, linked play build and real development packaging pass
(five existing wasm warnings). First target/profile build: 5m 05s. An unchanged
cached package repeat: 6.85s total, Cargo 0.30s with no crates rebuilt. This proves
cache reuse, not Rust-edit latency, release optimization or browser gameplay.

## 2026-10-27 — Improved Mend Pet

The living combat wolf regenerates 1% maximum health per second per rank (two
ranks). This automatic Survivors conversion does not heal the Hunter, revive a
defeated wolf, shorten recovery or dispel effects; pauses stop regeneration.

194 core tests, 373 app tests, core/app-test Clippy and native build pass. The
actual pet-system regression checks healing over time, rank bounds, pause,
health cap and defeat. Native 1024×720 at 125% verifies the inline preview and
saved spec-3 purchase (35→36 points). Coverage is 240/432, Hunter 30/46;
live balance remains open.

![Hunter trees](images/screenshots/2026-10-27-mend-pet/trees.png)
![Mend Pet preview](images/screenshots/2026-10-27-mend-pet/preview.png)
![Saved Mend Pet purchase](images/screenshots/2026-10-27-mend-pet/purchased.png)

## 2026-10-27 — Starter heading alignment

Class, Hero, Equipment and Zone now share the intention reference's centered
headings and fading gold dividers. Flexible lines follow each panel's width,
including the compact stacked layout. Native 1280×720 and 640×720 checks verify
all four headings and scrolling; native build and app-test Clippy pass.

![Before: starter headings](images/screenshots/2026-10-27-starter-dividers/before-1280.png)
![After: starter headings](images/screenshots/2026-10-27-starter-dividers/after-1280.png)
![Compact starter](images/screenshots/2026-10-27-starter-dividers/after-640.png)
![Compact equipment and zone sections](images/screenshots/2026-10-27-starter-dividers/scrolled-640.png)

## 2026-10-27 — Building reveal eligibility

Removed the WMO exclusion from both Survivors scenery shader paths. Buildings
between the camera and hero can now use the same circular reveal as static trees;
the shared depth limit preserves surfaces behind the hero. Actor/UI exclusions
and physical collision remain unchanged.

The scheduled visibility regression now covers WMO and doodad identities on
ordinary and merged paths, including the 500 ms fade and behind-hero reversal.
All 372 Survivors app tests, app-test Clippy and the native play build pass.
Rendered building screenshots and the exact Windows foliage replay remain open;
this correction does not establish the cause of the foliage report.

## 2026-10-27 — Cold Snap

Mage Cold Snap automatically resets owned Frostbolt, Frost Nova, Blizzard and
Cone of Cold cooldowns every 90 seconds of run time, first at 90 seconds. It
neither grants spells nor banks missed resets; other spells and existing areas
remain unchanged. This is an explicit automatic Survivors conversion.

193 core tests, 372 app tests, core/app-test Clippy and native build pass. Native
1024×720 at 125% verifies the inline preview and saved purchase (18→19 Frost
points). Coverage is 239/432 definitions, Mage 32/49. Live balance remains open.

![Mage talent trees](images/screenshots/2026-10-27-cold-snap/trees.png)
![Cold Snap preview](images/screenshots/2026-10-27-cold-snap/preview.png)
![Cold Snap purchased in spec 3](images/screenshots/2026-10-27-cold-snap/purchased.png)

## 2026-10-27 — Elwynn canopy visibility coverage

Added render-only regressions for all four Elwynn canopy models, from both sides,
with rotated/scaled placements and ordinary/merged geometry. All 371 app tests
and app-test Clippy pass. The actual Windows scene remains unverified; these tests do not check alpha
pixels, streaming visibility or the rendered fade. No interface change.

## 2026-10-27 — Direct saved-spec previews

The fixture launcher accepts paired `--hero` / `--spec` options with a talent
preset. It opens a chosen saved build in a new disposable account without manual
save edits. Invalid preset/spec combinations are rejected before creating the home,
so corrected requests can reuse the same path. Nine isolation tests pass locally
and are now included in the host/UI PR job; native 640×720 confirms hero 2, spec 2,
and the expected 29-point Enhancement spread. No Rust rebuild is needed.

![Selected hero and saved spec from the fixture command](images/screenshots/2026-10-27-spec-previews/selected-spec.png)

## 2026-10-27 — Bestial Discipline

Hunter training now shortens the combat wolf's bite cooldown by 10% per rank
(two ranks), multiplying the 20-point milestone and Frenzy haste. This explicit
Survivors pet-focus conversion leaves Hunter spells and wolf recovery unchanged.

192 core tests, 370 app tests, core/app-test Clippy and native build pass.
The actual wolf-system regression verifies no early bite, expiry, zero-time pause,
rank bounds and milestone/Frenzy combinations. Native 1024×720 at 125% verifies
the inline preview and a purchase saved to spec 3 (34→35 points). Definition
coverage is 238/432, Hunter 29/46; live balance and Windows acceptance remain open.

![Hunter talent trees](images/screenshots/2026-10-27-bestial-discipline/trees.png)
![Discipline before training](images/screenshots/2026-10-27-bestial-discipline/preview.png)
![Purchased rank and inline next upgrade](images/screenshots/2026-10-27-bestial-discipline/purchased.png)

## 2026-10-27 — Talent onboarding

Fresh accounts now get a direct **Create a hero** action on Talents, with a short
explanation of earned points, three saved specs and free resets. The heading and
action use the Armory's existing gold/black design.

Native 640×480 at 125% verifies the action opens the creation form without changing
saved account bytes. Native build and app-test Clippy pass; no gameplay rules changed.

![Previous empty talent page](images/screenshots/2026-10-27-talent-onboarding/before.png)
![Talent onboarding](images/screenshots/2026-10-27-talent-onboarding/after.png)
![Direct route to hero creation](images/screenshots/2026-10-27-talent-onboarding/create-route.png)

## 2026-10-27 — Creation feedback at minimum window size

Fixed stale name-validation feedback after correcting a creation or rename field.
Rejected input leaves feedback visible. Native 640×480 checks verify reachable
creation controls, empty-name rejection, longer names and Tab navigation, with
unchanged saved account bytes. 369 app tests, app-test Clippy and the native build
pass. Windows DPI and controller acceptance remain open.

![Empty-name rejection at minimum window size](images/screenshots/2026-10-27-creation-minimum/empty-name-640-480.png)
![Stale error before the fix](images/screenshots/2026-10-27-creation-minimum/stale-error-before.png)
![Corrected name clears the message](images/screenshots/2026-10-27-creation-minimum/corrected-name-640-480.png)

## 2026-10-27 — Clearer hero creation

Hero creation now labels both name fields and presents all nine classes directly.
The form wraps at compact widths and separates creation from account upgrades.
Creating a hero opens their appearance controls without obsolete “Open Look” guidance.

369 app tests, app-test Clippy and the native build pass. Native checks at
640×720 and 1024×720, both at 125%, verify name entry, Druid creation, appearance
persistence after restart, and draft class changes leaving the saved hero unchanged.

![Previous compact creation controls](images/screenshots/2026-10-27-hero-creation/before-640-125.png)
![Labeled compact hero creation](images/screenshots/2026-10-27-hero-creation/form-640-125.png)
![Independent class selection with an existing hero](images/screenshots/2026-10-27-hero-creation/form-1024-125.png)
![Saved appearance after restart](images/screenshots/2026-10-27-hero-creation/restarted-1024-125.png)

## 2026-10-27 — Incremental browser development builds

`WEB_PROFILE=play scripts/web-build.sh` now reuses incremental compilation and
skips optimization/compression. Stale compressed files are removed so the local
host serves the new build. Also fixed `WEB_DEBUG=1` on macOS Bash 3.2.
Default release/ship packaging retains its postprocessing.

Four build-orchestration regressions and shell syntax checks pass; CI includes
the tests. WebGPU compilation passes with five existing warnings. Full wasm
packaging/timing and Windows CI remain unverified. No interface change.

## 2026-10-27 — Conflagrate training

Conflagrate makes successful Immolate casts advance an owned Shadowburn cooldown
by 2 seconds, adding to the Destruction milestone for 4 seconds total. It triggers
once per cast, preserves the burn, and cannot grant a spell or bank a charge.

191 core tests, 369 app tests, core/app-test Clippy and native build pass.
Native previews verify green inline values, purchase/save/restart, and readable
1024×720/640×720 layouts at 125%. Coverage: 237/432 nodes, Warlock 27/50.
Live balance and Windows acceptance remain open.

![Proportional Warlock talent trees](images/screenshots/2026-10-27-conflagrate/trees-1024-125.png)
![Conflagrate inline upgrade preview](images/screenshots/2026-10-27-conflagrate/preview-1024-125.png)
![Saved Conflagrate in the compact layout](images/screenshots/2026-10-27-conflagrate/restarted-640-125.png)

## 2026-10-27 — Elwynn tree route regression

Added movement playback around six actual Elwynn tree hulls from four directions
(24 routes). Each case requires clear endpoints, a blocked direct body sweep, a
bounded A* detour, and body-clear steps all the way to the destination. Existing
fence and three chest-model cases remain covered.

368 Survivors app tests, app-test Clippy and formatting pass with checkout-local
data. This is headless validation
on flat ground, not Windows scene acceptance; no runtime or visual change.
Rotated placements, terrain, crowds and foliage rendering remain live checks.

## 2026-10-27 — Achievable completion totals

Completion now counts the 56 achievements with defined criteria. Deathless Hour
and no u remain visible and locked without preventing 100% completion; completed
Feats correctly shows 3/3. Old saved placeholder IDs cannot award credit.

190 core tests, 367 app tests, core/app-test Clippy and native build pass.
Seeded native previews at 640×720/125% verify completed Feats and the Rewards total.

![Completed available Feats with pending cards retained](images/screenshots/2026-10-27-achievement-completion/feats-640-125.png)
![Rewards counts available achievements](images/screenshots/2026-10-27-achievement-completion/rewards-640-125.png)

## 2026-10-27 — Enlarged Codex and source navigation

Verified populated Codex and Drop Tables at 640×720/125%. All 39 shared jewelry
entries wrap within the catalog; selecting an amulet preserves its scroll position
and shows the green bonus. Collection's source button opens Elwynn's table with
the matching item selected and its chance visible.

Native macOS preview on `7d054a5`, seeded disposable account; no code changes or
new test run. These captures verify layout/navigation, not natural acquisition or
completion of the unfinished zone tables.

![Populated compact Codex](images/screenshots/2026-10-27-scaled-codex/codex-640-125.png)
![Collection source navigation](images/screenshots/2026-10-27-scaled-codex/source-640-125.png)
![Matching drop-table destination](images/screenshots/2026-10-27-scaled-codex/drops-640-125.png)

## 2026-10-27 — Improved Drain Life and readable talent details

Improved Drain Life strengthens Death Coil drain healing by 2% per rank (10%
at rank five), without increasing damage or Healthstone healing. Overkill is
excluded. Long talent descriptions now expand their panels so compact scrolling
can reach the entire explanation instead of clipping its ending.

189 core tests, 366 app tests, core/app-test Clippy and native build pass.
Native 640×720 and 1024×720 at 125% verify green inline text, saved purchase,
full descriptions and proportional trees. Coverage: 236/432 nodes, Warlock 26/50.
Live balance and Windows acceptance remain open.

![Before: talent explanation clipped](images/screenshots/2026-10-27-drain-life/clipped-before.png)
![Full compact talent explanation](images/screenshots/2026-10-27-drain-life/talents-640-125.png)
![Three proportional talent trees](images/screenshots/2026-10-27-drain-life/trees-1024-125.png)
![Readable expanded details](images/screenshots/2026-10-27-drain-life/talents-1024-125.png)

## 2026-10-27 — Drain and leech exclude overkill

Drain and gear leech now heal from enemy health actually removed, matching
Siphon Life. Direct multistrikes and periodic damage no longer turn overkill
into extra healing. Damage totals are unchanged.

366 app tests, app-test Clippy and native build pass. The regression covers stacked
healing, health caps, dead heroes and no repeat healing from dead targets.
No interface change; live balance remains open.

## 2026-10-27 — Tabards now dress the hero

Fixed a missing connection between saved tabard selection and the character
compositor. Guild Tabard now appears over chest armor; removing it restores
the equipped chest look. The shared menu/run hero path receives the cosmetic
slot without changing gear or stats.

366 app tests, app-test Clippy and native build pass. Native 640×720/125%
checks reproduce the missing garment, verify the saved tabard after restart,
and verify removal. Live combat and other bodies remain acceptance work.

![Compact tabard selection](images/screenshots/2026-10-27-scaled-rewards/tabards-640-125.png)
![Before: saved tabard absent](images/screenshots/2026-10-27-scaled-rewards/tabard-before.png)
![After: saved tabard rendered](images/screenshots/2026-10-27-scaled-rewards/tabard-after.png)
![Removing the tabard restores chest armor](images/screenshots/2026-10-27-scaled-rewards/tabard-removed.png)

## 2026-10-27 — Enlarged achievement and reward checks

At 640×720/125%, achievement cards, companion locks and green perk previews
remain readable and scrollable. Confirming Banish spends 15 points and saves
rank 1; the balance changes from 285 to 270. Deathless Hour and no u retain
pending criteria and locked companions.

Native macOS preview on `e254ce2`, disposable fixture; save-file comparison
confirms the purchase. No code change or new test run. Tabards, other scales
and Windows acceptance remain open.

![Compact achievement criteria](images/screenshots/2026-10-27-scaled-rewards/achievements-640-125.png)
![Pending perk with inline upgrade](images/screenshots/2026-10-27-scaled-rewards/perks-pending-640-125.png)
![Confirmed reward balance](images/screenshots/2026-10-27-scaled-rewards/confirmed-640-125.png)
![Companion appearance locks](images/screenshots/2026-10-27-scaled-rewards/companions-640-125.png)

## 2026-10-27 — Enlarged Collection and Talents checks

Verified full Collection and Warlock Talents at 640×720 with 125% interface
size. Equipment rows wrap; item details and talent panels scroll. Unequip and
re-equip preserve cosmetic overrides, and a talent purchase saves correctly.
Talent art retains its proportions with aligned icons and connectors.

Native macOS preview on `e254ce2`, disposable full-account fixture; no code
changes or new test run. Exact Windows foliage, fence and chest scenes remain
open in the implementation plan.

![Compact Collection at 125 percent](images/screenshots/2026-10-27-scaled-armory/collection-640-125.png)
![Unequipped item with inline comparisons](images/screenshots/2026-10-27-scaled-armory/unequipped-640-125.png)
![Proportional talent artwork at 125 percent](images/screenshots/2026-10-27-scaled-armory/trees-640-125.png)
![Purchased talent and scrolling details](images/screenshots/2026-10-27-scaled-armory/talents-640-125.png)

## 2026-10-27 — Adjustable interface size

Options → Graphics now offers Interface size at 75%, 85%, 100%, 110% and 125%,
independently of render resolution. The preference survives restart and remains
relative to window height. Graphics defaults restores automatic sizing.

365 app tests, app-test Clippy and native build pass. Tests cover 480/720/1080
heights at 1×/2× display scale, bounded adjustments and default restoration.
Native 1280×720 and 640×720 checks verify 125% Options, Character reflow,
restart persistence and reset. Other pages and gameplay at every scale remain
acceptance work.

![Enlarged interface with independent render scale](images/screenshots/2026-10-27-interface-scale/options-125.png)
![Compact Options at 125 percent](images/screenshots/2026-10-27-interface-scale/options-640-125.png)
![Compact Character after restarting at 125 percent](images/screenshots/2026-10-27-interface-scale/restarted-640-125.png)

## 2026-10-27 — Soul Link talent

Soul Link now adds a three-second, 5% maximum-health absorb to successful
Death Coil casts, combining with the Demonology milestone for 10%. Refreshes
do not stack, and stronger active shields are preserved. Green inline text
explains the upgrade; damage, cooldown and targeting are unchanged.

188 core tests, 364 app tests, core/app-test Clippy and native build pass.
Wide/compact native captures verify the description and saved purchase in a
seeded prerequisite fixture. Catalog coverage: 235/432 nodes (Warlock 25/50).
Demonology prerequisites still include unfinished talents; live balance remains open.

![Soul Link inline upgrade](images/screenshots/2026-10-27-soul-link/talents-1280.png)
![Compact purchased Soul Link](images/screenshots/2026-10-27-soul-link/talents-640.png)

## 2026-10-27 — Character body and saved appearance checks

Expanded native Character acceptance to dressed Orc female and Undead male
Warlocks at 1280×720, plus Gnome female at 640×720. Race/sex changes preserve
worn gear and independent looks. Compact hairstyle adjustment survives a full
restart; the hidden helm and alternate shoulder appearance remain intact.

Disposable full-collection fixture, runtime `25ebf07`; no gameplay changes.
363 app tests passed at this runtime. These captures extend macOS UI coverage;
the complete race/resolution matrix and Windows gameplay acceptance remain open.

![Dressed Orc in the reference-shaped three-column layout](images/screenshots/2026-10-27-warlock-character/orc-1280.png)
![Dressed Undead after race and sex changes](images/screenshots/2026-10-27-warlock-character/undead-1280.png)
![Compact customization after changing hairstyle](images/screenshots/2026-10-27-warlock-character/features-640.png)
![Gnome appearance restored after restart](images/screenshots/2026-10-27-warlock-character/restarted-640.png)

## 2026-10-27 — Curse of Exhaustion

Curse of Exhaustion now adds a 12-second slow to Curse of Agony applications.
Its improvement adds slow strength, requires the base talent and caps this
curse's boss slow at 25%. Damage ticks do not refresh it; stronger existing
slows are preserved. Green inline values explain both upgrades.

187 core tests, 363 app tests, core/app-test Clippy and native build pass.
Native wide/compact captures verify prerequisite unlocking and saved purchases.
Catalog coverage: 234/432 nodes (Warlock 24/50). Live balance and Windows combat
acceptance remain open.

![Curse of Exhaustion unlock](images/screenshots/2026-10-27-warlock-exhaustion/unlock-1280.png)
![Improved curse after purchase](images/screenshots/2026-10-27-warlock-exhaustion/improved-640.png)

## 2026-10-27 — Intensity and reliable area pulses

Intensity now extends Rain of Fire by 10% per rank, completing the effect for
Pyroclasm's prerequisite. Ground areas also deliver every pulse due during a
long frame, bounded by their remaining lifetime; expired areas cannot keep hitting.
Pulse rate, damage per pulse and radius are unchanged.

186 core tests, 362 app tests, core/app-test Clippy and native build pass.
Cast-to-area tests count actual hits across 0.1/0.6/2.5/20-second steps. Native
captures verify inline upgrades, the capped description and a saved rank-two
purchase. Catalog coverage: 232/432 nodes (Warlock 22/50). Live balance and
Windows combat acceptance remain open.

![Intensity inline duration upgrade](images/screenshots/2026-10-27-warlock-intensity/talents-1280.png)
![Compact capped Intensity](images/screenshots/2026-10-27-warlock-intensity/talents-640.png)

## 2026-10-27 — Warlock Destruction control

Aftermath now gives Destruction hits a chance to slow enemies; Pyroclasm gives
Rain of Fire and Hellfire pulses a chance to stun. Both preserve stronger control
and apply boss adjustments. Attached damage-over-time ticks do not trigger them.
Green inline descriptions show the next rank's chance.

185 core tests, 361 app tests, core/app-test Clippy and native build pass.
Native previews verify wide/compact text and a saved Pyroclasm purchase.
Catalog coverage: 231/432 nodes (Warlock 21/50). Live balance and Windows
combat acceptance remain open.

![Pyroclasm inline upgrade](images/screenshots/2026-10-27-warlock-control/pyroclasm-1280.png)
![Compact Aftermath description](images/screenshots/2026-10-27-warlock-control/aftermath-640.png)

## 2026-10-27 — Improved Kidney Shot

The Rogue talent now increases the target's damage taken by 3% per rank during
Kidney Shot's stun. It affects subsequent hits and periodic damage, refreshes
without stacking, uses shorter boss duration, and is not extended by other
stuns. Refunding the talent removes its damage bonus immediately.

184 core tests, 360 app tests, core/app-test Clippy and native build pass.
Enemy-update coverage also verifies independent expiry at three frame sizes and
a frozen simulation delta.
Native wide/compact previews verify green inline upgrades and a saved rank-one
purchase. Catalog coverage is 229/432 nodes (Rogue 19/51); live balance and
Windows combat acceptance remain open.

![Improved Kidney Shot upgrade](images/screenshots/2026-10-27-rogue-kidney/talents-1280.png)
![Compact view after training rank one](images/screenshots/2026-10-27-rogue-kidney/talents-640.png)

## 2026-10-27 — Explain equipment set changes

Item comparisons now name full-loadout and Raid Epic set bonuses gained or lost
when changing equipment. This explains stat changes beyond the item's own
bonuses; equipment mechanics are unchanged. Both legacy and supplemental item
panels share the explanation.

358 app tests, app-test Clippy and native build pass. Native 1024×768 captures
verify removing a shoulder piece and previewing its replacement; headless UI
coverage checks raid thresholds across all nine classes.

![Removing shoulders also loses the full-loadout bonus](images/screenshots/2026-10-27-set-comparison/loss-1024.png)
![Equipping shoulders restores the full-loadout bonus](images/screenshots/2026-10-27-set-comparison/gain-1024.png)

## 2026-10-27 — Druid Character at 1024×768

Native acceptance checks cover a full legacy Druid collection: dressed male and
female Night Elves, drag rotation, saved appearance changes without changing
selected gear, lower equipment/appearance controls, and the same shoulder item
after opening Collection. At this size the reference's three columns stack into
a scrollable page. No code changed; other race/resolution combinations and in-run
appearance remain open.

![Rotated dressed male Night Elf](images/screenshots/2026-10-27-druid-character/rotated-1024.png)
![Female Night Elf with the same equipment](images/screenshots/2026-10-27-druid-character/female-1024.png)
![Equipment and readable bonuses](images/screenshots/2026-10-27-druid-character/equipment-1024.png)
![Same shoulder item opened in Collection](images/screenshots/2026-10-27-druid-character/collection-1024.png)

## 2026-10-27 — Inline one-rank talent bonuses

One-rank talents with an unambiguous numeric bonus now show green changes inside
the description instead of a separate summary. Fixed explanatory numbers stay
unchanged; capped talents show only their current bonus. No mechanics changed.

183 core tests, core Clippy and native build pass. Native wide/compact captures
verify Swiftmend's upgrade and capped Nature's Grasp. Broader UI acceptance remains open.

![Swiftmend inline healing upgrade](images/screenshots/2026-10-27-single-rank-inline/swiftmend-1280.png)
![Capped Nature's Grasp without redundant summary](images/screenshots/2026-10-27-single-rank-inline/grasp-640.png)

## 2026-10-27 — Rogue finisher recovery

Ruthlessness and Relentless Strikes now shorten Eviscerate, Rupture and Kidney
Shot cooldowns, composing with existing training. Both explicitly describe their
Survivors conversions and show green inline upgrades. Native checks verify a
saved Relentless Strikes purchase and compact Ruthlessness details.

182 core tests, 357 app tests, core/app-test Clippy and native build pass.
Catalog coverage: 228/432 individual talents (Rogue 18/51). Live balance and
Windows acceptance remain open.

![Relentless Strikes inline preview](images/screenshots/2026-10-27-rogue-finishers/talents-1280.png)
![Compact Ruthlessness upgrade](images/screenshots/2026-10-27-rogue-finishers/talents-640.png)

## 2026-10-27 — Druid root-control training

Nature's Grasp and Improved Entangling Roots now shorten Entangling Roots'
cooldown; Improved Nature's Grasp extends its root duration. Bonuses compose
with the Balance milestone and preserve boss duration reduction. These are
explicit Survivors conversions, with green inline upgrade values.

181 core tests, 357 app tests, core/app-test Clippy and native build pass.
Hit-resolution coverage verifies normal/boss duration, non-stacking refreshes,
preservation of stronger roots and baseline duration after refunds. Native
wide/compact previews verify wrapping and a saved rank-two purchase.
Catalog coverage: 226/432 individual talents, including 23/47 Druid nodes.
Live balance and Windows combat acceptance remain open.

![Druid root duration upgrade](images/screenshots/2026-10-27-druid-roots/talents-1280.png)
![Compact preview after training rank two](images/screenshots/2026-10-27-druid-roots/talents-640.png)

## 2026-10-27 — Chest reward burst verification

Actual death-event batches now verify elite chest spacing and the five-per-run
cap, independent boss rewards, and no duplicate drops on idle frames. This
extends the existing pacing checks through deferred chest spawning. All 356 app
tests pass. No visual change; Windows foliage/fence/chest scene acceptance remains open.

## 2026-10-27 — Ordinary-enemy source shortcuts

Poor gear details now link directly to both Elwynn and Duskwood drop tables,
preserving the inspected item and progression locks. Raid gear still clearly
shows that acquisition is pending. Drop rates and ownership rules are unchanged.

355 app tests, app/test Clippy and native build pass. Native wide/compact captures
verify the two source links and Duskwood destination; Windows acceptance remains open.

![Poor gear’s two source links](images/screenshots/2026-10-27-poor-source-navigation/collection-1280.png)
![Duskwood source with the same item selected](images/screenshots/2026-10-27-poor-source-navigation/duskwood-1280.png)
![Compact Poor gear inspector](images/screenshots/2026-10-27-poor-source-navigation/collection-640.png)

## 2026-10-27 — Item source navigation

Collection item details now open the matching zone's Drop Tables directly. The
inspected item stays selected, destination scrolling resets, and the next-run
zone stays unchanged. This extends the reference's source panel with a shortcut.

354 app tests, the final scroll-reset regression, app/test Clippy and native build
pass. Native wide/compact clicks verify navigation and readable controls. Raid
sources remain pending; this change adds no drops or zone unlocks.

![Collection source shortcut](images/screenshots/2026-10-27-item-source-navigation/collection-1280.png)
![Matching source and inspected item](images/screenshots/2026-10-27-item-source-navigation/drops-1280.png)
![Compact source control](images/screenshots/2026-10-27-item-source-navigation/collection-640.png)

## 2026-10-27 — Shaman Nature's Swiftness

Nature's Swiftness now shortens Earth, Flame and Frost Shock cooldowns by 15%,
multiplying existing training and the Restoration milestone. Its green inline
preview shows the upgrade; buying it saves rank 1. This is the documented Survivors
instant-cast conversion, not a new active spell.

180 core tests, 352 app tests, core/app-test Clippy and native build pass. Native
wide/compact previews verify text, purchase and saved rank. Install audit: 223/432
individual talent definitions (Shaman 29/46). Live balance remains open.

![Nature's Swiftness upgrade preview](images/screenshots/2026-10-27-shaman-swiftness/talents-1280.png)
![Trained talent in the compact layout](images/screenshots/2026-10-27-shaman-swiftness/talents-640.png)

## 2026-10-27 — Saved raid-build integration coverage

Verified all nine classes' raid sets from saved equipment into run stats, including
every piece count through six, cosmetic-only ownership, unequipping, and preserving
an already-started run's stats. All 352 Survivors app tests pass. No gameplay or UI
change; raid acquisition and live balance remain unfinished.

## 2026-10-27 — Browser Survivors feature parity

The browser launcher now enables the same foliage visibility, terrain-height and
combat-text placement features as the native game. Previously, its Survivors entry
point compiled without those carries. WebGPU wasm compilation passes with the same
five existing warnings; Cargo's feature graph confirms all three features are enabled.
Browser rendering and gameplay validation remain open; no new visual capture is claimed.

## 2026-10-27 — Windows build coverage

Added a Windows CI gate for the native Survivors launcher, including its visibility,
terrain and text features, plus the engine-independent rules suite. Local validation:
179 rules tests pass and workflow YAML parses. The Windows job has not yet been
verified on its runner; foliage reveal, fence/chest navigation and crowded-wave
performance still require Windows scene replay. No visual changes in this checkpoint.


## 2026-10-27 — Ancestral Healing

Shaman totem deployment healing now activates Ancestral Healing: 12 armor per
rank for 10 seconds. Effective healing refreshes without stacking; full-health
casts, regeneration and totem attacks cannot trigger it. The HUD shows the active
armor and countdown, and the talent screen previews ranks inline.

179 core tests, 351 app tests, core/app Clippy and native build pass. Tests cover
actual deployment, overhealing, refresh, expiry, death and HUD status. Native macOS
menu capture below; live balance and Windows acceptance remain open. Install audit:
222/432 individual talent definitions.

![Ancestral Healing inline armor upgrade](images/screenshots/2026-10-27-ancestral-healing/talents-1280.png)

## 2026-10-27 — Shaman defensive training

Shield Specialization now adds up to 25% Earth Shock absorb strength, combining
with Guardian Totems after the shield milestone unlocks. Parry grants 5% damage
reduction. Both use green inline previews; the shield retains its existing
refresh and stronger-shield protection.

178 core tests, 349 app tests, core/app Clippy and native build pass. Install audit:
221/432 individual talent definitions. Native macOS wide/compact checks verify
preview, Parry purchase and saved rank. Combat balance and Windows acceptance remain open.

![Shield Specialization preview](images/screenshots/2026-10-27-shaman-defense/shield-1280.png)
![Trained Parry in compact layout](images/screenshots/2026-10-27-shaman-defense/parry-640.png)

## 2026-10-27 — Compound talent previews

Permafrost now shows both slow duration and strength changes inline. Arctic Reach
and Holy Reach show their targeting-range and area-radius upgrades separately,
with green values and no repeated summary. Mechanics and area targeting are unchanged.

177 core tests, 349 app tests, core Clippy and native build pass. Native macOS
wide/compact checks verify multi-value wrapping; Windows acceptance remains open.

![Permafrost duration and strength inline](images/screenshots/2026-10-27-compound-inline/permafrost-1280.png)
![Holy Reach's separate range and radius upgrades](images/screenshots/2026-10-27-compound-inline/holy-reach-640.png)

## 2026-10-27 — Pet, range and duration previews

Ten more talents now explain upgrades inline: six wolf talents, Hawk Eye, Flame
Throwing, Entrapment and Improved Shield Block. Green values distinguish the
upgrade from fixed effect numbers; seconds/yards appear once, with correct
second/seconds wording. Mechanics are unchanged.

176 core tests, 349 app tests, core Clippy and native build pass. Native macOS
wide/compact previews verify Frenzy and shield duration; remaining complex
previews and Windows acceptance stay open.

![Frenzy chance highlighted independently](images/screenshots/2026-10-27-pet-inline/hunter-1280.png)
![Compact shield duration upgrade](images/screenshots/2026-10-27-pet-inline/warrior-640.png)

## 2026-10-27 — More upgrades explained inline

Arcane Instability, Force of Will, Spiritual Healing, Improved Renew, Restorative
Totems and Purification now show current → next values inside their descriptions.
Changing values are green, units appear once, and maximum ranks omit the arrow.
The separate “This talent” summary is removed for these six talents.

175 core tests, 349 app tests, core Clippy and native build pass. Native macOS
wide/compact captures verify paired values and healing text wrapping. Mechanics
are unchanged; other complex descriptions and Windows acceptance remain open.

![Paired Mage talent values inline](images/screenshots/2026-10-27-healing-inline/mage-1280.png)
![Compact healing upgrade inline](images/screenshots/2026-10-27-healing-inline/shaman-640.png)

## 2026-10-27 — Guardian Totems

Shaman Guardian Totems now strengthens the unlocked Earth Shock shield by 10% per
rank, up to 20%. Duration stays fixed; repeated casts refresh without stacking,
and stronger existing shields are preserved. Inline green previews show each rank.

174 core tests, 349 app tests, core/app Clippy and native build pass. Install audit:
219/432 individual talent definitions. Native macOS wide/compact checks cover
rank preview and spending; Windows combat acceptance remains open.

![Guardian Totems inline upgrade](images/screenshots/2026-10-27-guardian-totems/talents-1280.png)
![Guardian Totems trained at compact size](images/screenshots/2026-10-27-guardian-totems/talents-640.png)

## 2026-10-27 — Saved Restoration builds verified

The packaged Shaman builds now have save-to-cast regression coverage: stacked
totem cooldowns and milestone healing survive reload and spec switching; reset
stays local to its hero/spec and leaves existing runs intact. Geometry stays
unchanged. 348 app tests with checkout data and Clippy pass. No visual changes.

## 2026-10-27 — Clearer milestone values

Talent milestone descriptions now highlight numeric effects and their units in
green, keeping the surrounding explanation white. Percentages, signed bonuses,
decimal times and multiple effects remain readable inline.

347 app tests with checkout data, Clippy and native build pass. Native macOS
checks cover wide Paladin and compact Hunter details; Windows acceptance remains open.

![Paladin milestone value highlighting](images/screenshots/2026-10-27-milestone-values/paladin-1280.png)
![Compact Hunter milestone values](images/screenshots/2026-10-27-milestone-values/hunter-640.png)

## 2026-10-27 — Paladin milestone artwork

Seven custom Paladin milestones now use themed, verified vanilla spell icons,
completing the nine illustrated milestone rows. Earned icons brighten; locked
icons stay dim. These are art substitutions for Survivors milestones, with no
changes to their mechanics.

346 app tests with checkout data, Clippy and native build pass. Native macOS
wide/compact checks cover all icons, alignment and icon-click inspection. README
captures refreshed; Windows and broader talent acceptance remain open.

![Paladin milestone artwork and earned inspection](images/screenshots/2026-10-27-paladin-milestone-art/talents-1280.png)
![Compact Holy and Protection milestone rows](images/screenshots/2026-10-27-paladin-milestone-art/talents-640.png)
![Compact Retribution milestone rows](images/screenshots/2026-10-27-paladin-milestone-art/retribution-640.png)

## 2026-10-27 — Reproducible talent builds

The `trained` UI fixture now provides two Shaman heroes with Enhancement,
Restoration and empty specs. Matching presets reproduce these screenshots from
one command, without editing saves or playing a run. No gear or clears are granted.

345 app tests pass with checkout data, including actual talent-catalog validation
for every saved spec; five fixture-tool tests pass. Native macOS checks cover both
sizes and switching builds. Other classes and Windows acceptance remain open.

![Packaged Enhancement build](images/screenshots/2026-10-27-trained-fixture/enhancement-1280.png)
![Packaged compact Restoration build](images/screenshots/2026-10-27-trained-fixture/restoration-640.png)

## 2026-10-27 — One-command isolated UI previews

The fixture helper now accepts `--preset` and `--run`: copy a watched UI request,
then launch the incremental build without manually setting environment variables.
Existing save directories remain protected, and inherited autoplay/capture settings
are cleared. The launcher uses direct process arguments for Windows-compatible
paths and reports build/launch failures without deleting the fixture.

Five Python tests pass, including CLI preparation, paths with spaces and launch
environment isolation. Windows execution remains unverified. No rendered UI change;
existing milestone screenshots remain current.

## 2026-10-27 — Restoration deployment training

Improved Healing Wave and Tidal Focus now shorten Searing/Magma deployment
cooldowns by 4% and 2% per rank respectively, multiplying existing training.
Healing still requires the Restoration milestone and occurs once per successful
deployment. Inline green values show the next upgrade.

173 core and 343 app tests, core/app Clippy, data audit and native build pass.
Production cast tests check consumed cooldown, healing and rejected casts; native
wide/compact previews checked on macOS. Balance and Windows acceptance remain open.

![Improved Healing Wave upgrade](images/screenshots/2026-10-27-shaman-cadence/healing-wave-1280.png)
![Compact Tidal Focus upgrade](images/screenshots/2026-10-27-shaman-cadence/tidal-focus-640.png)

## 2026-10-27 — Saved Shaman builds reach run stats

Verified Anticipation and Weapon Mastery through account saves and actual run stat
calculation. Switching specs and resetting removes their bonuses; other heroes do
not inherit them, and an ongoing run retains its original build.

343 app tests pass. No UI change; the Enhancement screenshots below remain current.
Live combat balance and Windows acceptance remain open.

## 2026-10-27 — Shaman Enhancement training

Anticipation now grants 1% damage reduction per rank; Weapon Mastery grants 2%
hero damage per rank, including spells and totems. Both have five ranks and replace
their generic fallback bonuses. Inline descriptions show the actual upgrade.

172 core and 342 app tests, core Clippy, data audit and native build pass. Native
wide/compact previews checked on macOS; live balance and Windows remain open.

![Weapon Mastery inline upgrade](images/screenshots/2026-10-27-shaman-enhancement/weapon-mastery-1280.png)
![Compact Anticipation details](images/screenshots/2026-10-27-shaman-enhancement/anticipation-640.png)

## 2026-10-27 — Raid collection reward boundaries

Documented and verified raid ownership across all nine classes: distinct pieces
advance Armed/Hoarder, while Full set also needs a same-class weapon. Duplicates,
cosmetic choices and save/reload cannot inflate progress or complete legacy
Legendary/54-piece cloak feats. No separate raid completion reward is defined.

342 app tests pass. This adds regression coverage and rules documentation without
changing UI or runtime behavior; existing screenshots remain current. Raid drops
and Windows gameplay acceptance remain pending.

## 2026-10-27 — Direct raid item previews

Live UI previews can open any class's Raid Epic armor directly, with reusable
1280×720 and 640×720 presets. Inspection does not grant or equip items; invalid
weapon and Drop Tables requests are rejected.

340 app tests, app Clippy and native build pass. Native Mage checks cover both
sizes, ordinary Epic switching and invalid-request rejection. Account bytes are
unchanged after closing. Raid acquisition and Windows acceptance remain pending.

![Direct unowned Mage raid preview](images/screenshots/2026-10-27-raid-preview/mage-1280.png)
![Compact raid inspection](images/screenshots/2026-10-27-raid-preview/mage-640.png)

## 2026-10-27 — Raid Collection browsing

Raid Epic now has its own eight-slot Collection row and ownership count, separate
from the legacy 54-piece total. The inspector shows equipped armor count and
active/inactive cumulative 2/4/6-piece bonuses; cosmetic ownership cannot activate
them. Unowned pieces remain inspectable without equip actions.

171 core and 339 app tests, app Clippy and native build pass. Synthetic native
1280×720/640×720 checks cover aligned rows, scrolling, unowned inspection and
six-to-five-to-six equip updates. Raid encounter acquisition, completion rewards,
Windows and live balance remain pending.

![Raid row with active set thresholds](images/screenshots/2026-10-27-raid-browser/collection-1280.png)
![Six-piece threshold becomes inactive after unequipping](images/screenshots/2026-10-27-raid-browser/inactive-1280.png)
![Compact raid Collection browsing](images/screenshots/2026-10-27-raid-browser/collection-640.png)

## 2026-10-27 — Raid armor and set rules

Added 72 distinct Raid Epic armor variants using verified class armor art, with
six-stat budgets and class-specific cumulative 2/4/6-piece bonuses. Only owned,
worn raid armor counts; duplicates and cosmetic overrides add no set power.
Saved loadouts, equipment comparisons and the item inspector resolve the new tier.

171 core and 338 app tests, core/app Clippy and native build pass. Removed an
unused test import exposed by core Clippy. Native synthetic-fixture checks verify
sixth-piece removal/re-equip, saved choices, Epic coloring and wide/compact details.
The legacy collection remains 18/54 in these captures. These items are **not yet
obtainable**: raid encounter sources, dedicated Collection rows, completion rules
and broader class/Windows/balance acceptance remain pending.

![Synthetic six-piece raid armor loadout](images/screenshots/2026-10-27-raid-foundation/character-1280.png)
![Raid tier and cumulative set rules](images/screenshots/2026-10-27-raid-foundation/inspector-1280.png)
![Sixth-piece equip comparison includes the set threshold](images/screenshots/2026-10-27-raid-foundation/set-comparison-1280.png)
![Compact raid item inspector](images/screenshots/2026-10-27-raid-foundation/inspector-640.png)

## 2026-10-27 — Totem cast integration

Totemic Mastery's regression now casts Searing and Magma through production
spell deployment before checking attack reach. It verifies spawned geometry,
damage, lifetime, cooldown and one deployed totem despite extra-target gear;
Searing selects one target and Magma hits every target in its single pulse area.

337 app tests and Clippy pass. This strengthens integration coverage without
changing gameplay or UI; existing screenshots remain current. Windows combat
acceptance and balance remain open.

## 2026-10-27 — Implementation review opened

[Draft PR #1](https://github.com/erjdev/azeroth-survivors-re/pull/1) now tracks the
starter, Armory and progression work, with remaining acceptance gaps called out.
The current browser build check passes with five warnings, and all 337
visibility-enabled app tests pass. This review checkpoint changes no visuals;
existing inline milestone captures remain current. Remote CI results are separate
from these local checks.

## 2026-10-27 — Flurry combat integration

Verified Warrior and Shaman Flurry through production damage resolution: eligible
criticals deal damage and refresh one timer; ticks, dead units, untrained talents
and foreign sources cannot activate it. Subsequent updates cannot replay old hits.
Shaman HUD text shows the active haste/time and disappears on expiry.

337 app tests and Clippy pass. This test-only checkpoint changes no UI or runtime
behavior; the preceding Flurry screenshot remains current. No native rebuild was
needed. Live Windows combat and balance acceptance remain open.

## 2026-10-27 — Shaman Flurry

Shaman Flurry now grants up to 25% haste for three seconds after a direct lightning
or shock critical. Further eligible criticals refresh without stacking; totems,
Lightning Shield and periodic damage cannot trigger it. It uses the existing
cooldown rule, timer and HUD status. The talent explicitly describes this
Survivors conversion and shows its haste upgrade inline.

168 core and 336 app tests, Clippy and native build pass. The local catalog audit
matches 214/432 individual talents. Native macOS validation confirms the final-rank
purchase saves; live combat balance and Windows acceptance remain open.

![Shaman Flurry upgrade and trigger rules](images/screenshots/2026-10-27-shaman-flurry/flurry.png)

## 2026-10-27 — Combined talent upgrades inline

Improved Moonfire now shows both damage and critical-chance upgrades in its
sentence, each with a separate green value span and one unit. Buying a rank
updates both comparisons; maximum rank shows both totals without arrows.
Other unsupported compound descriptions retain their explicit breakdown.

167 core and 336 app tests, Clippy and native build pass. Native macOS checks
at 1280×720 and 640×720 confirm both spans, wrapping and rank-two persistence.
Screenshots also refresh README. Gameplay values and intention-based layout
are unchanged; Windows acceptance remains open.

![Combined Moonfire upgrades](images/screenshots/2026-10-27-moonfire-inline/talents-1280.png)
![Purchased rank at compact size](images/screenshots/2026-10-27-moonfire-inline/talents-640.png)

## 2026-10-27 — Talent value highlighting

Linear talent descriptions now highlight just the values, arrow and unit in green;
explanatory text stays white. Text spans preserve the existing font and size and
wrap together. Switching talents clears old highlights without rebuilding spans
on every unchanged frame. Compound effects and milestones retain their existing
presentation.

167 core and 336 app tests, Clippy and native build pass. Native macOS checks at
1280×720 and 640×720 confirm typography, coloring and wrapping. README previews
are refreshed; the intention-based illustrated layout is retained.

![Wide talent value highlights](images/screenshots/2026-10-27-talent-value-color/talents-1280.png)
![Compact talent value highlights](images/screenshots/2026-10-27-talent-value-color/talents-640.png)

## 2026-10-27 — Inline percentage-point upgrades

Linear talents written in percentage points now show their current and next values
inside the description, with the unit once: `0 » 1 percentage point`, then
`1 » 2 percentage points`. Maximum ranks show only their total. Ambiguous and
compound effects retain the explicit breakdown; mechanics are unchanged.

167 core and 336 app tests, Clippy and native build pass. A native macOS purchase
check confirms both inline forms and the saved rank. The intention-based layout
is unchanged; Windows presentation remains unverified.

![First critical-chance rank](images/screenshots/2026-10-27-percentage-inline/critical-chance-first-rank.png)
![Next critical-chance rank](images/screenshots/2026-10-27-percentage-inline/critical-chance-next-rank.png)

## 2026-10-27 — Shaman survival talents

Improved Ghost Wolf now grants 5% movement speed per rank (two ranks, no form
required). Elemental Warding grants 2 percentage points of damage reduction per
rank (three ranks). Both are explicit Survivors conversions, with inline green
upgrade values and rank caps.

166 core and 336 app tests, Clippy and native build pass. The local catalog audit
matches 213/432 individual talents. Native macOS previews confirm both final-rank
purchases persist; Windows combat and balance remain unverified. The intention's
illustrated three-tree layout is retained.

![Improved Ghost Wolf upgrade](images/screenshots/2026-10-27-shaman-survival/ghost-wolf.png)
![Elemental Warding upgrade](images/screenshots/2026-10-27-shaman-survival/elemental-warding.png)

## 2026-10-27 — Totem healing integration

Verified Restorative Totems and Purification through actual totem deployment:
healing requires the Restoration unlock and a successful cast, happens once
regardless of extra targets, and respects cooldowns, death and maximum health.
Later totem updates do not repeat deployment healing.

336 app tests, Clippy, formatting and diff checks pass. This test-only checkpoint
changes no UI or runtime behavior; no new screenshot or native build is needed.
Windows combat and balance acceptance remain open.

## 2026-10-27 — Unavailable Rune-hound purchase

Rune-hound no longer charges points while rune spawning is unfinished. Its card
shows Unavailable; both direct purchases and staged rewards reject it. Existing
saved instincts remain intact. This safeguard does not complete rune gameplay.

335 app tests, Clippy and native build pass. A funded macOS preview confirms the
disabled control leaves all 35 points and the empty purchase draft unchanged.
The intention-based rewards layout is retained; Windows acceptance remains open.

![Unavailable Rune-hound in funded Rewards](images/screenshots/2026-10-27-rune-availability/companions-1280.png)

## 2026-10-27 — Hero/companion collection integration

Added a regression running the production companion → hero collection chain.
Shared coins pay the sniffer bonus once; already-pulled coins pay the hero normally.
Dead heroes receive neither payout, and a second update cannot repeat the award.

333 app tests and Clippy pass. This adds integration evidence without changing
runtime behavior or UI; no new build or screenshot is required for the test-only
change. Live Windows companion navigation remains open.

## 2026-10-27 — Shared companion pickups pay once

Companions now reserve collected pickups immediately, preventing both pets from
banking the same coin or experience wisp before deferred removal. Gold-sniffer's
bonus applies once; healing pickups remain untouched.

332 app tests, Clippy and native build pass. A real companion-system regression covers both pet
spawn orders, overlapping coins/wisps, the 10 → 11 gold-sniffer payout, and no payout
on a second update. No UI change
or screenshot; live Windows gameplay remains unverified. Rune spawning is still open.

## 2026-10-26 — Browser compilation checkpoint

The current branch passes the PR's wasm build check and all seven web startup/journal
tests. The latter ran with an available bundled Node because the system Homebrew
Node has a missing llhttp library. No system installation was changed.

The wasm check retains five warnings (four unused imports and a Rune lint
expectation). This is compilation/helper-test evidence, not browser rendering,
save recovery, or gameplay acceptance. No visual change or screenshot.

## 2026-10-26 — Inline talent comparisons

Linear talent descriptions now put current » next values inside the effect sentence,
with the unit once. At maximum rank, the sentence shows the trained total. This
removes the repeated per-rank description and separate summary for supported effects;
compound or ambiguous descriptions retain their explicit breakdown.

165 core tests, 331 app tests, Clippy and native build pass. Native macOS checks cover
wide comparison text, purchase/save persistence, and compact maximum-rank text.
The intention-based tree layout is unchanged. Compound formatting and Windows
acceptance remain open.

![Inline talent comparison](images/screenshots/2026-10-26-talent-inline/talents-1280.png)

![Compact maximum-rank description](images/screenshots/2026-10-26-talent-inline/talents-640.png)

## 2026-10-26 — Imported reward accounting

Loaded accounts now cap remaining perk resets at two. A refund subtotal larger
than total spending enters protected recovery instead of risking companion
investment. Valid historical costs remain unchanged.

331 app tests, Clippy and native build pass. Regression checks cover zero/excess
resets, refund conservation, repeated-refund refusal, save round trips and preservation
of an invalid source file. No visual change or new screenshot for this accounting fix.

## 2026-10-26 — Reject ambiguous saved hero identities

Duplicate hero IDs now open the existing recovery flow instead of allowing XP,
talents or appearance updates to address the wrong hero. The original save remains
untouched; automatic ID reassignment would leave ambiguous references unresolved.

330 app tests, Clippy and native build pass. File regressions cover duplicates
within one class and across classes; distinct legacy identities remain supported.
No layout change; the account-recovery screenshots below still show this flow.

## 2026-10-26 — Paladin seal training

Benediction now shortens Judgement's cooldown by 3% per rank. Improved Seal of
Righteousness increases its damage by 3% per rank. Both explicitly describe their
Survivors conversions and multiply existing training without changing other spells,
targets, area or healing.

164 core tests, 330 app tests, Clippy and native build pass. Native macOS purchases
and saved ranks are verified; the existing intention-based talent layout is unchanged.
The catalog audit reports 211/432 individual talents and 81/81 milestones. Live
balance and Windows acceptance remain open.

![Improved Seal of Righteousness](images/screenshots/2026-10-26-paladin-seals/righteousness.png)

![Benediction](images/screenshots/2026-10-26-paladin-seals/benediction.png)

## 2026-10-26 — Preserve accounts after load failures

Unreadable or unparseable accounts now retain their original file, disable saving
and run entry, and show recovery instructions. Missing saves still create new
accounts normally. The recovery notice remains visible across Armory pages.

327 app tests (default features), Clippy with visibility enabled, and the native
build pass. Native macOS checks verify refused entry, wide/compact notices and an
unchanged damaged fixture after shutdown. This prevents overwrite on reported
load errors; it does not repair saves or detect every invalid legacy row.
Windows acceptance remains open. The reference-based layout is unchanged.

![Account recovery notice and refused entry](images/screenshots/2026-10-26-account-recovery/recovery-1280.png)

![Compact Codex recovery notice](images/screenshots/2026-10-26-account-recovery/recovery-640.png)

## 2026-10-26 — Warlock curse, burst and healing training

Amplify Curse strengthens Curse of Agony by 20%; Shadowburn training strengthens
an acquired Shadowburn by 20%. Improved Healthstone adds 1% maximum-health healing
per rank to successful Death Coil casts, stacking with the Demonology milestone.
Descriptions explicitly identify these Survivors adaptations; training does not grant spells.

163 core tests, 328 app tests, Clippy and the native build pass. Native macOS menu
purchases and saved ranks were verified for all three nodes. Coverage is 209/432
individual talents and 81/81 milestones; live balance and Windows acceptance remain open.
The reference-based three-tree layout is unchanged.

![Improved Healthstone training](images/screenshots/2026-10-26-warlock-training/healthstone.png)

![Amplify Curse training](images/screenshots/2026-10-26-warlock-training/amplify-curse.png)

![Shadowburn training](images/screenshots/2026-10-26-warlock-training/shadowburn.png)

## 2026-10-26 — Preserve hero identity after stale save counters

Named and automatic starter creation now skip existing hero IDs, reserve zero,
and wrap safely at the counter limit. Missing or stale save counters can no longer
make a newly created hero share another hero's progression reference.

328 app tests, Clippy and native build pass. Save round-trip checks cover missing,
zero, stale and maximum counters while preserving existing XP and appearances.
This prevents new collisions; it does not repair pre-existing duplicate hero IDs.
No visual change or new screenshot for this persistence correction.

## 2026-10-26 — Direct drop-table previews

UI preview files can now select a named drop-table zone and inspected item together.
This changes the browser only; the next run's map/stage stay intact. Invalid requests
preserve the current view. Two reusable wide/compact presets are included.

327 app tests, Clippy and native build pass. Native macOS checks confirm direct
Duskwood/Poor launch, invalid-zone retention and compact Elwynn/Common reload.
The existing reference-based layout is unchanged; Windows validation remains open.

![Direct Duskwood item preview](images/screenshots/2026-10-26-drop-previews/duskwood-1280.png)

![Compact Elwynn item inspector](images/screenshots/2026-10-26-drop-previews/elwynn-640.png)

## 2026-10-26 — Inspect ordinary-enemy drops

Drop Tables now lists all nine class-specific Poor items in Elwynn and Duskwood,
with icons, ownership and approximate per-item odds. Selecting a row opens its
existing inspector and equipment controls. Scheduled encounters remain first.

326 app tests, Clippy and native build pass. Native macOS checks cover scrolling
at 1280×720 and 640×720, owned/missing item inspection, and equipping gloves with
saved state. The reference's list/detail arrangement is retained; Poor rows extend
its original catalog. Windows/controller acceptance remains open.

![Poor source list with gloves equipped](images/screenshots/2026-10-26-poor-drop-browser/drops-1280.png)

![Compact ordinary-enemy catalog](images/screenshots/2026-10-26-poor-drop-browser/drops-640.png)

## 2026-10-26 — Spell efficiency talents

Hunter Efficiency, Priest Mental Agility and Mage Frost Channeling now shorten
cooldowns for their listed spells, adapting mana-cost talents to automatic combat.
Maximum reductions are 10%, 10% and 15%, multiplied with existing training.
Talent coverage reaches 206/432 individual nodes; 81/81 milestones remain implemented.

162 core tests, 326 app tests, Clippy and native build pass. Native macOS menu
checks confirm all three final-rank purchases and saved ranks at 1280×720.
Live balance and Windows acceptance remain open; the illustrated layout is unchanged.

![Hunter Efficiency at maximum rank](images/screenshots/2026-10-26-cost-talents/hunter-1280.png)

![Priest Mental Agility and its unlocked milestone](images/screenshots/2026-10-26-cost-talents/priest-1280.png)

![Mage Frost Channeling at maximum rank](images/screenshots/2026-10-26-cost-talents/mage-1280.png)

## 2026-10-26 — Hero-specific preview feedback

Switching heroes through UI previews now clears the previous hero's action message.
Same-hero reloads and rejected requests preserve current feedback. Native macOS
checks cover all three paths after buying Totemic Mastery, then switching to Priest.
Clippy, native build, formatting and diff checks pass; no gameplay changed and the
app suite was not rerun for this display-state correction.

![Same-hero reload retains its action message](images/screenshots/2026-10-26-preview-notes/shaman-note.png)

![Switching heroes clears the previous action message](images/screenshots/2026-10-26-preview-notes/priest-cleared.png)

## 2026-10-26 — Continuous talent backgrounds

Talent artwork now scales as one cached image per tree instead of four independent
tiles, removing visible joins at fractional panel sizes. The original painted crop,
aspect ratio, opacity and talent positions remain unchanged.

325 app tests, Clippy and native build pass. A pixel regression covers the complete
tile composition and malformed input. Native macOS comparisons confirm the observed
seams are gone for Priest at 1280×720 and Shaman at 640×720. Other class/resolution
combinations and Windows visual acceptance remain open.

![Continuous Priest backgrounds](images/screenshots/2026-10-26-talent-art/talents-1280.png)

![Continuous compact Shaman backgrounds](images/screenshots/2026-10-26-talent-art/talents-640.png)

## 2026-10-26 — Draggable talent scrolling

Overflowing talent pages now show a fixed gold scrollbar. Dragging reaches the
remaining trees and details on compact screens; fitted wide pages hide the bar.
The reference's illustrated panels and existing hero/spec scroll memory remain.

Native macOS checks cover dragging to the bottom at 640×720, refunding without
losing that view, and the fitted 1280×720 layout. Clippy, incremental native build
(24 seconds), formatting and diff checks pass. No input or talent rules changed;
the app suite was not rerun for this layout-only checkpoint.

![Compact talent details reached by dragging](images/screenshots/2026-10-26-talent-scrollbar/talents-640.png)

![Wide talents with no unnecessary scrollbar](images/screenshots/2026-10-26-talent-scrollbar/talents-1280.png)

## 2026-10-26 — Direct hero UI previews

UI preview files can now select an existing fixture hero with `hero=<ID>`, together
with the page, inspected talent and dimensions. This removes manual class selection
between comparisons. Invalid heroes or mismatched talents preserve the current view;
preview selection does not write the saved menu pick or load another map.

324 app tests, Clippy and the native build pass. Native macOS checks cover direct
Priest launch, rejection of a mismatched Shaman talent without resizing, and a live
switch to Shaman at 640×720. The existing reference-based layouts are unchanged.
Reusable preset and usage instructions are included; Windows acceptance remains open.

![Direct Priest preview at launch](images/screenshots/2026-10-26-hero-preview/priest-1280.png)

![Shaman preview after live class and size switch](images/screenshots/2026-10-26-hero-preview/shaman-640.png)

## 2026-10-26 — Totemic Mastery

Shaman Totemic Mastery extends Searing Totem attack range and Magma Totem pulse
radius by 20%. This authored Survivors conversion preserves damage, healing,
duration and target counts. Searing still selects one enemy; Magma pulses once
around its totem, including with extra-target gear.

161 core and 323 app tests pass, including enemies inside, within the extension,
and beyond reach. Clippy and native build pass. Native macOS menu checks confirm
purchase, saved rank and readable details. Talent coverage is 203/432; live balance
and Windows acceptance remain open.

![Purchased Totemic Mastery and its range description](images/screenshots/2026-10-26-totemic-mastery/talents-1280.png)

## 2026-10-26 — Holy Nova talent

Priest Holy Nova now heals 5% maximum health once per successful cast. It adds to
the Holy 30-point milestone for 10%, before Spiritual Healing. This Survivors
adaptation requires acquiring the spell during the run; damage and lifesteal stay unchanged.

160 core and 322 app tests pass, including empty casts, multiple targets, rank
clamping and healing modifiers. Native macOS menu validation confirms purchase,
saved rank and readable details. Clippy, native build and formatting pass.
Talent coverage is now 202/432; Windows combat and balance validation remain open.

![Purchased Holy Nova talent and healing description](images/screenshots/2026-10-26-holy-nova-talent/talents-1280.png)

## 2026-10-26 — Readable compact Codex

Compact screens stack the jewelry catalog above a full-width details panel instead
of squeezing descriptions into a narrow sidebar. The catalog and details scroll
independently; selected artwork sits beside its description. Wide screens retain
the reference's side-by-side layout. Crossing the layout threshold now rebuilds Codex.

Native macOS 640×720 checks cover legendary/shared jewelry selection and dragging
to the catalog's final rows. Resizing to 1280×720 and back retains selected details.
App Clippy, native build, formatting and diff checks pass. This layout-only change
did not rerun the app suite; Windows/controller acceptance remains open.

![Compact legendary description](images/screenshots/2026-10-26-codex-compact/legendary-640.png)

![Compact shared jewelry after scrolling](images/screenshots/2026-10-26-codex-compact/trinket-640.png)

![Wide Codex after resizing](images/screenshots/2026-10-26-codex-compact/trinket-1280.png)

## 2026-10-26 — Armory keeps your place

Wide Collection, Drop Tables, Character and Codex columns now remember their own
scroll positions across item selection and page rebuilds, independently of compact
page scrolling. Positions clamp when the available content becomes shorter.

322 app regression tests pass, including column rebuild and compact-offset isolation.
App Clippy, native build, formatting and diff checks pass. Native macOS Collection
validation confirms selecting gloves and leaving/returning to the page preserves the
scrolled equipment column. Windows/controller acceptance remains open.

![Selecting gloves preserves the equipment scroll position](images/screenshots/2026-10-26-armory-scroll-memory/selection-1280.png)

## 2026-10-26 — Visible Armory scrollbars

Collection, Drop Tables, Character and Codex now share fixed draggable scrollbars
on overflowing columns. Compact Collection, Drop Tables and Character keep one
page scrollbar with space beside the content; fitted panels hide the bar.

Native macOS menu checks confirm Collection equipment and compact list bottoms,
Drop Tables item details, and Character appearance controls remain reachable by
dragging. Wide Collection/Drop Tables and Codex retain their column layouts.
App Clippy, incremental native build (24 seconds), formatting and diff checks pass.
The input handler is unchanged; the app suite was not rerun for this layout-only
checkpoint. Windows and physical-controller acceptance remain open.

![Wide Collection equipment scrollbar](images/screenshots/2026-10-26-armory-scrollbars/collection-1280.png)

![Compact Collection at the end of the list](images/screenshots/2026-10-26-armory-scrollbars/collection-640.png)

![Compact Drop Tables item details](images/screenshots/2026-10-26-armory-scrollbars/drops-640.png)

## 2026-10-26 — Compact starter scrollbar

Small-window starter layouts now expose the same draggable scrollbar as wide
columns, with reserved space beside the stacked panels. The fixed header and
entry controls remain accessible while browsing all twelve zones.

Native 640×720 checks cover dragging to both ends, last-zone selection, lock
messaging and scroll restoration after switching to 1280×720 and back. Scoped
Clippy, native build and formatting pass. This layout-only change reuses the
previously tested scrollbar handler; the app test suite was not rerun. Windows
acceptance and scrollbar adoption on other Armory pages remain open.

![Compact starter with a visible scrollbar](images/screenshots/2026-10-26-compact-scrollbar/starter-640.png)

![Last zone selected without losing compact scroll position](images/screenshots/2026-10-26-compact-scrollbar/zones-640.png)

## 2026-10-26 — Draggable starter scrollbars

Overflowing wide starter columns now show gold scrollbars inspired by the reference.
Thumbs reflect the visible content fraction, support dragging and track clicks, and
hide when content fits. Short press–move–release gestures use ordered input events.

321 app tests, scoped Clippy, native build and formatting pass. Native 1280×720
and 1100×720 checks cover bidirectional dragging, track clicks, hidden fitted-column
bars and independent hero scrolling. Windows and compact scrollbar acceptance
remain open.

![Zone scrollbar at the late campaign stages](images/screenshots/2026-10-26-starter-scrollbars/zones-1280.png)

![Independent hero and zone scrollbars](images/screenshots/2026-10-26-starter-scrollbars/columns-1100.png)

## 2026-10-26 — Campaign markers and reachable starter columns

Zone banners now show the reference's twelve campaign markers, labeled “Stage X
of 12.” Fixed wide starter columns ignoring the mouse wheel: later zones and
long hero details can now be reached, with independent positions retained through
selection rebuilds. Compact mode keeps its separate scroll position.

319 app tests (including independent scroll memory), scoped Clippy and native build
pass. Native 1280/1100/640×720 checks cover marker clicks, late-zone selection,
lock messaging and independent hero scrolling. Windows acceptance remains open.

![Starter campaign order](images/screenshots/2026-10-26-zone-stages/starter-1280.png)

![Late zones remain visible after selection](images/screenshots/2026-10-26-zone-stages/late-zones-1280.png)

![Independent hero and zone scrolling](images/screenshots/2026-10-26-zone-stages/columns-1100.png)

![Compact campaign markers](images/screenshots/2026-10-26-zone-stages/late-zones-640.png)

## 2026-10-26 — Pre-run zone briefings

All twelve campaign zones now show an authored title and threat briefing instead
of victory text before play. Battlegrounds describe faction-themed survival;
Blackrock remains in preparation. Existing lock reasons and final-fight timing remain.

159 core tests, scoped app Clippy, native build and formatting pass. Native wide
and compact menu checks cover Elwynn copy and Duskwood's lock messaging; this is
macOS presentation validation, not a gameplay or Windows acceptance pass.

![Elwynn pre-run briefing](images/screenshots/2026-10-26-zone-briefing/elwynn-1280.png)

![Locked Duskwood briefing](images/screenshots/2026-10-26-zone-briefing/duskwood-1280.png)

![Compact Duskwood briefing](images/screenshots/2026-10-26-zone-briefing/duskwood-640.png)

## 2026-10-26 — Stable starter and Armory backdrop

Replaced the moving close-up world scenery behind starter/Armory pages with an
original sunset and mountain silhouette shader inspired by the intention reference.
The backdrop scales with the UI; portraits and the legacy Look preview remain visible.

Verified native 1280×720 and 640×720 layouts, zone selection, Character and legacy
Look transitions. Scoped Clippy, native build and formatting pass; no shader errors
were logged. This is macOS menu validation, not Windows or gameplay acceptance.

![Starter sunset backdrop](images/screenshots/2026-10-26-menu-backdrop/starter-1280.png)

![Character portrait over the stable backdrop](images/screenshots/2026-10-26-menu-backdrop/character-1280.png)

![Compact starter backdrop](images/screenshots/2026-10-26-menu-backdrop/starter-640.png)

## 2026-10-26 — Inspect gear while navigating

The Collection detail panel now follows keyboard/controller focus, showing the
focused item's stats, source and ownership before equip. Browsing keeps equipment
and appearance selections separate.

Verified native Poor/Common navigation and owned/unowned details at 1280×720;
the account file stayed byte-for-byte unchanged. All 319 app tests, scoped Clippy
and the native build pass. Physical-controller acceptance remains open.

![Focused uncollected gear shows its own details](images/screenshots/2026-10-26-gear-focus/uncollected-1280.png)

![Focused collected gear shows its own details](images/screenshots/2026-10-26-gear-focus/owned-1280.png)

## 2026-10-26 — Equip gear with keyboard or controller

Backspace/controller west now equips or unequips the focused Collection item.
The grid keeps focus when equipped items also appear in the sidebar. Ownership
checks and independent appearance selections are preserved.

Verified native keyboard equip/unequip and retained focus at 1280×720, plus
compact help at 640×720. All 319 app tests, scoped Clippy and native build pass.
Physical-controller acceptance remains open.

![Keyboard unequip keeps the selected appearance](images/screenshots/2026-10-26-gear-secondary/unequipped-1280.png)

![Keyboard re-equip retains grid focus](images/screenshots/2026-10-26-gear-secondary/equipped-1280.png)

![Compact equipment shortcut help](images/screenshots/2026-10-26-gear-secondary/help-640.png)

## 2026-10-26 — Consistent Collection item details

Poor gear now uses the same bordered item header and section hierarchy as other
rarities. Sources, equip/unequip comparisons and wrapping appearance controls are
aligned with the Collection design reference. Item stats and behavior are unchanged.

Verified at 1280×720 and 640×720; appearance toggles preserve equipped bonuses.
All 317 Survivors app tests, scoped Clippy and the native build pass.

![Unified Poor-item details](images/screenshots/2026-10-26-poor-detail/detail-1280.png)

![Compact item details and appearance actions](images/screenshots/2026-10-26-poor-detail/detail-640.png)


## 2026-10-26 — Direct Poor-gear UI previews

Collection's fast preview now accepts `piece = poor:<slot>` for every class and
visible gear slot. It inspects the actual catalog item without granting, equipping
or saving it. Reusable wide/compact requests make the Poor row quicker to review.
Switching to a legacy rarity clears the previous Poor inspection.

317 data-required app tests pass, including all 81 class/slot combinations,
selection retention and existing preview validation. Clippy, formatting and native
build pass. Native wide/compact checks verified direct inspection, switching to
Epic, rejection of an invalid slot, and an unchanged fixture account file.

![Direct Poor-gear preview](images/screenshots/2026-10-26-poor-preview/poor-1280.png)

![Compact Poor-gear details](images/screenshots/2026-10-26-poor-preview/poor-640.png)


## 2026-10-26 — Bestial Wrath capstone

Training Bestial Wrath upgrades the automatic milestone from +50% bite damage
for ten seconds to +100% for eighteen seconds in each thirty-second cycle. It
replaces the base Wrath bonus while combining with other wolf damage training.
Hunter spells remain unchanged; the capstone grants no immunity.

159 core and 316 data-required app tests pass, including actual bite damage at
the extended window boundaries. Clippy, formatting and native build pass. Talent
coverage is 201/432. Wide and compact menu previews checked; live combat and
Windows acceptance remain open.

![Bestial Wrath capstone](images/screenshots/2026-10-26-pet-wrath/wrath-1280.png)

![Compact Bestial Wrath details](images/screenshots/2026-10-26-pet-wrath/wrath-640.png)


## 2026-10-26 — Intimidation

Intimidation empowers the wolf's nearest bite every 60 seconds, stunning for three
seconds and forcing ordinary enemies to focus the wolf for six. Bosses receive a
0.9-second stun and keep targeting the hero. Extra bite targets are unaffected;
defeat and recovery cannot reset the cooldown.

158 core and 315 data-required app tests pass, including boss stun reduction,
targeting expiry and cooldown persistence. Clippy and formatting pass. Talent
coverage is 200/432. Native build and 1280×720 / 640×720 menu checks pass;
live combat and Windows acceptance remain open.

![Intimidation talent](images/screenshots/2026-10-26-pet-intimidation/intimidation-1280.png)

![Compact Intimidation details](images/screenshots/2026-10-26-pet-intimidation/intimidation-640.png)


## 2026-10-26 — Spirit Bond and faster wolf recovery

Spirit Bond restores 1% / 2% maximum health to Hunter and wolf every ten seconds
while the wolf lives. Improved Revive Pet reduces recovery from ten seconds to
eight / six seconds. Healing caps at full health, and defeat resets the bond timer.

158 core and 312 data-required app tests pass, including actual healing, timing,
overheal limits and recovery. Clippy and formatting pass; talent coverage is
199/432. Native build passed; previews were inspected at 1280×720 and 640×720.
Live combat and balance acceptance remain open.

![Spirit Bond upgrade](images/screenshots/2026-10-26-pet-bond/spirit-bond-1280.png)

![Improved Revive Pet](images/screenshots/2026-10-26-pet-bond/recovery-1280.png)

![Compact Spirit Bond details](images/screenshots/2026-10-26-pet-bond/spirit-bond-640.png)


## 2026-10-26 — Combat wolf health and recovery

The Hunter wolf now takes damage, draws nearby ordinary enemies, and recovers ten
seconds after defeat. Bosses continue targeting the hero. Endurance Training adds
3% maximum wolf health per rank; level growth preserves its health percentage.
The HUD reports wolf health or recovery time.

157 core and 311 data-required app tests pass, covering target switching, shared
attack cooldowns, defeat/recovery, boss targeting and health growth. Talent coverage
is 197/432. Full threat, other pet talents and live combat acceptance remain open.
Clippy, formatting and the native build pass. Menu previews were inspected at
1280×720 and 640×720; the in-map HUD and death animation need live acceptance.

![Endurance Training health upgrade](images/screenshots/2026-10-26-pet-health/endurance-1280.png)

![Compact Endurance Training details](images/screenshots/2026-10-26-pet-health/endurance-640.png)


## 2026-10-26 — Individual Beast Mastery training

Unleashed Fury, Ferocity, Frenzy and Bestial Swiftness now affect the combat wolf.
Bites use global critical chance/damage; trained criticals can refresh an eight-second
haste buff. Pet damage, haste and movement bonuses combine with the automatic
milestones while leaving pickup companions and Hunter spell bonuses separate.

157 core and 307 data-required app tests pass, including real critical damage,
cooldown changes, grounded movement, rank bounds and buff expiry/refresh. Other
pet talents, health/threat and live combat acceptance remain unfinished.

The DBC audit reports 196/432 individual talents. Clippy, native build and formatting
pass. The seeded 30-point spec and its text were inspected at 1280×720 and 640×720.

![Trained Frenzy and next-rank chance](images/screenshots/2026-10-26-beast-training/frenzy-1280.png)

![Ferocity at maximum rank](images/screenshots/2026-10-26-beast-training/ferocity-1280.png)

![Compact Frenzy details after scrolling](images/screenshots/2026-10-26-beast-training/frenzy-640.png)


## 2026-10-26 — Beast Mastery milestones

The Hunter wolf now gains Unleashed Fury (+20% bite damage), automatic Frenzy
(2 → 1.6 sec bite cooldown), and automatic Bestial Wrath (+50% damage for 10 sec
every 30 sec of run time). Bonuses affect the combat wolf only; pauses preserve
the Wrath window. The two damage bonuses multiply.

155 core and 305 data-required app tests pass, including threshold/timer boundaries
and real enemy damage and bite cooldowns. Individual pet talents, pet health/threat,
combat appearance and balance remain unfinished.

Native menu inspection passes at 1280×720 and 640×720; these captures use an empty
saved-spec fixture. Clippy, native build and formatting pass. The final text-only
fix was rebuilt and visually checked after the full test run.

![Unleashed Fury](images/screenshots/2026-10-26-beast-mastery/fury-1280.png)

![Frenzy with readable cooldown units](images/screenshots/2026-10-26-beast-mastery/frenzy-1280.png)

![Bestial Wrath](images/screenshots/2026-10-26-beast-mastery/wrath-1280.png)

![Compact Wrath description after scrolling](images/screenshots/2026-10-26-beast-mastery/wrath-640.png)


## 2026-10-26 — Hunter combat wolf foundation

Hunter runs summon a separate combat wolf with grounded movement, obstacle routing,
melee line-of-sight checks and normal damage/kill/loot credit. Pickup companions
keep their existing role. Initial bites deal 8 damage (+2 per run level after the
first), scaled by global damage, every 2 seconds. Extra-target bonuses add nearby
melee victims without extending reach.

304 data-required Survivors app tests pass, including pet lifecycle, kill/loot
credit, cooldown, extra-target reach, solid-wall rejection and model availability.
App Clippy, the incremental native build and formatting pass.

This foundation does not complete Beast Mastery: pet health/threat, pet talents and
milestones remain open. Live appearance, animation, pathing and balance acceptance
are pending; there is no new visual acceptance screenshot for this checkpoint.


## 2026-10-26 — Poor gear drops in starter zones

Ordinary Elwynn/Duskwood enemies now have an initial 2% chance to award one random
Poor piece. New pieces fill empty slots; duplicates add no power and never
re-equip. Boss/elite loot and chest counts are unchanged. Poor ownership counts
toward piece-count and Full set achievements; the cloak's 54-cell requirement
remains separate. Collection and Drop Tables explain the source and rate.

Verified 154 core and 299 data-required app tests, including direct/periodic kill
awards, exclusions, duplicate handling and save restoration. Native build, Clippy
and menu inspection pass. Screenshots use a seeded menu fixture; live Windows
drop-rate/balance acceptance remains open.

![Poor item acquisition details](images/screenshots/2026-10-26-poor-drops/collection-source.png)

![Starter-zone Poor drops alongside scheduled encounters](images/screenshots/2026-10-26-poor-drops/drop-tables.png)

## 2026-10-26 — Poor rarity collection row

Poor gear now appears in the Collection grid with nine slots, ownership counts,
dim uncollected icons and equipment/appearance markers. Right-click and
Shift-right-click work consistently across tiers. Removed the temporary extra
button list; the existing 54-piece counter now clearly names Common–Legendary.

Verified 296 data-required app tests, native build, Clippy and menu layouts at
1280×720 and 640×720. The seeded preview confirmed unowned inspection and separate
equipment/appearance changes. Natural Poor drops remain unfinished.

![Poor rarity row with combined equipment and appearance marker](images/screenshots/2026-10-26-poor-grid/collection-1280.png)

![Compact Collection grid with Poor tier](images/screenshots/2026-10-26-poor-grid/collection-640.png)

## 2026-10-26 — Poor gear equipment and appearance

Owned Poor pieces now support inspection, equip/unequip and independent appearance
choices. Their saved selections feed runtime bonuses, equipment icons and the
player appearance resolver; existing collection progress stays intact.
Natural drops and raid gear/set builds remain unfinished.

Verified: 153 core tests, 295 data-required app tests, native build and Clippy.
Menu checks at 1280×720 and 640×720 confirmed equip/unequip and hidden-look state.
These screenshots use a seeded menu fixture; in-map and Windows appearance checks
remain open.

![Poor gloves: equip preview at 1280×720](images/screenshots/2026-10-26-poor-loadout/collection-1280.png)

![Poor gloves: compact inspector with independent hidden appearance](images/screenshots/2026-10-26-poor-loadout/collection-640.png)

## 2026-10-26 — Unified gear resolution rules

Added a shared resolver for legacy and supplemental worn pieces and appearances.
A supplemental piece replaces its slot's power; cosmetic overrides remain separate.
Ownership, class and slot are checked on selection and resolution, and mixed full
loadouts receive the existing nine-slot set bonus exactly once.

This is a rules-layer checkpoint. Live account equipment, saves, model rendering
and UI still need to adopt the resolver; no new equip action is enabled yet.
All 153 core tests, native build, core Clippy, formatting and diff checks pass.

## 2026-10-26 — Supplemental gear ownership saves

Added separate, stable-key ownership records for Poor gear. Duplicate records do
not stack; unknown keys and class mismatches are rejected. Old saves default to
empty supplemental ownership, and existing gear, stats and collection feats retain
their original behavior.

This prepares persistence only. Drops, equip/appearance actions and collection UI
remain unfinished; supplemental ownership alone grants no power.
All 150 core and 293 app tests, native build, Clippy, formatting and diff checks pass.

## 2026-10-26 — Poor gear identities

Added all 81 Poor class/slot identities from the pinned vanilla catalog: four armor
families and nine class-appropriate weapons. Stable named keys keep these separate
from the existing 54-piece collection. The gear audit now includes their icons and
display rows.

This is catalog groundwork. Ownership, acquisition and equipment/UI integration
remain pending; no new obtainable gear or rendered appearance is claimed.
All 149 core and 292 data-required app tests pass, including the expanded icon audit;
native build, Clippy, formatting and diff checks pass.

## 2026-10-26 — Explicit gear progression tiers

Added separate Poor and Raid Epic rules without changing existing collection/save
indexes. Poor profiles carry one stat; Raid Epic profiles carry six, strengthening
ordinary Epic's five. Existing gear values remain unchanged.

This is the rules foundation. New item catalogs, acquisition, save extensions and
raid set builds remain unfinished; no new obtainable gear or UI is claimed.
All 148 core tests, native build, core Clippy, formatting and diff checks pass.

## 2026-10-26 — Preserve talents in reordered saves

Talent rows now load even when they precede their character row. All three specs
retain their ranks and run effects; unknown hero references are ignored and later
talent rows still replace earlier ones. Malformed hero IDs no longer target hero 0.

All 292 Survivors app tests, native build, Clippy, formatting and diff checks pass.
This is a save-loading correction with no interface change or new visual claim.

## 2026-10-26 — Consistent Druid healing bonuses

Improved Rejuvenation, Gift of Nature and Nature's Focus now scale Rejuvenation's
immediate Swiftmend/milestone heal as well as its periodic healing. Previously the
immediate portion missed these bonuses. Full healing ranks raise a combined 10%
base pulse to 13.915% maximum health; healing over time is not consumed.

All 146 core and 291 Survivors app tests pass, along with native build and Clippy.
This is a rules correction, with no layout change or new visual acceptance claim.
Live balance and Windows gameplay acceptance remain open.

## 2026-10-26 — Shaman totem healing talents

Restorative Totems and Purification now strengthen the unlocked totem-deployment
heal instead of granting generic damage. Their bonuses add: maximum ranks raise
healing from 3% to 4.05% maximum health. Damage, shields and regeneration stay outside
this healing scope.

![Purification purchased](images/screenshots/2026-10-26-totem-healing/talent-1280.png)

![Compact Restorative Totems details](images/screenshots/2026-10-26-totem-healing/talent-640.png)

Native purchase/refund persistence and wide/compact descriptions verified. All 145
core and 291 Survivors app tests pass, alongside native build and Clippy. Definition
coverage reaches 192/432; gameplay balance and Windows acceptance remain open.

## 2026-10-26 — Appearance choice indicators

Character customization now includes the intention design's position dots below
each counter. The selected dot updates in place when cycling choices, preserving
scroll position. Indicators scale with the interface typography.

![Character choice indicators](images/screenshots/2026-10-26-feature-dots/character-1280.png)

![Compact feature controls](images/screenshots/2026-10-26-feature-dots/character-640.png)

Native 1280×720 and 640×720 checks confirm selection changes and wraparound for
Night Elf hair styles. Native build, app Clippy, formatting and diff checks pass.
Windows visual acceptance remains open.

## 2026-10-26 — Hunter appearance verification

Verified Dwarf and Night Elf, male and female, on the native Character page at
1280×720. Body, identity and customization ranges follow each selection; the final
appearance saves without changing equipment or talent ranks. The small starter
portrait's suspected Human/Dwarf mismatch was not a model bug.

![Female Dwarf Hunter](images/screenshots/2026-10-26-hunter-appearance/dwarf-female.png)

![Male Dwarf Hunter](images/screenshots/2026-10-26-hunter-appearance/dwarf-male.png)

![Male Night Elf Hunter](images/screenshots/2026-10-26-hunter-appearance/night-elf-male.png)

![Female Night Elf Hunter](images/screenshots/2026-10-26-hunter-appearance/night-elf-female.png)

Visual verification only; no runtime code changed. Equipped armor/weapons, other
race/class combinations and Windows acceptance remain open.


## 2026-10-26 — Hunter Survivalist and full-health starts

Survivalist now grants 2% maximum health per rank, up to 10%, scaling equipped
health and composing with passive, renown and racial bonuses. The starter shows
its percentage. All entry paths fill health after saved talents are applied;
recalculating stats during a run does not heal the player.

![Survivalist purchase](images/screenshots/2026-10-26-survivalist/talent-1280.png)

![Compact Survivalist details](images/screenshots/2026-10-26-survivalist/talent-640.png)

![Starter talent health summary](images/screenshots/2026-10-26-survivalist/starter-1280.png)

144 core and 291 Survivors app tests, native build, app Clippy and catalog audit
pass. Native checks cover purchase, persistence, wide/compact descriptions and the
starter bonus. Live combat and Windows acceptance remain open.


## 2026-10-26 — Priest healing talent

Spiritual Healing now increases Renew and Holy Nova's unlocked healing by 2% per
rank, up to 10%, replacing its generic damage placeholder. It multiplies Improved
Renew and does not grant spells or increase damage, shields or Shadow lifesteal.

![Spiritual Healing purchase and next rank](images/screenshots/2026-10-26-spiritual-healing/talent-1280.png)

![Compact details after refund](images/screenshots/2026-10-26-spiritual-healing/talent-640.png)

Native checks cover purchase, saved rank, refund and readable details at 1280×720
and 640×720. All 143 core and 290 Survivors app tests pass, along with the native
build, app Clippy and formatting. Catalog coverage is 189/432 individual talents;
live combat balance and Windows acceptance remain open.


## 2026-10-26 — Retained foliage visibility coverage

Added a regression check for collider-free foliage in the retained renderer through
the camera reveal's public world query. It covers selected/culled scenery, clear
rays and disabling the retained renderer; CI now runs it. This is synthetic query
coverage, not a new visual change or confirmation of the reported Windows scene.
The new test and two adjacent retained-picking tests pass, as do formatting and
diff checks. No new screenshot is attached to this test-only checkpoint.


## 2026-10-26 — Page-by-page screenshot gallery

The README now groups 20 representative captures by interface page, with capture
dates and explicit limits for older gameplay evidence. Earlier README milestones
and all 187 referenced screenshots are preserved in [screenshot history](SCREENSHOT-HISTORY.md).
Every existing inline changelog image remains in place. Local image/document links
were checked; this documentation-only checkpoint adds no new runtime screenshots.


## 2026-10-26 — Character identity follows appearance

Character now shows gender, race and a class-colored class name beneath the hero's
name, matching the intention layout. The line follows appearance changes and saved
hero selection, with consistent inline typography.

![Human Warrior identity](images/screenshots/2026-10-26-character-identity/human-1280.png)

![Changed Tauren appearance at 640×480](images/screenshots/2026-10-26-character-identity/tauren-640.png)

![Switching to a saved Paladin](images/screenshots/2026-10-26-character-identity/paladin-640.png)

Native checks cover male/female Human, female Tauren and switching to a saved
Paladin at wide/minimum sizes. Native build and app Clippy pass. This does not
complete the full race/model matrix or Windows DPI acceptance.


## 2026-10-26 — Character portrait on short windows

The compact Character portrait now scales within both the column width and window
height. At 640×480 the full figure and rotation buttons fit without scrolling past
an oversized portrait. Appearance and equipment remain available below.

![Before: rotation controls below the viewport](images/screenshots/2026-10-26-character-short-viewport/before-640x480.png)

![After: full figure and rotation controls at 640×480](images/screenshots/2026-10-26-character-short-viewport/character-640x480.png)

![Character at 1024×768](images/screenshots/2026-10-26-character-short-viewport/character-1024x768.png)

![Character at 640×720](images/screenshots/2026-10-26-character-short-viewport/character-640x720.png)

Native checks cover these three sizes, both turn buttons and scrolling to appearance
and equipment. Native build and app Clippy pass. Other races and Windows DPI remain
open; this layout-only checkpoint does not change game rules.


## 2026-10-26 — Druid Restoration talents

Nature's Swiftness now shortens Rejuvenation's cooldown by 15%. Swiftmend adds
an immediate 5% maximum-health heal per successful Rejuvenation cast, combining
with its tree milestone for 10% without consuming healing over time.

![Trained Nature's Swiftness](images/screenshots/2026-10-26-druid-capstones/swiftness-1280.png)

![Trained Swiftmend](images/screenshots/2026-10-26-druid-capstones/swiftmend-1280.png)

![Compact Restoration details](images/screenshots/2026-10-26-druid-capstones/swiftmend-640.png)

142 core and 288 app tests pass; native build and app Clippy pass. Native purchases
persisted both ranks in spec 3. Catalog coverage: 188/432 individual talents;
live combat balance and the remaining catalog are still open.


## 2026-10-26 — Direct Collection previews

Live UI preview files can select gear by rarity and slot (`piece = epic:hands`),
so item details and equipment comparisons can be inspected without a gameplay run.
Invalid edits preserve the previous view; preview selection does not equip items.

![Wide Epic Hands preview](images/screenshots/2026-10-26-piece-preview/epic-hands-1280.png)

![Compact item details after scrolling](images/screenshots/2026-10-26-piece-preview/epic-hands-640.png)

287 visibility-enabled app tests, native build and app Clippy pass. Native previews
verified valid/invalid live edits and compact scrolling; the saved account hash
remained unchanged through shutdown. Windows gameplay acceptance remains open.

## 2026-10-26 — Visibility regression gate

The PR workflow now runs Survivors rules and visibility-enabled integration tests.
New checks verify that collider-free ordinary/merged scenery drives the 500 ms
reveal fade, releases it behind the hero, and clears it in menus or when disabled.
Removed the unused collision-only reveal helper. All 285 app and 141 core tests
pass locally. This changes verification, not rendering; the exact Windows foliage
scene remains open, and there is no new visual milestone screenshot.

## 2026-10-26 — Nightfall procs

Warlock Nightfall now gives Corruption ticks a 2%/4% chance to ready an owned
Shadow Bolt. It rolls once per damaging tick, ignores dead-target overkill and
keeps the separate Affliction milestone's cooldown reduction.

![Nightfall rank-one upgrade preview](images/screenshots/2026-10-26-nightfall/talents-1280.png)

![Maximum Nightfall rank at compact width](images/screenshots/2026-10-26-nightfall/talents-640.png)

Native checks verify both purchases, saved ranks and wide/compact readability.
141 core and 283 app tests, native build and workspace Clippy pass. Definition
coverage is 186/432; remaining talents and live balance acceptance stay open.

## 2026-10-26 — Trainable Siphon Life

Warlock Siphon Life now heals 5% of Corruption/Curse of Agony damage actually dealt,
adding to the Affliction milestone for 10% total. Overkill and dead targets add no
healing. The talent description explains this automatic Survivors conversion.

![Siphon Life trained and saved](images/screenshots/2026-10-26-siphon-life/talents-1280.png)

![Compact Siphon Life details](images/screenshots/2026-10-26-siphon-life/talents-640.png)

Native checks verify spending, persistence and wide/compact readability. 140 core
and 281 app tests plus the native build pass. Definition coverage is 185/432;
remaining talents and live balance acceptance stay open.

## 2026-10-26 — Browser build verification

The WebGPU WebAssembly compile gate now passes locally. Browser startup tests now
model the loading screen's CSS properties, child elements and animation timers;
all seven startup/journal tests pass. This changes test infrastructure only, so
there is no new interface screenshot. Browser runtime and Windows gameplay
acceptance remain open.

## 2026-10-26 — Keep Armory keyboard/controller focus

Character, Collection and other Armory actions now retain focus through page
rebuilds, extending the existing Rewards behavior. Repeated appearance changes no
longer require navigating back to the same control. Page, hero and category changes
clear focus, and restoration never repeats an action.

![Skin control stays focused after repeated changes](images/screenshots/2026-10-26-armory-focus/character-1280.png)

Native keyboard checks cover skin changes 2→1→10, wide-to-compact resizing and a
compact hair-style change. The saved fixture reflects those changes. The reference
Character layout is preserved; the white outline provides navigation feedback.
282 app tests, native build, workspace Clippy, formatting and render lint pass.
Physical-controller acceptance remains open.

## 2026-10-26 — Fast passive card previews

Live UI previews now accept passive acquisitions and upgrades, including selected
hero talents. Wide/compact presets show Renew's actual trained regeneration without
a run. Invalid names/ranks or mixed card sources preserve the current preview.

![Trained Renew acquisition and upgrades](images/screenshots/2026-10-26-passive-preview/renew-1280.png)

![Final Renew upgrade reached by compact scrolling](images/screenshots/2026-10-26-passive-preview/renew-640.png)

Native checks cover selected-profile values, invalid-request retention, compact
scrolling and returning to Talents. The fixture account remains byte-for-byte unchanged.
The cards retain the intention's bordered layout with the requested inline green
values. All 12 preview tests, native build, workspace Clippy, formatting and render
lint pass. This is a development preview; picks remain disabled.

## 2026-10-26 — Improved Renew and accurate regeneration previews

Priest Improved Renew now increases acquired Renew regeneration by 5% per rank,
up to 15%, replacing its temporary damage bonus. Other healing stays independent.
Upgrade cards and passive tooltips use the same trained values as the stat sheet;
small gains retain enough precision to remain visible (0.525 health/sec at rank 1).

![Improved Renew after spending a point](images/screenshots/2026-10-26-renew-talent/talents-1280.png)

![Compact Improved Renew details](images/screenshots/2026-10-26-renew-talent/talents-640.png)

Native wide/compact Talents checks verify spending, saved rank 2 and readable details.
The tree layout follows the intention reference; the description explicitly explains
the passive conversion. Card values are covered by regression tests, not a live
combat capture. 139 core and 279 app tests, native build, workspace Clippy, formatting
and render lint pass. Talent definition coverage is 184/432; remaining talents and
gameplay balance stay open.

## 2026-10-26 — Visible cloak progression

Character and Collection now show cloak quality, active bonuses, the four unlock
requirements, and progress toward the account legendary cloak. Talent-cap credit
cannot come from a hero of another class. Also fixed development-only talent preview
references leaking into non-development builds.

![Cloak progress beside equipment](images/screenshots/2026-10-26-cloak-progress/character-1280.png)

![Compact cloak progress](images/screenshots/2026-10-26-cloak-progress/character-640.png)

Native seeded previews checked at 1280×720 and 640×720. 277 app tests, native play
build, non-development library check and workspace Clippy pass. The non-development
check has existing feature-specific warnings; wasm is not checked (target absent).
This exposes existing cloak rules; it adds no cosmetic cloak model.

## 2026-10-26 — Cloaks require a completed collection

Fixed the cloak collection feat granting credit for just nine occupied slots. It
now requires all 54 pieces for one class. Full set still means nine slots; existing
achievement points remain earned, while cloak power follows the corrected rule.

![Cloak locked at 53 pieces](images/screenshots/2026-10-26-collection-cloak/53-pieces.png)

![Cloak earned at 54 pieces](images/screenshots/2026-10-26-collection-cloak/54-pieces.png)

Native fixture reloads verify the boundary. The final piece also earns the separate
Legendary achievement, so the screenshot total rises by 35 points (10 + 25).
138 core and 276 app tests, native build, workspace Clippy, formatting and render
lint pass. Poor and raid-Epic catalog expansion remain open.

## 2026-10-26 — Direct talent inspection in live previews

The development preview accepts a talent ID to pin its real description and saved
rank without spending points. It validates against the selected hero's client-data
tree; invalid requests preserve the current page and viewport. Removing the selector
restores ordinary inspection. A reusable Grim Reach preset is included.

![Direct Grim Reach preview](images/screenshots/2026-10-26-talent-preview/grim-reach-1024.png)

![Live compact talent selection](images/screenshots/2026-10-26-talent-preview/destructive-reach-640.png)

46 Armory tests pass. Native checks cover direct launch, live changes, cross-class
rejection and clearing the selector; fixture files remain byte-for-byte unchanged.
Native build, workspace Clippy, formatting and render lint pass. This improves the
UI iteration workflow; full talent content and platform acceptance remain open.

## 2026-10-26 — Talent icons scale with their artwork

Fixed crowded talent rows at 1024×768: icons now scale with tree width, capped at
their existing size. Rank labels remain readable and prerequisite lines keep their
centers. Added compact, 4:3 and wide live-preview presets.

Before (fixed-size icons crowded rank labels):

![Before at 1024×768](images/screenshots/2026-10-26-talent-responsive/before-1024.png)

After:

![Proportional talent icons at 1024×768](images/screenshots/2026-10-26-talent-responsive/warlock-1024.png)

![Compact talent description](images/screenshots/2026-10-26-talent-responsive/warlock-640.png)

![Wide talent layout](images/screenshots/2026-10-26-talent-responsive/warlock-1280.png)

Native macOS checks cover all three sizes, long descriptions, lower-panel scrolling,
spend/refund and independent spec state. Native build, workspace Clippy, formatting
and render lint pass. Other classes, physical controllers and Windows visual
acceptance remain open; this checkpoint changes layout only.

## 2026-10-26 — Targeting-range talents

Hawk Eye, Storm Reach, Grim Reach, Destructive Reach and Nature’s Reach now extend
their named spells’ targeting distance. Area size, Chain Lightning bounce distance
and target counts stay unchanged. Talent previews show the green range benefit.

![Hawk Eye preview](images/screenshots/2026-10-26-reach-talents/hawk-eye-1280.png)

![Destructive Reach preview](images/screenshots/2026-10-26-reach-talents/destructive-reach-1280.png)

137 core and 273 app tests pass; native build, workspace Clippy, formatting and
render lint pass. The data audit verifies 183/432 talent definitions. Native 1280×720 checks cover
Hunter and Warlock descriptions and prerequisite refusal. Live combat range checks
and the full responsive matrix remain open.

## 2026-10-26 — Paladin Retribution damage talents

Sanctity Aura now boosts Paladin Holy attacks by 10%. Vengeance grants 3% per rank
for eight seconds after a direct Paladin Holy critical, refreshing without stacking.
The active buff appears in combat status; other classes' spells stay unaffected.

![Vengeance talent preview](images/screenshots/2026-10-26-paladin-retribution/vengeance-1280.png)

![Sanctity Aura talent preview](images/screenshots/2026-10-26-paladin-retribution/sanctity-1280.png)

136 core and 272 app tests pass; the data audit verifies 178/432 talent definitions.
Native build, workspace Clippy, formatting and render lint pass.
Native 1280×720 checks cover descriptions and locked-node prerequisites. Combat
balance, purchased-node gameplay and the full responsive matrix remain open.

## 2026-10-26 — Shared jewelry artwork completed

Mark of the Champion now has its item artwork in the Codex and chest cards.
All 39 shared jewelry entries have verified mappings. The chosen vanilla variant
changes presentation only; stats and discoveries are unchanged.

![Completed Codex jewelry art](images/screenshots/2026-10-26-champion-icon/codex-1280.png)

![Compact chest card](images/screenshots/2026-10-26-champion-icon/cards-640.png)

271 app tests with checkout data, native build, workspace Clippy, formatting and
render lint pass. Native Codex and compact card checks were performed on macOS.

## 2026-10-26 — A useful Divine Shield legendary

Divine Shield's amulet now grants 15% spell-local cooldown reduction, replacing
bonuses immunity could not use. Cards and Codex show the same green benefit;
duplicates do not stack and replacement removes it.

![Divine Shield legendary offer](images/screenshots/2026-10-26-divine-legend/cards-1280.png)

![Compact legendary offer](images/screenshots/2026-10-26-divine-legend/cards-640.png)

![Divine Shield Codex detail](images/screenshots/2026-10-26-divine-legend/codex-1280.png)

Native wide/compact previews and Codex selection checked on macOS. Combat balance
and the remaining spell-specific legendary designs are still open.
135 core and 271 app tests, native build, workspace Clippy, formatting and render lint pass.

## 2026-10-26 — Windows playtest corrections

- Foliage reveal checks rendered scenery instead of collision hulls, with the existing 500 ms fade.
- Spawns reject trunk interiors and unverified fence connections; more enemies receive route work each frame.
- Elite chests: first after 60 seconds, then 90 seconds apart, maximum five per run. Boss rewards remain separate. Corrected elite/boss models that incorrectly referenced a sarcophagus and grave marker.
- Talent backgrounds preserve the painted art's proportions and crop transparent padding; icons stay aligned.

![Talent backgrounds and aligned rows](images/screenshots/2026-10-26-playtest-fixes/talents-1280.png)

![Compact talent layout](images/screenshots/2026-10-26-playtest-fixes/talents-640.png)

![Corrected chest models in the preview world](images/screenshots/2026-10-26-playtest-fixes/chests-native.png)

135 core tests and 270 app tests pass, including authored tree/chest collision
checks and foliage without colliders. Native build, workspace Clippy, formatting
and render lint pass.

Native UI and chest models checked on macOS. The model capture uses the preview
world; it is not an Elwynn routing replay. Exact Windows foliage/fence/chest scenarios still need
retesting; these fixes do not constitute a Windows gameplay acceptance pass.

## 2026-10-26 — Fast jewelry previews and clearer legendary cards

Chest offers can now be inspected from a watched preview file, with reusable wide
and compact presets. Legendary cards show the actual spell name and icon even
without that spell in the current loadout.

![Jewelry and legendary cards](images/screenshots/2026-10-26-jewelry-preview/cards-1280.png)

![Compact scrollable jewelry cards](images/screenshots/2026-10-26-jewelry-preview/cards-640.png)

266 app tests, native build, workspace Clippy, formatting and render lint pass.
Native checks verify responsive wrapping, second-row scrolling,
disabled reward picks and return to the undiscovered Codex. No balance or save changes.

## 2026-10-26 — Clearer Codex spell details

Legendary entries now describe the actual rank-one spell effect and recharge time.
Healing and shields no longer appear as generic damage. Codex and legendary cards
label healing/absorb bonuses correctly and omit critical bonuses those spells cannot
use; replacement losses stay red.

![Rejuvenation healing detail](images/screenshots/2026-10-26-codex-prose/healing-1280.png)

![Compact absorb detail](images/screenshots/2026-10-26-codex-prose/shield-640.png)

265 app tests, native build, workspace Clippy, formatting and render lint pass.
Native wide/compact checks verify readable details with a synthetic account. This
corrects presentation; authored spell-specific legendary effects and balance remain
unfinished, including an effective benefit for Divine Shield's generic amulet.

## 2026-10-26 — Hunter Survival milestones

Survival now improves poison/trap damage at 10 points, grants a Concussive Shot
absorb at 20, and extends Serpent Sting control at 30. These provisional Survivors
effects compose with individual training and have explicit green previews.

![Monster Slaying preview](images/screenshots/2026-10-26-hunter-survival/slaying-1280.png)

![Surefooted preview](images/screenshots/2026-10-26-hunter-survival/surefooted-1280.png)

![Wyvern Sting preview](images/screenshots/2026-10-26-hunter-survival/wyvern-1280.png)

135 core and 263 app tests pass, covering thresholds, damage parameters, boss-scaled
stuns and non-stacking absorbs. Native empty-spec previews verify descriptions.
Build, workspace Clippy, DBC audit, formatting and render lint pass. Milestones:
78/81; Beast Mastery awaits combat pets. Live combat balance remains open.

## 2026-10-26 — Saved Companion unlocks

Loading a save now checks both Companion looks against the completed achievement
list. Unavailable or unknown looks fall back to Snowshoe Rabbit; earned Veteran
and legacy Chicken access are preserved. This closes a path that could show a
locked appearance as worn after loading.

![Unavailable saved looks repaired](images/screenshots/2026-10-26-companion-save/restored-1280.png)

![Independent looks retained after restart](images/screenshots/2026-10-26-companion-save/restarted-1280.png)

262 app tests pass, including both slots, free/locked/unknown looks, row ordering
and legacy access. A synthetic native save verifies repair, second-slot selection,
saving and restart persistence. Native build, workspace Clippy, formatting and
render lint pass; no in-world appearance validation is claimed.

## 2026-10-26 — Companion appearance controls

Companion looks now share the gold hover outline and burgundy selection treatment
used by Tabards. Locked looks keep their achievement labels but no longer expose
click or keyboard/controller action targets.

![Selected Companion appearance](images/screenshots/2026-10-26-companion-controls/looks-1280.png)

![Compact Companion appearances](images/screenshots/2026-10-26-companion-controls/looks-640.png)

Native 1280×720 and 640×720 checks verify icon clicks, selection changes, locked
Panda Cub rejection and readable scrolling with a synthetic account. Native build,
workspace Clippy, formatting and render lint pass. Physical controller acceptance
and in-world companion appearance remain separate validation work.

## 2026-10-26 — Druid Restoration training

Five more talents replace generic damage with explicit armor, damage reduction,
Rejuvenation healing and cooldown effects. Green rank previews show the benefit;
these are provisional Survivors adaptations of the original talents.

![Improved Mark armor preview](images/screenshots/2026-10-26-restoration-nodes/mark-1280.png)

![Healing Touch cooldown preview](images/screenshots/2026-10-26-restoration-nodes/healing-touch-1280.png)

![Nature's Focus healing preview](images/screenshots/2026-10-26-restoration-nodes/natures-focus-1280.png)

134 core and 261 app tests pass, covering rank caps, spell scope, resets and milestone
composition. Native previews verify spending, row unlocks and refund/reallocation
with a synthetic account. Build, workspace Clippy, formatting and render lint pass.
DBC audit: 176/432 individual nodes; live combat and balance acceptance remain open.

## 2026-10-26 — Consistent achievement cards

Achievement cards retain their proportional column width on incomplete rows instead
of stretching the last card across the panel. This follows the intention’s two-column
alignment and applies to category, recent and almost-complete cards.

![Consistent Feats card widths](images/screenshots/2026-10-26-achievement-widths/feats-1280.png)

![Compact Feats layout](images/screenshots/2026-10-26-achievement-widths/feats-640.png)

Native 1280×720 and 640×720 previews verify the five-card Feats layout, readable
content and scroll reachability using a synthetic account. Native build, workspace
Clippy, formatting and render lint pass. No mechanics changed.

## 2026-10-26 — Direct category previews

The fast UI preview now opens Rewards and Achievement categories directly using
`category = ...`. Reusable Feats and compact Tabards requests are included under
`docs/ui-previews/`; changing the watched file updates page, category and size
without restarting or entering a run. Invalid combinations preserve the current view.

![Direct launch into Feats](images/screenshots/2026-10-26-preview-categories/feats-1280.png)

![Live switch to compact Tabards](images/screenshots/2026-10-26-preview-categories/tabards-640.png)

261 app tests, native build, workspace Clippy, formatting and render lint pass.
Native checks cover direct launch, rejected category/size changes and live cross-page
reload with a synthetic account. This extends development controls; player layout
and progression rules are unchanged.

## 2026-10-26 — Rapid menu selection

Fixed a dropped selection when a direction and Enter/Space (or controller South)
arrive in the same frame. Navigation now runs first, then selects the destination.
The first combined gesture only establishes focus and cannot accidentally start a run.

![Rapid Left then Enter selects None](images/screenshots/2026-10-26-coalesced-input/selected-1280.png)

Native rapid Right+Space and Left+Enter checks pass on the isolated preview account.
260 app tests cover keyboard/controller coalescing, first-focus protection, one-time
activation and modal blocking. Native build, workspace Clippy, formatting and render
lint pass. Physical controller acceptance remains open.

## 2026-10-26 — Tabard selection feedback

Tabard cards now share the Armory’s hover/pressed feedback, gold outlines and
burgundy selected state. The whole card remains clickable; keyboard focus stays
visible after selecting a look. Tabards remain free account cosmetics with no stats.

![Guild Tabard selected](images/screenshots/2026-10-26-tabard-controls/worn-1280.png)

![Compact tabard keyboard focus](images/screenshots/2026-10-26-tabard-controls/focus-640.png)

Native 1280×720 and 640×720 checks cover icon selection, removal, scrolling,
keyboard selection and the saved choice after restart, using a synthetic account.
Build, workspace Clippy, formatting and render lint pass. The Rewards reference’s
shared framing guides this page; there is no dedicated Tabards reference image.
The catalog still has only the verified Guild Tabard; in-world rendering was not retested.

## 2026-10-26 — Legendary class collection

The reference’s Legendary achievement now grants 25 points for collecting all
nine legendary gear pieces of one class. Pieces split across classes and duplicate
drops do not advance it. Duplicate achievement IDs in imported saves no longer
inflate points or earned counts.

![Eight pieces remain incomplete](images/screenshots/2026-10-26-legendary-achievement/progress-1280.png)

![Completed class collection earns 25 points](images/screenshots/2026-10-26-legendary-achievement/earned-1280.png)

133 core and 259 app tests, native build, workspace Clippy, achievement artwork
audit, formatting and render lint pass. Native screenshots use a synthetic account;
tests cover rarity, class boundaries, duplicate drops, save reload and one-time awards.

## 2026-10-26 — Druid milestone builds

All nine Druid milestones now have explicit Survivors adaptations. Balance adds
root cooldown support, Moonfire damage and spell critical chance. Feral supports
Thorns with cooldown, absorb and healing benefits. Restoration strengthens
Rejuvenation and adds Swiftmend’s immediate heal without consuming its ongoing heal.
Forms and feral attacks remain unfinished; tuning is provisional.

![Swiftmend milestone](images/screenshots/2026-10-26-druid-milestones/swiftmend-1280.png)

![Heart of the Wild milestone](images/screenshots/2026-10-26-druid-milestones/heart-of-the-wild-1280.png)

132 core and 258 app tests, native build, workspace Clippy, talent data audit,
formatting and render lint pass. Tests cover threshold boundaries, individual-rank
composition, cast parameters, capped healing, no revival and absorb non-stacking.
Native previews use an untrained synthetic character; live combat/balance acceptance
remains open. Catalog coverage is 171/432 nodes and 75/81 milestone descriptions.

## 2026-10-26 — Jeweler’s Eye follows the Codex

Jeweler’s Eye now counts discovered rings, trinkets and amulets across the account,
including class legendary amulets, as the reference intends. It requires half the
current catalog, rounded up (56 of 112), and grants 10 points once. Existing awards
remain earned. The separate 36-legendary achievement is now named Legendary discoveries.

![Shared jewelry contributes to achievement progress](images/screenshots/2026-10-26-jewel-achievement/progress-1280.png)

![Achievement earned from combined jewelry discoveries](images/screenshots/2026-10-26-jewel-achievement/earned-1280.png)

131 core and 257 app tests, native build, workspace Clippy, formatting and render
lint pass. Tests cover threshold boundaries, mixed families, duplicates, persistence
and legacy rewards. Native synthetic fixtures verify 39 shared discoveries, then
17 legendary discoveries reaching the threshold, the 10-point grant and saved date.

## 2026-10-26 — Achievement card artwork

Achievement cards now use themed zone, class, equipment, jewelry, cloak and
lifetime icons instead of repeating each category's icon. Related tiers share a
visual identity. Both placeholder achievements retain their pending criteria.

![Recently earned artwork](images/screenshots/2026-10-26-achievement-art/summary-1280.png)

![Collection achievement artwork](images/screenshots/2026-10-26-achievement-art/collection-1280.png)

![Zone achievement artwork](images/screenshots/2026-10-26-achievement-art/zones-1280.png)

![Class achievement artwork](images/screenshots/2026-10-26-achievement-art/classes-1280.png)

![Cloak and placeholder artwork](images/screenshots/2026-10-26-achievement-art/feats-1280.png)

The new `--achievement-art` audit verifies all 57 catalog icon files against the
explicit checkout data folder. 130 core tests, native build, workspace Clippy,
formatting and render lint pass. Native 1280×720 captures use a synthetic account;
these authored vanilla icon choices follow the reference's illustrated-card style.

## 2026-10-26 — Achievement earned dates

New achievements save their first-earned UTC date and show it on summary/category
cards. Older achievements stay undated; loading date metadata cannot unlock
achievements or award points. Recent ordering continues to follow grant order.

![New dated achievement alongside legacy achievements](images/screenshots/2026-10-26-achievement-dates/summary-1280.png)

![Saved date after restarting at compact size](images/screenshots/2026-10-26-achievement-dates/reloaded-640.png)

256 app tests pass, including save round-trips, repeated grants, legacy saves,
invalid metadata and leap-year boundaries. Native 1280×720 / 640×720 checks use
an isolated synthetic account and confirm persistence without a second grant.
Native build, workspace Clippy, formatting and render lint pass. Dates use the
local system clock in UTC; historical dates cannot be recovered from older saves.

## 2026-10-26 — Achievement category navigation

Achievement categories now use full-width action rows, matching the intention
reference: icon, label, count and progress share one click/focus surface. Counts
align separately from labels, and selection uses the shared burgundy/gold style.

![Achievement summary and category rows](images/screenshots/2026-10-26-achievement-nav/summary-1280.png)

![Whole-row keyboard focus at compact size](images/screenshots/2026-10-26-achievement-nav/focus-640.png)

Native 1280×720 / 640×720 checks cover icon/progress-bar clicks, keyboard focus,
Space activation and scrolling to the compact summary. Captures use a synthetic
account. Native build (34 seconds), workspace Clippy, formatting and render lint
pass. Unique achievement artwork and earned dates remain separate work;
physical controller acceptance was not performed.

## 2026-10-26 — Keep Rewards focus while editing

Keyboard/controller focus now stays on the same Rewards control after staging or
undoing an upgrade. Disabled controls clear focus; switching categories or using
the pointer cannot restore an obsolete action. The reference Confirm/Undo flow is
preserved, and the plan's stale per-card undo note is corrected.

![Focus retained after two keyboard upgrades](images/screenshots/2026-10-26-reward-focus/two-upgrades-1280.png)

![Focus retained after undoing one rank](images/screenshots/2026-10-26-reward-focus/undo-focused-1280.png)

254 app tests, native build, workspace Clippy, formatting and render lint pass.
Native checks cover consecutive upgrades/undos, repricing, disabled-control inactivity
and an unchanged account file. Physical controller acceptance remains open.

## 2026-10-26 — Warlock milestone builds

All nine Warlock milestones now have explicit Survivors effects: Affliction adds
cooldown support and damage-based healing; Demonology strengthens spells and Death
Coil defenses; Destruction adds damage, critical damage and an Immolate→Shadowburn
cooldown benefit. Tuning is provisional; demon companions remain separate work.

![Siphon Life milestone](images/screenshots/2026-10-26-warlock-milestones/siphon-life-1280.png)

![Conflagrate milestone](images/screenshots/2026-10-26-warlock-milestones/conflagrate-1280.png)

130 core and 253 app tests, native build, workspace Clippy, data audit, formatting
and render lint pass. Tests cover thresholds, node composition, direct/periodic
healing without overkill or revival, cast parameters and bounded shields/cooldowns.
Native screenshots verify descriptions on an untrained character. Live combat/balance
acceptance and remaining individual nodes stay open.

## 2026-10-26 — Druid talent training

Twelve Druid talents now strengthen Balance spells, Thorns, Rejuvenation and hero
stats. Moonfire previews show damage and critical chance; Rejuvenation previews
say healing. Coverage is 171/432 individual definitions, including 12/47 Druid nodes.

![Purchased Moonfire talent with both upgrade benefits](images/screenshots/2026-10-26-druid-nodes/moonfire-1280.png)

![Untrained Rejuvenation healing preview](images/screenshots/2026-10-26-druid-nodes/rejuvenation-1280.png)

128 core and 251 app tests, native build, workspace Clippy, data audit, formatting
and render lint pass. Native checks cover the five-point row gate, purchases,
saved Wrath/Moonfire ranks and the locked healing preview. Tests cover rank caps,
reset, spell/class scope and healing/retaliation parameters. Forms, remaining nodes,
Druid milestones and live balance acceptance remain open.

## 2026-10-26 — Warlock talent training

Twelve Warlock talents now train shadow, fire, curses and health, with green next-rank
previews and explicit Survivors conversions. Coverage reaches 159/432 individual
definitions; Warlock has 12/50 nodes, with its milestones still pending.

![Improved Corruption cooldown preview](images/screenshots/2026-10-26-warlock-nodes/corruption-1280.png)

![Demonic Embrace health preview](images/screenshots/2026-10-26-warlock-nodes/demonic-embrace-1280.png)

125 core and 250 app tests pass, including rank caps, reset, class isolation and direct,
periodic and area spell parameters. Native build and data identity audit pass;
purchases, green previews and persisted ranks were checked on an isolated character.
Workspace Clippy, formatting and render lint also pass. Live balance acceptance
remains open.

## 2026-10-26 — Shaman talent training

Thirteen Shaman nodes now train lightning, shocks, totems, critical hits and defenses,
with green next-rank previews. Shared rules apply their effects to combat; tuning is
provisional. Coverage is now 147/432 individual definitions, including 13/46 Shaman nodes.

![Convection cooldown training](images/screenshots/2026-10-26-shaman-nodes/convection-1280.png)

![Ancestral Knowledge health training](images/screenshots/2026-10-26-shaman-nodes/ancestral-knowledge-1280.png)

123 core and 249 app tests, native build, workspace Clippy, formatting and render lint
pass. The data audit verifies names and rank caps; native checks verify purchases,
previews and saved ranks. Remaining nodes and live combat/balance acceptance stay open.

## 2026-10-26 — Measurable talent coverage

Added concise per-class talent coverage and detailed missing-node reports against
an explicit game-data folder. The current audit finds 134/432 matching individual
effect definitions and 57/81 milestone descriptions; these are not combat-acceptance
claims. The implementation plan now records the gaps and corrects stale equipment,
settlement and Codex status notes.

Both reports ran against checkout data; audit-crate Clippy, formatting and whitespace
checks pass. No interface change; the latest visual evidence remains below.

## 2026-10-26 — Shaman milestone builds

All nine Shaman milestones now have described, implemented Survivors effects:
Elemental improves lightning and shocks; Enhancement adds Earth Shock protection
and offensive bonuses; Restoration adds totem-deployment healing and cooldown
support. Tuning is provisional and documented in the implementation plan.

![Mana Tide Totem milestone](images/screenshots/2026-10-26-shaman-milestones/mana-tide-1280.png)

![Stormstrike milestone](images/screenshots/2026-10-26-shaman-milestones/stormstrike-1280.png)

121 core and 248 Survivors app tests, native build, workspace Clippy, formatting
and render lint pass. Native checks confirm descriptions on an untrained preview
character. Automated
coverage checks thresholds, class/spell scope, cast parameters, healing/cooldown
caps, non-stacking absorbs and tooltip text. Individual Shaman nodes and live
combat/balance acceptance remain open.

## 2026-10-26 — Mage Arcane and Fire milestones

The six pending Arcane/Fire milestone rows now describe implemented effects.
Arcane gains critical chance, a Missiles→Explosion cooldown benefit and damage;
Fire gains Fireball damage, critical chance and critical damage. All nine Mage
milestones now work through the shared spell rules, with provisional tuning.

![Presence of Mind milestone description](images/screenshots/2026-10-26-mage-milestones/presence-of-mind-1280.png)

![Combustion milestone description](images/screenshots/2026-10-26-mage-milestones/combustion-1280.png)

120 core and 246 Survivors app tests, native build, workspace Clippy, formatting
and render lint pass. Coverage includes thresholds, class/spell isolation, node
composition, cast parameters, cooldown floors and the green tooltip. Native checks
confirm starter descriptions. Individual pending nodes and live combat/balance
acceptance remain open; these screenshots do not claim a trained build.

## 2026-10-26 — Keep talent focus while editing a spec

Buying or refunding a rank now keeps the talent focused and refreshes its green
preview. Focus survives tree rebuilds and layout delays without repeating actions;
leaving the page, switching heroes or handing control to the mouse clears it.

![Talent remains focused after consecutive refunds and a purchase](images/screenshots/2026-10-26-talent-retain/focus-after-purchase-1280.png)

244 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native keyboard validation covers
two consecutive refunds, a purchase, updated previews and saved final rank.
Physical controller acceptance and focus retention on other pages remain open.

## 2026-10-26 — Talent refunds without a mouse

Backspace or controller west refunds one point from the focused talent through
the existing row and prerequisite checks. Milestones and unrelated controls ignore
the refund shortcut; page help and README controls describe it.

![Keyboard refund returns a point to the active spec](images/screenshots/2026-10-26-talent-refund/refunded-1280.png)

243 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native validation refunded Improved Frostbolt from rank 4 to 3, increased available
points from 2 to 3, and verified the saved spec. Automated input coverage includes
one-frame requests and modal exclusion. Physical controller acceptance remains
open. Purchases/refunds currently rebuild the tree and clear focus; retaining
focus across that rebuild remains a usability task.

## 2026-10-26 — Inspect talents with directional focus

Talent and milestone descriptions now follow keyboard/controller focus, including
green rank previews for supported talents. Inspection spends no points; milestone
rows remain informational. Page help explains arrows and D-pad navigation.

![Focused talent with green rank preview](images/screenshots/2026-10-26-talent-focus/talent-1280.png)

![Focused milestone with its current implementation status](images/screenshots/2026-10-26-talent-focus/milestone-1280.png)

243 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover directional inspection and unchanged points after Enter
on a milestone. Physical controller acceptance and other hover surfaces remain
open. The pictured Mage milestone effect is still pending; this changes navigation,
not talent mechanics.

## 2026-10-26 — Starter and Armory focus navigation

D-pad and arrow keys now move a visible focus outline between starter/Armory
controls. South, Enter or Space selects through the existing action handler;
focused Enter cannot also start a run. Focus reveals scrolled controls with room
for the outline, and clears for Options, text entry and pointer clicks.

![Focused achievement category](images/screenshots/2026-10-26-starter-focus/achievements-640.png)

![Compact starter scrolled to the focused action](images/screenshots/2026-10-26-starter-focus/starter-scroll-640.png)

242 Survivors tests, native build, workspace Clippy, formatting and render lint pass.
Native compact checks cover directional focus, Options and Achievements activation,
Space selection and equipment/lower-action scroll reveal. Automated checks cover
controller edges, nested non-scrolling wrappers, display scaling, one-frame
activation, modal/text-entry exclusion and Enter consumption. Physical controller
acceptance, focus-driven hover details and secondary actions remain open.

## 2026-10-26 — Controller navigation in Options

Survivors now enables controller discovery. Start opens Options from the starter
or pause menu; D-pad moves focus/adjusts sliders, south selects and east returns.
Controller and keyboard navigation share the same focus and setting actions.

![Compact Options help and keyboard focus](images/screenshots/2026-10-26-options-controller/options-640.png)

240 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Simulated-controller tests cover both menu states, controls, sliders and
modal boundaries. Native validation covers backend startup, compact help and
keyboard focus. Physical controllers, other starter pages and controller gameplay
still need implementation/acceptance as applicable.

## 2026-10-26 — Camera and zoom sensitivity

Controls gains independent, saved camera and zoom speed sliders from 25% to 300%.
Mouse dragging and keyboard steps share the same settings; Controls defaults
restore 100% without changing Graphics. Compact guidance keeps the footer inside
the panel.

![Camera and zoom speed controls](images/screenshots/2026-10-26-controls-speed/controls-1280.png)

![Compact Controls with keyboard focus](images/screenshots/2026-10-26-controls-speed/controls-640.png)

238 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover clicks/dragging, keyboard steps and limits, category
reset isolation, wide/compact fit and saved speeds after relaunch. Runtime tests
cover rotation/zoom scaling; live in-world feel still needs acceptance.

## 2026-10-26 — Camera input isolation

Camera controls now ignore gestures while the loading screen covers the world.
Regression checks exercise the runtime input path across menus, pause, rewards
and loading, including draining scroll events before resume. They also cover
axis inversion, returning to fixed view, trackpad/wheel equivalence and camera
limits. The interface remains as pictured in the Controls milestone below.

236 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Live in-world acceptance remains open.

## 2026-10-26 — Mouse camera controls

Options gains a Controls page with saved X/Y mouse inversion, category defaults
and movement/camera guidance. Camera rotation, zoom and mode changes now accept
input only during active gameplay. Graphics resets preserve Controls preferences.

![Controls with both axes inverted](images/screenshots/2026-10-26-options-controls/controls-1280.png)

![Compact Controls with vertical inversion](images/screenshots/2026-10-26-options-controls/controls-640.png)

233 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover mouse/keyboard toggles, reset isolation, wide/compact fit
and inversion after relaunch. Tests cover direction mapping and save compatibility;
in-world inversion and paused camera isolation still need live acceptance checks.
Sensitivity, key rebinding and gamepad support remain unfinished.

## 2026-10-26 — Options checkboxes

Audio and camera switches now use labeled gold checkboxes aligned with the
sliders, following the intention reference. The whole row accepts clicks and
keyboard activation; compact labels remain readable.

![Audio checkboxes](images/screenshots/2026-10-26-options-checkboxes/audio-1280.png)

![Compact camera checkboxes](images/screenshots/2026-10-26-options-checkboxes/graphics-640.png)

38 focused HUD tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover label/box clicks, Space activation, checked/unchecked
states, Graphics defaults and saved music state after relaunch. Audible output
and in-world camera effects were not retested. Remaining categories and gamepad
navigation are still open.

## 2026-10-26 — Keyboard Options

Options now has visible keyboard focus: Tab/Shift-Tab or Up/Down moves between
visible controls, Left/Right adjusts sliders, Home/End chooses limits, and
Enter/Space activates buttons. Mac modifier handling preserves Shift-Tab.
Done closes settings without starting or resuming a run.

![Keyboard focus on Audio](images/screenshots/2026-10-26-options-keyboard/audio-1280.png)

![Compact Graphics keyboard focus](images/screenshots/2026-10-26-options-keyboard/graphics-640.png)

232 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native wide/compact checks cover focus, volume limits, category selection, graphics
adjustment, Done, pointer handoff and build notes. Gamepad navigation, checkbox
styling and remaining categories are still open. Audible output was not tested.

## 2026-10-26 — Options sliders

Audio and Graphics now use proportional gold sliders with separate labels and
values. Audio supports 1% adjustments; graphics snap to the existing supported
levels. Click or drag to adjust, including short drags and movement past an edge.

![Audio sliders](images/screenshots/2026-10-26-options-sliders/audio-1280.png)

![Compact Graphics sliders](images/screenshots/2026-10-26-options-sliders/graphics-640.png)

230 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover clicking, dragging, clamping, graphics stops, defaults,
Enter isolation, wide/compact fit and 37% volume after relaunch. Keyboard/gamepad
slider navigation, checkbox styling and remaining categories are still open;
audible output was not tested.

## 2026-10-26 — Restore category defaults

Audio and Graphics now restore their own defaults without changing the other
category. Graphics includes camera push-in and scenery cutaway. Options also uses
the reference display font and groups Defaults/Done at the panel's lower right.

![Restored Audio defaults](images/screenshots/2026-10-26-options-defaults/audio-1280.png)

![Compact Graphics defaults](images/screenshots/2026-10-26-options-defaults/graphics-640.png)

228 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Wide/compact native checks verify resets, category isolation and persistence
after relaunch. Sliders, bindings and remaining categories are still unfinished;
audible output and in-world camera effects were not retested.

## 2026-10-26 — Options categories

Options now uses the reference's category navigation with focused Audio, Graphics
and About panels. About restores access to build notes and local restart controls
from the starter. Done/Escape returns to the existing page.

![Audio category](images/screenshots/2026-10-26-options-categories/audio-1280.png)

![Compact Graphics category](images/screenshots/2026-10-26-options-categories/graphics-640.png)

![About and build notes access](images/screenshots/2026-10-26-options-categories/about-640.png)

Native 1280×720 / 640×720 checks cover category switching, saved edits, Enter
isolation, Done and build-notes navigation. All 33 focused HUD tests, native build,
workspace Clippy, formatting and render lint pass. Sliders, defaults, bindings,
remaining categories and full reference styling are still open; audible output
and an actual restart were not tested in this checkpoint.

## 2026-10-26 — Settings before a run

Options now opens the shared audio, graphics and camera settings from the starter
and every Armory page. Back/Escape returns to the same page; Enter cannot start a
run while settings is open. Escape also closes paused settings before resuming.

![Settings with persisted preferences](images/screenshots/2026-10-26-starter-settings/settings-1280.png)

![Compact settings panel](images/screenshots/2026-10-26-starter-settings/settings-640.png)

226 Survivors tests, native build, workspace Clippy, formatting and render lint
pass. Native checks cover edits, restart persistence, Enter isolation, Back/Escape
and wide/compact fit. The reference's category navigation, gamepad/key bindings,
sliders and remaining settings are still unfinished; audible output was not tested.

## 2026-10-25 — Starter action cards

Starter actions now share the Armory's gold outlines and burgundy emphasis.
Armory, Talents and entry controls include their supporting text inside the
clickable card. Entry width is proportional, with wrapping for long zone names.

![Starter action cards](images/screenshots/2026-10-25-starter-actions/starter-1280.png)

![Compact long-name entry and lock](images/screenshots/2026-10-25-starter-actions/locked-zone-640.png)

Native wide/compact checks cover Armory/Talents navigation, locked entry,
Options navigation and Quit. Build, workspace Clippy, formatting and render lint
pass. Options still opens placeholder notes; a complete settings interface,
background and outdoor zone artwork remain open.

## 2026-10-25 — Starter portrait identity

The starter portrait now carries the reference's faction label and circular class
badge. Both follow the selected hero; the badge scales with the portrait.

![Horde Warrior portrait identity](images/screenshots/2026-10-25-starter-badges/horde-1280.png)

![Alliance Mage at compact width](images/screenshots/2026-10-25-starter-badges/alliance-640.png)

Native 1280×720 / 640×720 checks cover Horde→Alliance class switching, portrait
dragging and scroll clipping. Native build, workspace Clippy, formatting and
render lint pass. Remaining starter controls, background and zone artwork still
need reference alignment.

## 2026-10-25 — Companion reference artwork

Matched Long Leash and five companion looks to the intention screenshot using
verified local 1.12.1 icons. The whistle, frog, serpents, Zergling and cockroach
now use the reference artwork.

![Companion cards with the reference whistle](images/screenshots/2026-10-25-companion-reference-icons/cards-1280.png)

![Complete companion looks at compact width](images/screenshots/2026-10-25-companion-reference-icons/looks-640.png)

Native 1280×720 / 640×720 inspection, patch-chain asset test, build, formatting
and render lint pass. The preceding integration run passed all 224 Survivors app
tests. This artwork checkpoint does not establish in-world pet animation acceptance.

## 2026-10-25 — Shadow Weaving

Shadow Weaving now builds vulnerability on each enemy: 20% application chance per
rank, +3% subsequent Shadow damage per stack, five stacks maximum. Shadow hits
and Pain ticks refresh the 15-second duration after dealing their damage.

![Shadow Weaving alongside healing and control](images/screenshots/2026-10-25-shadow-weaving/cards-1280.jpg)

![Complete compact combined-effect description](images/screenshots/2026-10-25-shadow-weaving/scream-640.jpg)

119 core tests, direct/periodic hit-order regression, DBC audit, native build,
Clippy, formatting and render lint pass. Wide/compact card text is verified;
live vulnerability balance and remaining talents are open.

## 2026-10-25 — Vampiric Embrace

Vampiric Embrace now heals for 20% of enemy health removed by Priest Shadow
hits and Pain ticks. Its two improvement ranks raise this to 25% / 30% and require
the base talent. Overkill and dead targets grant no additional healing; dead
players cannot revive through it. This is always-active self-healing in Survivors.

![Shadow cards with Vampiric Embrace](images/screenshots/2026-10-25-vampiric-embrace/cards-1280.jpg)

![Compact combined healing, stun and shield description](images/screenshots/2026-10-25-vampiric-embrace/scream-640.jpg)

118 core tests, the direct/periodic healing regression, DBC audit, native build,
Clippy, formatting and render lint pass. Wide/compact descriptions fit and scroll;
live healing balance and remaining talents are open.

## 2026-10-25 — Blackout and Shadow control

Blackout now gives Priest Shadow damage a 2% stun chance per rank, including Pain
ticks. Stuns last 3 seconds (0.9 on bosses), refresh without stacking and preserve
longer stuns. Cards and tooltips include the effect inline.

![Blackout on high-rank Shadow cards](images/screenshots/2026-10-25-blackout/cards-1280.jpg)

![Compact Psychic Scream with stun and shield effects](images/screenshots/2026-10-25-blackout/scream-640.jpg)

117 core tests, the direct/periodic engine regression, DBC audit, native build,
Clippy, formatting and render lint pass. Wide/compact card text and scrolling are
verified; live crowd-control balance remains open.

## 2026-10-25 — Priest defensive talents

Fortitude now grants 10 maximum health per rank, Inner Fire grants 2 armor, and
Spell Warding grants 2% incoming damage reduction. These replace provisional
damage bonuses. Fortitude/Inner Fire are always active; Warding currently covers
all damage because incoming damage schools are not modeled.

![Inner Fire purchase/refund and armor preview](images/screenshots/2026-10-25-priest-defenses/inner-fire-1280.jpg)

116 core tests, the saved-spec/combat-sheet regression, DBC audit, native build,
Clippy, formatting and render lint pass. Native Inner Fire purchase/refund is
verified at 1280×720. Combat balance and remaining Priest talents stay open.

## 2026-10-25 — Preview the selected build without a run

Card previews now accept `profile = selected`, using the normal starting build's
gear, talents, race and account bonuses. Change cards, ranks, size or profile by
editing the preview file; no combat session or rebuild is needed.

![Selected Priest build at 1280×720](images/screenshots/2026-10-25-selected-card-preview/wide-1280.jpg)

![Talent healing descriptions at 640×720](images/screenshots/2026-10-25-selected-card-preview/compact-640.jpg)

Six preview tests, native build, workspace Clippy, formatting and render lint pass.
Native checks cover high-rank Holy healing, compact scrolling, baseline reload and
return to Talents; the isolated account checksum is unchanged. Other classes and
passive/jewelry card previews remain outside this check.

## 2026-10-25 — Healing visible on upgrade cards

Talent-triggered healing now appears in the shared card and tooltip description,
including “once per successful cast.” Changed values use the existing green inline
comparison; unchanged healing stays plain. Combat tuning is unchanged.

![Holy Fire with trained healing](images/screenshots/2026-10-25-card-healing/holy-fire-1280.jpg)

30 HUD tests, native build, workspace Clippy, formatting and render lint pass.
The trained Holy Fire card fits at 1280×720; selecting it advances the spell to
rank 2 and resumes combat. Compact talent-added descriptions remain to be checked.

## 2026-10-25 — Force of Will

Priest Force of Will now grants 1% Holy/Shadow spell damage and one percentage
point of direct critical chance per rank. Periodic damage stays non-critical;
shields and foreign-class/jewelry spells receive no bonus.

![Force of Will next-rank preview](images/screenshots/2026-10-25-force-of-will/talent-1280.jpg)

116 core tests, the ranked engine regression, DBC audit, native build, Clippy,
formatting and render lint pass. Native purchase/refund is verified at 1280×720.
Live combat balance and remaining individual talents are still open.

## 2026-10-25 — Long-card layout acceptance

Checked high-rank ground, chain and damage-over-time descriptions at 1280×720 and
640×720. Text stays inside the cards, and the compact third card is reachable by
scrolling. Reusable presets are committed under `docs/ui-previews/`.

![Long descriptions at 1280×720](images/screenshots/2026-10-25-long-cards/wide-1280.jpg)

![Ground and chain descriptions at 640×720](images/screenshots/2026-10-25-long-cards/compact-640.jpg)

![Scrolled damage-over-time card](images/screenshots/2026-10-25-long-cards/compact-scrolled-640.jpg)

Native visual validation only; no engine change. Talent-added text and other
viewport sizes remain outside this acceptance check.

## 2026-10-25 — Direct, reloadable card previews

Upgrade cards can now open directly from the UI preview file. Change spell keys,
ranks and window size without recompiling or playing a run. The production cards
still animate and scroll; picks and gameplay starts are disabled in this view.

![Three-card preview at 1280×720](images/screenshots/2026-10-25-card-preview/wide-1280.jpg)

![Wrapped third card reached by scrolling at 640×720](images/screenshots/2026-10-25-card-preview/compact-640.jpg)

Five preview tests, native build, workspace Clippy, formatting and render lint pass.
Native checks cover resize, scrolling, rejected input,
invalid requests and return to Talents; the account checksum is unchanged.
Setup and baseline-stat limitations are in [development instructions](SURVIVORS-DEVELOPMENT.md).

## 2026-10-25 — Casting distance on upgrade cards

Targeted spell cards now state their effective casting distance inline, separately
from splash or area radius. Range changes use green comparisons with one trailing
unit; unchanged distances remain ordinary text.

![Live Smite upgrade with Holy Reach's 24-yard range](images/screenshots/2026-10-25-card-range/smite-1280.jpg)

29 HUD tests pass. The live 1280×720 card fits, and selecting it advances Smite to
rank 2 and resumes combat. Native build, Clippy, formatting and render lint pass.
This capture verifies the Smite card; other card layouts still need visual coverage.

## 2026-10-25 — Priest reach talents

Holy Reach extends Smite/Holy Fire casting range and Holy Nova's single area.
Shadow Reach extends Shadow Word: Pain/Mind Blast range. Both reach 20% at maximum
rank, with scoped green descriptions; neither adds targets or extra AoE locations.

![Holy Reach at 1280×720](images/screenshots/2026-10-25-priest-reach/holy-1280.jpg)

![Shadow Reach at 640×720, with maximum-rank purchase guard](images/screenshots/2026-10-25-priest-reach/shadow-640.jpg)

115 core tests, the ranked engine parameter test, DBC catalog audit, native build,
Clippy, formatting and render lint pass. Holy purchase/refund and compact Shadow
text were checked natively. Live combat range acceptance remains open.

## 2026-10-25 — Spirit Tap recovery

Priest Spirit Tap now gives kills a 20% chance per rank to regenerate 1% maximum
health per second for 15 seconds. Further procs refresh the duration without
stacking; the active-buff readout shows time remaining.

![Spirit Tap purchase/refund preview at 1280×720](images/screenshots/2026-10-25-spirit-tap/talent-1280.jpg)

114 core tests and four engine tests pass, covering direct/periodic kills, expiry,
refreshes and frame-independent healing. Native purchase/refund, build, Clippy,
formatting and render lint pass. Live combat visuals and balance remain unverified.

## 2026-10-25 — Priest tree milestones

All nine Priest milestones now have explicit effects: shield protection, linked
spell cooldowns, Holy healing and critical chance, and Shadow targeting/absorption.
The UI describes each automatic Survivors conversion.

![Holy Lightwell healing milestone](images/screenshots/2026-10-25-priest-milestones/lightwell-1280.jpg)

![Compact Shadow Weaving interaction](images/screenshots/2026-10-25-priest-milestones/shadow-weaving-640.jpg)

113 core tests and two Priest combat tests pass. Actual casts verify no-target and
multi-target behavior; native UI, build, Clippy, formatting and render lint pass.
Initial balance, presentation refinement and remaining individual talents stay open.

## 2026-10-25 — Priest spell training

Eight Priest talents now improve their named spells: Holy damage/critical chance,
casting cooldowns, Shadow damage and duration, Psychic Scream cadence and shield
absorption. Green previews explain their scope and Survivors conversions.

![Searing Light next-rank damage at 1280×720](images/screenshots/2026-10-25-priest-training/searing-light-1280.jpg)

![Shadow Word Pain duration at 640×720](images/screenshots/2026-10-25-priest-training/pain-640.jpg)

112 core tests and the ranked combat-parameter regression pass. Native UI checks,
build, workspace Clippy, formatting and render-pass lint pass. These are initial
balance values; Priest milestones and remaining nodes are still unfinished.

## 2026-10-25 — Saved specs follow talent-tree rules

Saved DBC specs now require valid class/tree membership, rank caps, earlier-row
points and prerequisites before granting talent effects. Invalid ranks remain
visible for repair; valid sibling specs still work. Effects wait for the catalog
at startup, and the page explains invalid specs without changing its reference layout.

![Invalid saved Vigor rank remains visible with repair guidance](images/screenshots/2026-10-25-talent-catalog/invalid-1280.jpg)

![Refunding Vigor restores the remaining valid spec](images/screenshots/2026-10-25-talent-catalog/repaired-1280.jpg)

Native 1280×720 checks verified the warning and refund recovery. All 207 Survivors
app tests pass, including catalog rejection, preserved ranks, sibling specs,
recovery and pending-catalog behavior. Native build, workspace Clippy, formatting
and render-pass lint pass. Legacy non-DBC spreads retain their existing rules;
this does not complete the remaining individual talent implementations.

## 2026-10-25 — Saved talent consistency

Repeated talent rows now load once per spec at their highest saved rank, so they
cannot inflate tree points or unlock extra milestones. Malformed rows are rejected.
Separate specs and unrelated progression bonuses remain intact.

All 42 account tests pass, including duplicate/malformed imports, spec isolation,
spell effects and save round trips. Workspace Clippy, formatting and render-pass
lint pass. This is a save-parser change with no visual change; full catalog rank,
class, row and prerequisite validation remains open.

## 2026-10-25 — Character switching acceptance

The ten-hero roster and creation controls fit at 640×720. Switching from Warrior
to Paladin restores the saved female model and class gear; switching back restores
the Warrior's five feature values. This checkpoint adds visual evidence, without
changing runtime code.

![Saved Paladin after switching heroes](images/screenshots/2026-10-25-character-acceptance/paladin-640.jpg)

![Ten-hero management at compact width](images/screenshots/2026-10-25-character-acceptance/roster-640.jpg)

![Warrior features restored after switching back](images/screenshots/2026-10-25-character-acceptance/restored-640.jpg)

Native checks used an isolated populated account. Full-HD acceptance remains open:
macOS clamped the requested 1920×1080 window, invalidating that capture. No new
creation or restart-persistence claim is made by this checkpoint.

## 2026-10-25 — Imported talent budget protection

Overspent saved specs now keep their ranks but grant no talent effects until their
point budget is repaired. Valid sibling specs remain usable. The talent page
explains the recovery options and dims inactive milestones.

![Saved spec exceeds its earned points](images/screenshots/2026-10-25-talent-budget/inactive-1280.jpg)

![Refunding the excess rank restores the spec](images/screenshots/2026-10-25-talent-budget/repaired-1280.jpg)

Native 1280×720 checks verified the warning and recovery after refunding one rank.
The regression covers preserved saves, suspended bonuses, independent specs and
reactivation through refunds or earned points. All 202 Survivors app tests pass.
Native build, workspace Clippy, formatting and render-pass lint pass. Full
imported-catalog validation remains open.

## 2026-10-25 — Rogue cooldown talents

Three talents replace provisional damage with explicit cooldown effects: Improved
Slice and Dice speeds up Blade Flurry, Elusiveness speeds up Gouge/Kidney Shot,
and Vigor speeds up Rogue spells. Green previews label the Survivors conversions
and their exact scope. Rogue now has sixteen implemented individual talents.

![Vigor at maximum rank](images/screenshots/2026-10-25-rogue-cooldowns/vigor-1280.jpg)

![Compact Elusiveness next-rank preview](images/screenshots/2026-10-25-rogue-cooldowns/elusiveness-640.jpg)

111 core tests, a cast-parameter regression across ability ranks 1/3/8, local DBC
audit, native build, workspace Clippy, formatting and render-pass lint pass.
Native checks verified Vigor refund/repurchase and compact Elusiveness refund/restore.
Live combat feel and final balance remain open; other Rogue nodes are still provisional.

## 2026-10-25 — Focused Character appearance actions

Character now follows the reference’s focused Features/Look column. Appearance
actions stay visible at the bottom of the wide panel; “Inspect in Collection”
opens the selected item’s full sources, equipment comparisons and equip controls.
Both pages share the same appearance actions.

![Character with visible Look actions and helmet hidden](images/screenshots/2026-10-25-character-look/character-1280.jpg)

![Compact equipment and Look section](images/screenshots/2026-10-25-character-look/look-640.jpg)

Native 1280×720 checks verified helmet hiding with unchanged equipment bonuses
and selected-shoulder navigation into Collection. The 640×720 Look section remains
reachable below equipment. Native build, workspace Clippy, formatting and
render-pass lint pass. Full race/model/resolution acceptance remains open.

## 2026-10-25 — Character appearance feature cards

Appearance controls now follow the Character intention’s bordered rows: muted
labels and centered counters, with previous/next buttons at opposite edges. Rows
use the available panel width and grow with their contents.

![Character feature cards at 1280×720](images/screenshots/2026-10-25-feature-cards/character-1280.jpg)

![Scrollable appearance controls at 640×720](images/screenshots/2026-10-25-feature-cards/features-640.jpg)

Native 1280×720 and 640×720 checks verified all five rows, increment/decrement
and stable compact scroll position. Native build, workspace Clippy, formatting
and render-pass lint pass. This is a presentation change; full race/model and
resolution acceptance remains open.

## 2026-10-25 — Native cosmetic shortcut verified

Shift-right-click now reads native modifier flags as well as physical keys. A
macOS trace confirmed Shift flags arrived without key events; event-order tracking
fixes the cosmetic gesture and clears state on focus loss or window destruction.

![Shift-right-click selects the legendary look while the common helmet remains equipped](images/screenshots/2026-10-25-native-look-shortcut/cosmetic-1280.jpg)

Native 1280×720 validation retained the common helmet and its bonuses while
selecting the legendary look. Subsequent ordinary right-clicks removed/restored
equipment without changing that look. Five shortcut regressions pass, including
release ordering and window isolation. Native build, workspace Clippy, formatting
and render-pass lint pass. This resolves the preceding macOS fixture
failure; other platforms and full model acceptance remain open.

## 2026-10-25 — Cosmetic shortcut release handling

Right-click now remembers Shift from the start of the gesture and accepts a
Shift release in the mouse-up frame. Two regressions cover both Shift keys,
same-frame press/release and an ordinary following click without modifier leakage.
Native build, workspace Clippy, formatting and render-pass lint pass.

**Native acceptance remains open:** the automated macOS modified click still
equips the item. The automation tool cannot issue a standalone held Shift key,
so this does not establish the source of the remaining mismatch. Explicit look
buttons remain usable; this checkpoint is not a claim that the observed shortcut
problem is resolved.

![Unresolved automated gesture equips the legendary helmet instead of choosing its look](images/screenshots/2026-10-25-shift-look/automated-unresolved-1280.jpg)

## 2026-10-25 — Save failure and restart verification

Failed account writes now log a diagnostic while retaining pending changes. A
real filesystem failure regression verifies the old save remains intact and a
successful retry preserves equipment, looks, heroes, specs and rewards.

![Common helmet equipped, legendary look selected, shoulder empty after restart](images/screenshots/2026-10-25-equipment-restart/restored-1280.jpg)

All 39 account tests, native build, workspace Clippy, formatting and render-pass
lint pass. A full native restart retained the deliberately weaker helmet, separate
legendary appearance, empty shoulder slot and unchanged totals. The explicit
“This piece” control was verified; automated Shift-right-click instead equipped
the item, so that shortcut still needs investigation. This is not full appearance
or platform acceptance.

## 2026-10-25 — Equipment change previews

Collection now previews total equipment bonuses before equipping or removing an
owned piece. Gains are green, losses red, and units appear once. The comparison
includes gaining or losing the nine-piece worn-set bonus; cosmetic choices and
collection ownership stay independent.

![Unequip preview includes set-bonus losses](images/screenshots/2026-10-25-equipment-preview/set-loss-1280.jpg)

![Compact equip preview restores the set bonus](images/screenshots/2026-10-25-equipment-preview/set-gain-640.jpg)

110 core and 194 Survivors app tests, native build, workspace Clippy, formatting
and render-pass lint pass. Tests cover downgrade and set changes without mutating
the loadout. Native 1280×720/640×720 checks verified removal/restoration against
the displayed equipment totals. These are isolated account fixtures, not earned
progression or balance acceptance.

## 2026-10-25 — Collection item identity card

The selected item's icon, name, quality and ownership now share a compact bordered
card, following the Collection intention. Names wrap within the available width;
the card grows vertically instead of clipping them.

![Collection item card at 1280×720](images/screenshots/2026-10-25-collection-header/collection-1280.jpg)

![Collection item card at 640×720](images/screenshots/2026-10-25-collection-header/collection-640.jpg)

Native build, workspace Clippy, formatting and render-pass lint pass. An isolated
collected Warrior fixture verified selection changes and a long shoulder name at
both sizes. This is presentation-only. Full collection-page visual acceptance,
Poor drops and distinct raid-epic/set content remain open.

## 2026-10-25 — Rogue proc combat regression

The actual auto-cast system now has regression coverage for all four Remorseless
Attacks spells: no target preserves the bonus, both targets share it on a
successful cast, and the next cast uses ordinary critical chance. All 194
Survivors app tests, workspace Clippy, formatting and render-pass lint pass.
This is test-only; the UI captures below remain current. Live combat acceptance
and an active-proc HUD indicator are still open.

## 2026-10-25 — Remorseless Attacks

Rogue kills now grant +20% critical chance per rank to the next successful
Sinister Strike, Eviscerate, Gouge or Kidney Shot cast. The bonus expires after
20 gameplay seconds and refreshes without stacking; areas and bleeds do not
consume it. This replaces the node's provisional permanent damage bonus.

![Remorseless Attacks rank preview](images/screenshots/2026-10-25-remorseless-attacks/talent-1280.jpg)

![Remorseless Attacks in the compact layout](images/screenshots/2026-10-25-remorseless-attacks/talent-640.jpg)

109 core and 193 Survivors app tests, native build, workspace Clippy, formatting
and render-pass lint pass. Tests cover rank limits, eligible spells, expiry,
refresh and consumption. Native refund/rebuy and description wrapping were
checked at 1280×720 and 640×720. Live combat acceptance remains open.

## 2026-10-25 — Rogue defense and mobility choices

Lightning Reflexes and Sleight of Hand now reduce damage taken, Deflection adds
armor, and Camouflage increases movement speed. These replace provisional damage
bonuses; descriptions explicitly name the Survivors conversions instead of
implying simulated dodge, parry, threat or stealth.

![Camouflage movement-speed preview](images/screenshots/2026-10-25-rogue-defense/camouflage-1280.jpg)

![Deflection armor preview](images/screenshots/2026-10-25-rogue-defense/deflection-1280.jpg)

108 core tests, the local talent-data audit, native build, workspace Clippy,
formatting and render-pass lint pass. Rank caps, class isolation and provisional
bonus removal are covered. Camouflage and Deflection refund/rebuy and descriptions
were checked at 1280×720; these nodes still need compact and live-combat acceptance.

## 2026-10-25 — Subtlety milestone interactions

Subtlety now links control spells to defense and finishers: Gouge grants a small
absorb at 10 points, Kidney Shot advances Gouge at 20, and Gouge advances Eviscerate
at 30. Triggers run once per successful cast, affect only acquired abilities,
and never stack the absorb. Ability text names the affected cooldown in green.

![Premeditation at 1280×720](images/screenshots/2026-10-25-rogue-subtlety/premeditation-1280.jpg)

![Master of Deception in the compact talent layout](images/screenshots/2026-10-25-rogue-subtlety/deception-640.jpg)

107 core and 192 Survivors app tests pass, including threshold isolation,
cooldown floors, dead-hero exclusion, stronger-shield preservation and the existing
Paladin interaction. Native build, workspace Clippy, formatting and render-pass
lint pass. All three Subtlety descriptions were checked at 640×720; Premeditation
was also captured at 1280×720. All nine Rogue milestones now have effects;
remaining individual talents and live-combat/balance acceptance are still open.

## 2026-10-25 — Assassination and Combat milestones

Six Rogue milestones now have explicit effects. Assassination adds critical
chance/damage and guaranteed Eviscerate criticals at 30 points. Combat shortens
cooldowns and widens Blade Flurry while keeping a single area. Descriptions name
the automatic Survivors conversions; [the rules](SURVIVORS.md#rogue-tree-milestones)
record values and stacking.

![Cold Blood on the active Assassination spec](images/screenshots/2026-10-25-rogue-milestones/cold-blood-1280.jpg)

![Adrenaline Rush after switching to Combat](images/screenshots/2026-10-25-rogue-milestones/adrenaline-rush-1280.jpg)

106 core tests and 191 Survivors app tests pass, including threshold, composition,
critical-roll and area invariants. Native build, workspace Clippy, formatting and
render-pass lint pass. Live 1280×720 checks verified the two independent 30-point
specs and their milestone descriptions. Subtlety, remaining individual nodes,
compact milestone checks and live-combat/balance acceptance remain open.

## 2026-10-25 — Rogue control and critical strikes

Combat-path follow-up: two regressions verify ranked casts, critical rolls,
Rupture tick damage and Gouge stun/marker timing with equipment modifiers.
All 31 combat milestone tests and workspace Clippy pass, along with formatting
and render-pass lint. No UI change; the captures below remain current.

Improved Gouge now extends its stun by 25% per rank. Lethality adds 6 percentage
points of critical damage per rank to Sinister Strike and Gouge; Aggression adds
2% damage per rank to Sinister Strike and Eviscerate. These replace provisional
rank bonuses and compose with the earlier Rogue talents.

![Improved Gouge purchased with its duration preview](images/screenshots/2026-10-25-rogue-control/gouge-1280.jpg)

![Lethality effect and unmet prerequisite](images/screenshots/2026-10-25-rogue-control/lethality-1280.jpg)

All 105 core tests, talent-data audit, native build, workspace Clippy, formatting
and render-pass lint pass. Live 1280×720 checks covered all three descriptions,
Gouge purchase/refund, and Lethality/Aggression gates. The three-tree layout follows
the intention reference; live-combat and compact-layout acceptance for these
new descriptions remain open.

## 2026-10-25 — First explicit Rogue talents

Save-integrity follow-up: all 38 account tests pass, including a new Rogue
regression for save/reload, character isolation, spec switching and reset
persistence. Workspace Clippy, formatting and render-pass lint pass. This is
coverage only; the UI captures below remain current.

Five audited Rogue talents now replace their provisional damage bonus with real
spell effects: Improved Eviscerate (+5% damage/rank), Malice (+1% Rogue critical
chance/rank), Improved Sinister Strike (5% shorter cooldown/rank), Serrated Blades
(+10% Rupture damage/rank), and Deadliness (+2% Rogue spell damage/rank). UI text
labels Survivors conversions and excludes unsupported armor penetration.

![Improved Eviscerate rank and effect](images/screenshots/2026-10-25-rogue-training/eviscerate-1280.jpg)

![Sinister Strike cooldown conversion](images/screenshots/2026-10-25-rogue-training/sinister-strike-1280.jpg)

All 104 core tests, the checkout talent-catalog audit, native build, workspace
Clippy, formatting and render-pass lint pass. Tests cover spell/class isolation,
composition, rank caps and removal of the provisional bonus. Spending, refund,
reset and two descriptions were verified live at 1280×720. Remaining Rogue nodes,
milestones and live-combat acceptance are still open.

## 2026-10-25 — Racials for every playable race

All eight races now have authored Survivors bonuses. Human rules stay intact;
Orcs gain Blood Fury, Dwarves Stoneform, Night Elves timed protection, and Trolls
Berserking. Undead regeneration, Tauren durability and Gnome utility round out the
set. Starter descriptions list exact effects; [the rules](SURVIVORS.md) document
values and stacking. These are initial balance values.

![Orc racial rules on the starter](images/screenshots/2026-10-25-racial-bonuses/orc-1280.jpg)

![Racial descriptions at 640×720](images/screenshots/2026-10-25-racial-bonuses/orc-640.jpg)

Two core timing tests and four run-stat tests pass, covering all races, window
boundaries, invalid time, stat integration and removal without accumulation.
Native build, workspace Clippy, formatting and render-pass lint pass. Orc starter text was checked
live at both sizes; extended combat/balance playtests remain outstanding.

## 2026-10-25 — Companion reference artwork

Quick Paws now uses the cat artwork; Crimson Macaw and Great Horned Owl use the
owl artwork shown in the intention reference. Both icon paths were verified and
decoded from this checkout's data archive, then checked in the native UI.

![Verified cat and owl artwork](images/screenshots/2026-10-25-companion-art/companions-1280.jpg)

Native build, workspace Clippy, formatting and render-pass lint pass. Appearance
IDs and unlock rules are unchanged. Other differing Companion icons remain under
review; no extracted game assets are committed.

## 2026-10-25 — Consistent reward grids

Companion upgrades now keep two equal columns, including the last card. Looks
and Tabards use adaptive grid columns, so incomplete rows retain the widths of
the rows above. The Companion Looks grid shows four columns in the wide preview
and three at 640×720, following the reference's aligned tile layout.

![Aligned Companion upgrades and looks](images/screenshots/2026-10-25-reward-grids/companions-1280.jpg)

![Tabard selection at 1280×720](images/screenshots/2026-10-25-reward-grids/tabards-1280.jpg)

![Compact Looks selection and final row](images/screenshots/2026-10-25-reward-grids/looks-640.jpg)

Verified wide and compact Companion layout, look selection, and wide Tabard
selection/removal live. Native build, workspace Clippy, formatting and render-pass
lint pass. Artwork and remaining page-specific reference details are unfinished;
this changes layout only.

## 2026-10-25 — Compact Companion upgrades

Companion cards now place rank markers beside their names and cost/+⁄− controls
at the right, following the Companion intention screenshot. Purchased ranks use
a gold-tinted background. Plus is disabled when unavailable or unaffordable;
minus removes pending upgrades. Green inline scaling and instinct destinations
remain visible.

![Companion upgrades at 1280×720](images/screenshots/2026-10-25-compact-companions/companions-1280.jpg)

![Pending Long Leash rank and disabled next purchase](images/screenshots/2026-10-25-compact-companions/pending-1280.jpg)

![Scrolled Companion cards at 640×720](images/screenshots/2026-10-25-compact-companions/companions-640.jpg)

Verified preview, disabled purchases, individual removal with point restoration,
compact text fit, and switching/restoring the first companion's look live. Native
build, workspace Clippy, formatting and render-pass lint pass. Costs and effects
are unchanged. Artwork, the final card's full-width stretch, and Looks/Tabards
grid alignment remain open.

## 2026-10-25 — Rewards sidebar

Available points now stand out above full-width category controls. Confirm/Undo
stay visible and dim when there is no pending purchase; pending changes survive
category switching. This follows the currency and navigation hierarchy in the
Rewards intention reference.

![Rewards sidebar at 1280×720](images/screenshots/2026-10-25-reward-rail/rewards-1280.jpg)

![Pending purchase enables Confirm and Undo](images/screenshots/2026-10-25-reward-rail/pending-1280.jpg)

![Rewards sidebar at 640×720](images/screenshots/2026-10-25-reward-rail/rewards-640.jpg)

Verified disabled controls, purchase preview, category switching, Undo and compact
text fit live. Native build, workspace Clippy, formatting and render-pass lint
pass. Companion/Tabard card styling and other page-specific reference details
remain unfinished; costs and effects are unchanged.

## 2026-10-25 — Armory section framing

Major Armory section headings are now centered between flexible gold divider
lines that fade toward the panel edges, matching the intention references.
Item-source labels and equipment totals retain their existing alignment.

![Collection section framing](images/screenshots/2026-10-25-section-dividers/collection-1280.jpg)

![Achievements heading and summary](images/screenshots/2026-10-25-section-dividers/achievements-1280.jpg)

![Rewards section dividers](images/screenshots/2026-10-25-section-dividers/rewards-1280.jpg)

![Compact Talent Points panel](images/screenshots/2026-10-25-section-dividers/talents-640.jpg)

Verified wide Collection, Achievements and Rewards, page switching, and the
scrolled 640×720 talent panel live. Native build, workspace Clippy, formatting
and render-pass lint pass. Gameplay is unchanged. Currency/category rails,
remaining texture buttons, portrait badges and other page-specific details still
need reference alignment.

## 2026-10-25 — Compact reward perks

Perk cards now group rank markers beside their names and put cost and +/− controls
on the right, following the Rewards intention screenshot. Inline green upgrade
changes remain in the description. Plus is disabled at max rank or when points
are insufficient; minus removes only pending purchases. Confirm/Undo still apply
to the whole draft.

![Compact two-column perks](images/screenshots/2026-10-25-perk-layout/perks-1280.jpg)

![Pending rank with max-rank and unaffordable upgrades disabled](images/screenshots/2026-10-25-perk-layout/pending-1280.jpg)

![Scrolled perks at 640×720](images/screenshots/2026-10-25-perk-layout/perks-640.jpg)

Verified +/−, point restoration, disabled controls and wide/compact text fit live.
Native build, workspace Clippy, formatting and render-pass lint pass. Costs and
perk effects are unchanged. Currency/category rail styling, section dividers and
Companion/Tabard reference alignment remain unfinished.

## 2026-10-25 — Outlined Armory controls

Navigation and shared Armory buttons now use thin gold outlines, a burgundy
selected state and hover/pressed feedback, following the intention references.
Tabs size to their labels and wrap; the starter no longer shows a redundant Done
button. Armory pages retain Done to return to the starter.

![Collection controls at 1280×720](images/screenshots/2026-10-25-armory-controls/collection-1280.jpg)

![Reward preview and confirmation controls](images/screenshots/2026-10-25-armory-controls/rewards-pending-1280.jpg)

![Compact navigation after undoing a reward preview](images/screenshots/2026-10-25-armory-controls/rewards-640.jpg)

![Compact starter navigation](images/screenshots/2026-10-25-armory-controls/starter-640.jpg)

Verified tab navigation, equipment unequip/re-equip, reward preview/Undo, hover
feedback and 1280/640×720 layouts live. Native build, workspace Clippy, formatting
and render-pass lint pass. Gameplay rules are unchanged. Texture-based starter
and Options/Quit controls, section dividers and broader page styling remain open.

## 2026-10-25 — Armory display headings

The Armory title and primary section headings now share the intention design's
Morpheus face across Character, Collection, Drop Tables, Codex, Talents,
Achievements and Rewards. Item names, stats and controls keep their readable face.

![Character at 1280×720](images/screenshots/2026-10-25-armory-headings/character-1280.jpg)

![Collection at 1280×720](images/screenshots/2026-10-25-armory-headings/collection-1280.jpg)

![Drop Tables at 1280×720](images/screenshots/2026-10-25-armory-headings/drops-1280.jpg)

![Codex at 1280×720](images/screenshots/2026-10-25-armory-headings/codex-1280.jpg)

![Talents at 1280×720](images/screenshots/2026-10-25-armory-headings/talents-1280.jpg)

![Achievements at 1280×720](images/screenshots/2026-10-25-armory-headings/achievements-1280.jpg)

![Rewards at 1280×720](images/screenshots/2026-10-25-armory-headings/rewards-1280.jpg)

![Compact talent headings after scrolling](images/screenshots/2026-10-25-armory-headings/talents-640.jpg)

Verified all seven pages live at 1280×720, tab rebuilds, and wrapped/scrolled
Talents at 640×720. Native build, workspace Clippy, formatting and render-pass
lint pass. No mechanics changed. Buttons, decorative dividers and broader
reference alignment remain unfinished; this is not full interface acceptance.

## 2026-10-25 — Starter display headings

The starter title, section headings and zone names now use the installed Morpheus
font, bringing their letterforms closer to the intention screenshot. Descriptions,
stats and controls retain the readable body face. Fonts remain in the user's game
data; heading text keeps its existing face until Morpheus loads.

![Starter display headings at 1280×720](images/screenshots/2026-10-25-display-headings/starter-1280.jpg)

![Compact starter headings](images/screenshots/2026-10-25-display-headings/starter-640.jpg)

![Long zone names in the compact list](images/screenshots/2026-10-25-display-headings/zones-640.jpg)

Verified font rendering, outlined glyph alignment, zone-selection rebuilds and
640×720 fit live. Native build, workspace Clippy, formatting and render-pass lint
pass. No gameplay rules changed. Other Armory pages, button typography, portrait
badges and outdoor zone art remain reference-alignment work.

## 2026-10-25 — Equipment wrapping

Starter equipment slots now share grid columns across wrapped rows. The final
main-hand slot no longer stretches across the entire panel at 1100 pixels.
Worn/empty slots retain their rarity borders and Collection inspection action.

![Consistent wrapped slot widths at 1100×720](images/screenshots/2026-10-25-equipment-wrap/starter-1100.jpg)

![Wrapped main-hand slot opens the matching item](images/screenshots/2026-10-25-equipment-wrap/main-hand-inspection.jpg)

![Single-row equipment at 1280×720](images/screenshots/2026-10-25-equipment-wrap/starter-1280.jpg)

![Scrolled compact equipment at 640×720](images/screenshots/2026-10-25-equipment-wrap/starter-640.jpg)

Verified the three layouts and wrapped-slot inspection live. Native build,
workspace Clippy, formatting and render-pass lint pass. No mechanics changed;
gameplay tests were not rerun. Reference typography, portrait badges and outdoor
zone art remain unfinished.

## 2026-10-25 — Starter class grid

The wide starter now uses the intention screenshot's three-by-three class grid,
with proportional icon sizing, centered labels and class-colored selection.
Compact screens keep a wrapping row above the hero panel.

![Three-by-three class grid](images/screenshots/2026-10-25-class-grid/mage-1280.jpg)

![Grid at the 1100-pixel layout boundary](images/screenshots/2026-10-25-class-grid/mage-1100.jpg)

![Compact selection at 640×720](images/screenshots/2026-10-25-class-grid/warrior-640.jpg)

Verified class switching, label centering and fit live at 1280×720, 1100×720 and
640×720. Native build, workspace Clippy, formatting and render-pass lint pass;
no gameplay rules changed. Typography and portrait badges remain open. The
1100-pixel capture also exposes an existing stretched last equipment slot when
its row wraps; that layout needs a follow-up.

## 2026-10-25 — Reliable name typing

Starter renaming and character creation now process keyboard events in order.
Rapid letters, repeated keys and Backspace no longer collapse into one frame's
unordered key set; capitalization comes from OS text. Tab switches fields before
the next character, and Enter finishes editing without starting a run.

![Rapid typing saved Leena correctly](images/screenshots/2026-10-25-ordered-names/starter-1280.jpg)

![Anna Moon created through the same input path](images/screenshots/2026-10-25-ordered-names/created-1280.jpg)

Verified both flows live at 1280×720. Two regressions cover same-batch ordering,
repeated letters/deletion, Tab/Enter, release events, filtering and the 16-character
limit. Native build, workspace Clippy, formatting and render-pass lint pass.
This fixes input reliability; the reference's field styling and typography remain open.

## 2026-10-25 — Starter name editing

The starter Hero panel now supports an explicit name draft with Save/Cancel,
Enter/Escape and Tab between fields. Drafts belong to the selected hero;
blank names are rejected. Saving with Enter no longer also starts a run.

![Inline name editor](images/screenshots/2026-10-25-starter-names/editor-1280.jpg)

![Name editor at 640×720](images/screenshots/2026-10-25-starter-names/editor-640.jpg)

![Saved name after restarting](images/screenshots/2026-10-25-starter-names/persisted-640.jpg)

Verified Save, Cancel, Escape, blank rejection, Enter isolation, narrow layout
and restart persistence live. Draft isolation regression, native build, workspace
Clippy and formatting pass. The existing key-state text handler can lose/reorder
rapid input; replacing it with ordered text events remains an immediate follow-up.
The editor uses existing Armory controls; reference typography remains unfinished.

## 2026-10-25 — Specialization headings

Talent-tree headings now show their installed specialization icons and align
point totals opposite the name, matching the reference's structure. Icons resolve
through TalentTab and SpellIcon; no artwork is bundled. Missing art retains the
text heading.

![Mage specialization headings](images/screenshots/2026-10-25-talent-headings/mage-1280.jpg)

![Paladin headings match the reference icons](images/screenshots/2026-10-25-talent-headings/paladin-1280.jpg)

![Paladin headings at 640×720](images/screenshots/2026-10-25-talent-headings/paladin-640.jpg)

Verified Mage/Paladin headings live, Paladin at 640×720, and the text-only fallback
for custom Paladin milestones. The catalog regression covers icon-ID resolution,
missing IDs and truncated input. Native build, workspace Clippy, formatting and
render-pass lint pass. Typography and custom milestone artwork remain unfinished;
this does not validate all classes' talent effects.

## 2026-10-25 — Illustrated talent milestones

Talent milestone rows now include their matching installed spell icons, following
the intention screenshot. Earned icons brighten; locked icons dim with their text.
Selecting the icon still shows the milestone description. Custom milestones
without a matching vanilla talent retain text-only fallback.

![Mage milestone icons and detail inspection](images/screenshots/2026-10-25-milestone-icons/mage-1280.jpg)

![Milestones at 640×720](images/screenshots/2026-10-25-milestone-icons/mage-640.jpg)

![Scrolled Frost milestones at 640×720](images/screenshots/2026-10-25-milestone-icons/frost-640.jpg)

Verified all nine Mage icons, bright/dim states, icon inspection and narrow
scrolling live. Native build, workspace Clippy, formatting and render-pass lint
pass. No gameplay rules changed; gameplay suites were not rerun. Tree-heading
icons, typography and custom Paladin milestone artwork remain unfinished;
other classes and missing-art fallback still need live visual acceptance.

## 2026-10-25 — Named talent prerequisites

Blocked talent purchases and refunds now identify the dependency and required
rank: “Shatter requires Improved Frost Nova rank 2.” A named prerequisite no
longer reports a misleading row-point refund warning.

![Refund explains the named prerequisite](images/screenshots/2026-10-25-talent-prerequisites/named-refund-blocker.jpg)

![One prerequisite rank is insufficient](images/screenshots/2026-10-25-talent-prerequisites/partial-prerequisite-blocked.jpg)

![Completing the prerequisite enables Shatter](images/screenshots/2026-10-25-talent-prerequisites/prerequisite-complete.jpg)

Verified live at 1280×720 against the installed talent catalog: missing/partial
prerequisites reject spending, full rank permits it, dependent-first refunds
work, and trained Shatter protects its prerequisite. The saved build survived
restart. The targeted DBC prerequisite/refund regression test, native build,
workspace Clippy, formatting and render-pass lint pass.
The talent reference remains the layout guide; typography, tree-heading icons
and milestone icons still differ. Higher-tier and cross-class live checks remain.

## 2026-10-25 — Talent refund acceptance

Verified the Mage Frost tree live at 1280×720: five first-row points unlock the
second row; spending there prevents a supporting first-row refund. Refunding
the dependent point first allows the earlier refund and returns both points.
Trying to buy the now-locked second row leaves ranks and available points intact.

![Supporting refund blocked](images/screenshots/2026-10-25-talent-refunds/supporting-refund-blocked.jpg)

![Dependent-first refunds return two points](images/screenshots/2026-10-25-talent-refunds/dependent-first-refund.jpg)

![Locked row rejects spending](images/screenshots/2026-10-25-talent-refunds/locked-row-spend-blocked.jpg)

This is an acceptance checkpoint using an isolated save and the existing build;
no game code changed or test suites were rerun. Named prerequisite chains and
higher-tier builds still need live checks. The illustrated tree remains aligned
with the talent reference; this checkpoint verifies interactions, not new styling.

## 2026-10-25 — Zone details expand in place

Selecting a starter zone expands its description inside that row, following the
main-menu reference. The previous zone collapses. The selected name is larger,
its lock reason is highlighted, and its artwork adapts to the expanded shape.

![Warsong details inside its selected row](images/screenshots/2026-10-25-zone-expansion/warsong-1280.jpg)

![Available Elwynn expands without a lock message](images/screenshots/2026-10-25-zone-expansion/elwynn-1280.jpg)

![Compact selection collapses Elwynn and expands Warsong](images/screenshots/2026-10-25-zone-expansion/warsong-640.jpg)

Verified available/locked selection, text wrapping and artwork cropping at
1280×720 and 640×720. Native build, workspace Clippy, formatting and render-pass
lint pass. Gameplay rules and catalog descriptions are unchanged; gameplay tests
were not rerun. Outdoor artwork and complete zone-content acceptance remain open.

## 2026-10-25 — Illustrated instance zone rows

Starter rows now show installed loading artwork for Warsong Gulch, Arathi Basin,
Scarlet Monastery, Alterac Valley and Naxxramas. Artwork follows the map catalogs,
crops to the row's current shape and stays behind readable names and lock reasons.
A gold border identifies the selected zone. Outdoor illustrations remain unfinished;
shared continent loading art is not substituted for distinct zones.

![Starter with instance zone banners](images/screenshots/2026-10-25-zone-art/starter-1280.jpg)

![Artwork clicks select Warsong while retaining its lock](images/screenshots/2026-10-25-zone-art/warsong-selected-1280.jpg)

![Resized banners and Naxxramas selection at 640×720](images/screenshots/2026-10-25-zone-art/naxxramas-selected-640.jpg)

Verified runtime artwork, wide/narrow cropping, lock text and banner clicks at
1280×720 and 640×720. Native build, workspace Clippy, formatting and render-pass
lint pass. No gameplay rules changed; gameplay test suites were not rerun.
Artwork is read from the player's data, not bundled. This does not establish
world-content readiness or complete the reference's twelve illustrated zones.

## 2026-10-25 — Starter appearance navigation

The starter's Look button opens Character's appearance controls for the selected
hero. Escape returns to the starter with the same hero, gear and zone selection.
The older creation workflow remains separate.

![Look opens Character with the selected hero](images/screenshots/2026-10-25-look-navigation/look-opens-character.jpg)

![Escape returns to the starter](images/screenshots/2026-10-25-look-navigation/return-to-starter.jpg)

Verified the button and return path live at 1280×720 with an isolated Mage save.
Native build, formatting and diff checks pass. This is a navigation-only change;
no gameplay rules changed and the gameplay test suites were not rerun.

## 2026-10-25 — Human racial benefits

Implemented the starter reference's Human rules: Perception grants +15% damage
for 10 seconds every 30 seconds (first activation at 30); Human Spirit adds 10%
run experience and Diplomacy adds 20% gold. The starter uses the same values as
gameplay. Other races explicitly show that their rules remain undefined.

![Human benefits on the starter](images/screenshots/2026-10-25-human-racials/human-1280.jpg)

![Other races remain explicitly pending](images/screenshots/2026-10-25-human-racials/gnome-pending-1280.jpg)

![Human benefits remain readable at 640×720](images/screenshots/2026-10-25-human-racials/human-640.jpg)

Validation: 101 core tests, 183 Survivors app tests, native build, workspace
Clippy, formatting and render-pass lint pass. Tests cover activation boundaries,
stat/legendary stacking and hero/companion pickups. Live checks cover starter
text and race switching, not combat timing or effects. Existing nearest-coin
rounding applies to each gold award. Other racial rules and Perception's visible
combat feedback remain open.

## 2026-10-25 — Live hero portrait on the starter

The starter now shows the selected hero's saved body and appearance beside their
name and race, using the same model renderer as Character. The portrait scales
with its panel and supports drag-to-turn. Name and look controls wrap as needed.
This brings the hero section closer to the [main-menu reference](intentions/survivors_main_menu.png).

![Warrior starter portrait](images/screenshots/2026-10-25-starter-portrait/warrior-1280.jpg)

![Switching class refreshes the saved Mage appearance](images/screenshots/2026-10-25-starter-portrait/mage-1280.jpg)

![Proportional portrait in the scrolled 640×720 starter](images/screenshots/2026-10-25-starter-portrait/mage-640.jpg)

![Drag-to-turn keeps the Mage and staff framed](images/screenshots/2026-10-25-starter-portrait/mage-turned-640.jpg)

Verified with isolated Warrior/Mage saves at 1280×720 and 640×720: class switching,
Character round trip, scroll clipping and portrait rotation. Native build,
181 Survivors app tests without default features, workspace Clippy, formatting
and render-pass lint pass. Zone artwork, decorative typography, faction/class
portrait badges and racial bonuses remain open; this is not full starter acceptance.

## 2026-10-25 — Starter equipment strip and narrow layout

The starter replaces nine equipment text rows with worn-item icons, quality borders
and labeled empty slots, closer to the [main-menu reference](intentions/survivors_main_menu.png).
Selecting a slot opens its worn piece in Collection for names, stats and equipment
controls. Below 1100 UI units, Class, Hero and Zone stack in one scrollable surface;
the entry controls remain visible. Starter scrolling survives a Collection round trip.

![Wide starter with worn equipment icons](images/screenshots/2026-10-25-starter-equipment/starter-1280.jpg)

![Selecting shoulders opens the correct worn item](images/screenshots/2026-10-25-starter-equipment/inspect-shoulders.jpg)

![Stacked starter at 640×720](images/screenshots/2026-10-25-starter-equipment/starter-640.jpg)

![Scrolling reaches the complete zone list](images/screenshots/2026-10-25-starter-equipment/zones-640.jpg)

![Zone position retained after Collection and Escape](images/screenshots/2026-10-25-starter-equipment/zone-scroll-restored.jpg)

Live checks used an isolated Warrior account at 1280×720 and 640×720. Validation:
181 Survivors app tests without default features, native build, workspace Clippy,
formatting and render-pass lint pass. The reference's hero portrait, illustrated
zone rows, decorative typography and racial bonuses remain separate work.

## 2026-10-25 — Starter finale follows the encounter schedule

The starter now names the actual final boss and scheduled time: Elwynn shows
**Fedfennel · 16:00**, replacing the legacy Hogger label. Starter and Drop Tables
share the combat schedule's finale lookup, including simultaneous final bosses.
Locked zones continue to show their progression requirement beneath Enter.

![Elwynn's actual scheduled finale](images/screenshots/2026-10-25-starter-finale/elwynn-finale.jpg)

![Locked Duskwood retains its entry requirement](images/screenshots/2026-10-25-starter-finale/locked-zone.jpg)

Live checks used an isolated account at 1280×720. Validation: 100 core tests,
181 Survivors app tests without default features, native build, workspace Clippy,
formatting and render-pass lint pass. A regression checks finale names/times against
combat spawns for every map. This changes menu copy, not encounters or zone readiness.

## 2026-10-25 — Page-local action feedback

Armory navigation now clears the previous page's action message and text-entry
focus. Talent refunds still show feedback on Talents, but that message no longer
appears on Character or the starter after Escape. Pending reward purchases remain
staged; the run-entry redirect sets its own Confirm/Undo instruction.

![Talent refund feedback on its own page](images/screenshots/2026-10-25-page-feedback/talent-feedback.jpg)

![Character after navigation, without stale Talent feedback](images/screenshots/2026-10-25-page-feedback/character-clean.jpg)

![Starter after Escape, without stale Talent feedback](images/screenshots/2026-10-25-page-feedback/escape-clean.jpg)

Verified live at 1280×720 with an isolated account. Native build, 181 Survivors app
tests without default features, workspace Clippy, formatting and render-pass lint
pass. The starter's legacy finale label still needs to use the encounter schedule;
the capture does not establish zone-content completion.

## 2026-10-25 — Saved Talent builds: live acceptance

Verified the [Talent interface](intentions/survivors_talent_tab.png) with an isolated
Warrior/Mage account at 1280×720. Warrior specs retain distinct Arms/Fury allocations;
Mage uses its own third spec. Both heroes restore their selected build after an app
restart. Resetting Warrior spec 2 refunds its points without clearing spec 1 or Mage's
saved build. Save-file readback agrees with the interface. This is a validation
checkpoint, with no gameplay or balance changes.

![Warrior first build: three Arms points](images/screenshots/2026-10-25-talent-specs/warrior-spec1.jpg)

![Warrior second build: two Fury points](images/screenshots/2026-10-25-talent-specs/warrior-spec2.jpg)

![Mage third build: one Frost point](images/screenshots/2026-10-25-talent-specs/mage-spec3.jpg)

![Mage selected build restored after restart](images/screenshots/2026-10-25-talent-specs/mage-restarted.jpg)

![Warrior selected build restored after switching back](images/screenshots/2026-10-25-talent-specs/warrior-restored.jpg)

![Reset affects only Warrior's second spec](images/screenshots/2026-10-25-talent-specs/reset-spec2.jpg)

![First spec remains usable after resetting the second](images/screenshots/2026-10-25-talent-specs/spec1-after-reset.jpg)

All-class combat effects and deeper prerequisite/refund behavior still need their
own acceptance. A visual issue remains: Talent action messages carry into Character
when changing pages. Narrow per-spec scroll restoration has automated coverage but
was not exercised in this wide-screen session.

## 2026-10-25 — Responsive Character and saved appearances

Character now stacks its proportional hero preview, appearance controls, equipment
and item details on narrow screens. Wide screens retain the
[reference's](intentions/survivors_character_tab.png) three-pane composition.
Appearance edits and tab return preserve Character scrolling independently of
Collection and Drop Tables. Switching to a hero with a saved appearance no longer
shows the obsolete “Open Look” prompt.

![Wide Character composition](images/screenshots/2026-10-25-character-responsive/character-1280.jpg)

![Proportional Character preview at 640×720](images/screenshots/2026-10-25-character-responsive/character-640.jpg)

![Hero management at narrow width](images/screenshots/2026-10-25-character-responsive/manage-heroes-640.jpg)

![Saved appearance restored after hero switching and restart](images/screenshots/2026-10-25-character-responsive/appearance-restored-640.jpg)

![Appearance value and scroll retained after Collection tab return](images/screenshots/2026-10-25-character-responsive/appearance-tab-return-640.jpg)

![Character preview at 640×900](images/screenshots/2026-10-25-character-responsive/character-640x900.jpg)

![Appearance controls at 640×900](images/screenshots/2026-10-25-character-responsive/appearance-640x900.jpg)

Actual native captures use an isolated two-hero account. Live checks covered body
and feature edits, Warrior/Mage switching, tab return and restart persistence.
Validation: 181 Survivors app tests without default features, native incremental
build, workspace Clippy, formatting and render-pass lint pass. Final typography,
wide-column scrolling and per-hero/spec Talent restoration remain open.

## 2026-10-25 — Responsive Drop Tables

Drop Tables now wraps zone choices and stacks encounter sources, item details and
equipment at narrow widths. Item selection, equip actions and tab return retain the
page's independent scroll position. Wide encounter rows put the boss/time beside its
item, following the [reference](intentions/survivors_drop_tables_tab.png); zone choices
show quality and collection counts. Finale labels use the actual combat schedule.

![Wide encounter rows and quality counts](images/screenshots/2026-10-25-drop-tables-responsive/drops-1280.jpg)

![Wrapped zone choices at 640×720](images/screenshots/2026-10-25-drop-tables-responsive/drops-640.jpg)

![Lower source selection retains scroll and updates item details](images/screenshots/2026-10-25-drop-tables-responsive/selected-source-640.jpg)

![Equipped item and scroll survive a Collection round trip](images/screenshots/2026-10-25-drop-tables-responsive/equipped-tab-return-640.jpg)

![Unfinished zone tables remain explicit](images/screenshots/2026-10-25-drop-tables-responsive/unfinished-zone-640.jpg)

Live checks used an isolated synthetic Warrior account at 1280×720 and 640×720.
All twelve zone choices and each zone's source actions are covered in both layouts;
browsing does not select the next run's zone. Validation: 99 core tests, 181 Survivors
app tests without default features, native build, workspace Clippy, formatting and
render-pass lint pass. Missing zone content, final typography and wide-column scroll
restoration remain open; this checkpoint does not claim those are complete.

## 2026-10-25 — Distinct gear stat profiles

Gear slots now have distinct stat progressions: chest armor starts with health and
armor, boots with movement and pickup, wrists with critical chance and cooldown
reduction. Every higher current band improves retained stats and adds one; duplicate
copies still add no power. Gloves retain the requested examples. This changes existing
items' bonuses while preserving ownership and selected equipment. See the
[initial tuning and remaining raid/quality work](SURVIVORS-GEAR-STATS.md).

![Common chest: two stats](images/screenshots/2026-10-25-gear-profiles/common-chest-1280.jpg)

![Epic chest: five stats](images/screenshots/2026-10-25-gear-profiles/epic-chest-1280.jpg)

![Equipping Epic chest updates the worn item and totals](images/screenshots/2026-10-25-gear-profiles/equipped-epic-1280.jpg)

![Boots emphasize movement and pickup](images/screenshots/2026-10-25-gear-profiles/common-boots-1280.jpg)

![Wrists show critical chance and clearly labeled cooldown reduction](images/screenshots/2026-10-25-gear-profiles/common-wrist-1280.jpg)

Actual 1280×720 captures use an isolated synthetic account and retain the
[Collection reference's](intentions/survivors_collection_tab.png) green item details
and equipment totals. Typography, final balance, Poor drops, raid epics and distinct
raid set bonuses remain open. Validation: 98 core rules tests, 180 Survivors app
tests without default features (including mixed-equipment combat stats and downgrade),
native incremental build (1m32s), workspace Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Codex jewelry icons and selection

Legendary amulets now read as jewelry with an associated spell badge, following the
[Codex reference](intentions/survivors_codex_tab.png). Shared jewelry uses aligned
category/count rows. Selecting an entry highlights its cell and updates a larger
detail image without rebuilding or resetting the catalog. Legendary titles are
formatted as spell names instead of exposing internal underscores.

![Catalog with synthetic discovered and undiscovered entries](images/screenshots/2026-10-25-codex-icons/catalog-1280.jpg)

![Legendary selection and spell-specific bonuses](images/screenshots/2026-10-25-codex-icons/legendary-1280.jpg)

![Undiscovered legendary remains dim in the catalog](images/screenshots/2026-10-25-codex-icons/undiscovered-1280.jpg)

![Shared ring selection replaces the legendary badge](images/screenshots/2026-10-25-codex-icons/ring-1280.jpg)

![Lower rows remain selected and scrolled at 640×720](images/screenshots/2026-10-25-codex-icons/lower-selection-640.jpg)

Live checks used an isolated synthetic account with four discoveries, covering badge
clicks, unseen entries, shared jewelry and narrow scrolling. Legendary amulets share
one verified necklace silhouette for now; distinct artwork, catalog expansion,
Mark of the Champion's missing icon and reference typography remain open. These
captures demonstrate presentation, not completion of content or progression.
Validation: 183 Survivors app tests, native build, workspace test compilation,
Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Responsive Collection page

Collection now stacks its log, item details, equipment and missing drops on narrow
screens. Labeled tiles wrap without shrinking into overlapping headings or stretching
the final tile. Wide screens retain the [reference](intentions/survivors_collection_tab.png)
three-pane layout. Narrow scrolling survives item actions and tab round trips.

![Readable slots at 640×720](images/screenshots/2026-10-25-collection-responsive/collection-640.jpg)

![Wrapped tiles at 640×900 keep their size](images/screenshots/2026-10-25-collection-responsive/collection-640x900.jpg)

![Equipping retains the detail scroll position](images/screenshots/2026-10-25-collection-responsive/equipped-640.jpg)

![Independent appearance and scroll position survive a tab round trip](images/screenshots/2026-10-25-collection-responsive/tab-return-640.jpg)

![Wide layout and saved helm selection after restart](images/screenshots/2026-10-25-collection-responsive/collection-1280.jpg)

An isolated Warrior account verified selection, Equip, independent look selection,
scrolling through equipment, tab return and restart persistence. Empty/full catalogs
for all nine classes build every slot in both layouts. Validation: 182 Survivors app
tests, native build, workspace test compilation, Clippy, formatting and render-pass
lint pass. Reference typography, richer item stats and content acceptance remain open;
this checkpoint resolves the narrow Collection crowding recorded below.

## 2026-10-25 — Shared Armory header

Armory pages now share the reference's left-aligned title, wrapping navigation and
class-colored collection summary. Removed the duplicate Collection summary and
generic hero-selection hint. The starter retains its game title.

![Character with the shared Armory header](images/screenshots/2026-10-25-armory-header/character-1280.jpg)

![Collection without a duplicate class summary](images/screenshots/2026-10-25-armory-header/collection-1280.jpg)

![Switching classes updates the collection context](images/screenshots/2026-10-25-armory-header/codex-paladin-1280.jpg)

![Header wraps at 640×720; Collection slot labels still need reflow](images/screenshots/2026-10-25-armory-header/collection-640-open-layout-gap.jpg)

Live checks covered populated Warrior and empty Paladin collections, tab navigation,
and Done returning to the starter. The narrow capture records an **open Collection
layout defect**: crowded slot headings and compressed tiles; it is not whole-page
responsive acceptance. Typography and tab styling still need reference refinement.
Validation: 182 Survivors app tests, native build, workspace test compilation,
Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Undo individual pending Rewards

Perk and Companion cards now show pending counts and a minus control that removes
one pending upgrade while keeping the rest. Pending resets have their own Undo.
Removing a prerequisite cannot silently move an instinct to another companion or
invalidate later purchases: the draft stays intact until dependencies are undone.
Confirmed purchases retain the existing reset/refund rules.

![Three upgrades staged across two cards](images/screenshots/2026-10-25-reward-minus/three-pending-1280.jpg)

![Removing one rank returns 25 points and retains Banish](images/screenshots/2026-10-25-reward-minus/one-removed-1280.jpg)

![Per-card undo remains reachable at 640×720](images/screenshots/2026-10-25-reward-minus/pending-640.jpg)

![Companion upgrades use the same pending controls](images/screenshots/2026-10-25-reward-minus/companion-pending-1280.jpg)

![Pending resets can be undone independently](images/screenshots/2026-10-25-reward-minus/reset-pending-1280.jpg)

Isolated-account checks verified rank repricing, narrow-screen removal, confirmation
of only the retained 15-point perk, and undo of a reset and Long Leash without saving
either. Dependency and companion-destination cases have regression coverage.
Validation: 182 Survivors app tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass. Reference typography/control placement still
need refinement; these controls edit pending purchases only.

## 2026-10-25 — Tabard cosmetic gallery

Tabards now use illustrated selection tiles based on the Companion Looks reference,
with clear **Worn**, **Wear**, and **Remove tabard** states. Removed client display IDs
from the player-facing page. Selection stays immediate, account-wide and stat-free.

![Guild Tabard selected without spending points](images/screenshots/2026-10-25-tabards/worn-1280.jpg)

![Removing a tabard in the narrow layout](images/screenshots/2026-10-25-tabards/removed-640.jpg)

![The worn selection persists after restart at 640×720](images/screenshots/2026-10-25-tabards/restart-640.jpg)

An isolated account verified Wear/Remove save changes, unchanged reward balance,
narrow scrolling and restart persistence. The catalog still contains one verified
tabard; more cosmetics and character-model previews remain open. Validation: 179
Survivors app tests, native build, workspace test compilation, Clippy, formatting
and render-pass lint pass.

## 2026-10-25 — Illustrated achievement cards and progress

Rebuilt Achievements around the [summary](intentions/survivors_achievements_tab_summary.png)
and [zone](intentions/survivors_achievement_tab_zones.png) references: account points,
category navigation, recently earned cards, and ranked “Almost there” progress.
Cards explain their actual completion rules and known Companion rewards. Placeholder
feats remain impossible to earn. Each category retains its session scroll position.

![Achievement summary and closest goals](images/screenshots/2026-10-25-achievement-cards/summary-1280.jpg)

![Zone clears precede all-class champion goals](images/screenshots/2026-10-25-achievement-cards/zones-1280.jpg)

![Lower zone goals remain reachable after a Collection round trip](images/screenshots/2026-10-25-achievement-cards/zones-return-1280.jpg)

![Unachievable placeholder feats retain their planned rewards](images/screenshots/2026-10-25-achievement-cards/feats-1280.jpg)

![Navigation stacks at 640×720](images/screenshots/2026-10-25-achievement-cards/navigation-640.jpg)

![Narrow summary content remains reachable by scrolling](images/screenshots/2026-10-25-achievement-cards/summary-640.jpg)

Captures use an isolated synthetic account. Live checks covered category selection,
lower-card reachability, navigation scroll restoration and resizing without restart.
Category icons currently substitute for bespoke achievement art; the decorative crest
and typography still need refinement. Earned dates are not stored, so cards say “Earned”.
Validation: 97 core tests, 179 Survivors app tests, native build, workspace test
compilation, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Placeholder feats and locked Companion looks

Added **Deathless Hour** and **no u** under Feats, marked “Criteria pending”. As
requested, neither can currently be earned or award points. Panda Cub and Sapphiron
now appear in the [reference's Companion catalog](intentions/survivors_rewards_tab_companions.png),
locked behind those entries. Existing saves and selection indices remain compatible.

![Placeholder achievements under Feats](images/screenshots/2026-10-25-placeholder-feats/feats-1280.jpg)

![Panda Cub and Sapphiron remain locked](images/screenshots/2026-10-25-placeholder-feats/locked-looks-1280.jpg)

![All thirteen looks reflow at 640×720](images/screenshots/2026-10-25-placeholder-feats/locked-looks-640.jpg)

Live checks confirmed both locked tiles reject selection. Model/icon mappings resolve
from local data; Sapphiron's companion scale and animations still need validation
before its future unlock can be enabled. Achievement-page styling remains unfinished.
Validation: 97 core tests, 178 Survivors app tests, native build, workspace test
compilation, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Confirm and Undo reward purchases

Perk/Companion purchases and perk resets now preview together, following the
[Rewards reference](intentions/survivors_rewards_tab_companions.png). Confirm applies
the whole batch; Undo discards it. Pending choices survive tab changes, require
resolution before run entry, and are discarded on app close. Cosmetics stay immediate.

![Two perks pending, with updated ranks and remaining points](images/screenshots/2026-10-25-reward-confirm/pending-1280.jpg)

![Undo restores the original balance and ranks](images/screenshots/2026-10-25-reward-confirm/undo-1280.jpg)

![Confirmed purchases saved to the account](images/screenshots/2026-10-25-reward-confirm/confirmed-1280.jpg)

![Companion upgrades share the same pending purchase flow](images/screenshots/2026-10-25-reward-confirm/companions-pending-1280.jpg)

![Confirm and Undo remain reachable at 640×720](images/screenshots/2026-10-25-reward-confirm/pending-640.jpg)

![Restart retains confirmed perks and drops pending Companion upgrades](images/screenshots/2026-10-25-reward-confirm/restart-640.jpg)

Live synthetic-account checks verified an unchanged save while staging, Undo, the
30-point confirmed spend, and restart behavior. 177 Survivors tests, native build,
workspace test compilation, Clippy, formatting and render-pass lint pass. Per-card
minus controls and closer typography remain open.

## 2026-10-25 — Expanded Companion catalog and Veteran unlock

Added six looks from the [Companion reference](intentions/survivors_rewards_tab_companions.png):
Crimson Macaw, Great Horned Owl, Tree Frog, Sprite Darter, Zergling and Undercity Cockroach.
Existing selection indices stay stable. Corrected Black Whelpling's missing display
ID to the local pet-sized black whelpling. Icons remain generic for several critters.

![Eleven looks with Chicken locked before Veteran](images/screenshots/2026-10-25-companion-catalog/catalog-1280.jpg)

![New Crimson Macaw selected](images/screenshots/2026-10-25-companion-catalog/macaw-1280.jpg)

![Expanded catalog at 640×720](images/screenshots/2026-10-25-companion-catalog/catalog-640.jpg)

![Chicken wearable after the synthetic account reaches Veteran](images/screenshots/2026-10-25-companion-catalog/veteran-1280.jpg)

New accounts unlock Westfall Chicken with Veteran (100 runs). Existing saves retain
prior unrestricted access through a persisted migration flag; this grants no points.
Live isolated-fixture checks covered blocked selection, Macaw selection, narrow reflow
and the 100-run achievement unlock. These captures validate menus, not in-world pet
animation. Panda Cub and Sapphiron await definitions for their pictured achievement gates.

Validation: 174 Survivors tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass.

## 2026-10-25 — Illustrated Companion appearances

Looks now uses illustrated, wrapping tiles with a gold selection border and green
“Worn” state. Added the free Companion card and restored the [reference's card order](intentions/survivors_rewards_tab_companions.png),
with carrier, leash and movement icons loaded from the local patch chain.

![Base companion and upgrade cards](images/screenshots/2026-10-25-companion-looks/companion-1280.jpg)

![First companion wearing Black Whelpling](images/screenshots/2026-10-25-companion-looks/first-whelpling-1280.jpg)

![Second companion independently wearing Squirrel](images/screenshots/2026-10-25-companion-looks/second-squirrel-1280.jpg)

![Saved looks after restart at 640×720](images/screenshots/2026-10-25-companion-looks/looks-640.jpg)

Live isolated-account checks: first-slot Whelpling, second-slot Squirrel then Kingsnake;
instincts and points remained unchanged, and both looks survived restart. Checked
1280×720 and 640×720 scrolling/reflow. Native build, workspace Clippy, formatting and
render-pass lint pass. The five existing looks are illustrated; expanding to the reference's
achievement-gated catalog, denser spacing and Confirm/Undo purchases remain unfinished.

## 2026-10-25 — Companion cards and upgrade previews

Companion upgrades now use illustrated two-column cards with rank pips, costs and
green inline comparisons derived from gameplay values. The [Companion intention](intentions/survivors_rewards_tab_companions.png)
remains the reference. Appearance selection stays separate from instinct assignment.

![Companion upgrade cards at 1280×720](images/screenshots/2026-10-25-companion-cards/companions-1280.jpg)

![Quick Paws purchase updates its rank and both effects](images/screenshots/2026-10-25-companion-cards/purchased-1280.jpg)

![Instincts assigned to first and second companions](images/screenshots/2026-10-25-companion-cards/instincts-1280.jpg)

![Narrow cards and appearance controls at 640×720](images/screenshots/2026-10-25-companion-cards/companions-640.jpg)

Checked live with an isolated collected account: Quick Paws charged 15 points,
second companion 40, and each instinct 30; Gold-sniffer filled the first slot and
Wisp-herder the second. Narrow scrolling reaches every card and the Looks controls.
Native build, workspace Clippy, formatting and render-pass lint pass. Captures are
Retina window images. Remaining design work: base-companion card, closer icon choices,
illustrated/expanded Looks catalog and staged purchases. Rune spawning remains unavailable.

## 2026-10-25 — Rewards scroll continuity

Rewards preserves its session scroll position when purchases rebuild the page and
when returning from another Armory tab. Perks, Companions and Tabards keep separate
positions; a category change cannot misattribute the previous page's offset.

![Scrolled Perks before purchase at 640×720](images/screenshots/2026-10-25-reward-scroll/before-purchase-640.jpg)

![Purchase updates the card without jumping to the top](images/screenshots/2026-10-25-reward-scroll/after-purchase-640.jpg)

![Returning from Collection restores the same position](images/screenshots/2026-10-25-reward-scroll/returned-640.jpg)

Live validation used an isolated collected Warrior at 640×720. Second Thoughts
charged 15 points and updated in place; a Collection round trip retained the view.
The layout remains based on the [Rewards intention](intentions/survivors_rewards_tab_perks.png).
Cross-category bookkeeping has an ECS regression test; live Companion/Tabard scroll
checks remain open. Positions reset when the application exits and are not save data.
All 172 Survivors app tests, workspace Clippy, formatting, render-pass lint and the
native build pass.

## 2026-10-25 — Inline Perk upgrade previews

All six Perk cards now show the current and next effect inline in green, with the
unit once: for example, “20 » 40% more talent experience” and “3 » 4 cards.” Capped
perks show only the current benefit. Skip explains its unlock; Keen Eye explicitly
uses percentage points. Preview values share the core reward mechanics. Reset-count
wording is also corrected. The [Rewards intention](intentions/survivors_rewards_tab_perks.png)
remains the layout reference; inline comparisons follow the requested presentation.

![Green inline Perk previews](images/screenshots/2026-10-25-reward-previews/perks-1280.jpg)

![Veteran's Lessons after the first purchase](images/screenshots/2026-10-25-reward-previews/rank-one-1280.jpg)

![Maximum rank and narrow preview wrapping](images/screenshots/2026-10-25-reward-previews/capped-640.jpg)

Live isolated-account purchases charged 20 then 40 points, updated the preview and
removed the arrow at the cap. Checked 1280×720 and a live resize to 640×720 (Retina
captures including title bar). All 96 core tests, workspace Clippy, formatting,
render-pass lint and native build pass. Companion previews, staged purchases and
Rewards scroll continuity remain unfinished.

## 2026-10-25 — Rewards sidebar and illustrated Perks

Rewards now uses the [intention's sidebar and two-column Perk cards](intentions/survivors_rewards_tab_perks.png):
account currency, achievement totals, category navigation and perk reset sit together;
cards show icons, earned rank pips, costs, effects and max-rank state. At narrower
widths the sidebar stacks above the cards in one scrollable page. Purchases still
apply immediately and are labelled as such; Confirm/Undo staging remains unfinished.

![Rewards and Perk cards at 1280×720](images/screenshots/2026-10-25-rewards-cards/perks-1280.jpg)

Live isolated-account check: Second Thoughts charged 15 points (275→260), filled
one pip and offered rank two for 25 points. Reset refunded all 15 points and consumed
one reset. Existing purchase/save logic is unchanged.

![Purchased rank and updated currency](images/screenshots/2026-10-25-rewards-cards/purchased-1280.jpg)

![Perks after reset](images/screenshots/2026-10-25-rewards-cards/reset-1280.jpg)

![Stacked Rewards layout after live resize to 640×720](images/screenshots/2026-10-25-rewards-cards/perks-640.jpg)

Native build, workspace Clippy, formatting and render-pass lint pass. Captures use
the synthetic collected Warrior account and include the Retina title bar. Companion
and Tabard content now share the frame but still need their illustrated layouts.
Green current→next effect previews, staged purchasing, scroll continuity across
purchases, the singular reset label and typography remain UI acceptance work.

## 2026-10-25 — Taller Talent presentation

Talent panels now follow the available page height, giving the illustrated trees
and seven talent rows more space in line with the
[Talent reference](intentions/survivors_talent_tab.png). Narrow windows retain wrapping
and one page-level scroll area. Tree totals now say “1 point” when appropriate.

![Full-height Talent panels at 1280×720](images/screenshots/2026-10-25-talent-height/talents-1280.jpg)

![One spent point and green preview](images/screenshots/2026-10-25-talent-height/one-point-1280.jpg)

![Wrapped lower panels at 640×720](images/screenshots/2026-10-25-talent-height/wrapped-640.jpg)

Verified with an isolated collected Warrior fixture: spending a point, live resizing,
and scrolling to the lower tree/spec panel. Native build, workspace Clippy, formatting
and render-pass lint pass. Header/milestone icons, typography and the black margins
around the tree artwork still differ from the reference; other-class visual checks
remain open. Captures include the Retina title bar.

## 2026-10-25 — Responsive Talents live validation

Validated the current native UI with an isolated collected Warrior account, against
[the Talent reference](intentions/survivors_talent_tab.png), without a gameplay run.
At 800×720, details wrap beneath three trees. Spending Deflection changes 51→50
available points; resetting restores 51 without losing scroll. Leaving for Collection
and returning to Talents also restores that position.

![Wrapped Talent page at 800×720](images/screenshots/2026-10-25-talent-responsive/trees-800.jpg)

![Spent point and green preview in the scrolled detail panel](images/screenshots/2026-10-25-talent-responsive/spent-scrolled-800.jpg)

![Scroll position after resetting and returning from Collection](images/screenshots/2026-10-25-talent-responsive/tab-restored-800.jpg)

Live preview resizing to 640×720 produces two columns. The lower Protection tree
and spec controls remain reachable. Left-click spending and right-click refunding
Shield Specialization update rank/totals and retain the scrolled view.

![Two-column Talent layout at 640×720](images/screenshots/2026-10-25-talent-responsive/trees-640.jpg)

![Spending in the wrapped Protection tree](images/screenshots/2026-10-25-talent-responsive/protection-spent-640.jpg)

![Refunding in place](images/screenshots/2026-10-25-talent-responsive/protection-refunded-640.jpg)

Screenshots are Retina captures including the title bar. No runtime code changed.
This closes the sampled reflow/scroll interaction checks, not all-class acceptance:
per-spec/per-hero restoration still needs live coverage. Reference header/milestone
icons, full-height tree art and typography remain unfinished; spending one point
also exposes the remaining “1 points” label.

## 2026-10-25 — Starter UI visual checkpoint

Captured build `4dcf72e` at 1280×720 logical resolution (Retina, title bar included),
using the isolated collected fixture: ten heroes, all gear, unspent talents and rewards.
Opened Character → Manage heroes → Talents → Rewards/Companions without a run.
These are current captures; older images below remain attached to their original milestones.

Compared with the [Character reference](intentions/survivors_character_tab.png),
the three-column layout and larger model are present; feature pips, ornamentation
and typography still need work. All ten heroes appear in management.

![Current Character page](images/screenshots/2026-10-25-starter-review/character-720.jpg)

![Ten-hero management view](images/screenshots/2026-10-25-starter-review/manage-heroes-720.jpg)

Talents displays all three trees and the spec panel at this size. Narrow wrapping,
scroll restoration and spend/refund interactions still need live checks.

![Current Talent page](images/screenshots/2026-10-25-starter-review/talents-720.jpg)

Companion destination, costs and effects are visible. The
[Rewards reference](intentions/survivors_rewards_tab_companions.png) remains the target:
illustrated cards, rank pips, appearance icons and a currency/sidebar layout are missing.
Purchases were not exercised in this capture pass.

![Current Companion page](images/screenshots/2026-10-25-starter-review/companions-720.jpg)

## 2026-10-25 — Companion purchase clarity

Rewards now names the companion receiving an instinct, any replaced instinct,
the cost and the effect. Current instincts and locked companion state are visible.
Initial purchases retain the reference's first-then-second assignment; appearance
selection stays separate. Invalid instincts cannot charge points, and locked slots
reject appearance edits. All 171 Survivors tests, Clippy and native build pass.
Initial rendered capture appears in the visual checkpoint above; purchase interactions remain unverified.

## 2026-10-25 — Talent scroll continuity

Talent pages restore their scroll position after rebuilds, with separate session
positions for each hero and spec. Restored offsets wait for layout before clamping
to content bounds. All 170 Survivors tests, workspace Clippy, formatting,
render-pass lint and the native build pass. Live scrolling, spending/refunding,
spec switching and resized-layout verification remain pending.

## 2026-10-25 — Talent panel reflow

Talent trees and the spec/detail panel now wrap at narrower widths instead of
shrinking indefinitely into one row. One page-level scroll area keeps wrapped
panels reachable without nested wheel scrolling. Workspace Clippy, formatting,
render-pass lint and the native build pass (21 seconds). Rendered wrapping,
interaction and screenshot checks remain pending.

## 2026-10-25 — Isolated UI account fixtures

Added fresh, partial and fully collected synthetic accounts for repeatable UI
checks without gameplay. A small script copies each fixture into a new isolated
state folder and prints its launch command; existing folders are never replaced.
The collected fixture includes ten heroes for roster checks. All 168 Survivors
tests, two script isolation tests and workspace Clippy pass. Rendered comparisons,
spent talent specs and purchased reward fixtures remain pending.

## 2026-10-25 — Character preview and hero management

Character now gives the dressed hero a larger central preview with a centered name
and class. Roster, creation, renown and relic controls move into Manage heroes;
selecting or creating a hero returns to the preview. Removed the roster's eight-hero
display limit. 167 Survivors tests, workspace Clippy, formatting, render-pass lint
and the native build pass. Rendered layout and interaction checks remain pending
because the Mac was locked; no new screenshot is claimed.

## 2026-10-25 — Live menu preview controls

Native development builds can now switch production Armory pages and resize the
window from an opt-in local file, without restarting or playing a run. Invalid
edits retain the current view; account progress and name drafts are preserved.
165 Survivors app tests, workspace Clippy and the native build pass. Startup logs
confirm the control was applied; visual switching/resizing remains unverified
because the Mac was locked. [Usage](SURVIVORS-DEVELOPMENT.md#starter-ui-iteration-priority).

## 2026-10-25 — Interface milestone and deferred physics scope

Recorded the current PR's interface/setup acceptance boundary: characters, talents,
gear, progression and achievements, with persistence, responsive pages and fast UI
previews. Box3D integration is a separate phase after this PR is complete and merged.
Documented the existing Avian backend and the separate future server architecture
decision; no physics dependencies or runtime behavior changed.

## 2026-10-25 — Conditional cutaway and starter-page previews

Cutaway activation now checks the camera-to-player sightline and fades reveal
strength over 500 ms of real time. Buildings behind or beside a visible player
stay opaque. Added direct development entry into starter/Armory pages; Collection
was verified without a run and captured in README. 95 core, 165 app and one camera
regression test pass, with Clippy and native build. The reported building scene's
visual fade check remains open. Starter UI takes priority; unlockable game-speed
options are recorded for later.

## 2026-10-25 — Initial enemy obstacle detours

Stalled enemies now plan and follow collision-checked local detours, sharing a
bounded search budget across frames. Routes reset when targets move substantially
or enemies leave play. All 164 app tests pass, including real-physics wall-detour
playback; Clippy and native build pass (78 seconds). Live foliage/indoor routing,
narrow passages and crowded-queue performance remain open.

## 2026-10-25 — Safe distant-enemy placement

Repositioned enemies now respect arena bounds, supported ground and body clearance,
just like new spawns. Missing or obstructed geometry defers repositioning. All 160
Survivors app tests, Clippy and native build pass. Routing around obstacles remains
in progress; this fixes a placement bypass, not pathfinding.

## 2026-10-25 — Blackrock floor investigation

Rejected seven unsupported interior arena candidates. The floor audit now separates
holes from other storeys and flags nearby unsupported samples, including gaps hidden
by a fully supported boundary ring. [Evidence and next layout requirements](SURVIVORS-LOCATIONS.md#blackrock-interior-floor-rejection--2026-10-25)
are recorded. Eight audit tests and Clippy pass; Blackrock remains pending a safe,
connected arena layout.

## 2026-10-25 — Alterac faction-survival candidate

Added ten wave entries and nine bosses ending with Drek'Thar, with PvP rewards
for every class and slot. [Gameplay and card captures](images/screenshots/2026-10-25-alterac-roster/survival-720.jpg)
are linked in README. 94 core and 160 app tests, Clippy and native build pass
(23 seconds). The data audit has three remaining gaps; Alterac stays locked pending
full encounters, traversal, attachments and camera validation.

![Gameplay and card captures](images/screenshots/2026-10-25-alterac-roster/survival-720.jpg)

## 2026-10-25 — Arathi faction-survival candidate

Added a surveyed stables-area arena, ten faction wave entries and nine authored
bosses with Rare rewards across all slots. Duplicate rewards shared with Feralas
preserve existing gear. [Live 720p capture](images/screenshots/2026-10-25-arathi-roster/survival-720.jpg)
is linked in README. 93 core and 159 app tests, Clippy and native build pass
(22 seconds). Arathi stays locked pending traversal, camera and full encounters;
the content audit now reports 85 remaining gaps.

![Live 720p capture](images/screenshots/2026-10-25-arathi-roster/survival-720.jpg)

## 2026-10-25 — Warsong faction-survival candidate

Added Warsong's map, ten wave entries, nine authored bosses and Uncommon drops for
all slots. Cross-zone duplicates preserve gear and appearances. [Live 720p capture](images/screenshots/2026-10-25-warsong-roster/survival-720.jpg)
is linked in README. 92 core and 158 app tests, Clippy and native build pass
(22 seconds). The content audit is down to 86 gaps; Warsong stays locked pending
full traversal, camera and encounter validation.

![Live 720p capture](images/screenshots/2026-10-25-warsong-roster/survival-720.jpg)

## 2026-10-25 — Mage school training and battleground direction

Arcane Instability now improves Mage spell damage and critical chance; Elemental
Precision improves Fire/Frost damage. Both replace generic stat placeholders,
with scoped effects and accurate previews. Battleground zones are confirmed as
horde survival with faction-themed encounters. Remaining Mage talents and
battleground content are still in progress. 91 core and 157 app tests, DBC audit,
Clippy and native build pass (14 seconds); live validation remains open.

## 2026-10-25 — Long upgrade-card layout validation

Added an opt-in card inspection mode that holds level-up and chest choices.
Frost Nova's full defensive description fits at 720p and 1024×768, with single-unit
green comparisons; selecting after resize grants rank two. [Screenshots](images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)
are in the README. Clippy and native build pass (42 seconds); live defensive timing remains open.

![Screenshots](images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)

## 2026-10-25 — Mage Frost defenses and milestones

Frost milestones now add Frostbolt targeting, Frost critical damage and a Nova
barrier. Ice Block and Ice Barrier talents add brief Nova immunity and an absorb,
with no stacking or replacement of stronger defenses. Cards and tooltips explain
the benefits. 90 core and 156 app tests, DBC audit, Clippy and native build pass
(15 seconds). Live timing and new card-layout evidence remain open.

## 2026-10-25 — Menu performance evidence

Separated screenshot-associated Metal presentation stalls from quiet menu timing.
Saved a [controlled frame-journal comparison](validation/mage-menu-presentation.md).
Performance runs will exclude concurrent UI capture; no runtime fix or combat
performance clearance is claimed.

## 2026-10-25 — Winter's Chill

Mage Frost hits can now build a per-enemy critical-chance debuff with five-stack
cap, refresh and expiry. It combines with Shatter without freezing the target.
Verified spend/refund and green previews at 720p; [screenshots](images/screenshots/2026-10-25-mage-talents/winters-chill-preview-720.png)
are linked in the README. 89 core and 153 app tests, DBC audit, Clippy and native
build pass (95 seconds). Live combat acceptance remains open.

![screenshots](images/screenshots/2026-10-25-mage-talents/winters-chill-preview-720.png)

## 2026-10-25 — Mage Fire procs

Impact now gives fire hits a trained stun chance, with shorter boss stuns and no
Ignite recursion. Incinerate increases Fire Blast critical chance. Affected spell
tooltips show Impact in green. 87 core and 152 app tests, DBC audit, Clippy and native
build pass (19 seconds). Live combat acceptance remains open.

## 2026-10-25 — Mage reach talents

Flame Throwing and Arctic Reach now extend the appropriate targeting ranges or
Nova/Cone radii. Tooltips use effective range; ground spells retain one location
and unchanged area size. 86 core and 151 app tests, DBC audit, Clippy and native
build pass (21 seconds). Live range-boundary verification remains open.

## 2026-10-25 — Armory save fix and Mage UI milestone

Armory edits now save immediately, fixing lost talent changes when closing the window
directly from Talents. Permafrost previews compare both strength and duration.
Verified refund/spend, independent specs and close/relaunch persistence; added
[Mage milestone screenshots](images/screenshots/2026-10-25-mage-talents/permafrost-720.png) to the README.
85 core and 150 app tests, Clippy and native build pass; live combat verification remains open.

![Mage milestone screenshots](images/screenshots/2026-10-25-mage-talents/permafrost-720.png)

## 2026-10-25 — Mage frost slows

Permafrost strengthens and extends frost slows; Improved Blizzard adds slow strength.
Shared combat/card values now show slow-strength changes inline with one trailing unit.
84 core and 150 app tests, DBC audit, Clippy and native build pass (14 seconds).
Live visual verification remains open.

## 2026-10-25 — Mage Shatter

Mage spells gain rank-scaled critical chance against frozen enemies, including area
ticks. Frost Nova and Frostbite track freezing separately from ordinary roots, with
green conditional-chance tooltips. 83 core and 148 app tests, DBC audit, Clippy and
native build pass (65 seconds). Live combat and screenshot verification remain open.

## 2026-10-25 — Mage Frostbite

Frost damage can now freeze enemies with a rank-scaled chance. Boss roots are shorter,
longer roots are preserved, and trained spell tooltips show the chance in green.
82 core and 147 app tests, DBC audit, Clippy and native build pass (21 seconds).
Shatter and live visual verification remain open.

## 2026-10-25 — Mage Ignite

Critical fire hits now build one Ignite burn, carrying unpaid damage into refreshes
without postponing the next tick. Added green spell-tooltip details and regression
coverage for rank limits, school restrictions, refresh damage and recursion prevention.
Native build, DBC audit, Clippy, 81 core and 146 app tests pass; live visual evidence remains open.

## 2026-10-25 — Mage talent effects

Implemented 16 Mage talents across Arcane, Fire and Frost, with shared previews and
combat values. Fixed ground-area ticks ignoring talent critical bonuses. Remaining
Mage nodes and milestones are still in progress. Native build, DBC audit, 80 core and
144 app tests, and Clippy pass. Talent-page screenshots remain pending a UI input-tool issue.

## 2026-10-25 — Placed-tree camera regression

Added a real Elwynn trunk fixture for blocked/clear camera paths, sphere clearance and
hull removal/reload. This strengthens collision coverage; the reported close-camera
symptom still needs an exact live reproduction. All 141 Survivors tests and Clippy pass.

## 2026-10-25 — Camera obstruction diagnostics

Added optional position/zoom tracing to distinguish collision push-in from visual
occlusion. An Elwynn run kept all 98 sampled gameplay booms at 30 yards; the earlier
close-camera symptom remains unresolved. Native build, 140 Survivors tests and Clippy
pass. Evidence and next checks are in the plan.

## 2026-10-25 — Indoor minimap imagery

Fixed Scarlet's blank map by compositing authored building-group textures. Unchanged
backgrounds are reused while markers update. All 140 Survivors tests, Clippy and native
build pass; live Scarlet zoom and Elwynn terrain checks pass.
[Scarlet screenshot](images/screenshots/2026-10-25-indoor-minimap/scarlet.png).
Broader indoor alignment, camera/navigation and performance acceptance remain open.

![Scarlet screenshot](images/screenshots/2026-10-25-indoor-minimap/scarlet.png)

## 2026-10-25 — Scarlet Monastery candidate

Added the Cathedral courtyard, 18 verified wave identities and eight timed bosses.
Campaign encounters without collection loot now require their final boss defeat.
All 79 core tests, Clippy and native build pass.
[Early combat screenshot](images/screenshots/2026-10-25-scarlet-candidate/combat.png)
reveals wall-navigation and minimap work still needed; Scarlet remains locked.

![Early combat screenshot](images/screenshots/2026-10-25-scarlet-candidate/combat.png)

## 2026-10-25 — Ahn'Qiraj Legendary armor sources

Added eight timed outdoor encounters supplying Legendary armor for every class, with
verified Silithus creature identities. All 78 core and 135 Survivors tests pass, plus
Clippy and native build. A live probe saved three Legendary Warrior pieces.
[Milestone screenshot](images/screenshots/2026-10-25-aq-roster/level-up.png).
The zone remains locked for world acceptance; the content audit still reports 87 gaps.

![Milestone screenshot](images/screenshots/2026-10-25-aq-roster/level-up.png)

## 2026-10-25 — Foreground performance validation

Corrected the profiling setup: prior unattended/background FPS was not representative
of attended play. Foreground Gurubashi held about 90 FPS at a chest; six short combat
samples measured 45–75 FPS. Auto logs now count unfocused/unknown-focus frames; live output, all 134 Survivors
tests, Clippy and the native build pass.
The [development guide](SURVIVORS-DEVELOPMENT.md#measure-attended-performance-explicitly)
documents repeatable foreground measurements. Full performance acceptance remains open.

## 2026-10-25 — Cache repeated combat-text placement

Frozen damage numbers reuse identical overlap placements. A dense 160-label check keeps
every rectangle identical while reducing repeat placement from about 10 ms to 0.001 ms.
All 133 Survivors, 12 combat-text and 10 solver tests pass, plus Clippy and native build.
[Live chest evidence](images/screenshots/2026-10-25-combat-text-cache/chest.png) preserves
the numbers; selecting a reward resumes them. Overall Gurubashi performance remains open.

![Live chest evidence](images/screenshots/2026-10-25-combat-text-cache/chest.png)

## 2026-10-25 — Reuse unchanged minimap images

Paused scenes now reuse their minimap image until visible inputs change. Native build,
all 131 Survivors app tests and workspace Clippy pass; live chest selection confirms the
map updates after combat resumes. Gurubashi remains slow. A new stall sample points to
combat-text overlap placement as the next investigation target; no FPS gain is claimed.

## 2026-10-25 — Correct chest jewelry artwork

Chest jewelry now uses the verified item icons shared with the Codex, replacing generic
spell art. Added a held-chest inspection mode and [live trinket evidence](images/screenshots/2026-10-25-chest-item-icons/trinkets.png)
to README. All 131 app tests and Clippy pass. Mark of the Champion's identity remains
unresolved; reward mechanics and saves are unchanged.

![live trinket evidence](images/screenshots/2026-10-25-chest-item-icons/trinkets.png)

## 2026-10-25 — Spawn clearance and live Epic rewards

Spawn candidates now reject body overlaps with registered collision geometry and retry
other directions. Verified walls, ceilings, slopes and arena-edge spawning; all 130 app
tests and Clippy pass. Native build took 22 seconds. Added a [Gurubashi results milestone](images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png);
isolated accelerated attempts persisted three Epic pieces. Slow combat and generic chest
jewelry icons remain open; Gurubashi stays locked.

![Gurubashi results milestone](images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png)

## 2026-10-25 — Arena collision evidence and catalog screenshot

Verified Gurubashi's placed arena boundary with body/camera probes and 128 grounded
radial routes across the play area. Added the seeded [54/54 PvP menu milestone](images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png)
to README. Live streaming, camera rendering and crowd acceptance remain open;
Gurubashi stays locked.

![54/54 PvP menu milestone](images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png)

## 2026-10-25 — Gurubashi Epic reward schedule

Added nine timed Epic reward encounters with verified outdoor Stranglethorn identities,
replacing the old mixed raid roster. The drop UI and combat schedule share all nine
slots; all-class save/reload coverage verifies equipment, appearances and duplicate
rewards. All 77 core and 127 app tests pass. Gurubashi remains locked pending world and
live encounter validation; the initial terrain footprint is flat and dry.

## 2026-10-25 — Complete collection identity catalog

Added the 81 PvP class/slot identities, completing all 54 named and mapped cells per
class. All 76 core and 126 app tests pass, including local display/icon reads; native
incremental build took 13 seconds. Acquisition remains incomplete (26 cells per class
lack drops), and full appearance/set-build validation remains open. The new menu capture
is pending because the Mac is locked; existing README screenshots remain historical.

## 2026-10-25 — Legendary armor identities

Added 72 verified class armor identities, bringing each collection to 45 of 54 named
cells. Existing Legendary weapons and numeric save slots remain unchanged. All 75
core and 126 app tests pass, including new display/icon reads; native build took 13 seconds.
The PvP row, remaining acquisition schedules, live appearances and raid-set powers remain open.

## 2026-10-25 — Complete Epic item identities

Added 81 verified Epic armor/weapon identities across all nine classes: each collection
now has 37 of 54 named cells. Existing rarity power and save slots are preserved. Added
an [equipped-menu milestone](images/screenshots/2026-10-25-epic-catalog/equipped-menu.png)
to the README. All 74 core and 126 app tests pass, including new display/icon reads.
Epic sources, full appearance checks and raid-epic/set mechanics remain open.

![equipped-menu milestone](images/screenshots/2026-10-25-epic-catalog/equipped-menu.png)

## 2026-10-25 — Rare rewards and full-health cards

Verified all 81 Rare class/slot rewards through save/reload and duplicate replay,
preserving selected gear, independent appearances, empty slots and class isolation.
Healthstone now clearly says when health is already full; actual heals retain green
inline comparisons. All 126 Survivors app tests pass. Remaining live Rare encounters
and appearance checks stay open.

## 2026-10-25 — Feralas live validation and useful combat metrics

The accelerated Warrior fixture reached 219 enemies and persisted four Rare armor slots
in an isolated save. A [live level-up milestone](images/screenshots/2026-10-25-feralas-live/late-warrior.png)
is linked from the README. Autopilot logs now separate combat FPS/time and hitches from
menu and level-up frames; the new fields were checked in a native run. All 124 Survivors
app tests pass. Early loading hitches and full zone acceptance remain open; Feralas stays locked.

![live level-up milestone](images/screenshots/2026-10-25-feralas-live/late-warrior.png)

## 2026-10-25 — Faster horde collision

Survivors now slides along planes found by successive sweeps, avoiding repeated nearby-face
collection. The 220-body Feralas probe fell from 9.80 to 5.77 ms p95 (about 41%), with
more advancing steps and passing root, slope and two-wall corner checks. All 122
Survivors app tests pass; the native incremental build took 19 seconds. Live horde
profiling and broader world acceptance remain open.

## 2026-10-25 — Horde collision profiling

Added a repeatable, opt-in Feralas collision timing probe at up to 220 bodies. It exposes
roughly 9.8 ms p95 movement cost on this Mac; final-position overlap checks account for
much less in an independent replay. Solver optimization and live gameplay profiling
remain open. Measurements and the command are in the implementation plan. All 121
Survivors app tests with checkout data and workspace Clippy pass; runtime is unchanged.

## 2026-10-25 — Grounded movement across slopes and roots

Walking follows gentle terrain without wedging, recovers missed floor probes at triangle
edges, and rejects ground adjustments that put the character inside a root. The actual
Feralas terrain/root test now advances 39 yards before contact without penetration;
airborne clearance and existing collision checks pass. Native incremental build: 50 seconds;
121 Survivors app tests with checkout data pass. Live horde performance and full Feralas
world acceptance remain open; the arena stays locked.

## 2026-10-25 — Feralas geometry checkpoint

Added a [candidate world view](images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)
to the README and identified nearby root/tree collision hulls. A placed-root regression
covers capsule contact and collider removal/reload. Grounded traversal, gameplay camera
behavior and live encounters remain open; Feralas stays locked.
All 120 Survivors app tests with checkout data, workspace Clippy, formatting and
render-pass lint pass.

![candidate world view](images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)

## 2026-10-25 — Feralas world and encounter foundation

Added a provisional Feralas arena, 15 verified enemy identities and nine timed bosses.
Its drop schedule covers all Rare slots and waits for the final boss before victory.
Terrain sampling found a dry supported candidate; Feralas remains locked pending full
world, camera, combat and reward validation. Existing saved map indices are preserved.
Native build, 73 core tests, 119 app tests, Clippy and formatting pass; Feralas model
files resolve. The full content audit still reports 476 unfinished-content gaps.

## 2026-10-25 — Rare weapon identities

Added nine verified class weapon identities, completing the Rare tier's nine slots.
Each class now has 28 of 54 named cells; 26 remain unnamed. Weapons retain Survivors
stats. Rare acquisition sources and live weapon attachment/pose checks remain open.
Native build, 72 core tests, 119 app tests with data (including new display/icon reads),
workspace Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Rare armor identities

Added 72 verified armor identities across all nine classes, bringing each collection to
27 of 54 named cells. Rare items use their own icons and appearance metadata; stats remain
Survivors rules. Rare weapons, acquisition sources and live appearance checks remain open.
Added [collection](images/screenshots/2026-10-25-rare-armor/collection-720.png) and
[Warrior preview](images/screenshots/2026-10-25-rare-armor/character-720.png) milestones
from a seeded validation account. Native build, 72 core tests, 119 app tests with data,
workspace Clippy, formatting and render-pass lint pass.

![collection](images/screenshots/2026-10-25-rare-armor/collection-720.png)

## 2026-10-25 — Complete uncommon source table

Clutchmother Zavas, Gruff and U'cha now supply uncommon wrists, hands and weapons at
4:30, 7:30 and 15:00. All nine slots have named sources for every class. Victory requires
all nine scheduled bosses and the 16-minute clock. Un'Goro stays locked pending world
acceptance; live encounters, rewards and balancing remain open.
Native build, 71 core tests, 118 Survivors app tests with checkout data, formatting,
workspace Clippy and render-pass lint pass.
Follow-up: all nine classes now have reward/save replay coverage, including duplicate
drops, selected gear, independent appearances and class isolation; 119 app tests pass.

## 2026-10-25 — Uncommon weapons

Added nine verified weapon identities, completing the uncommon tier's nine slots for
every class. Collections now have 19 of 54 named cells per class; 35 remain unnamed.
Icons and attachments use pinned vanilla identities; rarity stats remain Survivors rules.
Weapon acquisition sources and live attachment validation remain open. Native build,
71 core tests, 118 Survivors app tests (including display/icon reads) and workspace Clippy pass.

## 2026-10-25 — Live card and chest validation

Verified keyboard banish, preserved neighboring cards, charge consumption and mouse
selection at 720p. A [single-card chest](images/screenshots/2026-10-25-banish/single-chest-720.png)
remains centered and usable, including after a jewelry replacement. README evidence added.
Chest instructions now say “Choose your chest reward” to cover both spells and jewelry.

![single-card chest](images/screenshots/2026-10-25-banish/single-chest-720.png)

## 2026-10-25 — Banish replacement safety

Banish now excludes every option in the current hand, including the card being replaced,
and keeps the replacement in its original position. If no distinct reward rolls, the hand
and charge are preserved without granting a free reroll. The banish key no longer falls
through into selecting a card. Native build, 70 core tests, 118 Survivors app tests and
workspace Clippy pass; live input/visual acceptance remains open.

## 2026-10-25 — Distinct chest rewards

New chest hands no longer repeat the same spell upgrade or jewelry piece. With fewer
eligible upgrades, hands can contain fewer than three cards; exhausted upgrade rolls do
not turn into free jewelry. Luck and legendary category thresholds are unchanged.
The two-spell duplicate seen during Un'Goro validation is covered by a regression test.
Native build, 69 core tests and 115 Survivors app tests pass. Reduced-hand live rendering
and duplicate prevention during banish replacement remain follow-up work.

## 2026-10-25 — Un'Goro gameplay investigation

Captured [early Hunter combat](images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)
on the southern candidate and refreshed the README. Moving away from the central tree
opens the view; naming, full traversal, camera, performance and boss/reward checks remain
open. Preview overrides were restored and the zone remains locked.

![early Hunter combat](images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)

## 2026-10-25 — Un'Goro encounter sources

Six timed bosses now roll uncommon armor from 3:00 through King Mosh at 16:00, using the
same table for combat and item sources. Creature identities and zone placements are
verified against pinned data. The [drop browser](images/screenshots/2026-10-25-ungoro-drops/sources-720.png)
explains why the zone remains locked; arena, camera and live encounter acceptance remain
open. Native build, 68 core tests, 115 Survivors tests and lint checks pass. README evidence updated.

![drop browser](images/screenshots/2026-10-25-ungoro-drops/sources-720.png)

## 2026-10-25 — Collection heading spacing

Centered slot headings within their proportional columns and adjusted text sizing to
separate the crowded Shoulders/Chest labels. Verified selection and resizing at
[1024×720](images/screenshots/2026-10-25-collection-headings/collection-1024.png) and
[1280×720](images/screenshots/2026-10-25-collection-headings/collection-720.png).
Native build and formatting pass; README screenshots refreshed.

![1024×720](images/screenshots/2026-10-25-collection-headings/collection-1024.png)

## 2026-10-25 — Uncommon armor catalog

Added 72 named armor appearances across all nine classes. Each class now has 18 of 54
named cells; new drop sources and the remaining 36 names are still pending. Existing
rarity stats and ownership are preserved. Native build, 67 core tests and 115 Survivors
tests pass, including display/icon checks. Hunter collection details were checked at
[720p](images/screenshots/2026-10-25-uncommon-armor/hunter-720.png) and
[1024×720](images/screenshots/2026-10-25-uncommon-armor/hunter-1024.png); narrow slot-heading
spacing and full armor-on-character validation remain open. README gallery updated.

![720p](images/screenshots/2026-10-25-uncommon-armor/hunter-720.png)

## 2026-10-25 — Class-specific Atiesh appearances

Priest, Mage, Warlock and Druid Atiesh cells now use their verified class reward identities
for collection icons and equipment rendering. Ambiguous name-only lookups stay unresolved.
Native build, 67 core tests and 115 Survivors tests pass, including local display/icon checks.
Live staff attachment checks, acquisition paths and unnamed collection cells remain open.

## 2026-10-25 — Hunter control talents

Scatter Shot and Wyvern Sting add brief shot stuns; Entrapment roots enemies hit by
Explosive Trap. Boss durations are reduced and longer active controls are preserved.
Ability tooltips show both durations. Hunter now has sixteen individual effects; remaining
class work and live control validation remain open. Native build, 66 core tests, 115
Survivors app tests, catalog audit, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Hunter Marksmanship milestones

Efficiency shortens shot cooldowns; Mortal Shots adds direct critical damage; Trueshot Aura
adds two Multi-Shot targets and strengthens Aimed Shot. These compose with individual ranks
and the existing baseline tree bonuses. Volley and trap areas are unchanged. Other Hunter
milestones, remaining talents, pets and live milestone validation remain open. Native build,
65 core tests, 113 Survivors app tests, workspace test compilation, Clippy, formatting and
render-pass lint pass.

## 2026-10-25 — Hunter talent UI evidence

Verified rank refund/spend, independent spec reset and saved allocation after relaunch.
The three-tree page and green descriptions fit 1280×720 and a live resize to 1024×720.
Added [milestone screenshots](images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg)
to the README. No runtime errors or panics appeared; broader visual acceptance remains open.

![milestone screenshots](images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg)

## 2026-10-25 — Hunter Marksmanship bonuses

Barrage, Lethal Shots, Mortal Shots and Ranged Weapon Specialization now affect explicit
ability groups. Shot critical bonuses compose with equipment; traps and Volley receive
no shot-only critical bonus. Barrage improves Volley damage without adding area locations.
Hunter now has thirteen individual effects; remaining talents, pets and milestones are open.
Native build, 64 core tests, 111 Survivors tests, catalog audit, Clippy, formatting and
render-pass lint pass. Live visual acceptance remains open.

## 2026-10-25 — Hunter training foundation

Nine Hunter talents now affect shots, traps, armor, movement, damage reduction or critical
chance instead of generic damage. Tooltips identify Survivors conversions. Rank caps,
selected-spec reset and single-area targeting are preserved. Pets, remaining talents,
milestones and visual acceptance remain unfinished. Native build, 63 core tests,
110 Survivors app tests, catalog audit, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Grounded tree collision

Fixed a reproduced Un'Goro trunk penetration: steep surfaces now constrain horizontal
sliding instead of lifting actors before grounding. Added a real placed-tree/terrain
fixture covering initial clearance, 80 grounded steps and collider removal. Native build,
workspace test compilation, Clippy, formatting and 109 Survivors tests pass (including
12 floor tests). Full zone traversal,
streaming and navigation validation remain open; Un'Goro stays locked.

## 2026-10-25 — Upgrade comparison units

Upgrade comparisons use one trailing unit: `4 » 3.7 sec`, `5 » 7 yd`.
The inline card text already follows this rule; the remaining ability-summary formatter
now follows it too.

## 2026-10-25 — Un'Goro candidate visual evidence

Added a 720p world-viewer capture to the README and recorded nearby tree-hull probes.
The southern candidate renders jungle terrain with a large trunk near its center; player
clearance, grounded traversal and gameplay-camera checks remain open. No runtime errors
or panics appeared. Validation launch overrides were restored; the zone remains locked.

## 2026-10-25 — Recover terrain boundary heights

Survivors now checks nearby emitted terrain triangles when the fast height lookup misses.
The reproduced Un'Goro boundary resolves correctly; successful queries and real holes
retain their behavior. The change is opt-in for Survivors and its audit. Arena placement
and zone readiness remain unapproved pending collision, visual and content validation.
The southern candidate now resolves all 1,257 survey heights. Native build, workspace
test compilation, Clippy, formatting and 122 scoped tests pass; the feature-off format
build also checks.

## 2026-10-25 — Un'Goro arena investigation

The current summit footprint contains submerged and steep terrain. Surveyed nearby
alternatives and found a height-lookup discrepancy: 17 apparent gaps at a southern
candidate have rendered-mesh support. The audit now reports these discrepancies and
sample coordinates. Location evidence records the reproduction; Un'Goro remains locked.
Audit build, seven tests, scoped Clippy and workspace formatting pass.

## 2026-10-25 — Accurate critical-hit descriptions

Ability tooltips now show the effective critical damage percentage, including talents and
gear, instead of always promising double damage. Critical-chance passive cards/tooltips
now describe chance alone; inline green comparisons still show the unit once. Native build,
workspace test compilation, Clippy, formatting, render-pass lint and 12 HUD tests pass.
Live tooltip validation remains open.

## 2026-10-25 — Warrior strike training

Improved Overpower adds Heroic Strike critical chance; Improved Slam shortens Mortal
Strike's cooldown. Both use the selected spec and existing cast rules, with explicit
Survivors conversion text. They do not grant abilities. Warrior now has 34 supported nodes;
remaining talents and class completion are still open. Native build, workspace test
compilation, Clippy, formatting, 62 core tests, 107 Survivors tests and the local talent
catalog audit pass. Live talent-panel/combat validation remains pending.

## 2026-10-25 — Cutaway scales with the viewport

The foliage reveal now follows camera zoom, perspective and aspect ratio, occupying 35%
of the shorter viewport side. Its center shares the camera's shoulder-height focus.
Native macOS build, workspace Clippy and 61 core tests pass, including projection checks
at near/default/far zoom and landscape/portrait aspects. Live retained-renderer 720p on/off and farther-zoom captures now appear in the README;
the hero and nearby enemies remain visible through the canopy. Resize, other renderer
zoom checks and the broader visibility matrix remain pending.

## 2026-10-25 — Foliage visibility cutaway

Added a saved Scenery cutaway option that reveals the hero and nearby combat through
foliage in both scenery renderers, independently of camera auto-push. Live 720p on/off
comparisons and restart persistence verified; README screenshots added. Collision is unchanged.

Native build, formatting, workspace Clippy, workspace test compilation and focused suites
passed: 106 Survivors, 563 world (6 ignored), 103 assets and 59 core tests. Documentation-link and
render-pass lints also pass. Cleaned up earlier test-style lint failures. Interior visibility,
colored silhouettes, camera motion/zoom and the wider resolution matrix remain unfinished.

![Foliage reveal](images/screenshots/2026-10-25-cutaway/retained-gameplay.jpg)

## 2026-10-24 — Cutaway geometry foundation

Added a tested camera-to-hero reveal calculation with a depth-scaled circular footprint,
soft edge and protection for geometry behind the hero. All 59 core tests pass. This is the
CPU reference only: shader integration, scenery filtering and visibility controls remain
unfinished, so there is no visible cutaway effect yet.

## 2026-10-24 — Live tree contact and visibility

Captured a nearby Elwynn trunk during forward contact and a successful sidestep; links are
in the README. This approach appears blocked, while the trunk obscures the hero and combat.
The reported collision issue remains open across other trees/approaches, as do cutaway and
silhouette rendering. Temporary XP tuning and launch settings were restored.

## 2026-10-24 — Real-placement tree fixture

Added tree 12580 with its actual transform and terrain tile to collision regression coverage.
It reproduces both runtime endpoints without a detected center-ray face crossing. Added
source winding diagnostics to investigate starts inside open hulls. All 106 focused
Survivors tests and the audit-tool tests pass. Rendered trunk confirmation remains open;
no gameplay collision policy changed.

## 2026-10-24 — Repeated tree-walking regression

Extended the six-variant Elwynn hull regression to 2,304 grounded movement segments.
Each approach must respond to the hull; independent center-ray checks reject solid-face
crossings while allowing sliding around trunks. Workspace test compilation and all 105
focused Survivors tests pass. The rotated placement/terrain reproduction remains open.

## 2026-10-24 — Surface-aligned collision diagnostics

Added repeated walking queries and saved reproducible tree-surface points. Canopy tree
12580 allows one approach but blocks the reverse; recorded exact starts, height-sample
failures and lateral movement in [surface evidence](validation/elwynn-tree-surface-queries.txt).
Native build passes. This is a narrowed reproduction candidate, not a collision fix.

## 2026-10-24 — Elwynn collision evidence

Audited source hulls and queried twelve nearby tree placements in the streamed world.
Several trunks block movement queries; canopy-origin paths need exact trunk alignment
before diagnosing a failure. Saved [query evidence](validation/elwynn-tree-queries.txt)
and the next reproduction steps. The reported tree-collision issue remains open.

## 2026-10-24 — Responsive results

Results use proportional widths, bounded controls and wrapping stat tiles/text. Live resize
checks passed at [1280×720](images/screenshots/2026-10-24-results-layout/wide-720.jpg),
[640×720](images/screenshots/2026-10-24-results-layout/narrow-720.jpg) and
[480×720](images/screenshots/2026-10-24-results-layout/wrapped-720.jpg).
Native incremental build passed in 30 seconds. Temporary validation tuning was restored;
these captures verify layout, not balance or the rest of the interface at narrow sizes.

![1280×720](images/screenshots/2026-10-24-results-layout/wide-720.jpg)

## 2026-10-24 — Defeat screen evidence

Added [720p defeat results](images/screenshots/2026-10-24-defeat/results-720.jpg) to the README.
An isolated save and temporary development tuning exercised the death-to-results flow;
normal tuning was restored. A follow-up verified Play again returns to hero selection and
starts a fresh run with full health, a restarted timer and zero damage; see the
[paused replay](images/screenshots/2026-10-24-defeat/replay-720.jpg). Corrected “1 marks”
to “1 mark.” Victory and normal-balance acceptance remain open.

![720p defeat results](images/screenshots/2026-10-24-defeat/results-720.jpg)

## 2026-10-24 — Consistent lethal-hit handling

Spell healing, lifesteal and pickups can no longer revive a hero after a lethal hit but
before the death animation. Pickups also cannot open a level-up choice in that interval.
New casts, chest collection and stale card selections require a living hero. Existing
in-flight attacks still resolve. Manual death/result-screen acceptance remains open.

Native build, workspace test compilation and 105 focused Survivors tests pass.

## 2026-10-24 — Charge training and paused mobility

Improved Charge lengthens Charge's stun; Improved Intercept shortens its cooldown. The
cast, cooldown sweep and tooltip share the trained values. Warrior has 32 supported nodes.
Mobility cooldowns and active speed/vanish effects now freeze during pause, level-up and
loading. Values remain provisional; fresh screenshots and manual Charge checks remain open.

Native build, workspace test compilation, 56 core tests, 102 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Reactive talent HUD

Enrage, Flurry and Blood Craze now show green effect values and remaining seconds beneath
the clock. Lines disappear on expiry or death. Live 720p checks verified activation,
multiple active effects and clearing; screenshots show [active Flurry](images/screenshots/2026-10-24-reactive-buffs/flurry-720.jpg)
and the [cleared HUD](images/screenshots/2026-10-24-reactive-buffs/expired-720.jpg).

Native build, workspace test compilation and 100 focused Survivors tests pass. Dedicated
buff icons, broader viewport checks and full combat acceptance remain open.

![active Flurry](images/screenshots/2026-10-24-reactive-buffs/flurry-720.jpg)

## 2026-10-24 — Flurry training

Direct Warrior criticals can now trigger a three-second Flurry haste bonus. Further
criticals refresh without stacking; bleeds, area ticks and reflection do not trigger it.
The bonus follows existing haste cooldown rules. Warrior now has 30 supported nodes,
and the run-only talent snapshot has room for complete class catalogs without save changes.
Balance, dedicated buff presentation and manual combat acceptance remain pending.

Native build, workspace test compilation, 56 core tests, 99 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Blood Craze and lethal-hit healing

Blood Craze heals over six seconds after health damage; repeated hits refresh without
stacking. Immunity and full absorbs do not trigger it, and spell healing stays independent.
Regeneration can no longer undo a lethal hit before the death check. Warrior now has 29
supported nodes. Values remain provisional; buff presentation and manual combat checks
remain pending.

Native build, workspace test compilation, 55 core tests, 98 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Inline card visual checkpoint

Verified single-unit green comparisons and removed prose rank labels in the live checkout.
The same hand fits [720p](images/screenshots/2026-10-24-inline-units/cards-720.jpg),
[1280×800](images/screenshots/2026-10-24-inline-units/cards-800.jpg) and
[1024×768](images/screenshots/2026-10-24-inline-units/cards-1024.jpg); selecting Cleave after
resizing grants rank two. A [second hand](images/screenshots/2026-10-24-inline-units/area-1024.jpg)
shows seconds, yards and percentages without repeated units. README evidence refreshed.
Chest/jewelry captures and the complete resolution/UI-scale matrix remain pending.

![720p](images/screenshots/2026-10-24-inline-units/cards-720.jpg)

## 2026-10-24 — Warrior utility training

Death Wish extends Retaliation, including its active Fury risk. Improved Disarm extends
Demoralizing Shout's weakening; Improved Taunt shortens its cooldown. These Survivors
adaptations bring Warrior to 28 supported nodes; values remain provisional.

Native build, workspace test compilation, 54 core tests and 97 focused Survivors tests
pass; the local talent audit verifies node identities and rank caps. Fresh screenshots and manual combat validation remain pending.

## 2026-10-24 — Inline green upgrade descriptions

Cards now embed before/after values and units directly in their descriptions, with changes
green and surrounding prose neutral. This replaces the separate summary. Rank/new-choice
context appears only beneath the spell name; jewelry replacement losses remain red. Tooltips
share the ability prose. Comparison units appear once, after the new value.

Live captures: [new spell/passive previews](images/screenshots/2026-10-24-inline-upgrades/new-720.jpg),
[rank-up and area comparisons at 720p](images/screenshots/2026-10-24-inline-upgrades/upgrade-720.jpg),
[same hand after resizing to 1280×800](images/screenshots/2026-10-24-inline-upgrades/upgrade-800.jpg).

Native build, workspace test compilation and 96 focused Survivors tests pass. Fresh chest
and jewelry captures, plus the full resolution matrix, remain pending.

![new spell/passive previews](images/screenshots/2026-10-24-inline-upgrades/new-720.jpg)

## 2026-10-24 — Inline upgrade presentation planned

Updated the design contract: embed green before/after values, arrows and units directly in
card descriptions, replacing the separate change summary. Implementation and fresh screenshots
are pending; existing summary-style cards are not acceptance evidence for this requirement.

## 2026-10-24 — Card lighting stays on the card

Removed the oversized animated rarity shadow that left a hard rectangular edge across
the backdrop. Hover now brightens the rounded border while retaining lift and tilt;
the card-face lighting and landing sheen remain inside the card.

Native build passes. Verified hover and readable upgrade text at
[1280×720](images/screenshots/2026-10-24-card-lighting/hover-720.jpg) and after a live resize to
[1280×800](images/screenshots/2026-10-24-card-lighting/hover-800.jpg); selecting the resized
card correctly grants its spell. Full rarity/animation/scale coverage remains open.

![1280×720](images/screenshots/2026-10-24-card-lighting/hover-720.jpg)

## 2026-10-24 — Front-door talent activation and Enrage status

Fixed the front-door Enter action dropping the selected hero's active talent snapshot.
Permanent sheet stats were present, but spell-specific ranks and milestones could be missing
in the run. The entry path now copies them just like the other launch paths. Enrage also
has a green HUD line beneath the clock showing its damage bonus and seconds remaining.
Ability descriptions now use the same rank- and talent-scaled effect durations as their
green upgrade previews. [Live Enrage HUD](images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg).
[Trained shout card](images/screenshots/2026-10-24-warrior-talents/trained-shout-card-720.jpg).

Native build, workspace test compilation and 92 focused Survivors tests pass. Live 720p
checks confirm talent activation, Enrage's HUD line and the corrected duration description.

![Live Enrage HUD](images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg)

## 2026-10-24 — Reactive Warrior Enrage

Enrage now grants a four-second damage bonus after a hit removes health. Further hits
refresh the effect without stacking; immunity and full absorbs cannot trigger it.
The green talent preview shows the conditional damage increase. Twenty-five Warrior
nodes now have effects. Dedicated buff presentation and manual combat validation remain open.

Native build, workspace test compilation, 54 core tests, 90 focused Survivors tests and
the local talent-data audit pass. Live combat and fresh description captures remain pending.


## 2026-10-24 — Warrior absorb talents

Shield Specialization and Shield Slam now strengthen Heroic Strike's absorb; Improved
Shield Block extends its duration. Their effects combine with the Protection milestone
without stacking shields or replacing a stronger active shield. Green previews show shield
strength and added seconds. Twenty-four Warrior nodes now have effects; balancing and
manual combat acceptance remain pending.

Native build, workspace test compilation, 54 core tests, 89 focused Survivors tests and
the local talent-data audit pass. Fresh shield-description captures and manual combat
acceptance remain pending.


## 2026-10-24 — Warrior control talents

Piercing Howl adds slow duration to Demoralizing Shout; Concussion Blow adds stun duration
to Heroic Strike. Both compose with their tree milestones, preserve boss resistance and
show green rank comparisons in seconds. Twenty-one Warrior nodes now have effects.
Manual combat validation and balancing remain pending.

Native build, workspace test compilation, 53 core tests, 88 focused Survivors tests and
the local talent-data audit pass. Both green previews fit at 1280×720 on an empty validation
spec. These checks do not establish full manual combat acceptance.
[Piercing Howl](images/screenshots/2026-10-24-warrior-talents/piercing-howl-node-720.jpg) ·
[Concussion Blow](images/screenshots/2026-10-24-warrior-talents/concussion-blow-node-720.jpg).

![Piercing Howl](images/screenshots/2026-10-24-warrior-talents/piercing-howl-node-720.jpg)

## 2026-10-24 — Warrior bleed talents

Deep Wounds now adds bleed damage per talent rank; Improved Rend increases that bleed.
Both replace placeholder bonuses and show green current/next-rank values. Bleeds retain
three-second, non-stacking refresh behavior and cannot trigger themselves. Nineteen
Warrior nodes now have implemented effects; other nodes and manual combat acceptance
remain unfinished. Coefficients are provisional pending balance work.
Native build, workspace test compilation, 52 core tests, 88 focused Survivors tests and
the local talent identity/rank audit pass. Live 1280×720 green descriptions verified:
[Deep Wounds](images/screenshots/2026-10-24-warrior-talents/deep-wounds-ranks-720.jpg) ·
[Improved Rend](images/screenshots/2026-10-24-warrior-talents/improved-rend-720.jpg).
These captures use an empty validation spec; manual combat acceptance remains pending.

![Deep Wounds](images/screenshots/2026-10-24-warrior-talents/deep-wounds-ranks-720.jpg)

## 2026-10-24 — Placed obstacle regression

Added a real-data Warsong stump regression covering player-capsule contact and collider
removal/re-registration. It reproduces the documented ray hit using the actual placement.
[Evidence and limits](SURVIVORS-LOCATIONS.md#placed-stump-capsule-regression).
Workspace test compilation and 87 focused Survivors tests pass with checkout data.
Live streamed collision and navigation remain pending; this is validation, not a collision fix.

## 2026-10-24 — Developer instance preview

The development launcher can now open the engine's world viewer with `--world-preview`.
Verified Warsong terrain and obstacles in a live 720p render; campaign unlocks are unchanged.
[Capture](images/screenshots/2026-10-24-world-preview/warsong-720.jpg) ·
[Launch instructions](SURVIVORS-LOCATIONS.md#developer-instance-preview).
Native build passes. Player collision, navigation and gameplay camera checks remain pending.

![Capture](images/screenshots/2026-10-24-world-preview/warsong-720.jpg)

## 2026-10-24 — Warsong obstacle evidence

Model audits now report transformed collision bounds and rays from the candidate center.
Warsong's proposed arena contains solid stump obstacles; a documented 8.9-yard hit supplies
a fixture for runtime collision and routing checks. [Evidence](SURVIVORS-LOCATIONS.md#warsong-source-model-obstacles).
Six audit tests and Clippy pass. Runtime collision and world rendering remain unverified.

## 2026-10-24 — Arena terrain surveys

Added `--terrain` to measure sampled support, water, impassable terrain and slopes across
candidate arenas, including tile boundaries. Six local surveys reject Arathi's wet midpoint
and narrow the next world checks to a 60-yard Warsong arena. [Evidence and limits](SURVIVORS-LOCATIONS.md#arena-terrain-surveys).
Five audit tests and Clippy pass. No new zone is unlocked by these geometry checks.

## 2026-10-24 — Campaign location evidence

Added a read-only `--locations` audit using local map/area metadata and terrain. Documented
three battleground candidates and the remaining content gaps in the
[location report](SURVIVORS-LOCATIONS.md). Zones remain locked pending traversal, encounter
and reward completion. Validation: four audit tests, local metadata report and three WMO
floor probes pass; these are evidence checks, not playability acceptance.

## 2026-10-24 — Control duration scaling

Slow, root, stun, fear and weakening durations now grow by 10% of base duration per ability
rank after the first. Duration talents compose with rank growth, and green upgrade previews
show effect duration separately from area/buff lifetime. Boss resistance remains in place.
Improved Demoralizing Shout now increases weakening duration instead of placeholder damage.

[Shout duration talent at 720p](images/screenshots/2026-10-24-warrior-talents/shout-duration-720.jpg).
These changes affect combat balance; coefficients remain provisional and full manual combat
validation is still pending.
**Validation:** native build, workspace test compilation, 86 Survivors tests, 51 core tests
and the local talent-data audit pass. Live talent description verified at 720p.

![Shout duration talent at 720p](images/screenshots/2026-10-24-warrior-talents/shout-duration-720.jpg)

## 2026-10-24 — Green talent upgrade previews

Supported talent effects now appear in green beneath white name/rank headings. Each node
shows its current contribution and next-rank value, or its final value at maximum rank.
Milestone effects also use green; unfinished effects retain a clear pending notice.
Comparisons use font-compatible text and distinguish node bonuses from total character stats.

[Green rank preview at 720p](images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg).
**Validation:** native build, workspace test compilation, 85 Survivors tests and 51 core
tests pass. Live 720p checks confirm the comparison, green effect text, white heading and
color reset for an unfinished talent.

![Green rank preview at 720p](images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg)

## 2026-10-24 — Warrior defensive talents

Four more individual talents replace placeholder damage: Improved Revenge increases
Retaliation damage, Improved Shield Wall extends it, and Last Stand/Bloodthirst add capped
healing to Retaliation/Execute. Node healing combines with milestone healing once per cast.
A longer Retaliation also extends Death Wish's incoming-damage penalty.

[Defensive talent details at 720p](images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg).
Coefficients remain provisional; remaining talents and manual combat validation are pending.
**Validation:** native build, workspace test compilation, 85 Survivors tests, 50 core tests,
and the local talent-data audit pass. Improved Shield Wall details verified at 720p.

![Defensive talent details at 720p](images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg)

## 2026-10-24 — Protection milestones

Last Stand heals on Retaliation, Concussion Blow adds a brief Heroic Strike stun, and
Shield Slam grants a small absorb once per Heroic Strike cast. Healing caps at maximum
health; absorbs do not stack or refresh a stronger shield. All nine Warrior milestone
behaviors are implemented; remaining talents and manual combat validation are unfinished.

[Protection details at 720p](images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 84 Survivors tests and 50 core
tests pass. Live 720p description verified. Coefficients remain provisional.

![Protection details at 720p](images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg)

## 2026-10-24 — Fury milestones

Piercing Howl adds a slow to Demoralizing Shout. Death Wish boosts Retaliation with an
incoming-damage tradeoff while active. Bloodthirst heals once per successful Execute cast.
The implementation plan records the provisional coefficients and how these compose with
existing effects.

[Fury details at 720p](images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 83 Survivors tests and 49 core
tests pass; the Death Wish description fits at 720p. A full manual Fury encounter is still
unverified. Protection milestones and remaining individual talents are unfinished.

![Fury details at 720p](images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg)

## 2026-10-24 — Arms milestones and reliable DoT ticks

Warrior Arms now gains Deep Wounds at 10 points, a larger Sweeping Strikes aura at 20,
and stronger Mortal Strike at 30. The bleed refreshes without stacking or postponing its
next tick. Damage-over-time catches up across long frames and stops at expiry.

[Arms milestone details at 720p](images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 82 Survivors tests and 48 core
tests pass. Live 720p description checked; a full manual Arms encounter remains unverified.
Fury/Protection milestones and remaining individual talents are still in progress.

![Arms milestone details at 720p](images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg)

## 2026-10-24 — Warrior talent effects

Twelve audited Warrior talents now improve specific attacks, shout radius, critical stats,
armor or damage reduction. Their effects replace placeholder damage, follow the selected
hero/spec, and appear in talent details. Other Warrior nodes and milestone behaviors remain
unfinished. The save format is unchanged.

**Validation:** native build, workspace test compilation, 79 Survivors tests, 47 core rules
tests, and the checkout talent-data audit pass. Live Heroic Strike and Booming Voice details
fit at 720p. [Milestone screenshot](images/screenshots/2026-10-24-warrior-talents/talents-720.jpg).

![Milestone screenshot](images/screenshots/2026-10-24-warrior-talents/talents-720.jpg)

## 2026-10-24 — Spell-specific legendary bonuses

Legendary amulets now enhance only their associated spell. Green comparisons name that
spell; replacements show lost bonuses in red. Extra targets remain limited to eligible
single-target attacks, with no additional AoE locations.

[Single-target bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-direct-720.jpg) ·
[Area bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-area-720.jpg).
Live 720p checks confirm both descriptions. Native build, workspace test compilation,
77 Survivors tests and 45 core rules tests pass. Bonus values remain provisional.

![Single-target bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-direct-720.jpg)

## 2026-10-24 — Codex effect descriptions

Selected jewelry shows its implemented standalone bonuses in green, using the same rules
as chest comparisons. This checkpoint exposed global legendary bonuses; the later scope correction above fixes them.
**Validation:** native build, workspace test compilation and 75 Survivors tests pass.
[Live 720p capture](images/screenshots/2026-10-24-codex-discovery/codex-effects-720.jpg)
confirms green ring bonuses match the rules.

![Live 720p capture](images/screenshots/2026-10-24-codex-discovery/codex-effects-720.jpg)

## 2026-10-24 — Shared jewelry artwork

Codex rings, trinkets and amulets now use compact icon cells, dimmed until offered.
Thirty-eight items have verified display mappings from the pinned vanilla catalog. Mark of the Champion
remains a selectable question mark because its name identifies two different displays.
**Validation:** native build, workspace test compilation and 74 Survivors tests pass,
including all mapped icon files in the explicit data chain. The
[720p capture](images/screenshots/2026-10-24-codex-discovery/codex-jewelry-720.jpg) shows
the compact grid; mapped and unresolved cells both open their details.

![720p capture](images/screenshots/2026-10-24-codex-discovery/codex-jewelry-720.jpg)

## 2026-10-24 — Codex browsing

Selecting an entry updates the details text in place, preserving the catalog's scroll
position. Native build and a live 720p check with two amulet selections pass.
Workspace test compilation and all 73 Survivors tests pass.

## 2026-10-24 — Codex layout

The catalog and selected-entry details now use proportional, independently scrolling panes.
Class rows use class colors and the footer totals discoveries across heroes. Shared entries
show whether they have been offered. Jewelry artwork and legendary amulet badges remain
unfinished; the current rows retain names and spell icons.

**Validation:** app library check, Clippy and native build passed (1m08s).
[720p capture](images/screenshots/2026-10-24-codex-discovery/codex-layout-720.jpg) verifies
amulet selection/details and reachable footer. Selection still resets catalog scrolling.

![720p capture](images/screenshots/2026-10-24-codex-discovery/codex-layout-720.jpg)

## 2026-10-24 — Shared Codex discovery

Rings, trinkets and shared amulets now keep account-wide discovery when their cards are
dealt. Legendary discovery follows the actual hand too, so discarded banish candidates
no longer count. Shared rows show discovered totals instead of always showing completion.
Jewelry effects remain run-only; old saves begin with no shared discoveries recorded.

**Validation:** workspace test rebuild, 73 Survivors tests and workspace Clippy pass.
The native build passed. A 1280×720 live run confirms green level-up and chest scaling
previews ([level up](images/screenshots/2026-10-24-codex-discovery/level-up-720.jpg),
[chest](images/screenshots/2026-10-24-codex-discovery/chest-720.jpg)). Jewelry offer/restart
validation and the Codex icon/detail layout remain unfinished; shared amulets clip at 720p.

![level up](images/screenshots/2026-10-24-codex-discovery/level-up-720.jpg)

## 2026-10-24 — Validation follow-up

Corrected another US-locale test assumption while retaining checks for base string tables
and wrapped localization assignments. The audit now identifies duplicated Survivors UI
labels that need installed-string lookups. The complete workspace run finished with
7,917 passed, five failed and 18 ignored across 143 reported suites. Remaining failures:
reference strings, UI VM dependencies, web assets and the world API boundary. The full
repository gate is not green; the optional addon corpus is absent.

## 2026-10-24 — Class Codex completion

Class amulet completion now uses that class’s actual spell catalog for the Codex, cloaks
and achievement totals. Duplicate save rows no longer inflate progress. Unknown legacy
names are preserved but do not count. Recognized legacy names are normalized to restore
ability-key underscores after reload. Shared jewelry discovery and the Codex layout remain open.

**Validation:** 45 core and 71 Survivors app tests pass, including all-class completion,
legacy save reload and duplicate-count protection. Formatting and workspace Clippy pass;
the full gate is still running.

## 2026-10-24 — Portrait input during loading

Portrait rotation now stops under the loading cover and cancels held gestures. OS window
events remain available, including close requests. Added the missing input-audit decisions
for the portrait and browser heartbeat.

**Validation:** workspace test rebuild and nine focused portrait/loading-cover checks pass.
Formatting and workspace Clippy also pass. The full gate passed the cover audit, then
found a stale launcher example; its missing Lua patch and fork URLs are corrected, with
both launcher checks passing. Later gate stages and live interaction remain unverified.

## 2026-10-24 — Locale-aware validation

Corrected UK-only test assumptions for US client data. Glue strings are checked against
Lua execution, both clock formats are tested, and the verified US locale symbol is recorded
in the API reference without allowing arbitrary locale globals. Gameplay is unchanged.

**Validation:** 15 focused tests and the workspace test rebuild pass. The full gate passes
formatting, Clippy and the app unit suite, then stops at the loading-cover input audit:
portrait window events and browser wakeup types need explicit handling/classification.
Later gate stages remain unverified.

## 2026-10-24 — Loading and development integration

Added a loading-cover warmup draw for ability cooldown overlays. Moved native tuning/layout
reload, F12 captures and collision probes behind the existing development-only entry point;
player builds retain embedded defaults.

**Validation:** workspace test rebuild, 79 focused tests, formatting and workspace Clippy
pass. The player-only check passes with five warnings (four unused imports and an
unfulfilled rune lint expectation). The three remaining locale failures are confirmed enGB
test/reference assumptions against this checkout’s enUS assets. No UI layout changed.

## 2026-10-24 — Validation baseline

Normalized formatter drift in 22 files, simplified two core test setups, and named the
arena-scout candidate structure. No gameplay or visual changes.

Named repeated Bevy query shapes, restricted test-only helpers to test builds, and cleaned
up test initializers and placement. The reserved rune pickup rule keeps a specific warning
expectation until its spawning exists; no global warning suppression was added.

**Validation:** workspace formatting and workspace/all-target Clippy with warnings denied
now pass. The workspace run reached the app suite: 4,294 passed, six failed and ten were
ignored. The mock-network failure passes with loopback permission outside the sandbox;
five failures reproduce (material warmup, dev boundary, two localization checks and the
minimap clock format). Later gate stages have not run. The optional addon corpus is absent.
Earlier core validation passed 45 tests. The earlier
terrain-only diagnosis was incomplete because the gate prints a log tail.

## 2026-10-24 — Drop Tables browser

Replaced the text list with zone browsing, encounter/item rows, collection counts and shared
appearance, equipment and item-detail controls. Browsing leaves the run destination intact.
Missing tables are labeled explicitly. Drop Tables, Collection sources and item details
now include Keen Eye using combat’s shared chance calculation. Scheduled encounters are
labeled as bosses rather than deriving a final-boss claim from the zone-summary name.

**Validation:** native build and 72 focused tests passed. Live 1280×720 checks covered
Elwynn/Duskwood sources, item selection, missing content and the unchanged run destination.
The full resolution matrix and missing content remain open. Existing terrain formatting
still blocks the repository gate.

Keen Eye follow-up: native build, 45 core tests and 73 focused app tests passed; live rank-two Drop Tables and Collection labels
showed 70% from a 60% base at 1280×720. Combat odds are unchanged.

![Drop Tables at 1280×720](images/screenshots/2026-10-24-drop-tables/keen-eye-720.jpg)

## 2026-10-24 — Shared Collection appearance

Collection now shares Character’s faction portraits and race/sex controls, with class
restrictions and per-hero saving. Equipment remains directly below the controls.

**Validation:** native build and 71 focused tests passed. Live 1280×720 checks covered
race/sex changes, the matching Character model, and visibility of all nine empty slots.
The broader resolution/account matrix remains open; existing terrain formatting still
blocks the repository gate.

![Collection appearance at 1280×720](images/screenshots/2026-10-24-collection-appearance/collection-720.jpg)

## 2026-10-24 — Collection artwork

Collection cells now show implemented item artwork, dim unowned pieces, and keep E/L badges
in the corner. Missing-item source entries share the dimmed artwork, and rarity rows
list zones from implemented drop tables. Content without verified artwork retains a placeholder.

**Validation:** native build and 71 focused tests passed. Live checks covered owned icons,
combined E/L badges and legendary selection at 1440×900, plus unowned artwork and clickable
missing-item sources at 1280×720.
Existing terrain formatting still blocks the full repository check.

![Collection artwork](images/screenshots/2026-10-24-collection-icons/collection.jpg)
[Missing-item sources at 1280×720](images/screenshots/2026-10-24-collection-icons/sources-720.jpg)

## 2026-10-24 — Equipment rows

Character and Collection now show equipped item icons beside rarity-colored names, with
an explicit selection highlight and aligned empty slots. Icon sizes follow the UI text scale.

**Validation:** native build and 71 focused tests passed. Live 1440×900 checks covered
all nine equipped icons, selection/details, unequip/re-equip and the shared Collection rows.
Existing terrain formatting still blocks the full repository check.

![Equipment rows and dressed preview](images/screenshots/2026-10-24-equipment-rows/character.jpg)

## 2026-10-24 — Character appearance controls

Race and sex can now be selected above Character equipment. The preview updates immediately
and each named hero saves its own choice. Every race in each class's existing catalog now
offers both sexes, filling missing bodies with verified same-race trainer presets while
preserving previous appearances. Skin, face, hair and race-specific features now have live
controls and randomization, saved independently for each hero. Race portraits are grouped
by faction, follow the selected sex, and dim unavailable class choices. Feature edits update the counters and model in place, preserving the
Character screen and preview rotation. Short mouse drags now rotate the model reliably,
with sensitivity proportional to the preview width.

**Validation:** native build and 71 focused tests passed, including local DBC
identities and readable model files for every body. Live 1440×900 checks covered human,
night elf, newly available female orc/dwarf warriors, separate heroes and restart persistence.
Feature edits, hero switching and restart persistence also passed at 1440×900 / 1280×720.
Repeated feature edits and retained preview rotation were checked at 1280×720. Transient
missing button glyphs remain open, along with the full race/class/gear
visual matrix. Existing terrain formatting errors
still block the repository gate.

Short/reverse drags and rejection of presses outside the preview passed live at 1280×720.
Portrait selection, sex changes and disabled Paladin races also passed at 1280×720.
Feature controls now appear when a new hero’s body becomes ready; creation, immediate hair
edits and retained preview rotation passed live at 1280×720.

![Character race portraits, 1280×720](images/screenshots/2026-10-24-character-features/character-portraits.jpg)
[Independent hero appearance](images/screenshots/2026-10-24-character-appearance/character.jpg)
· [Drag rotation](images/screenshots/2026-10-24-character-features/character-drag.jpg)

## 2026-10-24 — Readable upgrade benefits

Level-up and chest choices show stat changes in green. New spells and passives preview
their next rank; jewelry compares the resulting loadout, with replacement losses in red.
Next-rank tooltips use the same green. Gameplay values are unchanged.

**Validation:** native build and 65 Survivors/preview tests passed, including replacement
losses and area/defensive labels. Live level-up cards checked at 1440×900; jewelry comparison
is covered by rules tests, not yet a live chest screenshot. Existing terrain formatting
errors still block the repository-wide gate.

![Green upgrade and next-rank previews, 1440×900](images/screenshots/2026-10-24-upgrade-benefits/level-up.jpg)

## 2026-10-24 — Dressed heroes and independent cosmetics

Selected armor and weapons now dress the world hero and Character preview. Hidden pieces
and appearance overrides leave equipped bonuses unchanged. The preview backs away to fit
large attachments; newly loaded weapon appearances refresh without a second click.
Weapon poses and the broader race/sex/weapon matrix remain unfinished.

**Validation:** native build, 44 core tests and 62 Survivors/preview tests passed. Seeded
accounts verified plate armor, hiding helmet/chest, independent weapon appearance and
world entry; weapon framing checked at 1280×720 and 1440×900, with the fixed first-load
replacement verified at 1280×720.
Repository checks still stop on two pre-existing terrain formatting errors.

![Dressed Character preview](images/screenshots/2026-10-24-dressed/character.jpg)
[Hidden helmet](images/screenshots/2026-10-24-dressed/hidden-helm.jpg) ·
[Large weapon](images/screenshots/2026-10-24-equipment-framing/weapon-appearance.jpg) ·
[1280×720 layout](images/screenshots/2026-10-24-equipment-framing/character-720.jpg)

## 2026-10-24 — Item visual identities

Collection and Character item details now show verified vanilla icons. Added 91 named
item mappings and corrected two item labels without changing saves. Atiesh variants and
unnamed rarity cells remain unresolved; dressing the hero in selected gear is still pending.

**Validation:** native build, 43 core tests and 58 Survivors tests passed, including local
DBC/icon availability. Helmet → axe → unresolved-item switching checked at 1440×900.

![Collection item detail](images/screenshots/2026-10-24-gear-icons/collection.jpg)

## 2026-10-24 — Character loadout layout

Character now has proportional, independently scrollable equipment, hero and item-detail
columns, plus a full-body current-hero preview with turn buttons. Named heroes retain their
own race and sex; older saves migrate their existing choice. Slot inspection and
appearance choices share Collection's loadout state. Native build and 57 Survivors tests
passed; framing, turn
buttons and tab transitions checked at 1440×900. Drag validation, selected-gear dressing,
inline features, populated-account checks and the resolution matrix remain unfinished.

![Character preview](images/screenshots/2026-10-24-preview/character.jpg)

## 2026-10-24 — Armory, progression and world interaction

- **UI:** three-column front door and Collection, illustrated talent trees, and a 14-screen
  README gallery. Upgrade cards wrap and scroll on narrower windows; shine stays on the
  card face and the oversized background rays are removed.
- **Progression:** selected heroes persist across restarts; worn equipment is separate from
  collected items. Duplicate drops add no power; rarity improves stats. Talent refunds
  preserve row requirements, with Paladin stat, spell, healing and shield effects implemented.
- **World:** movement sweeps against world geometry, nearby-floor placement for interiors,
  terrain-following mobility on gentle slopes, and a saved camera push-in option with
  smooth zoom restoration. Missing campaign content
  stays locked. Source-versus-live Elwynn probes expose offset canopy hulls; the reported
  tree-collision issue still needs visual reproduction.
- **Development:** incremental native builds, an engine-independent rules crate, live tuning
  and layout reloads, plus opt-in F12 screenshots. Failed replacement launches keep the
  current game open; validation shutdown avoids contacting the terminated macOS app.

**Validation:** native builds pass; recent Survivors regression suite: 54 tests passed.
Live Elwynn mobility probes traverse all four 12-yard open-ground routes.
Card reflow was exercised at 1000×800 and the gallery at 1440×900 logical resolution.
The repository check still stops at two existing terrain-stream formatting failures.

**Still in progress:** remaining Armory layouts, full gear/zone catalogs, other class talents,
raid sets, tree-collision investigation, foliage cutaway and colored silhouettes, the full
resolution matrix, and the authenticated updater. Screenshots show current UI, not completion
of every underlying feature.

| Front door | Collection |
| --- | --- |
| ![Front door](images/screenshots/2026-10-24/front-door.jpg) | ![Collection](images/screenshots/2026-10-24/collection.jpg) |
| Talent-tree presentation; effects incomplete | Responsive upgrade cards |
| ![Talent trees](images/screenshots/2026-10-24/talents.jpg) | ![Upgrade cards](images/screenshots/2026-10-24/level-up.jpg) |

[Full interface gallery](../README.md#interface-gallery) ·
[Build and capture instructions](SURVIVORS-DEVELOPMENT.md)
