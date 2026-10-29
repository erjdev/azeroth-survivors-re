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
30 yards. Survivors now has an acquired Curse of Weakness and node 1006
uses those exact rank bonuses.

Curse-owned control is separate from Frostbolt, roots, channel slows and other
effects, allowing replacement without erasing or extending unrelated effects. The
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

Ordinary timed snares now also retain independent source expiry; see
[SLOWS.md](SLOWS.md). Curse ownership remains separate from those sources and
from connected channels.

## Acquired Curse of Weakness — 2026-10-29

Authored Survivors rank-one balance is **3 less damage per physical hit**, a
**12-second** duration, **6-second** recast and **30-yard** range. This intentionally
shortens the installed duration for horde combat. Ability ranks scale potency by
30% and duration by 10% of the base per rank, reduce cooldown by the ordinary
rank curve, and add targets at ranks 3/5/7. Extra-target stats affect these
individual victims; area bonuses never create additional curse locations.

Improved Curse of Weakness multiplies potency by 1.06/1.13/1.20. Its own legendary
power also multiplies potency; global damage, Shadow damage and critical bonuses
do not. Grim Reach affects its targeting distance. Cards and talent descriptions
show the same changing quantities inline in green, with one unit per change.
The ability and pool entry are appended to preserve older catalog indices.

One curse from the current solo player may affect a target. Weakness removes
Agony's periodic effect, supplemental Exhaustion slow and aura mark; Agony removes
Weakness's reduction and aura mark. Reapplying Agony replaces its old damage-over-
time effect rather than stacking copies. Corruption, unrelated controls and other
marks remain intact. Only original applications can replace curses. Invalid or
older timestamps cannot replace current curse state.

Outgoing physical contact damage first loses the flat reduction (floored at zero),
then receives percentage weakening and the victim's armor/reduction. The same
attacker-side reduction reaches hero and pet contact paths. Magical/environmental
sources do not use this physical-contact path. Zero damage cannot heal the victim.
Death resets all curse state while keeping corpse movement cleanup available.

The current solo run owns the curse. Multiplayer needs caster-scoped curse ownership.
Periodic catch-up still follows the shared settlement-time damage pipeline; historical
sub-frame ordering of replacements and overdue periodic ticks remains unverified.
Attended combat visuals, Windows acceptance and balance remain open. Native macOS purchase/refund, spec isolation, save/relaunch and card/talent layouts
at 1280×720 and 640×720 pass. Screenshots are embedded in the changelog.
All 290 core and 566 runtime tests pass with explicit checkout data; Clippy including
tests and eight-package scoped checks pass (31 missing-addon skips).
The native incremental build took 2m52s.

The pre-fix hit-pipeline regression reproduced `(speed=0.5, duration=12)`
instead of `(speed=0.5, duration=2.5)`. Validation of the independent clock is
recorded in the changelog. The preceding independent-clock checkpoint passed all 560 Survivors runtime tests with the explicit
checkout data path, including the prior Scorch school tests. Feature Clippy and
eight-package scoped checks pass (31 missing-addon skips). These rules do
not claim multiplayer caster ownership or attended combat acceptance.

Verification commands for this checkpoint:

```sh
WOW_DATA="$PWD/wow-data/Data" cargo survivors-test
WOW_DATA="$PWD/wow-data/Data" cargo survivors-clippy -- -D warnings
WOW_DATA="$PWD/wow-data/Data" scripts/check.sh
```
