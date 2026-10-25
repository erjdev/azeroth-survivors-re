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

Warrior individual talents now use the same selected-spec snapshot and comparison path.
The following audited nodes replace their provisional +1% damage per rank; spent points
still count toward existing tree milestones. These are Survivors conversions, not claims
that rage, parry, or the original MMO talent behavior exists. Unsupported individual nodes remain unfinished; all nine named Warrior milestones
have implementations described below.

| Talent ID | Name | Ranks | Survivors effect per rank |
| --- | --- | --- | --- |
| 124 | Improved Heroic Strike | 3 | Heroic Strike cooldown shortened by 5% |
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

Current Codex audit: class cloak/achievement checks incorrectly compare raw seen-name counts
with the entire ability catalog. Count unique, recognized legendary amulets from that class’s
start/pool instead, and share the same catalog with the visible grid. Duplicate or foreign
save entries must never advance completion. Shared jewelry rows currently display full
counts without discovery tracking; persist account-wide discovery when cards are dealt,
including abandoned runs, while keeping the jewelry’s effects run-only. Replace the text
chips with verified icons and a responsive selected-item detail pane. Match the reference’s
class-colored labels, amulet artwork with small spell badges, shared jewelry rows and
bottom discovery total/bar. Keep counts tied to implemented offers rather than copying
the screenshot’s larger catalog totals.

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
