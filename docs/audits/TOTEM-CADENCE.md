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
