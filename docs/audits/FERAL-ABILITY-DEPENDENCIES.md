# Feral ability dependencies — 2026-11-01

**Policy update:** [Approved combat rules](../SURVIVORS-COMBAT-POLICY.md)
supersede conflicting design requirements below. Installed-data evidence remains
reference material; it does not require reproducing rejected vanilla mechanics.

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
| Pounce 9005 → 9007 | Cat | Prowling and behind-target restrictions; two-second stun, triggered bleed, one combo point. Redesign as an ordinary range-gated stun/bleed; no Prowl, positioning or combo prerequisite. |
| Ferocious Bite 22568 | Cat | Combo-point damage, extra-Energy conversion and attack-power scaling. Raw combo coefficient 36 and damage-multiplier field 1 are insufficient to reproduce the complete server formula. |
| Demoralizing Roar 99 | Bear | Ten-yard area, 30-second attack-power reduction. Needs outgoing melee-damage conversion and strongest-effect/refresh semantics. |
| Enrage 5229 | Bear | Ten-second Rage generation, 60-second recovery and form-specific base-armor penalty. Use an authored recovery benefit; do not add a resource system. |
| Prowl 5215 | Cat | Persistent concealment with movement penalty. Rejected for Survivors: replace dependent talent benefits without implementing concealment. |
| Faerie Fire (Feral) 16857 | Cat/Bear | 30-yard range, six-second category recovery, 40-second armor reduction and suppression of stealth/invisibility. Needs shared debuff identity and physical mitigation; player stealth is not a prerequisite. |

Raw Rage cost fields use different units from displayed Rage. Preserve that
separation when interpreting Ferocity's two cost-modifier effects. Installed
recovery, category recovery and global recovery are separate fields; a zero
`recovery_ms` does not mean an ability has no recovery.

## Integration order

1. Implement the approved starter-kit policy (2026-11-01): tree-learned abilities
   replace the class default or augment the starting kit; at most three abilities
   by default and six maximum. Do not treat tree learning as merely enabling a
   future random card. Energy/Rage cost reductions become recovery reductions;
   no resource pools are planned. Automatic Claw/Maul remain outside slots under
   the earlier explicit decision. The follow-up confirms automatic melee attacks
   stay outside the six slots and should extend to Warrior/Rogue and other melee
   builds. Treat the three-starting cap as slotted abilities; add and validate
   the corresponding weapon consumers, including melee-focused Shaman.
2. Add form eligibility centrally to card generation, starter pool presentation,
   saved run setup and runtime cast validation. Test all four forms, no form,
   conflicting imports, six occupied slots and persistence. Preserve existing
   Druid spells. Never rely solely on a UI filter to enforce eligibility.
3. Implement Rake/Swipe and their physical hit/bleed semantics, then fully scope
   Ferocity and Savage Fury. Natural Weapons and Predatory Strikes must explicitly
   include or exclude new attacks; do not silently extend automatic-only wording.
4. Redesign Shred/Bite/Blood Frenzy without combo points or positional checks.
   Use normalized physical damage and authored recovery/proc effects; document
   the replacement rather than implementing a combo-point store.
5. Redesign Pounce/Feral Instinct without Prowl or opener state. Bash/Pounce must
   have actual stun consumers before claiming Brutal Impact complete. Charge
   additionally needs collision-safe displacement and an interrupt consumer.
6. Implement Roar, Enrage and Faerie Fire with their debuff/recovery behavior,
   then dependent talents. Threat becomes recovery or damage percentage;
   Energy/Rage becomes recovery reduction. Do not add resource or player-stealth
   systems to satisfy installed reference text.
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

## Starter-kit integration points verified — 2026-11-01

`hud::start_run` currently adds only `class.start`. `Run::finish_start` receives
the selected immutable talent snapshot afterward, making it the shared point
for applying tree-granted starter abilities across mouse, keyboard and production
preview setup. `armory::preview::selected_run` already calls both in that order.
Do not implement a separate preview-only kit.

The current explicit `AbilityUnlock` consumers are Priest Mind Flay/Silence and
Paladin Repentance. They presently gate random card acquisition, so the newly
approved policy requires migrating these existing talents as well as new feral
nodes. A Priest with both fits three slotted starting abilities; Paladin has two
slotted abilities plus its previously approved separate weapon attack. Preserve
rank-one cooldown initialization and prevent duplicate grants, invalid-class
imports or accidental modifications to an active run. If later tree combinations
exceed the starting limit, redesign grants to replace or augment existing abilities.
Do not add an overflow choice menu or silently drop grants in catalog order.

Tests must cover every startup route, no talent, both Priest unlocks, Paladin,
refund/spec switching, immutable active runs, the three-start cap and six-slot
acquisition ceiling. Starter labels and green descriptions must show the actual
kit; old text claiming only future card eligibility must be replaced.

The [starting-kit checkpoint](TALENT-STARTING-ABILITIES.md) implements the three
existing explicit unlocks through that shared initialization. This supersedes
their card-only behavior described above; new feral grants and redesign of any
over-cap combinations remain open.

## Rake/Swipe consumers — 2026-11-02

The [feral starting-kit checkpoint](DRUID-FERAL-KIT.md) now grants Rake for Cat and
Swipe for Bear alongside Wrath, through shared starting-kit initialization.
Form eligibility applies to starter pools, upgrade selection and runtime casts.
Rake has direct damage plus a separate bleed; Swipe uses one player-centered
area unaffected by extra targets. Both use normalized damage and independent
recovery. This supersedes their missing-consumer status above. Ferocity/Savage
Fury and the remaining redesigned attacks are still pending.

## Ferocity/Savage Fury consumers — 2026-11-02

Both nodes now have [scoped runtime effects and visual acceptance](DRUID-FERAL-TRAINING.md)
on Claw, Maul, Rake and Swipe. This supersedes their pending status above. Other
feral talents and redesigns remain unfinished; existing explicitly automatic-only
talents are not silently expanded by this checkpoint.

## Physical talent scope — 2026-11-02

[Physical integration](DRUID-FERAL-PHYSICAL.md) extends Natural Weapons, Sharpened
Claws and Leader of the Pack to Rake/Swipe, superseding their automatic-only
scope in earlier checkpoints. Predatory Strikes remains automatic-only. Other
feral redesigns and attended combat/platform validation remain open.
