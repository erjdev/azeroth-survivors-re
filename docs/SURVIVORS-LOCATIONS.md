# Campaign location evidence — 2026-10-24

Reproduce from this checkout only:

```bash
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --locations
```

The command is read-only and does not unlock zones. WorldMapArea matching uses the
required instance ID where one is specified, otherwise internal art or localized AreaTable
names. A missing row means this lookup could not establish a candidate, not that the map
or geometry is absent. Coordinates below are raw WoW world coordinates.

The current full content audit reports six missing configured maps, three missing
encounter/drop schedules among configured maps, an unresolved Naxxramas start floor,
and 44 unnamed/unsourced cells per class. Its 802-gap total counts each class's cell
coverage separately; it is not a count of unique missing assets.

Additional center probes:

| Destination | Map | Probe x, y, z | Terrain z | WMO floor in grounding window |
| --- | --- | --- | --- | --- |
| Warsong Gulch | 489 | 1244.792, 1468.750, 310.296 | 309.296 | None |
| Arathi Basin | 529 | 922.917, 980.208, -52.598 | -53.598 | None |
| Alterac Valley | 30 | -327.083, -337.500, 9.038 | 8.038 | None |

Use `--floor <map> <x> <y> <z>` with the same executable/data path to reproduce.
These probes loaded complete WMO groups referenced by the center ADT. Their lack of a
nearby WMO floor does not prove clearance, dryness, arena coverage or connected routes.
Before authoring spawns, inspect terrain/water and collision over the intended arena,
then validate rendered identity, camera behavior and movement. Roster and drop sources
must be verified separately. Existing Elwynn/Duskwood spawns are not replaced by these
map-art midpoints.

Derived from the explicitly supplied local Map.dbc, WorldMapArea.dbc and terrain data.
Map-art midpoints are investigation candidates, not safe spawn positions or traversal proof.

## Elwynn Forest

WorldMapArea row 30: `Elwynn`, map 0, area 12.

Map-art midpoint: x -9096.875, y -200.000.

Terrain at midpoint: z 69.305. Check water, slope, WMO floors, obstacles and arena footprint before use.

## Duskwood

WorldMapArea row 34: `Duskwood`, map 0, area 10.

Map-art midpoint: x -10616.666, y -516.667.

Terrain at midpoint: z 146.373. Check water, slope, WMO floors, obstacles and arena footprint before use.

## Warsong Gulch

Required instance: 489, Some("Warsong Gulch"), directory Some("PVPZone03").

WorldMapArea row 443: `WarsongGulch`, map 489, area 3277.

Map-art midpoint: x 1244.792, y 1468.750.

Terrain at midpoint: z 309.296. Check water, slope, WMO floors, obstacles and arena footprint before use.

## Un'Goro Crater

WorldMapArea row 201: `UngoroCrater`, map 1, area 490.

Map-art midpoint: x -7200.000, y -1316.667.

No verified terrain support at midpoint: Ok(None). WMO/interior inspection may be required.

## Arathi Basin

Required instance: 529, Some("Arathi Basin"), directory Some("PVPZone04").

WorldMapArea row 461: `ArathiBasin`, map 529, area 3358.

Map-art midpoint: x 922.917, y 980.208.

Terrain at midpoint: z -53.598. Check water, slope, WMO floors, obstacles and arena footprint before use.

## Feralas

WorldMapArea row 121: `Feralas`, map 1, area 357.

Map-art midpoint: x -4683.333, y 1966.667.

No verified terrain support at midpoint: Ok(None). WMO/interior inspection may be required.

## Gurubashi Arena

No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.

## Scarlet Monastery

Required instance: 189, Some("Scarlet Monastery"), directory Some("MonasteryInstances").

No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.

## Alterac Valley

Required instance: 30, Some("Alterac Valley"), directory Some("PVPZone01").

WorldMapArea row 401: `AlteracValley`, map 30, area 2597.

Map-art midpoint: x -327.083, y -337.500.

Terrain at midpoint: z 8.038. Check water, slope, WMO floors, obstacles and arena footprint before use.

## Blackrock Mountain

No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.

## Gates of Ahn'Qiraj

No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.

## Naxxramas

Required instance: 533, Some("Naxxramas"), directory Some("Stratholme Raid").

No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.


## Arena terrain surveys

Reproduce with `survivors-audit <Data> --terrain <map> <x> <y> <radius>`.
Samples lie on a two-yard grid within the disk. The survey loads adjacent tiles for both
samples and one-yard gradient neighbours; a missing tile file fails the command. Terrain
holes remain missing samples. Liquid counts include ADT liquid more than 0.1 yard above
terrain; steep counts use a central-difference gradient greater than 1 (45 degrees).
Counts can overlap. Neither these samples nor their gradient estimates prove continuous
walkability or detect WMO/doodad walls, liquids, floors, overhead geometry or camera issues.

| Candidate (raw WoW x, y) | Map | Radius | Supported / total | Wet | Steep | Missing gradient neighbours |
| --- | --- | --- | --- | --- | --- | --- |
| Warsong midpoint (1244.792, 1468.750) | 489 | 60 | 2821 / 2821 | 0 | 0 | 0 |
| Warsong midpoint (1244.792, 1468.750) | 489 | 150 | 17573 / 17665 | 0 | 702 | 18 |
| Arathi midpoint (922.917, 980.208) | 529 | 60 | 2821 / 2821 | 551 | 372 | 0 |
| Arathi blacksmith (990.934, 1008.034) | 529 | 30 | 709 / 709 | 0 | 0 | 0 |
| Alterac midpoint (-327.083, -337.500) | 30 | 60 | 2821 / 2821 | 0 | 9 | 0 |
| Alterac midpoint (-327.083, -337.500) | 30 | 40 | 1257 / 1257 | 0 | 8 | 0 |

All six surveys found zero impassable-chunk samples. The 150-yard Warsong disk crosses
two ADTs; other surveys use one. The blacksmith candidate comes from the actual WMO
placement printed by the Arathi center floor probe, not an invented coordinate; a building
occupies that location, so dry terrain beneath it is not evidence of an unobstructed arena.

Decisions: retain the 60-yard Warsong disk for collision/render/traversal inspection; do
not extend it to 150 yards based on center support. Reject the Arathi map midpoint as a dry
arena; investigate the blacksmith surroundings while accounting for its building. Alterac
needs local slope inspection or a better location even with the smaller radius. None of
these surveys unlocks a destination or validates encounters/rewards.


## Warsong source-model obstacles

Reproduce with `survivors-audit <Data> --doodads 489 1244.792 1468.750 310.296`.
The probe now reports transformed hull bounds and four horizontal rays from the supplied
point. Rays are one-sided, limited to 120 yards and use parsed ADT doodad hulls from the
surrounding existing 3×3 tiles whose origins lie within 200 yards. They do not include
terrain, WMO geometry/props, distant-origin overlaps or runtime collision registration.

Within 60 yards of the candidate center, 25 placement origins were found: 17 with parsed
hulls and eight without. A zero hull count does not establish intentional pass-through.

| Direction in raw WoW coordinates | Nearest source hull hit | Placement |
| --- | --- | --- |
| Negative Y | 8.898 yards | 596699 |
| Positive Y | 96.022 yards | 596694 |
| Negative X | None within probe scope | — |
| Positive X | None within probe scope | — |

Placement 596699 is `ashenvaletreestump02.mdx`, at
`[1240.6113, 1458.3818, 308.66428]`, scale `0.82910156`, with 24 hull triangles.
Its transformed AABB is `[1229.9644, 1450.9308, 306.7777]` to
`[1249.2847, 1468.3196, 318.40982]`: only 0.430 yards horizontally from the candidate
center, despite an origin distance of 11.2 yards. The AABB is conservative, not exact
occupancy; the 8.898-yard ray hit is at the supplied fixed height, not a capsule sweep
following terrain. This is a concrete obstacle fixture for runtime slide/route tests.

Next acceptance work: load map 489 at the documented center in a dedicated world-validation
scene, verify renderer identity and floor height, then compare capsule contact and sliding
against stump 596699 from multiple directions. Verify route finding around it, nearby
stumps and tile unload/reload before setting arena readiness. The existing engine viewer
supports both `WOW_MAP` and `WOW_WORLDVIEW_AT`; the Survivors development launcher now
exposes it with `--world-preview`.
Campaign map/encounter/reward definitions remain unchanged and Warsong remains locked.


## Developer instance preview

Use the incremental development build and a separate validation state directory:

```bash
WOW_DATA="$PWD/wow-data/Data" \
BENILLA_HOME="$PWD/target/survivors-validation/world-preview-state" \
WOW_MAP=489 WOW_WORLDVIEW_AT=1244.792,1468.750,310.296 \
WOW_UNATTENDED=1 WOW_NOSOUND=1 WOW_WIN=1280x720 \
cargo survivors -- --world-preview
```

This starts the engine's free-camera world viewer without campaign progression systems.
The flag requires the `dev` feature; normal launches still enter Survivors.
The native build and live launch passed. Runtime logs identify `PVPZone03` (map 489),
and the [720p capture](images/screenshots/2026-10-24-world-preview/warsong-720.jpg)
shows Warsong terrain, structures and stump obstacles. This verifies instance rendering,
not player capsule contact, sliding, navigation, combat or gameplay camera behavior.
Those acceptance checks remain outstanding; Warsong remains locked in the campaign.


## Placed stump capsule regression

`survivors::floor::tests::placed_warsong_stump_blocks_the_candidate_crossing_and_unregisters`
loads placement 596699 directly from the local Warsong ADTs, checks its model identity and
24 hull triangles, and bakes its actual rotation, translation and scale with the engine's
coordinate helpers. In a headless physics world, the body ray reproduces the 8.898-yard
source hit (within 0.02 yards), and the Survivors hero capsule cannot complete the same
24-yard crossing. Removing the collider clears both the ray and crossing; registering it
again restores the same capsule result within 0.02 yards.

The fixture deliberately isolates the hull at a fixed height. It exercises `Floor::slide`
and the engine collision query, not the async streamer, terrain-following movement,
route finding or a live player. It therefore narrows the remaining investigation without
closing the reported Elwynn collision issue or claiming Warsong traversal acceptance.


## Un'Goro arena suitability and height-lookup discrepancy — 2026-10-25

The configured Fire Plume Ridge center `(-7136, -1309)`, radius 32, has 797/797
fast-height-supported samples, but 66 are submerged and 130 exceed the 45-degree gradient
threshold. Height ranges from -196.720 to -173.435. Eight adjacent centers, offset by 40
yards on either axis (including diagonals), each also contain steep, wet or unresolved
samples. Do not approve this summit disk as a dry, traversable arena.

Crater-floor alternatives, all sampled at radius 40 and two-yard spacing:

| Center x, y | Fast-height support / total | Wet | Steep | Notes |
| --- | --- | --- | --- | --- |
| -6836, -1309 | 1257 / 1257 | 59 | 5 | Reject this footprint |
| -7436, -1309 | 1240 / 1257 | 0 | 0 | Unresolved height samples need investigation |
| -7136, -1009 | 1240 / 1257 | 0 | 0 | Unresolved height samples need investigation |
| -7136, -1609 | 1240 / 1257 | 0 | 0 | All 17 misses have emitted-mesh support |

Reproduce the last row with:

```bash
cargo run -p survivors-audit -- "$PWD/wow-data/Data" --terrain 1 -7136 -1609 40
```

The terrain audit now compares unresolved fast-height samples with the actual emitted
triangle indices, reporting counts and up to eight coordinates. At `(-7100, -1625)` the
fast lookup returns no height while the mesh yields -271.66095; the first eight misses
share x=-7100 and y=-1625 through -1611 at two-yard intervals. This is evidence of a
height-lookup discrepancy, not an absent rendered surface. The diagnostic leaves original
support counts unchanged and does not supply runtime floors or fill authored holes.

Next: reproduce the lookup discrepancy at the chunk/cell boundary, verify physical
collision and placed obstacles at the southern candidate, then inspect it in the world
viewer before changing the authored arena. Existing map coordinates and campaign locks
are unchanged. Encounter/drop/class-item gaps remain separate requirements.


### Southern candidate after boundary recovery

With `survivors-terrain` enabled (the Survivors launcher and audit default), the same
`--terrain 1 -7136 -1609 40` survey now resolves all 1,257 samples: zero unresolved heights,
wet samples, impassable samples, steep samples or incomplete gradient neighbourhoods.
Height range remains -272.114 to -264.071. All 17 previous fast-index misses are recovered
from existing triangles. The earlier table records the pre-fix result, not current coverage.

This makes the southern disk a candidate for collider/obstacle and live-view inspection;
it does not prove between-sample safety, WMO clearance, connected routes, camera behavior,
or readiness to unlock Un'Goro. The authored summit location remains unchanged.


### Southern candidate rendered scene and obstacles — 2026-10-25

The checkout world viewer launched on map 1 (`Kalimdor`) with
`WOW_WORLDVIEW_AT=-7136,-1609,-269.4892`, isolated state and a 1280×720 window.
[Rendered candidate](images/screenshots/2026-10-25-ungoro-candidate/world-preview-720.jpg)
shows jungle ground, a large nearby trunk and dense canopy. The viewer is an elevated free
camera, not the Survivors camera or a player traversal test. Its log contains no errors
or panics. The original launch wrapper was restored after closing the viewer.

The center terrain height is -270.4892. The start-ADT WMO probe found no floor at that XY.
The surrounding 3×3-ADT doodad probe found placement 304715 (`feralastree03.mdx`) at
`(-7135.717, -1602.6602, -272.30038)`, scale 0.8144531, 30 hull triangles and an origin
6.3 horizontal yards from the center. Its conservative horizontal hull AABB is only
0.497 yards from the center. At raw z=-268, the +Y hull ray hits it after 2.6105921 yards;
this is a source ray, not capsule clearance. The other rays hit placement 304699 after
47.00218 yards (-Y), placement 304693 after 35.11947 yards (-X), and nothing within 120
yards (+X). These directions are raw WoW axes.

Reproduce with `--floor 1 -7136 -1609 -268` and `--doodads 1 -7136 -1609 -268`.
The nearest tree now provides a concrete placed-hull fixture for body-clearance, sliding
and grounded movement checks. Do not infer an unobstructed arena from the dry terrain
survey; distant-origin overlapping models and WMO props remain outside this source probe.
No campaign location, content lock or reward data changed.

### Placed Un'Goro tree capsule fixture

`survivors::floor::tests::placed_ungoro_tree_keeps_candidate_clear_and_blocks_the_hull_crossing`
loads tree 304715 from the candidate's actual ADT, checks its model and 30-triangle hull,
and applies the placement's position, rotation and scale. It checks initial player-capsule
intersection at the terrain-supported candidate center, reproduces the +WoW-Y source ray
at z=-268, and compares a fixed-height 20-yard capsule crossing before and after collider
removal. It also loads the ADT's terrain colliders and applies 80 quarter-yard walking
steps, grounding after each sweep and checking the capsule against the tree (0.02-yard
intersection tolerance). Before the correction, step 10 penetrated the hull: sliding up
its angled face followed by grounding moved the capsule inside the base. Steep contact
normals now constrain horizontal sliding as walls, without the artificial upward motion.
The grounded path finishes at Bevy `(1607.4258, -270.63077, 7136.2515)` from
`(1609, -270.4892, 7136)`, remaining outside the trunk throughout.

All 12 floor tests pass with the checkout data, including existing Elwynn variants,
placed-canopy movement, Warsong stump removal, tangential wall movement and gentle mobility
slopes. This fixture covers one Un'Goro approach and the starting capsule; other props,
async streaming, route finding and gameplay-camera behavior remain unverified. The zone
remains locked pending its complete placement and content acceptance.


### Un'Goro encounter identity and source table — 2026-10-25

The pinned MaNGOS Zero [creature templates](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql)
and [placements](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature.sql)
confirm the nine roster bosses. All representative placements below are on map 1
inside Un'Goro; they establish zone membership, not a spawn at the proposed arena center.

| Creature | Entry | Display | Placement GUID | Placement x, y | Survivors time | Uncommon slot |
| --- | --- | --- | --- | --- | --- | --- |
| Ravasaur Matriarch | 6581 | 11319 | 24433 | -7593.43, -1773.54 | 3:00 | Helm |
| Clutchmother Zavas | 6582 | 11084 | 24434 | -8072.02, -1269.65 | 4:30 | Wrist |
| Uhk'loc | 6585 | 8129 | 24438 | -6278.97, -2000.4 | 6:00 | Shoulders |
| Gruff | 6583 | 10932 | 24435 | -7829.1, -874.524 | 7:30 | Hands |
| Blazerunner | 9376 | 1204 | 24430 | -7183.41, -1294.81 | 9:00 | Chest |
| Baron Charr | 14461 | 14517 | 590000 | -7153.77, -1085.68 | 12:00 | Waist |
| Tyrant Devilsaur | 6500 | 5240 | 23745 | -7464.14, -1321.6 | 14:00 | Legs |
| U'cha | 9622 | 8844 | 24254 | -6276.53, -1987.97 | 15:00 | Main Hand |
| King Mosh | 6584 | 5305 | 24437 | -6854.36, -772.935 | 16:00 | Feet |

Times, slots and 60% base chances are authored Survivors tuning, not imported vanilla
loot. Every class resolves these nine cells to its uncommon armor and weapon identities. Combat
scheduling, boss-only Keen Eye chances and collection/drop-page sources share the same
rows. Victory waits until 16:00 and all nine bosses are defeated. Duplicate rolls retain
the existing no-additional-power rule. All nine uncommon slots now have sources in this table.

The zone remains locked with **world content is not ready**, including on an otherwise
fully progressed hero. The rejected summit coordinates remain configured until the
candidate's gameplay placement is accepted. This source table does not approve those
coordinates, routes, camera, spawning or live combat. Downloaded SQL remains ignored.


### Southern candidate gameplay check — 2026-10-25

An isolated Hunter fixture tested center `[-7136, -1609, -268]`, leash 40, in a temporary
preview binary. The fixture supplied earlier-zone clears and enabled this zone only for
validation. The committed world lock and summit configuration remain unchanged; the
preview override and launcher were restored afterwards.

The [1280×720 gameplay capture](images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)
shows elapsed 00:22, level 3, 55 kills and the grounded-looking player, enemies, projectiles
and pickups. A chest selection granted Arcane Shot rank two; subsequent level selection
added Serpent Sting. Moving right away from the central trunk opened the camera view.
Earlier near-center views were substantially occupied by that trunk. Auto-push camera and
scenery cutaway were enabled in Settings. This does not prove either feature's behavior
at all occluders, or measure capsule clearance from screenshots.

The minimap labels this location **The Marshlands**, inconsistent with the current
**Fire Plume Ridge** arena name. Before adopting it, resolve the location/name contract,
record player positions through a complete traversal, check overlapping props and enemy
routes, and validate camera occlusion from the starting view and arena edges. The six
scheduled bosses and rewards still require a complete live run. The log contains frame
hitches around one second and stall samples; no panic/error lines were found. Performance
acceptance remains open. This short combat check is not zone approval.

## Feralas provisional arena and source table — 2026-10-25

The same pinned MaNGOS Zero [creature templates](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql) and [placements](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature.sql) establish these map-1 identities.
Representative placements prove membership in the wider Feralas region, not that all
creatures normally share the proposed arena. Dire Maul outdoor-arena rares are explicitly
included as authored Feralas encounters; they are not taken from instance map 429.

| Creature | Entry | Display | Placement GUID | x, y, z |
| --- | --- | --- | --- | --- |
| Longtooth Runner | 5286 | 165 | 50802 | -4184.84, -740.034, -34.2743 |
| Ironfur Bear | 5268 | 3201 | 50643 | -4189.08, -784.331, -40.3838 |
| Woodpaw Mongrel | 5249 | 8598 | 50382 | -4745.8, 620.225, 45.5468 |
| Gordunni Ogre | 5229 | 597 | 50168 | -3552.33, 220.407, 149.381 |
| Gordunni Brute | 5232 | 11554 | 50186 | -3531.52, 224.624, 149.388 |
| Woodpaw Trapper | 5251 | 10796 | 50423 | -4871.36, 99.1926, 87.5937 |
| Feral Scar Yeti | 5292 | 3207 | 50968 | -5091.67, 1558.33, 53.6518 |
| Gordunni Warlock | 5240 | 11560 | 50291 | -5715.53, 1178.7, 66.6073 |
| Frayfeather Hippogryph | 5300 | 3210 | 51143 | -5184.62, 1624.39, 77.3243 |
| Woodpaw Mystic | 5254 | 10794 | 50481 | -5027.9, 71.1246, 120.282 |
| Woodpaw Reaver | 5255 | 10795 | 50525 | -4803.16, 881.535, 143.113 |
| Rage Scar Yeti | 5296 | 3220 | 51025 | -3873.59, 1618.23, 136.71 |
| Gordunni Mauler | 5234 | 11557 | 50219 | -5078.35, 1220.73, 51.696 |
| Woodpaw Alpha | 5258 | 10793 | 50561 | -4805.34, 873.073, 143.113 |
| Wandering Forest Walker | 7584 | 9592 | 50797 | -3432.24, 2382.82, 43.9324 |
| Snarler | 5356 | 780 | 51683 | -4200, -133.828, 56.2126 |
| Old Grizzlegut | 5352 | 706 | 51839 | -5120.65, 1308.87, 37.6403 |
| Diamond Head | 5345 | 1817 | 51843 | -4748.69, 2897.92, -39.7154 |
| Qirot | 5350 | 11142 | 51681 | -5435.75, 114.241, 28.2281 |
| Arash-ethis | 5349 | 7569 | 51844 | -3525.88, 2328.92, 63.964 |
| Antilus the Soarer | 5347 | 10889 | 51840 | -5802.39, 1577.41, 73.785 |
| Mushgog | 11447 | 14382 | 590013 | -3755.33, 1094.43, 131.969 |
| The Razza | 11497 | 10807 | 590014 | -3755.33, 1094.43, 131.969 |
| Skarr the Unbreakable | 11498 | 10169 | 590015 | -3755.33, 1094.43, 131.969 |

A two-yard-grid terrain survey at Snarler’s placement `[-4200, -133.828]`, radius 50,
found 1961/1961 supported samples, zero wet/impassable/greater-than-45-degree samples,
and heights 54.973–61.482. The center floor query is 55.826717; the provisional map uses
55.827. This measures ADT/MCLQ only, not between-sample hazards, props, route connectivity,
camera or combat. Comparisons at Old Grizzlegut, Arash-ethis and Qirot were rejected:
194, 405 and 1475 steep samples respectively; Qirot also had three wet samples.

| Time | Encounter | Rare slot | Base drop chance |
| --- | --- | --- | --- |
| 3:00 | Snarler | Helm | 60% |
| 4:30 | Old Grizzlegut | Shoulders | 60% |
| 6:00 | Diamond Head | Chest | 60% |
| 7:30 | Qirot | Waist | 60% |
| 9:00 | Arash-ethis | Legs | 60% |
| 10:30 | Antilus the Soarer | Feet | 60% |
| 12:00 | Mushgog | Wrist | 60% |
| 14:00 | The Razza | Hands | 60% |
| 16:00 | Skarr the Unbreakable | Main Hand | 60% |

Times, rewards, speed/scale and boss promotion are Survivors tuning. Creature minimum
levels and first model IDs come from the templates; health/damage/XP use the existing
Survivors ladder. Final victory waits for 16 minutes and nine defeated bosses. All nine
Rare cells resolve for every class. No vanilla loot or creature AI is imported.

The map is appended to preserve saved map indices. Feralas remains `ready: false` and
reports world content is not ready, even on a fully progressed account. Prop collision,
camera, traversal, streaming, combat pacing, live rewards and screenshots remain open.

### Feralas candidate prop and visual inspection — 2026-10-25

The [1280×720 world preview](images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)
uses `WOW_MAP=1`, `WOW_WORLDVIEW_AT=-4200,-133.828,55.827` and isolated preview state.
The initial free-camera view shows supported ground through a forest corridor, large
trunks and canopy at the sides, and surface roots. It is not a Survivors follow-camera
or actor traversal test. The viewer was closed and the validation launcher restored.

The 200-yard-origin doodad probe finds these nearby authored collision hulls:

| Placement | Model suffix | Origin distance | Horizontal AABB distance | Hull triangles |
| --- | --- | --- | --- | --- |
| 245514 | feralastreeroots01.mdx | 25.1 | 13.581 | 78 |
| 244410 | feralastree03.mdx | 47.5 | 27.238 | 30 |
| 245518 | kalidarbush02.mdx | 35.4 | 28.674 | 4 |
| 244159 | feralastree04.mdx | 81.5 | 28.954 | 369 |
| 245516 | feralastreeroots02.mdx | 45.4 | 32.800 | 60 |
| 245515 | feralastreeroots02.mdx | 39.6 | 37.100 | 60 |

AABBs are conservative bounds, not occupancy. The large tree with an origin outside
the arena still overlaps its footprint: an origin-only clearance rule is insufficient.
At the candidate height, the negative-WoW-X source ray hits root 245515 at 38.137535 yards.
Its placement is `[-4239.0977,-127.80469,53.789024]`, rotation `[-2,282.7598,-21]`,
scale `0.5839844`. Other cardinal rays find no hull within 120 yards in positive X/Y,
and a negative-Y hit at 76.98257 yards. These are one-sided rays, not capsule or route proofs.

Grounded root traversal, streamed collider registration, complete arena connectivity,
camera push-in/cutaway and live enemy movement remain acceptance requirements. The
terrain-only screen is insufficient to open Feralas.

## Gurubashi outdoor roster and Epic schedule — 2026-10-25

The pinned MaNGOS Zero [creature templates](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql)
and [placements](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature.sql)
establish the following outdoor Stranglethorn identities on map 0. The previous roster
mixed outdoor creatures with Zul'Gurub raid identities. Shadowmaw Panther also used the
wrong display (11452); its template's first display is 633. The new roster uses each
verified template's first display and minimum level. Alternate source displays are not
randomized. The spawn GUID is one supporting outdoor placement, not an arena spawn claim.

| Role | Creature | Entry | Display | Level | Map-0 spawn GUID |
|---|---|---:|---:|---:|---:|
| Horde | Shadowmaw Panther | 684 | 633 | 37 | 1109 |
| Horde | Cold Eye Basilisk | 690 | 8797 | 39 | 758 |
| Horde | Jungle Stalker | 687 | 11317 | 40 | 1181 |
| Horde | Elder Mistvale Gorilla | 1557 | 838 | 40 | 1939 |
| Horde | Zanzil Zombie | 1488 | 1065 | 43 | 759 |
| Horde | Thrashtail Basilisk | 1550 | 8802 | 41 | 371 |
| Horde | Bloodsail Sea Dog | 1565 | 796 | 43 | 380 |
| Horde | Skullsplitter Berserker | 783 | 4623 | 43 | 658 |
| Horde | Bloodsail Raider | 1561 | 793 | 39 | 2549 |
| Horde | Skullsplitter Hunter | 669 | 4628 | 41 | 1157 |
| Horde | Skullsplitter Mystic | 780 | 4616 | 39 | 1147 |
| Horde | Bloodsail Warlock | 1564 | 4461 | 41 | 949 |
| Horde | Zanzil Hunter | 1489 | 828 | 43 | 2333 |
| Horde | Jungle Thunderer | 1114 | 845 | 37 | 392 |
| Horde | Stranglethorn Tigress | 772 | 614 | 37 | 804 |
| Boss | Kurmokk | 14491 | 3188 | 42 | 49126 |
| Boss | Verifonix | 14492 | 7232 | 42 | 49152 |
| Boss | Scale Belly | 1552 | 12342 | 45 | 1577 |
| Boss | Lord Sakrasis | 2541 | 4910 | 45 | 32043 |
| Boss | Rippa | 14490 | 14528 | 44 | 49058 |
| Boss | Tethis | 730 | 8472 | 43 | 1342 |
| Boss | Bhag'thera | 728 | 613 | 40 | 1348 |
| Boss | King Bangalash | 731 | 616 | 43 | 2169 |
| Boss | King Mukla | 1559 | 792 | 51 | 823 |

Arena arrivals, speeds, scales and reward assignment are authored Survivors mechanics,
not vanilla encounters or loot. The nine bosses arrive at 3:00, 4:30, 6:00, 7:30, 9:00,
10:30, 12:00, 14:00 and 16:00 in the order above. They roll the nine Epic slots in grid
order at 60% base chance, using the existing boss-luck modifier. Victory requires all
nine defeats and the last scheduled time. Campaign summary now names King Mukla.

The campaign readiness flag is explicitly false, so adding the schedule cannot unlock
unverified world content. Existing center/leash are unchanged. Floor support, arena walls,
traversal, camera behavior, crowd performance and live rewards still require acceptance;
outdoor placements establish map identity, not those gameplay properties.

Checkout terrain probe: `survivors-audit <Data> --terrain 0 -13205 272 30`
sampled 709 points at 2-yard spacing, all supported at z=21.857, with no liquid deeper
than 0.1 yard, impassable terrain, slopes above 45 degrees or incomplete gradient
neighborhoods. This only covers sampled ADT/MCLQ; WMO floors/walls, intervening hazards,
props, connectivity and camera behavior are excluded. It does not unlock the zone.
