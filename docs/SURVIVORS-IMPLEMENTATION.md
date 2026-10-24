# Survivors implementation contract

This plan implements [the Armory specification](SURVIVORS.md) and its
[intention images](intentions/). It is a work breakdown and acceptance contract, not a
replacement design or a claim that the illustrated game already exists. Git records
completed changes. Each phase must pass its acceptance criteria before it is called done.

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

The collection currently names nine common items and a legendary main hand per class.
The other bands are incomplete. Strongest-owned gear supplies stats automatically; there
is no independently chosen worn loadout and appearance loadout. Collection and drop pages
remain predominantly text. Most campaign zones lack complete verified content. Talent
points feed generic tree bonuses rather than the promised individual spell behaviors.
A visible control or stored field alone is not a completed feature.

Two correctness gaps precede expansion: settlement credits the first hero of a class
instead of the selected hero, and displayed drop times are disconnected from the periodic
boss scheduler (the advertised 16-minute drop exceeds the 15-minute victory timer).

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
coverage in the 57-test Survivors suite. Live multi-hero switching still needs verification.
The current hero uses the stock player compositor and attachments for selected armor,
weapons and independent cosmetic overrides in both the world and full-body preview.
The [gear visual catalog](SURVIVORS-GEAR-VISUALS.md) maps 91 distinct existing item names
to vanilla identities and supplies item-detail icons; Atiesh variants and missing rarity
cells remain unresolved. At 1440×900, a seeded human male account verified plate armor,
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

## Phase 4 — all twelve zones and gear content

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
and vertical jump ceilings need a separate acceptance check.

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

Level-up/chest cards show green before/after scaling values from the casting parameters;
new abilities and passives also preview their next rank. Jewelry compares the resulting
worn loadout, including red replacement losses and a neutral no-change state. Next-rank
hover text uses green as well. Validate every choice family at compact and wide sizes;
current live evidence covers level-up cards at 1440×900, with jewelry replacement covered
by rules tests. This does not change balance or imply missing item effects are implemented.
