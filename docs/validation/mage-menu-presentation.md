# Mage menu presentation check — 2026-10-25

The roughly one-second stalls recorded during Winter's Chill UI validation are
associated with screenshot inspection. They are not sufficient evidence of a
talent-system regression. No rendering or gameplay code was changed for this check.

## Setup

- Source: Winter's Chill implementation committed as `db87cd3`. The reused play
  binary was built before that commit and reports its prior stamp, `c4b4119`.
- Checkout-only Survivors Checkout Validation app; isolated `chill-talents-state`
  account, Mage Talents page, 1280×720 logical window, normal vsync.
- Apple M1 Max, Metal, macOS 14.8.9. GPU pass timings unavailable, not zero.
- Native screenshot API via CUA. No game input during either measured interval.
- Second run enabled the existing `WOW_FPS_JOURNAL` to the checkout's
  `target/survivors-validation/presentation-journal.csv`. The
  [committed excerpt](mage-menu-presentation.csv) retains journal seconds 25–81.

## Observations

First run: no hitch warnings during 12:49:29–12:50:09 UTC. Three screenshot calls
were followed by 982 ms and 956 ms warnings at 12:50:23 and 12:50:29.
A subsequent quiet interval did have one 359 ms hitch; quiet operation is not
claimed hitch-free in general.

Journal repeat: 40 quiet rows at t=25–65 seconds had mean frame times
15.13–16.70 ms (median of row means 16.40 ms). Per-row p95 values were
19.24–20.03 ms. Main-thread CPU means were 4.81–5.02 ms; whole-process CPU
means were 10.77–11.32 ms. These are per-interval summaries, not whole-run
percentiles or a claim that every frame met 16.67 ms.

Three screenshot calls reproduced:
- 12:53:48.021 UTC: 994 ms frame hitch.
- 12:53:50.449 UTC: 1005 ms frame hitch.
- 12:53:53.087 UTC: 1003 ms frame hitch.

The five journal rows at t=74–81 span inspection/recovery. Their frame means
were 11.91–66.55 ms, with main-thread CPU means 4.71–16.63 ms.

Saved samples from the original run show the render thread inside
`CAMetalLayer nextDrawable` → `CAMetalLayerPrivateNextDrawableLocked` →
`_dispatch_semaphore_wait_slow`. The main thread waits in Bevy's
`renderer_extract` / `futures_lite::block_on`. The first sample had
171 of 173 render-thread samples in `nextDrawable`, and 171 main-thread
samples waiting on a condition variable. The follow-up inspection sample
also contains `nextDrawable`. Local raw evidence remains in:
`target/survivors-validation/chill-talents-state/Diagnostics/stall-81201-*.txt`
and `stall-82557-95315.txt`.

## Consequence and limits

Separate screenshot/AX inspection sessions from performance acceptance. Use the
frame journal or existing automated combat telemetry without CUA observations
during measured intervals, then inspect the resulting files. This experiment
supports capture-associated presentation waits; it does not establish why macOS
holds a drawable, rule out all game-side hitches, or prove combat performance.

Both validation runs closed normally; the normal launch wrapper was restored.
The broader resolution matrix, world traversal and combat frame-budget checks
remain open. No runtime fix is claimed by this diagnostic checkpoint.
