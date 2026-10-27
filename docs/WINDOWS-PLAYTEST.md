# Windows playtest follow-up

Current implementation reviewed at `cf7cc59`. These reports remain open for
Windows scene acceptance; the checks below are not a claim that the reported
scenes have been reproduced.

| Report | Current behavior | Remaining acceptance |
| --- | --- | --- |
| Foliage hides the hero without a vision circle | Camera-to-hero rays test rendered scenery, including canopy triangles, independently of collision. Reveal fades over 500 ms. | Replay the obstructed view and the adjacent clear view on Windows. Check ordinary and retained rendering; transparent texture regions remain a known limitation of triangle picking. |
| Enemies spawn inside trees or stick at fences | Spawn placement checks body clearance; incremental local A* validates route edges with body/floor sweeps. | Check actual Elwynn placements, rotated fences, terrain and crowded waves. Search is bounded to 512 nodes and is not a complete zone navmesh or IDA*. |
| Excessive elite chests obstruct enemies | Elite rewards begin after 60 seconds, require 90 seconds between drops and stop at five per run. Boss rewards are independent. Chest geometry remains solid and participates in detours. | Verify live routing around clustered chests and confirm boss encounters do not overwhelm the intended reward cadence. |
| Talent artwork stretches or misaligns | Four stock tiles compose into one 300×331 painted image; panels retain that aspect ratio and icons use relative positions. | Review all classes on Windows at normal and enlarged UI scales, including compact and wide windows. |

Verification on 2026-10-27: all **385 Survivors app tests passed** on macOS with
checkout-local WoW data and `survivors-visibility` enabled. Coverage includes real
canopy geometry, tree/chest collision hull detours, bounded routing, elite death
batches, and talent tile composition. No new Windows execution or GPU capture
is claimed in this review.

Existing native visual evidence (not new captures):

![Continuous talent artwork at 1280×720](images/screenshots/2026-10-26-talent-art/talents-1280.png)

![Compact talent layout at 640×720](images/screenshots/2026-10-26-talent-art/talents-640.png)

When replaying a failure, retain the build commit, zone/position, camera view,
renderer settings and screenshot together so the same scene can be compared.
