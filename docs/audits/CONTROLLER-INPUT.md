# Controller input acceptance

## Evidence

The design reference `docs/intentions/survivors_paused_options_gamepad.png` shows
camera inversion, camera/zoom speed and a 25% stick dead zone. Current Survivors
settings implement inversion/speeds; `armory/controller.rs` and
`hud/settings/keyboard.rs` consume gamepad buttons for navigation. `hero.rs`
now also consumes analog movement/look, bumper/trigger zoom and South jump.
The dead zone remains fixed at 25%; its settings control is still required.

The locally resolved Bevy 0.18.1 `bevy_input/src/gamepad.rs` defines
`Gamepad::left_stick`/`right_stick` over processed axis values. `AxisSettings`
defaults to a ±0.05 per-axis dead zone and 0.01 event threshold. Applying a radial
filter to that already reshaped input would distort small diagonal gestures.
The runtime therefore consumes normalized raw axis events. It does not alter
trigger/button calibration or unrelated stock-client input. A synthetic custom
per-axis calibration verifies that gameplay remains independent of those processed
stick values.

## Implemented foundation

`survivors_core::controller` filters normalized raw sticks with a circular dead
zone (25% default, bounded to 0–50%). Remaining travel is continuously rescaled;
direction is retained, diagonal length is capped at one and invalid values cannot
produce movement. A frame groups movement/look/zoom/jump and clears every action
when inactive. Tests cover boundary drift, half/full travel, diagonal angles,
invalid preferences/device values and inactive-to-active samples.

## Runtime integration

`controller_input::sample` consumes raw stick events after Bevy input processing,
keeps one connected device selected and clears disconnected-device state. Keyboard
movement takes precedence; partial stick travel retains its strength through
camera-relative movement, collision and animation. Right-stick camera motion and
bumper/trigger zoom use time-based rates, existing inversion/speed preferences and
camera bounds. South jumps; Start requests pause.

Menu, level-up, pause, death, loading, open settings and focus loss suppress pad
gameplay. Resume and device handoff require neutral sticks and released action
buttons, preventing held menu gestures from becoming movement or a jump. Synthetic
Bevy-event tests cover these boundaries, device selection/disconnection and raw
versus processed axes. Camera tests compare frame subdivisions and inversion.
These are automated checks, not physical-device or attended gameplay acceptance.

## Acceptance checklist (remaining items stay required)

1. Implemented: sample one selected connected device deterministically. Handle hotplug and
   disconnect by clearing current input; never combine unrelated controllers.
2. Implemented: apply the dead zone once. Preserve analog movement magnitude through
   `drive_hero`, camera-relative movement, collision, jump and animation speed.
   Keyboard input takes precedence without faster diagonal movement.
3. Implemented: map right-stick turning and bumper/trigger zoom through the camera's existing
   inversion, speed, pitch/zoom clamps and solid-world push-in. Use time-based
   rates; idle sticks must not change camera mode or requested zoom.
4. Implemented: gate all gameplay gestures on active play, living hero, uncovered world and
   closed modals. Consume blocked gestures rather than replaying them on resume.
   Keep menu South/Start actions from leaking into jump or pause transitions.
5. Persist a bounded dead-zone preference with backward-compatible defaults.
   Expose it with the reference-style slider, keyboard adjustment and category
   reset. Save/restart and displayed values must agree with filtering.
6. Test simulated Bevy device events, hotplug/disconnect, multiple pads,
   frame-rate independence, pause/level-up/menu/loading, mouse/keyboard handoff
   and drift. Capture compact/wide settings after wiring. Physical controller
   movement/camera feel and Windows acceptance require attended validation.
