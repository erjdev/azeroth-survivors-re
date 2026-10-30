# Current Survivors acceptance status

Reviewed 2026-10-30 against runtime `7d4959a` and documentation `6dcf009`.
This is an index into the [seven-phase contract](SURVIVORS-IMPLEMENTATION.md), not
a replacement scope or a completion declaration. Historical checkpoints describe
their own dates; later evidence can supersede their pending notes.

## Immediate order

1. Finish starter/Armory behavior, persistence and responsive acceptance, including
   working talent effects. Use the [intention screenshots](intentions/) as the
   design reference and production UI previews for iteration.
2. Resolve the outstanding gameplay choices below and implement their actual
   consumers before counting associated talents as complete.
3. Complete remaining world/content and integration acceptance without bypassing
   readiness locks. Further zone expansion follows the interface pass.
4. Review and merge the current PR only when its acceptance boundary is satisfied.
   Box3D belongs in a separate post-merge effort. Fast mode is also later work.

## Seven phases: evidence and remaining proof

| Phase | Existing evidence / implementation | Still required for acceptance |
| --- | --- | --- |
| 1. Progression | Selected-hero settlement, idempotence, shared schedules and drop odds have runtime regressions. | Attended start/death/win/abandon and restart flows; all advertised encounters and final drops in every ready zone. |
| 2. Equipment | Selected loadouts and independent appearances persist; duplicates grant no power; malformed/legacy/save-failure cases have tests. Native purchase/appearance/restart examples exist. | Remaining platform/body cases, full-set and raid-set effects through gameplay, full gate before landing. |
| 3. Character and Collection | Responsive production pages, owned/empty/locked states, source navigation and comparisons; latest [starter inspection acceptance](audits/STARTER-EQUIPMENT-ACCEPTANCE.md). | Remaining platform/resolution/body matrix and complete art/content correspondence, not just populated cells. |
| 4. Twelve zones / gear | Current content audit finds names and drops for every class's 54 ordinary collection cells. | Three audit gaps below, then traversal, encounters, world identity, icons/models, raid sets and attended readiness validation. |
| 5. Talents | 309/432 named definitions; all 81 milestone descriptions. Purchases/refunds/spec snapshots have regression and selected native evidence. | 123 nodes still provisional, plus actual consumers and acceptance for each implemented node. Definitions/descriptions alone are not playable completeness. |
| 6. Codex / achievements / rewards | Seen-item persistence, catalog counts, reward drafts/refunds and independent cosmetics implemented; responsive native examples recorded. [Companion fetch routing](audits/COMPANION-FETCHING.md) has fence/return and blocked-collection regressions; [adaptive reward cards](audits/COMPANION-READABILITY.md) have three native layout captures. | Complete perk/companion gameplay and platform acceptance; catalog totals, settlement and persistence end to end. Placeholder achievement criteria deliberately remain unearnable. |
| 7. Integration / visuals | Native production-page previews, retained build caches, screenshot reporting and scoped gates; synthetic camera/silhouette evidence. | Attended game loop, real foliage/interior camera cases, Windows and physical-controller checks, full land gates and all reference-page acceptance. |

## Current audit results

Reproduce with the checkout data only:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-gaps
cargo run -p survivors-audit -- wow-data/Data
```

| Class | Matching talent definitions / selectable nodes |
| --- | ---: |
| Warrior | 42/52 |
| Paladin | 28/44 |
| Hunter | 42/46 |
| Rogue | 38/51 |
| Priest | 31/47 |
| Shaman | 33/46 |
| Mage | 38/49 |
| Warlock | 29/50 |
| Druid | 28/47 |
| Total | 309/432 |

The current content audit fails with exactly these three reported gaps:

- Blackrock Mountain: no configured map entry. The [source-face audit](SURVIVORS-LOCATIONS.md#blackrock-source-face-connectivity--2026-10-30) now identifies conservative connected components; clearance and routing remain open.
- Scarlet Monastery: start-floor validation still needed; ADT terrain returns 0.
- Naxxramas: start-floor validation still needed; ADT terrain returns about 124.75.

The [body-art audit and native follow-up](audits/COLLECTION-BODY-ART.md) now pass
646/646 variants using 746 armor textures. A narrow Survivors fallback repairs
Virtuous Robe’s absent male lower sleeve; Human male/female previews are verified.
This does not establish the full race/platform/equipment appearance matrix.
Attachment source assets now also pass 1,055 variants across 869 models and 221
textures, including all race/body helmet suffixes. This checks geometry and
material availability; rendered attachment alignment remains unverified.

These results do not imply every other zone is ready. ADT checks cannot prove WMO
interior support, traversability or encounters. All nine classes report zero
unnamed cells and zero cells without drops; this does not verify every icon,
appearance, creature identity, set bonus or reward delivery.

## Unanswered gameplay choices

These are pending design decisions already raised in the conversation. An
approved conversion for one spell does not authorize applying it to all others.

| Choice | Work it governs |
| --- | --- |
| Druid form selection: starter, run cards, or manual switching | [Form lifecycle and dependent talents](audits/DRUID-TALENT-DEPENDENCIES.md); six faction models are visually verified. |
| Warlock demon selection before a run versus a summon ability slot | [Demon acquisition and dependent talents](audits/WARLOCK-DEMONS.md). |
| Clearcasting's proposed next-damaging-spell recovery refund | [Mage proc/consumption integration](audits/MAGE-TALENT-DEPENDENCIES.md); keep the 15-second one-use window. |
| Ancestral Healing: replace, combine with, or retain the existing authored armor effect | [Shaman critical-heal armor behavior](audits/SHAMAN-HEALING-DEPENDENCIES.md). |
| Stormstrike weapon-damage input | [Shaman melee attack and vulnerability application](audits/SHAMAN-MELEE-DEPENDENCIES.md). |
| Disarmed enemies stop weapon attacks or retain an unarmed attack | [Enemy equipment and disarm consumers](audits/ENEMY-EQUIPMENT.md). |
| Paladin basic physical attack outside slots versus acquired attack | [Weapon-scoped talents and seals](audits/PALADIN-WEAPON-TALENTS.md). |
| Tranquility movement and attack handling | [Channel lifecycle and healing](audits/TRANQUILITY.md). |

Resolved rules remain resolved: Stoneclaw forces ordinary captured enemies to
attack it while alive; bosses are immune. Illumination refunds half the effective
critical heal's recovery on its rank-scaled proc. Battlegrounds retain horde
survival with faction-themed encounters. Gear duplicates give no power; higher
rarities add/strengthen stats. Extra targets affect single-target attacks only.

## Reporting and verification

Keep feature commits and pushes on the feature branch. Embed native screenshots
for visual milestones in [CHANGELOG](CHANGELOG.md), the README and PR; store them
under `docs/images/screenshots/`. No UI change means no invented screenshot.
Run `cargo survivors-build` and the scoped `scripts/check.sh` for source changes;
run full land gates before merging. The runtime gate at `7d4959a` and subsequent floor-connectivity audit gate passed all
eight scoped packages with 31 optional-addon skips; that is not Windows, attended
combat, or full-workspace land acceptance. Model and UI previews use isolated
profiles and checkout data, never the installed Applications game.

The Virtuous Robe appearance checkpoint also passes the eight-package scoped
gate (31 optional-addon skips), with native Human male/female screenshots linked
in the body-art audit. Other body/platform cases remain open.
