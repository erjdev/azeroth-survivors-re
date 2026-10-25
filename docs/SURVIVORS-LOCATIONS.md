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

