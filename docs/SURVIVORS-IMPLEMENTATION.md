# Survivors implementation contract

This plan implements [the Armory specification](SURVIVORS.md) and its
[intention images](intentions/). It is a work breakdown and acceptance contract, not a
replacement design or a claim that the illustrated game already exists. Git records
completed changes. Each phase must pass its acceptance criteria before it is called done.

## Visual reporting acceptance (hard requirement)

Every major UI checkpoint includes actual inline screenshots in `docs/CHANGELOG.md`,
stored under `docs/images/screenshots/<milestone>/`, plus a concise summary and
feature-branch commit/push. Links alone are insufficient. Compare against the matching
intention screenshot as the visual north star; record intentional departures and
unfinished details. Keep README screenshots current. Follow the
[checkpoint workflow](SURVIVORS-DEVELOPMENT.md#following-feature-milestones).
The 2026-10-25 starter checkpoint captures Character, ten-hero management, Talents
and Companions at 1280×720. Responsive variants and interaction acceptance remain open.

## Current priority — starter interface first (2026-10-26)

Achievement cards now keep the same proportional column width on incomplete rows,
matching the intention's aligned columns. Native wide/compact Feats captures verify
five-card alignment, readable text and scrolling. The shared card style also serves
recent/almost-complete summaries; broader layout acceptance remains open.

Fast native UI previews now support all three Rewards categories and all six
Achievement categories through a validated optional `category` key. Page, category
and dimensions apply together; omission preserves the manual category and invalid
combinations retain the previous view. Reusable wide Feats/compact Tabards requests
and documentation are checked in. All 261 app tests pass; native direct launch,
invalid-request retention and cross-page live reload have inline screenshot evidence.
Broader fixture automation and remaining page-specific preview controls stay open.

Tabard cards now use shared hover/pressed feedback, gold outlines and burgundy
selection. Native wide/compact validation covers icon selection, removal, keyboard
focus/selection, scrolling and saved appearance after restart. Inline screenshots
are in CHANGELOG/README. The single verified Guild Tabard remains the catalog;
in-world rendering and additional cosmetics remain open. The rapid direction-plus-select
issue is now fixed: when both arrive in one frame, navigation runs before selection.
An initial combined gesture establishes focus without activating; Enter is consumed
so it cannot fall through to starting a run. Keyboard/controller ECS regression
coverage verifies the destination, one-frame activation and modal blocking. Native
Right+Space and Left+Enter sequences now work without a second press; this does not
establish physical-controller or arbitrary multi-event ordering acceptance.

Achievement category rows now share one full-width action/focus surface with
separate label/count alignment and progress beneath, following the summary
reference. Native wide/compact checks cover icon and progress clicks, whole-row
focus, Space selection and compact summary scrolling. Inline evidence is in the
changelog/README. First-earned UTC dates now persist for new grants and appear on
summary/category cards. Legacy achievements remain undated; date metadata never
grants points or unlocks, and repeated grants preserve the original date. Native
wide/compact restart evidence and save compatibility/calendar tests verify this.
The 58 catalog entries now use authored requirement-themed vanilla icons; related
zone/class tiers share icons. The explicit-data `--achievement-art` audit checks
all referenced files, and native summary/category captures document the artwork.
Jeweler’s Eye now measures half the complete jewelry Codex (currently 56/112),
combining deduplicated shared discoveries with valid class legendary discoveries.
Legacy awards remain earned; the separate 36-legendary rule remains under the
clearer Legendary discoveries name. Core/account tests and native before/after
fixtures verify thresholds, persistence, duplicate protection and a single grant.
Legendary now awards 25 points for all nine legendary gear pieces of one class.
The best single-class collection drives progress; duplicate drops, lower rarities
and cross-class pieces cannot fill it. Saved duplicate achievement IDs cannot inflate
points/counts. Core/account tests and native 8/9-to-earned screenshots verify this.
Reference catalog expansion and broader per-page interaction acceptance remain open.

Options now opens shared pause settings from the starter and all Armory pages.
The menu leaves layout while settings is visible; its mouse/keyboard actions are
blocked. Back/Escape restores the existing page/draft; Enter cannot start a run.
Paused Escape closes settings before a subsequent Escape resumes. A bounded,
opaque bordered panel improves readability and supports wide/compact rendering.
Native 1280×720 / 640×720 checks verified MasterVolume 100→90%, auto-push On→Off,
both values after restart, Enter isolation and return to Talents. The two ECS
regressions cover modal key consumption and menu layout/input removal. All 226
Survivors tests, native build, workspace Clippy, formatting and render lint pass.
Inline screenshots are in CHANGELOG/README.

The category checkpoint adds three inline screenshots. Native wide/compact
checks cover switching all three panels, MasterVolume 90→80%, auto-push Off→On,
saved values, Enter isolation, Done and the About→build-notes route. All 33 focused
HUD tests and workspace Clippy pass; the new regression guards the notes action
against leaving a paused run. An actual restart and audible output were not retested.

Audio/Graphics now each restore their own registered defaults; Graphics also resets
camera preferences to the built-in defaults rather than reloading saved values.
The title uses the installed Morpheus font and Defaults/Done align at lower right.
All 228 Survivors tests pass, including category isolation and normal CVar change
callbacks. Native 1280×720 / 640×720 checks verify both resets, unrelated-category
preservation, About hiding defaults, and saved values after relaunch. Build,
workspace Clippy, formatting and render lint pass; two inline screenshots record
this checkpoint. Audible output and in-world camera effects were not retested.

Numeric Audio/Graphics controls now use proportional gold sliders with separate
labels and readouts. Audio snaps to 1%; graphics retain all supported stops. Ordered
window events preserve short drags, clamp past the track and stop on release,
window focus loss or leaving Options. Tests cover short drags at 1×/2× display
scale and every supported graphics stop; all 230 Survivors tests pass. Native
1280×720 / 640×720 checks verify click/drag, maximum clamping, 37% audio, all four
graphics sliders, defaults/fills, Enter isolation and volume after relaunch.
Build, workspace Clippy, formatting and render lint pass; screenshots are inline
in CHANGELOG/README. Audible output was not tested.

Compared against `survivors_paused_options_gamepad.png`: Audio, Graphics and About
now use left-side category navigation and a focused bordered content panel.
Options now supports Tab/Shift-Tab and Up/Down focus, Left/Right slider steps,
Home/End limits and Enter/Space activation. A white outline marks the focused
control; hidden categories are skipped. Mac raw modifier events preserve Shift-Tab.
Done runs after the HUD shortcut handler, preventing the same Enter from starting
or resuming a run. ECS tests cover both phases; native wide/compact checks cover
focus, slider steps/limits, category activation, Done, pointer handoff and build
notes. Screenshots are embedded in CHANGELOG/README.
Audio and camera switches now use reference-style labeled gold checkboxes,
aligned with slider controls. The entire row accepts clicks and keyboard focus;
checkmarks are drawn from two strokes rather than relying on a font glyph.
Native wide/compact checks cover label/box clicks, Space activation, checkbox
states, Graphics defaults and persisted music state after relaunch. All 38 focused
HUD tests, native build, workspace Clippy, formatting and render lint pass. New
screenshots are inline in CHANGELOG/README; audible output and in-world camera
effects were not retested.
Controls now exposes saved mouse X/Y inversion with category defaults and current
movement/camera guidance. The camera applies inversion to orbit drag, and camera
rotation/zoom/mode input is limited to Playing. Old saves default both axes off;
Graphics resets preserve inversion, and Controls resets preserve collision/cutaway.
233 Survivors tests cover direction mapping and persistence compatibility; native
wide/compact checks cover mouse/keyboard toggles, both reset boundaries and saved
inversion after relaunch. Build, workspace Clippy, formatting and render lint pass;
screenshots are inline in CHANGELOG/README. In-world inversion and paused camera
isolation still need live acceptance checks. The shared runtime camera-input path
now has regression coverage for all non-playing phases, loading-screen coverage,
blocked wheel-event consumption, independent inversion, fixed-view restoration,
line/pixel zoom equivalence and pitch/zoom limits. Loading coverage also suppresses
camera gestures while the world is not ready. All 236 Survivors tests, native
build, workspace Clippy, formatting and render lint pass. Controls now includes independent saved camera/zoom sensitivity (25–300%,
100% default), with matching pointer and keyboard steps. Controls reset restores
both speeds and inversion while preserving Graphics; Graphics reset preserves
Controls. Old or invalid saves retain safe speed defaults. All 238 Survivors tests
pass, including rotation/zoom scaling. Native wide/compact checks cover clicks,
dragging, keyboard limits/steps, reset isolation and persistence after relaunch;
new screenshots are embedded in CHANGELOG/README. In-world feel, gamepad navigation,
bindings and other settings remain required.
Controller discovery is enabled for Survivors (the stock client keeps it disabled).
Start opens Options from Menu/Paused; D-pad up/down focuses, left/right adjusts,
south selects and east closes. This shares keyboard focus and setting actions,
and only one controller supplies new presses per frame. Synthetic ECS checks cover
both menu states, toggles, slider changes, back, held input, gameplay rejection and
card-preview rejection. Native compact validation covers backend startup and help
layout, with a keyboard-focused screenshot in CHANGELOG/README. Physical controller
acceptance, other starter pages and gameplay remain unfinished.
The starter/Armory primary actions now share directional controller/keyboard focus:
D-pad/arrows navigate by screen position, south/Enter/Space activates the existing
Act path, and controller east returns to the starter. First controller selection
establishes focus; held input cannot repeat activation. Focus clears for Options,
text entry, loading, pointer presses and non-menu phases. Scrolled controls reveal
through the actual vertical scroll container, skipping non-scrolling wrappers and
leaving a margin for the outline. Focused Enter is consumed before the global run
shortcut. Native compact evidence covers Options/Achievements, Space category
selection, and equipment/lower-action scroll reveal; two screenshots are embedded
in CHANGELOG/README. Controller ECS tests cover the shared path and nested-scroll
regression. Physical devices, complete per-page traversal, other hover details,
non-talent secondary actions, name entry without a keyboard and controller gameplay
remain open.
Talent and milestone descriptions now follow keyboard/controller focus before
mouse hover, using the same title, rank preview and supported-effect tint. Pure
milestone rows join focus navigation without becoming purchase actions. Inspection
never activates a talent or spends a point; the page help describes arrows/D-pad.
Other hover surfaces and physical-device acceptance remain open.
Talent refunds now accept Backspace/controller west on the focused talent through
the same row/prerequisite checks as right-click. Requests last one frame, clear
under modals/text entry and cannot trigger secondary equipment actions. Native
Backspace validation refunded rank 4→3, returned one point and verified the saved
spec. Talent and milestone focus now survives tree entity replacement, including
the intermediate layout frame, using rank-independent identity scoped to the hero.
Restoration does not replay an action and clears on page/hero changes or modal
and pointer handoff. Rewards now also retains focus by action, hero and category
through rebuild/layout delay. Repeated +/− actions preserve their outline; missing
or disabled controls drop focus, and category/page changes, modals and pointer input
clear the identity. Native 1280×720 checks verify two upgrades, two individual undos,
correct repricing, disabled-control inactivity and an unchanged account file.
Physical controller acceptance and focus retention on other pages remain open.
This exposes existing controls; it does not complete the reference Options page
or establish audible output/engine setting effectiveness. About exposes the existing
notes/restart page from the starter; it is unavailable during a paused run.
Downloaded self-updates remain future work. Run Options still only reports Normal difficulty.


Starter actions now use the shared gold-outline/burgundy control style. Armory,
Talents and entry actions contain both title and subtitle within the button.
Enter uses proportional width with bounded wrapping rather than a fixed texture.
Options/Quit share the new style on all Armory pages; Random/Look use text-sized
controls. Native 1280×720 / 640×720 checks verified full-card Armory/Talents
navigation, long Gates of Ahn'Qiraj text, lock preservation, Options navigation and
Quit. Build, workspace Clippy, formatting and render lint pass. Screenshots are
embedded in CHANGELOG/README. Options now opens shared preferences;
Run Options still only reports Normal difficulty.
The starter now has an original silhouette backdrop (see Stable starter backdrop below).
Outdoor zone illustrations remain open.

Starter portraits now include the reference's faction pill and circular class
badge, derived from the selected race/class and runtime class atlas. Badge size
and corner offsets follow the portrait. Native Horde Warrior → Alliance Mage
switching, drag rotation, 1280×720 / 640×720 layout and scroll clipping passed;
three screenshots accompany the changelog/README. Native build, workspace Clippy,
formatting and render lint pass. Starter control styling, background and outdoor
zone illustrations remain open.

Collection equip/unequip actions now preview the complete equipment contribution,
including worn-set gains/losses. The shared bonus formatter uses green/red numeric
comparisons with one unit. 110 core and 194 app tests pass; native wide/compact
checks verified preview totals against actual removal/restoration. Screenshots
are inline in the changelog. All-class and all-item visual acceptance remains open.

Collection detail now groups icon/name/quality/ownership in the bordered identity
card from the Collection intention. A collected Warrior fixture verifies selection
changes and long-name wrapping at 1280×720 and 640×720, with inline changelog
captures. This does not complete collection-page styling or the item catalog:
Poor gear now has catalog, save, UI and ordinary-enemy drop integration; distinct
raid-epic identities, acquisition and set builds remain open.

Rogue has sixteen explicit individual talents: Improved Slice and Dice, Elusiveness,
Vigor, Remorseless Attacks, Lightning Reflexes, Deflection,
Camouflage, Sleight of Hand, Improved Eviscerate, Malice,
Improved Sinister Strike, Serrated Blades, Deadliness, Improved Gouge, Lethality
and Aggression. Their effects replace provisional rank damage; exact rules are
in `SURVIVORS.md`. All 105 core tests pass. Account coverage verifies save/reload,
hero/class isolation, spec switching and reset persistence. Native 1280×720
checks cover descriptions, purchase/refund and prerequisite/row gates, with
inline changelog screenshots compared against the talent-page intention.
Two combat-parameter/hit regressions also cover ability ranks 1/3/8, equipment
composition, critical rolls, Rupture ticks and matching Gouge stun/marker durations.
Assassination and Combat now also implement all six 10/20/30-point milestones:
critical chance/damage and guaranteed Eviscerate criticals; cooldown reductions
and a wider single Blade Flurry area. 106 core and 191 Survivors app tests pass.
Two live captures show independent active 30-point specs and exact descriptions.
Subtlety completes the nine Rogue milestones with a Gouge absorb, Kidney Shot
advancing Gouge, and Gouge advancing Eviscerate. Shared cooldown-advance handling
preserves Paladin behavior. All 107 core and 192 app tests pass. Subtlety descriptions
were checked in the compact layout, with wide/compact inline screenshots.
The four defense/mobility nodes replace provisional damage with reduction, armor
or movement speed. 108 core tests and the local talent audit pass; native
Camouflage/Deflection refund/rebuy checks and two screenshots cover 1280×720.
Remorseless Attacks adds a non-stacking, 20-second kill-triggered bonus to the
next successful eligible melee cast. 109 core and 193 app tests pass, including
rank limits, expiry, refresh and consumption. Wide/compact captures document
the description; native refund/rebuy was verified. An additional ECS regression
exercises all four eligible auto-casts with two targets: failed target selection
preserves the proc, the successful cast shares it, and the next cast loses it.
An active-proc HUD indicator
and live combat acceptance remain open, alongside other Rogue nodes and
remaining compact descriptions.

The user authorized lore-appropriate racials for all races. All eight now have
explicit rules in `SURVIVORS.md` and shared core effects applied to the run sheet;
Human rules remain intact. Timed effects use gameplay time. Two core tests and
four run-stat tests pass, and wide/compact Orc starter screenshots verify the
visible descriptions. Extended combat and balance acceptance remains open.

Companion artwork now matches the reference's Quick Paws, bird, Long Leash,
frog, serpent, Zergling and cockroach icons. Runtime paths were verified against
the checkout patch chain and native wide/compact captures are inline in the
changelog. In-world pet animation/scale and broader Rewards layout acceptance
remain open; icon matching does not establish those checks.

Companion upgrade rows now keep equal widths, including Second Companion.
Looks and Tabards use adaptive grid columns; Looks fits four columns at 1280×720
and three at 640×720 without stretching its last row. Live layout/selection checks
and three inline screenshots cover this checkpoint. Companion artwork and other
page-specific details remain open; this is not full Rewards visual acceptance.

Companion upgrade cards now follow the compact Perk structure: inline rank pips,
right-side cost/+⁄− controls and purchased-state tint. Preview, disabled purchases,
pending removal/point restoration, compact text fit and appearance switching
passed live checks; three new inline screenshots preserve the earlier captures.
Artwork and other page-specific details still need reference alignment.
No companion cost or effect changed.

Rewards currency/category rail now emphasizes available points and uses full-width
category buttons. Confirm/Undo remain visible but disabled without a draft.
Preview, category switching with pending changes, Undo and 1280/640×720 fit
passed live checks; three screenshots are inline in the changelog. Companion/
Tabard cards and other page-specific reference details remain unfinished.

Major Armory sections now have centered headings and flexible, fading gold
rules. Wide Collection/Achievements/Rewards and compact Talent Points were
verified live with four inline screenshots. This completes the shared section
framing pass; remaining texture buttons, portrait
badges and page-specific details still need reference alignment.

Reward perks now use compact reference-style cards with name-adjacent rank
markers and cost/+/− controls at right. Disabled max-rank/unaffordable purchases,
pending-rank removal and point restoration passed live checks at 1280×720;
640×720 wrapping was captured after scrolling. Three screenshots are inline in
the changelog. Companion/Tabard styling remains open; no costs or perk effects
changed.

Navigation and shared Armory controls now use gold outlines, burgundy selection
and hover/pressed feedback. Text-sized tabs wrap; Done only appears inside the
Armory. Equipment actions, reward preview/Undo, navigation and 1280/640×720 fit
were checked live. Texture-based starter and Options/Quit controls still need
alignment, along with the other remaining items below.

Starter title, section headings and zone names now use runtime-loaded Morpheus;
body copy keeps the HUD face. Wide/compact rendering, outline alignment, long
zone names and zone-selection rebuilds passed live checks with three screenshots.
Armory primary section headings now share that face across all seven pages;
1280×720 page switching and compact/scrolled Talents passed live checks, with
eight inline changelog screenshots and refreshed README gallery images. Buttons,
decorative dividers, portrait badges, outdoor zone art and broader styling still
need reference alignment. This does not establish full interface acceptance.

Starter class selection now follows the reference's three-by-three wide grid,
using proportional icon sizing, centered labels and class-colored selection.
Wide 1280/1100 and compact 640×720 layouts, plus class switching, were verified
live with three inline changelog screenshots. Equipment now uses shared grid
columns, fixing the stretched final slot at 1100 pixels. Worn/empty states,
1280/1100/640 layouts and wrapped main-hand inspection were verified live.
Typography, portrait badges and broader starter styling still need reference alignment.

Starter manual naming now uses a hero-bound draft with Save/Cancel, Enter/Escape
and Tab. Live checks cover blank rejection, save/cancel, Enter isolation,
640×720 layout and restart persistence; screenshots are inline in the changelog.
The shared name handler now uses ordered keyboard events: rapid repeated letters,
Backspace, Tab and Enter passed both regression and live starter/creation checks.
OS text supplies capitalization; the existing ASCII name filter and 16-character
limit remain. Reference typography and control styling remain open.

Talent headings now resolve specialization artwork through the installed
TalentTab → SpellIcon catalogs, with points aligned opposite the name. Missing
art leaves a text heading. The Survivors adapter is separate from the stock
talent catalog. Mage and Paladin were checked live at 1280×720, plus Paladin at
640×720; its custom milestone text-only fallback also passed. Three screenshots
are inline in the changelog. Typography and custom Paladin milestone artwork remain open.

Talent milestone icons now resolve by the milestone name within its installed
talent tab, then use that talent's spell icon. Locked icons dim alongside their
text; earned icons brighten. Custom milestones absent from the vanilla tree
retain text-only fallback. Custom Paladin milestone artwork and typography remain open reference-alignment work.

Named prerequisite checkpoint: Shatter/Improved Frost Nova verified live with
missing, partial and complete prerequisite ranks, protected refunds and restart
persistence. Blocked purchase/refund feedback now names both talents and the
required rank. Three screenshots accompany the changelog. Higher-tier and
cross-class acceptance remain open; custom milestone icons and typography still need alignment.

Mage Frost row gating now has live 1280×720 evidence: blocked supporting refund,
dependent-first refund returning two points, and rejected spending after the row
relocks. Three screenshots are inline in the changelog. Named prerequisite chains
and higher-tier builds remain open; this does not complete the talent audit.

Starter selected-zone details now expand inside the selected row, matching the
reference's placement. Switching Elwynn/Warsong collapses the old row; available
and locked states, wrapping and crop updates were verified at 1280×720/640×720.
Existing catalog descriptions remain separate from verified encounter completeness.

Starter zone artwork checkpoint verified at 1280×720 and 640×720: runtime art,
responsive cropping and locked banner clicks passed for Warsong/Naxxramas.
Three captures are inline in the changelog. Use installed Map.dbc → LoadingScreens.dbc artwork
for instance/battleground rows, cropped to the current row aspect ratio. Keep
names and lock reasons visible and retain text-only fallback. Continent loading
screens are shared and do not qualify as distinct outdoor zone illustrations;
Elwynn, Duskwood, Un'Goro, Feralas, Gurubashi, Gates of Ahn'Qiraj and the undefined
Blackrock Mountain map still need appropriate zone-specific artwork. Never bundle
extracted client assets. Loading artwork is a visual aid, not evidence of world
content readiness.

Human racial checkpoint: the reference's Perception (+15% damage for 10 seconds
every 30), Human Spirit (+10% run XP) and Diplomacy (+20% gold) now reach the run
sheet and appear on the starter. First Perception activation is at 30 seconds.
Core boundary tests and app tests cover stat stacking, legendary damage and
hero/companion pickups. Human/Gnome switching and 1280×720/640×720 descriptions
were checked live. Other racial rules, Perception combat feedback and full live
combat acceptance remain open. Existing per-award gold rounding is retained.
The starter Look button now opens Character's body controls. Look → Character
→ Escape was verified live at 1280×720 with the selected Mage retained. The
older creation workflow remains separate.

Starter portrait checkpoint verified at 1280×720 and 640×720: the selected
hero's saved body/appearance renders beside their name and race in a proportional
pane. Live Warrior/Mage switching, Character round trip, scroll clipping and
drag rotation passed. The shared renderer keeps rigid equipment framed. Four
actual captures are inline in the changelog. Faction/class portrait badges and
inline name editing remain differences from the reference.

Starter equipment/responsive checkpoint verified at 1280×720 and 640×720:
worn-item icons use verified equipment display identities with quality borders;
empty slots remain labeled. Slot selection opens the worn piece in Collection.
Below 1100 UI units, Class/Hero/Zone stack in a shared scroll surface with an
independent session offset; entry controls stay outside the scroll. Live shoulders
inspection, lower-zone reachability and Collection/Escape scroll restoration passed.
Still open against the main-menu intention: illustrated zone rows,
decorative typography, remaining racial bonuses and wider viewport acceptance.

Talent saved-build live checkpoint verified at 1280×720 with an isolated Warrior
and Mage: distinct Warrior specs (Arms 3/Fury 2), Mage spec 3 (Frost 1), selected-spec
restoration across hero switches and restart, and resetting only the active Warrior
spec. Save-file readback preserves Warrior spec 1 and Mage spec 3 after the reset.
Screenshots are inline in the changelog. Still open: all-class combat acceptance,
deep prerequisite/refund interaction checks and narrow per-spec scroll live validation.
Stale action feedback is now cleared by page navigation; Talent → Character and
Talent → Escape were checked live at 1280×720. Starter finale copy now shares the
combat schedule lookup with Drop Tables and displays names plus time. Elwynn's
Fedfennel at 16:00 and locked Duskwood's progression requirement were checked live;
a regression covers finale identity/time for every map. Zone readiness remains separate.

Character responsive checkpoint verified at 1280×720, 640×720 and 640×900:
below 1100 UI units, stack a proportional square hero preview, appearance/features,
equipment and item details in one scroll surface. Character, Collection and Drop
Tables retain separate offsets. Wide screens retain the intention's
appearance/model/features composition. Live body/feature edits, Warrior/Mage
switching, tab return and restart persistence passed with an isolated account.
Saved appearances no longer trigger the obsolete “Open Look” prompt. Actual captures
are inline in the changelog. Final typography, wide-column scrolling remain separate acceptance; the Talent checkpoint above
records the verified two-hero/spec persistence scope.

Drop Tables responsive checkpoint verified at 1280×720 and 640×720: below 1100 UI units, wrap zone
choices and stack sources, item details and equipment in one scroll surface.
Retain an independent session scroll offset on selection and tab return. Wide
encounter rows place source/time beside item/icon, matching the reference; zone
choices show quality and collection count. Browsing must never select the next
run zone. Live lower-row selection, equip and Collection round trip retain scrolling.
Finale labels derive from the combat schedule, not the legacy zone boss name.
Source identities, missing zone content and wide-column scroll restoration remain
separate acceptance. Actual screenshots are inline in the changelog.

Gear stat checkpoint verified at 1280×720: [slot profiles](SURVIVORS-GEAR-STATS.md) replace
the shared non-glove bonus sequence. Each current band adds one stat and strengthens
retained stats; only selected equipment contributes, regardless of duplicate count.
This is initial tuning. Poor gear now has authored identities and starter-zone
sources; distinct raid epics, build-specific raid sets and combat balance remain open.

Shared Armory header checkpoint verified: title, wrapping tabs and selected
class/collection subtitle follow the references; the starter keeps its game title.
Collection's duplicate subtitle is removed. The 640-pixel check exposes crowded Collection
slot labels. Narrow Collection reflow is verified at 640×720 and 640×900: below 1100 UI units,
stack the collection, selected-item details, equipment and missing drops in one
scroll surface. Wrap labeled tiles at a readable minimum width; preserve scrolling
on item actions and tab return. A 1280×720 restart confirms the reference's three
panes and saved equipment/appearance choices. Typography and item-stat/content
acceptance remain open.

Codex presentation checkpoint verified at 1280×720 and 640×720: legendary amulets use a verified necklace
silhouette plus an associated spell badge, shared jewelry aligns with the class rows,
and selection highlights the catalog cell and shows a larger detail image. The generic
legendary silhouette is representative art, not a claim that generated amulets have
vanilla item identities. Distinct legendary artwork and complete catalog expansion
remain required. Live badge clicks, unseen/shared entries and lower-row selection
retain the catalog scroll position; actual captures are inline in the changelog.
Mark of the Champion art is resolved by the explicit variant choice recorded below.

Tabard presentation checkpoint: replaced the plain name buttons and
client display-ID text with illustrated cosmetic tiles using the Companion Looks
reference. Keep the verified one-tabard catalog, immediate account persistence,
and a clear None/Remove choice. Live wide/narrow selection, unchanged points and
restart persistence are captured in the changelog. Additional tabards require
verified client mappings; character-model previews remain open.

Achievement presentation checkpoint: replaced plain rows with the intention's
points badge, category rail, recently earned cards, five category progress bars,
and counted achievement cards. Descriptions derive from the same catalog variants
that evaluate completion. No earned dates are stored, so cards use “Earned”. Category
icons are verified local substitutes; boss-specific art and decorative crest work
remain open. Responsive layout uses wrapping proportional columns and scrolling,
with separate session offsets for Summary and each category. Live 1280×720 and
640×720 captures, lower-zone reachability and navigation scroll restoration are
recorded inline in the changelog.


Rewards confirmation checkpoint: Perk/Companion purchases and perk resets are held
in a session draft. Confirm validates the entire batch against the latest account
before applying it; Undo discards the batch. Navigation retains pending choices,
run entry requires Confirm/Undo, and closing the app discards unconfirmed choices.
Cosmetic selections remain immediate. Per-card minus controls remove the latest
pending purchase of that type and replay the remaining draft. Removal is atomic:
if it invalidates a later purchase or changes an instinct destination, the player
must undo that dependent purchase first. Confirmed purchases still use Reset perks
for refunds; a card minus only edits the current draft. Live wide/narrow removal,
rank repricing, confirm, reset undo and Companion undo are captured in the changelog.
Three regression tests cover repricing and dependency/destination preservation.

User clarification implemented: Deathless Hour and no u are visible placeholder
achievements with no satisfiable criterion or points. Panda Cub and Sapphiron remain
locked behind those entries until future criteria are implemented; rules are not
inferred from their names.


Rewards scroll checkpoint: session offsets are tracked independently for the three
categories and attached to the rendered category. At 640×720 a live Perk purchase
and Collection round trip preserve the scrolled view; the changelog contains before,
after and return captures. ECS coverage checks category changes and rejects invalid
offsets. Live Companion/Tabard category scroll coverage remains open.


Perk preview checkpoint: all six Perks render green inline current→next effects with
units once, using core mechanics for XP, hand size and drop chance. Capped perks
show their current effect only. Live Veteran's Lessons purchases verified both ranks
and max state; 640×720 wrapping is captured in the changelog. Core tests cover unit
formatting and all caps. Companion previews, staged purchasing and Rewards scroll
continuity remain open; reset-count wording is now corrected.


Rewards visual checkpoint: currency/reset/category rail and illustrated two-column
Perk cards replace the draft rows. Existing account purchase actions are reused.
Live isolated-account validation confirms a 15-point purchase, rank pip/next cost,
refund and reset consumption, plus 640×720 stacked layout. See the changelog for
inline captures. Still required: green current→next effect previews, Confirm/Undo
staging, Companion/Tabard illustrated content, scroll preservation across purchase
rebuilds, reset singular wording, and typography matching the intention.


Talent presentation checkpoint: panels use the available page height, with a readable
minimum and the existing wrapping/outer scroll. Live Warrior captures at 1280×720
and 640×720 verify taller trees, spending and lower-panel reachability. Singular
“1 point” is corrected. Reference icons, typography and artwork black margins remain
open; see the latest changelog for inline evidence and validation scope.


Live Talent validation (2026-10-25): isolated Warrior fixture confirms 800×720
three-tree-plus-wrapped-detail layout, wheel reachability, spending/reset totals,
and scroll restoration after reset and a Collection round trip. In-process preview
resize to 640×720 confirms two-column layout and lower-tree left-click spend /
right-click refund without losing scroll. See inline evidence in the changelog.
Remaining: live per-spec/per-hero restoration, other classes, reference header and
milestone icons, full-height illustrated tree art, typography, and singular “point”.


Companion purchase clarity: preserve the reference's first-instinct-to-first,
next-to-second assignment. The existing replacement behavior targets the first
companion once owned slots are filled; show that destination, the replaced instinct
and the 30-point cost before purchase. Appearance slot selection is independent of
instinct assignment and must not imply otherwise. Display each companion's current
instinct and the three concrete effects. Reject unknown instincts without charging,
and reject appearance changes to an unowned second companion. Perk resets continue
to leave companion purchases intact. The full illustrated Rewards layout, pips,
icons and live purchase/screenshot checks remain acceptance work.
Validation: all 171 Survivors tests pass, including assignment, duplicate rejection,
exact charges, insufficient funds, save/reload and perk-refund isolation. Workspace
Clippy, formatting, render-pass lint and the native build (28 seconds) pass. The
rendered page and end-to-end purchase interactions have not been visually verified.

Talent scroll continuity checkpoint: a session-local cache remembers each rendered
hero/spec's scroll position before menu actions rebuild the tree. Returning to a
tab or spec restores its own position; a different hero starts independently.
The shared scroll clamp waits for a nonzero computed layout before constraining
a restored offset to actual content bounds. Two ECS regression tests cover old-tree
identity during spec switching and restored-offset clamping after layout. All 170
Survivors tests, workspace Clippy, formatting, render-pass lint and the native build
(2m05s) pass. Live wheel,
spend/refund, tab/spec-switch and resize checks still require visual validation.

Talent responsiveness checkpoint: tree and spec/detail panels use proportional
growth with a 240-unit preferred basis and 200-unit minimum width, wrapping into
additional rows as available width falls. The page owns vertical scrolling instead
of each nested panel; talent detail text already persists after pointer exit, so
the last inspected node can be read after scrolling to a wrapped detail panel.
Workspace Clippy, formatting, render-pass lint and the native build (21 seconds)
pass. Rendered 16:9/4:3/narrow comparisons, icon/label fit, wheel behavior and
scroll-position behavior after spending/refunding or switching specs remain to be
verified. No new screenshot or responsive visual acceptance is claimed.

Preview fixture checkpoint: `scripts/survivors-ui-fixture.py` now copies checked-in
synthetic fresh/partial/collected saves into new isolated state directories and
prints a direct-page launch command. Existing directories and symlink destinations
are refused. The real account parser and save round trip validate the fixtures;
168 Survivors tests, two script isolation tests, workspace Clippy, formatting and
render-pass lint pass. The collected fixture contains all 54 ownership cells per
class and ten heroes, not verified item/zone completion. All fixtures intentionally
omit zone clears and spent talent builds. Rendered fixture/resolution comparisons,
populated spec fixtures and purchased reward states remain open. See the
[fixture workflow](SURVIVORS-DEVELOPMENT.md#deterministic-account-fixtures).

Prioritize the starter/menu and Armory interface pages before further zone content
or in-map expansion. Build a fast, repeatable UI preview loop: direct page/fixture
selection, isolated deterministic account data, immediate viewport changes and
screenshots without playing through a session. Reuse compiled engine dependencies;
use hot-reloaded layout/style/content where practical. Preview the production UI
components so a separate mockup cannot drift from the game. Existing held-card
and F12 tools help but do not complete this requirement.

The reported cutaway defect is an immediate correction: activate only when scenery
blocks the actual camera-to-player sightline; buildings behind or beside a visible
player must remain opaque. Fade reveal strength linearly over 500 ms of real time,
and fade back when visibility returns. Preserve collision throughout.

Later feature: unlockable fast mode with 1.0× normal, 1.1×, 1.25×, 1.5× and 2× game
speed. Unlock criteria remain to be designed. Scale simulation consistently
(movement, attacks, cooldowns, waves and encounter clocks), leaving UI/input and
visibility fades on real time. Do not enable this feature during the starter-UI
pass; record its unlock/persistence, timing and reward/balance acceptance later.

## Current PR boundary and post-merge physics (2026-10-25)

Fast-preview checkpoint: native dev builds accept `WOW_SURVIVORS_UI_PREVIEW`, an
opt-in local file with page/width/height. Complete valid edits switch production
pages and logical window size during the menu without restarting; incomplete or
invalid edits retain the view, and active runs defer updates. Account state and
character-name drafts remain intact. Two new tests cover validation and applying
the request; all 165 Survivors tests in the app-only feature build pass, as do
workspace Clippy, formatting, render-pass lint and the native play build (1m48s).
The app-only test command caused a different dependency feature build (7m59s);
prefer the established workspace test build plus filtered executable for subsequent
checks to avoid switching cache variants. Startup logs show the preview request
was applied. Live rendered page-switch/resize verification and screenshots remain
pending: the Mac was locked, so the validation app was stopped without UI inspection.
Deterministic account fixtures and hot reload across the remaining pages are still open.

Finish the interface and initial account/setup experience in this PR before starting
Box3D work. The acceptance scope includes character creation/selection, talents and
saved specs, gear/appearance and rarity rules, progression/unlocks, achievements and
rewards, and the documented starter/Armory expectations. These must have working
state transitions and persistence, clear locked/empty/owned states, responsive
layouts, inline green upgrade values with units shown once, and milestone screenshots
in README. A displayed page or placeholder is not acceptance. Preserve the detailed
phase criteria below; this priority change does not silently mark outstanding rules
or functionality complete. Finish the fast production-UI preview workflow and verify
representative fresh and progressed accounts across supported screen sizes.

Further zone expansion follows the interface pass. Explicitly later features, including
unlockable fast mode, remain later work. Review the PR's remaining acceptance gaps,
complete required checks, and merge the interface/setup PR before beginning a separate
Box3D integration branch/PR. Do not add Box3D dependencies or migrate physics in this PR.

Post-merge physics direction: evaluate [Box3D](https://github.com/erincatto/box3d)
against the existing Avian 0.6 integration. This is a backend migration/extension,
not the first physics system: movement, floor support, scene queries, camera collision
and enemy detours already depend on collision queries. Box3D's upstream library is
C17; plan a pinned native build, Rust bindings and a safe ownership layer, followed
by a repository-owned Bevy plugin. Do not assume a fork of Bevy itself is necessary.
Audit collider import, terrain/WMO/M2 geometry, collision layers, ray/shape/overlap
queries, character movement, fixed stepping, events and teardown before replacement.
Prove native platform builds and the existing wasm target's linking/thread/SIMD
constraints independently; upstream's Emscripten example does not prove compatibility
with this workspace's wasm32-unknown-unknown build. Benchmark against Avian and retain
regression coverage for floors, walls, foliage, camera occlusion and crowded movement.

Survivors currently has no authoritative gameplay server. The realm/login/relay
service is not a Survivors physics host. A future multiplayer design must separately
define authority, shared collision assets, fixed ticks, input/snapshot protocols,
prediction and reconciliation before deciding which simulation runs on the server.
Box3D alone supplies neither networking nor navigation; the current local A* planner
would consume the chosen backend's queries. Server integration is a future architecture
decision, not an automatic dependency of the interface PR or a reason to modify the
realm service now.

## Architecture and ownership

The game is local simulation in Rust, on Bevy 0.18.1 and wgpu 27. The
`warcraft-survivors` launcher calls into `benilla-app`; Survivors owns its mode under
`crates/benilla-app/src/survivors/`. Native Bevy UI and the existing glue widgets draw
menus. MPQ archives, DBC catalogs, item displays, models, icons and terrain come from the
player's read-only 1.12.1 install. There is no gameplay server for this mode.
Do not replace the menu with a website or bundle reference art as runtime assets.

Keep changes in Survivors-owned files. Reuse `benilla-world`, the model attachment
systems, glue art, tooltip primitives, and catalog loaders through their existing
interfaces. Any necessary upstream carry must obey AGENTS.md. Native is the daily loop;
wasm uses the same rules with the existing web entry and platform persistence seams.

| Responsibility | Implementation seam |
| --- | --- |
| Mode state, run resources, stat assembly | `survivors/mod.rs` |
| Class abilities, map/creature catalogs | `survivors-core/src/data.rs` |
| Campaign gates, pieces, drop rows, talent rules | `survivors-core/src/campaign.rs` |
| Named heroes, class progress, saves, settlement | `survivors/account.rs` |
| Armory actions and pages | `survivors/armory.rs`, `survivors/armory/door.rs` |
| Glue appearance and selected hero | `survivors/menu.rs`, `survivors/hero.rs` |
| Enemy timing, movement, pickups | `survivors/enemies.rs` |
| Combat and death awards | `survivors/abilities.rs` |
| Drafts, temporary jewelry, pause/results | `survivors/cards.rs`, `survivors/hud.rs` |
| Achievement/perk catalog and companions | `survivors-core/src/rewards.rs`, `survivors/companions.rs` |
| Formula boundaries | `survivors-core/src/sheet.rs` |

## Existing foundations and remaining gaps

The three-column Armory door, twelve-zone registry, locked-zone explanations, a 54-cell
collection save, three saved specs, real Talent.dbc presentation, achievement categories,
perks, companion controls, tabard selection, jewelry catalogs, and combat already exist.
They must be extended rather than replaced with independent copies.

Current source audit (2026-10-26): all six existing bands have named gear entries.
Selected worn equipment and independent cosmetic choices persist; only worn pieces
supply bonuses (`account.rs`, `survivors-core/gear_loadout.rs`). Collection and drop
pages use icons and detail panes. Poor gear has 81 class/slot identities, a grid
row, stable-key saves and ordinary-enemy starter-zone acquisition. A distinct
raid-Epic catalog, acquisition and build sets remain open.

Settlement credits the selected valid hero and guards repeated awards; regression
coverage includes save/reload and invalid references. This supersedes the original
first-hero settlement defect. Zone/content and full run acceptance remain open.
Individual talents remain substantially incomplete; the coverage table below is
catalog evidence, not a substitute for combat or visual acceptance.

### Talent definition coverage — 2026-10-26

Read from this checkout's `wow-data/Data` using:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-coverage
cargo run -p survivors-audit -- wow-data/Data --talent-gaps
```

| Class | Matching individual definitions / selectable nodes | Milestone descriptions / 9 |
| --- | ---: | ---: |
| Warrior | 34 / 52 | 9 / 9 |
| Paladin | 21 / 44 | 9 / 9 |
| Hunter | 18 / 46 | 6 / 9 |
| Rogue | 16 / 51 | 9 / 9 |
| Priest | 21 / 47 | 9 / 9 |
| Shaman | 14 / 46 | 9 / 9 |
| Mage | 30 / 49 | 9 / 9 |
| Warlock | 16 / 50 | 9 / 9 |
| Druid | 20 / 47 | 9 / 9 |
| Total | 190 / 432 | 78 / 81 |

The command checks definition names/rank caps against actual selectable DBC nodes,
rejects unknown authored node IDs or mismatched tree ordering, and lists missing
nodes with `--talent-gaps`. Missing effects are reported without failing identity
validation. Milestone counts measure available descriptions, not runtime proof;
each milestone still needs the acceptance evidence described in Phase 5. Imported
saves and generic fallback bonuses do not make missing individual effects complete.
The next class-content work must address these gaps, including combat-pet/form
mechanisms where needed, rather than treating milestone coverage as class completion.

## Rule decisions

Preserve waves, horde counts, level-up drafts, chest trades, renown and relics until an
explicit replacement exists. Keep permanent class collection separate from run jewelry.
A class has nine equipment slots and six collection bands: Common, Uncommon, Rare, Epic,
PvP, Legendary. Two-handed weapons consume only main hand. Only worn gear affects stats;
cosmetic overrides and tabards never do. A full worn nine-slot set earns its bonus.

One hundred talent experience earns one point, capped at 51. Each hero stores three
independent specs. Tree rows require earlier-row investment; refunds must not strand
higher rows or prerequisite talents. Milestones at 10/20/30 must change actual abilities.

The user resolved the reference conflicts: duplicates add no power, and +target affects
single-target attacks only, never extra AoE locations. Quality strengthens existing stat
lines and adds a new stat line at each step. Raid epics need a tier beyond ordinary epics,
an additional stat, and build-oriented set bonuses. The glove numbers given by the user
are illustrative, not final universal coefficients. Preserve poor-to-epic progression and
represent raid epics explicitly while extending the existing six-band collection model.

Screenshot sample totals are illustrative, never fabricated progress. Item/creature names
pictured on a different map require verified substitutions documented in the catalog.

Use only this checkout for game-data discovery. Do not inspect or use any installed
Azeroth Survivors application as a source. Fetch the separate data through this repository's
`scripts/fetch-data.sh` into gitignored `wow-data/Data`, and pass that exact path via
`WOW_DATA` for validation. Keep validation scratch files under `target/`.

Unspecified balance coefficients are implementation decisions: centralize and document
them, test monotonicity/caps, and distinguish them from numbers explicitly specified by the
references. Talent XP must eventually incorporate survival time, kills, defeated bosses,
win, later-zone scaling and Veteran's Lessons. Spawned bosses are not defeated bosses. Initial tuning retains the level × 2, kills / 10
and win 40 (loss 8) base, adds one XP per ten survived seconds and ten per defeated
boss, then adds 10% per later campaign zone. The award is capped by the hero's 51-point
limit. Catch-up compares the best hero of the class with the best hero of the account.

## Phase 1 — progression integrity

1. Credit the selected valid hero at settlement; handle absent/stale/wrong-class selections
   without corrupting another class. Keep class-wide drops and clears class-wide.
2. Make rewards idempotent for repeated settlement, and cap XP without integer overflow.
3. Use one campaign encounter schedule for displayed times and actual spawns. Resolve
   names against the map roster. A missing encounter must prevent content readiness.
4. End campaign runs only when the final scheduled boss can actually be defeated. Preserve
   the non-campaign arena timer where no campaign schedule exists.
5. Count defeated bosses separately and implement the documented XP inputs and catch-up.

Acceptance: selected second hero receives rewards, first hero does not; repeat settlement
cannot pay twice; save/reload preserves rewards. Every listed encounter spawns exactly
once at its listed threshold, including across a large time step. Final drops remain
obtainable; dead heroes cannot win simultaneously. UI and combat use the same drop odds.

## Phase 2 — equipment and persistence

2026-10-25 evidence: all 39 account tests pass, including a real temporary-file
write failure that preserves the old save, keeps dirty state and succeeds on
retry. A native full restart retained a common equipped helmet, a legendary
cosmetic override and an explicitly empty shoulder slot with identical totals.
The screenshot is inline in the changelog. Explicit appearance buttons work.
The automated macOS Shift-right-click mismatch was traced to native aggregate
modifier flags arriving without physical key events. Reading those flags in event
order now selects the legendary look while retaining the common equipped helmet
and unchanged totals. A following ordinary right-click removed/restored equipment
without changing the look. Five shortcut regressions cover key release, aggregate
flags, focus loss, window destruction and window isolation. Native macOS shortcut
acceptance is verified for this fixture; other platforms/model variants remain open.


Add per-class selected gear for all nine slots, with explicit empty slots. Migrate old
saves to their former strongest-owned loadout once. New drops fill an empty slot but do
not replace a deliberate choice. Reject unowned, invalid-slot and out-of-range choices.
Store appearance separately: worn item, a collected override, or hidden. Validate all
save references, preserve old heroes/specs/achievements, and retain dirty state after a
failed write. Keep saves under the existing local-state resolver.

Acceptance: choosing a lower-quality piece changes combat and displayed totals; owning a
stronger piece alone does not. Unequipping removes its stats and a full-set bonus. Duplicates never increase power. Cosmetic changes never alter stats. Class isolation,
legacy migration, malformed references and round trips have regression tests.

## Phase 3 — collection and character surfaces

Character Look checkpoint (2026-10-25): the wide Features column now ends in
shared appearance actions rather than repeating the full Collection detail panel.
“Inspect in Collection” retains the selected item and exposes its full sources,
comparisons and equipment actions there. Native 1280×720 verified helmet hiding
with unchanged bonuses and navigation to the selected shoulder; 640×720 verified
reachable actions below equipment. Captures are inline in the changelog. Native
build, workspace Clippy, formatting and render-pass lint pass. Other race/model
and resolution combinations remain open.

Feature-control presentation checkpoint (2026-10-25): the five appearance controls
use full-width bordered rows with centered labels/counters and edge-aligned arrows,
following `intentions/survivors_character_tab.png`. Native 1280×720/640×720 checks
verified increment/decrement and stable compact scroll position. The wide capture
also shows the dressed model and rotation controls fitting the central column.
Inline captures are in the changelog; native build, workspace Clippy, formatting
and render-pass lint pass. This does not close the full race/model/resolution matrix.

Character composition checkpoint (2026-10-25): the name/class now sit inside the
central column above a larger proportional model pane. The default view no longer
stacks roster, creation, renown and relic controls below the model; Manage heroes
opens those controls in the central column. Selection/creation returns to the model,
and an absent selected hero exposes management automatically. All saved heroes are
listed rather than only the first eight. Equipment and appearance columns remain
available, and preview-file navigation resets only the local management view.
167 Survivors app tests pass following the workspace test build, with Clippy,
formatting, render-pass lint and a one-minute native build. The direct 1280×720
Character run could not be inspected because the Mac was locked and was stopped.
Rendered fit, Manage/Back/Escape interaction, large-roster scrolling and updated
README screenshots still need verification; this is not visual acceptance.

Build the nine-by-six collection grid with quality headers, owned/unseen states and
selected-item detail. Use catalog-backed item icons, names, source, chance, stats, worn
state and appearance controls. Show real collection counts, never 54 named placeholders.
Character has appearance controls, equipment, summed bonuses, and a rotatable dressed
paper doll. Share selected hero/race/sex/features with the existing appearance flow.
Use the reference gold/black frame, spacing, hierarchy and readable scroll behavior.

Acceptance: every owned cell can be inspected and equipped; worn and cosmetic changes
appear on the paper doll and in a run; tooltips remain inside the viewport. Test empty,
partial and full accounts at 1280×720 and 1920×1080 plus UI scaling. Compare screenshots
with the intention images; compilation alone does not establish visual completion.
Apply the responsive-layout contract below to every surface, including the combat HUD
and draft cards, rather than treating these two resolutions as fixed design canvases.

Collection shares Character’s inline faction portraits and race/sex controls. Live
1280×720 edits carried through to the Character model, with all nine empty equipment
slots visible. The full account/resolution matrix remains outstanding.

Collection cells now render verified artwork with dim unowned pieces and corner E/L badges.
Owned icons, combined badges and legendary selection passed at 1440×900; unowned artwork
passed at 1280×720. Missing-item entries share dimmed icons, and rarity rows list zones
from implemented drops. Clicking a source icon selected its matching grid cell and details
at 1280×720. Missing content remains a placeholder, pending phase 4.

Current Character checkpoint: equipment rows now include runtime item icons, selected-item
highlights and aligned empty slots, shared with Collection. Live 1440×900 checks covered
all nine icons, selection/details and unequip/re-equip; the native build and 71 focused tests passed.
Proportional scroll columns expose worn equipment,
summed equipment bonuses and shared item/look actions alongside existing hero controls.
Named heroes now persist independent race/sex choices, with legacy migration and invalid-save
coverage in the 57-test Survivors suite. The 2026-10-25 compact acceptance checkpoint
verified Warrior → Paladin → Warrior switching, saved sex/model/class gear and
restoration of all five Warrior feature values in a ten-hero account. Roster and
creation controls fit at 640×720. Actual 1920×1080 acceptance remains open: macOS
clamped the requested window, so that capture was rejected.
The current hero uses the stock player compositor and attachments for selected armor,
weapons and independent cosmetic overrides in both the world and full-body preview.
The [gear visual catalog](SURVIVORS-GEAR-VISUALS.md) maps 91 distinct existing item names
to vanilla identities and supplies item-detail icons; class-specific Atiesh variants now
resolve through pinned quest rewards. Missing rarity cells remain unresolved. At 1440×900, a seeded human male account verified plate armor,
helmet/chest hiding, weapon appearance independent of equipped bonuses, and world entry.
Native build, 44 core tests and 62 Survivors/preview tests passed. The preview now fits
rigid attachment bounds using proportional padding; Thunderfury framing was checked at
1440×900 and 1280×720, rotation at 1440×900 and a fixed uncached axe replacement at
1280×720. Pending local attachment loads
keep the equipment resolver awake until ready. Weapon poses, other races, sexes and
ranged attachments still need visual checks. Turn buttons work. Character now exposes class-valid race and sex choices beside the
loadout, persisted per named hero through the existing body model. Missing sexes now
use another verified trainer preset of the same race; every existing class race resolves
both sexes against the local DBC/model data without changing its preferred presets. Skin, face, hair
and race-specific features now have bounded dials and randomization, persisted per hero.
Feature descriptor changes refresh the stock compositor and preview. Native build and 71
focused tests passed; live edits, switching heroes and restart persistence were checked at
1440×900 and 1280×720. Race portraits now use the runtime atlas, grouped by faction,
with selected-sex artwork and unavailable class races dimmed and inert. Live portrait/sex
selection and disabled Paladin choices passed at 1280×720. Broader multi-resolution
acceptance remains unfinished. Creation-time feature readiness now participates in the page refresh signature, while
ordinary dial values do not. A newly created Paladin exposed all five controls immediately;
hair edits retained preview rotation in live 1280×720 validation. Feature-only edits update counters in place, preserving the
Character tree, scroll state and preview. Transient dark or missing
button glyphs still appear after UI actions, even without a full rebuild; removing shadows
or raising caption layers did not resolve the broader issue. Model dragging now consumes
ordered window events so short gestures are retained. Short/reverse drags and rejection of
presses outside the preview passed live at 1280×720; sensitivity follows preview width.

Drop Tables now has a proportional zone browser and encounter/item rows alongside shared
appearance, equipment and item details. Browsing does not change the run destination.
Source counts and entries come from DROPS; unimplemented tables remain explicit. Native
build and 72 focused tests passed, including all twelve views’ source/action wiring.
Live 1280×720 checks covered Elwynn/Duskwood, source selection, missing-content state and
returning to the unchanged run destination. Drop Tables, Collection sources and item details now show effective Keen Eye rates using
the same capped probability function as combat. Boss eligibility comes from the encounter
schedule, not the zone-summary boss name. Live rank-two labels (70% from 60% base) fit
at 1280×720. The wider layout/content matrix remains open.

## Phase 4 — all twelve zones and gear content

Read-only [location evidence](SURVIVORS-LOCATIONS.md) now records local map metadata and
terrain-supported candidates for Warsong Gulch, Arathi Basin and Alterac Valley. Reproduce
with `survivors-audit <Data> --locations`; none of these candidates establishes readiness.


Complete the ordered path in SURVIVORS.md. For each zone, verify terrain/map coordinates,
creature displays, named encounter roster, schedule, boss, slot/band drops and class item
mapping. Use body-collision floors for heroes, enemies, companions and mobility destinations, including
WMO interiors and bridges. A local grounding probe starts two yards above the feet and ends
eight yards below; it must hold the current/authored height when no floor is available rather
than snap to distant terrain. Jump height stays relative to those feet. Validate outdoor
slopes and layered interior fixtures separately. Bounded grounding is not wall collision,
spawn placement, pathfinding, or proof of indoor camera visibility; those need their own
traversability checks before a zone can become ready.

Spawn candidates must fit their body radius inside the arena and have support under the
center and four footprint edges. Clamping a ring near the boundary must not place a monster
on the hero: require at least six yards, or body radius plus two yards if larger. Retry
other directions when support is missing. Scheduled bosses remain pending until spawned;
initial chests and surrounding waves are planned as complete batches before counters advance.
A missing streamed floor must neither consume an encounter nor duplicate part of a batch.
These checks do not prove line of sight or a walkable route through walls.

Horizontal movement sweeps a capsule against body collision and slides along obstacles;
Survivors uses its existing separation radius with a one-yard cylindrical section and a
0.03-yard floor clearance. Blink and Charge stop along their straight path and require a
supported destination. Charge can stun only a reached target with a clear solid-world line
of sight; enemy contact damage likewise requires a clear line. Validate walls, tangential
movement, outdoor slopes, stairs and indoor combat. This collision layer is not route finding,
and vertical jump ceilings need a separate acceptance check. Steep contact normals
must constrain sliding horizontally: allowing a capsule to climb a sloped trunk and then
snap to the ground can push it inside the hull. The placed Un'Goro tree regression checks
80 grounded steps for capsule penetration; see [location evidence](SURVIVORS-LOCATIONS.md).
This does not establish navigation or complete zone readiness.

Audit Elwynn tree trunks specifically: visible trunks must block heroes, enemies, companions
and mobility sweeps. Trace missing collision through source model collision, transforms,
streaming registration and the body-query filter; do not assume visible foliage triangles
are a suitable solid collider. Where source trunk collision is absent, author a conservative
trunk proxy without making the whole canopy impassable. Verify contact from several sides,
sliding, tile unload/reload and representative tree variants. Missing tree collision is an
open defect even when walls already pass collision tests.

Navigation is a separate requirement: choose a navmesh or equivalent walkable-route solution
after auditing the collision data. Actors must route around trunks and walls, traverse valid
stairs/doorways, respect disconnected floors, and recover when streamed routes change. A
navmesh does not repair a missing movement collider; a movement sweep does not prove routing.
Do not mark a zone ready from floor probes alone.

Connect item ids to names/icons/displays from available catalogs. Fill all 54
cells for each of nine classes with verified content. Content readiness is derived from
these dependencies rather than a trusted boolean. Never unlock a label with no world.

Acceptance: every zone is traversable after its previous clear and talent gate; every
piece has an obtainable source and every source resolves to a scheduled creature. New
accounts start in Elwynn; old clears retain stable identities. Drop Tables and actual
awards read one catalog, including Keen Eye. Verify each world location against game data.

## Phase 5 — talents that change play

Audit the checkout’s configured data catalog with
`cargo run -p survivors-audit -- "$PWD/wow-data/Data" --talents` before assigning effects
to real talent IDs. The audit reports each class's native tab order, node IDs, row/column,
rank spells and prerequisites, and rejects an order that differs from Survivors' three
tree identities. Use an explicit checkout data path; never discover another installed game.

The Talents page uses three illustrated tree panels and a separate spec/details sidebar.
Load each tree’s four background tiles from the explicitly configured client data. Display
rank counters, prerequisite connections, and 10/20/30 milestone rows; keep all controls
reachable at 1280×720 and 1920×1080, including after spending or refunding a rank.

Keep three real named class trees and three saved specs. Define explicit supported talent
and milestone effects against Survivors abilities. Use vanilla rank patterns and row
requirements while following the named Survivors milestone design. Implement each effect
through shared combat/stat calculations, not tooltip-only prose. Paladin's named milestones
are the first vertical slice; then cover every class. Surface effects and selected spec
on the door and run sheet. Preserve migration from the skeleton arrays.

Individual Paladin talents use audited DBC node IDs and vanilla rank caps. Their
listed effect replaces the provisional +1% damage per spent rank; it does not stack on top
of that placeholder. All spent ranks still count toward tree milestones. These are explicit
Survivors conversions: there is no mana resource or separate parry/miss roll in this mode.

| Talent ID | Name | Ranks | Survivors effect per rank |
| --- | --- | --- | --- |
| 1450 | Divine Strength | 5 | +2% damage |
| 1449 | Divine Intellect | 5 | +1% cooldown reduction |
| 1422 | Improved Devotion Aura | 5 | +8 armor |
| 1630 | Precision | 3 | +1 percentage point critical chance |
| 1423 | Toughness | 5 | +6 armor |
| 1629 | Anticipation | 5 | +1 percentage point damage reduction |
| 1401 | Improved Blessing of Might | 5 | +2% damage |
| 1403 | Deflection | 5 | +1 percentage point damage reduction |
| 1411 | Conviction | 5 | +1 percentage point critical chance |
| 1634 | Pursuit of Justice | 2 | +4% movement speed |
| 1631 | Improved Judgement | 2 | Judgement cooldown shortened by 10% |
| 1521 | Improved Hammer of Justice | 3 | Hammer of Justice cooldown shortened by 10% |
| 1627 | Holy Power | 5 | Holy Shock +2 percentage points critical chance |
| 1435 | Consecration | 1 | Consecration +20% damage |
| 1502 | Holy Shock | 1 | Holy Shock +25% damage |
| 1444 | Healing Light | 3 | Holy Shock heals 1% maximum health once per successful cast |
| 1443 | Improved Lay on Hands | 2 | Divine Shield heals 5% maximum health once per successful cast |
| 1425 | Guardian's Favor | 2 | Divine Shield +15% duration |
| 1421 | Redoubt | 5 | Divine Shield cooldown shortened by 3% |

Healing Light adds to Daybreak's 3% healing, and can heal before Daybreak is unlocked.
Improved Lay on Hands is a Survivors conversion onto Divine Shield because Lay on Hands
is not in the eight-ability class pool. Guardian's Favor and Redoubt improve that same
survival window rather than adding absent block or blessing mechanics. Healing occurs
only after a successful cast, once per cast regardless of target count, and cannot exceed
maximum health. Divine Shield still requires a nearby enemy before it casts. Its duration
modifier affects actual invulnerability; shield absorption is a separate mechanic.


Spell-specific cooldown reductions add within that node and multiply with the ability's
normal cooldown and milestone modifier. Spell damage multiplies with ordinary damage
and milestones; critical chance adds and is capped by shared combat parameters. These
nodes improve abilities already available in the class pool; they do not grant an extra
ability slot or bypass level-up acquisition. They never add area targets or extra healing
triggers. Snapshot ranks with the selected hero/spec so edits cannot alter an active run.


Use the selected spec's ranks in both the account sheet and run-start snapshot. Reset
removes only that spec's effects. A talent detail line must distinguish these implemented
conversions from nodes whose individual effects still need implementation; do not display
vanilla spell prose as if mana, parry or other absent mechanics were functional. The catalog
audit must reject mismatched IDs, names or rank caps for supported nodes.

Warrior individual talents now use the same selected-spec snapshot and comparison path.
The following audited nodes replace their provisional +1% damage per rank; spent points
still count toward existing tree milestones. These are Survivors conversions, not claims
that rage, parry, or the original MMO talent behavior exists. Unsupported individual nodes remain unfinished; all nine named Warrior milestones
have implementations described below.

| Talent ID | Name | Ranks | Survivors effect per rank |
| --- | --- | --- | --- |
| 124 | Improved Heroic Strike | 3 | Heroic Strike cooldown shortened by 5% |
| 127 | Improved Rend | 3 | Deep Wounds bleed damage +10% |
| 121 | Deep Wounds | 3 | Direct criticals add 4% of hit damage to the three-second bleed |
| 130 | Deflection | 5 | +1 percentage point damage reduction |
| 128 | Improved Thunder Clap | 3 | Thunder Clap cooldown shortened by 10% |
| 662 | Impale | 2 | +10 percentage points critical damage |
| 133 | Sweeping Strikes | 1 | Sweeping Strikes +20% damage |
| 135 | Mortal Strike | 1 | Mortal Strike +25% damage |
| 158 | Booming Voice | 5 | Demoralizing Shout +10% radius |
| 161 | Improved Demoralizing Shout | 5 | Demoralizing Shout +10% weakening duration |
| 157 | Cruelty | 5 | +1 percentage point critical chance |
| 166 | Improved Cleave | 3 | Cleave +10% damage |
| 1542 | Improved Execute | 2 | Execute cooldown shortened by 10% |
| 138 | Anticipation | 5 | +1 percentage point damage reduction |
| 140 | Toughness | 5 | +6 armor |
| 147 | Improved Revenge | 3 | Retaliation +10% reflected damage |
| 150 | Improved Shield Wall | 2 | Retaliation +15% duration |
| 153 | Last Stand | 1 | Retaliation heals an additional 5% maximum health once per cast |
| 167 | Bloodthirst | 1 | Execute heals an additional 2% maximum health once per cast |
| 160 | Piercing Howl | 1 | Demoralizing Shout gains two seconds of 50% slow; bosses are slowed 25% |
| 152 | Concussion Blow | 1 | Heroic Strike gains one second of stun; bosses receive 30% duration |
| 1601 | Shield Specialization | 5 | Heroic Strike absorbs +1% maximum health per rank for three seconds |
| 145 | Improved Shield Block | 3 | Heroic Strike absorb lasts +0.5 seconds per rank; requires an absorb source |
| 148 | Shield Slam | 1 | Heroic Strike absorbs an additional 3% maximum health |
| 155 | Enrage | 5 | After health damage from a nonlethal hit, +4% damage per rank for four seconds |

Ranks add within each node; cast multipliers compose with ordinary stats and legendary
bonuses. Booming Voice changes the actual shout area, never its number of locations.
Damage talents improve already-acquired abilities rather than granting free abilities.
Improved Shield Wall extends the actual Retaliation buff, including Death Wish's incoming-
damage risk when that milestone is active. Last Stand and Bloodthirst node healing adds to
the same-name milestone healing, caps at maximum health, and applies once per successful
cast regardless of extra targets. The nodes do not grant the corresponding ability.
The data audit now checks every supported class catalog for missing nodes as well as
wrong names and rank limits. Save format remains unchanged.

Warrior Arms milestones now have these explicit Survivors effects in addition to the
existing baseline tree stat bonuses:

| Points | Milestone | Behavior |
| --- | --- | --- |
| 10 | Deep Wounds | Critical Cleave, Heroic Strike, Execute and Mortal Strike hits bleed for 12% of their pre-execute/echo hit damage over three seconds |
| 20 | Sweeping Strikes | Sweeping Strikes aura radius multiplied by 1.30; no additional areas |
| 30 | Mortal Strike | Mortal Strike damage multiplied by 1.25 |

Deep Wounds snapshots its damage, keeps one bleed per enemy, and refreshes to three seconds
while retaining the stronger DPS and next tick time. A weaker critical cannot weaken the
active bleed. Ticks cannot crit or recursively apply the bleed; they retain the existing
DoT multistrike/leech rules and appear under Deep Wounds in the damage meter. The six-effect
DoT cap still applies. Damage-over-time now processes every due whole-second tick within
its remaining duration, including the final tick across a long frame; it never ticks beyond
expiry. Individual Deep Wounds node ranks and the Fury/Protection milestones are specified below.
These coefficients are provisional Survivors balance, not original MMO behavior.

Warrior Fury milestones use these explicit Survivors conversions:

| Points | Milestone | Behavior |
| --- | --- | --- |
| 10 | Piercing Howl | Demoralizing Shout also slows movement by 50% for three seconds; bosses by 25% |
| 20 | Death Wish | Retaliation reflects 30% more damage; incoming damage increases by 10% only while that buff is active |
| 30 | Bloodthirst | A successful Execute cast heals 3% maximum health once, capped at full health |

Piercing Howl preserves the shout's existing damage weakening, keeps any stronger active
slow, and does not create extra areas. Death Wish snapshots the reflected damage at cast;
its incoming penalty multiplies raw damage before normal mitigation and shield absorption,
while immunity still prevents damage. Other reflection buffs do not trigger the penalty.
Bloodthirst heals per cast, not per target, echo, or kill; Execute still needs a valid target.
Individual talent ranks and the existing baseline tree bonuses compose with these effects.
Protection milestones are specified below.

Warrior Protection milestones use these explicit Survivors conversions:

| Points | Milestone | Behavior |
| --- | --- | --- |
| 10 | Last Stand | A successful Retaliation cast heals 10% maximum health once, capped at full health |
| 20 | Concussion Blow | Heroic Strike stuns each struck target for one second; bosses for 0.3 seconds |
| 30 | Shield Slam | A successful Heroic Strike cast grants an absorb of 3% maximum health for three seconds, once per cast |

Concussion Blow preserves a longer existing stun. Shield Slam does not multiply with target
count or stack absorption. It replenishes an equal/weaker active shield or replaces an
expired shield; it neither weakens nor extends a stronger active shield. Normal shield
absorption and the HUD use the same existing shield state. Last Stand composes with Fury's
Retaliation damage/risk. All nine Warrior milestone behaviors are implemented, but full
manual build validation and remaining individual talent effects are still required.
Coefficients are provisional Survivors balance rather than original MMO behavior.

Paladin milestone balance uses the following explicit Survivors behavior. These are custom
survival-mode effects, not claims about the original MMO spells. Existing per-point/stat
bonuses remain the baseline; named milestone behavior is additional. Snapshot tree points
from the selected hero/spec at run start, including legacy spreads. A cast benefit occurs
only on a successful cast and once regardless of how many targets that cast hits.

| Tree | 10 points | 20 points | 30 points |
| --- | --- | --- | --- |
| Holy | Divine Favor: Holy Shock gains 20 percentage points of critical chance | Daybreak: Holy Shock restores 3% maximum health, capped at full health | Light's Grace: Holy Shock cooldown multiplied by 0.8 |
| Protection | Hallowed Ground: Consecration radius and duration multiplied by 1.25 | Ardent Defender: reduce the portion of damage below 35% health by 20%, after shields, including hits crossing that threshold | Hammer of the Righteous: Hammer of Justice gains two direct targets |
| Retribution | Seal of Command: Judgement damage multiplied by 1.25 | The Art of War: Judgement advances an owned Hammer of Wrath cooldown by one second, never below zero | Divine Storm: each Judgement cast releases one hero-centered six-yard area dealing 50% of its modified damage, without critical strikes or recursive procs |

Area bonuses scale Divine Storm's radius; extra targets cannot duplicate its location or
Daybreak healing. Combat and spell tooltips must use shared cast parameters. Verify the
thresholds, mitigation across the threshold, healing cap, cooldown floor, class isolation,
spec selection and persistence. Remaining individual talent nodes and other classes require
the same explicit behavior contract and combat integration.

Mage now has implemented behavior for all nine tree milestones. Arcane and Fire
use these provisional Survivors conversions, additional to individual node ranks
and existing tree stat bonuses:

| Tree | 10 points | 20 points | 30 points |
| --- | --- | --- | --- |
| Arcane | Arcane Focus: Arcane Missiles/Explosion gain 5 percentage points critical chance | Presence of Mind: each successful Arcane Missiles cast advances an owned Arcane Explosion cooldown by 2 seconds, never below zero | Arcane Power: Arcane Missiles/Explosion deal 20% more damage |
| Fire | Improved Fireball: Fireball deals 25% more damage, including splash | Critical Mass: Fire spells gain 10 percentage points critical chance, additive with node ranks | Combustion: Fire spells gain 20 percentage points critical damage |

Fire scope is Fireball, Fire Blast, Flamestrike, Pyroblast and Blast Wave; Ignite
remains excluded from recursive cast benefits. The increased direct critical damage
can still strengthen its existing Ignite seed. These milestones do not add targets
or area locations. Presence of Mind never grants an unowned spell. Shared cast
parameters and the completion handler drive both combat and tooltip descriptions.
Frost's existing milestones remain unchanged; individual unimplemented Mage nodes
and full live combat/balance acceptance remain open.

Shaman now implements all nine milestones using its existing ability pool. These
are provisional automatic Survivors conversions; they do not claim the original
MMO spells or add a mana resource, Healing Wave spell, or a new totem type.

| Tree | 10 points | 20 points | 30 points |
| --- | --- | --- | --- |
| Elemental | Convection: Lightning Bolt, Chain Lightning and all three shocks have 10% shorter cooldowns | Elemental Focus: Lightning Bolt/Chain Lightning gain 10 percentage points critical chance | Elemental Mastery: Chain Lightning always critically strikes |
| Enhancement | Ancestral Knowledge: each successful Earth Shock cast grants a non-stacking 3% maximum-health absorb for 3 seconds | Elemental Weapons: Searing/Magma Totem and Lightning Shield deal 20% more damage | Stormstrike: Earth Shock deals 25% more damage and hits one additional direct target |
| Restoration | Improved Healing Wave: deploying Searing/Magma Totem heals 3% maximum health once | Nature's Swiftness: Earth/Flame/Frost Shock cooldowns multiply by 0.85, composing with Convection | Mana Tide Totem: deploying either Searing/Magma Totem advances an owned counterpart's cooldown by 2 seconds |

Totem pulses cannot repeat deployment healing or cooldown benefits. Healing caps
at maximum health, cooldowns stop at zero, and the absorb never stacks or replaces
a stronger active shield. Totem area sizes and counts remain unchanged. Milestone
thresholds, cast parameters and tooltips share the engine-independent rules.
Thirteen audited Shaman nodes now have individual effects, replacing their generic
damage fallback. Values below are per rank and provisional Survivors tuning.

| Talent ID / name | Ranks | Effect |
| --- | --- | --- |
| 564 Convection | 5 | Lightning and shocks: 2% shorter cooldown; mana-cost conversion |
| 563 Concussion | 5 | Lightning and shocks: +2% direct and periodic damage |
| 561 Call of Flame | 3 | Searing/Magma Totem: +5% damage |
| 575 Reverberation | 5 | All three shocks: 3% shorter cooldown |
| 562 Call of Thunder | 5 | Lightning Bolt/Chain Lightning: +1 percentage point critical chance |
| 721 Lightning Mastery | 5 | Lightning Bolt/Chain Lightning: 4% shorter cooldown; cast-time conversion |
| 565 Elemental Fury | 1 | Lightning and shocks: +50 percentage points critical damage |
| 614 Ancestral Knowledge | 5 | +4 maximum health; mana conversion |
| 613 Thundering Strikes | 5 | +1 percentage point hero critical chance, including spells/totems; conversion |
| 607 Improved Lightning Shield | 3 | Lightning Shield: +5% damage |
| 615 Toughness | 5 | +8 hero armor; conversion |
| 595 Totemic Focus | 5 | Searing/Magma Totem: 3% shorter deployment cooldown; mana-cost conversion |
| 594 Tidal Mastery | 5 | Lightning and shocks: +1 percentage point critical chance; deployment healing cannot crit |

Lightning means Lightning Bolt/Chain Lightning; shocks means Earth/Flame/Frost Shock.
Tests cover rank caps, reset, class/spell isolation, milestone composition and direct,
periodic and totem parameters. Native purchases verify green previews and persisted
ranks on an isolated character. The other 33 nodes and full live combat/balance
acceptance remain open.

Warlock has sixteen audited individual effects. Values are per rank and provisional:

| Talent ID / name | Ranks | Effect |
| --- | --- | --- |
| 1002 Nightfall | 2 | Corruption ticks have a 2% chance per rank to ready an owned Shadow Bolt; instant-cast conversion, no stored charge |
| 1041 Siphon Life | 1 | Corruption/Curse of Agony heal 5% of health actually removed; adds to the 30-point milestone for 10% total; automatic conversion |
| 1021 Grim Reach | 2 | Corruption, Curse of Agony and Death Coil: +10% targeting range |
| 964 Destructive Reach | 2 | Shadow Bolt, Immolate, Searing Pain, Shadowburn and Rain of Fire: +10% targeting range; no radius or Hellfire change |
| 1003 Improved Corruption | 5 | Corruption: 4% shorter cooldown; cast-time conversion |
| 1284 Improved Curse of Agony | 3 | Curse of Agony: +2% periodic damage |
| 1042 Shadow Mastery | 5 | Shadow Bolt, Corruption, Curse of Agony, Death Coil, Shadowburn: +2% damage |
| 1223 Demonic Embrace | 5 | +5 maximum health; stamina conversion |
| 944 Improved Shadow Bolt | 5 | Shadow Bolt: +4% damage; conversion from critical-hit vulnerability |
| 941 Cataclysm | 5 | Destruction spells: 2% shorter cooldown; mana-cost conversion |
| 943 Bane | 5 | Shadow Bolt/Immolate: 4% shorter cooldown; cast-time conversion |
| 981 Devastation | 5 | Destruction spells: +1 percentage point critical chance |
| 965 Improved Searing Pain | 5 | Searing Pain: +2 percentage points critical chance |
| 961 Improved Immolate | 5 | Immolate: +5% direct and periodic damage; Survivors conversion |
| 967 Ruin | 1 | Destruction spells: +50 percentage points critical damage |
| 966 Emberstorm | 5 | Immolate, Rain of Fire, Hellfire, Searing Pain: +2% damage |

Destruction includes Shadow Bolt, Immolate, Rain of Fire, Hellfire, Searing Pain
and Shadowburn. These definitions replace generic damage fallback and do not grant
unowned abilities. Area locations, target counts and durations stay unchanged.
The remaining 34 nodes and live balance acceptance remain open. No demon companion
system is claimed by this first group.

All nine Warlock milestones now use explicit automatic Survivors adaptations:

| Tree | 10 points | 20 points | 30 points |
| --- | --- | --- | --- |
| Affliction | Improved Corruption: Corruption cooldown ×0.9 | Nightfall: each successful Corruption cast advances an owned Shadow Bolt cooldown by 1 second | Siphon Life: Corruption/Curse of Agony heal 5% of enemy health actually removed |
| Demonology | Improved Healthstone: each successful Death Coil cast heals 3% maximum health, additional to its drain | Demonic Sacrifice: all nine Warlock spells gain 10% direct/periodic damage | Soul Link: Death Coil grants a non-stacking 5% maximum-health absorb for 3 seconds |
| Destruction | Improved Shadow Bolt: Shadow Bolt damage ×1.25 | Ruin: Destruction spells gain 20 percentage points critical damage | Conflagrate: each successful Immolate cast advances an owned Shadowburn cooldown by 2 seconds |

Damage/cooldown bonuses multiply individual nodes; critical damage adds to Ruin
training. Cast benefits occur once, not per target or tick. Cooldowns stop at zero
and never grant unowned spells; absorbs preserve stronger active shields. Siphon
Life uses the existing direct/periodic damage pipeline, excludes overkill and dead
targets, caps healing and cannot revive the player. Demonology conversions require
no demon. These provisional rules still need live combat and balance acceptance.

Druid has seventeen audited individual effects. Values are per rank and provisional:

| Talent ID / name | Ranks | Effect |
| --- | --- | --- |
| 762 Improved Wrath | 5 | Wrath: 4% shorter cooldown; cast-time conversion |
| 763 Improved Moonfire | 5 | Moonfire: +2% direct/periodic damage and +2 percentage points critical chance |
| 782 Improved Thorns | 3 | Thorns: +10% retaliation damage |
| 792 Vengeance | 5 | Wrath/Moonfire/Starfire: +10 percentage points critical damage |
| 784 Improved Starfire | 5 | Starfire: 3% shorter cooldown; cast-time conversion |
| 783 Moonglow | 3 | Wrath/Moonfire/Starfire/Rejuvenation: 3% shorter cooldown; mana-cost conversion |
| 790 Moonfury | 5 | Wrath/Moonfire/Starfire: +2% direct/periodic damage |
| 794 Thick Hide | 5 | +8 hero armor; conversion |
| 807 Feline Swiftness | 2 | +5% hero movement speed; conversion without forms |
| 798 Sharpened Claws | 3 | +2 percentage points hero critical chance; conversion without forms |
| 830 Improved Rejuvenation | 3 | Rejuvenation: +5% healing |
| 828 Gift of Nature | 5 | Rejuvenation: +2% healing |
| 821 Improved Mark of the Wild | 5 | +4 hero armor; permanent Mark conversion |
| 824 Improved Healing Touch | 5 | Rejuvenation: 3% shorter cooldown; cast-time conversion |
| 823 Nature's Focus | 5 | Rejuvenation: +2% healing; healing reliability conversion |
| 841 Subtlety | 5 | +1 percentage point hero damage reduction; threat conversion |
| 843 Tranquil Spirit | 5 | Rejuvenation: 2% shorter cooldown; healing mana-cost conversion |

Healing previews identify healing explicitly. Individual Rejuvenation ranks multiply
its existing total heal over the same duration; those ranks do not add an instant heal.
Moonfire previews show both damage and critical chance. These effects replace the
generic damage fallback. Remaining 30 nodes, forms/feral attacks and live
combat/balance acceptance remain open.

All nine Druid milestones have explicit automatic Survivors adaptations:

| Tree / points | Effect |
| --- | --- |
| Balance 10 · Nature's Grasp | Entangling Roots cooldown ×0.85 |
| Balance 20 · Improved Moonfire | Moonfire direct/periodic damage ×1.25 |
| Balance 30 · Moonkin Form | Wrath/Moonfire/Starfire +5 percentage points critical chance; no form required |
| Feral 10 · Ferocity | Thorns cooldown ×0.85; feral attacks remain separate work |
| Feral 20 · Heart of the Wild | Thorns cast grants a 5% maximum-health absorb for 3 sec; no stacking |
| Feral 30 · Leader of the Pack | Thorns cast heals 3% maximum health once, not per retaliation |
| Restoration 10 · Improved Healing Touch | Rejuvenation total healing ×1.2 |
| Restoration 20 · Nature's Swiftness | Rejuvenation cooldown ×0.85 |
| Restoration 30 · Swiftmend | Rejuvenation cast also immediately heals 5% maximum health; preserves the HoT |

Multipliers compose with individual training. These milestones do not add targets,
area locations or duration; periodic damage does not gain critical strikes. Cast
healing caps at maximum health and cannot revive; absorbs preserve stronger active
shields. Core/app tests cover thresholds and runtime parameters/finish behavior.
Native screenshots verify preview text on an empty spec, not live combat balance.

Acceptance: each selectable talent has a documented implemented effect; 10/20/30 milestone
threshold tests and representative combat tests prove those behaviors. Switching specs
changes the next run without losing either spread. Free reset and right-click refund keep
all dependency constraints valid. A row requires five points per tier in strictly earlier
rows of the same tree, after the proposed edit; its own ranks and deeper ranks cannot
support it. Preserve the full prerequisite rank (DBC stores that rank zero-based), and
validate both proposed spends and refunds against the catalog. No points leak across heroes.

## Phase 6 — codex, achievements and rewards

Turn jewelry lists into icon grids with details and unseen states. Expand only with actual
implemented offers; ensure every legendary amulet maps to a class ability and seen status
persists when dealt, including abandoned runs. Provide achievement summary, recent earns,
five categories, progress and total points calculated from the catalog. Expand feats with
real content. Finish reward pips, costs, two perk refunds, shared point economy, verified
companion looks, distinct instincts and cosmetic tabards.

Acceptance: purchases and refunds cannot duplicate points or reset companion investment;
perks change the described run behavior; companions fetch coins/wisps without opening
chests or behaving like a magnet. All cosmetics use verified displays. Totals reconcile
across tabs, saves and achievements; locked rows explain requirements.

Codex source status (2026-10-26): `Account::legendary_progress` counts recognized
entries in the selected class's ability catalog, excluding duplicate/foreign save
names. `remember_jewelry` records discovery when cards are dealt, including shared
jewelry; these discoveries persist without equipping run-only effects. The page
uses icon grids and selected-item details. This supersedes the initial raw-count
and missing-discovery defects. Keep expansion tied to implemented offers and
retain the end-to-end acceptance requirements above.

## Phase 7 — integration and visual acceptance

Finish the shared frame and all reference layouts, including Done, Run Options, zone title
on Enter World, equipment bonuses and class/race identity. Test first launch, existing
save, creation, appearance, selected spec, locked/open zones, start/pause/abandon/death/win,
reward settlement, restart and reload. Resolve overflow and keyboard/mouse focus; typing
must not accidentally start a run. Missing art/data should produce an honest usable state.

Run the native play-profile build (`cargo survivors-build`), focused Survivors tests,
`scripts/check.sh`, and wasm check where the wasm toolchain is available. Run repository
land gates before landing a PR. Record actual results and limitations in commit/PR text,
not a claim of passing from code inspection. No auto-merge or deployment is part of this
implementation task.

Complete the responsive UI, bounded card effects, camera visibility options and README
screenshot gallery specified below as part of visual acceptance.

## Evidence and completion

Each phase needs rule tests, save migration coverage where applicable, and an integration
check of the user-visible effect. Use seeded data for probabilistic boundary tests rather
than flaky sampling. Do not write fixtures into the player's save or install. Keep tests
independent of a real install where possible; explicitly report data-dependent skips.

Full completion means all seven phases satisfy their acceptance criteria and all intended
screens and mechanics work together. Missing game data, unverified assets, unresolved
mechanics or unavailable rendering evidence remain open requirements; they are not grounds
to mark the full goal complete. Progress check-ins should name completed behavior, evidence,
and the next unresolved dependency. Commit coherent verified pieces on the implementation
branch as the user requested.

## Development and distribution requirements

The user's additional requirement is fast iteration with as much hot reloading as practical,
and a distributable executable that can offer future updates. These are part of completion,
not optional follow-ups. Follow [development and distribution](SURVIVORS-DEVELOPMENT.md):
measure warm rebuilds, keep engine compilation cached, isolate pure game rules, grow validated
live data/UI reloads, and implement a prompted authenticated updater once its release channel
and signing configuration are defined. A no-op build, pure-rule test, live tuning edit and
packaged restart/update/rollback each need their own evidence. Do not claim arbitrary Rust
hot reloading from asset watching or claim self-updating from the existing local restart.

## Gear profiles and collection acceptance details

Each ordinary quality step adds exactly one stat line and increases all existing lines.
Gloves use health/pickup/area/reduction/damage as their initial stat identity. The user's
example coefficients are initial tuning, not claims about vanilla item stats. Other slots
need their own profiles in the complete item catalog. Poor and raid tiers need explicit
collection identities and obtainable sources; formulas alone do not complete a tier.

Health and pickup bonuses must reach the run sheet and item details. Keep meaningful
fractional percentages visible. Cover exact line counts, monotonic prior lines, lower-quality
selection, duplicate immunity, invalid indices and application-level stat propagation.

The collection must show owned counts by row, equipped and appearance markers, completion
progress, and selectable missing-item sources filtered by the chosen hero's open zones.
Item detail and combat must read the same slot-specific formula. Complete icon-backed cells,
three-column composition and actual appearance rendering, then validate against the reference
screen with the repository-local data. Record test and build evidence in commit messages;
compilation and formula tests do not replace rendered acceptance.


## Responsive interface and bounded effects

Preserve the composition, hierarchy and relative proportions in `docs/intentions/` across
reasonable screen sizes. Use viewport-relative panel sizes, proportional columns, anchors,
and shared UI scaling instead of a single fixed-pixel canvas. Ratios need minimum readable
text and usable hit areas, sensible maximum widths, and bounded spacing; shrinking every
pixel indefinitely is not acceptable. At narrower widths, use deliberate reflow or scrolling
while retaining the reference hierarchy. Keep tooltips, dialogs and essential actions inside
the usable viewport. Pointer hit regions must track the rendered layout.

Cover the front door, every Armory tab, appearance, Run Options/settings, combat HUD,
level-up cards, pause and results. Recompute on live window resize and display-scale changes;
support windowed and fullscreen modes. Validate at least 1280×720 and 1920×1080 (16:9),
1280×800 (16:10), 1280×1024 (5:4), 2560×1080 (ultrawide), and 3840×2160 with an appropriate
readable UI scale. Also check high-DPI scaling and long text/full collections. Treat these as
acceptance targets, not claims of current support. Report unavailable native sizes as
unverified; a window clamped by macOS does not establish the requested framebuffer size.

The level-up card sheen currently appears to extend outside the card. Investigate shader
coordinates, effect bounds, clipping and scaling. Clip sheen to each card's intended shape
and rounded corners; intentional outer glow must be a separate bounded effect. Test every
rarity, hovered/selected cards, the full animation cycle, live resizing and non-default UI
scales. No sheen may wash across neighboring cards, backdrop, text panels or the screen.

## Camera visibility, silhouettes and solid-world push-in

Use separate policies for foliage visibility, body collision and solid-world camera collision.
A canopy obscuring the hero is not by itself a reason to collapse the camera boom. The
single-ray visual-geometry push-in prototype is not the intended foliage solution and does
not establish visual acceptance.

Provide a localized see-through/cutaway effect in obstructing scenery around the character,
large enough to reveal nearby enemies and friendlies as well as the hero. Preserve scenery
outside this region, with soft edges and stable transitions as the hero/camera moves. The
mask must follow the projected character and remain appropriately sized across zoom levels,
aspect ratios and UI scales. Support retained static geometry and ordinary/merged entity
rendering; an entity-only effect misses much of the scenery. Do not change body solidity or
remove whole trees to achieve visibility. Verify alpha-cutout foliage as well as opaque
occluders, and ensure the reveal does not flicker when geometry is culled or streamed.

Add persisted visibility filters for occluded players/friendlies and enemies, independently
selectable (including both off). Render occluded entities as plain-color silhouettes: enemies
red, players/friendlies green. Show only the occluded portions through scenery, retain normal
rendering for visible portions, and classify by allegiance rather than mesh type. Use a depth-aware
shader/pass or equivalent renderer implementation; respect spawn/despawn and normal entity
visibility rules. Expose the controls in settings with immediate visual feedback. Verify
mixed groups behind one obstacle and that each filter affects only its intended group.

Add an explicit persisted “Auto-push camera in” option for solid-world obstructions. When
enabled, caves, instance walls/ceilings and other solid boundaries must constrain the camera
so its near plane does not stick outside the interior. Preserve the user's requested zoom,
then smoothly restore it when clear, without oscillation, clipping or abrupt popping. Do not
treat heroes or enemies as camera obstacles. Keep this control independent of the foliage
cutaway and silhouette filters. Disabling it disables automatic boom adjustment, not actor
collision or the independently enabled visibility effects.

Validate dense Elwynn trees, clear outdoor terrain, cave entrances, narrow interiors,
ceilings, corners and layered rooms separately. Check hero and nearby combat readability,
body blocking, camera clipping, zoom restoration, filter persistence and combinations, streamed
tile changes, frame rate and stability. Floor probes or a readable hero alone cannot establish
all of these behaviors.

## README interface screenshot gallery

Export real screenshots of the implemented checkout UI to `docs/images/screenshots/<milestone>/` and embed them
in README with descriptive captions and alt text. Cover the front door/zone selection,
appearance, Character, Collection/item detail, Drop Tables, Codex, Talents, Achievements,
Rewards, Run Options/settings (including visibility controls), combat HUD, level-up cards,
pause and results. Include representative narrow/wide layouts and foliage cutaway/occluded
silhouettes once implemented. Capture with isolated validation saves and checkout-local data.

Show the actual interface, not intention mockups or fabricated finished states. Record capture
viewport/UI scale in captions or a gallery index, keep images reasonably compressed and legible,
and refresh them when the corresponding interface materially changes. Existing README images
do not establish coverage of missing screens. Review every image for clipping, sheen leakage,
readability and obsolete controls before calling the gallery complete. Unimplemented surfaces
remain pending rather than receiving misleading screenshots.

### Upgrade readability checkpoint

**Updated requirement — inline comparisons.** Upgrade changes
belong inside the descriptive sentence, replacing the separate comparison summary below it.
For example: `Deal 20 » 43 damage to all enemies within 5 » 7yd of the player.`
Render the complete `20 » 43 damage` and `5 » 7yd` spans in green, including both values,
the arrow and units/stat wording. Show units only once, after the new value (for example,
`4 » 3.7 sec`, `5 » 7 yd`, `20 » 40%`). Keep surrounding prose in its normal description color;
do not color the whole sentence or duplicate the comparison in a footer.

Apply this presentation consistently to ability, passive, item and other selectable upgrade
cards in level-up/chest flows. Preserve next-rank previews for newly offered abilities/items,
using the existing `Rank X » Y`, `New spell` or `New talent` label beneath the name
for context. Do not repeat “Next rank” or “New rank” inside the description. Compare the same effective before/after values used by gameplay, including current
talents, rarity and equipment; retain truthful red replacement losses and neutral no-change
states. Use mixed-color text spans that wrap as one paragraph, not separate fixed-position
labels. Verify multiple changing values, unchanged values, units, long descriptions, and
wrapping at the required resolutions. Capture new inline previews for README milestones.

The following describes the previous implementation and evidence, which does **not** satisfy
the new inline requirement:

Level-up/chest cards show green before/after scaling values from the casting parameters;
new abilities and passives also preview their next rank. Jewelry compares the resulting
worn loadout, including red replacement losses and a neutral no-change state. Next-rank
hover text uses green as well. Validate every choice family at compact and wide sizes;
current live evidence covers level-up cards at 1440×900, with jewelry replacement covered
by rules tests. This does not change balance or imply missing item effects are implemented.

### Integration checkpoint — 2026-10-24

Workspace formatting and `cargo clippy --workspace --all-targets -- -D warnings` now pass.
The cleanup names repeated Bevy query/bridge types, limits renown accessors and the legacy
chest helper to tests, removes an unused chest-display alias, and corrects stale comments,
Copy usage, defaults, test initializers and test-module placement. Rune pickup rules retain
a narrowly scoped dead-code expectation because actual rune spawning remains incomplete.

The full gate used checkout-local `wow-data/Data` and stopped at the app suite: 4,294
passed, six failed, ten ignored. The empty-account mock-network test passes outside the
sandbox with loopback permission. Five failures reproduce in the exact compiled binary:

- `pipe_warm::menagerie`: `SweepUiMaterial` has no loading-cover warmup contributor.
- `run_mode`: Survivors hot-reload modules directly reference the dev feature boundary.
- `glue_strings`: local English AUTH_BANNED text is unchanged by Localize.
- `ui_script::minimap_tests`: time tooltip returns `9:07 PM`, expected `21:07`.
- `ui_script::surface_gate`: the loaded UI exposes `LOCALE_enUS`.

Fix the two Survivors integration failures; investigate the locale/reference assumptions
without weakening tests. The optional addon corpus is absent. The workspace gate is not
green; no-install, player-test and live server round-trip success are not established.
Reusing the exact test executable avoids a redundant package-only rebuild caused by different
workspace feature unification. The diagnostic package-only rebuild was canceled deliberately;
the original full workspace test process completed normally with failure status.

The checkout-local patch chain confirms the locale mismatch: GlueLocalization.lua has an
empty `Localize()` and FrameXML Localization.lua declares `LOCALE_enUS`, without setting
24-hour time. The current tests/reference explicitly assume enGB. Add coverage for both
locales while retaining the checks on actual overrides, tooltip refresh and unexpected globals.

The warmup and dev-boundary failures are now repaired: cooldown UI draws a tiny real
material node under the entry cover, and native instruments live in
`capture/survivors_tools` behind `DevProbesPlugin`. The rebuilt workspace test binaries
pass 69 Survivors tests, five pipeline-warm tests and five run-mode tests. Runtime layout
resources and packaged defaults remain in gameplay; the watchers and probes are dev-only.

Workspace/all-target Clippy with warnings denied and formatting pass. The native
`warcraft-survivors --no-default-features` check passes with five existing-site warnings:
four dev-only reexports and the unfulfilled rune dead-code expectation. Clean those up
without suppressing player warnings globally. Full player tests and a live shader-warmup
trace remain unverified; these focused checks do not establish a green full gate.

### Locale integration checkpoint — 2026-10-24

The three locale-dependent checks now pass against checkout-local enUS assets (15 focused
tests total). Glue parsing is compared with actual Lua execution of the installed scripts;
tooltip tests exercise both 12-hour and 24-hour modes. The reference adds only the verified
`LOCALE_enUS` symbol, with provenance and generator support; a negative test rejects an
invented locale global. The previous documentation-link gate passed. The full gate rerun
passed formatting, Clippy and the app unit suite, then failed `cover_input_wall`: the
portrait reads ordered `WindowEvent` input, and the browser bridge names
`EventLoopProxyWrapper` / `WinitUserEvent`. The latter sends only WakeUp; the portrait
consumes cursor/button events, so ensure loading-cover behavior is enforced before adding
its verdict. Later gate stages remain unverified.

### Loading-cover checkpoint — 2026-10-24

The portrait now clears its own ordered-event reader and resets held drag state while the
loading cover is active, before either drag or rotation-button handling. The mixed window
event stream stays available to OS close/focus/lifecycle readers. The cover audit explicitly
classifies this consumer rule and the browser’s WakeUp-only heartbeat. A regression test
checks discarded gestures, preservation of the close request and fresh dragging afterward.
The workspace test rebuild and nine focused checks pass, including the input audit.
Formatting and workspace Clippy pass. The full gate passed the cover audit, then stopped
at `launcher_shape`: the standalone example omitted `mlua-sys`. The example now includes
all three patches and uses the confirmed fork URLs consistently. Both launcher-shape tests
pass against the saved example. Continue the full gate; live interaction remains unverified.

### Class Codex correctness checkpoint — 2026-10-24

The visible legendary grid, cloak requirements and achievement snapshot now share
`ClassDef::ability_keys()`, a unique start/pool catalog. Counting recognized catalog entries
prevents duplicate or foreign names from inflating progress. Save parsing deduplicates
seen rows while retaining unknown legacy names for compatibility. The new reload test
exposed the legacy underscore/space encoding collision; recognized names are normalized
against the class catalog to restore canonical ability keys. Regression coverage
checks all nine classes before/after the final discovery, save reload and inflated input.
The 45 core and 71 Survivors app tests pass, as do formatting and workspace Clippy.
The full gate is still running. Shared jewelry
discovery, icons, details and the responsive Codex layout are still unfinished.

### Discovery and localization audit — 2026-10-24

Shared discovery must observe the presented `Offer`, not all random candidates. Currently
`banish_card` calls `trade_choices` and keeps only its first card, but `trade_choices` records
every generated legendary candidate in `Run.seen_legends`. Record only displayed jewelry,
persist it when offered (before selection/abandon), and keep equipment effects run-only.
Regression cases must cover skipped offers, banish replacement, unshown candidates,
cross-class shared discovery, duplicate offers and restart.

The full gate passed launcher validation and reached `reference_strings`. Its assumption
that each localization patch contains sentences is invalid for enUS. Base tables still
must contribute; an independent fixture now verifies parsing inside `Localize()` and
accepting an empty patch. The fixture passes. The real audit now exposes 29 Survivors
literals duplicating shipped keys across seven files: account, armory, armory/door,
appearance, hud, menu and armory/character. Resolve these from the installed string tables
without adding a blanket exemption.

The completed workspace `--no-fail-fast` run reports 7,917 passed, five failed and 18 ignored
across 143 suites. Four failing targets remain:

- `reference_strings`: the 29 duplicated Survivors literals above.
- `ui_script_wall`: 42 gameplay-classified files name UiScript, ceiling 40. The new
  glue-string oracle is test-only but lives in a gameplay file; separate test placement
  from genuine gameplay dependencies before changing the boundary.
- `web_assets` (two failures): absent `crates/wenilla-realm/templates/play.html` and a
  missing `#fs` control in `web/index.html`. Reconcile these with this fork’s actual web
  delivery surface rather than inventing an unused realm template.
- `world_api_wall`: 108 unpublished engine items, ceiling 107. Identify the new dependency
  and either remove it or publish the intentional API with its rationale.

The optional addon corpus is absent. The no-install, documentation, render-pass and player
stages of the full gate remain unverified for this checkpoint. This aggregate run establishes
all current workspace test failures, not a green release gate.

### Shared discovery checkpoint — 2026-10-24

Discovery now records the displayed hand in `cards::deal_cards`, once the card row exists,
and saves before a choice is required. Candidate generation no longer writes a run-level
legendary journal. Only presented legendary entries from the run class advance its Codex;
shared jewelry uses account-wide family/name keys in `jewel_seen` lines. Missing lines in
old saves mean undiscovered, and unknown future names are retained without counting them.
Returning to the menu retries any pending save. Equipment and run-only worn jewelry are
unchanged. Shared Codex counts now reflect saved discovery rather than catalog size.

Regression tests cover cross-class deduplication, invalid indices, save reload, unshown
candidates and unchanged class equipment. The workspace rebuild and 73 Survivors tests
pass, as does workspace Clippy. The native play build passed (3m05s). An isolated
1280×720 live run verified green new-spell, talent and chest upgrade previews; the reached
chest offered spells, so jewelry offering, skipping, banishing, abandonment and restart
still need live validation. The empty Codex correctly showed zero shared discoveries,
but its shared amulet row clips below the viewport at 720p. The reference icon grid,
selected-entry details and responsive layout remain open. Card screenshots are in
`docs/images/screenshots/2026-10-24-codex-discovery/`.


### Codex layout checkpoint — 2026-10-24

Replaced the draft page with a 74/26 catalog/detail split using the existing scrollable
Armory columns. Class labels sit beside wrapping spell rows and use catalog class colors;
all three shared categories and the account-wide discovered total/bar belong to the
scrollable catalog. Selection has its own state instead of reusing another tab's status
message. Shared detail text reports offered/unoffered state. Library check, app Clippy
and native build pass (1m08s). A live 720p check reached all shared categories and the
footer by scrolling, then selected Onyxia Tooth Pendant and verified its unseen details.
Selection currently resets catalog scrolling; preserve that offset in the next pass.
Jewelry artwork, amulet/spell badges,
selected-icon treatment and detailed effect descriptions remain open. This is a layout
checkpoint, not completion of the reference Codex.


### Codex browsing follow-up — 2026-10-24

Codex selection now updates a marked detail text node without rebuilding the catalog.
A live 720p check scrolled to the footer, selected Onyxia Tooth Pendant and then The Eye
of Hakkar, and verified both detail changes with the catalog/footer unmoved. The native
play build passed (1m56s). Workspace test compilation and all 73 Survivors tests pass.


### Shared jewelry artwork checkpoint — 2026-10-24

The three shared Codex categories now use icon cells backed by the existing ItemDisplayInfo
loader. The pinned item-template catalog resolves 38 item-to-display mappings; four
abbreviated labels keep their saved names and map to the full source names documented in
SURVIVORS-GEAR-VISUALS.md. Royal Seal variants share one display. Mark of the Champion's
two distinct displays remain unresolved and its selectable cell shows a question mark.
Icons dim according to saved discovery. Native build (1m44s), workspace test compilation
and 74 Survivors tests pass with WOW_DATA set to the checkout data chain, including
all 38 display/icon checks. A live 720p capture verifies the full grid and footer fit;
Onyxia Tooth Pendant and the unresolved Mark both open the correct details.
Legendary necklace art with spell badges, selected-cell indication and detailed effects
remain open, along with live jewelry offer/restart persistence validation.


### Codex effect audit — 2026-10-24

Selected entries now show green bonuses computed by equipping the item into an empty
`campaign::Worn`, using the same comparison formatter as chest cards. The heading makes
clear this is the item's standalone bonus; chest cards still compare the actual loadout
and expose replacement losses. Native build, workspace test compilation and 75 Survivors
tests pass; live 720p validation confirmed the ring damage/critical-chance values in green.

Mechanics discrepancy found at that checkpoint: `Worn::bonus` granted every legendary amulet global
+5% damage, +15 percentage points of critical damage and +1 direct-attack target,
regardless of the associated spell. This does not satisfy the intended one-spell
legendary enhancement. The following scope correction addresses this; Phase 6 remains incomplete.
Shared jewelry still uses draft index-based bonus tables and needs a deliberate content
pass, rather than importing vanilla stats or presenting the current draft as balanced.


### Legendary spell scope — 2026-10-24

Legendary bonuses now enter cast parameters only for their associated spell, leaving the
shared character stat sheet unchanged. Direct casts and lingering areas/totems use that
spell scope for critical damage; existing areas retain their damage snapshot, while their
critical bonus follows the currently worn amulet as before. Single-target direct attacks
can gain one target; area spells keep one area. Live damage tuning scales the bonus once.

Chest comparisons label the affected spell and show replacement losses in red. Codex
details show the same scoped bonuses and omit inapplicable extra targets. Bonus magnitudes
remain draft values; distinct spell-specific legendary designs and balance are still pending.

Validation: native play build and workspace test compilation pass; all 77 Survivors tests
and 45 core rules tests pass. Live 720p checks confirm Consecration omits extra targets and
Hammer of Wrath includes them; both captures are linked in the changelog. This does not
clear the previously recorded full-workspace acceptance failures.


### Warrior talent checkpoint — 2026-10-24

Twelve nodes now use explicit class-specific effects. Rank snapshots are class-catalog
indexed; re-snapshotting clears old training. Tests cover every supported rank, foreign
classes and spells, caps, duplicate IDs, resetting training, account spec switching,
save/reload, and the actual cast parameters for cooldown, damage and shout radius.
Native build, workspace test compilation, 79 Survivors tests and 47 core tests pass.
The configured 1.12.1 talent audit passes for all supported IDs, names and rank caps.
Live 720p Heroic Strike and Booming Voice details fit; the changelog links the capture.
Remaining class nodes and named milestones are still Phase 5 requirements.


### Arms milestone checkpoint — 2026-10-24

Deep Wounds, Sweeping Strikes and Mortal Strike now have their documented 10/20/30-point
behaviors. The hit handler applies the bleed from the original critical hit, and shared
DoT processing catches up due ticks without extending effects beyond expiry. Core tests
check thresholds and excluded damage sources; app tests check refresh/no stacking,
preserved tick timing, multiple frame sizes, expiration and cast/rank composition.
Native build, workspace test compilation, 82 Survivors tests and 48 core tests pass.
The Deep Wounds description fits in the live 720p sidebar; screenshot is in the changelog.
This is not proof of a full manual Arms encounter or the full Phase 5 acceptance criteria.


### Fury milestone checkpoint — 2026-10-24

All three Fury milestones now use the documented combat paths. Core tests cover thresholds,
class isolation, boss slowing and inactive-buff risk. App tests cover reflected cast damage,
healing caps, absorption, immunity, expiry and an unrelated reflection buff.
Native build, workspace test compilation, 83 Survivors tests and 49 core tests pass.
Live 720p validation confirms the Death Wish benefit and penalty remain readable; the
changelog links the screenshot. Full manual Fury combat validation remains outstanding.


### Protection milestone checkpoint — 2026-10-24

Last Stand, Concussion Blow and Shield Slam now use shared cast, hit and absorption paths.
Core tests cover thresholds, class isolation, boss duration and mixed Fury/Protection
allocation. App tests cover capped healing, extra-target cast parameters, shield refresh,
absorption and preservation of stronger defenses. Native build, workspace test compilation,
84 Survivors tests and 50 core tests pass. Live Shield Slam details fit at 720p; the changelog
links the screenshot. Full manual Protection combat validation remains outstanding.


### Warrior defensive talent checkpoint — 2026-10-24

Sixteen individual Warrior nodes now have specific effects. Improved Revenge, Improved
Shield Wall, Last Stand and Bloodthirst use existing reflected damage, duration and capped
cast-healing paths. Shared rank tests exercise each new node at every rank; an app test
checks cast parameters, milestone composition, healing caps and training reset. Native
build, workspace test compilation, 85 Survivors tests, 50 core tests and the talent-data
audit pass. Live 720p Improved Shield Wall details are captured in the changelog. Manual
combat acceptance and remaining Warrior nodes are still unfinished.


### Talent upgrade readability

Supported individual talents show their implemented effect in green with a current-to-next-
rank comparison. The comparison is explicitly the node's contribution, excluding other
nodes, milestones and equipment. Maximum rank shows its final contribution without an
next-rank comparison. Armor uses flat points; percentage effects use percentage units. Supported
milestone effects also use green text. Unknown nodes retain an explicit pending notice in
white. Names and rank headers remain white, and both talent layouts use the shared details
system. This extends the green upgrade language used by level and chest cards.

Validation: native build, workspace test compilation, 85 Survivors tests and 51 core tests
pass. Live 720p comparison and supported-to-pending color reset verified; screenshot linked
in the changelog and README. Maximum-rank formatting is covered by core tests.


### Crowd-control duration contract

Timed hit effects (slow, root, stun, fear and weakening) now gain 10% of their base duration
per ability rank after rank one, matching the existing duration growth of buffs and DoTs.
Spell-duration talent multipliers compose with that rank multiplier. The resulting effect
is captured in cast parameters and carried into direct/delayed hits and ground areas;
attached state markers use the same duration on direct hits. Strength, target count and area
locations do not change. Drain fractions and execute thresholds are not durations and stay
unchanged. Existing boss resistance is applied by the hit processor after this scaling.
Upgrade cards compare effect duration separately from an area/buff/DoT lifetime.

Improved Demoralizing Shout (audited ID 161, five ranks) replaces placeholder damage with
10% additional weakening duration per rank. At ability rank three and talent rank five,
its five-second base weakening lasts nine seconds. The separate Piercing Howl milestone
slow still uses its own fixed three-second duration. These coefficients are provisional.

Validation: native build, workspace test compilation, 86 Survivors tests, 51 core tests
and the local talent-data audit pass. The new app regression checks rank/talent composition,
hit/marker duration, unchanged strength/count/radius and non-timed effects. Live 720p talent
details verified; full combat timing and balance validation remain outstanding.


### Arena terrain survey checkpoint — 2026-10-24

`survivors-audit <Data> --terrain <map> <x> <y> <radius>` now samples ADT terrain support,
liquid depth, impassable chunks and estimated slope, including neighbouring tile loads.
Six local surveys narrowed the next world-validation target to Warsong's 60-yard midpoint
arena. Arathi's midpoint is wet/steep; Warsong's 150-yard disk includes holes and steep
terrain; shrinking Alterac's midpoint disk does not remove its steep samples. The detailed
[location evidence](SURVIVORS-LOCATIONS.md) records coordinates, counts, method and limits.
Five audit tests and Clippy pass. World traversal, source model identity, WMO/doodad
collision, camera behavior and encounter/reward implementation remain outstanding.


### Warsong obstacle checkpoint — 2026-10-24

The model audit now reports world-space hull bounds and center rays. Local data identifies
17 hull-bearing props and eight zero-hull props with origins inside the 60-yard candidate.
A center ray hits stump 596699 at 8.898 yards; its bounds approach much closer than its
origin. Use this fixture for runtime collision and route tests; terrain-only clearance is
insufficient. The location report records exact coordinates, evidence limits and the
instance preview instructions below. Six audit tests and Clippy pass.


### Instance rendering checkpoint — 2026-10-24

Added development-only `--world-preview` to the Survivors launcher, exposing the existing
engine viewer's map and coordinate overrides. Native build and live Warsong rendering
passed at 1280×720; the README and changelog link the captured milestone. Reproduction is
in the [location report](SURVIVORS-LOCATIONS.md#developer-instance-preview).
This corrects the earlier missing-map-override claim: `WOW_MAP` already existed.
Capsule contact/sliding, routing, tile reload and gameplay camera acceptance remain open.


### Placed obstacle contact checkpoint — 2026-10-24

Added a data-backed headless regression for Warsong stump 596699, using its actual ADT
placement and the player's capsule radius. The source ray and body sweep are checked before
removal, after removal and after re-registration. This tests the same `Floor::slide` used by
player movement; it does not substitute for streamed-world traversal or Elwynn reproduction.
See [fixture scope](SURVIVORS-LOCATIONS.md#placed-stump-capsule-regression).
Workspace test compilation and all 87 focused Survivors tests pass with explicit checkout data.


### Warrior bleed talents checkpoint — 2026-10-24

Nineteen Warrior nodes now have explicit effects. Deep Wounds adds four percentage points
per rank to the direct-critical bleed fraction, alongside the Arms 10-point milestone's
12%. Improved Rend multiplies that bleed damage by 1 + 10% per rank. At maximum ranks,
a 100-damage eligible critical with the milestone creates 31.2 damage over three seconds.
Improved Rend alone does not grant a bleed. These are explicit Survivors conversions;
no separate Rend ability is implied. The green rank preview uses the same coefficients.

Existing non-stacking refresh behavior and tick scheduling remain unchanged. Bleed ticks,
area pulses, reflected damage and other classes cannot trigger this bleed; execute and
multistrike expansion still happen after the triggering damage snapshot is selected.
Remaining Warrior nodes and full manual combat acceptance remain open.
Native build, workspace test compilation, 52 core tests, 88 focused Survivors tests and
the local talent identity/rank audit pass. Live 1280×720 validation confirms both new
descriptions and green next-rank comparisons fit the sidebar. The empty validation account
cannot spend points; its prerequisite/no-points messages are visible in the captures linked
from README and the changelog. These are layout checks, not evidence of combat acceptance.


### Warrior control talents checkpoint — 2026-10-24

Twenty-one Warrior nodes now have explicit effects. Piercing Howl adds two seconds to
Demoralizing Shout's secondary slow; Concussion Blow adds one second to Heroic Strike's
secondary stun. Each node can supply its control effect, and adds duration when its
same-name milestone is active (five-second slow / two-second stun). Boss slow remains
25% rather than 50%; boss stun lasts 30% of the ordinary duration. These secondary
durations are fixed contributions, independent of the shout's rank-scaled weakening.
The effects refresh existing control timers rather than adding durations on repeated hits.

Green node previews show seconds rather than percentages. Spec reset removes the nodes'
contributions; the milestones keep their own behavior. No extra AoE locations, free abilities
or change to shout damage weakening is introduced. Other talents and manual combat
acceptance remain unfinished.

Native build, workspace test compilation, 53 core tests, 88 focused Survivors tests and
the local talent-data audit pass. Both green previews fit at 1280×720 on an empty validation
spec. These checks do not establish full manual combat acceptance.


### Warrior absorb talents checkpoint — 2026-10-24

Twenty-four Warrior nodes now have effects. Shield Specialization grants Heroic Strike an
absorb equal to 1% maximum health per rank; Shield Slam adds 3%. Both add to the Protection
30-point milestone's 3%. All use a three-second base lifetime, extended by Improved Shield
Block's 0.5 seconds per rank. Duration alone does not grant a shield. At all three node caps
with the milestone, a 100-health hero receives 11 absorb for 4.5 seconds once per cast.

The existing shield refresh policy remains: no stacking, no multiplier from extra targets,
and no replacement or lifetime extension of a stronger active shield. These are Survivors
conversions rather than an equipment-dependent block system. Green previews distinguish
percentage of maximum health from added seconds. Remaining nodes, balancing and full
manual combat acceptance remain open.

Native build, workspace test compilation, 54 core tests, 89 focused Survivors tests and
the local talent-data audit pass. Fresh shield-description captures and manual combat
acceptance remain pending.


### Reactive Warrior talent checkpoint — 2026-10-24

Twenty-five Warrior nodes now have effects. Enrage triggers on actual health damage from
a nonlethal hit, after immunity, mitigation and absorbs. It multiplies damage by
1 + 4% per rank for four seconds; further qualifying hits refresh without stacking.
The runtime timer uses the same gameplay clock as the other timed buffs, starts empty
in a new run, and is not persisted. Its multiplier includes cast-local legendary damage
while preserving each legendary's spell scope. Existing projectiles/DoTs retain their
normal snapshot behavior. Green rank text names the conditional bonus explicitly.

This is a Survivors conversion: it does not require an enemy critical-hit system.
Immune hits, fully absorbed hits, lethal hits and untrained nodes do not activate it.
Combat tests cover partial absorption, refresh, expiry and the lethal boundary. Dedicated
buff artwork/indicator, manual combat feel and final balance remain pending.

Native build, workspace test compilation, 54 core tests, 90 focused Survivors tests and
the local talent-data audit pass. Live combat and fresh description captures remain pending.


### Front-door talent integration checkpoint — 2026-10-24

Live validation with an isolated Fury spec exposed an omitted talent snapshot in Armory's
`try_enter`: permanent stats were copied, while spell-specific ranks and milestones reverted
to defaults. The new front door uses this path. It now assigns the selected hero's active
spec immediately after run initialization, matching the existing menu/restart/auto paths.
A regression invokes the actual entry function with a selected second hero and second spec,
then switches to an untrained hero to check that old effects do not leak into the next run.
This finding means earlier unit and talent-page checks did not prove front-door combat
activation; full class combat acceptance remains open.

Enrage's runtime bonus and rounded-up remaining seconds appear in green under the combat
clock. The line clears on expiry, lack of the talent, or death, and follows the HUD's parent
layout/visibility. Dedicated buff artwork is still pending.

The 1280×720 live run now shows the trained shout radius/duration in green upgrade
comparisons and the active Enrage bonus/countdown. The [HUD capture](images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg)
uses an isolated checkout validation save with a legal 20-point Fury allocation.
White ability prose also now reads the cast's scaled effect instead of its raw definition,
so shout duration agrees with the green preview. A regression covers base, trained and
rank-two trained durations. This is targeted integration evidence, not full class acceptance.

Native build, workspace test compilation and 92 focused Survivors tests pass. The corrected
[shout card](images/screenshots/2026-10-24-warrior-talents/trained-shout-card-720.jpg)
was also verified live: white text says 7.5 seconds; green next-rank text says 7.5 → 8.2 seconds.
The hover glow still has a visibly rectangular outer boundary; card-effect visual acceptance
remains open. Existing broad-workspace failures remain tracked separately.

### Card lighting checkpoint — 2026-10-24

The rectangular spill was the animated outer `BoxShadow`, separate from the card-sized
landing sheen. Removed that shadow; hover now brightens the rounded border while preserving
lift/tilt. The internal rarity-lighting surface also has rounded corners.

Native build passes. Live captures show Epic, Rare and Uncommon cards together, without the
outer rectangle, at [1280×720](images/screenshots/2026-10-24-card-lighting/hover-720.jpg) and
after resizing the same open hand to [1280×800](images/screenshots/2026-10-24-card-lighting/hover-800.jpg).
The render log confirms a 2560×1600 framebuffer for the latter on this 2× display. The long
shout description and all green comparisons fit; selecting it after resizing adds its HUD
icon and advances to the next hand. The UI tool required rebinding the still-running window
after resize; the app was not restarted. These captures cover settled hover states, not
every frame of dealing/selection. Common/Legendary, other aspect ratios, non-default UI
scales and complete animation-cycle acceptance remain open.

### Inline card-description checkpoint — 2026-10-24

The shared level/chest card renderer now emits a single wrapped text paragraph with colored
spans. Ability changes live inside the description: damage, radius, count, duration and
cooldown use actual cast parameters; talent-scaled control durations use the cast effect.
Unchanged values retain normal prose color. Both values, the arrow and relevant units are
green when changed, with additional precision for small cooldown differences. New abilities
and passives show their rank-one/rank-two comparison without repeating the rank label
inside the description; the existing subtitle supplies that context. Passives and
health restoration use inline descriptions; jewelry retains red scoped replacement losses
and neutral duplicate/no-change notices within its paragraph. No separate summary is rendered.
Plain ability tooltips reuse the same prose implementation.

Native build passes. Live validation covers new spells/passives, Cleave rank-up, Whirlwind
damage/radius and Retaliation duration at 1280×720 and the same hand after live resize to
1280×800. See the [720p capture](images/screenshots/2026-10-24-inline-upgrades/upgrade-720.jpg)
and [resized capture](images/screenshots/2026-10-24-inline-upgrades/upgrade-800.jpg).
Fresh chest/jewelry/healthstone captures and the full resolution matrix remain unverified.

Workspace test compilation and 96 focused Survivors tests pass. New regressions verify
the mixed-color damage/radius spans, unchanged values, trained duration, cooldown precision,
passive/healing/absorb units, and red/neutral jewelry replacement semantics. This does not
clear the previously recorded broad-workspace gate failures.


### Warrior utility training checkpoint — 2026-10-24

Twenty-eight Warrior nodes now have explicit effects. Death Wish extends Retaliation by
25%, including the existing Fury milestone risk while that buff remains active. Improved
Disarm adds 10% weakening duration per rank to Demoralizing Shout; Improved Taunt reduces
its cooldown by 10% per rank. Duration bonuses multiply with existing duration training.
These are Survivors adaptations with provisional balance values, not vanilla spell claims.
The shout retains one area and its separate slow duration. Generic damage-per-point is
removed for these named effects through the shared stat adjustment. Remaining nodes,
fresh talent screenshots and manual combat acceptance remain open.

Validation: native build (22.5 seconds), workspace test compilation, 54 core tests,
97 focused Survivors tests and the local talent-data audit pass. The suite also caught
and corrected three stale Codex-page expectations for the single-unit comparison format.


### Inline text visual checkpoint — 2026-10-24

Live checkout validation at 1280×720, then resizing the same paused hand to 1280×800 and
1024×768, confirms readable single-unit comparisons and no prose rank labels. Selecting
Cleave at the narrow size grants rank two. The next hand verifies area, cooldown and
percentage comparisons at 1024×768. Screenshots are linked in README and the changelog.
This proves those sampled hands at default UI scale; it does not close chest/jewelry,
all-rarity, long-description or complete viewport/UI-scale acceptance. The isolated
validation save was used; the launch wrapper was restored and the app closed without
querying a stale app handle.


### Blood Craze and lethal-hit healing checkpoint — 2026-10-24

Blood Craze (local DBC node 661, three ranks) restores 1% maximum health per rank over
six seconds after a surviving health-damaging hit. Additional hits refresh one timer;
immunity and full absorbs cannot trigger it. This is an explicit Survivors adaptation
with provisional balance: it triggers on health damage rather than an enemy critical.
Healing uses the current maximum health and only the active portion of a frame, caps at
maximum health, and remains separate from cast healing. The timer follows gameplay time
and therefore pauses with the run. It is run-only, with no save-format change.

Regeneration now refuses to heal at zero health, preventing passive or timed healing from
undoing a lethal hit before the death check. Tests cover frame-size independence, refresh,
immunity/absorption, capped healing, lethal damage, talent caps, reset and class isolation.
Warrior has 29 supported nodes. Fresh talent/HUD captures, dedicated buff presentation,
manual combat acceptance and remaining class nodes are still open.

Validation: native build, workspace test compilation, 55 core tests, 98 focused Survivors
tests and the local talent-data audit pass. Full workspace gates were not rerun.


### Flurry training checkpoint — 2026-10-24

Flurry (local DBC node 156, five ranks) grants 5% haste per rank for three seconds after
a successful direct critical from Cleave, Heroic Strike, Execute or Mortal Strike.
Further eligible criticals refresh one timer; periodic areas, bleeds and reflection do
not trigger it. This is a provisional Survivors adaptation, not vanilla's charge model.
It uses the existing haste rule (percentage shorter cooldowns, additive with other haste
and subject to its floor); it affects subsequent cast cooldowns, not an already-running
cooldown. Gameplay pause freezes the timer. Expiry removes the bonus, and dead heroes
cannot activate it. It changes neither damage nor target count.

Warrior now has 30 supported nodes. The run-only training snapshot holds 64 entries so
continued class implementation can exceed the previous 32-node limit; saves still store
DBC IDs/ranks. A capacity regression checks every implemented class catalog. Remaining
nodes, dedicated Flurry buff presentation, fresh screenshots and manual combat checks
remain open.

Validation: native build, workspace test compilation, 56 core tests, 99 focused Survivors
tests and the local talent-data audit pass. Full workspace gates were not rerun.


### Reactive talent HUD checkpoint — 2026-10-24

The green status area beneath the clock now lists Enrage damage, Flurry haste and Blood
Craze healing rate with their remaining seconds. Multiple active talents occupy separate
lines, with no text for inactive, expired or untrained effects or a dead hero. Blood Craze
shows percent maximum health per second, matching its six-second healing calculation.
This extends the existing text HUD; dedicated buff icons/artwork remain open.

Native build, workspace test compilation and 100 focused Survivors tests pass. Live
1280×720 checks with an isolated 30-point Fury spec confirmed all three lines during a
level-up pause, Blood Craze alone during combat, Flurry alone during combat, and clearing
after expiry. README/changelog link gameplay captures of Flurry and the cleared HUD.
The overlay dims underlying HUD text as before. Larger/smaller viewport acceptance and
full manual combat validation remain open; the launch wrapper was restored after closing.


### Charge training and mobility clock checkpoint — 2026-10-24

Improved Charge adds 20% stun duration per rank; Improved Intercept reduces Charge's
cooldown by 10% per rank. Both have two ranks verified against local DBC identities.
At maximum training Charge stuns for 2.1 seconds and recharges in 6.4 seconds. These are
provisional Survivors adaptations. A resolved mobility definition supplies the actual cast,
HUD cooldown sweep and tooltip, preserving the 25-yard range, level-five unlock, collision
checks and no-target lunge. Other classes' mobility is unaffected by Warrior ranks.
Warrior now has 32 explicitly implemented nodes.

Mobility cooldown, speed and vanish timers previously advanced outside gameplay. They now
freeze on pause, level-up, menu, game over and loading screens; mobility cannot be cast
behind a loading screen. Tests cover the resolved training/reset/class boundaries and
all paused clock states. Fresh Charge tooltip captures and manual movement/combat checks
remain open.

Validation: native build, workspace test compilation, 56 core tests, 102 focused Survivors
tests and the local talent-data audit pass. Full workspace gates were not rerun.


### Lethal-hit boundary checkpoint — 2026-10-24

Living means positive health with no death transition in progress. Shared combat healing
now enforces that boundary for regeneration, spell/milestone heals, direct and periodic
lifesteal, pickups and healthstone choices. Healing remains capped and returns the actual
amount restored; nonpositive/nonfinite inputs grant nothing. Fresh-run health initialization
is separate. A lethal hero cannot start new automatic or mobility casts, collect gems/chests,
receive companion pickups, or accept a stale card choice before the death animation starts.
This prevents same-frame healing from reviving the hero and XP from opening a level-up
choice during the death transition. Already-launched attacks can still finish.

Regression coverage includes shared healing caps and lethal-hit rejection, cast healing/
shield rejection, and healing/XP/gold pickup rejection with no queued level-up. Manual
death/result-screen validation and full workspace gates remain open.

Validation: native build (33.9 seconds), workspace test compilation and 105 focused
Survivors tests pass. Full workspace gates were not rerun.

Live follow-up: death reached the results screen at 1280×720 on an isolated Night Elf
Warrior save. Screenshot: `images/screenshots/2026-10-24-defeat/results-720.jpg`.
Damage, XP and pickup multipliers were temporarily 0.1 to shorten the run, then restored
to 1.0. This is results-flow evidence, not balance or same-frame healing-race verification.
Replay follow-up at 1280×720: clicking Play again returned to hero selection; entering
Elwynn again reset the clock (captured at 00:02), full health, level-one Cleave and zero
damage. Evidence: `images/screenshots/2026-10-24-defeat/replay-720.jpg`. The same temporary
tuning was restored afterward. Singular reward text now uses “1 mark”; the older results
capture predates that text fix. Victory, keyboard replay and the results resolution matrix
remain open.

### Responsive results checkpoint — 2026-10-24

Results layout now uses a proportional stat-row width with a maximum readable width,
proportional tiles that wrap below their minimum readable size, constrained wrapping text,
and a proportional replay button with minimum/maximum bounds. These bounds operate within
the existing height-based UI scale rather than fixing the layout to one resolution.

Native build passed in 29.84 seconds. Live resizing of one defeat screen verified 1280×720,
640×720 and 480×720: summary text wraps and the smallest width places the five stat tiles
in centered rows of three and two, with both buttons visible. Captures live under
`images/screenshots/2026-10-24-results-layout/` and are linked in the README. This used the
isolated death save and temporary 0.1 damage/XP/pickup tuning, restored after closing.
The narrow checks cover results only; victory, larger counters and other interface pages
still need their own validation. Full workspace gates were not rerun for this layout change.

### Elwynn tree collision investigation — 2026-10-24

Source-hull audit at map 0, `[-9548.5, 115, 61]` found collision geometry on the nearby
thin-trunk and canopy-bearing tree variants (12 and 52 triangles respectively). The existing
read-only runtime probe queried twelve placement origins within 50 yards, after the streamed
world became presentable. Exact starts, directions and results are retained in
`validation/elwynn-tree-queries.txt`; reproduction points are the placement positions from
`cargo run -p survivors-audit -- "$WOW_DATA" --doodads 0 -9548.5 115 61`.

Placement 12578 (ElwynnTreeMid01) blocks all four capsule approaches before the requested
12 yards; the positive-X slide advances 5.569 yards and the straight mobility sweep 4.000.
Placement 12580 (ElwynnTreeCanopy03) permits all four origin-centered straight sweeps.
This rules out a blanket absence of tree colliders, but does not prove canopy trunk failure:
placement origins and broad source bounds do not establish exact solid occupancy, and some
approaches lack a valid starting floor. The next check must align a short repeated movement
path with a visible/source trunk surface, compare it against the registered hull, and repeat
after tile reload. No collision fix or navmesh requirement is claimed by these probes.

Surface-aligned follow-up adds 48 quarter-yard walking queries with the same floor-height
fallback as the hero, reporting lateral drift and missing floor samples. This is a static
world snapshot, not live input. Reproduce with `WOW_SURVIVORS_COLLISION_PROBE` pointing to
`docs/validation/elwynn-tree-surface-points.txt`; the points come from the source audit's
first triangle/plane intersections one yard above each placement origin. Evidence is in
`validation/elwynn-tree-surface-queries.txt`.

Canopy placement 12580 now has a concrete asymmetric case: from Bevy
`(-106.507454, 59.379368, 9520.363)` toward +X, repeated walking advances 11.992 yards
with one missing floor sample. Reverse approach advances only 5.483 yards, with no missing
floor samples. The -Z approach advances 5.802 yards with 0.435 yards lateral drift.
This narrows investigation to surface orientation/terrain-height interactions at that exact
location; it does not yet prove a rendered trunk penetration. Compare rendered placement
and per-step height against its hull before changing the shared one-sided collision policy.
Native build passed (10.61 seconds on the final probe revision); full gates were not rerun.

The authored-tree regression now also follows 96 quarter-yard grounded walking steps for
each of six tree variants from four directions. It checks that the capsule responds to
the hull and ray-checks every resulting center segment for solid-face crossings. Passing
the tree's center plane
is not itself penetration: the canopyless variant can slide around its trunk with over
three yards of lateral movement. This fixture uses level terrain and unrotated source
hulls, so it cannot settle the placed-tree 12580 terrain interaction or rendered alignment.

Validation: workspace test compilation and 105 focused Survivors tests pass, including
all 2,304 repeated-walking segments. Center-ray checks do not prove clearance of every
capsule edge or rendered surface; full workspace gates were not rerun.

Real-placement follow-up loads tree 12580 with its authored rotation/scale and all terrain
chunks in its tile. Both 48-step paths reproduce the runtime endpoints exactly, and each
resulting center segment passes the independent one-sided ray check. This rules out a
streamer-only cause for that particular asymmetry, but does not prove capsule-edge clearance.

The source audit now reports generalized winding at the requested point. At the passing
start's one-yard center, raw WoW `[-9520.363, 106.507454, 60.379368]`, tree 12580 yields
`0.760050`; the opposite start `[-9520.363, 94.507454, 60.64867]` yields `-0.066071`.
This supports an inside/open-hull start hypothesis, not an exact occupancy verdict: closed,
consistently wound hulls should yield magnitude 1 inside and 0 outside. The new audit test
covers a closed tetrahedron, reversal, exterior points and an intentionally missing face.
Do not turn this candidate into a two-sided collision policy change without rendered trunk
confirmation and an approach that demonstrably starts outside the solid hull.

Validation: workspace test compilation, 106 focused Survivors tests and audit-tool tests
pass. This fixture adds actual terrain/placement coverage; visual traversal and full gates
remain open.

### Live Elwynn trunk contact — 2026-10-24

From a fresh isolated Warrior run, W approached the large visible trunk north of the spawn.
The paused 00:08 approach and 00:18 contact captures retain the same trunk/ground framing
following further forward input, consistent with blocking. A subsequent D sidestep changes
that framing substantially at 00:24, showing movement away from the contact. Captures are
under `images/screenshots/2026-10-24-tree-contact/` and linked in the README. The pause overlay
partly covers the hero; these images are qualitative evidence, not measured travel distances
or a verified placement-ID match. This does not close collision acceptance for every tree,
angle, enemy/companion or tile reload.

The trunk visibly obscures the hero/combat, reinforcing the separate cutaway/silhouette
requirement. XP tuning was temporarily reduced to 0.1 after early level-up interruptions;
movement/damage stayed normal and all tuning/launch settings were restored after closing.
No gameplay or rendering code changed in this visual check.

### Cutaway implementation foundation — 2026-10-24

`survivors-core::visibility::Cutaway` defines the CPU reference for a projected circular
reveal between camera and hero focus. Radius is specified in yards at the focus plane and
scales with depth, with a smooth feather band and a focus-plane clearance. Off-axis scenery,
geometry behind the camera/hero, disabled and invalid inputs stay visible. Tests cover the
soft-edge midpoint, rotated/translated camera, depth scaling, hard edge and degenerate focus.
All 59 core tests passed at this foundation checkpoint; renderer integration follows below.

Renderer investigation found two load-bearing scenery paths: the retained `static_gx`
shader and the `WowModelMaterial` shader for ordinary model draws/fade variants. Integration
must cover both, keep actor/UI models out of the mask, and preserve alpha-tested foliage
and depth behavior. Supply the same per-frame camera/hero focus to both, without rebuilding
every material bind group each frame. Add persisted cutaway and ally/enemy silhouette
controls independently of camera auto-push, then validate the recorded tree-contact scene
and cave/interior cases. No collision changes are implied by this rendering work.

### Scenery cutaway renderer — 2026-10-25

The Survivors launcher enables the `survivors-visibility` feature across app, world and
assets. A shared 32-byte tail after the fixed-capacity skin palette supplies camera/focus
to both retained static scenery and ordinary/merged model shaders. A three-yard aperture
at the hero focus plane uses a 25% feather with ordered coverage; discarded fragments
leave depth clear for the hero and nearby combat. Actors, UI previews, ground clutter and
WMO architecture are excluded from this first foliage implementation. Physical colliders
are unchanged. The saved Scenery cutaway option is independent of Auto-push camera in;
old settings default to cutaway enabled. Menu/results clear the focus buffer.

Live validation at 1280×720: retained scenery and `WOW_STATIC_GX=0` ordinary model
rendering both reveal the hero and nearby enemies through an Elwynn canopy; toggling off
at the same paused camera restores the obstruction. Shader loading produced no errors.
Settings fit and retain cutaway On/auto-push Off across restart. The original near-side
trunk approach was mostly beyond the hero, so its unchanged appearance is not an occluded
trunk test. The canopy captures include alpha-cutout leaves and an opaque branch.

Evidence: [retained gameplay](images/screenshots/2026-10-25-cutaway/retained-gameplay.jpg),
[ordinary gameplay](images/screenshots/2026-10-25-cutaway/ordinary-gameplay.jpg), and each
path's on/off pair linked in the README. The captures use an isolated save, temporary
experience multiplier 0.1 and auto-push Off to isolate the visibility effect. Tuning and
launcher overrides were restored. No claim about balance or collision follows from these
captures. Native build, workspace test compilation, 106 Survivors tests, 563 world tests
(6 ignored), 103 asset tests and 59 core tests passed; final workspace Clippy, formatting, test compilation, the 106 Survivors tests and
documentation/render-pass lints also pass after test-style cleanup. Full workspace test
acceptance remains open (the previously recorded five failures were not re-audited here).

Still to validate: camera motion/zoom and minimum useful reveal size, streaming/culling
transitions, alternate aspect ratios and UI scales, opaque trunk occlusion, and all
material variants.
WMO/interior visibility, red/green depth-aware silhouettes and the full resolution matrix
remain required; this integration does not close those acceptance items.

### Viewport-scaled cutaway — 2026-10-25

Replaced the initial three-yard radius with a projected diameter of 35% of the shorter
viewport side. Radius follows the actual camera distance (including push-in), vertical FOV
and current aspect ratio, after camera/transform updates. Camera and cutaway share their
1.4-yard focus height. Invalid projection data disables the aperture.

All 61 core tests pass, including near/default/far distances, two FOVs, landscape/portrait
aspects and invalid projections. Native macOS build and workspace Clippy pass. This is
mathematical and compile validation: live zoom/resize evidence, visual size judgment and
the remaining visibility acceptance items above are still pending. Existing screenshots
show the preceding fixed-radius implementation and are not evidence for this sizing change.

Live follow-up: at 1280×720 in the retained renderer, toggling cutaway at the same paused
canopy view removes/restores the obstruction. After wheel zoom-out, the hero and nearby
wolves remain visible inside the centered aperture, with the feathered edge visible.
[Farther-zoom capture](images/screenshots/2026-10-25-cutaway-sizing/farther-zoom.jpg);
[on](images/screenshots/2026-10-25-cutaway-sizing/on.jpg) /
[off](images/screenshots/2026-10-25-cutaway-sizing/off.jpg).
No shader errors or panics appeared in the validation log. Exact camera distances were
not instrumented, so these captures do not prove endpoint coverage or the precise 35%
ratio. Resize, ordinary-renderer zoom and the wider matrix remain open. Isolated save,
XP multiplier 0.1 and auto-push Off were used; launcher and tuning overrides were restored.

### Warrior strike training — 2026-10-25

Improved Overpower (131, two ranks) adds 10 percentage points of Heroic Strike critical
chance per rank; the shared combat calculation caps total chance at 100%. Improved Slam
(168, five ranks) shortens Mortal Strike cooldown by 5% per rank, multiplying with ordinary
cooldown modifiers. These are explicit provisional Survivors adaptations: there is no
Overpower dodge trigger or Slam cast bar. Neither grants a spell or changes target counts.
Both replace their generic per-point damage fallback, follow the selected-spec snapshot,
and reset with that spec. Local DBC identity/rank auditing covers both; Warrior now has
34 implemented individual nodes. Remaining nodes and other classes are still required.

Validation: native macOS build, workspace test compilation, workspace Clippy, formatting,
62 core tests, 107 focused Survivors tests and local talent DBC audit pass. Tests include
reset/class isolation, the actual combat critical cap and unchanged damage/radius/targets.
Live talent-panel and combat validation remain pending; no new visual milestone is claimed.
Full workspace test acceptance remains open.

### Effective critical-hit descriptions — 2026-10-25

Ability tooltips display critical chance and critical damage from the same effective cast
parameters used in combat, including the 100% chance cap and minimum 100% damage multiplier.
This replaces the misleading fixed double-damage claim when Impale or equipment adds
critical damage. Critical-chance passive descriptions state only the chance they change;
the inline comparison remains green with a single percentage unit. Combat rules are unchanged.

Validation: native macOS build, workspace test compilation, workspace Clippy, formatting,
render-pass lint and all 12 focused HUD tests pass. Coverage includes effective critical
chance, baseline/boosted/minimum multipliers and green single-unit passive comparisons.
Live tooltip validation and the broader acceptance contract remain open.

### Un'Goro location investigation — 2026-10-25

[Location evidence](SURVIVORS-LOCATIONS.md#ungoro-arena-suitability-and-height-lookup-discrepancy--2026-10-25)
now rejects the configured summit footprint as a dry arena and records four crater-floor
alternatives. The southern candidate exposes 17 fast-height lookup misses despite emitted
mesh support. The audit reports these separately; runtime floors, map coordinates and
locks remain unchanged. Resolve the lookup discrepancy and inspect collider/obstacle and
rendered behavior before authoring the replacement. Encounters and all-class drop mapping
remain required before readiness.

### Terrain boundary recovery — 2026-10-25

The Survivors launcher and audit opt into `survivors-terrain`. The format adapter preserves
successful indexed height queries, then checks emitted triangles in nearby chunks only
when the fast lookup misses. The broad-phase margin only chooses chunks; exact triangle
containment determines support, and omitted hole triangles remain absent. The original
lookup stays available to builds without the feature. This also reaches the world stream's
existing height query through the shared format API; no colliders or authored maps change.

The local Un'Goro regression reproduces the old miss at `(-7100, -1625)`, recovers the mesh
height -271.66095, checks adjacent queries, and rejects an emptied/fully holed mesh and
outside-tile points. This is height-query correctness, not approval of an arena, route
connectivity, enemy spawn placement, or the zone's missing encounters and equipment drops.

After the fix, the southern Un'Goro survey resolves all 1,257 samples with no wet,
impassable or over-45-degree samples. Native macOS build, workspace test compilation,
workspace Clippy, formatting, render-pass lint, five terrain tests, 108 focused Survivors
tests and nine world-collision tests pass. The format crate also checks without default
features or `survivors-terrain`. These are scoped tests; full workspace test acceptance,
live movement/streaming and arena approval remain open.

### Un'Goro candidate visual follow-up — 2026-10-25

The southern location now has a README screenshot from the checkout world viewer and
source-obstacle measurements in [location evidence](SURVIVORS-LOCATIONS.md). Tree placement
304715 lies near the proposed center and supplies the next body-clearance/sliding fixture.
The preview verifies rendering only: no live player, horde, gameplay camera, pathfinding
or arena-readiness claim follows. Launch overrides were restored; authored coordinates,
locks and reward definitions remain unchanged.


### Hunter individual training — first nine nodes

The explicit checkout-data audit confirms the following native node identities and caps.
These are provisional Survivors effects, replacing each supported rank's temporary 1%
global damage bonus. They do not grant abilities; selected-spec reset removes them.

| Tree | Node | Ranks | Survivors effect per rank |
| --- | --- | --- | --- |
| Beast Mastery | 1395 Thick Hide | 3 | +8 hero armor; flat-armor conversion |
| Beast Mastery | 1384 Pathfinding | 2 | +4% hero movement speed; no aspect requirement |
| Marksmanship | 1341 Improved Concussive Shot | 5 | +10% slow duration, unchanged slow strength; conversion from stun chance |
| Marksmanship | 1345 Aimed Shot | 1 | +25% owned Aimed Shot damage |
| Marksmanship | 1346 Improved Arcane Shot | 5 | 5% shorter Arcane Shot cooldown |
| Marksmanship | 1348 Improved Serpent Sting | 5 | +2% direct and periodic Serpent Sting damage |
| Survival | 1311 Deflection | 5 | +1% damage reduction; no parry simulation |
| Survival | 1306 Clever Traps | 2 | +15% direct and periodic Explosive Trap damage |
| Survival | 1321 Killer Instinct | 3 | +1 percentage point critical chance |

The same definitions supply node descriptions and rank previews. Spell multipliers apply
through the existing combat parameter path, including periodic damage and control markers.
Multi-Shot/Volley targeting and all area locations remain unchanged. Tests cover rank caps,
duplicate saved ranks, reset, class isolation, removal of the generic damage baseline, and
rank-one/rank-three combat parameters. Hunter pets, remaining nodes and nine milestones
are still required; this is not complete Hunter acceptance. Visual talent-page validation
and final balancing remain open.

Validation for this checkpoint: 63 core tests, 110 Survivors app tests, native build,
workspace test compilation, explicit-data talent audit, Clippy, formatting and render-pass
lint pass. These checks do not prove live talent-page interaction or complete class balance.


### Hunter Marksmanship groups

Four more audited nodes bring Hunter to thirteen individual effects. Lethal Shots (1344,
five ranks) adds one percentage point of critical chance per rank; Mortal Shots (1349,
five ranks) adds six percentage points to the critical damage multiplier per rank.
Their explicit shot group is Arcane Shot, Multi-Shot, Serpent Sting, Concussive Shot and
Aimed Shot. These critical bonuses affect eligible direct hits, not periodic ticks.
They exclude Volley and Explosive Trap. Mortal Shots is a Survivors conversion: a
200% critical becomes 230% at five ranks, or 250% with another 20 points from equipment.

Ranged Weapon Specialization (1362, five ranks) adds 1% direct/periodic damage per rank
to that same group. Barrage (1347, three ranks) adds 4% damage per rank to Multi-Shot and
Volley. Their damage multipliers compose: maximum Multi-Shot training yields
`1.05 × 1.12 = 1.176` before other modifiers. Counts, ranges, radii and area locations
are unchanged. The selected-spec definitions also feed tooltip previews and remove the
temporary generic damage contribution for these ranks. Remaining Hunter work, including
pets, other nodes and all milestones, is still open.

Regressions cover every supported rank and affected/unaffected spell, class isolation,
reset, duplicate/capped saves, critical chance capped at 100%, equipment critical damage,
and unchanged area targeting. Live visual and full-class acceptance remain unverified.

Validation: native build, workspace test compilation, 64 core tests, 111 Survivors app
tests, explicit-data talent audit, Clippy, formatting and render-pass lint pass.


### Hunter talent visual and persistence checkpoint

The native build containing the thirteen individual Hunter effects was exercised with
an isolated, seeded 51-point account at 1280×720. Spec 1 started with 33 Marksmanship
points. Clicking capped Mortal Shots kept 5/5 and displayed its green description.
Right-click refunded one rank: 32 points spent, 19 unspent, and `24 to 30%` in the green
preview. Spec 2 started empty with 51 unspent; spending one Improved Concussive Shot
rank showed its slow-duration effect and left Spec 1 at 32. Resetting Spec 2 cleared only
that spec. Spec 3 remained empty throughout.

Returning to Spec 1 and live-resizing to 1024×720 kept all three trees, milestone rows,
spec controls, reset and the full description visible. Clicking Mortal Shots at the new
position restored 5/5 and 33 points. Closing and reopening restored that allocation and
the empty second spec, confirmed in the page and saved DBC-rank payload. Neither run
logged an ERROR or panic. The app was closed through its current window handle without
querying the exited app, and launch overrides were restored.

Evidence: [Mortal Shots preview](images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg),
[independent second spec](images/screenshots/2026-10-25-hunter-talents/spec-two-720.jpg),
[narrow layout](images/screenshots/2026-10-25-hunter-talents/mortal-shots-1024.jpg).
This validates these interactions and two dimensions, not the full resolution matrix,
all talent nodes, live combat balance or complete Hunter acceptance. The single-point
heading still reads `1 points`; copy polish remains open.


### Hunter Marksmanship milestones

Hunter Marksmanship now has explicit Survivors milestone behavior, additional to the
existing baseline tree bonuses (the same convention as Warrior and Paladin). Effects are
snapshotted from the selected hero/spec at run entry, including legacy tree point totals.

| Points | Milestone | Behavior |
| --- | --- | --- |
| 10 | Efficiency | Arcane Shot, Multi-Shot, Serpent Sting, Concussive Shot and Aimed Shot cooldowns multiplied by 0.9 |
| 20 | Mortal Shots | Those shots gain 20 percentage points of direct critical damage, added to equipment and individual training |
| 30 | Trueshot Aura | Multi-Shot gains two direct targets; Aimed Shot damage multiplied by 1.25 |

These are provisional survival-mode conversions. They do not grant unowned abilities.
Volley and traps receive none of these milestone effects; their area targeting remains
unchanged. Existing global baseline bonuses still apply through the normal stat rules.
Maximum Improved Arcane Shot composes with Efficiency as `0.75 × 0.9 = 0.675`.
Mortal Shots' five node ranks and its milestone add 50 percentage points together;
with a 220% equipment-adjusted critical multiplier, eligible direct hits reach 270%.
The Aimed Shot node and Trueshot Aura compose as `1.25 × 1.25 = 1.5625`.

Tests cover threshold boundaries, unsupported trees/spells, cast parameters with existing
extra targets, equipment critical damage, independent selected heroes/specs, persistence
and reset. Beast Mastery and Survival milestones, remaining nodes, pets, live milestone
UI/combat validation and final balance remain open.

Validation: 65 core tests, 113 Survivors app tests, native build, workspace test compilation,
Clippy, formatting and render-pass lint pass. No new live milestone screenshot is claimed.


### Hunter control training

Three audited nodes bring Hunter to sixteen individual effects. These are explicit,
deterministic Survivors conversions, not the original chance-based/disorient/sleep rules.

| Node | Ranks | Behavior |
| --- | --- | --- |
| 1353 Scatter Shot | 1 | Concussive Shot also stuns for 1 second; bosses for 0.3 seconds |
| 1304 Entrapment | 5 | Explosive Trap hits root for 0.4 seconds per rank; bosses receive 30% duration |
| 1325 Wyvern Sting | 1 | Serpent Sting also stuns for 1.5 seconds; bosses for 0.45 seconds |

The normal hit path applies these controls alongside the spell's existing damage, periodic
payload and slow. Existing longer stun/root timers win; refreshing does not add durations.
Periodic damage ticks do not invoke this control path. Area hit pulses can refresh their
root, but the talent creates no additional trap locations or targets. Boss control scaling
is applied once. Reset clears the trained effects for subsequent snapshots; unowned spells
are not granted. Individual node previews show seconds, and ability tooltips show ordinary
and boss durations (including 0.45 seconds without rounding it to 0.5).

Regression coverage includes source/class isolation, caps, reset, each Entrapment rank,
ordinary/boss enemy timer application, preservation of stronger existing controls/slow,
and tooltip duration agreement. Live control visuals, full-class balance, remaining
Hunter nodes, pets and six other milestones remain open.

Validation: 66 core tests, 115 Survivors app tests, native build, workspace test compilation,
explicit-data talent audit, Clippy, formatting and render-pass lint pass.


### Class-specific legendary visual identities — 2026-10-25

Atiesh now resolves each existing Priest/Mage/Warlock/Druid band-five main-hand cell to
its class quest reward and display. The [catalog evidence](SURVIVORS-GEAR-VISUALS.md#class-specific-atiesh)
records the pinned quest/item join. Name-only lookups remain ambiguous. Ownership, stats,
drops and zone locks are unchanged; this does not fill the 44 unnamed cells per class.
Live attachment validation and complete obtainable collection coverage remain open.

Validation: native play build (14.89 seconds), workspace test compilation, 67 core tests
and 115 Survivors app tests pass. The explicit checkout data verifies all four Atiesh
display rows and readable icon files. This is catalog evidence, not live attachment evidence.


### Uncommon armor identities — 2026-10-25

Added the eight dungeon-upgrade armor appearances for each class to band one (72 cells).
The [catalog](SURVIVORS-GEAR-VISUALS.md#uncommon-armor-catalog) records the authored
class/set assignments and pinned source. Names and rendering metadata share one identity
table; common gear, weapons, save indices, rarity stats and duplicate behavior are unchanged.
Each class now has 18 named cells and 36 unnamed cells. The new tier's weapon and all
new acquisition rows remain pending; item details explicitly show unavailable sources.
This is progress toward the full catalog, not completion of obtainable 54-piece collections.

Live evidence: Hunter's Beastmaster cap displays its icon, uncommon title, three stat
bonuses and unavailable-source message at [1280×720](images/screenshots/2026-10-25-uncommon-armor/hunter-720.png)
and [1024×720](images/screenshots/2026-10-25-uncommon-armor/hunter-1024.png). Ownership
remains 0/54 and no equip action is offered. At 1024 the Shoulders/Chest headings are
too close; spacing remains a responsive-layout follow-up. Other classes' live armor
attachments are not established by this collection-page check.

Validation: native incremental build (24.22 seconds), workspace test compilation, 67
core tests and 115 Survivors app tests pass. The slot-identity checks now cover both
armor tiers for all classes; the local-data icon/display scan traverses every mapped
collection cell, including the 72 additions. Formatting and render-pass lint pass.


### Collection slot-heading spacing — 2026-10-25

Resolved the narrow-grid heading issue found in the uncommon catalog check. Slot headings
now occupy their full proportional columns, use centered alignment and derive their text
size from the layout's body size. Live resize from 1024×720 to 1280×720 retains selection
and separates Shoulders/Chest; Main Hand wraps only where needed. Evidence:
[1024×720](images/screenshots/2026-10-25-collection-headings/collection-1024.png),
[1280×720](images/screenshots/2026-10-25-collection-headings/collection-720.png).
Native play build and formatting pass. This checks the default layout at these two sizes,
not the full resolution/layout-setting matrix or other Armory pages.


### Un'Goro scheduled armor sources — 2026-10-25

The six existing roster bosses now have timed uncommon helm-through-feet rolls, backed
by [pinned creature identity and placement evidence](SURVIVORS-LOCATIONS.md#ungoro-encounter-identity-and-source-table--2026-10-25).
All nine classes have named visual identities for those cells. The encounter clock and
source pages use the same rows; King Mosh arrives at 16:00 and victory requires all six
kills. The drop page now explains the selected zone's lock, even when a table exists.
Un'Goro remains world-locked: the current summit is unsuitable and the southern candidate
still needs live gameplay/camera/spawn acceptance. Wrist/hands/weapon sources remain open.

Live 1280×720 evidence verifies the [world-lock message and new sources](images/screenshots/2026-10-25-ungoro-drops/sources-720.png),
scroll access to the sixth row, and [King Mosh's matching boots detail](images/screenshots/2026-10-25-ungoro-drops/final-item-720.png).
Selection rebuilds the page and returns its source scroll to the top; preserving that
scroll position remains UI follow-up. A transient automation-pipe disconnect recovered
with the app still running and the requested selection applied. No live encounter or
loot-award acceptance is claimed from this menu check.

Validation: final native play build (29.08 seconds), workspace test compilation, 68 core
tests, 115 Survivors app tests with explicit checkout data, workspace Clippy, formatting
and render-pass lint pass. Core coverage checks every scheduled boundary, all-class name
and display resolution, capped Keen Eye chances, last-boss victory timing and the retained
world lock. No full workspace test pass or live loot-award completion is claimed.


### Un'Goro early combat evidence — 2026-10-25

The southern candidate now has [actual gameplay evidence](SURVIVORS-LOCATIONS.md#southern-candidate-gameplay-check--2026-10-25):
22 seconds, level 3, 55 kills, chest upgrade and three functioning Hunter abilities.
Moving away from the near-center tree opens the view. The minimap calls it The Marshlands,
so arena naming must be reconciled before adoption. Full traversal with position evidence,
prop/actor collision, camera occlusion, frame hitches and the complete encounter/reward run
remain acceptance work. Temporary coordinates and readiness overrides were restored;
Un'Goro remains locked. README now shows gameplay instead of only a free-camera preview.


### Distinct initial chest hands — 2026-10-25

The Un'Goro gameplay check showed Multi-Shot twice in one chest. Chest generation now
samples unused rewards within each rolled category (upgrade, ordinary jewelry, legendary).
An exhausted category adds no card, so one or two eligible spell upgrades produce a smaller
hand without promoting an unsuccessful jewelry roll. Empty upgrade pools still offer
ordinary jewelry. Legend identities retain the owned-ability index, including sparse lists.

Core regression coverage reproduces the two-spell hand and checks uniqueness, eligibility
and nonempty hands across zero, one, two and four eligible spells, repeated random picks,
low/high luck and category boundaries. This covers initial chest generation; banish
replacement can still independently select a card already present and needs the same
hand-wide exclusion contract. Live rendering of the reduced chest hand remains to verify.

Validation: native play build 53.14 seconds; workspace test compilation 3m 23s; 69 core
and 115 Survivors app tests pass. Core Clippy, scoped formatting, diff whitespace and
render-pass lint pass. No new live UI screenshot is claimed for this change.


### Banish hand exclusions — 2026-10-25

Level and chest replacement now exclude the entire original hand, including the selected
card. Successful replacements preserve card order and spend one charge. An exhausted
category keeps the original hand, charge and random state, disarms banish and cannot fall through into
normal selection on the same number key. This closes the replacement duplicate path
recorded above. Initial-hand and replacement draws share chest category thresholds;
exclusions cannot promote an upgrade roll into jewelry.

Regression cases cover varied level-hand seeds, preserving neighboring cards, one-card
chests with both successful jewelry rolls and exhausted upgrade rolls, invalid indexes,
fully capped level hands, and sparse owned-ability indexes for legendary exclusions.
Live keyboard/mouse validation and reduced-hand screenshots remain open.

Failed attempts restore the random state as well as keeping the charge, preventing repeated
free retries from fishing for jewelry on an unchanged hand; the chest regression checks it.

Final validation: native build 21.88 seconds, workspace test compilation 2m 09s, 70 core
tests, 118 Survivors app tests with checkout data, workspace Clippy, scoped formatting,
diff whitespace and render-pass lint pass. The tests include the final RNG restoration.


### Live distinct-hand and banish validation — 2026-10-25

Ran the committed behavior in Elwynn at 1280×720 with an isolated Hunter save and three
fixture banish charges. No world-readiness or combat-tuning overrides were used. The
[first hand](images/screenshots/2026-10-25-banish/before-720.png) offered Multi-Shot,
Detect Lesser Invisibility and Serpent Sting. B then 2 replaced only the middle card
with [Blessing of Kings](images/screenshots/2026-10-25-banish/after-720.png), preserving
its neighbors and decreasing charges from three to two without selecting anything.
Mouse selection of Kings resumed play and its passive icon appeared on the HUD.

Keeping Arcane Shot as the sole owned spell produced a centered, readable
[single-card chest](images/screenshots/2026-10-25-banish/single-chest-720.png) at 00:33.
B then 1 rolled [Pendant of the Qiraji Guardian](images/screenshots/2026-10-25-banish/chest-replacement-720.png),
spent the remaining charge and left the picker open. Clicking it advanced to the queued
[next level hand](images/screenshots/2026-10-25-banish/chest-selected-720.png), with no
banish hint. This live run sampled successful replacement, not the exhausted-roll case;
that failure path remains proven by regression tests only. Two-card layout and additional
screen sizes remain visual acceptance work.

The jewelry screenshot exposed a misleading spell-only chest subtitle. It now reads
“Choose your chest reward.” Captures above predate this wording change. The validation
app was closed via its window, the launcher restored, and the log contains no panic/error
lines. Fixture save and rewards remain isolated under target/survivors-validation/banish-state.


### Uncommon weapon identities — 2026-10-25

Added the [nine class weapon identities](SURVIVORS-GEAR-VISUALS.md#uncommon-weapon-catalog)
to the shared uncommon table. All names are unique in the pinned SQL, their class masks
permit the selected class, and entry/display/inventory/subclass/material/sheath fields
match the source. All nine slots of band one now have names and visuals for all classes:
19 named collection cells per class, 35 still unnamed. These are authored Survivors tier
choices, not claims of vanilla set membership or equivalent vanilla rarity.

Core coverage verifies class-specific weapon identities, weapon class/subclass and
inventory placement, completeness of all band-one slots, distinct common/legendary
weapons, and invalid-cell rejection. Existing app icon coverage enumerates every mapped
cell against checkout data. Weapon sources, live race/sex attachment rendering, the other
unnamed tiers and complete obtainability remain acceptance work. No ownership, save
layout, stats, drop chance or zone readiness changes are included.

Validation: native play build 11.76 seconds; workspace test compilation 59.93 seconds;
71 core tests and 118 Survivors app tests with explicit checkout data pass, including
mapped display/icon reads. Workspace Clippy, scoped formatting and render-pass lint pass.
No live attachment or collection screenshot is claimed for the new weapons yet.


### Complete uncommon source table — 2026-10-25

Added three named encounters to the existing Un'Goro schedule: Clutchmother Zavas at 4:30
(wrists), Gruff at 7:30 (hands), and U'cha at 15:00 (weapon). Pinned creature templates and
map-1 placements verify their identities, models and Un'Goro membership; the complete
[source table](SURVIVORS-LOCATIONS.md#ungoro-encounter-identity-and-source-table--2026-10-25)
records entries, displays, GUIDs and coordinates. Times, boss status, combat levels/scales,
60% base chances and slot assignments are authored Survivors tuning.

All nine uncommon cells per class now have a source. Scheduling and preview probabilities
still share the drop rows, and victory now waits for nine defeats and 16:00. Core coverage
checks every time boundary, all nine slots exactly once, name/visual resolution for all
classes and the retained world lock. Duplicate/no-extra-power rules are unchanged.
This completes table coverage, not actual zone obtainability: arena acceptance, live
boss/reward runs and tuning remain open. No new zone unlock or save migration is included.

Validation: native play build 21.83 seconds; workspace test compilation 1 minute 56 seconds;
71 core tests and 118 Survivors app tests with explicit checkout data pass. Workspace
Clippy, formatting, render-pass lint and diff whitespace checks pass. These checks do not
replace live nine-boss/reward validation or the outstanding full-workspace gate failures.

### Uncommon reward persistence — 2026-10-25

An account integration regression now replays every Un'Goro drop for every class. It
checks ten owned pieces (the starting common helmet plus nine uncommon rewards), keeps
the deliberately selected common helmet until explicitly changed, and verifies that
equipping the uncommon helmet increases damage while its common appearance remains.
It then saves/reloads an empty weapon slot and hidden gloves, repeats every reward, and
checks unchanged serialization, stats, class isolation and clear flags. This exercises
the real collection/equipment/save path without claiming live combat or zone readiness.

Validation: workspace test compilation and all 119 Survivors app tests with explicit
checkout data pass; workspace Clippy, formatting and render-pass lint pass. No runtime
behavior changed, so no new native play build or screenshot is claimed for this regression.

### Rare armor identities — 2026-10-25

The next eight armor cells per class now resolve to verified names and appearance metadata
from the pinned item templates. The [rare catalog](SURVIVORS-GEAR-VISUALS.md#rare-armor-catalog)
records nine sets and their class masks. This is an authored Survivors tier assignment,
not vanilla stat or raid-loot import. All 72 entries have distinct item IDs, correct armor
subclasses and slot/robe inventory types. Names and visuals share the same table.

Each class now has 27 of 54 named cells. Rare weapons, actual sources, live appearance
validation, the remaining bands and explicit raid-epic/set mechanics remain incomplete.
No ownership, save-slot mapping or world-readiness flag changes with this catalog addition.

Validation: native build 12.75 seconds, workspace test compilation 19.84 seconds; 72 core
tests and 119 app tests with explicit checkout data pass, including every mapped display
and readable icon. Workspace Clippy, formatting and render-pass lint pass. A separate
`rare-armor-state` fixture seeds 27 Warrior pieces and wears eight Rare armor pieces with
the common axe. At 1280×720, collection icons, Rare helmet stats and missing-source text
render without overlap; collection and human male preview captures are in the README.
This is not natural acquisition or complete model/attachment/race/sex acceptance. The
validation window was closed and its ordinary isolated launcher configuration restored.

### Rare weapon identities — 2026-10-25

Nine class-appropriate weapons complete the Rare tier's identities. The
[catalog](SURVIVORS-GEAR-VISUALS.md#rare-weapon-catalog) records exact pinned template
entries, display IDs, inventory types, subclasses, materials and sheath metadata. All
names resolve uniquely and class masks permit their assigned class. Survivors rarity,
stats and one-main-hand rules remain authoritative; vanilla raid loot, procs and weapon
skill/talent restrictions are not imported.

Each class has 28 of 54 named cells. All 81 Rare identities are distinct and have typed
slot metadata; the weapon choices differ from each class's common, uncommon and legendary
weapon. Acquisition sources, live attachments/poses and remaining content remain open.

Validation: native play build 12.86 seconds, workspace test compilation 20.38 seconds;
72 core tests and 119 app tests with explicit checkout data pass. Mapped display/icon
reads include all nine new weapons. Workspace Clippy, formatting and render-pass lint
pass. The README distinguishes the earlier eight-piece Rare screenshot from the current
nine-piece catalog; no new live weapon-rendering evidence is claimed.

### Feralas world and encounter foundation — 2026-10-25

Appended Feralas to the map registry, preserving every existing persisted map index.
Pinned templates/placements supply 15 enemy and nine boss identities; the source table
and terrain evidence are in [locations](SURVIVORS-LOCATIONS.md#feralas-provisional-arena-and-source-table--2026-10-25).
The 50-yard Snarler-area candidate passes sampled terrain support, water and slope checks.
This is provisional world geometry evidence, not complete arena acceptance.

The nine authored encounters provide every Rare slot at 60% base chance, from Snarler at
3:00 to Skarr the Unbreakable at 16:00. The drop UI and scheduler share these rows. Victory
requires all nine defeats; fully progressed heroes still receive the world-not-ready lock.
The roster uses the existing health/damage/XP ladder and provisional speed/scale tuning.
Live enemy behavior, camera/props, traversability, streaming, reward delivery and balance
remain incomplete. No campaign unlock or new save layout is included.

Validation: native build 20.49 seconds; workspace test compilation 2 minutes 28 seconds;
73 core tests and 119 Survivors app tests with explicit checkout data pass. Workspace
Clippy, formatting and render-pass lint pass. The rebuilt content audit resolves all
Feralas creature model files and agrees on its 55.83 start/terrain height, but exits with
476 overall gaps: five absent maps, two missing schedules, Naxxramas floor validation,
and 26 unnamed plus 26 unsourced cells per class. This is not a green full-content gate.

### Feralas rendered candidate and root collision — 2026-10-25

The initial 1280×720 free-camera view confirms a forest corridor with visible roots and
large nearby trunks/canopy. A screenshot is linked from the README. The doodad survey
identifies nearby root/tree hulls, including a large tree whose origin is outside the
arena but whose bounds overlap it. Details and exact placements are recorded in
[locations](SURVIVORS-LOCATIONS.md#feralas-candidate-prop-and-visual-inspection--2026-10-25).

The new placed-root regression loads actual ADT placement 245515 and its 60-triangle hull,
applies the real rotation/scale, and reproduces the negative-WoW-X source ray. Capsule
sweeps, explicit collider removal and re-registration test collision lifecycle in isolation
from terrain. This does not cover stream-driven registration or grounded traversal.
No readiness flag changed; the next acceptance work remains grounded actors, routes,
gameplay camera visibility and live Feralas fights/rewards.

Validation: workspace test compilation 2 minutes 6 seconds; all 120 Survivors app tests
with explicit checkout data pass, including the placed Feralas root. Workspace Clippy,
formatting, render-pass lint and diff whitespace checks pass. The preview used the prior
verified native build; this checkpoint changes tests and evidence, not runtime behavior.

### Grounded terrain and root traversal — 2026-10-25

Extending the Feralas placement test with its actual terrain exposed three movement
problems: horizontal sweeps stalled on rising ground, exact shared-edge floor rays
could miss, and post-sweep grounding could place the capsule inside a widening root.
Grounded walking now follows gentle slopes with rounded-base clearance, retries missed
floor rays one centimetre away, and rejects a final grounded pose that overlaps solid
geometry. Airborne walking retains its actual elevated sweep.

The 220-step Feralas regression checks emitted terrain height and root overlap at every
step, and requires at least 30 yards of progress before contact. The observed path ends
39.24 yards ahead of the start, beside placement 245515. Synthetic uphill/downhill
walking, wall sliding, and airborne clearance provide adjacent coverage. This is still
a controlled geometry test: stream-driven registration, live horde performance of the
additional overlap query, routes, gameplay camera behavior and encounters remain open.
Feralas remains locked.

Validation: all 121 Survivors app tests pass with explicit checkout data. Workspace test
compilation took 2 minutes 53 seconds; the incremental native play build took 49.93 seconds.
Workspace Clippy, formatting, render-pass lint and diff whitespace checks pass. No gameplay screenshot was
used as proof of capsule contact; the numeric geometry regression provides that evidence.

### Horde collision timing baseline — 2026-10-25

The Feralas geometry regression has an opt-in timing probe, using actual terrain and
root placement 245515 with 1, 60 and 220 bodies (the current late-run population target).
It excludes loading, rendering, enemy separation, other props and streaming. Bodies
advance 0.08 yards per simulated frame with radii 0.6/1.3; 60 warm-up frames precede
600 samples. This is an optimized test-profile measurement on the local arm64 Mac,
not a gameplay FPS result or a portable performance guarantee.

| Bodies | Movement median | Movement p95 | Movement max | Final-pose replay p95 | Advancing steps |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.008 ms | 0.054 ms | 0.056 ms | 0.005 ms | 414 / 600 |
| 60 | 0.312 ms | 2.267 ms | 2.345 ms | 0.234 ms | 24,687 / 36,000 |
| 220 | 4.241 ms | 9.800 ms | 10.041 ms | 0.825 ms | 77,564 / 132,000 |

Final-pose replay independently times floor lookup plus overlap checking at the resulting
positions; it is diagnostic, not an exact subtraction of time within the full movement
pass. A first run without replay also measured 9.779 ms p95 for 220 bodies. Some bodies
stop at geometry; advancing-step counts expose that limitation. The sweep/contact solver
is the next profiling target. A 9.8 ms collision-only p95 is not accepted as completion
of the horde performance requirement; full gameplay profiling and optimization remain open.

Reproduce from the checkout (keep the workspace feature graph):

```bash
WOW_DATA="$PWD/wow-data/Data" WOW_SURVIVORS_FLOOR_BENCH=1 \
  cargo test --workspace \
  survivors::floor::tests::placed_feralas_root_blocks_the_candidate_crossing_and_unregisters \
  -- --exact --nocapture
```

No timing threshold is asserted on shared CI hardware. The normal collision assertions
still run before the probe. All 121 Survivors app tests with checkout data and workspace
Clippy pass; the new probe is test-only and does not change the native executable.

### Survivors sweep-and-slide optimization — 2026-10-25

Survivors walking now resolves contact planes from successive body sweeps instead of
collecting all nearby contact faces after each hit. It retains the shared one-sided
body cast, 0.01-yard skin, four-pass limit, steep-wall flattening and final grounded
overlap guard. Blocking planes accumulate so a slide cannot undo an earlier wall.
This lives entirely in Survivors; upstream client movement is unchanged.

All 15 floor tests pass, including a new two-wall corner that must permit tangential
progress and reject penetration on repeated steps. Actual Elwynn canopy, Warsong stump,
Un'Goro tree and Feralas root cases remain covered. Feralas's grounded crossing ends
at Bevy `(135.10509, 56.806805, 4239.2686)`, with terrain/root checks at every step.

The same opt-in workload measured 220-body movement p50 **2.994 ms**, p95 **5.772 ms**,
max **5.861 ms**, versus baseline 4.241/9.800/10.041 ms. Advancing steps increased from
77,564 to 78,625 of 132,000; the improvement is not from immobilizing more bodies.
For 60 bodies, p95 fell from 2.267 to 1.332 ms. These are controlled test-profile
measurements, not live FPS evidence. Live horde/rendering/streaming profiles, broader
route coverage and Feralas gameplay acceptance remain open.

Validation: all 122 Survivors app tests with checkout data, workspace Clippy, formatting,
render-pass lint and diff whitespace checks pass. Native incremental build: 19.00 seconds;
workspace test compilation: 2 minutes 38 seconds. No live gameplay performance claim is made.

### Feralas live combat and partial reward persistence — 2026-10-25

Ran the native checkout build with `WOW_SURVIVORS_AUTO=Warrior:late`,
`WOW_SURVIVORS_MAP=Feralas`, explicit checkout `WOW_DATA`, and a new isolated
`BENILLA_HOME=target/survivors-validation/feralas-live-state`. The existing validation
autopilot can enter the locked candidate without changing player-facing readiness.
The logical window was 1280×720; the saved Retina screenshot includes the title bar.

The run began at simulated 600 seconds with max-rank Warrior abilities and reached
219 living enemies. The final logged sample reached level 45, 398 kills and 130,820
damage at simulated 624 seconds. Repeated level-ups paused combat, so this is not a
normal progression/balance run. The screenshot shows a late-run single Healthstone
card; its full-health wording (“Restores your health from 320 health”) needs cleanup.

After close, the isolated account file owns Warrior cells `(band 2, slots 0..3)` and
selects those four pieces: Helm, Pauldrons, Breastplate and Belt of Might. Other class
collections remain empty. This supplies live drop-to-save evidence for four Rare
slots; five slots, reload appearance checks and the final encounter remain unverified.

Two early hitches were logged at 1,830 and 1,006 ms. The captured stall sample includes
render/executor waits and does not establish their root cause. The old autopilot FPS
number divided all accumulated frames by ten, including menu/level-up frames, so no
combat FPS claim is made from that output. Validation telemetry now records actual
interval duration, separate combat frames/time/FPS, and worst combat-frame duration;
windows with no combat report `n/a`. Menu time is reset out of the sample.

The README links the captured milestone. The app was closed through its fresh close
control and the ordinary validation launcher restored. Feralas remains locked; live
traversal, occlusion, full encounters/rewards, hitch diagnosis and performance remain open.

A second isolated native run verified the new telemetry in real output: a 9.99-second
window reported 49 overall FPS, 328 combat frames over 7.38 real seconds (44.5 combat
FPS), and a 1,893.9 ms worst combat frame. This includes startup/asset work and is not a
steady-state benchmark. Both validation apps were closed, and the normal launcher restored.
Validation: all 124 Survivors app tests with checkout data pass; native incremental build
took 1 minute 32 seconds. Workspace Clippy, formatting, render-pass lint and diff
whitespace checks pass.

### Rare reward/save coverage and full-health card text — 2026-10-25

The all-class reward integration contract now runs against both Un'Goro's uncommon
and Feralas's Rare tables. Across all nine Rare slots for all nine classes, it verifies
collection, preservation of selected common gear, explicit rarity upgrades with stat
increase, an independently selected common appearance, explicit empty/hidden slots,
save/reload stability, duplicate replay without power changes, class isolation and no
unearned zone clears. This extends automated coverage beyond the four Warrior slots
observed live; it does not substitute for remaining live encounters or appearance checks.

Healthstone now says “Already at full health.” when no healing is possible, fixing the
awkward wording visible in the prior live screenshot. Damaged characters still see their
current-to-maximum health comparison in green with one unit suffix. Full, over-cap and
damaged-health text cases pass; the previous screenshot remains historical evidence.
All 126 Survivors app tests pass with explicit checkout data.
Native incremental build: 19.42 seconds; workspace test compilation: 2 minutes 39 seconds.
Workspace Clippy, formatting, render-pass lint and diff whitespace checks pass.

### Complete Epic identity tier — 2026-10-25

Added 72 armor identities and nine class-compatible weapons from the pinned item
templates, giving every class 37 named cells out of 54. The eight-piece sets are Wrath,
Judgement, Dragonstalker, Bloodfang, Transcendence, The Ten Storms, Netherwind, Nemesis
and Stormrage. Exact entries, display/attachment metadata and authored tier decisions
are documented in [gear visuals](SURVIVORS-GEAR-VISUALS.md#epic-identity-catalog).
Survivors retains ordinary Epic stats and numeric save indices; no ownership, source,
vanilla proc or set power is granted. Explicit raid-epic affixes/set builds remain open.

All 81 entries are distinct and class/slot compatible. All 74 core tests and 126 app
tests with checkout data pass, including display/icon resolution for the new items.
Native incremental build: 12.63 seconds; workspace test compilation: 22.73 seconds.
The full content audit still exits nonzero: 395 gaps, including 17 unnamed and 26
unsourced cells per class plus the existing world/schedule gaps.

A seeded human male Warrior menu renders the nine Epic names, bonuses and 37/54
collection count; the README links its screenshot (1280×720 logical Retina window).
Collection navigation could not be completed: CUA coordinate clicks repeatedly failed
with `noWindowsAvailable` although AX/screenshot reads worked and the app was running.
No collection-detail interaction or weapon pose acceptance is claimed. The app closed
through its fresh AX close control, and the normal validation launcher was restored.
Epic acquisition schedules, all-class appearance checks and remaining bands stay open.
Workspace Clippy, formatting, render-pass lint and diff whitespace checks pass.

### Legendary armor identity completion — 2026-10-25

Added the eight armor identities for each of nine classes from the pinned vanilla
Naxxramas sets: 72 distinct entries with verified class masks, armor subclasses, inventory
slots and display/attachment metadata. Source-set rings are excluded. The existing
Legendary weapons, including class-specific Atiesh variants, are unchanged. These are
authored Survivors rarity assignments; vanilla raid loot and set powers are not imported.
Exact set IDs and provenance are in [gear visuals](SURVIVORS-GEAR-VISUALS.md#legendary-armor-identity-catalog).

Every class now has 45 of 54 named cells; the nine PvP cells remain unnamed. No ownership,
drop source or zone readiness is granted. Acquisition schedules, live all-class/race/sex
armor and attachment checks, and explicit raid-epic/set-build mechanics remain unfinished.

Validation: all 75 core and 126 app tests with checkout data pass, including local reads
of the new display/icon assets. Native incremental build took 12.66 seconds; workspace
test compilation took 22.17 seconds. Formatting, render-pass lint and diff whitespace
checks pass. The full content audit still exits nonzero with 323 gaps: nine unnamed and
26 unsourced cells per class plus existing world/schedule gaps. No new live visual
acceptance is claimed; the README labels the older Epic screenshot's historical count.
Workspace Clippy also passes.

### Complete PvP identity tier and collection mapping — 2026-10-25

Added 81 PvP class/slot cells from 74 distinct pinned vanilla item identities: six-piece
rank sets, compatible waist/wrist pieces and nine weapons. All nine classes now have
54 named and mapped cells (486 total). Exact provenance and authored faction-independent
assignments are in [gear visuals](SURVIVORS-GEAR-VISUALS.md). Vanilla honor/reputation
requirements, stats and set powers are not imported. Numeric save slots remain stable;
this catalog grants neither ownership nor acquisition sources. Existing Warlock Common
and Uncommon Deathmist appearances remain shared; uniqueness is checked within each tier.

Validation: 76 core and 126 Survivors app tests pass, including every mapped display/icon
read using checkout data. Native incremental build took 12.81 seconds, workspace test
compilation 21.86 seconds and workspace Clippy 17.10 seconds. Formatting and render-pass
lint pass. Full content audit remains nonzero with 242 gaps: 26 unsourced cells per class
and eight world/schedule gaps. These checks do not prove live appearance correctness.

An isolated seeded Warrior fixture with 54 owned cells and PvP equipment reached the
menu loading-complete log. Screenshot/interaction acceptance is pending: CUA reports the
Mac locked and automatic unlock paused after physical input. No new image or visual
acceptance is claimed. The launcher was restored to its normal configuration for future
launches; the already-running fixture has not been closed through UI while locked.
All-class/race/sex appearance checks, acquisition schedules and raid-epic/set mechanics
remain required before completion.

### Gurubashi Epic sources and outdoor identity correction — 2026-10-25

Replaced Zul'Gurub raid identities in the Gurubashi Arena roster with 15 outdoor
Stranglethorn foes and nine verified map-0 bosses. Corrected Shadowmaw Panther's display
and the campaign's final-boss summary. Preserved tuning for retained foes and King
Bangalash; new encounters use provisional authored tuning. The pinned creature templates
and placement GUIDs are recorded in [locations](SURVIVORS-LOCATIONS.md#gurubashi-outdoor-roster-and-epic-schedule--2026-10-25).

Nine timed encounters now roll all Epic slots for each class, from Kurmokk at 3:00 to
King Mukla at 16:00. The shared drop table drives labels, rolls and combat scheduling.
The acceptance test covers exact time boundaries, exhausted schedules, final victory
requirements, all-class item resolution and the explicit world-readiness lock. The
all-class reward/save contract additionally covers Epic ownership, selected equipment,
independent appearances, duplicates, empty/hidden slots and isolation between classes.

Gurubashi remains unavailable to ordinary progression: its previously optimistic ready
flag is now false. A 709-point terrain survey supports the existing 30-yard disk at
z=21.857, but does not establish wall collision, navigation, camera behavior or live
encounter acceptance. No screenshot or live reward verification is claimed for this tier.

Validation: all 77 core and 127 Survivors app tests pass with explicit checkout data.
Native incremental build took 22.92 seconds; initial workspace test compilation took
2m46s, followed by 23.10 seconds after retained-creature tuning was preserved. Workspace
Clippy (17.16 seconds), formatting, render-pass lint and diff whitespace checks pass.
The content audit resolves all new roster model files and now reports 160 gaps: 17
unsourced cells per class, five missing maps, Ahn'Qiraj encounters and Naxxramas floor
validation. Audit source coverage includes locked zones and is not proof of obtainable
live progression; the zone readiness and full acceptance requirements remain separate.

### Gurubashi arena collision evidence and PvP screenshot — 2026-10-25

Added an explicit-data runtime fixture for the placed Gurubashi arena WMO and ADT.
Eight sampled body/camera directions hit the surrounding structure outside the 30-yard
play disk. Hero-radius and 1.3-yard bodies complete 64 grounded radial routes each,
120 quarter-yard steps per route, with supported height and no WMO overlap. The camera
helper shortens its boom to the sphere hit minus its safety margin. WMO removal clears
the checked elevated corridor. Exact placement, ranges and scope are recorded in
[locations](SURVIVORS-LOCATIONS.md#gurubashi-placed-wmo-boundary-evidence).

The fixture uses separate parsed walking/camera face sets and does not exercise live
streamer layer assignment, all wall headings, enemy routing, rendered cutaway or crowd
performance. Gurubashi remains locked. This checkpoint changes test coverage only;
no gameplay runtime change or new native build is needed for the fixture.

The previously blocked PvP menu capture is now saved under
`docs/images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png` and linked in README.
It shows seeded Warrior ownership of 54 cells and PvP names/bonuses, not earned drops
or full appearance acceptance. The still-running binary predates the Gurubashi schedule,
so its zone-lock wording is historical. The fixture closed through a fresh AX close
control with no subsequent query of the closed app. Normal launcher configuration had
already been restored in the prior checkpoint.

Validation: all 128 Survivors app tests with explicit checkout data pass. Workspace
test compilation took 1m57s for the final fixture; Clippy took 14.69 seconds and passes.
Formatting, render-pass lint and diff whitespace checks pass. Core gameplay files are
unchanged from the prior 77-test checkpoint; the overall goal remains incomplete.

### Spawn body clearance — 2026-10-25

Spawn placement previously checked the floor at the candidate center and four footprint
edges but did not test the body's occupied space. It now checks a capsule with the actual
spawn radius against body-collidable geometry after support passes, using the same slope
clearance as movement. Candidates overlapping wall edges, low ceilings or other registered
collision faces retry the remaining angular positions. When all candidates fail, existing
boss scheduling retains the pending encounter instead of consuming it; ordinary spawns
wait for a usable position.

Regression coverage checks small/large bodies near a wall, a low ceiling and its removal,
missing/mismatched floors, supported slopes, angular retry and fully occupied rings.
The placed Gurubashi fixture also checks clearance and spawn candidates from a hero near
the arena boundary across its 64 sampled headings for both tested body radii.

This uses registered collision geometry. It is not navigation, a guarantee of a connected
route to the player, or a volumetric containment test for an entire capsule enclosed by
a hollow triangle mesh without touching its faces. Live streaming, crowd-performance and
remaining zone acceptance still need validation. No readiness flag changes.

Validation: all 130 Survivors app tests pass with explicit checkout data, including the
real arena edge-spawn checks. Final workspace test compilation took 2m11s; native play
build took 22.03 seconds; workspace Clippy took 14.94 seconds and passes. Formatting,
render-pass lint and diff whitespace checks pass.

An isolated native Warrior `:late` run in Gurubashi starts at 600 simulated seconds with
all nine abilities at maximum rank. The captured results show 10:36, level 24, 409 kills,
150.2k damage and six bosses. Autopilot began a second attempt before the app was closed;
final save evidence spans both attempts. Warrior cells 27, 29 and 30 (Epic helm, chest,
waist) are owned and selected (`404400000`); all zone-clear flags remain zero. This
validates some actual reward persistence, not all nine encounters or normal acquisition.
Screenshot: `docs/images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png`.

The live log reaches 206 recorded enemies and has no ERROR/panic entries, but performance
is not accepted: observed hitches reach 1,882ms, and later combat samples fall to
roughly 7–9 FPS. The lowest reported combat sample is 4.3 FPS across only 2.10 combat
seconds/nine frames, including a 1,090.9ms hitch; this is not a steady-state benchmark.
Profiling must distinguish collision/spawning, rendering/effects and streaming before
attributing the regression to a subsystem. Normal launcher restored; app closed via
fresh AX control, without querying it after exit.

A separate UI issue was observed during this run: chest jewelry cards still use generic
spell-ID art (`jewel_card` in `hud.rs`), while the verified item-display mappings are only
used by the Codex page. The trinket card showed a question-mark icon. Wire chest cards to
the same item icon catalog and validate their live appearance in a subsequent checkpoint.
The “no change” bonus line may represent an equivalent replacement and is not by itself
proof of a missing effect.

### Chest item artwork and repeatable inspection — 2026-10-25

Resolved the chest-art issue observed in Gurubashi. Card descriptions now carry a typed
spell/item/missing icon source. Ordinary jewelry routes through the same 38 verified
item displays as the Codex instead of three unrelated generic spell IDs. Legendary
amulets retain their associated spell icon; ordinary ability/passive/Healthstone cards
retain spell art. Mark of the Champion remains explicitly unresolved, and invalid item
indices cannot borrow an unrelated image. No reward, stat, rarity or save change.

Added `WOW_SURVIVORS_AUTO=Warrior:chest` for repeatable UI inspection: it uses the existing
late-run setup but leaves chest choices to manual input while ordinary level-ups still
auto-pick. The workflow and isolation requirements are documented in development notes.

Live 1280×720 logical Retina checks show Don Mauricio's Band, Ring of Protection and
Onyxia Tooth Pendant with distinct item artwork. Selecting the ring updates the next
hand's bonuses. Further manual selection reaches Talisman of Ephemeral Power and
Zandalarian Hero Medallion with item artwork; the former no longer shows the question
mark observed in the prior run. Both milestone screenshots are under
`docs/images/screenshots/2026-10-25-chest-item-icons/` and linked in README. The run used
an isolated save and accelerated max-rank abilities, not normal acquisition. No ERROR or
panic entries appeared. Normal launcher restored and app closed through fresh AX state,
without querying the closed app.

Validation: all 131 Survivors app tests pass with explicit checkout data, including
item-vs-spell source routing, missing identities and all shared icon reads. Native build
took 1m19s; workspace Clippy took 48.96 seconds and passes. Final test compilation after
the inspection-mode addition took 2m20s. Formatting, render-pass lint and diff whitespace
checks pass. The full game goal remains incomplete, including Mark of the Champion,
remaining zone/appearance acceptance and the recorded Gurubashi performance problem.

### Unchanged minimap compositing — 2026-10-25

The held-chest Gurubashi probe reproduces slow frames without combat updates: 218 live
enemies remain frozen at 608 simulated seconds. Over 30 journal samples after 50 seconds
of wall time, median per-window mean frame time was 71.52 ms, process CPU time per frame
102.10 ms and main-thread CPU time 54.05 ms. These CPU counters are not GPU timing;
Metal reported no GPU spans. The earlier stall sample included minimap compositing,
but does not establish it as the principal bottleneck.

The Survivors minimap now retains its composite when map directory, zoom, horizontal
hero position, facing and ordered visible enemy positions are exactly unchanged.
Dead and out-of-range enemies do not enter that snapshot. Changed inputs still use the
same tile sampling, masking and ordered alpha blending; two reusable position buffers
avoid allocating on each moving frame. The snapshot is committed only after image data
is written. This removes redundant CPU rasterization and image uploads at a held chest
without throttling moving scenes. Live performance acceptance remains open.

Validation: native play build passed (1m57s initial, 20.81s final incremental rebuild);
all 131 Survivors app tests pass on the final source with checkout data. Workspace
Clippy, formatting and render-pass lint pass. A live 1440×900 logical Retina run shows
the held map and updated dot layout after selecting a chest reward and resuming combat.
No ERROR or panic entries appeared; validation app closed and normal launcher restored.

This is not a demonstrated FPS improvement. The second run held 224 enemies with a
different combat-text distribution, so it is not an identical-scene A/B benchmark.
After compilation finished, 15 journal samples from 118–135 seconds had median
per-window mean frame time 100.27 ms, process CPU 133.75 ms and main-thread CPU 80.51 ms.
A stall sample during the earlier held portion contains no minimap compositor samples,
but 174 samples in the combat-text SmartBucket overlap solver under float_combat_text.
Investigate repeated overlap placement of frozen combat text next; confirm timings
without concurrent compilation and test resumed combat as well. Logs and journals
remain in the ignored target/survivors-validation/minimap-before/after paths (the log
and CSV filenames use those stems). The full performance and content gates remain open.

### Combat-text placement reuse — 2026-10-25

The held-chest stall sample identifies SmartScreenRect placement beneath
float_combat_text as substantial repeated main-thread work. Survivors now enables a
separate survivors-text feature that wraps the stock claim bucket with an exact
placement-prefix cache. Each frame still lays out and draws the text. An identical
desired rectangle and viewport can reuse its prior solved seat only while all preceding
requests and claims match; the first mismatch sends the rest of the frame through the
stock solver. Shrinking crowds truncate stale suffixes. Changed caller claims are
recorded, so they invalidate dependent placements in the current and following frames.
No numbers, lifetime rules, sorting, color, movement, or overlap policy are removed.

The carry lives in survivors/text_placement.rs; the upstream combat-text system has only
a cfg-gated bucket parameter selection. Launchers without survivors-text keep the stock
bucket. Regression cases compare actual rectangles with SmartBucket across paused
frames, movement, viewport changes, reordering, removal/regrowth, and changed claims.

A focused optimized harness includes the actual SmartBucket and new wrapper source,
linked against this checkout's play-profile bevy_math. It seats 160 rectangles of
32×25 pixels in a 1440×900 viewport, centers at (720 + 3*(i%16), 450 + 3*(i/16)),
and asserts every cached result equals the stock result. Initial measurement:
10.411 ms cold, 0.001 ms on each of three identical cached frames, 10.752 ms for
a repeated stock frame. This isolates placement cost and is not an FPS benchmark.
The first live attempt overlapped compilation and suffered multi-second frames;
it is excluded from performance conclusions.

Final validation: 133 Survivors app tests, 12 combat-text tests and 10 stock solver tests
pass with explicit checkout data. Workspace Clippy, formatting, pass-span lint (1646
files) and diff whitespace checks pass. Native final incremental build: 36.18 seconds.
Final workspace test compilation: 3m35s including build-lock wait. Native macOS was
validated; other platforms and the full historical workspace gate failures remain open.
The focused harness repeated after compilation: 9.815 ms cold, 0.001 ms cached,
10.109 ms stock, with identical output.

The fresh isolated 1440×900 logical Retina Gurubashi probe started only after all builds
and tests finished. It held 159 enemies at level 7 with zero combat updates. Across 39
journal samples from 60–100 seconds, median per-window mean frame time was 60.23 ms,
p95 70.64 ms, process CPU/frame 89.58 ms and main-thread CPU/frame 45.08 ms. This is
still only about 16 FPS. Different crowds and damage-number distributions prevent an
identical-scene comparison with prior runs; do not claim an overall FPS speedup from it.

A fresh one-second CPU sample of that held scene contains no SmartBucket::resolve
samples and only six samples under float_combat_text (one in the cached resolver).
Rig-pose evaluation appears prominently (37 samples in the main-thread call tree),
alongside rendering and waiting; investigate the broader frame schedule next rather
than declaring animation the sole bottleneck. GPU timestamps remain unavailable.
The sample and journal are in target/survivors-validation/text-cache-idle-sample.txt
and text-cache-idle.csv.

The screenshot under docs/images/screenshots/2026-10-25-combat-text-cache/chest.png
shows preserved paused numbers. Selecting Band of the Unicorn resumes combat, raises
kills from 20 to 30, and produces new damage numbers and the next chest; the new ring
bonuses are reflected in subsequent comparisons. That short resumed combat window
measured 23.4 FPS over 1.54 seconds, with a 267.5 ms hitch, so combat acceptance remains
open too. No ERROR/panic entries appeared. Closed the app through fresh AX state,
did not query it after exit, and restored the normal launcher. README links the evidence.

### Foreground performance correction and focus telemetry — 2026-10-25

Revalidated the previous slowdown evidence before changing animation. Unattended runs
normally open behind the user's work through BgWinPlugin. Its documented macOS
occlusion throttling makes background FPS unsuitable as attended-performance evidence.
Earlier CPU observations still identify real redundant work removed by the caches, but
the reported 7–16 FPS runs must not be treated as normal foreground gameplay benchmarks.

With the same checked-in executable (4e49764), no compilation, WOW_BG=0 and
WOW_FRAME_PHASES=25, a held Gurubashi chest with 228 enemies stayed at about 90 FPS.
Across 40 journal samples at 40–80 seconds, median per-window mean frame time was
11.11 ms, p95 15.54 ms, process CPU/frame 21.73 ms and main-thread CPU/frame 9.02 ms.
The phase stamps perturb scheduling, so a second run omitted them and used Warrior:late
to continue combat. Six combat windows measured 74.5, 58.6, 55.2, 72.8, 64.4 and
45.0 FPS, spanning 1,380 frames and 20.81 seconds of combat across accelerated attempts.
Their longest recorded frame was 46.4 ms; one sample had 227 live enemies. Neither run
reported ERROR or panic. These are short, accelerated Warrior probes, not a normal
balance run, a twelve-zone acceptance pass, or a controlled identical-scene A/B test.
GPU timing was unavailable. Do not attribute the full FPS difference to either cache.

The automated log now reports total sampled frames, unfocused frames, and frames whose
window focus was unavailable, accumulated across the whole sample. Invalid/nonpositive
deltas do not increment any of these counts. A regression covers mixed focus and missing
window data while preserving the existing combat-only accounting. Focus alone does not
prove visibility or absence of OS throttling. The development guide now supplies an
explicit foreground, isolated-save profiling command and explains instrumentation limits.
No animation behavior was changed: the evidence does not justify that change yet.

Validation: all 134 Survivors app tests pass with checkout data, workspace Clippy passes
(15.47 seconds), and the native incremental build passes (1m03s). Formatting,
render-pass lint and diff whitespace checks pass. A final foreground live run on the
new binary reported 812 frames, zero unfocused frames and zero unknown-focus frames
over 9.99 seconds; its combat-only interval was 457 frames over 6.01 seconds (76.0 FPS).
This verifies the new fields reach the log, not broad performance acceptance. The
validation apps were closed through fresh AX controls without querying after exit,
and the normal launcher was restored. No ERROR/panic entries appeared.


### Ahn'Qiraj Legendary armor acquisition checkpoint — 2026-10-25

The outdoor Gates of Ahn'Qiraj catalog now uses 18 waves and eight named Silithus
bosses with identities verified against pinned creature templates and regional map-1
placements. Authored encounter timing supplies all eight Legendary armor slots for
all nine classes; Naxxramas retains the Legendary main hand. Setis is the final
encounter at 16:00. See [the source and tuning table](SURVIVORS-LOCATIONS.md#gates-of-ahnqiraj-outdoor-roster-and-legendary-armor--2026-10-25).
This replaces raid-interior creatures in the outdoor candidate, without claiming
vanilla loot or natural placement at the arena.

All 78 core and 135 Survivors app tests pass, including every class's reward acquisition,
duplicate neutrality, selected equipment, independent appearances and save reload.
Workspace test compilation, Clippy, formatting, the render-pass lint and the native
play build pass. The content audit reports 87 remaining gaps: 81 PvP source cells,
five missing map definitions and the Naxxramas start floor. Its scope does not prove
world readiness, full equipment appearance, talents or UI acceptance.

The isolated Warrior:late live probe entered the outdoor world with max-rank abilities
at simulated 10:00. The captured level-up hand is at 10:26, level 23 and 305 kills.
The saved account contains Legendary helm, shoulders and waist from the new encounters;
other classes remain untouched and no zone clear is recorded. No ERROR or panic was
logged. Loading/early combat hitches and an unknown creature sound kit 5519 were logged;
the mixed-focus run is not attended-performance or natural-balance evidence. The app
was closed and its normal launch wrapper restored.

The terrain survey supports all 5,025 points but finds 111 steep samples. This zone
remains locked pending traversal, camera, model-scale, full encounter and performance
acceptance. Screenshot: [level-up during the roster probe](images/screenshots/2026-10-25-aq-roster/level-up.png).


### Scarlet Monastery catalog and encounter checkpoint — 2026-10-25

Added map 189 using the Cathedral courtyard candidate, 18 wave identities and eight
bosses verified against the pinned map-189 creature placements. The roster combines
wings in a Survivors arena; arrival order, speeds and scales are authored tuning.
The exact identity table and floor limitations are in [location evidence](SURVIVORS-LOCATIONS.md#scarlet-monastery-candidate-and-roster--2026-10-25).
No collection drops are assigned to this zone. Campaigns without collection loot now
have an explicit boss schedule across 16 minutes, and victory waits for every boss
rather than treating an empty drop table as an empty encounter schedule. Existing
collection-drop schedules are preserved. Scarlet ends with Whitemane at 16:00.

All 79 core tests pass, including before/at spawn times, schedule exhaustion, final
kill/time victory boundaries and the locked campaign gate. Workspace Clippy, formatting,
render-pass lint and the native play build pass. The data audit finds all roster models;
its 87 remaining gaps now comprise four missing maps, two floor validations and 81 PvP
source cells. The aggregate count stays unchanged because adding Scarlet exposes its
WMO floor requirement, replacing its missing-map gap. The ADT floor at z=0 is not the
courtyard walking surface at z=18.3377.

An isolated Warrior:idle probe renders early combat on the courtyard floor. The
[screenshot](images/screenshots/2026-10-25-scarlet-candidate/combat.png) shows 00:14,
level 4 and 28 kills. It also shows enemies gathered against the side walls and a blank
minimap background. Navigation around these barriers, complete footprint/body/camera
validation, minimap imagery, all bosses, scale tuning and sustained performance remain
open. This brief stationary probe does not establish traversal or balance. The campaign
stays locked. The validation app was closed and its normal launcher restored.


### Indoor minimap composition checkpoint — 2026-10-25

Scarlet's blank minimap was reproduced in the courtyard. The explicit local
`md5translate.trs` contains Cathedral group textures; its outdoor MonasteryInstances
tiles instead reference the same uniform texture. The Survivors compositor previously
looked up only ADT tiles. Building-group composition now uses the engine's existing
portal flood selection, model placement, group ordering and authored tile grid. It
samples group tiles with a half-texel inset and alpha threshold before applying the
round mask, player arrow and enemy markers. Terrain composition remains the outdoor
path. Background caching is separate from marker changes and invalidates on map,
position, zoom, building transform, group or height changes.

All 140 Survivors tests pass, including rotated placement, alpha threshold, layer
order and the reused engine portal-selection regressions. Workspace test compilation,
Clippy, formatting, render-pass lint (1,647 files) and native play build pass. The final
native build took 58 seconds. The engine helper is compiled from its existing source
with a narrowly scoped duplicate-module lint allowance; no upstream source is changed.

The final live pass, after compilation stopped, verifies Cathedral textures, zoom-out,
pause/resume and changing enemy markers. [Scarlet screenshot](images/screenshots/2026-10-25-indoor-minimap/scarlet.png).
Short focused combat samples measured 61–73 FPS, but occasional one-second hitches
remain. The first concurrent-build probe was much slower and is not a benchmark.
An [Elwynn regression capture](images/screenshots/2026-10-25-indoor-minimap/elwynn.png)
shows terrain imagery and markers still render after the automatic hero moved. Its
camera is pushed very close to the character; that broader camera-obstruction issue
remains open and is not hidden by the minimap check. Both validation apps were closed
and the normal launcher restored.

These checks do not prove all indoor floors, building transitions, exact texture
alignment throughout rotated interiors, sustained performance or Scarlet traversal.
The existing Survivors zoom radius remains shared indoors/outdoors; matching separate
stock indoor zoom preferences is not claimed. No zone readiness gate was changed.

### Camera obstruction instrumentation — 2026-10-25

Added opt-in `WOW_SURVIVORS_CAMERA_TRACE=1`: one sample per real second records
the focus, requested eye, collision-limited eye and smoothed eye in raw WoW coordinates,
plus all three boom lengths. It leaves camera/collision behavior unchanged and does no
formatting or logging when disabled.

A fresh isolated moving Warrior run in Elwynn (map 0, default auto-push and cutaway on,
1440×900, `WOW_BG=0`, no concurrent build) produced
[107 samples](validation/elwynn-camera-trace.txt). All 98 samples with the requested
30-yard gameplay boom also had a 30-yard collision limit. Canopy/branch occlusion remained
visible during this run, but the previously captured close-camera state was not reproduced.
These one-second samples cannot exclude shorter transients or establish the cause of the
earlier screenshot. No tree collision fix is claimed.

Next: record the same trace during a controlled approach to an obscuring trunk, retaining
the exact focus/eye at any shortened boom, then compare its placed source hull and rendered
surface. Preserve cave/interior collision while deciding how tree obstruction and cutaway
should cooperate. Broad camera/navigation acceptance remains open.

Validation: native play build passed in 1m31s; workspace test compilation passed in
4m39s (including a build-lock wait), and all 140 Survivors app tests passed. Workspace
Clippy, formatting, render-pass lint and diff whitespace checks pass. No full workspace
test-pass or camera fix is claimed. The validation app was closed and normal launcher restored.

### Placed-tree camera regression — 2026-10-25

The camera fixture now loads Elwynn placement 12578 (ElwynnTreeMid01) from the explicit
checkout patch chain, applying its real position, rotation and scale. Raw WoW focus
`[-9517, 154.8, 61]`, a 61-degree elevation and a 30-yard boom toward negative WoW X
produce a 7.439-yard collision-limited arm. The resolved camera sphere must not overlap
the trunk; advancing it 0.15 yards must overlap. A parallel boom three yards away must
remain clear. Removing the hull restores the full arm; reattaching it restores the same
collision result.

This is a repeatable authored-geometry regression, not a live rendered-scene reproduction.
It excludes terrain, other placements, renderer alignment and actual tile streaming.
The first candidate at WoW Y=155.2 missed the tapered upper trunk; checking the authored
vertices corrected the fixture to Y=154.8. Do not interpret that miss as absent collision.
The earlier close-camera screenshot remains unexplained; its next live reproduction can
now be compared with this known source-hull contact.

Validation: all 141 Survivors app tests pass with the checkout data, including the
new overlap and reattachment assertions. Workspace test compilation, Clippy, formatting,
render-pass lint and diff whitespace checks pass. This checkpoint changes test code only;
the gameplay binary and camera behavior are unchanged.

### Mage individual talent effects — 2026-10-25

Verified these node names and rank caps against the checkout's Talent/Spell DBCs. These
explicit Survivors effects replace the temporary +1% damage-per-rank fallback for these
nodes only; existing tree bonuses remain. Talent acquisition does not automatically grant
a spell card.

| Node | Ranks | Effect per rank unless stated |
|---|---:|---|
| 76 Arcane Focus | 5 | Arcane Missiles/Explosion +2% damage; hit conversion |
| 80 Improved Arcane Missiles | 5 | Arcane Missiles +5% damage; pushback conversion |
| 1650 Magic Absorption | 5 | +1% damage reduction; resistance conversion |
| 81 Improved Arcane Explosion | 3 | Arcane Explosion +2% critical chance |
| 26 Improved Fireball | 5 | Fireball cooldown reduced 4%; cast-time conversion |
| 27 Improved Fire Blast | 3 | Fire Blast cooldown reduced 5% |
| 31 Improved Flamestrike | 3 | Flamestrike +5% critical chance |
| 29 Pyroblast | 1 | Pyroblast +20% damage |
| 33 Critical Mass | 3 | Fire spells +2% critical chance |
| 32 Blast Wave | 1 | Blast Wave +20% damage |
| 35 Fire Power | 5 | Fire spells +2% damage |
| 37 Improved Frostbolt | 5 | Frostbolt cooldown reduced 4%; cast-time conversion |
| 73 Ice Shards | 5 | Frost critical multiplier +0.10; explicit Survivors conversion |
| 62 Improved Frost Nova | 2 | Frost Nova cooldown reduced 10% |
| 61 Piercing Ice | 3 | Frost spells +2% damage |
| 64 Improved Cone of Cold | 3 | Cone of Cold +10% damage |

School membership is explicit: Arcane Missiles/Explosion; Fireball/Fire Blast/Flamestrike/
Pyroblast/Blast Wave; Frostbolt/Frost Nova/Blizzard/Cone of Cold. Cross-class and neutral
jewelry spells receive no school bonus. These talents do not add targets, area centers,
radius or control duration.

Ground-area ticks now combine their spell-scoped gear critical stats with the run's talent
snapshot; previously they ignored talent critical chance/multiplier even though cast
previews included them. This carries Flamestrike and Blizzard training into emitted hits.
UI talent descriptions and spell previews use the same definitions/effective parameters.

Remaining Mage nodes and tree milestones are still unfinished; this does not establish
complete Mage gameplay, balance or visual acceptance.

Validation: all 80 core and 144 Survivors app tests pass. Coverage includes school
composition, rank clamping, class isolation, selected character/spec save/refund,
effective cast/periodic damage, and actual area-hit messages for Flamestrike critical
chance and Blizzard critical multiplier. The DBC talent audit, workspace test compilation,
Clippy and native play build pass (native 35.77 seconds).

The native menu loaded the isolated Test Mage with five Fireball ranks. Talent-page
visual acceptance and a milestone screenshot remain pending: native coordinate input
repeatedly returned `noWindowsAvailable` while AX/window captures and the app log showed
the live menu. Raising/rebinding did not restore coordinate input. The app was closed via
its AX close button and the normal launcher restored. No live trained-combat or
multi-resolution claim is made for this checkpoint.

### Mage Ignite — 2026-10-25

DBC node 34 now has its five functional ranks. A critical Mage fire hit adds 8% of that
hit's pre-multistrike damage per rank to one Ignite burn on the enemy. Fireball splash
and critical Flamestrike area hits qualify; Frost, Arcane, neutral spells and Ignite
ticks do not. This is an explicit Survivors adaptation using the existing fire school list.

The burn pays over four one-second ticks. A fresh critical carries all unpaid scheduled
damage into the new four-tick pool, retaining the next tick deadline so repeated criticals
cannot postpone all damage. Existing unrelated DoTs retain their state; Ignite occupies
one slot under the existing six-DoT limit. Death cleanup and tick multistrike use the
existing damage-over-time path. Invalid/nonpositive additions leave the burn unchanged.

Talent rank previews and affected spell tooltips expose the percentage in green. Removing
the rank removes the effect/tooltip. Mage now has 17 functional individual nodes; remaining
nodes, milestones, balance and visual acceptance remain open.

Validation: 81 core and 146 Survivors app tests pass, including conservation of the unpaid
burn pool across a partial tick, unchanged tick phase, expiry, non-recursion, class/school
restrictions, capped ranks and the green trained-only spell tooltip. DBC audit, native
play build (22.12 seconds), workspace test compilation, Clippy, formatting, render-pass
lint and diff whitespace checks pass. A fresh CUA session again loaded the seeded Mage
menu but coordinate clicks returned `noWindowsAvailable`; no new talent-page screenshot
or live combat acceptance is claimed. Closed via AX and restored the normal launcher.


### Mage Frostbite implementation checkpoint

DBC node 38 now has three functional ranks: 5% freeze chance per rank on positive,
finite Mage frost damage. This Survivors conversion includes Frostbolt, Frost Nova,
Cone of Cold and Blizzard ground-area hits; unrelated spells and periodic burns do
not qualify. Successful procs root ordinary enemies for 5 seconds and bosses for
1.5 seconds, preserving any longer root. Damage does not break this root under the
existing Survivors control rules. The proc consumes randomness only on eligible hits.
Talent previews and affected spell tooltips show the trained chance in green.

Mage now has 18 functional individual nodes. Shatter's conditional critical bonus,
remaining nodes, distinct frozen-state semantics, balance and live visual acceptance
remain unfinished. Frostbite currently uses the existing root movement control.

Validation: 82 core and 147 Survivors app tests pass, including rank caps, class/source
restrictions, invalid damage, ordinary/boss root duration, longer-root preservation and
green trained-only tooltips. Workspace test compilation, explicit-data DBC talent audit,
Clippy, formatting, render-pass lint and diff checks pass. Native play build: 20.88 seconds.
No live combat or new screenshot evidence was collected for this checkpoint.


### Mage Shatter and frozen-state checkpoint

DBC node 67 has five functional ranks, each adding 10% critical chance to Mage spells
against frozen targets. Arcane, Fire and Frost spells qualify; jewelry, other classes
and periodic burns do not. Chance is capped at 100% after other critical bonuses.
Direct attacks read the target state when the cast rolls its damage; ground areas read
it on each tick. A spell that first applies a freeze does not retroactively gain the
bonus on that hit. Splash retains the originating hit's existing critical semantics.

Frost Nova and Frostbite now maintain a separate frozen timer alongside root movement
control. Both use the existing 30% boss duration. Longer ordinary roots do not extend
the freeze, and Hunter/other roots do not create it. Both timers decay during enemy
updates; damage still does not break roots under the current Survivors rules.
Green Shatter tooltips show the conditional chance on affected spells.

Mage now has 19 functional individual nodes. Remaining talents, milestones, balancing
and live visual acceptance remain open; this does not complete phase 5.

Validation: 83 core and 148 Survivors app tests pass. Deterministic actual-hit checks
cover Fireball, Flamestrike and Blizzard against frozen versus merely rooted enemies,
and Fireball after freezing ends while a longer root remains. Additional checks cover
rank caps, class/spell exclusions, boss freeze duration, longer-root preservation and
green trained-only tooltips. DBC talent audit, workspace test compilation, Clippy,
formatting, render-pass lint and diff checks pass. Native play build: 65 seconds.
Live combat and screenshot validation remain open.


### Mage frost slow talents checkpoint

Permafrost (DBC 65, three ranks) strengthens Frostbolt, Cone of Cold and Blizzard
slows by 3 percentage points and adds 1 second of slow duration per rank. Added
seconds apply after ability-rank scaling. Frost Nova/Frostbite freezes and non-frost
slows are unchanged. Improved Blizzard (DBC 63, three ranks) adds 10 percentage
points of slow per rank to the existing 40% baseline; this is an explicit Survivors
conversion rather than replacing the baseline with a talent requirement.

The two effects add together: maximum training makes level-one Blizzard slow by
79% for 4.5 seconds per tick. The existing boss cap remains 25% slow. Ground-area
lifetime, damage, radius, cooldown and number of locations are unchanged. Runtime
casts, area tick payloads, tooltips and cards use the same effective control values.
Inline comparisons now compare both slow strengths and show each unit once.
Mage now has 21 functional individual nodes; remaining talents, milestones, balance
and live visual acceptance remain incomplete.

Validation: 84 core and 150 Survivors app tests pass. Coverage includes rank caps,
combined control effects, exclusions, level-scaled duration, unchanged combat/area
parameters, actual Blizzard tick effects and green inline strength/duration comparisons.
DBC talent audit, workspace test compilation, Clippy, formatting, render-pass lint and
diff checks pass. Native play build: 13.85 seconds. No new live combat or screenshot
acceptance is claimed for this checkpoint.


### Mage UI verification and direct-close persistence repair

Native coordinate input worked again for this session. Verified the Mage talent page
at 1440×900 and 1280×720, refund/spend feedback for Shatter and Improved Blizzard,
Ignite in an independent second spec, and Frostbite/Permafrost previews. Six captures
are linked in the README under `docs/images/screenshots/2026-10-25-mage-talents/`.
Permafrost's preview now compares both strength and duration rather than leaving its
second benefit as a generic per-rank description.

The initial live run exposed a persistence gap: Permafrost was refunded from 3 to 2
on screen, but closing the OS window directly from Talents left rank 3 on disk.
Armory input now flushes dirty account edits at the end of its menu update; a clean
account does no disk I/O, and failed writes remain dirty for retry. This covers menu
edits without relying on a later run, page transition or explicit Done action.

Reproduction after the fix: refunded Permafrost to 2 at 720p, inspected the immediate
saved rank, closed via the window's close button, relaunched and reopened Talents.
Permafrost remained 2/3 with 25 Frost points; the independent Fire spec retained 10
points. `reloaded-spec-720.png` records that state. The isolated validation account
and launch wrapper were used; the normal wrapper was restored after closure.

Validation: 85 core and 150 app tests, workspace test compilation, Clippy, formatting,
render-pass lint and diff checks pass. Native build: 42.56 seconds. These are menu
layout/persistence checks, not combat, frame-time or full responsive-matrix acceptance.
Remaining class talents, other interface states and campaign/world acceptance remain open.


### Mage targeting range checkpoint

Flame Throwing (DBC 28, two ranks) gives targeted fire spells 10% more targeting
range per rank, an explicit Survivors conversion. Arctic Reach (DBC 741, two ranks)
gives Frostbolt and Blizzard 10% more targeting range per rank, and Frost Nova/Cone
of Cold 10% more radius per rank. The selected-spec cast snapshot now carries range
separately from radius; runtime selection and tooltip headers consume that same value.
Other classes and Arcane spells do not inherit these effects.

Fireball splash, Flamestrike area and Blizzard area retain their existing sizes.
Blizzard/Flamestrike still choose one location; extra-target statistics do not multiply
locations. Nova/Cone continue their existing area selection with the enlarged radius.
Mage now has 23 functional individual nodes; remaining nodes, milestones, balancing
and full live combat acceptance remain incomplete.

Validation: 86 core and 151 Survivors app tests pass. Actual cast-system tests place
enemies beyond untrained range, then verify trained Fireball/Frostbolt/Nova hits
and exactly one trained Flamestrike/Blizzard area of unchanged radius. Core checks
cover caps, reset, exclusions and target-count invariance; tooltip checks cover
trained fire/frost and unchanged Arcane ranges. Workspace test compilation, explicit-data
DBC talent audit, Clippy, formatting, render-pass lint and diff checks pass. Native
play build: 21.28 seconds. Live range-boundary and new screenshot acceptance remain open.

### Mage Impact and Incinerate checkpoint

Impact (DBC 30, five ranks) grants a 2% chance per rank on damaging Mage fire hits
to stun for one second, reduced to 0.3 seconds on bosses. This is an explicit
Survivors conversion. Direct, splash and ground-area fire hits share the hit path;
Ignite's periodic damage cannot trigger it. Longer existing stuns survive, and
stunning never marks the target frozen for Shatter.

Incinerate (DBC 1141, two ranks) adds 2% critical chance per rank to Fire Blast,
composing with Critical Mass and equipment. Scorch is absent from the current
ability pool, so this implementation does not claim a Scorch effect. The talent
description states the implemented scope. Impact's affected spell tooltips show
the trained proc chance in green. Mage now has 25 functional individual nodes;
remaining nodes, milestones and full live combat acceptance stay open.

Validation: 87 core and 152 Survivors app tests pass. Checks cover proc eligibility,
rank caps, invalid damage, normal/boss stun duration, preservation of longer stuns,
Fire Blast's combined critical chance at multiple ability levels, and green
spell-specific tooltips. Workspace test compilation, explicit-data DBC talent audit,
Clippy, formatting, render-pass lint and diff checks pass. Native play build:
19.48 seconds. These checks do not establish live proc frequency or combat feel;
new live combat/screenshot evidence remains open.

### Mage Winter's Chill checkpoint

Winter's Chill (DBC 68, five ranks) is now a per-enemy combat debuff. Each damaging
Mage Frost hit has a 20% chance per rank to add a stack, refreshing a 15-second
duration. Each stack adds two percentage points of Frost critical chance against
that enemy, capped at five stacks. This is an explicit Survivors conversion;
it applies to bosses without a control-duration reduction because it does not
immobilize them. Expiry clears all stacks; the next application starts at one.

Direct casts and ground-area ticks share target-dependent critical calculations.
Winter's Chill combines with Shatter and existing critical chance, capped at 100%;
it never makes an enemy frozen and never benefits Fire or Arcane damage. The hit
that adds a stack retains its already-resolved critical result. Other class hits,
zero/invalid damage and periodic Ignite damage cannot add stacks. New enemies
start unchilled, and stacks remain run-only state, outside the account save format.
Spell tooltips show the trained proc chance and the stack cap in green.

Mage now has 26 functional individual nodes. Remaining class talents, milestones,
balance, live combat feel and the broader seven-phase acceptance remain open.

Validation: 89 core and 153 Survivors app tests pass. Core checks cover rank caps,
spell/class exclusions, invalid damage, stack refresh/cap/expiry and Shatter
composition. Actual cast/area systems use matching random seeds to distinguish
Frostbolt and Blizzard criticals with and without stacks; expiry removes the
direct-cast bonus. Workspace test compilation, Clippy, DBC talent audit, formatting,
render-pass lint and diff checks pass. Native play build: 1m 35s.

Live 1280×720 validation spent Winter's Chill 4→5 and refunded 5→4, updating
Frost totals 30→31→30 and available points 21→20→21. The green final-rank
preview and maximum-rank text fit the side panel. Both screenshots are in the
README under the Mage milestone. The isolated account saved rank four; the app
was closed and the normal launch wrapper restored. This proves menu behavior
and layout, not live proc frequency, combat balance or the full resolution matrix.

The live menu log also records roughly one-second frame hitches during this
automated inspection. No panic/error was recorded and window closure completed,
but these captures are not performance acceptance. Investigate the saved local
stall samples before attributing the hitches to gameplay or UI automation.

### Menu presentation follow-up

[Controlled journal evidence](validation/mage-menu-presentation.md) associates
the one-second stalls seen in the Winter's Chill screenshots with capture-time
Metal drawable waits. Forty quiet journal rows had mean frame times
15.13–16.70 ms; three screenshot calls reproduced 994–1005 ms hitch warnings.
An earlier quiet interval also had a 359 ms hitch, so no general performance
clearance is claimed. Performance acceptance must use journal/combat telemetry
without simultaneous screenshot or accessibility inspection. No runtime change
was justified; gameplay, traversal and full frame-budget acceptance stay open.

### Mage Frost milestones and defensive conversions

The named Frost milestones now have spell-specific effects in addition to the
existing tree-stat rewards: 10 points (Improved Frostbolt) adds one Frostbolt
target; 20 (Ice Shards) adds 20 percentage points to Frost critical damage;
30 (Ice Barrier) grants a 10% maximum-health absorb for five seconds once per
successful Frost Nova cast. Area location count and radius remain unchanged.

Two individual DBC nodes now have explicit Survivors effects on the existing
spell pool. Ice Block (72, one rank) grants 0.5 seconds of immunity on a successful
Frost Nova; Ice Barrier (71, one rank, DBC prerequisite Ice Block) adds a 5%
maximum-health Nova absorb. The individual absorb lasts three seconds alone,
or five with the 30-point milestone. Together they absorb 15% maximum health.
These conversions do not claim standalone Ice Block/Barrier spells in the pool.
Mage now has 28 functional individual nodes and three functional Frost milestones.

Defenses apply once per successful cast, not per enemy, echo or extra target.
Failed target selection grants nothing; dead heroes cannot gain defenses. Absorbs
refresh without stacking and preserve stronger existing shields; immunity keeps
the longer remaining duration. The shared cast snapshot exposes these values to
inline cards, and spell tooltips now describe talent-granted shields and immunity
in green. Unit suffixes occur once per before/after comparison.

Validation: 90 core and 156 Survivors app tests pass. Boundary tests cover
9/10, 19/20 and 29/30 points, rank caps/reset and class/spell exclusions.
The actual cast system verifies out-of-range Nova grants no defenses and an
in-range Nova grants the combined absorb/immunity. Damage-path tests verify
immunity, subsequent absorption, non-stacking and stronger-defense preservation.
Tooltip and inline-span checks cover both benefits and one trailing unit.
Workspace test compilation, explicit-data DBC talent audit (including the Ice
Barrier prerequisite), Clippy, formatting, render-pass lint and diff checks pass.
Native play build: 15.23 seconds. Live defensive timing, new card wrapping/screenshots
and remaining Mage Arcane/Fire milestones still need acceptance.


### Frost Nova card wrapping evidence — 2026-10-25

The opt-in `WOW_SURVIVORS_AUTO=Mage:cards` inspection mode starts with the full
rank-one class pool and holds both level-up and chest offers for manual selection.
An isolated Mage account with 32 Frost points, including Ice Block and Ice Barrier,
reached a normal Frost Nova upgrade offer at level 12. The complete description
fits at 1280×720 and, with the same hand held, 1024×768. Damage, radius, freeze and
cooldown comparisons are green with one trailing unit; unchanged 15% shield,
5-second shield lifetime and 0.5-second immunity remain ordinary prose. All three
cards and the selection footer are visible. Selecting Nova after resize produces
`frost_nova:2` in the live run log and resumes play. Window close exits normally.

README captures: [720p](images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)
and [1024×768](images/screenshots/2026-10-25-mage-talents/frost-nova-card-1024.jpg).
The raw local log is `target/survivors-validation/nova-cards-live.log`; the normal
validation launcher was restored after exit. Clippy, native build (42.45 seconds),
formatting, render-pass lint and diff checks pass; seven existing inline-description
tests also pass. This closes this card's two-size wrapping check only. Live defense
timing, the full responsive matrix and remaining Mage implementation stay open.


### Battleground mode decision — 2026-10-25

User confirmed horde survival with faction-themed encounters for Warsong Gulch,
Arathi Basin and Alterac Valley. The earlier mode question is resolved. Author
map-appropriate waves and bosses, then verify placements, traversal, pacing and
the assigned reward tiers (Warsong Uncommon, Arathi Rare, Alterac PvP); do not
substitute spirit guides for combat bosses or
require capture-the-flag, resource-control or player-versus-player systems.


### Mage Arcane Instability and Elemental Precision — 2026-10-25

Arcane Instability (DBC 421, three ranks, Presence of Mind prerequisite) now grants
1% damage and one percentage point of critical chance per rank to the eleven Mage
spells. Its talent preview shows both contributions. Elemental Precision (1649,
three ranks) converts hit chance into 1% Fire/Frost damage per rank, consistent
with the existing Survivors hit-stat conversion. Arcane spells do not receive
Elemental Precision; other-class and neutral jewelry spells receive neither.
Both replace their temporary generic per-point damage. Effects compose with
school talents, affect direct and periodic spell damage, and leave area geometry,
target counts and cooldowns unchanged. Thirty Mage nodes now have explicit rules;
remaining nodes and Arcane/Fire milestones still require implementation.

Core coverage checks every Mage spell, rank caps, reset and foreign classes,
plus the dual-stat preview. App coverage checks direct/periodic damage, critical
chance and unchanged geometry at ranks 1, 3 and 8. Live talent-page and combat
acceptance for these two nodes remains open.

Validation: 91 core and 157 Survivors app tests pass, as do workspace test
compilation, Clippy, formatting, render-pass lint and diff checks. The explicit-data
DBC talent audit confirms both three-rank identities and Instability's Presence
of Mind prerequisite. Native incremental play build: 14.43 seconds. No new live
capture or full-workspace test-pass claim is made for this checkpoint.


### Warsong faction-survival content — 2026-10-25

Added the map-489 candidate at the previously surveyed 60-yard midpoint arena,
ten faction-themed wave archetypes, nine authored bosses and nine Uncommon slot
sources. The final Warsong Gulch Warmaster arrives at 16:00; victory waits for all
nine kills. Archetype template/display identities and the authored schedule are
recorded in [location evidence](SURVIVORS-LOCATIONS.md#warsong-faction-survival-roster--2026-10-25).
The shared source table drives combat schedules and the Armory drop page. All nine
classes resolve every reward, with duplicate Un'Goro sources sharing ownership.
Map indices remain stable because the destination was appended. The world-content
lock remains until full traversal, spawn/crowd, camera and live encounter checks pass.

Regression coverage checks each exact boss threshold, final victory, all slots,
class reward resolution and the retained lock. Account coverage collects/reloads
all nine classes, preserves selected gear and appearances, and checks cross-zone
duplicates cannot change saved state. Arathi, Alterac and Blackrock remain missing
map content; adding Warsong does not resolve the 81 PvP source-cell gaps.

Validation: 92 core and 158 Survivors app tests pass, including Armory lock text.
Workspace test compilation, Clippy, formatting, render-pass lint and diff checks
pass. Native incremental build: 22.42 seconds. Explicit-data content audit finds
Warsong terrain at z=309.30 and every roster model file. Its expected nonzero exit
now reports 86 gaps: three missing maps, two floor validations, and 81 PvP sources.

An isolated `Mage:all` run starts on map 489 and renders early faction enemies,
combat and the HUD at 1280×720. The README capture at game 00:32 shows level six
and 52 kills. It grants the full starting ability pool for validation and is not
normal progression or balance evidence. The log records loading/capture-associated
hitches and background frames; no performance clearance is claimed. Full boss
sequence, late crowd routing, attachments and camera acceptance remain open.
The app exited through its window close control and the validation launcher was
restored. Local log: `target/survivors-validation/warsong-roster-live.log`.


### Arathi faction-survival content — 2026-10-25

Terrain/floor and model-hull surveys found a dry, supported 40-yard candidate near
the stables, replacing the rejected map-art midpoint. The new map-529 definition
uses `[1210, 1130, -54.705]`. Ten faction wave entries and nine authored bosses
share the Rare source schedule, ending with Defiler Basin Warmaster at 16:00.
The [location evidence](SURVIVORS-LOCATIONS.md#arathi-stables-area-survival-candidate--2026-10-25)
records rejected candidates, measured support, obstacles, archetype provenance,
and every boss/source. Appended map ordering preserves saved indices. Normal entry
remains world-locked; survey support alone is not runtime traversal acceptance.

Tests cover all boss thresholds, finale/victory, slot/class resolution and the
retained lock. Account coverage checks all nine classes, selected equipment,
independent appearances, save reload, and Feralas duplicate-source idempotence.
No PvP-tier source cells are claimed by this Rare-tier map.

Validation: 93 core and 159 Survivors app tests pass, with workspace test
compilation, Clippy, formatting, render-pass lint and diff checks. Native play
build: 22.41 seconds. The explicit-data audit finds Arathi's floor at z=-55.71
and all roster model files. Its expected nonzero result is 85 gaps: two missing
maps, two floor validations and 81 PvP source cells. Full world acceptance is
broader than this data audit and remains required before changing `ready`.

Live evidence: an isolated `Mage:all` run rendered map 529, its faction enemies,
terrain, spells, HUD and minimap at 1280×720. The README capture shows 00:26;
the run continued through at least 163 game seconds before a normal window-close
exit. No ERROR or panic appeared in the local log. This artificial full-pool run
does not validate balance, the complete boss sequence, attachments or traversal.
Prominent rocks obscure parts of the arena; routing and camera/cutaway acceptance
remain open. The validation launcher was restored after closing, without querying
the closed app handle. Local log: `target/survivors-validation/arathi-roster-live.log`.


### Alterac faction-survival content — 2026-10-25

Added map 30's surveyed central 60-yard candidate, ten faction wave identities and
nine bosses ending with Drek'Thar at 16:00. Their PvP-band drops supply all nine
slots for all nine classes. Maps are appended to preserve saved indices; normal
entry remains world-locked. The [location record](SURVIVORS-LOCATIONS.md#alterac-central-survival-candidate--2026-10-25)
contains terrain comparisons, obstacle limits and template/display provenance.
This implements the confirmed survival direction, without PvP objectives.

Tests cover the boss thresholds, full schedule, victory timing, class/slot sources,
retained lock, equipment, independent appearances, duplicate ownership and save
reload. Live identity/attachment, complete-run, routing and camera evidence remain
required; roster model files alone do not establish those properties.

Validation: 94 core and 160 Survivors app tests pass, plus workspace test
compilation, Clippy, formatting, render-pass lint and diff checks. Native play
build: 23.26 seconds. The explicit-data audit finds all Alterac roster model files
and source coverage for all 54 collection cells per class; its expected nonzero
result is three gaps (Blackrock's absent map, Scarlet and Naxxramas floor checks).
This count excludes world-readiness and the broader acceptance requirements.

An isolated `Mage:all` preview loaded map 30 and rendered the Field of Strife,
early creatures, terrain, spell effects and HUD. README links the gameplay capture
at 00:16 and upgrade cards at 00:11, both 1280×720 logical resolution. The log
reached 28 game seconds before a normal window-close exit, without ERROR or panic.
This artificial full-pool run does not verify boss rendering, balance, full-run
routing, equipment attachments or camera obstruction. The normal launcher was
restored; no closed app handle was queried. Local log:
`target/survivors-validation/alterac-roster-live.log`.


### Blackrock floor and arena-shape investigation — 2026-10-25

Seven interior candidates fail to support an entire 30-yard circular arena.
The [floor evidence](SURVIVORS-LOCATIONS.md#blackrock-interior-floor-rejection--2026-10-25)
records actual source anchors, support counts and stacked-floor exclusions.
A fully supported boundary ring concealed interior gaps at Forgewright, so the
floor audit now reports missing-face versus other-storey samples and the nearest
unsupported sample. Shared triangle edges do not inflate the stacked-level count.

Blackrock remains unconfigured. Its next requirement is a connected, height-aware
playable region with safe spawn routes, body clearance and camera validation;
a circular leash and a valid center floor cannot establish that. This also informs
the existing indoor Scarlet/Naxxramas acceptance work. Eight audit tests and scoped
Clippy pass; formatting and diff checks pass. Both classified local-data probes
completed successfully. No native gameplay or navigation validation is claimed.


### Enemy recycling geometry contract — 2026-10-25

Routing investigation found that enemies more than 75 yards from the hero bypassed
safe-spawn checks: recycling directly assigned a point 38 yards ahead, falling back
to the hero's height when floor geometry was absent. This could place enemies
outside the arena or inside solid/unsupported geometry.

Recycling now uses the same bounded candidate selection as initial spawning:
arena/body margin, distance from the hero, supported footprint and body clearance.
If no candidate is safe, it retains the old position and retries on a later frame;
its movement animation is stopped while repositioning. This does not establish
connectivity to the hero. Obstacle routing and a height-aware Blackrock region
remain required.

The streamed-floor integration fixture now exercises recycling before floor load
and after load near the arena boundary, for ordinary and large enemy radii. It
checks deferral, body clearance, floor height and separation from the hero.

Validation: all 160 Survivors app tests pass, including the extended streamed-floor
fixture. Workspace test compilation, Clippy, native play build, formatting,
render-pass lint and diff checks pass. No live obstacle-routing acceptance is
claimed by this placement fix.


### Incremental obstacle detours — 2026-10-25

Stalled enemies now request local paths through the runtime collision geometry.
The planner searches a two-yard grid with separate half-yard height keys; each
edge must pass the existing grounded capsule travel and body-clearance checks,
and stay inside the arena. A final connection must reach the target's actual
height, so another storey is not accepted just because its XY matches.

Work is incremental: eight edge probes total per frame, round-robin across at
most 64 waiting requests, with at most 256 nodes per search. Movement continues
under the existing collision rules while a request waits. Failed searches retry
after a cooldown. Waypoints use grounded travel again during playback; a stale
path cannot bypass a newly streamed solid surface. Target movement beyond six
yards invalidates a route. Crowd control, fear, recycling, death/despawn and map
changes discard the relevant route state.

This is an initial local detour implementation, not full navigation acceptance.
Narrow passages below grid resolution, long indoor routes, crowded queue latency,
dynamic target behavior and real foliage/room geometry still need validation.
It does not unlock Blackrock, Scarlet or other pending zones. The Blackrock
playable-region and spawn-connectivity requirements remain open.


Validation: all 164 Survivors app tests pass. New tests verify incremental detours
around a finite wall, the global edge budget and queue rotation, dead/map/target
invalidation, missing support, rejection of a goal on another storey, and route
playback through real Avian capsule sweeps. The physics fixture confirms the direct
path is blocked, finds a detour, and checks body clearance while walking every
segment to the destination. Workspace test compilation, Clippy, formatting,
render-pass lint and diff checks pass. Native play build: 1 minute 18 seconds.
Live world navigation and performance acceptance are still pending.


### Visibility gating and direct page preview — 2026-10-25

The cutaway was previously applied whenever enabled. It now casts a camera/LOS
scenery ray from the actual camera to the player focus, stopping before the focus
plane; geometry behind or outside that sightline cannot activate it. Reveal strength
fades linearly in and out over 500 ms using real time. The existing ordered-coverage
shader interpolates scenery visibility while leaving colliders intact. This uses
scenery camera colliders; rendered foliage without corresponding LOS geometry still
requires visual coverage review.

Native development `WOW_SURVIVORS_UI_PAGE` opens a named starter/Armory page directly.
This uses production components, leaves progression intact and avoids playing a
session. A fresh isolated account opened Collection at 1440×900 logical resolution;
README includes the capture. This is direct page entry, not complete fixture or
hot-reload tooling for every page. Normal launcher settings were restored after exit.

Validation: 95 core tests, 165 app tests and one camera-layer physics regression
pass. The regression moves the same building collider behind, in front of and
beside the player sightline. Fade tests cover 500 ms timing and reversal without a
jump. Workspace test compilation, Clippy, formatting, render-pass lint and diff
checks pass; native play build took 2m 07s. The preview rendered successfully and
closed normally. The user's exact building scene and visual fade still need live
validation; the menu preview is not evidence for that behavior.

### Companion card checkpoint — 2026-10-25

Rewards now presents six companion upgrades/instincts as illustrated two-column cards.
Range, speed, rest and instinct previews derive from `companions::trip`, with green
current » next values and one unit. Purchases preserve account semantics and the
first-empty-slot instinct order independently of Looks selection. Live 1280×720 and
640×720 checks exercised Quick Paws, the second companion, Gold-sniffer then Wisp-herder;
inline milestone captures are in the changelog and README. Native build, Clippy,
formatting and render-pass lint pass. The reference's base Companion card, matching
icon choices, illustrated expanded Looks catalog and Confirm/Undo staging remain open.
Rune-hound labels the existing lack of rune spawning; no new gameplay is claimed.

### Companion appearance checkpoint — 2026-10-25

The existing five companion looks now render as wrapping illustrated tiles with a
selected border and green Worn label. Added the free base Companion description,
using trip-derived baseline range/rest/minimum sweep values, and reference card order.
Icon paths were verified against the checkout patch chain. Live 1280×720/640×720
checks selected first-slot Whelpling and second-slot Squirrel then Kingsnake without
changing instincts or points; restart preserved both slots. Inline screenshots are
in CHANGELOG and README. Native build, Clippy, formatting and render-pass lint pass.
Remaining: the expanded achievement-gated looks catalog, closer icon art/denser spacing,
Confirm/Undo purchases and broader resolution acceptance. No new pet models or gameplay
were introduced in this checkpoint.

### Companion catalog expansion — 2026-10-25

Append-only look indices preserve the original Rabbit/Squirrel/Chicken/Snake/Whelpling
saves. Added local DBC display mappings: Crimson Macaw 9554, Great Horned Owl 4615,
Tree Frog 6295, Sprite Darter 6294, Zergling 10993, Undercity Cockroach 2177.
Each catalog entry now owns its name, display and icon. Model and icon availability
are covered by a patch-chain test; this does not establish live animation acceptance.

Westfall Chicken follows the reference's Veteran requirement (achievement 65, 100 runs)
for new accounts. A trailing pets-record flag preserves unrestricted Chicken access
for existing seven-field saves, without granting achievement points or changing the
selected appearance. Account selection enforces the same gate displayed by the UI.
Panda Cub / Deathless Hour and Sapphiron / no u remain pending explicit achievement
requirements; the reference names these gates but the written rules do not define them.

The catalog check also exposed the old Black Whelpling display 629 as missing; it is
corrected to pet-sized black display 6288, retaining saved index 4. Live isolated
fixture checks verified Chicken rejects selection before Veteran and accepts it after
100 synthetic runs award the achievement, plus Macaw selection and 640×720 reflow.
Inline captures are in the changelog; in-world animation acceptance is still open.

Validation: 174 Survivors tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass.

### Rewards batch confirmation — 2026-10-25

Purchase methods mutate memory only; the Armory saves the validated account after
Confirm. The draft stores ordered reward codes and replays against the current
account, preserving intervening cosmetic/achievement changes. Failed validation
leaves both the live account and draft intact; there is no partial purchase.

Live isolated-account checks: two perks preview 275→245 without changing the save;
Undo restores both ranks/balance; Confirm writes the 30-point spend. Two Companion
upgrades preview 245→215 at 1280×720 and 640×720. Restart retains the confirmed perks,
245 points and two resets, discarding the pending Companion upgrades. Screenshots
are embedded in CHANGELOG and README. Tests cover reset preview/Undo, atomic stale
batch rejection, preserved intervening changes, insufficient points and duplicate
confirmation. 177 Survivors tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass. The subsequent per-card undo milestone
implemented minus controls for pending perk/companion purchases and resets, with
dependency-safe replay (see the 2026-10-25 changelog).

### Placeholder achievement gates — 2026-10-25

Per the user's clarification, Deathless Hour (54) and no u (55) are visible under
Feats with “Criteria pending”, no progress target, no points and an always-false
completion predicate. Saved IDs cannot turn placeholders into earned achievements
or selectable looks. Existing achievement IDs and look indices are unchanged.
Panda Cub (look 11, display 10990) and Sapphiron (look 12, display 16033) extend the
catalog; their tiles name the unavailable achievements. Icons and models resolve
from the local 1.12.1 patch chain. Sapphiron uses the FrostWurm boss display mapping;
companion-scale and animation validation are required before enabling its future gate.

Live checks verified both placeholder looks refuse selection, Feats lists both as
criteria pending, and all thirteen tiles fit after 640×720 reflow. Three inline
captures accompany the changelog. 97 core tests and 178 Survivors app tests pass,
as do the native build, workspace test compilation, Clippy, formatting and
render-pass lint. Achievement-page typography/layout remains a separate UI task.

### Rogue cooldown talents — 2026-10-25

Improved Slice and Dice (277, three ranks) reduces Blade Flurry cooldown by 5%
per rank. Elusiveness (247, two ranks) reduces Gouge/Kidney Shot cooldowns by 10%
per rank. Vigor (382, one rank) reduces the six Rogue spells’ cooldowns by 10%.
These explicitly labeled Survivors conversions replace provisional rank damage;
spell membership excludes other-class and neutral jewelry abilities. Different
reductions multiply with existing training, milestones and equipment.

111 core tests and a ranked cast-parameter regression pass, covering scope,
composition, rank clamping/reset and unchanged damage/control/targets. The local
DBC audit confirms identities and rank counts. Native 1280×720 Vigor refund and
repurchase and 640×720 Elusiveness refund/restore verify green text and readable
previews. Screenshots are inline in the changelog. Live combat feel, final balance
and remaining Rogue nodes are still open.

### Imported talent budgets — 2026-10-25

An imported spec with more points spent than the hero has earned keeps its saved
ranks for repair, but contributes no talent stat bonuses, trained spell effects or
milestones to a new run. Valid sibling specs still work. Refunding/resetting or
earning enough points restores eligibility. The talent page explains the inactive
state and dims milestone indicators. This enforces the earned-point budget only;
full catalog/row/prerequisite validation of imported ranks remains a separate gap.

The account regression covers save preservation, suspended bonuses and snapshots,
sibling-spec isolation, refund recovery, additional earned points and unchanged
existing run snapshots. Native 1280×720 evidence shows the warning and its removal
after refunding the excess Vigor rank; before/after captures are in the changelog.

All 202 Survivors app tests pass. Three older test fixtures now supply earned
points for their deliberately assigned ranks. Native build, workspace Clippy,
formatting and render-pass lint pass.

### Saved talent row normalization — 2026-10-25

Loading DBC specs now collapses repeated talent IDs within each spec to the highest
rank, retaining the first tree assignment. This matches core training's highest-rank
rule without multiplying tree points or milestone eligibility. Separate specs retain
independent ranks. Malformed fields, trees outside 0–2, zero IDs/ranks and extra fields
are discarded instead of coerced into a different tree. Canonical saves round-trip.
This does not establish class membership, catalog rank caps, row unlocks or prerequisites;
those checks remain open. No runtime UI or design change is included.

Validation: all 42 account tests, workspace Clippy, formatting and render-pass lint pass.

### Saved DBC catalog validation — 2026-10-25

Runtime specs now validate against the loaded class catalog before contributing
stat adjustments, spell training or milestones. Class/tree membership, maximum
ranks, earlier-row points and prerequisites share the purchase validator. Unknown
IDs, wrong trees and duplicate entries introduced outside the parser invalidate
the spec. Invalid ranks remain saved for refunds/reset; other specs are independent.
A pending catalog suspends DBC effects before menu/run consumers execute. Validation
is evaluated against current ranks, so refund/reset recovery does not need a restart.

The talent sidebar adds repair guidance and inactive milestones stay dim, preserving
the intentions' three-tree/sidebar structure. Native 1280×720 captures verify illegal
Vigor and recovery after refund. All 207 Survivors app tests, native build, workspace
Clippy, formatting and render-pass lint pass. This closes the DBC class/rank/row/
prerequisite gap recorded above. Legacy non-DBC spread rules, required-spell semantics
outside Survivors' talent prerequisites, and unfinished individual talent effects
are not expanded by this checkpoint.

### Priest individual spell training — 2026-10-25

Eight DBC-verified Priest nodes now have scoped spell effects rather than provisional
general damage: Improved Power Word: Shield, Holy Specialization, Divine Fury,
Searing Light, Improved Shadow Word: Pain, Improved Mind Blast, Improved Psychic
Scream and Darkness. The spec records exact initial values. Shield growth is labeled
absorption; conversion descriptions identify Survivors duration/cooldown rules.

112 core tests and the focused app regression pass. The app test covers ranks 1/3/8,
Holy Fire direct/burn damage, Pain duration, shield amount, critical chance and
cooldowns composed with gear; foreign spells, counts, radii and control effects stay
unchanged. Native 1280×720 Searing Light refund/repurchase and 640×720 Pain previews
are captured inline in the changelog, using legal saved specs and the intention's
three-tree/sidebar layout. Native build, workspace Clippy, formatting and render-pass
lint pass. Priest milestones, remaining nodes and live combat balance remain open.

### Priest tree milestones — 2026-10-25

All nine Priest 10/20/30 milestones now have explicit spell interactions and readable
Survivors conversion descriptions. Discipline links shield protection and Mind Blast;
Holy adds scoped critical chance and successful-cast healing; Shadow adds a Mind Blast
target, Pain cooldown advancement and a Psychic Scream absorb. Exact initial rules
are in the spec. Existing tree-point bonuses remain, as with other implemented classes.

113 core tests pass, covering exact thresholds, foreign-spell isolation, individual
rank composition and AoE target invariants. Two Priest app tests pass, including
actual cast-system execution with zero/three targets to verify once-per-cast healing,
cooldown advances, shielding and immunity. Native 1280×720 Lightwell and 640×720
Shadow Weaving descriptions are captured in the changelog from legal 30-point specs.
Native build, workspace Clippy, formatting and render-pass lint pass. Remaining
individual Priest talents, live combat balance and presentation refinement remain open.

### Spirit Tap recovery — 2026-10-25

The ninth individual Priest talent replaces provisional damage with a kill-triggered
health regeneration conversion: 20% proc chance per rank, 1% maximum health per
second for 15 seconds. Further procs refresh without stacking. Direct and periodic
kills share the trigger; dead enemies cannot trigger again. Active-buff text reports
remaining time. Removing training stops the healing benefit.

114 core tests and four focused engine tests pass, including failed rolls, refresh,
expiry across frame sizes, health caps and death. Native 1280×720 purchase/refund
confirms the green 80 to 100% preview; the changelog and README embed its capture.
Native build, workspace Clippy, formatting and render lint pass. Live combat
presentation/balance and remaining Priest nodes remain open.

### Priest casting range and area reach — 2026-10-25

Holy Reach (DBC 1635, two ranks) now adds 10% per rank to Smite/Holy Fire casting
range and Holy Nova radius. Shadow Reach (881, three ranks) adds one third of 20%
per rank to Shadow Word: Pain/Mind Blast casting range. Psychic Scream is unchanged.
Both replace provisional general damage. No added targets or duplicate AoE centers.

115 core tests pass, including all-rank scope and separate range/radius checks.
The engine parameter regression covers spell ranks 1/3/8 with gear haste/cooldown
and confirms target count/control remain unchanged. DBC audit, native build,
workspace Clippy, formatting and render lint pass. Native Holy purchase/refund
at 1280×720 and compact Shadow description/max-rank guard at 640×720 are captured
inline in the changelog. Live combat range acceptance and remaining nodes are open.

### Effective casting distance on cards — 2026-10-25

Card prose now includes targeting distance for bolts, strikes, melee attacks,
periodic targeted spells, chains and placed ground areas. Existing totem distance
text remains. Targeting distance and splash/area radius use separate effective
combat parameters; changed values are green with a single trailing unit.

29 HUD tests pass, including trained Priest range changes and unchanged-range
coloring. An isolated Priest:cards run in The Barrens showed Smite's trained
24-yard distance at 1280×720; selecting it advanced smite:1 to smite:2 and resumed
combat. The changelog/README embed the capture. Native build, workspace Clippy,
formatting and render lint pass. Other affected card layouts still need visual
coverage; this does not close live range-boundary or full responsive acceptance.

### Direct upgrade-card preview — 2026-10-25

The native development preview accepts `page=cards` and one to four explicit
`abilities=key:rank` entries. Rank zero means new ability; ranks 1–7 preview an
upgrade. It uses production card layout/animation/scrolling with baseline stats,
without waiting for combat or an offer. Live file edits resize or replace the hand.

The overlay stays in Menu, blocks card picks/menu gameplay starts/autoplay, and
skips discovery/reward writes. Returning to a regular page restores the original
run and offer. Invalid requests retain the last view; the initial request waits
for the UI to exist. Player/wasm builds do not expose the file-control entry point.

Five preview tests pass, including parser bounds and repeated-hand restoration.
Native 1280×720 and 640×720 captures show the same three-card hand and scrolling to
the third card. Clicking/number/Enter did not pick or start combat; an invalid key
kept the hand; returning to Talents succeeded. Account SHA-256 remained identical.
The changelog and README embed screenshots. Talent/gear-aware card fixtures and
non-ability reward previews remain future extensions of this development tool.

Native build, workspace Clippy, formatting and render-pass lint pass for this checkpoint.

### Long-description card acceptance — 2026-10-25

Rechecked `docs/intentions/survivors_level_up_selection_screen.png` for the framed
card hierarchy and green upgrade emphasis. Using the production preview, Blizzard
7→8, Chain Lightning 6→7 and Shadow Word: Pain 5→6 fit at 1280×720 and 640×720.
The compact third card wraps below the first row and is fully reachable by scrolling;
the footer stays visible. Casting distance, area/jump radius, duration, damage and
cooldown comparisons remain readable inside the card borders.

The exact baseline-stat inputs are committed as `docs/ui-previews/long-cards-1280.txt`
and `long-cards-640.txt`, with repeat instructions in the development guide. Three
native screenshots are embedded in the changelog and the README links the presets.
No engine code changed, so no rebuild was needed. This closes these three baseline
card layouts at these two sizes only; extra talent text, other sizes and other
card families retain their existing acceptance gaps.

### Force of Will — 2026-10-25

The twelfth individual Priest talent, DBC 1202 (five ranks), replaces provisional
general damage with +1% Priest Holy/Shadow damage and +1 percentage point direct
critical chance per rank. Shadow Word: Pain gains tick damage only; periodic ticks
retain their existing non-critical execution. Shield absorption, foreign-class and
jewelry spells remain unchanged. Damage multiplies with other scoped talents;
critical chance adds and remains bounded by combat's existing cap.

116 core tests pass, including all-rank scope, reset and combined training checks.
The ranked engine regression passes at spell ranks 1/3/8. DBC audit, native build,
workspace Clippy, formatting and render lint pass. Native 1280×720 purchase/refund
shows the green 4→5% damage and critical preview, embedded in changelog/README.
Live combat balancing and remaining Priest talents remain open.


### Talent healing on cards — 2026-10-25

The shared effective spell description includes cast healing for cards and tooltips,
with the maximum-health percentage and once-per-successful-cast limit. The old
separate tooltip line is removed. Changed healing follows green inline comparison
rules; unchanged healing stays plain. Runtime healing behavior is unchanged.

30 HUD tests cover trained Priest/Paladin healing and avoid duplicate tooltip
rewards. Native build, workspace Clippy, formatting and render lint pass. A live
Holy Fire rank 1 → 2 card with Holy talents fits at 1280×720; selection resumes
combat with rank 2. The screenshot is embedded in the changelog and README.
Compact talent-added descriptions and other healing card families still need
visual acceptance.


### Selected-build card previews — 2026-10-25

`profile = selected` extends the direct card preview through the production
run-start snapshot, including equipment, renown, relics, race, perks and validated
talents. It waits for catalog/class/look readiness, uses a private Selection and
keeps the original Run/Offer for restoration. Baseline is the default; profiles
are only accepted on cards. Invalid profiles/duplicate keys are rejected.

Six preview tests cover parsing, effect preservation, baseline clearing and
original-state restoration. Native build, workspace Clippy, formatting and render
lint pass. Native checks show Holy Fire/Holy Nova 7→8 talent healing fits at
1280×720 and 640×720, with the wrapped shield card reachable by scrolling.
Switching to baseline removes talent healing; returning to Talents works. The
isolated account SHA-256 is unchanged. Screenshots are embedded in README and the
changelog. This closes the compact Holy healing description check, not acceptance
for all talents/classes or passive/jewelry card previews.


### Priest defensive talents — 2026-10-25

Improved Power Word: Fortitude (DBC 344, two ranks) adds 10 maximum health per
rank. Improved Inner Fire (346, three ranks) adds 2 armor per rank. Spell Warding
(411, five ranks) adds 2% damage reduction per rank. These replace each node's
provisional damage bonus and flow through the selected hero/spec's account bonus
into the run sheet. Rank previews expose health, armor and reduction respectively.

Fortitude and Inner Fire are always active Survivors conversions, without upkeep
or charges. Spell Warding uses general incoming reduction because damage schools
are not yet modeled; its description explicitly covers all damage. The standard
armor-before-reduction formula and reduction cap still apply. These are initial
balance values, not a claim of completed Priest talent coverage.

Validation: 116 core tests include every implemented talent at every rank. The
app regression verifies health/armor/reduction reach the combat sheet, stay
independent between heroes/specs, persist and reset without changing an active
run snapshot. DBC audit, native build, workspace Clippy, formatting and render
lint pass. Native Inner Fire 2→3 purchase and 3→2 refund show the 4→6 armor preview
at 1280×720; screenshot embedded in README/changelog. Live damage balance and
other defensive talent layouts remain unverified.


### Blackout — 2026-10-25

Priest Blackout (DBC 464, five ranks) replaces provisional damage with a 2% chance
per rank for Priest Shadow damage to stun for three seconds (bosses: 0.9 seconds).
Direct hits and Shadow Word: Pain ticks roll independently per damage event;
multistrike damage is one event. The stun refreshes without adding duration and
never shortens a longer stun. Zero/non-finite damage and foreign/Holy spells do
not qualify. The selected spec snapshot controls the effect.

The shared spell description exposes the chance and durations on cards/tooltips,
using green comparisons when the chance changes. The core regression covers
rank caps, class/source scope, invalid damage and reset. The app regression runs
the real direct/periodic hit system for untrained/trained normal enemies and bosses,
including longer-stun preservation. 117 core tests, the engine regression, DBC
audit, native build, Clippy, formatting and render lint pass. Native rank 7→8
cards fit at 1280×720 and 640×720, including the long Psychic Scream shield/stun
text reached by scrolling. Screenshots are embedded in README/changelog. Live
crowd-control balance and remaining talents are still open.


### Vampiric Embrace — 2026-10-25

DBC 484 (one rank) grants always-active self-healing for 20% of health removed by
Priest Shadow spells. DBC 1638 (two ranks) adds five percentage points per rank,
requires the base talent, and caps at 30% total. Both replace provisional generic
damage. This is the Survivors conversion, without an upkeep debuff or party heal.
Direct and periodic paths heal from actual remaining enemy health, excluding
overkill; mixed periodic sources consume that health budget in tick order.
Existing generic leech/drain behavior is unchanged. Shared card/tooltip prose
shows the effective fraction and supports green inline changes.

118 core tests cover talent identity scopes, base prerequisite, bounded ranks,
invalid damage and reset. The app regression exercises real direct/periodic hits
at all improvement ranks with living/dead players, overkill and already-dead
targets. DBC audit, native build, Clippy, formatting and render lint pass. Native
1280×720 and 640×720 checks show combined healing/Blackout/shield text fits, with
the third compact card reachable by scrolling. Screenshots are embedded in README
and changelog. Live healing balance and remaining talents remain unverified.


### Shadow Weaving — 2026-10-25

DBC 461 (five ranks) replaces provisional damage with a 20% application chance per
rank on Priest Shadow damage, including Pain ticks. Each enemy independently
stores up to five 3% vulnerability stacks for 15 seconds. Successful application
refreshes the full duration after the triggering damage. Direct hits and periodic
ticks use the stacks present before that event; multistrike remains one event.
Expiry clears all stacks. Holy/foreign-class spells do not apply or benefit.
Vampiric Embrace receives the resulting health removed, still excluding overkill.

119 core tests cover bounded rank chance, spell/class scopes, invalid damage,
reset, stack cap, refresh and expiry. The engine regression verifies first-hit vs
second-hit damage for direct/periodic sources, trained/untrained and normal/boss
enemies. DBC audit, native build, Clippy, formatting and render lint pass. Native
1280×720 and 640×720 previews show vulnerability plus healing/control/shield text;
scrolling reaches the full compact third card. Screenshots are embedded in README
and changelog. Live vulnerability balance and remaining talents remain open.


### Companion reference icons — 2026-10-25

Compared `survivors_rewards_tab_companions.png` directly with decoded checkout
patch-chain icons. Long Leash now uses Ability_Hunter_BeastCall; Black Kingsnake
and Sprite Darter use Ability_Hunter_Pet_WindSerpent; Tree Frog uses Pet_Turtle,
Zergling Pet_Scorpid, and Undercity Cockroach Pet_Crab. These intentionally follow
the reference's illustrative choices rather than changing creature displays.
Saved indices, unlock requirements and mechanics are unchanged.

Native 1280×720 and 640×720 checks verified icon rendering and the complete
scrollable Looks catalog. Screenshots are embedded in CHANGELOG and README.
The actual-data model/icon test, native build, formatting and render lint pass;
224 Survivors app integration tests passed before this icon-only change.
In-world pet animation/scale acceptance and broader layout polish remain open.

### Companion appearance control consistency — 2026-10-26

Unlocked looks use the shared hover/pressed/selected control treatment. Locked
looks retain their icon and achievement requirement without a Button/Act, so they
are excluded from click and keyboard/controller action discovery. Native wide and
compact previews verify selection by icon, locked Panda Cub rejection and readable
scroll reachability. Screenshots are inline in CHANGELOG and the README gallery.
Native build, workspace Clippy, formatting and render lint pass. Physical gamepad
acceptance and in-world appearance validation remain open.

### Saved Companion appearance validation — 2026-10-26

Both saved look indices are checked after the complete account file has loaded,
so achievement row order cannot change availability. Unknown indices and locked
looks, including the two unachievable placeholders, fall back to Snowshoe Rabbit.
Valid choices, earned Veteran and legacy Chicken access remain intact. A legitimate
second-slot look is preserved even before that slot is owned. Regression coverage
checks both slots and serialization; native screenshots verify repair, independent
second-slot selection and restart persistence. All 262 app tests and the native
build, workspace Clippy, formatting and render lint pass. In-world rendering
acceptance remains open.

### Hunter Survival milestones — 2026-10-26

Three automatic Survivors adaptations complete Survival's milestone definitions:

| Points / milestone | Effect |
| --- | --- |
| 10 · Monster Slaying | Serpent Sting and Explosive Trap direct/periodic damage ×1.2 |
| 20 · Surefooted | Concussive Shot grants a 5% maximum-health absorb for 3 seconds once per successful cast; no stacking |
| 30 · Wyvern Sting | Serpent Sting adds a 1-second stun (bosses: 0.3 seconds), additive with individual Wyvern Sting training |

Monster Slaying applies to all enemy types as an authored poison/trap specialization;
Surefooted uses an absorb as its defensive adaptation; Wyvern Sting uses the existing
stun system rather than sleep. These provisional values require live combat and balance
acceptance. Maximum individual training yields ×1.32 Serpent Sting damage, ×1.56
Explosive Trap damage and a 2.5-second Serpent Sting stun (bosses: 0.75 seconds).
No ability is granted by a milestone. Areas, range, targeting and periodic duration
are unchanged; stronger existing shields are preserved. The absorb happens once per
successful cast, and the stun is applied on the direct hit, not each periodic tick.

135 core and 263 app tests pass, including threshold boundaries, rank composition,
cast parameters, enemy control and non-stacking defenses. Native empty-spec previews
verify all three descriptions; screenshots are inline in CHANGELOG/README. Native
build, workspace Clippy, DBC coverage audit, formatting and render lint pass.
Milestone coverage is 78/81. Hunter combat pets and the three Beast Mastery milestones
remain open together, along with remaining individual talent nodes.

### Codex spell prose and defensive legendary labels — 2026-10-26

Legendary entries now use the shared card/tooltip spell prose for an explicitly
unmodified rank-one preview, including recharge timing. Healing, absorbs, immunity,
control and damage retain their actual meaning rather than displaying every base
value as damage. Discovery labels match shared jewelry: previously offered/not offered.

Codex and legendary offer comparisons share contextual Healing/Absorb labels. They
omit critical-damage bonuses for Heal, Shield, Immune and Thorns, whose cast handlers
do not roll critical strikes; immunity also omits the unused power multiplier.
Replacement comparisons retain their red losses and green gains. This is a display
correction, with no change to the underlying generic legendary bonus or save format.
Distinct spell-specific legendary designs remain required: notably Divine Shield's
current generic amulet has no effective benefit and now reports that honestly.

Native 1280×720 healing and 640×720 absorb details fit with readable text and unchanged
catalog selection behavior. Screenshots are embedded in CHANGELOG/README. Tests cover
all catalog spell descriptions, defensive wording and legendary replacement/duplicate
comparisons. Full catalog content and legendary balance acceptance remain open.

Validation for this checkpoint: 265 app tests, native play build, workspace Clippy,
formatting and render lint pass.

### Live jewelry card previews — 2026-10-26

The watched menu preview accepts one to four ring, trinket, amulet or legendary
entries, with wide/compact presets under `docs/ui-previews/`. Catalog bounds and
mutually exclusive ability/jewelry hands are checked before applying a request.
Preview hand changes preserve the original run/offer until leaving cards; picks
remain disabled. This extends the fast inspection loop to chest offers without
requiring a run or granting discovery/progression.

Visual inspection exposed raw legendary spell keys and missing icons when the
spell was not learned. Legendary cards now resolve names and icons from the spell
catalog, independently of the current loadout. This does not change acquisition,
rarity, bonuses or saves. Passive-card presets and authored legendary effects remain
open.

Validation: 266 app tests, native play build, workspace Clippy, formatting and render
lint pass. Native 1280×720 and 640×720 checks cover corrected icons/names, green
bonuses, compact scrolling, disabled picks and return to the undiscovered Codex.
Screenshots are embedded in CHANGELOG and README.

### Windows playtest follow-up — 2026-10-26

- **Foliage reveal:** replace the collision-only activation probe with rendered
  scenery triangles from all three pick/render paths. Probe at 10 Hz after visibility
  classification, ignore units and scenery behind the hero, and retain the 500 ms
  fade. This is geometry visibility, not alpha-texture/pixel sampling; camera push-in
  remains a separate collision feature. Exact Windows foliage scene acceptance is
  still open.
- **Spawns and routes:** require a clear solid-world segment from hero to spawn in
  addition to supported, unoccupied body space. This conservatively rejects hollow
  trunk interiors and unverified connections across fences; it can defer waves in
  tight rooms. Existing enemies still use incremental A*, now servicing up to eight
  requests × eight edge probes per frame, with 512 nodes per search and 64 pending
  requests. IDA* has not been introduced; shared navigation regions, long fences,
  enclosed areas and crowded Windows performance remain acceptance work.
- **Elite chests:** first eligible named-elite death after 60 seconds, then at least
  90 seconds apart, at most five per run. Same-frame bursts cannot multiply drops.
  Boss/start/wooden rewards stay separate. Verified DBC displays are now 10/1/41
  (Chest01/02/03); the previous 11/12 references were a sarcophagus and grave marker.
  Chests remain solid and are routed around;
  overlapping drops/enemies and the reported live Windows jam still need scene replay.
- **Talent art:** use the painted 300×331 area of the padded 320×384 vanilla tile
  canvas. Crop right/bottom padding at runtime, preserve aspect ratio, and keep all
  icons and connectors in the same proportional coordinate space. Checked the
  bottom-right alpha extents of all 28 shipped background variants (one has a final
  transparent column). Native 1280×720/640×720 checks cover all three Hunter trees,
  scrolling, and purchase/right-click refund at the visible icon. Screenshots are
  embedded in CHANGELOG and README. No install assets are committed.

These changes address identified causes, but do not claim reproduction or closure
of the user's exact Windows gameplay scene. Recheck canopy reveal with scenery in
front/behind, fence detours under an elite wave, and enemy movement past dropped
chests on Windows before declaring those reports closed.

A short manual native check confirms the new tier-A models render as chests, with
an inline milestone capture. The UI preview's world remained at the Dark Portal;
this is model evidence only, not an Elwynn navigation or spawn acceptance run.

Validation: 135 core tests and 270 app tests pass with the visibility feature and
explicit checkout data. Regressions cover render-only foliage, authored Elwynn
trunk spawn rejection, actual body movement around fences and all three chest
models, bounded route scheduling, and elite-drop cadence. Native play build,
workspace Clippy, formatting, and render lint pass.

### Divine Shield legendary effect — 2026-10-26

The previously ineffective generic amulet now adds 15 percentage points of
spell-local cooldown reduction for Divine Shield. The existing cap/minimum timer,
rank scaling and talent composition apply. Immunity duration is unchanged. The
shared hero sheet and other spells do not gain this bonus, repeated copies do not
stack, and replacement removes it. No save migration is needed for run-only jewelry.

Native 1280×720 and 640×720 cards show the green benefit; selecting the undiscovered
Divine Shield Codex entry shows the same effect without marking it discovered.
Screenshots are embedded in CHANGELOG and README. This resolves the ineffective
Divine Shield reward noted above; remaining spell-specific legendary designs and
live combat balance still require work.

Validation: 135 core and 271 app tests pass. New checks cover rank-one/four/eight
cast timing, unrelated spells, duplicate equips, replacement removal, green gains
and red losses. Native play build, workspace Clippy, formatting and render lint pass.

### Shared jewelry artwork completion — 2026-10-26

The Codex audit reproduced the remaining question-mark icon. Mark of the Champion
now explicitly uses vanilla item 23206/display 35649, verified in the pinned item
templates and checkout data. The other same-name variant is not an additional
Survivors item. This resolves the earlier unresolved-art notes without changing
stats, discovery keys, or save format. All 39 shared jewelry entries must now resolve
item artwork in both the catalog audit and chest-card tests; no exception remains.

Native macOS checks cover the complete grid, selected undiscovered detail at
1280×720, and the compact chest card at 640×720. Inline screenshots are in the
changelog and README. Full legendary content and broader responsive acceptance
remain open.

Validation: all 271 app tests with explicit checkout data, native play build,
workspace Clippy, formatting and render lint pass.

### Paladin Retribution damage talents — 2026-10-26

Sanctity Aura adds 10% damage to the seven Paladin Holy attacks. Vengeance grants
3% damage per rank for eight seconds after a damaging direct Holy critical hit;
further qualifying criticals refresh without stacking. Consecration benefits from
the damage bonus when cast, retains that damage for its lifetime, and cannot trigger
Vengeance from its ticks. Immunity, other-class abilities and neutral jewelry spells
are excluded. Both effects multiply with existing cast bonuses; the shared hero
sheet is not inflated by the conditional spell-local buff. Refund/spec replacement
removes the bonus, expiry stops it, and a dead hero cannot activate it.

Core tests cover rank bounds, trigger exclusions, spell scope and reset. App tests
cover cast damage, refresh/expiry, no stacking, unrelated spells and removal. The
native 1280×720 empty-spec preview verifies both descriptions and prerequisite
refusal, not purchase or live combat balance. Screenshots are embedded in CHANGELOG
and README. The DBC audit verifies Paladin 21/44 nodes and total 178/432; all 78/81
milestone descriptions remain as before. Full talent and responsive acceptance is
still open. 136 core tests and 272 app tests pass with explicit checkout data.
Native play build, workspace Clippy, formatting and render lint pass.

### Targeting-range talents — 2026-10-26

Hawk Eye adds 2 yd per rank (three ranks) to Hunter shots, Serpent Sting and Volley,
excluding Explosive Trap. Storm Reach adds 10% per rank (two ranks) to Lightning
Bolt and Chain Lightning's initial targeting range. Grim Reach adds 10% per rank
to Corruption, Curse of Agony and Death Coil. Destructive Reach adds 10% per rank
to Shadow Bolt, Immolate, Searing Pain, Shadowburn and Rain of Fire. Nature's Reach
adds 10% per rank to Wrath, Moonfire, Starfire, Entangling Roots and Hurricane.
The latter three nodes have two ranks. Ground spells extend placement distance;
area radius and chain bounce distance do not change. Player-centered effects,
healing, unrelated spells and target counts are excluded.

Core tests cover the full ability catalog, bounded ranks, foreign classes and reset.
App tests exercise cast parameters and a target between base and trained range,
plus unchanged damage, duration, cooldown, radius and count. Native 1280×720 Hunter
and Warlock previews verify descriptions and locked-node refusal; live combat and
the full responsive matrix remain open. Screenshots are embedded in CHANGELOG and
README. The data audit verifies 183/432 nodes; milestone coverage remains 78/81.

Validation: 137 core and 273 app tests pass with checkout data. Native play build,
workspace Clippy, formatting and render lint pass.

### Talent icon scaling and responsive evidence — 2026-10-26

Compared with `intentions/survivors_talent_tab.png`: retain the three illustrated
trees, rank badges, centered prerequisite lines and separate detail panel. Compact
screens intentionally wrap panels. Native 1024×768 inspection exposed fixed-size
icons crowding adjacent rank badges as artwork shrank. Icons now use 12% of tree
width (36/300), capped at 36 logical pixels, square and centered on their grid
coordinates. Background aspect ratio and readable text sizes are preserved.

Native Warlock checks at 640×720, 1024×768 and 1280×720 verify spacing, long green
descriptions and reflow. Before the fix, compact interaction checks verified an
independent Spec 1 purchase and restoration of Spec 3's lower-panel scroll position.
After the fix, 4:3 left-click increased Improved Corruption from 1 to 2, right-click
restored rank 1 and the unspent total, and a locked Destructive Reach refused spend.
The updated compact page still reaches its lower tree and complete description.
Screenshots include before/after and final compact/wide views; README is updated.
These checks do not establish all-class, physical-controller or Windows acceptance.

Native play build (43 seconds), workspace Clippy, formatting and render lint pass.
No mechanics changed; the prior 137 core / 273 app result was not rerun for this
layout-only change. Reusable `talents-640.txt`, `talents-1024.txt` and
`talents-1280.txt` presets are under `docs/ui-previews/`.

### Direct talent inspection previews — 2026-10-26

The opt-in native development preview accepts a positive `talent` DBC ID only on
Talents. Validate against the selected hero's loaded tree before applying any page,
size or inspection changes; loading data waits, unknown/foreign nodes and absent
heroes reject the whole request. A valid request pins the actual node description
and saved rank without triggering controller actions, purchasing or refunding.
A valid reload omitting the selector clears the pin. Manual gameplay actions remain
normal, so interaction testing continues to require isolated accounts.

Native 1024×768 direct launch shows Grim Reach without clicking. Requesting Hunter
Hawk Eye while a Warlock is selected preserves the prior description and dimensions,
with a rejection log. A valid live change shows Destructive Reach at 640×720;
clearing the selector restores ordinary milestone inspection. Fixture hashes before
launch and after shutdown are identical. Inline screenshots are in CHANGELOG and
README; the production layout remains aligned to the existing talent intention.

46 Armory tests pass, including parse rejection, pinned detail priority, unchanged
ranks/actions and restoring ordinary detail/color handling. Native play build,
workspace Clippy, formatting and render lint pass. The client-data audit was rerun
and the main coverage table updated to 183/432. No new mechanics or all-platform
acceptance is claimed.

### Collection completion and cloak eligibility — 2026-10-26

The six-band equipment audit confirms one added stat line per band, stronger prior
lines and independent worn/cosmetic selection. It also exposed a separate defect:
cloak collection eligibility reused the nine-slot set predicate. A new complete-grid
predicate now requires every catalog cell (54 currently) in one class, in both the
selected-hero bonus path and achievement snapshot. Nine-slot Full set and worn-set
bonuses are unchanged. Duplicate counts and another class's pieces cannot substitute
for a missing cell. Ownership counts ignore bytes beyond the current catalog.

Core tests cover missing/short grids, stack counts and trailing data. Account tests
cover nine slots versus 53/54 cells, foreign-class isolation, exact cloak damage/speed
contribution, reload, single achievement grant and preservation of old earned points.
Legacy achievements do not supply cloak power if current feats are unmet. Native
1280×720 seeded fixture reloads show Cloak bearer locked at 53 and earned at 54;
that final legendary piece also grants Legendary (25 points), alongside the cloak's
10. This verifies loading/presentation, not acquisition during a live run.

Inline screenshots are embedded in CHANGELOG and README. Poor/raid-Epic identities,
obtainable sources, distinct raid sets and their save/UI expansion remain required.

Validation: 138 core and 276 app tests with checkout data pass, as do the native
play build, workspace Clippy, formatting and render lint.


### 2026-10-26 — Cloak progression presentation

Character and Collection share an explicit cloak section below equipment bonuses:
selected hero's class-cloak quality and actual bonus, talent-cap/raid/collection/amulet
requirements, and the account's epic-class-cloak count. Completed requirements are
green. The account legendary bonus is shown separately when eligible. Talent-cap
credit requires the selected ID to belong to the requested class; a regression test
covers foreign and missing IDs. Existing additive cloak mechanics are unchanged.

This intentionally extends the Character reference while retaining its wide
loadout/model/features layout and compact page scrolling. Native seeded screenshots
at 1280×720 and 640×720 show the section and are embedded in CHANGELOG and README.
No cosmetic cloak model or new acquisition source is included.

Also corrected the development preview's system parameter and selection code to
use the same compile gate as its module. Native play build, 277 app tests,
non-development library check and workspace Clippy pass. Existing warnings remain
in the non-development feature combination. The wasm target is not installed, so
this milestone does not claim a wasm check or Windows gameplay acceptance.


### 2026-10-26 — Passive-aware Improved Renew

The checkout DBC identifies Improved Renew as talent 406, three ranks, Holy row 0.
Renew is a Survivors passive, not an active spell: its talent now adds 5% regeneration
per rank to that passive only. Training alone grants no regeneration. Other passive
stats, gear/racial regeneration, Spirit Tap and healing spells are unaffected. The
normal supported-node adjustment removes its provisional 1% damage per point.

A shared Run passive-value calculation feeds the stat sheet, new/upgrade card prose,
benefit summaries and passive tooltips. Regeneration formatting retains up to three
decimal places, so the first talent rank's 0.5→0.525 health/sec is not hidden. Core
coverage checks class isolation, rank bounds and spec reset; runtime tests check all
Renew levels and an unrelated racial regeneration source; card tests check exact
inline amounts, green tint and a single unit. Native seeded previews verify rank
1→2 spending, saved rank 2, and wide/compact talent descriptions against the three-tree
intention layout. No gameplay healing capture or Windows acceptance is claimed.

Validation: 139 core and 279 app tests, native play build (27 seconds), workspace
Clippy, formatting and render lint pass. The refreshed checkout-data audit reports
184/432 matching talent definitions and 78/81 milestone descriptions; these counts
do not establish gameplay acceptance.


### 2026-10-26 — Passive card preview acceptance

The watched UI request now supports `passives=key:rank`, with rank 0 acquisition
and ranks 1–4 upgrades. It validates actual passive keys, bounded ranks, one-to-four
cards, cards-page scope and one source per request (abilities/passives/jewelry).
Selected profiles reuse run-start talents, while baseline profiles remove them.
Preview hands replace their temporary passive list; leaving restores the original
run, owned passives and offer. Picks and gameplay remain disabled by the existing
preview guard. Wide/compact presets are included under docs/ui-previews.

Native seeded checks show Improved Renew rank 1 yields 0.525→1.05, 1.05→1.575 and
2.1→2.625 health/sec for new, rank-2 and rank-4 cards. A maximum-rank request with a
new viewport preserves the existing view; a valid compact request resizes and scrolls
to the final card; returning to Talents removes the overlay. The account hash is
unchanged. Screenshots are inline in CHANGELOG and README. The bordered card layout
was compared with the level-up intention; inline green amounts follow the user's
later instruction. This verifies preview presentation, not live run acquisition.

Validation: all 12 preview tests pass, including catalog/rank/source rejection,
trained versus baseline values, switching hand types and original-state restoration.
Native build, workspace Clippy, formatting and render lint pass.


### 2026-10-26 — Armory action focus retention

Generalize the Rewards-only action identity to starter and Armory controls, scoped
by page, selected hero/class and applicable category (Rewards, Achievements, hero
management). Talent nodes retain their rank-independent identity. A unique visible
replacement receives focus after layout without replaying its action. Missing,
disabled or ambiguous replacements drop the identity; page/hero/category changes,
modal UI, typing and pointer handoff clear it. This supports repeated appearance,
loadout and navigation actions when a page rebuilds. Physical-controller acceptance
and comprehensive per-page traversal remain open.

Native keyboard checks confirm two successive skin changes (2→1→10) keep the same
outline, resizing to 640×720 restores/reveals the control, and navigating to Hair
style then selecting changes 2→1 while retaining focus. Saved fixture feature bytes
confirm both values. The wide screenshot is embedded in CHANGELOG/README; Character
layout follows the existing intention comparison. All 282 app tests pass, including
replacement/layout-delay/no-replay/hero-page boundaries across seven page/action
pairs and the existing Rewards category, disabled-control and pointer guards.
Native play build passes. This does not establish every page's live traversal or
physical controller acceptance.
Workspace Clippy, formatting and render lint also pass.


### 2026-10-26 — Browser compile gate verified

Installed the pinned Rust toolchain's wasm32-unknown-unknown target and the
repository-pinned WASI SDK 27 under ignored tools/wasi-sdk. The CI command
`cargo check --target wasm32-unknown-unknown -p wenilla --no-default-features --features webgpu`
passes locally (five existing client warnings). This supersedes the earlier
missing-target limitation; it establishes compilation, not linked browser or
WebGPU runtime acceptance.

Updated the browser startup test DOM mock for loading-screen CSS properties and
child elements, and isolated animation intervals. All seven startup/journal tests
pass using the bundled Node runtime; the system Node installation could not start
because its llhttp dynamic library is missing. No application behavior or visual
layout changed, so this checkpoint requires no new screenshot. Windows gameplay
and browser runtime checks remain open.


### 2026-10-26 — Individual Siphon Life training

The checkout catalog identifies Siphon Life as Affliction talent 1041, one rank,
row 4. Its Survivors conversion heals 5% of health actually removed by Corruption
and Curse of Agony, adding to the independent 30-point milestone for 10% total.
The shared direct/periodic hit pipeline already excludes overkill and dead targets
and prevents reviving the hero. Training replaces the placeholder generic damage
bonus; other spells/classes are excluded, and spec reset removes the trained part.

140 core tests and 281 Survivors app tests pass, including individual/milestone
composition, rank caps, invalid damage, class/spell scope, reset and actual hit
processing. Native play build passes. Catalog coverage is 185/432 definitions and
78/81 milestone descriptions. Native fixture checks verify purchase from 0/1 to
1/1, one point spent, persisted spec 3, and readable wide/compact details after
scrolling. Screenshots are embedded in CHANGELOG and README; preview presets pin
talent 1041 without a run. The three-tree art/panel arrangement follows the
intention reference and wraps at compact widths. Live combat balance remains open.
Workspace Clippy, formatting and diff checks also pass.


### 2026-10-26 — Individual Nightfall proc

Nightfall (Affliction talent 1002, two ranks) replaces its generic damage fallback
with a 2%/4% Corruption-tick chance to ready an owned Shadow Bolt. This Survivors
instant-cast conversion does not grant the spell or bank a charge while it is ready.
Each eligible tick rolls once after multistrike and actual health removal; direct
hits, other spells, exhausted target health and dead heroes cannot trigger it.
The 20-point Affliction milestone remains a separate guaranteed one-second advance
on a successful Corruption cast. Spec changes remove individual training normally.

141 core and 283 app tests pass, including a deterministic roll between the two
rank probabilities, missing/ready spell ownership, single-roll consumption, direct
versus periodic processing, invalid damage, class scope, caps and reset. The native
play build, workspace Clippy, formatting and diff checks pass. Catalog audit verifies
186/432 definitions and 78/81 milestone descriptions. Native fixture purchases
0→1→2 spend two points and persist talent 1002 rank 2 in spec 3. Wide and compact
screenshots show the next-rank preview and maximum-rank description without
clipping, using the intention's art-backed trees and separate details panel.
Screenshots are embedded in CHANGELOG/README and reusable preview presets are in
docs/ui-previews. Live combat balance and the remaining talent catalog stay open.


### 2026-10-26 — Render-only reveal integration and PR gate

The active visibility adapter uses WorldPick rendered triangles, not physics
collision. Removed its obsolete, unused collision-only helper from the world
visibility module. New scheduled-system coverage feeds collider-free ordinary
meshes and merged PickBlob members through the actual adapter into SceneryCutaway.
It measures five 100 ms real-time updates to fully reveal and five to restore
opacity after moving scenery behind the hero. It also checks focus/radius output,
continued real-time fading while paused, disabled-option clearing, and reset on
menu/game-over transitions. Existing triangle tests cover off-axis scenery and
actor exclusion. No runtime rendering policy changed in this checkpoint.

The previous PR workflow omitted Survivors tests, and the default documented test
command excluded this feature-gated adapter. CI now runs survivors-core and the
visibility-enabled Survivors app suite; the matching command is documented in
SURVIVORS-DEVELOPMENT. Both commands pass locally: 141 core and 285 app tests.
These fixtures do not prove alpha-texture-aware occlusion, retained StaticGx
rendering, WMO cutaway, or the user's exact Windows foliage scene. Those remain
open acceptance work. No new screenshot is claimed for this nonvisual checkpoint.
Visibility-enabled app Clippy, formatting and diff checks also pass locally.


### 2026-10-26 — Direct Collection preview acceptance

The live preview parser accepts `piece = <band>:<slot>` on Collection pages only.
All 54 existing cells are covered, including case/whitespace normalization and
invalid/duplicate/cross-page rejection. Omission preserves the current inspection;
selection does not grant or equip gear. Two reusable Epic Hands presets cover
1280×720 and 640×720. Native validation switched between Epic Hands and Legendary
Main Hand, rejected an invalid Off Hand request without resizing, and checked
compact item details by scrolling. The post-startup account SHA-256 stayed unchanged
through shutdown. 287 visibility-enabled app tests, native build, app Clippy,
formatting and diff checks pass. Actual screenshots are embedded in CHANGELOG/README.

Windows playtest follow-up remains an acceptance requirement for the existing
corrections: test a rendered leaf canopy without collision between camera and hero,
then move it behind the hero and verify the 500 ms fade; test Elwynn tree interiors
and both sides of a fence for spawn rejection/routing; test enemy routes around a
dropped chest and elite reward cadence (first at 60 seconds, 90-second spacing,
maximum five, boss rewards separate). Record build SHA, location and renderer settings
with captures. The implemented route search is bounded incremental A*, not IDA*.
Talent artwork must retain its 300:331 painted proportions with centered square
icons at wide and compact sizes. Existing macOS captures and synthetic tests do not
close these exact Windows reproduction checks.


### 2026-10-26 — Individual Druid Restoration capstones

Audited talent 831 (Nature's Swiftness, one rank) multiplies Rejuvenation cooldown
by 0.85, composing with existing individual cooldown talents and the 20-point
Restoration milestone. Talent 844 (Swiftmend, one rank) adds a 5% maximum-health
immediate heal once per successful Rejuvenation cast, adding to the 30-point
milestone for 10%. These are explicit automatic Survivors conversions. They do not
change Rejuvenation's below-85%-health cast condition, grant the spell, consume its
healing over time, change its geometry, or apply to other classes/spells.

142 core and 288 visibility-enabled app tests pass. Coverage includes class/spell
scope, rank caps, reset, milestone composition, cast parameters, capped healing,
full/dead hero behavior and preservation of regeneration. The native play build
(23 seconds), app Clippy, formatting and diff checks pass. Catalog audit confirms
188/432 individual definitions and 78/81 milestone descriptions. Native fixture
purchases spend two points (30→32) and persist 831/844 rank 1 in spec 3; wide and
compact screenshots show the trained effects using the existing proportional
art-backed trees. Fixture support includes the still-provisional Insect Swarm node;
this milestone does not claim the full Restoration tree is complete. Screenshots
are embedded in CHANGELOG/README, with Swiftmend preview presets for both widths.
Live combat balance and Windows gameplay acceptance remain open.


### 2026-10-26 — Character short-viewport acceptance

At the supported minimum 640×480 viewport, the compact Character portrait's
width-only sizing pushed its rotation controls below the initial view. Its square
frame now uses the smaller of 60% column width and 52% viewport height. This keeps
the reference's centered full-body figure while respecting short windows; the
three-column wide layout is unchanged. Native before/after evidence verifies the
full Human Warrior figure and both rotation controls, with actual left/right turn
interaction and scrolling to appearance/equipment. Live resizing to 1024×768 and
640×720 also keeps the full figure and controls visible. Added reusable Character
preview presets (`page = heroes`) for all three sizes. Screenshots are inline in
CHANGELOG and README. Native build (23 seconds), app Clippy, formatting and diff
checks pass; no new rule tests are warranted for this layout-only bound. The full
race/sex/equipment matrix, physical controllers and Windows DPI remain open.


### 2026-10-26 — Character identity reference alignment

Compared against intentions/survivors_character_tab.png, the central identity line
was missing gender and race. It now describes the selected rendered body followed
by the class name in its class color. Inline spans explicitly use the same font and
body size. A class-catalog mismatch suppresses the body prefix while selection is
loading, rather than describing an unrelated cached look. No save format changes.
Native validation covers Male Human Warrior → Female Human Warrior → Female Tauren
Warrior, then switching through Manage heroes to the saved Female Human Paladin.
1280×720 and 640×480 captures verify identity, model fit and class coloring; these
are embedded in CHANGELOG/README. The native build, app Clippy, formatting and diff
checks pass. This presentation-only change does not add rule tests. Full race/sex/
equipment coverage and Windows DPI remain open.


### 2026-10-26 — README gallery consolidation

The README had accumulated 781 lines of repeated milestone images alongside a
stale gallery caption that dated newer captures as October 24. It now presents
20 representative captures across starter, Character, hero management, Collection,
Drop Tables, Codex, Talents, Achievements, all three Rewards categories, Options,
upgrade cards and explicitly historical gameplay/results views. Dates are attached
to individual captures; fixtures and platform acceptance limitations are stated.
All 187 screenshot paths from the previous README milestone section are preserved
in docs/SCREENSHOT-HISTORY.md, including the existing image embeds, with original
files untouched. Existing
CHANGELOG image embeds were preserved verbatim. Future checkpoints update the
relevant README page while retaining inline milestone history in CHANGELOG.
Local image/document references were checked. No UI or runtime behavior changed;
this does not close fresh gameplay capture or full interface acceptance work.


### 2026-10-26 — Retained foliage query coverage

The retained renderer already participates in WorldPick independently of the entity
pick set. A new feature-gated world test casts through an AlphaTest retained triangle
with no entities or colliders, verifies its identity/distance, and checks clear rays,
culled geometry, visibility removal and absence of the retained resource. CI runs
this test alongside the existing ordinary/merged-mesh reveal integration tests.
This checkpoint changes no runtime rendering. Texture-alpha sampling, GPU cutaway
appearance, WMO cutaway and the exact Windows foliage replay remain open; synthetic
query coverage does not close any of those acceptance requirements.
Validation: the new public-query test and both existing retained-source tests pass;
formatting and diff checks pass. The first world-test configuration build took four
minutes; the adjacent cached test invocation finished in under one second. No game
was launched and no new visual evidence is claimed.


### 2026-10-26 — Individual Priest Spiritual Healing

DBC 404 (five ranks) now provides 2% healing per rank to the current Priest healing
sources: owned Renew and the 30-point Holy milestone's Holy Nova heal. It multiplies
Improved Renew; at full training Renew is 1.265× baseline and Nova heals 5.5% maximum
health. It does not grant an ability or unlock Nova healing early. Damage, absorbs,
Spirit Tap, Vampiric Embrace and racial regeneration remain unchanged. The normal
supported-node adjustment removes the old generic damage per point. Future Priest
healing abilities must explicitly join this scope when implemented.

Core coverage checks rank caps, class/spell isolation, milestone thresholds and
reset. App coverage checks owned Renew with separate Troll regeneration and Nova
healing, overheal/death caps. Native fixture spiritual-healing uses 25 support points
(including still-provisional Improved Healing), purchases rank 1, verifies persistence,
then refunds it at 640×720. Wide/compact screenshots are embedded in CHANGELOG and
replace the relevant README gallery cells; the earlier Druid captures stay in the
changelog/history. Reusable read-only preview requests are under docs/ui-previews/.
The checkout-data audit reports 189/432 definitions and 78/81 milestone descriptions.
This does not complete Priest talents, combat balance or Windows acceptance.
Validation: 143 core tests, 290 visibility-enabled app tests, native play build
(59 seconds), app Clippy, formatting and diff checks pass.


### 2026-10-26 — Hunter Survivalist and completed startup health

DBC 1622 replaces the generic damage placeholder with 2% final maximum health per
rank (five ranks). A dedicated talent multiplier is applied after flat gear, passive,
renown and racial health; no Bonus/save schema changes are needed. Core tests cover
rank caps, duplicate saved entries, class isolation and reset. App tests combine
Tauren health, renown, a health passive and flat gear bonuses, verify later flat
bonuses scale, and verify stat queries do not alter current health.

Startup previously filled health before setting the saved talent snapshot. All five
entry/preview sites now finish through one startup helper that installs perks and
talents before filling health. This helper is explicitly startup-only. The starter
shows the talent percentage separately from the class card's base-health number.
The latter is not a complete equipped character sheet.

Native isolated fixture survivalist has ten supported prerequisite points (Deflection
and Entrapment). Purchasing rank 1 persists; 1280×720 and 640×720 descriptions and
the starter +2% summary were captured in CHANGELOG/README. The starter capture is
bonus-text evidence, not acceptance of the full body/model matrix. Reusable talent
preview files are included. All 144 core and 291 visibility-enabled app tests,
native build, app Clippy, formatting and diff checks pass. Catalog coverage is
190/432 individual talents and 78/81 milestones. No combat session was run; Windows
and gameplay balance acceptance remain open.


### 2026-10-26 — Hunter body identity visual verification

A suspected Human/Dwarf mismatch in the previous small starter capture was checked
at full size after a clean launch. It is the female Dwarf model; no renderer change
was warranted. Native 1280×720 interaction then exercised female Dwarf → male Dwarf
→ male Night Elf → female Night Elf on the same isolated Hunter. Each selection
visibly updates the body, identity, race/gender selection and customization ranges;
the full figure and turn controls fit. Four captures are embedded in CHANGELOG;
README's Character cell now uses the Night Elf capture. The saved body ends at 4/1;
the saved talent ranks and equipment selections remain unchanged.

This adds evidence for these four unequipped Hunter bodies only. It does not prove
all feature values, gear attachment combinations, every race/class, starter-panel
resolution parity or Windows behavior. No runtime code changed and no code tests
were rerun. The validation application was closed and its launcher restored.


### 2026-10-26 — Character appearance position indicators

Added the reference design's gold selected/dim unselected position dots under each
appearance counter. Dot counts use the loaded race/body options; selection colors
update without rebuilding the page. Sizing follows interface typography and fits
within the available control width. Native Night Elf Hunter checks at 1280×720
verified hair-style wraparound 7→1; at 640×720, 1→2 updates the indicator without
resetting the scrolled feature panel. Both captures are embedded in CHANGELOG;
README's existing Character cell is updated. Native build and app Clippy pass.
This is menu evidence only; Windows gameplay reports remain open for exact-scene
acceptance as recorded in the playtest correction milestone.


### 2026-10-26 — Shaman Restoration deployment healing

DBC 588 Restorative Totems (5 ranks) and 592 Purification (5 ranks) now add 5% and
2% healing per rank respectively to Searing/Magma Totem's existing 10-point
Restoration deployment pulse. Both are explicit Survivors conversions; neither
unlocks healing early or changes damage, shield amounts or passive regeneration.
At full ranks the 3%-maximum-health pulse becomes 4.05%. Generic point damage is
removed for these supported nodes; tree milestone totals still count the points.

Core regression coverage checks all classes, the 9/10-point unlock boundary, capped
ranks, unrelated spells, passive scope and reset. Application coverage exercises
actual cast completion with missing health, overhealing and a dead hero. Native
1280×720 purchase and 640×720 refund of Purification persist correctly; both talent
descriptions fit and screenshots are embedded in CHANGELOG and the existing README
gallery cells. Preview presets are included. The fixture's row-zero supporting
nodes remain provisional; this milestone does not claim the entire Restoration tree.
No unattended combat was run; live balance and Windows acceptance remain open.

Validation: 145 core tests, 291 visibility-enabled Survivors app tests, native build,
core/app Clippy, formatting and diff checks pass. The DBC audit reports 192/432
individual definitions and 78/81 milestone descriptions; neither is full acceptance.


### 2026-10-26 — Druid healing composition correction

SpellHealing bonuses now multiply both the spell's healing payload and any unlocked
immediate heal. The immediate multiplier applies after collecting flat heals, so
adding a later talent definition cannot bypass it. This fixes Improved Rejuvenation,
Gift of Nature and Nature's Focus with Swiftmend and the Restoration 30-point pulse.
The 10-point periodic-healing milestone still affects only periodic healing. Neither
healing ranks alone nor this fix grants an immediate pulse before its unlock.

Regression coverage checks capped ranks, milestone-only/Swiftmend-only/combined
pulses, reversed saved rank order, refund, unchanged cooldown and unrelated spells.
Application cast completion verifies the scaled heal without consuming the existing
healing-over-time effect. The Swiftmend description clarifies that 10% is before
healing bonuses. This is a rules correction; no new layout screenshots or live combat
acceptance are claimed, and talent definition coverage is unchanged.

Validation: all 146 core and 291 visibility-enabled Survivors app tests, native
build, core/app Clippy, formatting and diff checks pass.


### 2026-10-26 — Order-independent talent save references

Talent rows are staged until all character records have loaded, matching appearance
reference handling. This prevents silent rank loss when a valid save's talent rows
precede character rows. The parser preserves last-row replacement behavior and
ignores nonexistent references; malformed IDs no longer fall back to hero 0.

The regression compares two heroes across all three specs in ordered/reordered
saves, including duplicate-rank normalization, milestone snapshots, armor effects,
and encode/decode persistence. It also checks duplicate row replacement, unknown
hero IDs and malformed IDs alongside a legacy hero 0. This does not alter catalog,
rank-budget or prerequisite validation. No interface or visual acceptance changed.

Validation: 292 visibility-enabled Survivors app tests, native build, app Clippy,
formatting and diff checks pass.


### 2026-10-26 — Explicit gear tier rules and migration boundary

Added `GearTier` to distinguish Poor, Common, Uncommon, Rare, ordinary Epic,
Raid Epic, PvP and Legendary. Existing six collection bands keep their exact indexes
and bonuses. Poor and Raid Epic deliberately have no legacy-band encoding: neither
may alias an existing ownership cell or silently shift a saved equipment choice.
Glove quality resolution now uses this shared tier mapping.

All nine Poor profiles have one primary stat below Common. Raid Epic has six lines,
strengthening ordinary Epic's five lines; initial tuning uses the existing PvP stat
budget while retaining a distinct tier identity. Legendary remains seven lines.
This is a rules foundation only: no new item is obtainable, no collection ownership
is fabricated, and the existing collection remains 54 cells per class.

Remaining integration sequence:
1. Add stable named item identities for supplemental Poor and Raid Epic catalogs,
   with class, slot, tier, verified display/icon and source references.
2. Store supplemental ownership and worn/look references separately from legacy
   band digits. Preserve legacy loadouts and the existing 54-piece collection feat;
   give supplemental raid progression its own explicit completion rules.
3. Add collection/source filters and comparison details using tier profiles;
   duplicate supplemental drops remain ownership-only, never added stats.
4. Author build-oriented raid sets with explicit 2/4/6-piece thresholds and scoped
   effects, applied only from worn pieces. Cosmetic overrides cannot count.
5. Connect verified encounter rewards and capture acquisition/equip/appearance,
   save migration and duplicate-drop behavior at wide/compact resolutions.

Regression coverage checks exact existing-band stats, distinct tier identities,
invalid slots/bands, one added stat per progression step and strictly stronger
existing lines for every slot. These checks are not catalog or gameplay acceptance.

Validation: 148 core tests, all-target core Clippy, formatting and diff checks pass.
Native play-profile build also passes; no runtime or visual session was needed for
this rules-layer foundation.


### 2026-10-26 — Verified supplemental Poor identities

The supplemental catalog now resolves all 81 Poor class/slot combinations with
stable named keys, exact template names and full visual metadata. Four armor
families share 32 items across appropriate classes; nine weapons complete the slots.
All 41 distinct templates were checked against the pinned source's Quality 0 rows.
The key parser rejects unknown classes/slots, wrong tiers and trailing components;
Raid Epic returns unresolved rather than borrowing ordinary Epic art.

Core coverage checks uniqueness, key round trips, armor class, inventory slot and
boundary rejection. The existing explicit-data application gear audit now checks
supplemental displays and readable icon files as well as the old catalog. Details
and source are in SURVIVORS-GEAR-VISUALS. Supplemental ownership/save/equipment,
acquisition, UI comparisons, rendered body acceptance and raid identities/sets
remain open. No screenshot is presented as evidence for an unrendered catalog.

Validation: 149 core tests and 292 visibility-enabled app tests pass with
`WOW_DATA=wow-data/Data` and `BENILLA_REQUIRE_DATA=1`, including the expanded gear
display/icon audit. Native build, core/app Clippy, formatting and diff checks pass.


### 2026-10-26 — Supplemental ownership persistence

Added a validated, ordered ownership set keyed by supplemental catalog identity.
`collect(class, key)` only accepts an existing piece of that class and returns true
once; duplicate grants never create ranks or stacks. Account rows use
`supplemental <class> <stable-key>`, independent of legacy grid and equipment digits.
Rows work before or after class records. Unknown keys, wrong classes, malformed IDs
and trailing fields are ignored. Old saves have an empty supplemental inventory.

Core coverage exercises all 81 keys, wrong-class attempts, duplicates, invalid keys
and deterministic ordering. Account coverage checks mixed-class ownership, duplicate
rows, row order and encode/decode persistence while preserving existing equipment,
clear flags, combat bonuses, collection feats and achievement IDs. Loading does not
mark the save dirty. Ownership alone does not equip a piece or grant stats.

Remaining: acquisition must use the ownership set and dirty/save transaction; worn
and appearance references must resolve supplemental identities with slot validation;
collection/source UI and comparisons need actual native screenshot acceptance. No
new obtainable gear or visual acceptance is claimed by this persistence checkpoint.

Validation: 150 core and 293 visibility-enabled app tests, native build, core/app
Clippy, formatting and diff checks pass.


### 2026-10-26 — Unified legacy/supplemental loadout resolver

Added a core Loadout wrapper owning legacy choices plus stable supplemental worn
and appearance references. Choosing a legacy worn piece clears that slot's
supplemental reference; choosing a supplemental piece clears its legacy worn slot.
Appearance setters similarly replace only the cosmetic choice. Worn/hidden modes
and cross-tier cosmetic overrides resolve independently from combat power.

Resolved pieces expose one name, visual and bonus. Resolution rechecks ownership,
class and slot; unowned or foreign references cannot grant power or visuals. The
combined bonus sums one item per slot and the existing full-nine-slot set bonus once.
No raid set behavior is claimed. Tests cover both override directions, empty slots,
hiding, wrong classes/slots, lost ownership, mixed-set gain/loss and exact stat
compatibility across all nine classes and six legacy bands.

This wrapper is not yet installed as ClassProgress equipment. Next integration must
replace its legacy-only equipment consumer paths, persist supplemental worn/look
references after ownership loads, and route Character/comparison/model consumers
through the resolver before enabling UI actions. Until then, no runtime supplemental
equip/appearance feature is claimed. This rules checkpoint has no visual change.

Validation: 153 core tests, native build, all-target core Clippy, formatting and
diff checks pass. No live UI or model acceptance was run for this core-only change.

### 2026-10-26 — Live supplemental loadouts

The account now adopts the shared loadout resolver for runtime stats, player
appearance and equipment previews. Stable worn/look save rows are applied after
ownership and legacy migration; malformed, unowned, foreign-class and wrong-slot
references are ignored. Existing gear digits and legacy collection completion
remain unchanged. New legacy drops do not overwrite an occupied supplemental slot.

Owned Poor pieces can be inspected, equipped, unequipped or chosen as an independent
appearance in Collection. The starter equipment strip resolves their real icons.
UI invalidation includes supplemental ownership, worn and appearance choices.

Still open: natural Poor acquisition, full catalog-row integration and completion
semantics, raid-specific identities/acquisition/set builds, and Windows in-map
appearance acceptance. This checkpoint does not grant Poor items to normal saves.

### 2026-10-26 — Poor tier in the Collection grid

Poor now occupies a nine-slot catalog row ahead of Common, using the same responsive
cells, dim uncollected art, selection borders and E/L markers as the existing tiers.
The temporary supplemental button list has been removed. Every class can inspect
its nine Poor pieces, including unowned entries; ownership gates equip/look actions.
Right-click equips or unequips owned Poor pieces; Shift-right-click selects their
appearance, matching the existing grid interaction.

The existing 54-piece count is explicitly labeled Common–Legendary; Poor has its own
row count and does not silently change collection-achievement or cloak requirements.
Natural acquisition and raid-specific catalog rows/sets still require implementation.

### 2026-10-26 — Natural Poor gear acquisition

Initial tuning gives ordinary Elwynn/Duskwood enemies a 2% aggregate Poor-drop
chance, choosing one of nine slots uniformly. Named scheduled sources retain
their existing table; bosses and elites cannot also roll this starter gear.
The shared kill handler covers direct damage and periodic kills. Newly owned
pieces fill empty slots only and use the existing discovery-save flow. Duplicates
do not add power or re-equip; existing worn gear and independent looks are retained.
Permanent equipment bonuses still apply at the next run boundary.

Collection details and Drop Tables describe the same source/rate constant.
Balancing and a supervised Windows in-map acceptance run remain open, as do
raid-specific acquisition and build sets. No new chest is spawned by Poor drops.

General piece-count and nine-slot Full set achievements include supplemental ownership;
the class cloak collection feat still requires the 54 Common–Legendary cells.

### Hunter combat companion foundation — 2026-10-26

Hunter runs now summon one Timber Wolf (CreatureDisplayInfo 604), independent of
purchased pickup companions. The wolf pursues living enemies within an 18-yard
hero leash, follows the hero otherwise, and uses the existing grounded body sweeps
and incremental local navigation. Bites require melee reach in three dimensions
and solid-world line of sight. Damage enters the same Hit pipeline as player
abilities, including kill credit, loot and the damage meter under “Hunter Pet”.

Initial authored tuning: 8 + 2 × (run level − 1) base damage, global damage
multiplier, 2-second attack cooldown, no criticals. Extra-target bonuses add nearby melee victims without increasing reach. Movement is
115% of the greater of hero speed and 7 yards/second. The pet despawns when the hero
is dead and is included in normal RunEntity cleanup. No save migration is needed.

This is a combat foundation, not completion of Beast Mastery. Pet health, enemy
threat, recovery, three BM milestones and individual pet talents remain open.
Enemies still target the hero; this wolf cannot tank. Unreachable targets are not
attacked through geometry or rescued by teleporting. Live model scale, animation,
Elwynn pathing, balance and Windows acceptance remain to be checked.

Validation: all 304 data-required Survivors app tests pass, including the wolf's
spawn/class/death lifecycle, damage-to-kill/loot integration, cooldown, extra-target
reach, rejection of bites through a solid wall and local model availability. These
headless checks do not replace live visual or Windows acceptance.

App Clippy, the incremental native build, formatting and diff checks pass.

### Beast Mastery automatic milestones — 2026-10-26

| Points / milestone | Combat wolf effect |
| --- | --- |
| 10 · Unleashed Fury | Bite damage ×1.2 |
| 20 · Frenzy | Bite cooldown ×0.8 (2 → 1.6 seconds) |
| 30 · Bestial Wrath | Bite damage ×1.5 during the first 10 seconds of each 30-second run-time cycle |

These are automatic Survivors adaptations, separate from individual DBC talent
training. Frenzy is a passive cooldown benefit, not a critical-strike proc. Wrath
starts active, ends at 10 seconds, and returns at 30 seconds; menu/pause time does
not advance its clock. The two damage bonuses multiply (×1.8 during Wrath), then
scale with the wolf's level and global damage. Shots, traps, Volley and collectible
pickup companions receive no bonus. No immunity or pet tanking is implied.

All 81 named class/tree milestones now have runtime effects and descriptions.
This count does not complete the individual talent catalog, pet health/threat,
visual combat acceptance, or class balance. Hunter pet health/recovery and the
remaining Beast Mastery individual nodes are still open.

155 core and 305 data-required app tests pass. New tests cover threshold boundaries,
periodic Wrath activation/expiry, class/tree isolation, unchanged Hunter spells,
and actual enemy health changes and bite cooldowns through the combat pipeline.

Native menu evidence: all three Beast Mastery descriptions inspected at 1280×720;
Bestial Wrath remains fully readable after scrolling at 640×720. Captures under
`docs/images/screenshots/2026-10-26-beast-mastery/` use an empty saved-spec fixture
and are embedded in CHANGELOG and the existing README talent gallery. They prove
menu presentation, not in-map pet rendering or balance. Clippy and native build
pass; a final text-only glyph/unit correction was rebuilt and visually checked.

### Individual Beast Mastery combat training — 2026-10-26

The local 1.12.1 catalog verifies Unleashed Fury 1396 (5 ranks), Ferocity 1393
(5), Frenzy 1397 (5, requires Ferocity 5), and Bestial Swiftness 1391 (1).
The existing saved-spec validator continues to enforce DBC row/prerequisite rules.

- Unleashed Fury: +4% pet damage/rank, multiplying the automatic damage milestones.
- Ferocity: +2 percentage points pet crit/rank, added to global critical chance.
  Wolf bites now use global critical damage; each extra melee victim rolls separately.
- Frenzy: 20% proc chance/rank on positive-damage pet criticals for 30% bite haste
  over 8 seconds. Repeat procs refresh to 8 seconds without stacking. The triggering
  bite starts the shortened cooldown; an already-running cooldown is not rescaled
  when the buff expires. Pause/selection screens freeze the timer.
- Bestial Swiftness: +30% grounded pet movement speed, leaving melee reach and the
  18-yard hero leash unchanged.

These pet-only nodes replace their temporary per-point global damage fallback;
Hunter spells and pickup companions gain none of the explicit pet bonuses. Full
training and damage milestones produce ×2.16 bite damage during Wrath. The automatic
Frenzy milestone and active proc combine to a 1.6 / 1.3 ≈ 1.231-second bite cooldown.
Pet health/threat, other pet nodes, live rendering/pathing and balance remain open.

Validation: 157 core and 307 data-required app tests pass. The new checks cover
rank bounds/reset, composition and isolation, Frenzy proc thresholds and nonstacking
refresh/expiry, deterministic real critical damage and cooldowns, and actual
supported movement with/without Bestial Swiftness. No live balance claim is made.

The local DBC coverage audit reports 196/432 individual nodes and 81/81 milestone
definitions. Clippy, native build, formatting and diff checks pass. Native menu
screenshots under `docs/images/screenshots/2026-10-26-beast-training/` show the valid
30-point seeded spec, Frenzy's next-rank chance and Ferocity's maximum-rank value.
The compact capture verifies the full Frenzy text after scrolling. Screenshots are
embedded in CHANGELOG and the existing README talent gallery; they do not certify
in-map animation, live pathing, Windows presentation or combat balance.

### Combat wolf health and recovery — 2026-10-26

The wolf now has 60 + 6 × (run level − 1) maximum health. Endurance Training
(DBC 1389, five ranks) multiplies that by 1 + 0.03 × rank. Level growth preserves
health percentage instead of fully healing an injured wolf.

Non-boss enemies choose a living wolf only when it is closer than the hero,
within four yards, and visible through the solid-world line check. Bosses keep
attacking the hero. This is proximity targeting, not a complete threat table or
an Intimidation taunt. Pet-target changes discard stale hero routes. Contact uses
three-dimensional distance, so actors on different floors cannot melee each other.
Enemy attack cooldowns remain shared across target switches. Pet damage does not
consume hero health, shields or trigger hero Thorns.

At zero health the wolf plays death, stops moving/attacking and loses Frenzy.
After ten seconds of run time it is replaced at the hero's supported position;
unsafe positions defer recovery. A new entity restores full health and clears
combat state. No healing, pet armor, resurrection spell, aggro persistence or
boss taunting is implied. The HUD shows current/max wolf health or recovery time.
Pause and selection screens do not consume the recovery timer.

Validation: 157 core and 311 data-required app tests pass, including nearby enemy
damage, boss exclusion, shared cooldown on target switch, no hero Thorns from pet
damage, defeat/recovery, HUD text, and level-growth health fractions. Clippy,
formatting and the native play build pass. DBC coverage is 197/432 individual
talents and 81/81 automatic milestones.

Native menu captures at 1280×720 and 640×720 show Endurance Training's 12 to 15%
preview in a valid 29-point spec. They are embedded in CHANGELOG and README.
The in-map HUD, wolf death animation and Windows combat behavior still require
live acceptance; headless tests and menu screenshots do not establish those.

### Hunter pet sustain — 2026-10-26

Implement DBC nodes 1625 (Improved Revive Pet) and 1388 (Spirit Bond), each with
two ranks. Recovery is snapshotted from the run's spec when the wolf spawns,
reducing ten seconds by 20% per rank. Spirit Bond restores 1% of each recipient's
own maximum health per rank at ten-second living-wolf intervals. Long frames
retain pulse count and remainder; health caps prevent overhealing. Death clears
the interval, recovery creates a fresh timer, and the Playing system gate freezes
it during pauses and selections. These effects replace their placeholder global
damage contribution. Full pet threat, remaining talents and live acceptance stay
open.

Validation: 158 core and 312 data-required app tests pass, covering rank bounds,
class isolation, actual dual healing, interval boundary, overheal caps, defeated
pet inactivity and six-second trained recovery. Clippy, formatting and native
build pass. The local DBC audit reports 199/432 individual talents and 81/81
automatic milestone definitions. Native previews show both new talents in a
valid 32-point spec, including the full compact Spirit Bond description after
scrolling. Captures are embedded in CHANGELOG and README; live combat and
Windows presentation remain unverified by these menu checks.

### Intimidation — 2026-10-26

DBC node 1387 now empowers one nearest wolf bite every 60 seconds: three seconds
of stun through the normal Hit pipeline, and six seconds of forced pet attention
for ordinary enemies. Extra targets still take normal bites. Bosses receive the
existing 0.3 stun multiplier and never forced attention. The cooldown lives on
Run, so death and recovery cannot refresh it. A taunt references the exact pet
entity and expires against run time; a dead or replaced wolf invalidates it.
Forced targets continue using grounded route planning around intervening solids.
This is an authored Survivors automatic ability, not a complete threat table.

Validation: 158 core and 315 data-required app tests pass. Tests cover one-target
stun versus extra bite targets, forced attention despite a nearer hero, expiry,
dead-wolf fallback, boss exclusion and reduced stun, plus cooldown persistence
through recovery and reactivation at 60 seconds. Clippy and formatting pass.
The DBC audit reports 200/432 individual talents and 81/81 milestone definitions.

Native play build passed. Captures at 1280×720 and 640×720 show the full
Intimidation description and aligned tree in a valid 33-point spec. Screenshots
are embedded in CHANGELOG and the existing README gallery. These menu checks
do not establish live combat or Windows gameplay acceptance.

### Trained Bestial Wrath — 2026-10-26

DBC capstone 1386 upgrades the existing automatic 30-point Beast Mastery window
to 18 seconds per 30-second run-time cycle and a 2× bite multiplier. It replaces
the original 10-second / 1.5× window; it does not stack a second Wrath bonus.
Unleashed Fury and individual damage training remain multiplicative. The saved
talent tree enforces its Intimidation prerequisite and 30 earlier points. Core
execution also requires the 30-point milestone, rejects invalid run times, and
keeps hero spells and pickup companions unchanged. No immunity is implied.

Validation: 159 core and 316 data-required app tests pass. Core checks cover
window boundaries, invalid time, rank bounds, reset, milestone gating and hero
spell isolation; actual bite tests confirm 19.2 damage at 10/17.999 seconds,
9.6 at 18 seconds, and 19.2 again at 30 seconds with baseline level-one stats.
Clippy, formatting and native build pass; coverage is 201/432 individual talents
and 81/81 automatic milestones. Native 1280×720 and 640×720 captures show the
capstone in a valid 34-point saved spec, embedded in CHANGELOG and README.
Live combat and Windows behavior remain separate acceptance work.

### Poor Collection fast previews — 2026-10-26

Collection preview requests now accept `piece = poor:<slot>` alongside the six
legacy qualities. Poor selections resolve through the selected class's actual
supplemental catalog; all nine visible slots are supported. Applying a legacy
rarity clears supplemental inspection, while omitting the piece preserves the
current inspected item. Invalid slots, unsupported rarities and use on another
page still reject the request. Preview changes do not mutate ownership, worn
gear, appearances or saves. Reusable wide/compact requests are checked in under
`docs/ui-previews/collection-poor-hands-*.txt`.

Validation: 317 data-required app tests pass, including all 81 Poor class/slot
resolutions, omitted-item retention and switching back to legacy inspection.
Clippy, formatting and native build pass. Native checks verified Poor hands,
Epic hands, and invalid `poor:off hand` retaining both selection and window size.
The full compact item detail is reachable by scrolling. The isolated account
file's hash remained unchanged after quit. Wide/compact captures are embedded
in CHANGELOG; README's Collection image is refreshed. The three-column wide
layout follows the Collection intention; the extra Poor row and stacked compact
layout are intentional extensions. This does not close the full page/platform
acceptance matrix or add unimplemented Raid Epic content.

### Consistent Collection item details — 2026-10-26

Poor items now use the same proportional bordered item header as Common through
Legendary: icon beside the name, quality/slot and collection status. This follows
the Collection intention's compact header hierarchy. Sources precede equipment
comparisons, comparisons name equip versus unequip, and appearance controls use
a labeled section with wrapping buttons in the same order as other rarities.
The Poor row and ordinary-enemy source text remain intentional extensions. No
item stats, ownership, equipment actions or appearance semantics change.

Validation: all 317 Survivors app tests, scoped Clippy, native play build and
formatting pass. Native menu fixtures at 1280×720 and 640×720 show readable
headers, sections and appearance actions. Hidden/This piece toggles update the
look marker while preserving the equipment marker and +8 health. Screenshots
are embedded in the changelog and the README Collection cell is refreshed.
This is menu validation on macOS, not in-world or Windows acceptance.

### Collection secondary navigation — 2026-10-26

Backspace/controller west on a focused Collection item now toggles equipment
through the same ownership checks as right-click, including Poor items. Talent
refunds keep their existing shortcut. Purchases, navigation and appearance buttons
cannot be activated by this secondary gesture. Appearance remains independently
selected through the Look controls or Shift-right-click. The Collection help
names the new shortcut. Physical-controller acceptance remains open.

Native checking exposed duplicate inspection actions in the collection grid and
equipped sidebar. Grid controls now carry a distinct navigation identity so the
focus is restored to the grid after an equip rebuild, including the layout delay.
A regression test covers duplicate actions, delayed layout and one-frame requests.

Validation: 319 Survivors app tests, scoped Clippy, native play build, formatting
and diff checks pass. Native macOS menu checking rejected an unowned Poor item,
then toggled gloves off/on with Backspace: +8 health disappeared/reappeared,
the chosen look remained selected, and the final build retained grid focus with
the equipped sidebar duplicate present. Compact 640×720 help wraps cleanly.
Screenshots are inline in CHANGELOG and the README Collection image is refreshed.
No physical controller or Windows runtime claim is made by this checkpoint.

### Collection inspection follows navigation — 2026-10-26

Keyboard/controller focus now updates the Collection inspector for both Poor and
standard gear. Players can read an item's bonuses, source and ownership before
activating equip. Moving focus does not dispatch an action or write account state;
modals still suppress navigation. The existing rebuild regression also checks
Poor-to-standard inspection and absence of activation/refund requests.

Validation: all 319 Survivors app tests pass; scoped Clippy passes after an
expression-only match-guard cleanup; native play build and formatting pass.
Native 1280×720 navigation showed Poor shoulders, Common shoulders, then an owned
Poor helm with the matching descriptions, sources and controls. Equipped gloves
and their +8 health stayed unchanged; the account file SHA-256 before launch and
after quitting matched. Screenshots are inline in CHANGELOG and the README
Collection image is refreshed. Physical controller and Windows acceptance remain open.

### Stable starter backdrop — 2026-10-26

The main-menu intention uses a warm distant mountain scene. The previous live
orbit around the hero put nearby trees/buildings behind the interface and changed
composition while reading. Starter/Armory now uses an original procedural sunset
and layered mountain silhouettes, drawn in normalized UI coordinates. It is a
stylized interpretation, not extracted art or a screenshot of an authored zone.
The legacy body-selection screen retains its world preview; gameplay and camera
collision behavior are unchanged. Menu visibility owns the backdrop, so it does
not cover gameplay or level-up previews. Outdoor zone illustrations remain open.

Validation: scoped Clippy, native play build, formatting and diff checks pass.
Native macOS 1280×720 and 640×720 previews confirm proportional coverage and readable
panels. Zone selection works; the Character portrait remains visible; switching to
the legacy Look page restores the live world and returning restores the backdrop.
The runtime log has no shader errors. Three screenshots accompany CHANGELOG and the
README starter/character images are refreshed. No gameplay, Windows or full test-suite
acceptance is claimed for this presentation-only checkpoint.

### Pre-run zone briefings — 2026-10-26

Each of the twelve campaign zones now has an authored title and briefing, separate
from the map's victory text. The starter shows the title, threat and encounter
roster for the selected zone, while preserving lock reasons and the schedule-derived
final fight under Enter. Battleground briefings describe faction-themed survival
assaults; Blackrock explicitly remains in preparation. Non-campaign selections no
longer display victory text before play either. This adds presentation copy, not
zone readiness, bosses, loot or progression changes.

Validation: 159 core tests, scoped app Clippy, native play build and formatting pass.
Native 1280×720 and 640×720 screenshots verify briefing wrapping and Duskwood lock
messaging; no run was entered. CHANGELOG embeds all three captures and README
refreshes the Starter gallery. Windows and gameplay acceptance remain open.

### Starter campaign stage indicators — 2026-10-26

The intention reference's twelve-marker zone headers now show campaign order:
one lit marker for Elwynn through twelve for Naxxramas. A visible “Stage X of 12”
label disambiguates order from clears, unlocks or selectable difficulty. Headers
wrap at narrow widths and keep existing lock reasons and selected-zone briefings.
This does not change readiness, progression requirements or encounter tuning.

Native review exposed that wide starter panels had scroll overflow but no wheel
handler components, leaving later zone rows clipped. All three wide columns now
use the shared wheel handler and keep separate in-session offsets through page
rebuilds. Compact starter scrolling retains its own offset. These are view offsets,
not saved character/account changes.

Validation: 319 app tests including independent scroll memory, scoped app Clippy,
native play build and formatting pass. Native 1280/1100/640×720 checks cover stage
marker clicks, late-zone selection without returning to the top, existing lock
messaging and independent hero scrolling. Four screenshots are embedded in
CHANGELOG; README Starter is refreshed. Windows acceptance remains open.

### Visible starter scrollbars — 2026-10-26

Wide starter columns now show fixed gold scrollbars only when their content
exceeds the viewport. Thumb length reflects the visible fraction; dragging keeps
the initial grab offset, and clicking the track centers the thumb there within
scroll bounds. The ordered window-event stream preserves complete short drags
within one frame and releases on mouse-up, focus loss or cursor exit. Existing
wheel scrolling and independent column memory remain in use. Unchanged geometry
and colors are not rewritten while idle. Compact mode retains its existing single
wheel-scroll surface; a compact scrollbar and wider Armory adoption remain open.

Validation: 321 app tests, scoped Clippy, native build and formatting pass. Native 1280×720
and 1100×720 checks cover bidirectional dragging, track clicks, hidden fitted-column
bars and independent hero scrolling. Windows and compact scrollbar acceptance
remain open.
Two captures are embedded in CHANGELOG and the README Starter gallery is refreshed.

### Compact starter scrollbar — 2026-10-26

The compact starter now uses a fixed scrollbar beside its single stacked-panel
viewport, with a reserved gutter so it does not cover content. Wide columns keep
independent scrollbars and offsets. Native 640×720 checks verify dragging to both
ends, selecting Naxxramas with its existing lock message, and preserving compact
scroll position through a 1280×720 transition and back. Scoped Clippy, native build
and formatting pass; the unchanged input handler retains the prior 321-test
checkpoint, without claiming a fresh test-suite run. Two screenshots are embedded
in CHANGELOG and linked from the README gallery. Other Armory scrollbars and
Windows acceptance remain open.

### 2026-10-26 — Armory scrollbars

Extended the shared framed column to attach a fixed scrollbar sibling and reserve
a right gutter. This covers wide Collection, Drop Tables, Character and Codex.
Compact Collection/Drop Tables/Character now share a viewport wrapper with the
same scrollbar and retain their existing page-specific scroll positions.

Native menu validation at 1280×720 and 640×720 confirms wide Collection equipment
and compact Collection bottom reachability, compact Drop Tables detail access,
compact Character look controls, and fitted Codex columns without unnecessary bars.
Three inline CHANGELOG captures document the checkpoint; README links the compact
examples. Clippy, play build (24 seconds), formatting and diff checks pass. No
input logic changed and no app-suite rerun was needed after the prior 321-test pass.
Wide-column offsets still reset on rebuild; full resolution/controller/Windows
acceptance and other remaining interface requirements are not closed by this work.

### 2026-10-26 — Independent Armory column positions

Resolved the wide-column reset noted in the previous checkpoint. Collection (three),
Drop Tables (four), Character (three) and Codex (two) columns restore independent
in-session positions through the existing pre-rebuild memory system. Compact page
offsets remain separate; existing post-layout clamping handles shorter content.

322 app tests pass, including rebuilding a scrolled column without overwriting the
compact offset. Native macOS Collection verification selected Poor gloves after
scrolling equipment to the bottom, then switched to Drop Tables and back; the
equipment offset survived both rebuilds. Screenshot embedded in CHANGELOG and
README Collection refreshed. App Clippy, native build, formatting and diff checks
pass. This is menu-state evidence, not Windows/controller or complete UI acceptance.

### 2026-10-26 — Compact Codex reading layout

Compared the live 640×720 Codex against the wide intention reference. The fixed
74/26 horizontal split squeezed descriptions into a narrow sidebar. Under the
existing 1100-logical-pixel compact threshold, Codex now uses a 62/38 vertical split,
keeping both independently scrollable panels present. Compact detail art sits beside
the text; wide presentation remains side-by-side. Added Codex to the responsive
rebuild signature and allowed shared columns to shrink vertically.

Native macOS checks cover Arcane Shot's legendary description, independent catalog
dragging to the final shared rows, Eye of the Beast selection and 640→1280→640 resizing
with selected details retained. Three inline changelog screenshots and the README
Codex preview record this intentional responsive departure from the wide reference.
Clippy, native build, formatting and diff checks pass. No new gameplay or discovery
logic changed and the app suite was not rerun. Catalog expansion and full resolution,
controller and Windows acceptance remain required.

### 2026-10-26 — Holy Nova individual talent

Implemented Priest Holy node 442 as an authored Survivors conversion: +5% maximum
health healed once per successful Holy Nova cast, additive with the existing Holy
30-point milestone and then multiplied by Spiritual Healing. The talent does not
grant the run spell, change damage/lifesteal or multiply healing by targets hit.

160 core tests and 322 app tests pass. Coverage includes rank caps, other-class and
other-spell isolation, removing ranks, milestone stacking, empty casts and multiple
targets through the real cast system. Native macOS 1280×720 preview confirms buying
the talent, the persisted rank and readable supported description. Screenshot is
embedded in CHANGELOG and README. Clippy/native build/fmt pass. The DBC audit now
reports 202/432 individual definitions and 81/81 milestones; remaining talents and
Windows/live balance acceptance are still open.
