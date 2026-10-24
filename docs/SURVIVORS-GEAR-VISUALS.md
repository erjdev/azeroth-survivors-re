# Gear visual identities

`survivors-core::gear_visuals` maps named gear to vanilla item entry, display ID and
inventory type. Gameplay stats and rarity remain Survivors rules; they are not imported
from the vanilla templates.

Source: [MaNGOS Zero item templates, pinned revision 325948a78015794e6cc626212bc19fee70c805b2](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/item_template.sql).
Mappings use exact, unique names and the `entry`, `displayid`, and `InventoryType` columns.
Only these factual identifiers are recorded; the SQL download stays in ignored validation
storage. Runtime icons, textures and models come from the user's data folder.

- Correct vanilla names: **Field Plate Vambraces**, **Drake Fang Talisman**. Save slots are
  numeric, so these label corrections do not alter collection ownership.
- **Atiesh** is deliberately unresolved: the source contains four equippable variants and
  a non-equippable entry with the same name. Each class needs a verified explicit entry.
- Unnamed rarity cells remain unresolved. An absent mapping must never borrow another
  item's visual.
- The catalog includes the existing auxiliary item names, but does not make them
  obtainable or change the nine-slot equipment model.

Validation checks all mapped display rows and reads their icon files from the explicit
1.12.1 data chain. This establishes icon availability, not armor texture/attachment
correctness: world and paper-doll dressing still require implementation and visual checks.
