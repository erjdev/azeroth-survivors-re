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
confirm the six existing roster bosses. All representative placements below are on map 1
inside Un'Goro; they establish zone membership, not a spawn at the proposed arena center.

| Creature | Entry | Display | Placement GUID | Placement x, y | Survivors time | Uncommon slot |
| --- | --- | --- | --- | --- | --- | --- |
| Ravasaur Matriarch | 6581 | 11319 | 24433 | -7593.43, -1773.54 | 3:00 | Helm |
| Uhk'loc | 6585 | 8129 | 24438 | -6278.97, -2000.4 | 6:00 | Shoulders |
| Blazerunner | 9376 | 1204 | 24430 | -7183.41, -1294.81 | 9:00 | Chest |
| Baron Charr | 14461 | 14517 | 590000 | -7153.77, -1085.68 | 12:00 | Waist |
| Tyrant Devilsaur | 6500 | 5240 | 23745 | -7464.14, -1321.6 | 14:00 | Legs |
| King Mosh | 6584 | 5305 | 24437 | -6854.36, -772.935 | 16:00 | Feet |

Times, slots and 60% base chances are authored Survivors tuning, not imported vanilla
loot. Every class resolves these six cells to its uncommon armor identities. Combat
scheduling, boss-only Keen Eye chances and collection/drop-page sources share the same
rows. Victory waits until 16:00 and all six bosses are defeated. Duplicate rolls retain
the existing no-additional-power rule. Wrist, hands and weapons still need sources.

The zone remains locked with **world content is not ready**, including on an otherwise
fully progressed hero. The rejected summit coordinates remain configured until the
candidate's gameplay placement is accepted. This source table does not approve those
coordinates, routes, camera, spawning or live combat. Downloaded SQL remains ignored.
