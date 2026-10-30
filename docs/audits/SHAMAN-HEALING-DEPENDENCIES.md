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


## Healing Wave training — 2026-10-29

Fresh raw evidence: `--talent-evidence 592` gives Purification spells 16178 and
16210–16213, healing effectiveness +2/4/6/8/10%; `--talent-evidence 594` gives
Tidal Mastery spells 16194 and 16218–16221, healing/lightning critical chance
+1/2/3/4/5 percentage points. These magnitudes use raw base points plus one.

Purification now multiplies Healing Wave's healing by its named rank bonus.
Its existing totem deployment healing still adds to Restorative Totems before
multiplication; Restorative Totems itself does not affect Healing Wave. Tidal
Mastery now includes Healing Wave's real healing-critical roll while preserving
its existing lightning/shock scope. Totem deployment healing remains noncritical.
Neither change affects other-class heals, shields, regeneration or spell recovery.
Healing Way continues to multiply the resulting Healing Wave cast healing.

Tests cover every rank, invalid classes, capped imported ranks, refunds, actual
critical/noncritical healing, no-damage outcomes and no RNG consumption when full.
Critical test rolls lie inside the newly granted chance, proving the new consumer.
Catalog-backed purchase/save/reload/spec/refund checks cover both nodes. Definition
coverage is unchanged: these are corrected consumers for two existing definitions.

Native tooltip captures cover both nodes at 1280×720 and 640×720 / 125% UI size.
A native purchase attempt returned `noWindowsAvailable`; the same app still
rendered correctly, but no purchase acceptance is claimed. Windows, controller
and attended combat remain open. Incremental native build: 16 seconds.
Improved Healing Wave, Tidal Focus and Ancestral Healing still retain their older
explicit totem conversions and require separate reconciliation.

Final verification: all 321 core tests and the eight-package scoped gate pass;
31 optional-addon tests skip because their corpus is unavailable. Earlier runs
identified two obsolete exclusion/tooltip expectations, updated for the new scope.


## Native saved-spec acceptance — 2026-10-29

The earlier automation failures are superseded for purchase/refund/save by this
successful macOS run of the `fd8da1a` implementation. An isolated Fenn Preview
profile started with 25 legal Restoration points and 26 unspent. Pointer input
trained Healing Way to 3/3, Tidal Mastery to 5/5 and Purification to 5/5: 38 spent,
13 unspent. Right-clicking each once produced 2/3, 4/5 and 4/5: 35 spent, 16 unspent.
Spec 2 stayed empty with 51 available; returning to spec 1 and restarting retained
all three refunded ranks and the point totals. The saved row independently
confirms `2:1648:2,2:594:4,2:592:4` in the first spec and empty other specs.

Wide captures include all three node counters, totals and independent-spec state.
A compact 640×720 / 125% capture confirms the restored Healing Way description.
Screenshots are stored in the Healing Way and Shaman healing training milestone
folders. Preview configuration selects the inspected talent; purchases and refunds
used native input. No account data was edited between purchase and restart.
Windows, physical controller, compact purchase of these nodes and attended combat
remain open. No runtime source changed; the preceding green eight-package gate
and 321 core tests remain applicable.

## Ancestral Healing decision pending

Raw node 581 references triggers 16177, 16236 and 16237. Their aura 101 values
are 8/16/25% armor and all last 15 seconds. The current authored conversion grants
12/24/36 flat armor for 10 seconds after effective Searing/Magma deployment
healing, without a critical roll. The new Healing Wave critical-event producer
makes the original trigger implementable. The user has been asked whether to
replace the old conversion, retain both effects or keep the existing behavior.
Do not treat that question as approval; the implementation remains unchanged.
