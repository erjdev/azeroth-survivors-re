# Changelog

Feature milestones, newest first. These are development checkpoints, not released versions.
Detailed changes remain in Git; unfinished work remains in the [implementation plan](SURVIVORS-IMPLEMENTATION.md).

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

## 2026-10-24 — Green talent upgrade previews

Supported talent effects now appear in green beneath white name/rank headings. Each node
shows its current contribution and next-rank value, or its final value at maximum rank.
Milestone effects also use green; unfinished effects retain a clear pending notice.
Comparisons use font-compatible text and distinguish node bonuses from total character stats.

[Green rank preview at 720p](images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg).
**Validation:** native build, workspace test compilation, 85 Survivors tests and 51 core
tests pass. Live 720p checks confirm the comparison, green effect text, white heading and
color reset for an unfinished talent.

## 2026-10-24 — Warrior defensive talents

Four more individual talents replace placeholder damage: Improved Revenge increases
Retaliation damage, Improved Shield Wall extends it, and Last Stand/Bloodthirst add capped
healing to Retaliation/Execute. Node healing combines with milestone healing once per cast.
A longer Retaliation also extends Death Wish's incoming-damage penalty.

[Defensive talent details at 720p](images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg).
Coefficients remain provisional; remaining talents and manual combat validation are pending.
**Validation:** native build, workspace test compilation, 85 Survivors tests, 50 core tests,
and the local talent-data audit pass. Improved Shield Wall details verified at 720p.

## 2026-10-24 — Protection milestones

Last Stand heals on Retaliation, Concussion Blow adds a brief Heroic Strike stun, and
Shield Slam grants a small absorb once per Heroic Strike cast. Healing caps at maximum
health; absorbs do not stack or refresh a stronger shield. All nine Warrior milestone
behaviors are implemented; remaining talents and manual combat validation are unfinished.

[Protection details at 720p](images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 84 Survivors tests and 50 core
tests pass. Live 720p description verified. Coefficients remain provisional.

## 2026-10-24 — Fury milestones

Piercing Howl adds a slow to Demoralizing Shout. Death Wish boosts Retaliation with an
incoming-damage tradeoff while active. Bloodthirst heals once per successful Execute cast.
The implementation plan records the provisional coefficients and how these compose with
existing effects.

[Fury details at 720p](images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 83 Survivors tests and 49 core
tests pass; the Death Wish description fits at 720p. A full manual Fury encounter is still
unverified. Protection milestones and remaining individual talents are unfinished.

## 2026-10-24 — Arms milestones and reliable DoT ticks

Warrior Arms now gains Deep Wounds at 10 points, a larger Sweeping Strikes aura at 20,
and stronger Mortal Strike at 30. The bleed refreshes without stacking or postponing its
next tick. Damage-over-time catches up across long frames and stops at expiry.

[Arms milestone details at 720p](images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 82 Survivors tests and 48 core
tests pass. Live 720p description checked; a full manual Arms encounter remains unverified.
Fury/Protection milestones and remaining individual talents are still in progress.

## 2026-10-24 — Warrior talent effects

Twelve audited Warrior talents now improve specific attacks, shout radius, critical stats,
armor or damage reduction. Their effects replace placeholder damage, follow the selected
hero/spec, and appear in talent details. Other Warrior nodes and milestone behaviors remain
unfinished. The save format is unchanged.

**Validation:** native build, workspace test compilation, 79 Survivors tests, 47 core rules
tests, and the checkout talent-data audit pass. Live Heroic Strike and Booming Voice details
fit at 720p. [Milestone screenshot](images/screenshots/2026-10-24-warrior-talents/talents-720.jpg).

## 2026-10-24 — Spell-specific legendary bonuses

Legendary amulets now enhance only their associated spell. Green comparisons name that
spell; replacements show lost bonuses in red. Extra targets remain limited to eligible
single-target attacks, with no additional AoE locations.

[Single-target bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-direct-720.jpg) ·
[Area bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-area-720.jpg).
Live 720p checks confirm both descriptions. Native build, workspace test compilation,
77 Survivors tests and 45 core rules tests pass. Bonus values remain provisional.

## 2026-10-24 — Codex effect descriptions

Selected jewelry shows its implemented standalone bonuses in green, using the same rules
as chest comparisons. This checkpoint exposed global legendary bonuses; the later scope correction above fixes them.
**Validation:** native build, workspace test compilation and 75 Survivors tests pass.
[Live 720p capture](images/screenshots/2026-10-24-codex-discovery/codex-effects-720.jpg)
confirms green ring bonuses match the rules.

## 2026-10-24 — Shared jewelry artwork

Codex rings, trinkets and amulets now use compact icon cells, dimmed until offered.
Thirty-eight items have verified display mappings from the pinned vanilla catalog. Mark of the Champion
remains a selectable question mark because its name identifies two different displays.
**Validation:** native build, workspace test compilation and 74 Survivors tests pass,
including all mapped icon files in the explicit data chain. The
[720p capture](images/screenshots/2026-10-24-codex-discovery/codex-jewelry-720.jpg) shows
the compact grid; mapped and unresolved cells both open their details.

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
