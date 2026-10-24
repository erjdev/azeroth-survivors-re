# Changelog

Feature milestones, newest first. These are development checkpoints, not released versions.
Detailed changes remain in Git; unfinished work remains in the [implementation plan](SURVIVORS-IMPLEMENTATION.md).

## 2026-10-24 — Collection artwork

Collection cells now show implemented item artwork, dim unowned pieces, and keep E/L badges
in the corner. Content without verified artwork retains a placeholder.

**Validation:** native build and 71 focused tests passed. Live checks covered owned icons,
combined E/L badges and legendary selection at 1440×900, plus unowned artwork at 1280×720.
Existing terrain formatting still blocks the full repository check.

![Collection artwork](images/screenshots/2026-10-24-collection-icons/collection.jpg)
[Unowned pieces at 1280×720](images/screenshots/2026-10-24-collection-icons/unowned-720.jpg)

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
