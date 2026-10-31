<div align="center">

# Azeroth Survivors

**An independent WoW survival-game implementation on a from-scratch WoW 1.12.1
client engine (Rust + Bevy), inspired by [Aethus’s Azeroth Survivors](https://wow.aethus.pl/).**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

</div>

This repository implements the design in [our intentions](docs/intentions/) and
[the Survivors specification](docs/SURVIVORS.md) on the benilla/wenilla engine. It is
inspired by Aethus’s game; it is not a build of that game's source code. Implementation
is ongoing; the [acceptance contract](docs/SURVIVORS-IMPLEMENTATION.md) tracks the intended
scope, including content and screens that are not complete yet.
Follow the concise [milestone changelog](docs/CHANGELOG.md) for feature summaries and screenshots.
The [current acceptance status](docs/SURVIVORS-STATUS.md) summarizes remaining work and design decisions.

Hero names support Command-A/Ctrl-A selection and replacement, including saved
starter renames. [Native evidence and limits](docs/audits/NAME-SELECTION.md).

![Name selection](docs/images/screenshots/2026-10-31-name-selection/creation-selected.jpg)

Minimum-window hero creation is verified through appearance changes and restart
at 640×480 on macOS. [Native evidence and limits](docs/audits/CHARACTER-CREATION-MINIMUM.md).

![Created hero after restart](docs/images/screenshots/2026-10-31-creation-minimum/starter-restarted.jpg)

Improved Sprint now removes current movement impairments on its trained proc,
with green inline chance and persisted training. [Native evidence and limits](docs/audits/IMPROVED-SPRINT.md).

![Improved Sprint inline chance](docs/images/screenshots/2026-10-31-improved-sprint/rank1-wide.jpg)

Rogues can acquire Kick, with separate Improved Kick silence training and green
inline upgrades. [Native purchase/refund/restart and card evidence](docs/audits/ROGUE-KICK.md).

![Kick inline upgrades](docs/images/screenshots/2026-10-30-rogue-kick/cards-wide.jpg)

Priest Silence now unlocks through training and can be acquired during a run.
[Native purchase, refund, restart and card evidence](docs/audits/PRIEST-SILENCE.md).

![Silence inline card upgrades](docs/images/screenshots/2026-10-30-priest-silence/cards-wide.jpg)

Counterspell now supports acquired interruption and Improved Counterspell silence,
with inline green growth and saved training. [Evidence and limits](docs/audits/COUNTERSPELL.md).

![Counterspell inline upgrades](docs/images/screenshots/2026-10-30-counterspell/cards-wide.jpg)

Keyboard save recovery now also passes at the minimum 640×480 window, including
failed retry and persistence after restart. [Evidence and limits](docs/audits/SAVE-RECOVERY-KEYBOARD.md#minimum-window-follow-up--2026-10-30).

![Minimum-window keyboard save recovery](docs/images/screenshots/2026-10-30-save-keyboard-compact/failed-640x480.jpg)

Vengeance now includes Paladin automatic weapon attacks, with its changing bonus
shown inline. [Native captures and validation](docs/audits/PALADIN-VENGEANCE.md).

![Vengeance weapon and Holy damage](docs/images/screenshots/2026-10-30-vengeance/compact-restarted.jpg)

Native keyboard save-recovery checks now cover failure, retry and restart without
losing the pending talent purchase. [Evidence and limits](docs/audits/SAVE-RECOVERY-KEYBOARD.md).

![Keyboard save recovery](docs/images/screenshots/2026-10-30-save-keyboard/failed-retry.jpg)

Paladin weapon specializations now describe matching one-/two-handed automatic
attack bonuses inline. Native purchase, refund, independent specs and restart
checks cover wide and compact layouts.
[Evidence and remaining limits](docs/audits/PALADIN-WEAPON-TALENTS.md#weapon-specialization-implementation--2026-10-30).

![Compact Paladin specialization after refund](docs/images/screenshots/2026-10-30-paladin-weapons/two-hand-refund-compact.jpg)

The native wide/short talent layout matrix now covers all nine classes at
1440×600 / 125%. [Sixteen new class captures and remaining limits](docs/audits/TALENTS-WIDE-SHORT.md#complete-class-layout-matrix--2026-10-30).

![Paladin talent artwork](docs/images/screenshots/2026-10-30-talent-class-matrix/paladin-top.png)

Native talent checks now cover wide, short windows with scrolling to every
milestone and a readable pinned inspector. [Evidence and limits](docs/audits/TALENTS-WIDE-SHORT.md).

![Native wide short talent page](docs/images/screenshots/2026-10-30-talents-wide-short/milestones.png)

Rewards supports Page Up/Down and Home/End across all three categories.

![Keyboard paging through Rewards](docs/images/screenshots/2026-10-30-rewards-paging/page-up-companions.png)

Native Companion checks verify confirmed purchases and appearance persistence,
while unconfirmed purchases are discarded on restart.

![Companion training after restart](docs/images/screenshots/2026-10-30-companion-interactions/restarted-640.png)

Companion reward cards adapt to the available reading width while preserving
our reference's two-column layout on wide screens.
[Native comparisons and limits](docs/audits/COMPANION-READABILITY.md).

![Readable compact Companion rewards](docs/images/screenshots/2026-10-30-companion-reading/after-800.png)

Native appearance checks now include Gnome and Tauren bodies with Dreadnaught
armor, plus weapon-look persistence across a compact restart.
[Evidence and limits](docs/audits/COLLECTION-BODY-ART.md#native-gnometauren-plate-checks--2026-10-30).

![Tauren appearance restored in a compact window](docs/images/screenshots/2026-10-30-plate-bodies/tauren-male-restarted-compact.png)

Collection quality rows now name their drop zones, including Poor and Raid Epic
gear. Zone readiness remains visible on the linked Drop Tables page.

![Collection source names](docs/images/screenshots/2026-10-30-collection-sources/wide.png)

Virtuous Robe now has fabric cuffs on the male body even when worn without
bracers. [Native comparisons and validation](docs/audits/COLLECTION-BODY-ART.md)
cover the robe alone, matching pieces and the female control.

![Repaired Virtuous Robe in the Character page](docs/images/screenshots/2026-10-30-virtuous-robe/male-robe-fixed.png)

Healing Wave now benefits from Purification healing and Tidal Mastery critical
chance, with the changing values highlighted inline. Native macOS checks verify
purchase, refund, independent specs and restart alongside Healing Way.

![Purification healing scope](docs/images/screenshots/2026-10-29-shaman-healing-training/purification-trained.png)

![Compact Tidal Mastery description](docs/images/screenshots/2026-10-29-shaman-healing-training/tidal-compact.png)

Illumination now refunds half a critical heal’s recovery on a successful talent
proc, with the rank’s chance highlighted inline. Native macOS checks cover
purchase, independent specs, refund, restart and compact scrolling.

![Illumination talent inspection](docs/images/screenshots/2026-10-29-illumination/trained.png)

![Compact Illumination tooltip](docs/images/screenshots/2026-10-29-illumination/compact-purchase.png)

Healing Way now has its three-rank proc chance and capped, timed healing bonus.

![Healing Way talent inspection](docs/images/screenshots/2026-10-29-healing-way/trained.png)

![Compact Healing Way tooltip](docs/images/screenshots/2026-10-29-healing-way/restarted-compact.png)

Shamans can now acquire Healing Wave, with inline healing and recovery upgrades.
Healing Way now adds its source-specific stacking healing bonus.

![Healing Wave acquisition and upgrades](docs/images/screenshots/2026-10-29-healing-wave/cards-wide.png)

![Compact Healing Wave cards](docs/images/screenshots/2026-10-29-healing-wave/cards-compact.png)

Failed account writes now show a visible warning and Retry save. Native checks
verify that pending appearance changes survive recovery and restart.

![Save warning at compact UI size](docs/images/screenshots/2026-10-29-save-warning/compact.png)

![Successful retry clears the warning](docs/images/screenshots/2026-10-29-save-warning/recovered.png)

Hunter crossbow previews now have wide/compact and saved-visibility acceptance.
Hiding the weapon preserves its equipped bonuses.

![Hunter crossbow appearance restored](docs/images/screenshots/2026-10-29-hunter-ranged/restored.png)

![Compact Hunter crossbow preview](docs/images/screenshots/2026-10-29-hunter-ranged/crossbow-compact.png)

Character controls now keep keyboard focus on nearby appearance arrows in compact
layouts. Troll feature changes survive restart without changing equipped gear.

![Saved Troll appearance after restart](docs/images/screenshots/2026-10-29-troll-appearance/restarted.png)

![Compact feature controls after keyboard update](docs/images/screenshots/2026-10-29-troll-appearance/tusks-changed.png)

Fresh accounts now choose or create a named hero before entering a run. Native
minimum-window onboarding and restart captures are in the changelog.

![Fresh starter onboarding at 640×480 and 125% UI](docs/images/screenshots/2026-10-29-starter-onboarding/after.jpg)


![Codex details support keyboard reading](docs/images/screenshots/2026-10-29-codex-reading/home.png)

![Healing Light ranks persist after restart](docs/images/screenshots/2026-10-29-flash-light/restarted.png)

![Flash of Light and direct-healing upgrades](docs/images/screenshots/2026-10-29-flash-light/cards-wide.png)

![Holy Light acquisition and inline healing upgrades](docs/images/screenshots/2026-10-29-holy-light/cards-wide.png)

![Retribution Aura saved talent ranks after restart](docs/images/screenshots/2026-10-29-retribution-aura/restarted.png)

![Retribution Aura acquisition and inline damage upgrades](docs/images/screenshots/2026-10-29-retribution-aura/cards-wide.png)

![Repentance talent-unlocked control and inline upgrades](docs/images/screenshots/2026-10-29-repentance/cards-wide.png)

![Repentance saved spec after restart](docs/images/screenshots/2026-10-29-repentance/restarted.png)

![Greater Heal acquisition and inline upgrades](docs/images/screenshots/2026-10-29-greater-heal/cards-wide.png)

![Pending Rewards keep Confirm and Undo visible while browsing](docs/images/screenshots/2026-10-29-reward-confirm/after.png)

![Compact Drop Tables opens selected drop stats and actions](docs/images/screenshots/2026-10-29-drop-inspector/detail.png)

![Compact Collection opens item details and equipment controls directly](docs/images/screenshots/2026-10-29-collection-inspector/detail.png)

Native wheel/scrollbar acceptance now covers the taller talent trees and independent
achievement panes on macOS, including category round-trip restoration.

![Independent achievement scrolling at minimum height](docs/images/screenshots/2026-10-29-pointer-scroll/achievement-roundtrip.png)

## Interface gallery

Actual checkout captures, dated below. Many use isolated fixture accounts; a screenshot
shows the pictured state, not complete content or cross-platform acceptance. The
[intention images](docs/intentions/) remain the design reference. Earlier iterations
remain inline in the [changelog](docs/CHANGELOG.md) and [screenshot history](docs/SCREENSHOT-HISTORY.md).

Refresh the relevant page here for a visual milestone, and add its before/after or
responsive variants to the changelog. See [capture instructions](docs/SURVIVORS-DEVELOPMENT.md#readme-captures).


### Druid form artwork

Native specimens verify the Night Elf/Tauren pairs for Cat, Bear and Moonkin.
These are visual prerequisites; playable shapeshifting is still pending.

![Six Druid form specimens](docs/images/screenshots/2026-10-30-druid-forms/faction-pairs.png)

### Starter equipment readability

The starter uses nine equipment icons and a shared hover/focus inspection line.
Native macOS checks cover collected/empty slots, independent appearances and
mouse/keyboard inspection at 1280×720 and 640×480 (125% requested, short-window
fit). Selecting a slot opens its worn item in Collection. Windows and physical
controller checks remain open. See the [acceptance record](docs/audits/STARTER-EQUIPMENT-ACCEPTANCE.md).

![Starter equipment remains independent of its appearance](docs/images/screenshots/2026-10-30-starter-inspection/wide-independent-look.png)

![Compact pointer inspection](docs/images/screenshots/2026-10-30-starter-inspection/compact-pointer.png)

### Achievement keyboard scrolling

Page Up/Down scroll the achievement cards; Home/End jump to the top/bottom.
Native 640×480 / 125% acceptance, 2026-10-29, reaches the full Veteran reward and
final zone rows. Categories now scroll independently from the cards; narrow
layouts show a compact button grid. Category switching retains reading positions.
Before/after captures are in the changelog.

![Independent achievement navigation and cards](docs/images/screenshots/2026-10-29-achievement-panes/compact-end.png)

### Earth and Fire Totem specimens

Settled native previews verify the solid models and visible effects, including a
repeat launch. The earlier incomplete-body capture did not establish a persistent
rendering failure. Combat deployment and Windows checks remain open; see
[the audit](docs/audits/EARTH-TOTEMS.md).

![Earth and Fire Totem specimens](docs/images/screenshots/2026-10-29-earthbind/model-pair.png)

### Combat demon specimens

Native menu-only lineup, 2026-10-29: Imp, Voidwalker, Succubus and Felhunter.
These verified assets are not yet summonable; combat behavior remains pending.

![Native demon specimens](docs/images/screenshots/2026-10-29-demon-specimens/lineup.png)

### Enemy equipment specimens

Native noncombat preview, 2026-10-29: verified NPC weapon and shield metadata now
feeds enemy rendering. Combat/disarm acceptance remains open.

![Riverpaw Taskmaster equipment](docs/images/screenshots/2026-10-29-enemy-equipment/taskmaster.jpg)

Disarm visibility now hides the weapon and restores it when status ends, retaining
the shield. Same-session before/during/after captures are inline in the changelog;
ability acquisition and disarmed damage remain pending.

![Disarmed specimen retains its shield](docs/images/screenshots/2026-10-29-disarm-visuals/active.jpg)

### Live scale previews

Preview files support temporary `scale = 1.25` alongside page and dimensions;
omitting it restores saved settings. See [preview controls](docs/SURVIVORS-DEVELOPMENT.md).

![Rewards at 640×480 with 125% interface size](docs/images/screenshots/2026-10-28-preview-scale/compact-125.jpg)

### Starter and Character

<table>
  <tr>
    <td width="50%"><strong>Starter</strong><br><img src="docs/images/screenshots/2026-10-27-starter-16-10/starter.png" alt="Starter"><br>2026-10-27 · Populated 1280×800 starter with bounded panel scrolling; synthetic collection.<br><a href="docs/images/screenshots/2026-10-27-starter-16-10/compact.png">Compact starter preview</a></td>
    <td width="50%"><strong>Character</strong><br><img src="docs/images/screenshots/2026-10-27-raid-foundation/character-1280.png" alt="Character"><br>2026-10-27 · Synthetic raid loadout with independent appearance; raid world access remains gated.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Hero management</strong><br><img src="docs/images/screenshots/2026-10-27-hero-creation/form-640-125.png" alt="Hero management"><br>2026-10-27 · Labeled creation and direct class selection at 125%.</td>
    <td width="50%"><strong>Minimum viewport</strong><br><img src="docs/images/screenshots/2026-10-26-character-identity/tauren-640.png" alt="Minimum viewport"><br>2026-10-26 · Character at 640×480; full figure and rotation controls.</td>
  </tr>
</table>


Native 24-hero roster acceptance at 640×480 and 125% UI scale: every hero is
reachable, including selection of the final hero and lower management controls.

![Compact 24-hero roster](docs/images/screenshots/2026-10-29-large-roster/compact-roster.png)

### Armory

<table>
  <tr>
    <td width="50%"><strong>Collection</strong><br><img src="docs/images/screenshots/2026-10-28-rogue-weapons/collection.png" alt="Collection"><br>2026-10-28 · Rogue mace/fist choices with independent equipment and appearances; seeded ownership.<br><a href="docs/images/screenshots/2026-10-27-weapon-alternatives/compact.png">Compact Collection preview</a> · <a href="docs/images/screenshots/2026-10-27-collection-800-125/unequipped.png">800×600 at 125%</a></td>
    <td width="50%"><strong>Drop Tables</strong><br><img src="docs/images/screenshots/2026-10-27-raid-source/drops-1280.png" alt="Drop Tables"><br>2026-10-27 · Raid finale armor odds and item inspection; Naxxramas readiness gate remains visible.<br><a href="docs/images/screenshots/2026-10-27-raid-source/drops-640.png">Compact scrolling preview</a> · <a href="docs/images/screenshots/2026-10-27-collection-800-125/source.png">800×600 source navigation</a></td>
  </tr>
  <tr>
    <td width="50%"><strong>Codex</strong><br><img src="docs/images/screenshots/2026-10-28-codex-acquisition/wide.jpg" alt="Codex"><br>2026-10-28 · Amulet acquisition guidance and required talent training.<br><a href="docs/images/screenshots/2026-10-28-codex-acquisition/compact.jpg">640×720</a></td>
    <td width="50%"><strong>Talents</strong><br><img src="docs/images/screenshots/2026-10-29-greater-heal-saves/restarted.png" alt="Improved Healing rank and spent points retained after restart"><br>2026-10-29 · Taller rows, proportional artwork; purchase, separate specs, restart and refund verified on macOS.<br><a href="docs/images/screenshots/2026-10-29-talent-row-spacing/compact-bottom.png">640×720</a> · <a href="docs/CHANGELOG.md">Milestone screenshots</a></td>
  </tr>
  <tr>
    <td width="50%"><strong>Achievements</strong><br><img src="docs/images/screenshots/2026-10-29-achievement-reading/wide.png" alt="Achievements"><br>2026-10-29 · Two columns on wide panels; full-width cards and progress on narrow panels. Compact keyboard paging verified.</td>
    <td width="50%"><strong>Perks</strong><br><img src="docs/images/screenshots/2026-10-26-reward-focus/two-upgrades-1280.png" alt="Perks"><br>2026-10-26 · Staged upgrades and retained keyboard focus.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Companions</strong><br><img src="docs/images/screenshots/2026-10-26-companion-save/restarted-1280.png" alt="Companions"><br>2026-10-26 · Independent saved companion looks.</td>
    <td width="50%"><strong>Tabards</strong><br><img src="docs/images/screenshots/2026-10-26-tabard-controls/worn-1280.png" alt="Tabards"><br>2026-10-26 · Cosmetic selection; catalog expansion remains open.</td>
  </tr>
</table>


### Browser source inspection

![Raid armor source and set details in the browser](docs/images/screenshots/2026-10-27-browser-raid/drops-1280.png)

2026-10-27 · WebGPU at 1280×720; actual source-row selection. Naxxramas remains
locked pending world readiness. This capture does not establish raid gameplay.

### Talent milestone

![Warrior talents with worn-weapon specialization](docs/images/screenshots/2026-10-27-sword-specialization/trained.png)

2026-10-27 · 1600×900 native preview: taller trees, adjacent saved specs and a
green inline upgrade values. Purchase and compact captures are in the [changelog](docs/CHANGELOG.md).

Healing Wave now receives Improved Healing Wave and Tidal Focus recovery training;
both at rank five reduce base recovery from 12 to 8.64 seconds.

![Healing Wave recovery training](docs/images/screenshots/2026-10-29-shaman-recovery/improved-wide.png)

Shaman Parry now avoids contact attacks with a worn melee weapon; its 5% chance
and weapon requirement appear inline. Purchase, refund and compact restart verified.

![Shaman weapon parry after restart](docs/images/screenshots/2026-10-29-shaman-parry/restarted-compact.png)

### Options and upgrade previews

<table>
  <tr>
    <td width="50%"><strong>Options</strong><br><img src="docs/images/screenshots/2026-10-27-menu-options/after.png" alt="Options"><br>2026-10-27 · Starter Options retains the landscape; shown at 800×600 and 125% UI size.</td>
    <td width="50%"><strong>Spell and passive upgrades</strong><br><img src="docs/images/screenshots/2026-10-29-stoneclaw/cards-wide.png" alt="Stoneclaw acquisition and upgrades"><br>2026-10-29 · Stoneclaw forced targeting and inline radius/recovery upgrades. Wide/compact cards and third-card scrolling verified.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Jewelry rewards</strong><br><img src="docs/images/screenshots/2026-10-26-jewelry-preview/cards-1280.png" alt="Jewelry rewards"><br>2026-10-26 · Held preview cards for rapid UI iteration.</td>
    <td width="50%"><strong>Compact Talents</strong><br><img src="docs/images/screenshots/2026-10-29-earths-grasp/talent-compact-refund.png" alt="Earth's Grasp shows both next-rank bonuses inline"><br>2026-10-29 · Earth's Grasp purchase/refund verified at 640×720 / 125%; saved training reaches the totem cards after restart. Earlier milestones remain inline in the changelog.</td>
  </tr>
</table>


### Audio settings

![Audio settings with runtime string bindings at 800×720](docs/images/screenshots/2026-10-27-label-bindings/audio.png)

2026-10-27 · Shared starter/pause settings; English native check and translated text-binding regression coverage.

### Appearance labels

![Race-specific labels resolved from the game data](docs/images/screenshots/2026-10-27-appearance-localization/tauren.png)

2026-10-27 · Tauren horn labels at 800×720; English live check, translation and fallback unit coverage.

### Additional responsive Character check

<table>
  <tr>
    <td width="50%"><strong>Character at 1024×768</strong><br><img src="docs/images/screenshots/2026-10-27-druid-character/female-1024.png" alt="Dressed female Night Elf Druid"><br>2026-10-27 · Dressed Night Elf model; appearance changes retain equipped gear.</td>
    <td width="50%"><strong>Scrolled equipment</strong><br><img src="docs/images/screenshots/2026-10-27-druid-character/equipment-1024.png" alt="Druid equipment and bonuses"><br>2026-10-27 · All nine slots and summed bonuses in the responsive single-column layout.</td>
  </tr>
</table>

<table>
  <tr>
    <td width="50%"><strong>Dressed Orc Warlock · 1280×720</strong><br><img src="docs/images/screenshots/2026-10-27-warlock-character/orc-1280.png" alt="Dressed female Orc and separate equipment and appearance controls"><br>2026-10-27 · Three-column Character layout with independent gear looks.</td>
    <td width="50%"><strong>Gnome after restart · 640×720</strong><br><img src="docs/images/screenshots/2026-10-27-warlock-character/restarted-640.png" alt="Compact female Gnome with restored hairstyle and equipped gear"><br>2026-10-27 · Compact appearance changes persist without replacing gear.</td>
  </tr>
</table>

### Interface sizing

Options → Graphics → **Interface size** adjusts readability from 75% to 125%
without changing render resolution. The setting is saved between launches.

![Compact Options at 125% interface size](docs/images/screenshots/2026-10-27-interface-scale/options-640-125.png)

Collection and Talents also support compact scrolling at enlarged interface size.

![Compact talent trees at 125% interface size](docs/images/screenshots/2026-10-27-scaled-armory/trees-640-125.png)

![Compact companion rewards at 125% interface size](docs/images/screenshots/2026-10-27-scaled-rewards/companions-640-125.png)

![Saved tabard rendered on the character](docs/images/screenshots/2026-10-27-scaled-rewards/tabard-after.png)

![Long talent descriptions remain scrollable at enlarged interface size](docs/images/screenshots/2026-10-27-drain-life/talents-640-125.png)

![Populated compact Codex at 125% interface size](docs/images/screenshots/2026-10-27-scaled-codex/codex-640-125.png)

![Completed available Feats with pending achievements retained](docs/images/screenshots/2026-10-27-achievement-completion/feats-640-125.png)

![Saved Conflagrate at 125% interface size](docs/images/screenshots/2026-10-27-conflagrate/restarted-640-125.png)

![Preparation training and saved spec](docs/images/screenshots/2026-10-27-preparation/purchased.png)

### Browser starter checkpoint

2026-10-27 · WebGPU package at `7aa09ef`: 1280×1024 talent trees reflow above
scrollable details, with green Combustion scaling and zero-point purchase blocking.
At 2560×1080, all three trees and complete details fit side by side; ultrawide
pointer acceptance remains open. In-map browser gameplay and Windows scene
acceptance remain unverified.

![Ultrawide Mage talents](docs/images/screenshots/2026-10-27-browser-resolutions/talents-2560x1080.png)

<table><tr>
<td width="50%"><img src="docs/images/screenshots/2026-10-27-browser-resolutions/starter-1280x1024.png" alt="Starter at 1280 by 1024"></td>
<td width="50%"><img src="docs/images/screenshots/2026-10-27-browser-resolutions/combustion-1280x1024.png" alt="Full Combustion description reached by scrolling at 1280 by 1024"></td>
</tr></table>

### Earlier gameplay captures

<table>
  <tr>
    <td width="50%"><strong>Combat HUD</strong><br><img src="docs/images/screenshots/2026-10-24/combat.jpg" alt="Combat HUD"><br>2026-10-24 · Historical Elwynn run-start capture.</td>
    <td width="50%"><strong>Pause</strong><br><img src="docs/images/screenshots/2026-10-24/pause.jpg" alt="Pause"><br>2026-10-24 · Historical pause controls; settings have since changed.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Results</strong><br><img src="docs/images/screenshots/2026-10-24-results-layout/wide-720.jpg" alt="Results"><br>2026-10-24 · Historical defeat results; fresh end-to-end captures remain required.</td>
    <td width="50%"><strong>Foliage reveal</strong><br><img src="docs/images/screenshots/2026-10-25-cutaway-sizing/farther-zoom.jpg" alt="Foliage reveal"><br>2026-10-25 · Historical reveal sizing; not proof of the reported Windows occlusion fix.</td>
  </tr>
</table>

## In this build

- **A three-column front door and Armory** using client glue art, with character appearance,
  collection, talents and progression screens. The intended campaign lists **twelve zones**;
  destinations with missing world, encounter or drop content remain visibly locked. These
  screens are under development, and the full campaign is not yet playable.
- **Level-up cards** that deal in, tilt toward the cursor and lift on hover. Their WoW item-quality
  colour says how strong a pick is, and each card lists exactly what changes, like
  `Damage 20 » 26` or `Targets 2 » 3`.
- **A class movement spell on F** from level 5, aimed at the cursor: Blink, Charge, Sprint, Dash,
  Fade, Ghost Wolf, Aspect of the Cheetah, Blessing of Freedom, Howl of Terror.
- **A camera you can free:** locked top-down by default with the hero facing the cursor. **C**
  unlocks it to swing and tilt, and the wheel zooms from over the shoulder out to 70 yards.
- A light sharpen-and-colour pass over the picture (**G** toggles it).

## Play it

```bash
git clone https://github.com/erjdev/azeroth-survivors-re
cd azeroth-survivors-re
make build     # fetch game data + install toolchain + build the wasm (~5.5 GB download)
make start     # → open http://127.0.0.1:8090/
```

`make build` chains three re-runnable scripts, each skipping what's already done:

| Step | What it does |
| --- | --- |
| `scripts/fetch-data.sh` | downloads the WoW 1.12.1 client and extracts its MPQs |
| `scripts/web-setup.sh` | installs the wasm toolchain (wasi-sdk, wasm-bindgen, binaryen) |
| `scripts/web-build.sh` | builds the client into `web/dist/` |

**Prerequisites:** [Rust](https://rustup.rs/), `make`, `curl`, `unzip`, and a bash shell. On Windows, run the wasm scripts from Git Bash; PowerShell and cmd can't run them. The native game is the other way around: from Cursor's terminal, run `scripts\survivors.cmd`.

### Windows, native

```text
scripts\survivors.cmd
```

That runs `cargo survivors`: the `play` profile, release optimizations with incremental compilation. The first build compiles the engine. After that, an edit recompiles the crates that changed and leaves the rest of the stack in `target\play`. `cargo build --release` does not — it rebuilds the client crate from scratch, so it is the wrong daily loop.

The script points `WOW_DATA` at `wow-data\Data` when that folder is already there. The program it produces is `target\play\warcraft-survivors.exe`. Closing the window does not throw the build away; the next launch compiles only if the source changed.

Use `cargo survivors-rules` for the engine-independent rule tests. Native development builds
also reload `crates/survivors-core/tuning.txt` while running, so damage, movement, experience
and pickup multipliers can be adjusted without a Rust build. See
[development and distribution](docs/SURVIVORS-DEVELOPMENT.md) for cache boundaries, live
tuning validation, and the planned prompted updater.

Run `make` (or `make help`) to list every target.

## Controls

See [CONTROLS.md](CONTROLS.md). The short version: **WASD** to move, **Space** to jump, **F**
for your class's movement spell, the **mouse wheel** to zoom, **C** to unlock the camera, **Esc**
to pause, **1/2/3** to pick a level-up card.

In the starter or pause menu, a controller's **Start** opens Options. **D-pad up/down**
moves focus, **left/right** adjusts sliders, **south** selects, and **east** closes
Options. On the starter and Armory pages, **D-pad / arrow keys** move the focus
outline between controls; **south / Enter / Space** selects. Focus scrolls into
view automatically. **East** returns to the starter. Text entry still needs a
keyboard. Talents and milestones show descriptions when focused. **Backspace / controller
west** refunds one point from the focused talent, respecting row and prerequisite
requirements. During a run, **left stick** moves, **right stick** turns the camera,
**south** jumps, **bumpers/triggers** zoom and **Start** pauses. Keyboard movement
has priority. Controls saves a **0–50% stick dead zone** (25% default); release
sticks/buttons after resume or controller handoff. Physical-controller and Windows
gameplay acceptance remain open.

## The game data (not in this repo)

This repo ships **no Blizzard content** — no art, models, sounds, maps, MPQ contents, or
FrameXML. The game reads a WoW 1.12.1 client's `Data` folder at runtime, which you supply.

`make build` (via `scripts/fetch-data.sh`) downloads the **English WoW 1.12.1 (build 5875)**
client from [archive.org](https://archive.org/details/WoW_1.12.1-5875) (~5.5 GB) and extracts the
MPQs into `wow-data/Data/` (git-ignored, resumable, skipped when already present). Or point at a
client you already have with `--data` / `WOW_DATA`.

### Changing the download source

If the archive.org item moves or you have your own mirror, override the URL without editing
anything:

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

Or set it permanently by editing the single clearly-marked `DEFAULT_DATA_URL` line at the top of
`scripts/fetch-data.sh`. Either way the URL must be a direct link to a zip holding the client's
`Data/*.MPQ` files.

## Talent implementation coverage

Check authored talent definitions against your explicit game-data folder:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-coverage
cargo run -p survivors-audit -- wow-data/Data --talent-gaps
```

The first command summarizes each class; the second lists pending node IDs/names.
Identity mismatches fail the command. Missing effects are reported as incomplete;
these counts do not establish combat or visual acceptance. Current counts and
remaining requirements are in the [implementation plan](docs/SURVIVORS-IMPLEMENTATION.md#talent-definition-coverage--2026-10-26).

## How it's built

The survivors mode is `crates/benilla-app/src/survivors/`, running on two upstream projects:

- [benilla](https://github.com/samwhosung/benilla) — a from-scratch WoW 1.12.1 client.
- [wenilla](https://github.com/Arnesen/wenilla) — benilla compiled to WebAssembly, with a local
  host and realm service.

## Legal

This independent fan project is inspired by **Aethus’s Azeroth Survivors**. It is not
affiliated with or endorsed by Blizzard Entertainment and ships no Blizzard game files. The screenshots in `docs/images/screenshots/`
show World of Warcraft art rendered from a user-supplied client; that art is © Blizzard
Entertainment. World of Warcraft is a trademark of Blizzard Entertainment, Inc. The engine code in
this repository is licensed under
[MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option; the vendored components under
`third_party/` keep their own upstream licenses.

### Run Options

Native compact settings at 640×480 with a temporary 125% preview scale. Keyboard
navigation and return paths verified. Independent saved silhouette filters are now
available under Graphics (experimental, off by default).

![Saved controller settings at 640×480/125%](docs/images/screenshots/2026-10-28-dead-zone/compact-defaults.jpg)

Developer rendering checkpoint (2026-10-29): a noncombat native fixture verifies
full/partial obstruction, scenery behind actors, alpha-cutout holes and concealed
actors at 4× MSAA on Metal. Only obscured portions receive red/green. The renderer
remains off by default; independent enemy/friendly switches save and survive
restart. Overlap/performance, Windows and real Elwynn acceptance remain open.

![Partial obstruction preserves visible actor shading](docs/images/screenshots/2026-10-29-silhouette-blockers/partial-msaa4.png)

Native Metal overlap checks also verify that disabling friendly color preserves
its depth against a farther enemy; neutral bodies retain the same ownership.
This is a noncombat fixture, not real-map or Windows acceptance.

![Disabled friendly occludes the farther enemy silhouette](docs/images/screenshots/2026-10-30-silhouette-overlap/enemies.png)

![Compact Graphics with saved visibility filters](docs/images/screenshots/2026-10-29-silhouette-filters/graphics-compact.png)


![Run Options](docs/images/screenshots/2026-10-27-run-options/wide.png)
