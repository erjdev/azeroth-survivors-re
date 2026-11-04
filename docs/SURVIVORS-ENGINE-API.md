# Survivors engine API

These are intentional APIs of this fork, not promises of upstream compatibility.
The world-API audit records them separately from renderer internals still awaiting
encapsulation. Its leak ceiling remains unchanged.

## Scenery visibility

`benilla_world::survivors_visibility` owns render integration for the requested
see-through effect. Its public surface is:

- `plugin`: installs resource extraction and the shared-light-buffer upload.
- `SceneryCutaway`: focus position/radius and camera position/reveal strength.
  Default zero values disable the aperture. The game supplies the hero, settings,
  projection-derived radius and 500 ms fade; the engine owns shader transport.
- `SceneryProbe::blocked(eye, focus)`: tests rendered scenery along the sightline,
  excluding the final half yard at the focus. It queries ordinary meshes, merged
  blobs and retained static batches through `WorldPick`. The game does not need
  to enumerate renderer storage or inspect `PickBlob`.

This deliberately uses render geometry rather than walking/camera collision.
Only doodads, WMOs and game objects obstruct the probe; units do not. Existing
regressions exercise colliderless foliage, scenery behind the hero, both sides
of installed Elwynn canopy geometry, ordinary/merged batches and the fade/reset
lifecycle. Actual Windows scene acceptance remains a separate requirement.

## Launcher map selection

`world_map::set_start_map(id)` is the existing pre-build launcher override for a
server-less starting map, including platforms without process environment writes.
It accepts a `Map.dbc` ID and precedes the environment/default selection. It is
not an in-session travel API: runtime changes use the existing map state/events.
Publishing this existing launcher contract does not change startup behavior.

## Scene visibility ordering

`exterior_cull::ExteriorCullSet` is the fork's scheduling boundary for the engine's
fresh WorldUnit visibility verdict in PostUpdate. Gameplay concealment runs after
this set and before Bevy VisibilityPropagate; it may hide an admitted actor but
must preserve the current culling verdict when revealing it. The noncombat render
fixture uses the same ordering. This publishes the existing set without changing
the culling algorithm or granting access to renderer storage. A schedule regression
covers reveal while culled and concealment after admission. The leak ceiling is
unchanged.
