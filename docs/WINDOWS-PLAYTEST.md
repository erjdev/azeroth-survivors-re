# Windows playtest follow-up

Current implementation reviewed at `03db910`. These reports remain open for
Windows scene acceptance; the checks below are not a claim that the reported
scenes have been reproduced.

| Report | Current behavior | Remaining acceptance |
| --- | --- | --- |
| Foliage hides the hero without a vision circle | Camera-to-hero rays test rendered scenery, including canopy triangles, independently of collision. Reveal fades over 500 ms. | Replay the obstructed view and the adjacent clear view on Windows. Check ordinary and retained rendering; transparent texture regions remain a known limitation of triangle picking. |
| Enemies spawn inside trees or stick at fences | Spawn placement checks body clearance; incremental local A* validates route edges with body/floor sweeps. | Check actual Elwynn placements, rotated fences, terrain and crowded waves. Search is bounded to 512 nodes and is not a complete zone navmesh or IDA*. |
| Excessive elite chests obstruct enemies | Elite rewards begin after 60 seconds, require 90 seconds between drops and stop at five per run. Boss rewards are independent. Chest geometry remains solid and participates in detours. | Verify live routing around clustered chests and confirm boss encounters do not overwhelm the intended reward cadence. |
| Talent artwork stretches or misaligns | Four stock tiles compose into one 300×331 painted image; panels retain that aspect ratio and icons use relative positions. | Review all classes on Windows at normal and enlarged UI scales, including compact and wide windows. |

Verification on 2026-10-27 at `03db910`: **422 Survivors app tests pass** on
macOS with checkout-local WoW data and `survivors-visibility` enabled. Coverage
includes canopy geometry, tree/chest hull detours, bounded routing, elite death
batches and talent tile composition. No Windows execution or new GPU capture is
claimed. The reported Windows build and affected talent class remain unknown.

Existing native visual evidence (not new captures):

![Continuous talent artwork at 1280×720](images/screenshots/2026-10-26-talent-art/talents-1280.png)

![Compact talent layout at 640×720](images/screenshots/2026-10-26-talent-art/talents-640.png)

When replaying a failure, retain the build commit, zone/position, camera view,
renderer settings and screenshot together so the same scene can be compared.

## Replay checklist

Record `git rev-parse --short HEAD` before building with `cargo survivors` so a
checkout update is not confused with an older executable. Keep each report open
until its failing view and an adjacent passing state have been checked:

- **Foliage:** stand still and orbit the camera from a clear view to a canopy
  covering the hero, then back. Reveal should fade in and out over 500 ms; a
  building behind the visible hero should stay opaque. Record whether the trunk
  or only the leaves obscure the hero.
- **Navigation:** record a stuck enemy's spawn point and the fence/tree blocking
  its route. Check that enemies take an available opening and that the opening
  is wide enough for their bodies. The current local search is bounded A*, not
  IDA*; a search-algorithm change alone does not establish valid spawn clearance
  or traversable geometry.
- **Chests:** count elite and boss chests separately. Elite chests should not
  appear before 1:00, should be at least 90 seconds apart, and should stop at five
  per run. Check enemy detours before and after a chest is collected. Boss chest
  counts are currently independent of this cap.
- **Talents:** capture the same class at the desktop's normal resolution and a
  compact window, including the UI scale setting. Tree art should retain its
  proportions; icons and rank labels should stay aligned while scrolling.

## Capture visibility evidence on Windows

Use the [Windows visibility trace workflow](SURVIVORS-DEVELOPMENT.md#diagnosing-scenery-reveal-on-windows)
for a short obstructed/clear camera replay with **Scenery cutaway** enabled. Keep
the log and screenshots together with the exact build commit. It distinguishes
missing rendered blockers from an active reveal that fails to appear on screen;
camera collision alone cannot establish whether foliage hides the hero.

## Additional talent artwork evidence

The existing Paladin capture below shows the composed artwork and aligned talent
rows on macOS. It is not a new capture or Windows acceptance evidence.

![Paladin talent artwork and alignment](images/screenshots/2026-10-27-sanctuary/preview.png)

## Talent spacing follow-up (2026-10-27)

After the report of short artwork and crowded icons, tree columns now wrap earlier
and keep equal widths rather than compressing three trees into narrow windows.
Native macOS checks cover 640×480, 800×600 and 1280×720; this does not close Windows
acceptance. On Windows, check narrow and wide windows, then scroll to the third
tree and saved specs; verify rank labels remain separated and every control is reachable.

![Taller compact talent trees](images/screenshots/2026-10-27-talent-spacing/640.png)
