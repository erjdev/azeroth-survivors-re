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

The starter shows worn equipment icons that open Collection details, the selected
zone's scheduled finale and progression locks. Narrow screens stack the three sections.

![Starter with expanded zone details](docs/images/screenshots/2026-10-25-zone-expansion/warsong-1280.jpg)

![Responsive starter at 640×720](docs/images/screenshots/2026-10-25-starter-portrait/mage-640.jpg)

Starter portraits now show faction and class badges, with wide/compact checks.

![Starter portrait identity](docs/images/screenshots/2026-10-25-starter-badges/alliance-1280.png)

Starter actions now use outlined cards with their supporting text inside the button.

![Starter action cards](docs/images/screenshots/2026-10-25-starter-actions/starter-1280.png)

Options opens shared audio, graphics and camera preferences without entering a run.

![Options Controls](docs/images/screenshots/2026-10-26-controls-speed/controls-1280.png)

![Compact Options navigation help](docs/images/screenshots/2026-10-26-options-controller/options-640.png)

![Starter focus and scroll reveal](docs/images/screenshots/2026-10-26-starter-focus/starter-scroll-640.png)

Talent descriptions follow directional focus before spending a point.

![Talent focus and rank preview](docs/images/screenshots/2026-10-26-talent-focus/talent-1280.png)

Talent focus and the green preview stay in place when buying or refunding ranks.

![Talent focus after editing ranks](docs/images/screenshots/2026-10-26-talent-retain/focus-after-purchase-1280.png)

All nine Mage milestones now have implemented effects; individual pending nodes remain.

![Mage Presence of Mind milestone](docs/images/screenshots/2026-10-26-mage-milestones/presence-of-mind-1280.png)

Shaman milestones add lightning, shock and totem build effects.

![Shaman Mana Tide milestone](docs/images/screenshots/2026-10-26-shaman-milestones/mana-tide-1280.png)

Shaman's first 13 individual talents now provide spell and defensive training.

![Shaman Convection training preview](docs/images/screenshots/2026-10-26-shaman-nodes/convection-1280.png)

Warlock's first 12 individual talents now train shadow, fire, curses and health.

![Warlock Corruption training](docs/images/screenshots/2026-10-26-warlock-nodes/corruption-1280.png)

Druid's first 12 talents add Balance, defensive and healing effects with explicit previews.

![Druid Moonfire damage and critical chance preview](docs/images/screenshots/2026-10-26-druid-nodes/moonfire-1280.png)

All nine Warlock milestones now have implemented Survivors effects.

![Warlock Siphon Life milestone](docs/images/screenshots/2026-10-26-warlock-milestones/siphon-life-1280.png)

Rewards keeps keyboard/controller focus while staging or undoing upgrades.

![Rewards focus after two keyboard upgrades](docs/images/screenshots/2026-10-26-reward-focus/two-upgrades-1280.png)

Achievements uses full-row category controls for pointer and keyboard navigation.

![Achievement category navigation](docs/images/screenshots/2026-10-26-achievement-nav/summary-1280.png)

New achievements retain their earned dates; older saves keep undated history.

![Achievement earned dates](docs/images/screenshots/2026-10-26-achievement-dates/summary-1280.png)

Achievement cards now have artwork themed to their requirements.

![Achievement artwork](docs/images/screenshots/2026-10-26-achievement-art/summary-1280.png)

Jeweler’s Eye now tracks discovery across the full jewelry Codex.

![Combined jewelry discovery achievement](docs/images/screenshots/2026-10-26-jewel-achievement/progress-1280.png)

Druid milestones now support Balance spells, Thorns defenses and Rejuvenation healing.

![Druid Swiftmend milestone](docs/images/screenshots/2026-10-26-druid-milestones/swiftmend-1280.png)

Legendary rewards a complete legendary gear collection for one class.

![Legendary collection earned](docs/images/screenshots/2026-10-26-legendary-achievement/earned-1280.png)

Tabard cards share Armory selection feedback and keyboard focus.

![Tabard selection](docs/images/screenshots/2026-10-26-tabard-controls/worn-1280.png)

Rapid menu navigation followed by selection now applies in the same frame.

![Keyboard selection after rapid navigation](docs/images/screenshots/2026-10-26-coalesced-input/selected-1280.png)

Fast preview files can open Rewards and Achievement categories directly.
See [the preview guide](docs/SURVIVORS-DEVELOPMENT.md) for reusable requests.

![Direct Feats preview](docs/images/screenshots/2026-10-26-preview-categories/feats-1280.png)

Achievement cards retain consistent widths on incomplete rows.

![Consistent achievement card widths](docs/images/screenshots/2026-10-26-achievement-widths/feats-1280.png)

Druid Restoration training previews explicit healing, cooldown and defensive benefits.

![Druid Restoration healing preview](docs/images/screenshots/2026-10-26-restoration-nodes/natures-focus-1280.png)

Companion looks share the selection styling and keep locked appearances visibly unavailable.

![Companion appearance controls](docs/images/screenshots/2026-10-26-companion-controls/looks-1280.png)

Companion appearance unlocks are checked when loading; valid independent choices persist across restart.

![Companion looks after restart](docs/images/screenshots/2026-10-26-companion-save/restarted-1280.png)

Character retains the reference's three-pane layout on wide screens and stacks a
proportional preview, appearance and equipment on narrow screens. Captures use an
isolated synthetic account; saved appearance and tab-return scrolling were checked.

![Current Character interface](docs/images/screenshots/2026-10-25-character-responsive/character-1280.jpg)

![Responsive Character at 640×900](docs/images/screenshots/2026-10-25-character-responsive/character-640x900.jpg)

Collection stacks into readable sections on narrow screens and retains its three-pane
layout at wider sizes. Item actions and tab return preserve narrow scrolling.

![Responsive Collection at 640×720](docs/images/screenshots/2026-10-25-collection-responsive/collection-640.jpg)

Gear now has distinct [slot stat progressions](docs/SURVIVORS-GEAR-STATS.md). Higher
quality strengthens existing stats and adds one; duplicates add no power. Initial
tuning shown here uses synthetic collected equipment.

![Epic chest bonuses and updated equipment totals](docs/images/screenshots/2026-10-25-gear-profiles/equipped-epic-1280.jpg)

Drop Tables now wraps zone choices on narrow screens and preserves scrolling through
item selection and tab return. Wide screens retain source and item details side by side.

![Drop Tables at 1280×720](docs/images/screenshots/2026-10-25-drop-tables-responsive/drops-1280.jpg)

Codex distinguishes legendary jewelry with spell badges and highlights the selected
entry, with larger detail art. Discoveries below are synthetic preview data.

![Codex jewelry and spell badges](docs/images/screenshots/2026-10-25-codex-icons/legendary-1280.jpg)

Talent trees now use the available page height, closer to the intention layout.

![Illustrated specialization headings and talent milestones](docs/images/screenshots/2026-10-25-talent-headings/paladin-1280.jpg)

Saved Talent builds were exercised across Warrior/Mage switching and restart;
resetting one spec preserves the other builds.

![Warrior saved spec restored after restart and hero switching](docs/images/screenshots/2026-10-25-talent-specs/warrior-restored.jpg)

Talent panels reflow at 640×720; scrolling reaches the lower tree and spec controls.
Spend/refund and tab-return scroll continuity were exercised with an isolated Warrior.

![Responsive Talents at 640×720](docs/images/screenshots/2026-10-25-talent-responsive/trees-640.jpg)

Rewards now groups currency and reset controls beside illustrated Perk cards.
Purchase/refund and narrow stacking were checked in an isolated account; staged
purchases and the remaining Rewards pages are still in progress.

![Rewards sidebar and inline Perk previews](docs/images/screenshots/2026-10-25-reward-previews/perks-1280.jpg)

Companion cards now show green inline upgrades, rank pips and the destination for
instinct purchases. Live purchases and narrow scrolling were checked; the expanded, achievement-gated
Looks catalog remains unfinished.

![Companion cards and upgrade previews](docs/images/screenshots/2026-10-25-compact-companions/companions-1280.jpg)

Companion appearances now have illustrated tiles and a green Worn state. Each slot's
look remains independent of its instinct; narrow layout and restart persistence were checked.

![Companion appearance tiles](docs/images/screenshots/2026-10-25-companion-art/companions-1280.jpg)

The Companion catalog now lists thirteen looks. New accounts unlock Westfall Chicken
with Veteran; existing saves retain access. Panda Cub and Sapphiron are locked behind
**Deathless Hour** and **no u**.
These achievements are visible placeholders; their criteria will be defined later.

![Companion catalog with placeholder achievement gates](docs/images/screenshots/2026-10-25-placeholder-feats/locked-looks-1280.jpg)

Long Leash and the remaining five mismatched looks now use the intention's icon artwork.

![Companion reference artwork](docs/images/screenshots/2026-10-25-companion-reference-icons/looks-1280.png)

Tabards now show illustrated Wear/Remove choices, with immediate account-wide saving.
The current catalog contains one verified Guild Tabard.

![Tabard cosmetic selection](docs/images/screenshots/2026-10-25-tabards/worn-1280.jpg)

Rewards purchases now preview before **Confirm**, with **Undo** to discard pending
changes. Confirmed purchases persist; unconfirmed choices are discarded on app close.

Each card also lets you undo one pending upgrade without discarding the rest.

![Pending rewards with individual undo controls](docs/images/screenshots/2026-10-25-reward-minus/three-pending-1280.jpg)


Scarlet Monastery now draws its authored building minimap, including zoom and enemy
markers. The candidate zone stays locked pending navigation and full encounter acceptance.
[Outdoor minimap regression](docs/images/screenshots/2026-10-25-indoor-minimap/elwynn.png)
(the close camera remains a separate unresolved issue).

![Scarlet building minimap](docs/images/screenshots/2026-10-25-indoor-minimap/scarlet.png)

Ahn'Qiraj outdoor roster milestone: an isolated accelerated Warrior run exercises the
new Legendary armor encounters. The zone remains locked pending world acceptance;
this screenshot does not establish normal progression or balance.

![Ahn'Qiraj level-up during the roster probe](docs/images/screenshots/2026-10-25-aq-roster/level-up.png)

Defeat results at 1280×720, captured with an isolated save and temporarily reduced damage/XP
tuning to reach defeat quickly. This validates the displayed results, not gameplay balance.

![Defeat results](docs/images/screenshots/2026-10-24-results-layout/wide-720.jpg)

Results resize live: [640×720](docs/images/screenshots/2026-10-24-results-layout/narrow-720.jpg)
and [480×720 with wrapped stat tiles](docs/images/screenshots/2026-10-24-results-layout/wrapped-720.jpg).

Un'Goro location investigation: early combat on the southern candidate, after moving away
from its central tree. Full traversal, camera and encounter acceptance remain open; the zone
stays locked. [Earlier free-camera view](docs/images/screenshots/2026-10-25-ungoro-candidate/world-preview-720.jpg).

![Un'Goro candidate early combat at 720p](docs/images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)

Elwynn tree investigation: [approach](docs/images/screenshots/2026-10-24-tree-contact/approach.jpg),
[forward contact](docs/images/screenshots/2026-10-24-tree-contact/contact.jpg),
[sidestep](docs/images/screenshots/2026-10-24-tree-contact/side.jpg).
This trunk blocks the observed approach but obscures the hero; broader collision validation remains.

Scenery cutaway now reveals the hero and nearby combat through foliage, with a saved toggle
independent of camera auto-push. Initial 720p validation, with temporary XP tuning:

The viewport-scaled reveal also stays centered after zooming farther out (1280×720,
retained renderer). [Same-camera on](docs/images/screenshots/2026-10-25-cutaway-sizing/on.jpg) /
[off](docs/images/screenshots/2026-10-25-cutaway-sizing/off.jpg).

![Viewport-scaled reveal after zooming out](docs/images/screenshots/2026-10-25-cutaway-sizing/farther-zoom.jpg)

Earlier fixed-radius validation:

![Foliage cutaway during combat](docs/images/screenshots/2026-10-25-cutaway/retained-gameplay.jpg)

Same paused scene: [cutaway on](docs/images/screenshots/2026-10-25-cutaway/retained-on.jpg) /
[off](docs/images/screenshots/2026-10-25-cutaway/retained-off.jpg).
The ordinary model renderer also passes: [gameplay](docs/images/screenshots/2026-10-25-cutaway/ordinary-gameplay.jpg),
[on](docs/images/screenshots/2026-10-25-cutaway/ordinary-on.jpg) /
[off](docs/images/screenshots/2026-10-25-cutaway/ordinary-off.jpg).
Interior visibility, colored silhouettes and the full zoom/resolution matrix remain unfinished.

[Fresh run after Play again](docs/images/screenshots/2026-10-24-defeat/replay-720.jpg):
paused at 00:02 with full health and zero damage, using the same isolated validation setup.

Pick a class and a battleground, then survive: WASD to move, your abilities cast themselves,
enemies swarm in waves, you level up and pick upgrades, and a boss shows up every couple of
minutes. **No server, no login, no account** — fully offline and self-contained.

Un'Goro source milestone: nine timed encounters now cover all uncommon slots and share
their schedule with the drop browser. The earlier capture below shows the first six.
The zone remains world-locked while arena validation continues.

![Un'Goro sources and availability at 720p](docs/images/screenshots/2026-10-25-ungoro-drops/sources-720.png)

[King Mosh item detail](docs/images/screenshots/2026-10-25-ungoro-drops/final-item-720.png)

Feralas candidate: a terrain-supported forest corridor with nearby root and tree hulls.
This free-camera preview establishes rendered geometry, not gameplay-camera or traversal
acceptance; the zone remains locked.

![Feralas candidate world preview at 720p](docs/images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)

Feralas live validation: an accelerated Warrior fixture reached 219 enemies and persisted
four Rare armor drops in an isolated save. This level-up capture is from a 1280×720 logical
window (Retina pixels); it is not a performance or complete-zone acceptance result.

![Feralas live Warrior level-up](docs/images/screenshots/2026-10-25-feralas-live/late-warrior.png)

Collection content milestone: the uncommon armor row now has real item identities for
all nine classes. Hunter item details and pending-source messaging were checked at
1280×720 and 1024×720. Slot headings are centered and separated at both sizes.

![Hunter uncommon armor collection at 720p](docs/images/screenshots/2026-10-25-collection-headings/collection-720.png)

[Narrow collection layout](docs/images/screenshots/2026-10-25-collection-headings/collection-1024.png)

Rare armor milestone: all nine classes now have eight named Rare armor pieces, bringing
the catalog to 27 of 54 identities per class in this capture. The subsequent nine-weapon
addition brought it to 28 of 54; the Epic identity tier now brings the catalog to 37 of 54.
This isolated Warrior fixture seeds ownership to inspect
the UI. Rare sources now have a Feralas encounter table, while the zone remains locked
pending world acceptance; this older capture predates those source labels.

![Warrior Rare collection and pending source at 720p](docs/images/screenshots/2026-10-25-rare-armor/collection-720.png)

[Human male armor preview](docs/images/screenshots/2026-10-25-rare-armor/character-720.png)
is a visual checkpoint, not complete race/sex or attachment acceptance.

Chest rewards now use verified item artwork. This isolated accelerated fixture holds
chest choices for inspection: [rings and amulet](docs/images/screenshots/2026-10-25-chest-item-icons/rings-and-amulet.png),
then trinkets after manual selections. Mark of the Champion's ambiguous identity remains unresolved.

![Chest trinket item icons](docs/images/screenshots/2026-10-25-chest-item-icons/trinkets.png)

Combat-text placement checkpoint: [paused chest with preserved damage numbers](docs/images/screenshots/2026-10-25-combat-text-cache/chest.png).
Repeated identical overlap solves are cached; resumed combat was checked. A later foreground
probe held about 90 FPS at a chest and measured 45–75 FPS in short combat samples.
Earlier background FPS was not representative of attended play; see the
[profiling method](docs/SURVIVORS-DEVELOPMENT.md#measure-attended-performance-explicitly).
Full crowd and zone performance acceptance remains open.

Gurubashi live checkpoint: an isolated accelerated Warrior run starts at 10:00 with
max-rank abilities. The captured defeat at 10:36 reports six bosses and 409 kills; this
is not a full-length run or balance acceptance. Slow combat samples remain under investigation.

![Gurubashi accelerated Warrior results](docs/images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png)

Catalog coverage: every class now has all **54 collection cells named and mapped**.
This completes identity coverage, not acquisition or gameplay acceptance: missing drop
sources, live appearance checks and raid-set mechanics remain tracked in the plan.

PvP identity milestone: this isolated Warrior fixture seeds all 54 owned cells and equips
the PvP row to inspect names and bonuses. It was launched before the Gurubashi schedule
change, so its zone-lock wording is historical. Ownership is seeded, not earned here;
full appearances, acquisition and raid-set powers still require acceptance.

![Warrior PvP equipped menu](docs/images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png)

Epic identity milestone: this capture shows 37 of 54 named cells; the subsequent Legendary
armor addition brings every class to 45 of 54. This isolated
Warrior account seeds the Wrath set and Ashkandi to inspect equipped names and bonuses;
Epic drops, raid variants/set powers, and full appearance acceptance remain unfinished.

![Warrior Epic equipped menu](docs/images/screenshots/2026-10-25-epic-catalog/equipped-menu.png)

Hunter talent milestone: Mortal Shots has a green next-rank preview, and spending/resetting
Spec 2 leaves Spec 1 intact. Verified at 1280×720 and after a live resize to 1024×720;
the saved allocation also survived relaunch. Thirteen Hunter nodes have explicit effects;
remaining nodes, pets and milestones are unfinished.

![Hunter Mortal Shots rank preview at 720p](docs/images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg)

[Independent second spec](docs/images/screenshots/2026-10-25-hunter-talents/spec-two-720.jpg) ·
[Narrow talent layout](docs/images/screenshots/2026-10-25-hunter-talents/mortal-shots-1024.jpg)

Mage talent milestone: green rank previews at 1440×900 and 1280×720. Permafrost
compares both slow strength and duration. Refunding a rank, closing directly from
Talents, and relaunching preserves the edit; the separate Fire spec stays intact.
These captures verify the UI, not live combat or completion of every Mage talent.

![Winter's Chill final-rank preview at 720p](docs/images/screenshots/2026-10-25-mage-talents/winters-chill-preview-720.png)

Winter's Chill spending/refunding and the [maximum-rank description](docs/images/screenshots/2026-10-25-mage-talents/winters-chill-max-720.png)
were checked at 720p with an isolated Mage account; combat feel remains under validation.

Frost Nova's longer description, including trained shield and immunity, fits at
1280×720 and [1024×768](docs/images/screenshots/2026-10-25-mage-talents/frost-nova-card-1024.jpg).
Selecting the same hand after resizing grants rank two; live defensive timing remains open.

![Frost Nova defensive upgrade card at 720p](docs/images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)

![Permafrost strength and duration preview at 720p](docs/images/screenshots/2026-10-25-mage-talents/permafrost-720.png)

[Shatter](docs/images/screenshots/2026-10-25-mage-talents/shatter-preview.png) ·
[Improved Blizzard](docs/images/screenshots/2026-10-25-mage-talents/blizzard-preview.png) ·
[Ignite in spec two](docs/images/screenshots/2026-10-25-mage-talents/ignite-spec-two.png) ·
[Frostbite](docs/images/screenshots/2026-10-25-mage-talents/frostbite-720.png) ·
[Allocation after relaunch](docs/images/screenshots/2026-10-25-mage-talents/reloaded-spec-720.png)

Warsong Gulch now has an authored faction-survival roster and nine Uncommon boss
reward sources. This isolated full-pool Mage run verifies early rendering at 720p;
the zone remains locked pending traversal, camera, complete encounters and balance.

![Warsong Gulch faction-survival gameplay at 720p](docs/images/screenshots/2026-10-25-warsong-roster/survival-720.jpg)

Arathi Basin now has ten faction wave entries and nine Rare boss reward sources.
This isolated full-pool Mage run verifies early rendering at 720p; the stables-area
candidate remains locked pending traversal, camera and complete encounter validation.

![Arathi Basin faction-survival gameplay at 720p](docs/images/screenshots/2026-10-25-arathi-roster/survival-720.jpg)

Alterac Valley's central candidate adds ten wave entries and nine bosses with PvP-band
rewards. This 720p preview confirms early terrain, enemies, spells and HUD rendering;
full encounters, traversal and camera validation remain open.

![Alterac Valley survival gameplay at 720p](docs/images/screenshots/2026-10-25-alterac-roster/survival-720.jpg)

[Inline upgrade comparisons during the Alterac run](docs/images/screenshots/2026-10-25-alterac-roster/upgrades-720.jpg).

Native development can now open a specific starter page directly with
`WOW_SURVIVORS_UI_PAGE=collection`, without a gameplay session.
[Direct Collection preview](docs/images/screenshots/2026-10-25-ui-preview/collection-direct.jpg)
· [preview instructions](docs/SURVIVORS-DEVELOPMENT.md#starter-ui-iteration-priority).
Use `python3 scripts/survivors-ui-fixture.py partial` to prepare an isolated account
for UI checks; [fresh, partial and collected fixtures](docs/SURVIVORS-DEVELOPMENT.md#deterministic-account-fixtures)
avoid earning test progress through gameplay.

All eight playable races now have authored Survivors bonuses, with exact rules
shown on the starter. Initial values still need combat balance testing.

![Orc racial bonuses](docs/images/screenshots/2026-10-25-racial-bonuses/orc-1280.jpg)

Sixteen Rogue talents now have explicit effects and truthful rank previews.
Rogue coverage is still incomplete. Imported specs that exceed earned points retain
their saved ranks, but talent effects stay inactive until the point budget is repaired.

![Rogue Vigor cooldown talent](docs/images/screenshots/2026-10-25-rogue-cooldowns/vigor-1280.jpg)

The Collection detail card groups item identity following the intention design;
long names wrap with the available panel width. Equipment comparisons preview
gains and losses, including the worn-set bonus. Shift-right-click selects a collected
look independently of worn gear; native macOS validation retained the common
helmet’s bonuses while selecting its legendary appearance.

![Collection with common helmet equipped and legendary appearance selected](docs/images/screenshots/2026-10-25-native-look-shortcut/cosmetic-1280.jpg)

Character appearance controls use bordered, responsive rows based on the intention
design. The counters update in place, preserving compact-page scroll position.
A focused Look section keeps appearance actions visible, with selected-item
navigation to Collection for full equipment details.

![Character appearance controls and Look section](docs/images/screenshots/2026-10-25-character-look/character-1280.jpg)

Priest now has nineteen individual talents and nine tree milestones with scoped previews.
Fortitude, Inner Fire and Spell Warding now provide health, armor and damage reduction.

![Priest defensive talent preview](docs/images/screenshots/2026-10-25-priest-defenses/inner-fire-1280.jpg)

Blackout adds a stun chance to Shadow hits and Pain ticks, shown inline on cards.

![Shadow cards with Blackout](docs/images/screenshots/2026-10-25-blackout/cards-1280.jpg)

Vampiric Embrace heals from Shadow damage, with both improvement ranks implemented.

![Shadow cards with healing and control](docs/images/screenshots/2026-10-25-vampiric-embrace/cards-1280.jpg)

Shadow Weaving builds a separate, expiring vulnerability on each enemy.

![Shadow Weaving and combined talent descriptions](docs/images/screenshots/2026-10-25-shadow-weaving/cards-1280.jpg)

Long ground, chain and damage-over-time cards have checked [wide](docs/ui-previews/long-cards-1280.txt) and [compact](docs/ui-previews/long-cards-640.txt) preview presets.

![Long card descriptions](docs/images/screenshots/2026-10-25-long-cards/wide-1280.jpg)

Card layouts can be opened and resized without a run using the [live UI preview](docs/SURVIVORS-DEVELOPMENT.md).

![Direct production card preview](docs/images/screenshots/2026-10-25-card-preview/wide-1280.jpg)

Upgrade cards now state effective casting distance separately from area radius.

![Smite card with trained casting range](docs/images/screenshots/2026-10-25-card-range/smite-1280.jpg)

Talent-triggered healing is included inline, with its once-per-cast limit.

![Holy Fire card with trained healing](docs/images/screenshots/2026-10-25-card-healing/holy-fire-1280.jpg)

The reloadable preview can use the selected character's starting build, without a
combat session. High-rank talent healing fits on compact cards.

![Selected-build card preview at 640×720](docs/images/screenshots/2026-10-25-selected-card-preview/compact-640.jpg)

![Force of Will damage and critical chance](docs/images/screenshots/2026-10-25-force-of-will/talent-1280.jpg)

![Priest range and area talent](docs/images/screenshots/2026-10-25-priest-reach/holy-1280.jpg)

![Spirit Tap recovery preview](docs/images/screenshots/2026-10-25-spirit-tap/talent-1280.jpg)

![Priest spell-training preview](docs/images/screenshots/2026-10-25-priest-training/searing-light-1280.jpg)

![Priest Holy milestone](docs/images/screenshots/2026-10-25-priest-milestones/lightwell-1280.jpg)

Saved talent specs now follow the catalog's class, rank, row and prerequisite rules.
Invalid specs retain their ranks for repair and grant no talent effects.

![Saved-spec repair guidance](docs/images/screenshots/2026-10-25-talent-catalog/invalid-1280.jpg)

Compact hero switching restores each hero's saved appearance and class gear.

![Ten-hero management at 640×720](docs/images/screenshots/2026-10-25-character-acceptance/roster-640.jpg)

## Interface gallery

Actual checkout captures from **2026-10-24**, at a **1440 × 900 logical viewport**
(2880 × 1800 Retina render, UI scale 1.171875), exported to 1440 × 900 JPEGs.
These show the current implementation, including unfinished pages; they are not the
[intended designs](docs/intentions/) or proof of responsiveness at every resolution.
See [capture instructions](docs/SURVIVORS-DEVELOPMENT.md#readme-captures) to refresh them.

<table>
  <tr>
    <td width="50%"><strong>Front door</strong><br><img src="docs/images/screenshots/2026-10-25-display-headings/starter-1280.jpg" alt="Front door: Class, hero and twelve intended destinations; incomplete content is visibly locked."><br>Class, hero and twelve intended destinations; incomplete content is visibly locked.</td>
    <td width="50%"><strong>Character</strong><br><img src="docs/images/screenshots/2026-10-25-armory-headings/character-1280.jpg" alt="Character: Equipped item icons, selection details and dressed preview on a seeded validation account, shown at 1280×720."><br>Equipped item icons, selection details and dressed preview on a seeded validation account, shown at 1280×720.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Collection</strong><br><img src="docs/images/screenshots/2026-10-25-section-dividers/collection-1280.jpg" alt="Collection: Slot and rarity catalog, item details and equipment controls on a validation account."><br>Slot and rarity catalog, item details and equipment controls on a validation account.</td>
    <td width="50%"><strong>Drop Tables</strong><br><img src="docs/images/screenshots/2026-10-25-armory-headings/drops-1280.jpg" alt="Drop Tables: Zone browser, encounter sources and shared item details; missing content remains."><br>Zone browser, encounter sources and shared item details; missing content remains.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Codex</strong><br><img src="docs/images/screenshots/2026-10-25-armory-headings/codex-1280.jpg" alt="Codex: Shared jewelry catalog and legendary spell badges at 1280×720; undiscovered entries are dimmed and one item icon remains unresolved."><br>Shared jewelry catalog and legendary spell badges at 1280×720; undiscovered entries are dimmed and one item icon remains unresolved.</td>
    <td width="50%"><strong>Talents</strong><br><img src="docs/images/screenshots/2026-10-25-armory-headings/talents-1280.jpg" alt="Talents: Warrior tree presentation; this image does not imply every talent effect is implemented."><br>Warrior tree presentation; this image does not imply every talent effect is implemented.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Achievements</strong><br><img src="docs/images/screenshots/2026-10-25-section-dividers/achievements-1280.jpg" alt="Achievements: Illustrated achievement cards, account points and progress."><br>Illustrated achievement cards, account points and progress.</td>
    <td width="50%"><strong>Rewards</strong><br><img src="docs/images/screenshots/2026-10-25-reward-rail/rewards-1280.jpg" alt="Rewards: Emphasized points, full-width categories and compact perk controls."><br>Emphasized points, full-width categories and compact perk controls.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Appearance</strong><br><img src="docs/images/screenshots/2026-10-24-character-features/character-portraits.jpg" alt="Appearance: Per-hero race, sex and feature editing with live preview; shown at 1280×720."><br>Per-hero race, sex and feature editing with live preview; shown at 1280×720.</td>
    <td width="50%"><strong>Run Options</strong><br><img src="docs/images/screenshots/2026-10-24/run-options.jpg" alt="Run Options: Currently displays a difficulty notice, not a dedicated options dialog."><br>Currently displays a difficulty notice, not a dedicated options dialog.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Combat HUD</strong><br><img src="docs/images/screenshots/2026-10-24/combat.jpg" alt="Combat HUD: Elwynn at run start: health, experience, minimap, abilities and damage meter."><br>Elwynn at run start: health, experience, minimap, abilities and damage meter.</td>
    <td width="50%"><strong>Level-up cards</strong><br><img src="docs/images/screenshots/2026-10-24-upgrade-benefits/level-up.jpg" alt="Level-up cards: Green scaling and next-rank previews; captured at 1440×900."><br>Green scaling and next-rank previews; captured at 1440×900.</td>
  </tr>
  <tr>
    <td width="50%"><strong>Pause</strong><br><img src="docs/images/screenshots/2026-10-24/pause.jpg" alt="Pause: Resume, settings, abandon and quit controls."><br>Resume, settings, abandon and quit controls.</td>
    <td width="50%"><strong>Settings</strong><br><img src="docs/images/screenshots/2026-10-24/settings.jpg" alt="Settings: Sound, graphics and the persisted auto-push camera toggle."><br>Sound, graphics and the persisted auto-push camera toggle.</td>
  </tr>
</table>

[Warrior talent-effect milestone at 720p](docs/images/screenshots/2026-10-24-warrior-talents/talents-720.jpg).
Twenty-five Warrior nodes, including [defensive talents](docs/images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg), plus the
[Arms](docs/images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg),
[Fury](docs/images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg) and
[Protection](docs/images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg) milestones
have implemented effects; the complete class talent pass remains underway.


[Piercing Howl node](docs/images/screenshots/2026-10-24-warrior-talents/piercing-howl-node-720.jpg) · [Concussion Blow node](docs/images/screenshots/2026-10-24-warrior-talents/concussion-blow-node-720.jpg).

Reactive talent status: [active Flurry at 720p](docs/images/screenshots/2026-10-24-reactive-buffs/flurry-720.jpg) · [cleared after expiry](docs/images/screenshots/2026-10-24-reactive-buffs/expired-720.jpg). Enrage, Flurry and Blood Craze show their effect values and remaining seconds; dedicated icons remain pending.

[Active Enrage HUD at 720p](docs/images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg): green damage bonus and remaining seconds, verified through front-door entry with a trained Fury spec.
[Trained shout card](docs/images/screenshots/2026-10-24-warrior-talents/trained-shout-card-720.jpg): current duration and green next-rank comparison include the selected talents.

[Deep Wounds ranks](docs/images/screenshots/2026-10-24-warrior-talents/deep-wounds-ranks-720.jpg) · [Improved Rend](docs/images/screenshots/2026-10-24-warrior-talents/improved-rend-720.jpg) · [Shout duration talent](docs/images/screenshots/2026-10-24-warrior-talents/shout-duration-720.jpg) · [Green talent rank comparisons at 720p](docs/images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg).

World validation: [Warsong developer preview at 720p](docs/images/screenshots/2026-10-24-world-preview/warsong-720.jpg) · [reproduce the preview](docs/SURVIVORS-LOCATIONS.md#developer-instance-preview). Rendering verified; traversal remains pending.

Upgrade previews at 1280×720: [new spells and next rank](docs/images/screenshots/2026-10-24-codex-discovery/level-up-720.jpg) · [chest upgrades](docs/images/screenshots/2026-10-24-codex-discovery/chest-720.jpg).

Distinct chest and banish validation at 1280×720:
[before banish](docs/images/screenshots/2026-10-25-banish/before-720.png) ·
[replacement with neighbors preserved](docs/images/screenshots/2026-10-25-banish/after-720.png) ·
[single-card chest](docs/images/screenshots/2026-10-25-banish/single-chest-720.png) ·
[jewelry replacement](docs/images/screenshots/2026-10-25-banish/chest-replacement-720.png).
These captures preceded the chest subtitle correction to “Choose your chest reward.”

Card lighting without the rectangular outer glow: [hover at 1280×720](docs/images/screenshots/2026-10-24-card-lighting/hover-720.jpg) · [after live resize to 1280×800](docs/images/screenshots/2026-10-24-card-lighting/hover-800.jpg).

Current inline upgrades use one unit per comparison and keep rank labels beneath the name.
![Inline upgrade cards at 1280×720](docs/images/screenshots/2026-10-24-inline-units/cards-720.jpg)

Same hand after live resize: [1280×800](docs/images/screenshots/2026-10-24-inline-units/cards-800.jpg) · [1024×768](docs/images/screenshots/2026-10-24-inline-units/cards-1024.jpg).
[Area, cooldown and percentage comparisons at 1024×768](docs/images/screenshots/2026-10-24-inline-units/area-1024.jpg).
Cleave selection after resizing granted rank two. These captures cover sampled level-up
cards; chest/jewelry states now have separate 720p evidence above. The full resolution/UI-scale matrix remains open.


Character comparisons: [inline race/sex controls](docs/images/screenshots/2026-10-24-character-appearance/orc.jpg) · [large weapon](docs/images/screenshots/2026-10-24-equipment-framing/weapon-appearance.jpg) · [1280×720](docs/images/screenshots/2026-10-24-equipment-framing/character-720.jpg).

Current victory/defeat results, reward claims, equipped appearances and responsive
viewport variants still need fresh captures. Earlier screenshots below are historical.

<details>
<summary>Earlier checkout captures</summary>

<table>
  <tr>
    <td width="50%"><img src="docs/images/screenshots/legacy/menu.jpg" alt="The pick screen: race, gender and class on the left, sixteen zones on the right, the hero in between"></td>
    <td width="50%"><img src="docs/images/screenshots/legacy/combat.jpg" alt="A Mage holding off Elwynn Forest's wolves with frost, fire and arcane"></td>
  </tr>
  <tr>
    <td><img src="docs/images/screenshots/legacy/level-up.jpg" alt="Level up: three spell cards, coloured by rarity, each showing exactly what its next rank changes"></td>
    <td><img src="docs/images/screenshots/legacy/results.jpg" alt="The results screen after a death in Redridge Mountains"></td>
  </tr>
</table>

</details>

### In this build

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
requirements. Other hover details and controller gameplay remain unfinished.

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
