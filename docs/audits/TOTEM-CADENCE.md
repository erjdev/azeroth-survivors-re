# Totem cadence across long frames

Searing and Magma Totems previously reset their attack timers after one pulse
per update. They also expired before processing the final update's due attacks.
This dropped attacks during stalls and accumulated cadence drift on normal frames.

The real `tick_areas` system now advances through only the totem's remaining
lifetime, preserves the timer remainder and processes every due attack before
fading. Searing retains its one-second cadence, Magma its two-second cadence,
and both retain the initial 0.3-second delay. Empty pulses consume their scheduled
time without banking attacks for a future target. Target selection, visibility,
damage rolls and missile travel still use the current update's world snapshot;
this is not a historical-position replay or the future fast-mode feature.

`totem_timing_tests.rs` exercises the actual system with a 5.25-second lifetime
partitioned into 0.1-second frames, 0.5-second frames or one six-second frame.
It checks five Searing attacks or three Magma pulses, matching emitted damage,
zero-delta inactivity, expiry/fade and no later attacks. A separate case leaves
the target unavailable through 2.8 seconds, then checks that only subsequently
scheduled attacks fire. Both regressions fail before the fix: the large frame
emits zero Searing hits instead of five, and the later-target case drops a hit.

This is headless runtime evidence. No UI change or new rendered screenshot is
claimed; attended gameplay and Windows remain unverified. Talent definition
coverage is unchanged. The existing Playing/loading-screen schedule gates remain
in place; this change does not alter pause or run-speed policy.

Validation: the eight-package scoped formatting, Clippy and test gate passes,
including both new regressions (31 optional-addon skips). The pre-fix failures
and final gate were captured in `/tmp/totem-timing-before.log` and
`/tmp/totem-timing-check.log` during this session.
The incremental native macOS play build passes in 1m07s; its existing linker
warning and the `block` dependency's future-compatibility warning remain.

## Fire-totem placement — 2026-10-30

Searing and Magma used an unchecked two-yard horizontal offset at the hero's
height. Their actual cast system now grounds that candidate with the shared
`Floor` support query and checks body clearance plus connectivity from the hero.
A rejected offset falls back to supported hero feet. If neither position is
valid (including absent world collision), no unit or cast effect is emitted;
the normal 0.2-second unsuccessful-cast retry applies instead of full recovery.
Existing attack cadence, damage, lifetime and stacking are unchanged.

`fire_totem_placement_tests.rs` casts both abilities against real synthetic
physics geometry. It covers missing support, a 1:2 slope and a solid wall at
the preferred offset. Before the fix, the missing-ground test spawned a floating
totem and the slope test spawned at `(2, 0, 0)` instead of `(2, 1, 0)`.
These are headless runtime regressions, not attended map or visual acceptance.
No interface changed, so this checkpoint adds no screenshot.

Both new actual-cast tests pass after the fix (`/tmp/fire-totem-after.log`);
the final native play-profile build passes in 14.08 seconds
(`/tmp/fire-totem-final-build.log`) after Clippy’s behavior-preserving `Option::and` simplification.

The full eight-package formatting, Clippy and test gate passes with 31 expected
optional-addon skips (`/tmp/fire-totem-check.log`). Two existing deployment tests
now supply real floor geometry while retaining their original healing, cooldown,
range and Totemic Mastery assertions; both pass alongside the new regressions.
