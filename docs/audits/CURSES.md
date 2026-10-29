# Curse ownership and control clocks

## Evidence and dependency

Read only from this checkout's `wow-data/Data`:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1006
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 702
```

Improved Curse of Weakness (node 1006, three ranks) increases its curse's effect
by **6%, 13%, 20%**. Spell 702 reduces physical damage by a flat amount and permits
one curse per Warlock on a target. Its installed duration is 120 seconds and range
30 yards. Survivors has no acquired Curse of Weakness yet; node 1006 stays pending.

Before that ability can replace Curse of Agony correctly, curse-owned control
must be separate from Frostbolt, roots, channel slows and other effects. The
existing Curse of Exhaustion conversion attaches a 12-second slow to Agony;
it remains a supplemental slow, not a separate acquired spell.

## Confirmed clock defect

The shared `Enemy.slow`/`slow_t` path took the minimum speed and maximum duration.
Applying a 30% curse slow to a target already slowed 50% for 2.5 seconds therefore
kept the **50%** reduction for **12 seconds**. Those values belong to different
spells. Curse refreshes must neither extend nor erase another spell's clock.

## Runtime contract

- Each target owns a separate curse-slow clock, measured in active run time.
- Effective movement takes the strongest active ordinary, curse and channel slow.
- Refresh replaces the curse's own potency and expiry; no stacking or borrowing
  another effect's duration. An expired stronger slow reveals the still-active curse.
- Only original Agony applications apply the curse slow. Periodic ticks cannot
  refresh it. Bosses retain the existing 25% slow cap.
- Death clears the state and despawn removes it; invalid or older timestamps
  cannot revive it. The empty component remains on corpses so the movement system
  can still stop their movement.

Ordinary timed snares still share the existing ordinary-control slot; source-by-source
expiry within that slot remains a separate control-system gap. This change isolates
the curse clock from that slot and from connected channels.

## Subsequent Curse of Weakness work

Add an acquired ability, exact rank bonuses and green inline potency previews.
Keep physical outgoing reduction separate from damage multipliers and prevent
negative damage. Preserve unrelated controls when replacing a curse, remove the
replaced Agony periodic effect and its own slow, and test replacement in both
directions. Verify acquisition, legal talent purchases/refunds, independent specs,
save/reload and native wide/compact previews before counting node 1006 complete.

The pre-fix hit-pipeline regression reproduced `(speed=0.5, duration=12)`
instead of `(speed=0.5, duration=2.5)`. Validation of the independent clock is
recorded in the changelog. All 560 Survivors runtime tests pass with the explicit
checkout data path, including the prior Scorch school tests. Feature Clippy and
eight-package scoped checks pass (31 missing-addon skips). These rules do
not claim that Curse of Weakness or multiplayer caster ownership is implemented.

Verification commands for this checkpoint:

```sh
WOW_DATA="$PWD/wow-data/Data" cargo survivors-test
WOW_DATA="$PWD/wow-data/Data" cargo survivors-clippy -- -D warnings
WOW_DATA="$PWD/wow-data/Data" scripts/check.sh
```
