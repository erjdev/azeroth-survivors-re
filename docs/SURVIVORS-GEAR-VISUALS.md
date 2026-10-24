# Gear visual identities

`survivors-core::gear_visuals` maps named gear to vanilla item entry, display ID and
inventory type, class/subclass, material and sheath metadata. Gameplay stats and rarity remain Survivors rules; they are not imported
from the vanilla templates.

Source: [MaNGOS Zero item templates, pinned revision 325948a78015794e6cc626212bc19fee70c805b2](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/item_template.sql).
Mappings use exact, unique names and the `entry`, `displayid`, `InventoryType`, `class`,
`subclass`, `Material` and `sheath` columns.
Only these factual fields are recorded; the SQL download stays in ignored validation
storage. Runtime icons, textures and models come from the user's data folder.

- Correct vanilla names: **Field Plate Vambraces**, **Drake Fang Talisman**. Save slots are
  numeric, so these label corrections do not alter collection ownership.
- **Atiesh** is deliberately unresolved: the source contains four equippable variants and
  a non-equippable entry with the same name. Each class needs a verified explicit entry.
- Unnamed rarity cells remain unresolved. An absent mapping must never borrow another
  item's visual.
- The catalog includes the existing auxiliary item names, but does not make them
  obtainable or change the nine-slot equipment model.

Visual-only local item templates feed the existing player compositor and attachment
resolver. Their combat stats are empty: Survivors equipment remains the source of power.
The local hero retains its GUID while using player rendering and its preset face/hair.

Validation checks all mapped display rows and icon files in the explicit 1.12.1 data
chain. Live checks on a seeded human male account cover plate armor, hidden helmet/chest,
and a Thunderfury appearance over an equipped Whirlwind Axe, including world entry.
Other races, sexes, ranged weapons and the complete catalog still need visual checks.
The Survivors preview fits rigid attachment bounds while retaining the authored body camera
as its closest view. Thunderfury and Whirlwind Axe were checked at 1440×900 and 1280×720,
with a fixed first-load appearance replacement at 1280×720. Pending attachment loads retry until the
renderer records them ready. Weapon poses remain open; icon availability alone does not
establish correct armor textures or attachments.
