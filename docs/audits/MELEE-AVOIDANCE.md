# Shared melee avoidance — 2026-10-29

The explicit checkout install reports the same parry ranks for Warrior Deflection
(130, spells 16462–16466), Paladin Deflection (1403, 20060–20064), and Rogue
Deflection (187, 13713/13853–13856): aura 47, base points 0–4 plus one die-side
point. Rogue Lightning Reflexes (186, 13712/13788–13791) uses aura 49 with the
same 1–5 percentage-point dodge ranks. Reproduce with:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 187
```

These now use the shared contact outcome system, joining Hunter Deflection.
The one-roll table resolves dodge before parry, then the ordinary armor/shield/hit
path. No base avoidance is invented. Invalid, immune, dead or untrained contacts
consume no avoidance roll. Avoidance retains attacker identity, consumes no shield
or health, and cannot trigger reflection or on-damage recovery. Non-contact damage
and pet victims keep their existing paths. Only trained Hunters can Counterattack.
Saved node IDs, ranks and specs are unchanged; old armor/reduction substitutions
are removed rather than stacked. Named definition coverage remains 300/432.

## Riposte dependency remains open

Installed node 301 requires Deflection rank five. Spell 14251 describes an attack
after a parry for 150% weapon damage, plus a six-second disarm. Parry was previously
unavailable to Rogues. This checkpoint supplies that prerequisite; it does not
pretend an arbitrary spell base is weapon damage or damage reduction is disarm.
Define shared equipped-weapon damage and enemy weapon/disarm eligibility before
implementing Riposte and its combat/visual acceptance. The same weapon dependency
applies to Stormstrike; preserve the pending user decision about that conversion.

## Verified checkpoint

296 core and 575 Survivors runtime tests pass with checkout data. Regressions cover
all ranks, exclusive dodge/parry boundaries, shields, non-contact damage, invalid
contacts, refund, duplicate import and invalid-spec suppression/recovery. Clippy
including tests and eight-package scoped checks pass (31 optional-addon skips).
Native incremental build: 20 seconds. Native purchases for all three classes,
Rogue refund/spec isolation, and restart retention pass. Wide/compact screenshots
are embedded in CHANGELOG; Windows attended combat and balance remain open.

## Shaman Parry — 2026-10-29

Installed node 616 teaches spell 18848 through spell 16268 (effect 36). The
passive uses effect 22, miscellaneous value 90, equipped class 2 and subclass
mask 0x2a5f3. Neither description supplies a numeric chance. Survivors explicitly
authors a 5% chance with a worn melee weapon; this is not claimed as a DBC-derived
probability. Cosmetic selections never provide weapon eligibility.

This replaces the prior flat reduction. Eligible contact is parried before
shields/health and does not reflect damage or grant Hunter Counterattack.
Non-contact damage is unchanged. Core tests cover class/rank bounds, weapon kinds,
refunds and green labels; runtime tests cover roll boundaries and actual damage.
Catalog-backed account tests verify purchase/save/spec/refund, unequipping,
cosmetic-only selection, re-equipping and stable earlier run snapshots.

Native macOS checks purchased, refunded, repurchased and restarted Parry in an
isolated profile. Wide and compact screenshots live in
`docs/images/screenshots/2026-10-29-shaman-parry/`. All 322 core tests and the
eight-package gate pass (31 optional-addon skips); incremental build took 18s.
Native equipment changes, Windows/controller and attended combat are not claimed.
