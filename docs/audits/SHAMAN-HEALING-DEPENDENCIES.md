# Shaman healing dependencies

Evidence read from this checkout's `wow-data/Data` with `survivors-audit`:
`--talent-evidence 1648`, `--spell-evidence 29203`, `--spell-evidence 331`.
The binary's coverage summary is stale; these raw DBC rows remain evidence of
installed mechanics only.

- Healing Wave 331: Nature school, healing effect 10, base points 33 plus die
  sides 11, 1500 ms casting time and 25 mana. These values do not define a
  Survivors cooldown or level curve.
- Healing Way 1648: three ranks, 33/66/100% chance to apply spell 29203 after
  Healing Wave, increasing subsequent Healing Waves on that recipient.
- Trigger 29203: 15000 ms duration, maximum three stacks, 6% healing per stack
  (raw base points 5 plus one die-side point).

## Implemented prerequisite

Healing Wave is an acquired direct self-heal with authored 40-health / 12-second
recovery tuning, using shared rank growth, below-85% readiness, health clamping,
and 150% healing criticals. Real cast-system tests cover health outcomes without
an enemy, no damage messages, and preserving RNG when not ready. Offer tests
cover acquisition and green inline healing/recovery values. Native card previews
cover 1280×720 and 640×720 / 125%. Preview picks are disabled, so the screenshots
are visual evidence rather than acquisition or combat evidence.

## Healing Way implementation

Node 1648 now uses the installed 33/66/100% proc chances and 6% / three-stack /
15-second rules. The Survivors conversion requires positive effective healing.
The cast applies existing stacks first, then rolls the new proc. It uses the run's
active elapsed time and clears on death and run reset. Other healing and damage
are excluded. The eight-package gate covers cast ordering, proc thresholds,
expiry/refresh/cap, invalid inputs, legal purchase, save/reload, specs and refunds.
31 optional-addon tests skip because their corpus is absent.

Native tooltip captures cover wide/compact layouts; native purchase input failed
with a computer-use window error, so interaction acceptance remains open alongside
Windows and attended gameplay. Existing totem training conversions still need
reconciliation before extending them to actual Shaman healing spells.
