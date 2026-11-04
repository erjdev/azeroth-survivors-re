# Tranquility integration — 2026-11-03

## Installed evidence

Read only from this checkout's `wow-data/Data`:

```sh
target/debug/survivors-audit wow-data/Data --spell-evidence 740
target/debug/survivors-audit wow-data/Data --talent-evidence 842
```

Rank-one Tranquility (740) has a ten-second duration, two-second periodic healing
amplitude, twenty-yard radius, and a five-minute category recovery. Its tooltip
targets nearby group members and requires channeling. Raw healing includes a die
range and level term; the base-points field is not the final heal amount.
Improved Tranquility (842; spells 17123/17124) reduces its healing threat by
50/100%. Survivors must convert that to recovery under the approved combat policy.
Movement and other attacks must continue throughout its independent channel.

## Healing clock prerequisite implemented

`HealingOverTime::refresh_pulsed` schedules a fixed number of equal pulses. The
first payment occurs after one interval. Continuous heals coexist independently;
recasting replaces only the matching source and resets that source's clock.
Long frames settle every due pulse, cap payment to the unpaid total, then expire.
Invalid schedules and zero/invalid elapsed time preserve existing effects.
A double-precision elapsed clock avoids losing final pulses through repeated
single-precision frame subtraction. The caller remains responsible for active
simulation time, health caps and life eligibility.

The ten-package scoped format/Clippy/test gate passes (31 optional addon-corpus
skips), as does the WebGPU check with six existing warnings. Build storage is
59.97 GiB, within budget. This is not the full landing gate.

372 core tests pass. Added tests check pulse boundaries, concurrent continuous
heals, long frames, 60 Hz/subsecond partitions, refresh, invalid values and no
post-expiry payment. The existing runtime uses this shared clock in regeneration;
this checkpoint does not schedule Tranquility from gameplay yet. No interface or
asset changes are made, so no new visual capture is claimed.

## Remaining integration

- Author explicit Survivors healing/recovery scaling and recipient policy,
  retaining the area-healing identity and owner attribution.
- Add acquisition and six-slot accounting, green inline pulse/area previews,
  moving visuals, independent cast recovery and active-time scheduling.
- Integrate Improved Tranquility, Tranquil Spirit, Nature's Focus, Gift of Nature,
  Tree of Life, Reflection and Subtlety with their documented spell scopes.
- Verify real healing settlement, health caps, no resurrection, pause/death/run
  reset, moving casts, concurrent attacks, refunds/specs and saved restart.
- Capture cards/talents at wide and compact sizes, then perform platform and
  attended combat acceptance. Coverage remains Druid 35/47, total 329/432 until
  actual ability/talent consumers exist.
