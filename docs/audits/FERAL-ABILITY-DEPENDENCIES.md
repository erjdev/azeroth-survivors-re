# Feral ability dependencies — 2026-11-01

Permanent Cat/Bear forms, automatic Claw/Maul, Natural Weapons and Predatory
Strikes are implemented; see [form evidence](DRUID-PROGRESSION-FORMS.md).
The next work must supply actual abilities and their consumers before claiming
Ferocity, Savage Fury, Brutal Impact or the remaining feral talents complete.

## Installed evidence

Reproduce using only the checkout's install:

```sh
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 1822
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 22568
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 796
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 805
```

The audit now prints required/excluded form masks and each effect's combo-point,
multiple-value and damage-multiplier fields. These are raw fields, not an assertion
that a server formula is implemented. Form mask 0x1 is Cat; 0x90 includes Bear and
Dire Bear; 0x91 includes all three. Survivors has one permanent Bear choice.

| Spell | Installed form restriction | Relevant evidence and required consumer |
| --- | --- | --- |
| Claw 1082 | Cat | Energy cost 45; weapon damage plus a flat addition; one combo point. Existing automatic Survivors attack deliberately uses authored damage/recovery instead. |
| Maul 6807 | Bear | Raw Rage cost 150; next-attack damage addition. Existing automatic Survivors attack deliberately uses authored damage/recovery instead. |
| Swipe 779 | Bear | Raw Rage cost 200; nearby target count field 3; melee range. Decide authored area versus bounded-target behavior explicitly; never multiply an AoE location with +target. |
| Rake 1822 | Cat | Direct damage, nine-second bleed with three-second ticks, one combo point. Direct and periodic components need consistent physical provenance and separate critical handling. |
| Shred 5221 | Cat | Behind-target restriction, weapon-relative damage and one combo point. Tooltip literal bonus and raw effect values differ; do not infer the full damage formula from one number. |
| Bash 5211 | Bear | Two-second stun, 60-second category recovery. Needs ordinary/boss control rules and interrupt policy. |
| Pounce 9005 → 9007 | Cat | Prowling and behind-target restrictions; two-second stun, triggered bleed, one combo point. Must inspect the triggered row and supply a real Prowl lifecycle. |
| Ferocious Bite 22568 | Cat | Combo-point damage, extra-Energy conversion and attack-power scaling. Raw combo coefficient 36 and damage-multiplier field 1 are insufficient to reproduce the complete server formula. |
| Demoralizing Roar 99 | Bear | Ten-yard area, 30-second attack-power reduction. Needs outgoing melee-damage conversion and strongest-effect/refresh semantics. |
| Enrage 5229 | Bear | Ten-second Rage generation, 60-second recovery and form-specific base-armor penalty. Needs an explicit resource-free conversion or a resource system. |
| Prowl 5215 | Cat | Persistent concealment with movement penalty. Needs approach/exit behavior, opener consumption and attack interaction. |
| Faerie Fire (Feral) 16857 | Cat/Bear | 30-yard range, six-second category recovery, 40-second armor reduction and suppression of stealth/invisibility. Needs shared debuff identity, physical mitigation and perception integration. |

Raw Rage cost fields use different units from displayed Rage. Preserve that
separation when interpreting Ferocity's two cost-modifier effects. Installed
recovery, category recovery and global recovery are separate fields; a zero
`recovery_ms` does not mean an ability has no recovery.

## Integration order

1. Resolve acquisition and resource handling. Questions are pending: compatible
   form abilities in the existing six-slot level-up pool versus additional
   automatic attacks; recovery reductions versus introducing Energy/Rage. The
   approved automatic Claw/Maul and retained spell pool remain the current rules.
2. Add form eligibility centrally to card generation, starter pool presentation,
   saved run setup and runtime cast validation. Test all four forms, no form,
   conflicting imports, six occupied slots and persistence. Preserve existing
   Druid spells. Never rely solely on a UI filter to enforce eligibility.
3. Implement Rake/Swipe and their physical hit/bleed semantics, then fully scope
   Ferocity and Savage Fury. Natural Weapons and Predatory Strikes must explicitly
   include or exclude new attacks; do not silently extend automatic-only wording.
4. Implement target-owned combo accounting or an approved explicit conversion
   before Shred/Bite/Blood Frenzy. Existing Rogue talents use recovery conversions;
   there is no existing generic combo-point store to reuse.
5. Implement Prowl/opener lifecycle before Pounce/Feral Instinct; Bash/Pounce must
   both exist before claiming Brutal Impact complete. Charge additionally needs
   collision-safe displacement and a real interrupt consumer.
6. Implement Roar, Enrage and Faerie Fire with their complete debuff/resource
   behavior, then their dependent talents. Current enemy concealment can be
   overridden by an observer-owned Hunter's Mark; this is not yet generic
   Faerie Fire suppression. Scenery visibility is a separate concern.
7. Verify core formulas, real ECS casts/settlement, card exclusion and inline
   upgrades, native wide/compact purchase/refund/restart, then attended gameplay
   and Windows. Keep definition coverage distinct from this acceptance matrix.

No new gameplay ability or talent is counted complete by this audit. Installed
art remains checkout-local; no raw install files are committed.

Validation: all 19 audit-tool tests, package Clippy with warnings denied and
workspace formatting pass. The rebuilt tool was run against all listed spells,
including Pounce's triggered bleed; Cat/Bear masks and Bite's combo coefficient
were observed directly. This does not require a native gameplay build or new UI
capture because it changes read-only evidence output and planning only.
