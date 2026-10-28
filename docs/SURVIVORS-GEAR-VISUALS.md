# Gear visual identities

`survivors-core::gear_visuals` maps named gear to vanilla item entry, display ID and
inventory type, class/subclass, material and sheath metadata. Gameplay stats and rarity remain Survivors rules; they are not imported
from the vanilla templates.

Source: [MaNGOS Zero item templates, pinned revision 325948a78015794e6cc626212bc19fee70c805b2](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/item_template.sql).
Mappings use exact, unique names and the `entry`, `displayid`, `InventoryType`, `class`,
`subclass`, `Material` and `sheath` columns.
Only these factual fields are recorded; the SQL download stays in ignored validation
storage. Runtime icons, textures and models come from the user's data folder.

## Warrior weapon alternative identities

The core catalog now validates these stable supplemental identities and the install
audit decodes both icons. They are **not yet obtainable or model-validated**.
Collection UI and actual reward delivery remain required before accepting Mace
and Polearm Specialization.

| Weapon | Entry | Display | Inventory type | Class | Subclass | Material | Sheath |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Worn Mace | 36 | 5194 | 21 | 2 | 4 | 2 | 3 |
| Short Spear | 15810 | 26500 | 17 | 2 | 6 | 1 | 1 |

Inventory type 21 identifies a main-hand weapon; type 17 identifies a two-handed
weapon. Subclasses 4 and 6 identify mace and polearm respectively. Survivors will
use Common quality for both, with the existing Common main-hand bonus. The item
name does not decide Survivors quality. No vanilla damage, level requirement,
vendor price or proc is imported.

## Existing catalog conventions

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

Each class now has all 54 collection cells named and mapped. All nine uncommon
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

## Rare armor catalog

Band two uses these eight-piece vanilla sets as an authored Survivors appearance tier.
Their original epic quality, raid acquisition and set powers are not imported: Survivors
retains its Rare stat rules. This does not represent the requested additional raid-epic
tier or implement its build-oriented set bonuses.

| Class | Appearance set | Vanilla item-set ID | Required class mask |
| --- | --- | --- | --- |
| Warrior | Might | 209 | 1 |
| Paladin | Lawbringer | 208 | 2 |
| Hunter | Giantstalker | 206 | 4 |
| Rogue | Nightslayer | 204 | 8 |
| Priest | Prophecy | 202 | 16 |
| Shaman | Earthfury | 207 | 64 |
| Mage | Arcanist | 201 | 128 |
| Warlock | Felheart | 203 | 256 |
| Druid | Cenarion | 205 | 1024 |

All 72 items were selected from the same pinned templates by `itemset`; each set contains
exactly one item per armor slot and has the assigned class's `AllowableClass` mask.
`crates/survivors-core/src/gear_upgrades/rare.rs` records only names and visual metadata.
Chest inventory type 20 remains a robe, including Earthfury and Cenarion vestments.
Rare weapons are listed below. All Rare slots now have authored Feralas encounter sources;
Feralas stays world-locked, and live acquisition and body rendering remain open.
Collection naming does not award pieces or unlock zones.

## Rare weapon catalog

Each weapon below is an exact unique name in the pinned item templates. All have item
class 2 and unrestricted `AllowableClass` masks (-1); the chosen subclasses fit their
assigned classes. This is an authored Survivors tier assignment, not vanilla rarity,
weapon-skill progression, loot or spell-power import. They use the existing Rare stats.
The Hunter gun attaches as ranged gear but occupies the single main-hand collection slot;
two-handed choices do not create an off-hand cell or a new talent requirement.

| Class | Weapon | Entry | Display | Inventory type | Subclass | Material | Sheath |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Warrior | Obsidian Edged Blade | 18822 | 34112 | 17 | 8 | 1 | 1 |
| Paladin | Bonereaver's Edge | 17076 | 32199 | 17 | 8 | 1 | 1 |
| Hunter | Blastershot Launcher | 17072 | 29163 | 26 | 3 | 1 | 0 |
| Rogue | Brutality Blade | 18832 | 31309 | 13 | 7 | 1 | 3 |
| Priest | Aurastone Hammer | 17105 | 34109 | 13 | 4 | 2 | 3 |
| Shaman | Earthshaker | 17073 | 32162 | 17 | 5 | 2 | 1 |
| Mage | Azuresong Mageblade | 17103 | 29677 | 21 | 7 | 1 | 3 |
| Warlock | Sorcerous Dagger | 18878 | 31337 | 21 | 15 | 1 | 3 |
| Druid | Finkle's Lava Dredger | 18803 | 31265 | 17 | 5 | 1 | 1 |

This completes all nine Rare identities per class. Skarr the Unbreakable at 16:00 is the
authored weapon source in Feralas; that zone remains world-locked. Live acquisition and
weapon attachment/pose checks remain open; the source table does not establish obtainability.

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
vanilla variants. Survivors explicitly chooses entry 23206, display 35649, for its
single shared trinket; entry 23207/display 35621 is not a second Survivors item.
This is an artwork choice, not an import of vanilla combat bonuses.
The Codex and chest choice cards both use these item-display mappings. Legendary amulet
cards retain their associated spell art; ordinary jewelry never resolves its item ID
through the spell icon catalog.

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
| Mark of the Champion | 23206 | 35649 | Explicit variant choice |
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

## Epic identity catalog

Band three now has nine verified identities per class: eight class-set armor pieces and
one compatible weapon. These are authored Survivors tier choices from the same pinned
item templates. Their vanilla raid sources, procs and set bonuses are not imported.
The ordinary Epic band retains its five-stat progression; the requested raid-epic variant
with an extra affix and build-oriented set bonuses remains separate, unfinished work.
No ownership or source availability is granted by naming these cells.

| Class | Appearance set | Vanilla item-set ID | Required class mask |
| --- | --- | --- | --- |
| Warrior | Wrath | 218 | 1 |
| Paladin | Judgement | 217 | 2 |
| Hunter | Dragonstalker | 215 | 4 |
| Rogue | Bloodfang | 213 | 8 |
| Priest | Transcendence | 211 | 16 |
| Shaman | The Ten Storms | 216 | 64 |
| Mage | Netherwind | 210 | 128 |
| Warlock | Nemesis | 212 | 256 |
| Druid | Stormrage | 214 | 1024 |

Armor maps by inventory type to the eight slots, with robes in the chest slot. All
81 item entries are distinct. The table lives in
`crates/survivors-core/src/gear_upgrades/epic.rs`; runtime art still comes from local data.

| Class | Weapon | Entry | Display | Inventory type | Subclass | Material | Sheath |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Warrior | Ashkandi, Greatsword of the Brotherhood | 19364 | 32000 | 17 | 8 | 1 | 1 |
| Paladin | The Untamed Blade | 19334 | 31999 | 17 | 8 | 1 | 1 |
| Hunter | Ashjre'thul, Crossbow of Smiting | 19361 | 32763 | 26 | 18 | 2 | 0 |
| Rogue | Chromatically Tempered Sword | 19352 | 31867 | 13 | 7 | 1 | 3 |
| Priest | Lok'amir il Romathis | 19360 | 31863 | 21 | 4 | 1 | 3 |
| Shaman | Herald of Woe | 19357 | 31878 | 17 | 5 | 1 | 1 |
| Mage | Staff of the Shadow Flame | 19356 | 31960 | 17 | 10 | 2 | 2 |
| Warlock | Claw of Chromaggus | 19347 | 31865 | 13 | 15 | 1 | 3 |
| Druid | Draconic Maul | 19358 | 31877 | 17 | 5 | 1 | 1 |

All nine weapon class masks are unrestricted in the source templates; selected weapon
types are usable by their assigned class. The Hunter crossbow is ranged, while the
other weapons occupy the existing main-hand presentation. No off-hand cell is added.
Epic acquisition sources and full body/attachment rendering acceptance remain open.

## Legendary armor identity catalog

Band five uses the eight armor pieces of the following vanilla class sets. Rings in
the source sets are deliberately excluded from the nine-slot equipment grid. This is
an authored Survivors rarity assignment: vanilla quality, raid acquisition and set
effects are not imported. Existing class-specific Legendary weapon entries, including
the four Atiesh variants, remain unchanged.

| Class | Appearance set | Vanilla item-set ID | Required class mask |
| --- | --- | --- | --- |
| Warrior | Dreadnaught | 523 | 1 |
| Paladin | Redemption | 528 | 2 |
| Hunter | Cryptstalker | 530 | 4 |
| Rogue | Bonescythe | 524 | 8 |
| Priest | Faith | 525 | 16 |
| Shaman | Earthshatter | 527 | 64 |
| Mage | Frostfire | 526 | 128 |
| Warlock | Plagueheart | 529 | 256 |
| Druid | Dreamwalker | 521 | 1024 |

All 72 armor entries have distinct IDs, exact source class masks and the correct armor
subclass/inventory slot. Names, display IDs, material and sheath fields are stored in
`crates/survivors-core/src/gear_upgrades/legendary.rs`, from the pinned templates above.
The existing Legendary numerical progression remains authoritative. No source schedule,
collection ownership, new raid-epic variant or build-oriented set power is added by this
identity mapping. Acquisition and live all-class/race/sex appearance acceptance remain open.

## PvP identity catalog and full grid coverage

Band four completes all 54 named/mapped cells per class (486 class/slot/quality cells).
The six rank-set armor slots are supplemented by compatible waist and wrist pieces.
This is an authored, faction-independent Survivors catalog: source honor/reputation
requirements, faction restrictions, vanilla stats and set powers are not imported.
Names and exact entry/display/attachment metadata come from the same pinned templates;
explicit IDs disambiguate reputation items with several level variants.

| Class | Six-piece rank-set ID | Waist entry | Wrist entry | Weapon entry |
| --- | --- | --- | --- | --- |
| Warrior | 384 | 20041 | 19578 | 18876 |
| Paladin | 402 | 20042 | 19578 | 18867 |
| Hunter | 395 | 20043 | 19582 | 18836 |
| Rogue | 394 | 20045 | 19587 | 18838 |
| Priest | 389 | 20047 | 19595 | 23454 |
| Shaman | 386 | 20195 | 19582 | 23464 |
| Mage | 388 | 20047 | 19595 | 23451 |
| Warlock | 392 | 20047 | 19595 | 18873 |
| Druid | 397 | 20046 | 19587 | 23455 |

The rank sets use Field Marshal/Marshal appearances, with Warlord/General mail for
Shaman. Waist entries are the level-58 Highlander girdles (Defiler mail for Shaman);
wrist entries are the level-60 Berserker, Windtalker, Forest Stalker and Dryad pieces.
All source class masks permit their assigned class, and armor subclasses/inventory
slots match. These 81 cells use 74 source items: compatible belts/bracers are shared
across classes, while each class has nine distinct PvP entries.

| Class | PvP weapon | Entry | Display | Inventory | Subclass | Material | Sheath |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Warrior | Grand Marshal's Claymore | 18876 | 31996 | 17 | 8 | 1 | 1 |
| Paladin | Grand Marshal's Battle Hammer | 18867 | 31954 | 17 | 5 | 2 | 1 |
| Hunter | Grand Marshal's Repeater | 18836 | 31757 | 26 | 18 | 2 | 0 |
| Rogue | Grand Marshal's Dirk | 18838 | 31379 | 13 | 15 | 1 | 3 |
| Priest | Grand Marshal's Warhammer | 23454 | 36064 | 13 | 4 | 2 | 3 |
| Shaman | High Warlord's Battle Mace | 23464 | 36078 | 13 | 4 | 2 | 3 |
| Mage | Grand Marshal's Mageblade | 23451 | 36045 | 13 | 15 | 1 | 3 |
| Warlock | Grand Marshal's Stave | 18873 | 31764 | 17 | 10 | 2 | 2 |
| Druid | Grand Marshal's Demolisher | 23455 | 36065 | 17 | 5 | 2 | 1 |

Full-grid coverage counts class/slot/quality cells, not globally unique item art.
Warlock Common and Uncommon armor already share Deathmist identities; that mapping is
preserved, with quality still selecting its own power. The tests require every valid
cell to resolve and each quality within a class to contain nine distinct items.
PvP acquisition schedules and live equipment/appearance acceptance remain open.


## Supplemental Poor catalog

`survivors-core::supplemental_gear` adds 81 class/slot identities outside the six
legacy collection bands. Stable keys such as `poor:warrior:hands` are independent
of display names and item IDs. The source remains the pinned MaNGOS Zero revision
linked above; all 41 distinct source items have vanilla Quality 0. Only identity
and visual metadata are imported. Vanilla level requirements and drop locations
are not Survivors acquisition rules.

| Classes | Armor family |
| --- | --- |
| Warrior, Paladin | Light Plate |
| Hunter, Shaman | Double Mail |
| Rogue, Druid | Hardened Leather |
| Priest, Mage, Warlock | Interlaced |

Each family includes head, shoulders, chest, waist, legs, feet, wrist and hands.
Weapons are Beaten Battle Axe (Warrior), Cracked Sledge (Paladin), Cracked Shortbow
(Hunter), Sharpened Letter Opener (Rogue), Withered Staff (Priest), Carpenter's Mallet
(Shaman), Rough Wooden Staff (Mage), Chipped Quarterstaff (Warlock), and Cedar Walking
Stick (Druid). This is authored Survivors progression, not imported vanilla item power.

The explicit-data gear audit includes every supplemental entry's display and icon.
It does not establish body texture/attachment correctness. Poor ownership, stable-key
equipment/appearance saves and the Collection row are now integrated. Survivors
acquisition uses a 2% aggregate chance from ordinary Elwynn/Duskwood enemies, with
nine equally likely slots; the rate is independent of vanilla item metadata.
Raid Epic identities and build-specific sets remain outstanding.

### Supplemental Raid Epic armor variants

The initial 72 Raid Epic armor identities reuse the verified eight ordinary Epic
armor visuals and item names for each class from `gear_upgrades/epic.rs`. Their
stable keys are `raid-epic:<class>:<slot>`; they are distinct Survivors progression
variants, not additional vanilla item IDs. Main-hand weapons are excluded. No
new install assets or unverified model references are introduced. Stats, set
bonuses and future acquisition are authored Survivors rules documented in
SURVIVORS.md, not imports of the vanilla items' powers or loot tables.
