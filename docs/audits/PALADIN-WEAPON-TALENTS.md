# Paladin weapon-specialization dependency

## Verified rules

Checkout Spell.dbc evidence resolves Protection node 1429 to five ranks of
One-Handed Weapon Specialization: 2/4/6/8/10% physical melee weapon damage.
Retribution node 1410 has three ranks of Two-Handed Weapon Specialization:
2/4/6%. Both use aura 79 with physical-school mask 1; the spell rows require
item class 2, with subclass masks 0xa891 and 0x21562 respectively. These facts do
not authorize boosting every spell merely because a weapon is equipped.

Reproduce with the checkout's data:

```sh
target/debug/survivors-audit wow-data/Data --talent-evidence 1429
target/debug/survivors-audit wow-data/Data --talent-evidence 1410
target/debug/survivors-audit wow-data/Data --spell-evidence 20196
target/debug/survivors-audit wow-data/Data --spell-evidence 20111
```

## Missing runtime consumer

The current Paladin pool in `survivors-core/src/data.rs` contains Hammer of Wrath,
Consecration, Holy Shock, Exorcism, Holy Wrath, Hammer of Justice, Divine Shield
and Judgement. Their configured spell IDs are 24275, 26573, 25912, 879, 2812,
853, 642 and 20271. Each resolves to school 1 (Holy) in checkout data. A `Strike`
ability shape only describes target selection; it does not make the hit a physical
weapon attack. There is no separate Paladin basic melee swing in the current run.

Existing Warrior specialization uses an explicit list of weapon attacks. Reusing
its one-/two-handed condition alone for Paladin would either produce no effect or
incorrectly boost Holy spells. Keep both Paladin definitions pending until their
actual physical-damage consumer exists. Coverage remains 300/432 (Paladin 25/44).

## Decision and integration

Pending user decision: add a basic automatic melee swing outside the six ability
slots, or a physical melee ability acquired through cards. This is separate from
the pending equipped-weapon-versus-authored-base damage decision for Stormstrike.
No new attack, slot policy or weapon damage formula is selected by this audit.

After resolving the attack policy:

1. Define its acquisition, range, cadence and damage source; use supported floor,
   collision and target visibility checks. Define how extra targets apply.
2. Use the selected hero's owned, worn weapon identity through
   `Loadout::melee_weapon`; appearance overrides must not supply power.
3. Apply 2% per rank only to eligible physical weapon damage, with five- and
   three-rank caps. Keep Holy abilities, periodic spells and combat pets independent.
4. Show the real conditional effect and green inline rank changes in Talents.
5. Verify purchase/refund/spec/save behavior, empty/unowned gear, one-/two-handed
   swaps, cosmetic independence, emitted hits and actual damage settlement. Capture
   native wide/compact talent previews; retain attended combat and Windows checks.

Validation: both complete talent rank reports and thirteen spell reports were
read from checkout data, including first/final specialization ranks and the actual
Holy Shock damage spell (25912). This is dependency evidence, not implementation
or visual acceptance. No installed assets are copied and no new screenshot is claimed.

Repository validation: the eight-package formatting/Clippy/test gate passes with
31 optional-addon skips. The existing rustdoc link gate also passes (150 benign
warnings). These checks do not enable or validate either missing talent.
