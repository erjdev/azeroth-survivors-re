# Controller input acceptance

## Evidence

The design reference `docs/intentions/survivors_paused_options_gamepad.png` shows
camera inversion, camera/zoom speed and a 25% stick dead zone. Current Survivors
settings implement inversion/speeds; `armory/controller.rs` and
`hud/settings/keyboard.rs` consume gamepad buttons for navigation. `hero.rs`
currently moves from keyboard/automation and orbits/zooms from mouse input.
No analog stick consumer or dead-zone control exists yet.

The locally resolved Bevy 0.18.1 `bevy_input/src/gamepad.rs` defines
`Gamepad::left_stick`/`right_stick` over processed axis values. `AxisSettings`
defaults to a ±0.05 per-axis dead zone and 0.01 event threshold. Applying a radial
filter to that already reshaped input would distort small diagonal gestures.
Runtime integration must deliberately configure the four stick axes for neutral
per-axis filtering, or consume appropriately normalized raw axis events. Do not
change trigger/button calibration or unrelated stock-client input.

## Implemented foundation

`survivors_core::controller` filters normalized raw sticks with a circular dead
zone (25% default, bounded to 0–50%). Remaining travel is continuously rescaled;
direction is retained, diagonal length is capped at one and invalid values cannot
produce movement. A frame groups movement/look/zoom/jump and clears every action
when inactive. Tests cover boundary drift, half/full travel, diagonal angles,
invalid preferences/device values and inactive-to-active samples.

This module has no live consumer yet. It does not establish controller support,
persistence, pause integration or physical-device acceptance.

## Required integration

1. Sample one selected connected device deterministically. Handle hotplug and
   disconnect by clearing current input; never combine unrelated controllers.
2. Apply the dead zone once. Preserve analog movement magnitude through
   `drive_hero` (its current normalization would erase partial travel), existing
   camera-relative movement, collision, jump and animation speed. Define keyboard
   precedence without making keyboard diagonals faster.
3. Map right-stick turning and bumper/trigger zoom through the camera's existing
   inversion, speed, pitch/zoom clamps and solid-world push-in. Use time-based
   rates; idle sticks must not change camera mode or requested zoom.
4. Gate all gameplay gestures on active play, living hero, uncovered world and
   closed modals. Consume blocked gestures rather than replaying them on resume.
   Keep menu South/Start actions from leaking into jump or pause transitions.
5. Persist a bounded dead-zone preference with backward-compatible defaults.
   Expose it with the reference-style slider, keyboard adjustment and category
   reset. Save/restart and displayed values must agree with filtering.
6. Test simulated Bevy device events, hotplug/disconnect, multiple pads,
   frame-rate independence, pause/level-up/menu/loading, mouse/keyboard handoff
   and drift. Capture compact/wide settings after wiring. Physical controller
   movement/camera feel and Windows acceptance require attended validation.
