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
- **Atiesh** uses explicit class reward entries (below). A name-only lookup stays
  unresolved; it cannot choose between four equippable variants and a non-equippable entry.
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


## Uncommon armor catalog

Survivors' band-one armor uses the eight-piece dungeon-upgrade sets below. This is an
authored progression choice: vanilla's mixed rare/epic qualities and quest acquisition
are not copied. Survivors retains its uncommon quality, three increasing stat lines and
existing ownership rules. The weapon catalog below completes this band’s nine identities.

| Survivors class | Appearance set | Vanilla item-set ID |
| --- | --- | --- |
| Warrior | Heroism | 511 |
| Paladin | Soulforge | 516 |
| Hunter | Beastmaster | 515 |
| Rogue | Darkmantle | 512 |
| Priest | Virtuous | 514 |
| Shaman | The Five Thunders | 519 |
| Mage | Sorcerer | 517 |
| Warlock | Deathmist | 518 |
| Druid | Feralheart | 513 |

All 72 names, item entries, displays and attachment metadata come from the pinned item
templates above, selected by `itemset` and mapped by `InventoryType` to the eight armor
slots (robe type 20 occupies chest). The checked-in identity table is
`crates/survivors-core/src/gear_upgrades.rs`. No SQL, vanilla stats, or item art is bundled.

Each class now has 19 named collection cells; 35 are still unnamed. All nine uncommon
cells now list authored Un'Goro boss sources; that zone remains
world-locked pending arena acceptance. Naming these pieces does not grant ownership, unlock zones or add raid
bonuses. Live race/sex rendering and complete source acceptance remain open.

## Uncommon weapon catalog

These are authored Survivors tier choices, using exact unique identities from the same
pinned templates. They are not vanilla dungeon-set members, and vanilla rarity, stats,
required levels and acquisition rules are not imported. All entries are weapons (item
class 2); their class masks permit the assigned class. Benediction specifically requires
Priest (mask 16). Each occupies the single Survivors main-hand collection slot; the Hunter
crossbow attaches as ranged equipment, and two-handed weapons add no off-hand cell.

| Class | Weapon | Entry | Display | Inventory type | Subclass | Material | Sheath |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Warrior | Arcanite Reaper | 12784 | 23904 | 17 | 1 | 1 | 1 |
| Paladin | Hammer of the Grand Crusader | 18717 | 23239 | 17 | 5 | 1 | 1 |
| Hunter | Carapace Spine Crossbow | 18738 | 31239 | 26 | 18 | 2 | 0 |
| Rogue | Heartseeker | 12783 | 23248 | 13 | 15 | 1 | 3 |
| Priest | Benediction | 18608 | 31347 | 17 | 10 | 2 | 2 |
| Shaman | The Hammer of Grace | 11923 | 21956 | 21 | 4 | 2 | 3 |
| Mage | Headmaster's Charge | 13937 | 24771 | 17 | 10 | 2 | 2 |
| Warlock | Rod of the Ogre Magi | 18534 | 30870 | 17 | 10 | 2 | 2 |
| Druid | Unyielding Maul | 18531 | 30867 | 17 | 5 | 6 | 1 |

All nine weapons list U'cha at 15:00 in the authored Un'Goro schedule. Live reward delivery
and body/attachment rendering remain open for these weapons.
Their numerical power still comes from the existing uncommon quality rules.

## Class-specific Atiesh

The same pinned revision's [quest templates](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/quest_template.sql)
join `RequiredClasses` and `RewItemId1` to the item catalog. The item equip masks alone
are shared and cannot identify the intended class.

| Class | Quest | Class mask | Item entry | Display |
| --- | --- | --- | --- | --- |
| Priest | 9257 | 16 | 22631 | 35634 |
| Mage | 9270 | 128 | 22589 | 35632 |
| Warlock | 9271 | 256 | 22630 | 35631 |
| Druid | 9269 | 1024 | 22632 | 35633 |

These resolve the four existing band-five main-hand cells through the class-aware piece
lookup used by icons and equipment rendering. All are two-handed staffs (inventory 17,
item class 2, subclass 10, material 2, sheath 2). They add no combat stats or availability;
Naxxramas acceptance and the remaining unnamed cells are still pending. Automated checks
cover all four display rows and readable icon assets; live staff attachment checks remain open.

## Shared jewelry icons

`survivors::jewel_icons` uses the same pinned item-template source. Four abbreviated
Survivors names map to their unique full vanilla name below; saved names stay unchanged.
Royal Seal variants all share one display. Mark of the Champion has two distinct
displays (35649 and 35621) and deliberately remains unresolved.

| Survivors name | Vanilla entry | Display | Source name when abbreviated |
| --- | --- | --- | --- |
| Band of the Unicorn | 7553 | 15422 | — |
| Seal of Wrynn | 2933 | 9845 | — |
| Ring of the Moon | 12052 | 9837 | — |
| Magni's Will | 12548 | 9837 | — |
| Ring of Protection | 15855 | 26537 | — |
| Don Mauricio's Band | 22433 | 24022 | Don Mauricio's Band of Domination |
| Ring of Fortitude | 10739 | 15422 | — |
| Band of Rumination | 18103 | 24022 | — |
| Ring of the Fallen | 21709 | 34336 | Ring of the Fallen God |
| Rune of the Guard Captain | 19120 | 31631 | — |
| Smotts' Compass | 4130 | 6562 | — |
| Defender of the Timbermaw | 21326 | 33702 | — |
| Ramstein's Lightning Bolts | 13515 | 1236 | — |
| Talisman of Ephemeral Power | 18820 | 31287 | — |
| Zandalarian Hero Medallion | 19949 | 32431 | — |
| Eye of the Beast | 13968 | 24784 | — |
| Briarwood Reed | 12930 | 8232 | — |
| Drake Fang Talisman | 19406 | 31936 | — |
| Royal Seal of Eldre'Thalas | 18465, 18466, 18467, 18468, 18469, 18470, 18471, 18472, 18473 | 29712 | — |
| Mindtap Talisman | 18371 | 30723 | — |
| Neltharion's Tear | 19379 | 31901 | — |
| Amulet of the Redeemed | 22327 | 9857 | — |
| Medallion of Grand Marshal Morris | 13091 | 23717 | — |
| Pendant of the Fallen Dragon | 19371 | 31889 | — |
| Mark of Fordring | 15411 | 23716 | — |
| Onyxia Tooth Pendant | 18404 | 9860 | — |
| Prestor's Talisman | 19377 | 31899 | Prestor's Talisman of Connivery |
| The Eye of Hakkar | 19856 | 32297 | — |
| Amulet of the Darkmoon | 19491 | 32008 | — |
| Choker of the Fire Lord | 18814 | 1399 | — |
| Gem of Nerubis | 22937 | 35817 | — |
| Pendant of the Qiraji Guardian | 21700 | 34204 | — |
| Amulet of Vek'nilash | 21608 | 34204 | — |
| Sadist's Collar | 23023 | 35358 | — |
| Stormrage's Talisman | 23053 | 35359 | Stormrage's Talisman of Seething |
| The Burrower's Shell | 23558 | 36290 | — |
| Talisman of Ascendance | 22678 | 24730 | — |
| Master Dragonslayer's Medallion | 19383 | 31907 | — |
