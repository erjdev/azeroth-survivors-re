# Demoralizing Roar / Feral Aggression integration audit — 2026-11-04

This is the next dependency plan, not implemented gameplay or additional talent
coverage. References come only from checkout `wow-data/Data`:

- `survivors-audit --spell-evidence 99`: Bear/Dire Bear mask `0x90`, ten-yard
  radius, 30-second melee attack-power reduction, Rage cost and no damage effect.
- `survivors-audit --talent-evidence 795`: five ranks, +8% Roar strength and
  +3% Ferocious Bite damage per rank. These are relative increases, not percentage
  points of incoming damage reduction. Neither talent effect grants an ability.

## Integration constraints

Preserve Roar's melee-defense identity with an authored percentage reduction,
independent of enemy weapon items or attack-power pools. It should be a Bear-only
acquired area ability, with no starter grant, no damage event and no proc generation.
Area scales its one player-centered radius; extra targets must not duplicate it.
Movement and other attacks continue while it fires. Recovery uses existing haste
and applicable feral recovery conversions. Ferocity's original Roar cost reduction
must be audited alongside its existing named scope before claiming full coverage.

Do not directly reuse `Effect::Weaken`: its consumer scales both contact damage
and Frostbolt. Do not reuse the Warlock `CurseState` ownership: replacing a curse
would remove Agony/Weakness and conflate unrelated effects. Store Roar's own
strength and expiry on the affected enemy. Equal strength may refresh; weaker
applications must not replace or extend a stronger active effect. Expired effects
must accept new weaker applications. Keep boss eligibility explicit.

Apply the reduction only on enemy physical contacts, including contacts redirected
to pets or Stoneclaw, before target mitigation. Preserve existing curse and general
weakening composition and all magic damage paths. At 20% authored base reduction,
five Feral Aggression ranks would mean 28% reduction, not 60%; final base strength,
duration and recovery remain tuning work.

The talent must separately increase Bite damage by 3% per rank, multiplying its
existing physical modifiers. Do not publish node 795 as complete after implementing
only one side. Add both scopes to inline green previews; current `Weaken` prose
uses the current strength for both sides, so it cannot be copied for a strength
upgrade without comparing the next effect.

## Required evidence

Verify Cat/Bear/no-form acquisition and starter limits; area, line-of-sight and
boss targeting; zero damage and absence of offensive procs; source-owned expiry,
stronger/equal/weaker refresh; actual contact damage against hero/pet/totem;
unchanged spell damage and curses; five talent ranks, refunds and unrelated spells.
Capture wide/compact cards and talent purchase/refund/spec/restart using production
UI. Complete scoped tests, native/WebGPU builds and the screenshot-backed branch
checkpoint. Attended combat and Windows remain separate acceptance work.
