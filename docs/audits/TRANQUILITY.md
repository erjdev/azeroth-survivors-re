# Tranquility implementation evidence

This is an implementation dependency audit, not a completed spell. Inspect only
this checkout's `wow-data/Data`; no installed Applications game is used.

## Reproduce

```sh
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 740
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 842
```

The spell-evidence report now prints resolved effect radii, all five attribute
words and channel-interruption flags. A zero cast range does not imply zero area.

## Installed rules

| Field | Rank-one Tranquility (740) |
| --- | --- |
| Duration / pulse interval | 10 seconds / 2 seconds |
| Area | Effect radius row 9: 20 yards, no per-level radius scaling |
| Recovery | Category 46: 300 seconds; individual recovery is zero |
| Resource | 375 flat mana |
| Channel | `attributes_ex=0x40`, `channel_interrupt_flags=0x7c0c` |
| Healing effect | Type 35, aura 8, base points 93, die sides 1, level term 0.6 |
| Targets | Effect target A=1, B=0; tooltip names nearby group members |

The raw heal fields are not a final authored Survivors heal. The channel mask
contains movement-cancel bit `0x8`, handled by the stock client's
`spell/inflight.rs`. Survivors has a separate automatic ability loop and does not
inherit that lifecycle. Improved Tranquility (842; 17123/17124) reduces this
spell's threat by 50/100%; it does not increase healing in the installed data.

## Current runtime constraints

- `survivors-core::channel::Channel` already schedules discrete active-time pulses,
  delays the first pulse until one interval, deduplicates targets, supports cancel,
  and prevents repeated/backwards timestamps from replaying healing or damage.
- `survivors::channeling::Channels` wraps that clock specifically for **damage**:
  it emits `Hit` messages and owns enemy snares. A healing channel must not feed
  this damage path or accidentally trigger offensive procs.
- `HealingOverTime` pays continuous, source-owned healing. Reusing it directly
  would erase Tranquility's pulse cadence and maintained-channel distinction.
- `hero::tick` advances active run time and existing periodic healing. Channel
  updates run in the active-play chain before automatic casts; mode setup resets
  the damage-channel resource. Healing state needs equivalent pause/death/reset
  handling and an owned state visual.
- Enemy targeting is hero/nearby-pet/taunt based (`enemies.rs`), with bosses keeping
  hero pressure. There is no healing-generated threat ledger for node 842 to reduce.

## Required next decisions and implementation

The movement/attack policy is awaiting the user's answer: allow movement and
other automatic attacks, or cancel on movement and suspend attacks while
channeling. Do not silently choose that gameplay tradeoff from the stock mask.

Once settled, implement a healing-channel state using the shared pulse clock,
with cast-snapshotted per-pulse healing, one active instance and no immediate heal.
Specify authored amount, recovery, duration/rank scaling and recipients separately
from installed values. Extra-target bonuses must never duplicate an area at multiple
locations. If friendly area healing is supported, eligibility and range must be
validated per pulse; do not claim party healing for a self-only implementation.

Wire Gift of Nature, Nature's Focus and Tranquil Spirit to the implemented healing
and recovery consumers. Keep Improved Tranquility provisional until its meaningful
threat or explicit Survivors conversion is implemented and documented. A plain
healing multiplier must not be labeled as the installed threat behavior.

Acceptance must cover first/last pulse, long-frame catch-up, repeated timestamps,
pause, death before a due pulse, restart/mode exit, cancellation, recast exclusion,
source isolation from Regrowth/Rejuvenation, overheal, no offensive procs, talent
scope/refund, acquisition and green inline upgrades. Add native wide/compact UI
captures when the spell is exposed. Windows and attended combat remain separate.
