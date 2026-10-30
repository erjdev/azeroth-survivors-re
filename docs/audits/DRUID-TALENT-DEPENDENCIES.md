# Druid talent dependencies — 2026-10-29

The initial checkout-data audit found 27/47 matching Druid definitions. Regrowth
and node 825 now have runtime consumers; nineteen nodes below remain provisional.
Current coverage is 28/47 Druid and 309/432 overall. Installed reference rules and
Survivors conversions remain distinct.

## Reproduce

Run `cargo run -p survivors-audit -- wow-data/Data --talent-gaps`, then
`--talent-evidence <node>` for the IDs below. Inspect referenced spells with
`--spell-evidence <spell>`. Only the checkout's installed data is evidence here.
Raw effect values require unit interpretation; tooltip references require their
referenced rows. No installed Applications game is used.

## Missing consumers

| Node | Installed rule | Required implementation |
| --- | --- | --- |
| 791 Natural Weapons | Physical attack damage +2/4/6/8/10%, all forms | A real Druid physical attack; do not apply to Nature/Arcane spells |
| 781 Natural Shapeshifter | Shapeshifting mana cost −10/20/30% | Form transitions and an explicit no-mana Survivors conversion |
| 788 Omen of Clarity | Melee-triggered Clearcasting; requires Natural Weapons 5 | Eligible melee events, proc policy and a consumable benefit; outer proc chance is not sufficient evidence of effective proc frequency |
| 793 Moonkin Form | Shapeshift, item-armor bonus, party spell-crit aura, Balance casting restriction | Form state, equipment-only armor accounting, aura ownership, cast eligibility and restoration on exit |
| 796 Ferocity | Reduced Maul, Swipe, Claw and Rake resource cost | Those four attacks and an explicit cost-to-recovery conversion |
| 795 Feral Aggression | Demoralizing Roar reduction +8%/rank; Ferocious Bite damage +3%/rank | Both abilities and their separate debuff/finisher consumers |
| 799 Feral Instinct | Bear threat +3%/rank and improved Prowl concealment | Form-aware attention and concealment, or separately documented Survivors conversions |
| 797 Brutal Impact | Bash/Pounce stun duration +0.5/1 sec | Both attacks and scoped stun duration |
| 804 Feral Charge | Charge, immobilize and interrupt | Collision-safe displacement, valid target selection and interrupt state |
| 802 Improved Shred | Shred Energy cost −6/12 | Shred and explicit no-resource conversion |
| 803 Predatory Strikes | Form melee attack power +50/100/150% of level | Form gating and authored level-to-damage formula; not a percentage multiplier on all damage |
| 800 Blood Frenzy | Cat combo-builder criticals: 50/100% chance for one extra combo point | Cat builder identity and target-owned combo accounting; requires Sharpened Claws 3 |
| 801 Primal Fury | Bear criticals: 50/100% chance for 5 Rage | Bear attack identity and explicit Rage conversion; requires Sharpened Claws 3 |
| 805 Savage Fury | Claw, Rake, Maul and Swipe damage +10/20% | Scope direct and periodic components to those attacks |
| 1162 Faerie Fire (Feral) | Rank-one armor reduction 175 for 40 sec; prevents stealth/invisibility | Form-compatible cast, shared debuff identity and concealment suppression |
| 822 Furor | 20%/rank chance on form entry for 10 Rage or 40 Energy | Transition events, distinct Cat/Bear benefit and no repeated grants from unchanged state |
| 826 Improved Enrage | Enrage immediately grants 5/10 Rage | Enrage lifecycle and explicit Rage conversion |
| 829 Reflection | 5/10/15% mana regeneration continues during casting | Explicit Survivors conversion; current design has no mana |
| 842 Improved Tranquility | Tranquility threat −50/100% | Tranquility plus threat handling or explicit conversion |
| 825 Improved Regrowth | Regrowth critical chance +10 percentage points/rank | Implemented: acquired Regrowth; direct-only 150% healing criticals, +10 points/rank; periodic healing stays independent and cannot crit |

## Form and triggered-spell evidence

| Form | Spell | Aura 36 form ID | Installed base-mana cost |
| --- | ---: | ---: | ---: |
| Cat | 768 | 1 | 55% |
| Bear | 5487 | 5 | 55% |
| Dire Bear | 9634 | 8 | 55% |
| Moonkin | 24858 | 31 | 35% |

All four spell rows have zero cast time, no individual recovery, and 1.5-second
start recovery. Form IDs are **not creature display IDs**. Race-specific model,
texture and animation mappings still require verification before visual work.
Moonkin passive 24905 increases item armor by 360%; aura 24907 grants 3 percentage
points spell critical chance. Neither is the existing Survivors milestone's 5-point
bonus. Do not silently substitute one for the other or count both as shapeshifting.

Clearcasting 16870 lasts 15 seconds and reduces the next eligible resource cost by
100%; Omen's outer ten-minute duration is not the proc lifetime. Blood Frenzy 16953
grants one combo point. Primal Fury 16959 grants 5 Rage; Furor references 17057
(10 Rage) and 17099 (40 Energy); Improved Enrage references 17080/17081 (5/10 Rage).
Rage magnitudes use tenths in the raw rows. Feral Charge 16979 has 8–25-yard range,
15-second recovery, and triggers 19675; inspect that row before implementing its
control duration. These are reference inputs, not adopted Survivors balance values.

## Integration order and acceptance

1. Define the Survivors form lifecycle: acquisition, six-slot interaction,
   switching trigger, eligible attacks, and recovery-based resource conversions.
   Keep the explicit no-mana design; do not add a resource bar solely to unlock a
   talent. Document authored values separately from installed facts.
2. Add shared-core form state and transition tests, then one complete Cat or Bear
   attack path. Verify invalid casts, pause, death/restart, target loss and repeated
   updates cannot duplicate transition bonuses. Runtime and future server must use
   the same rules.
3. Verify race-specific art and add a menu-only form specimen with native screenshots.
   Integrate body replacement, animation, equipment restoration and camera bounds.
   `benilla-world::model_forms` is a mesh cache, not a shapeshifting gameplay system;
   the stock stance-bar feed relies on server state and is not a Survivors simulator.
4. Enable only talents whose consumers exist. Test hero-only attribution, physical
   versus magical damage, individual attacks, combo ownership and form exit. Keep
   remaining nodes provisional and coverage unchanged until runtime wiring passes.
5. Reconcile existing explicit conversions in `talents/druid.rs`: Feline Swiftness,
   Sharpened Claws, Heart of the Wild and Leader of the Pack currently require no
   form; the Balance-30 milestone grants spell crit without a form. Any change must
   update descriptions, saves where necessary, and stacking tests together.
6. Finish Regrowth/Tranquility and non-form dependencies separately. Capture native
   starter/talent changes at wide and narrow sizes; attended gameplay and Windows
   validation remain separate acceptance steps.

## Healing foundation now wired

Rejuvenation now uses shared-core source-owned healing effects rather than a single
run-wide timer. Recasting refreshes only that spell, other sources retain their
remaining healing, and long frames pay only the unexpired amount. Health caps,
lethal-hit exclusion and the existing automatic below-85% trigger remain enforced
by the runtime. Swiftmend still leaves the periodic effect intact. Regrowth is now in the acquisition pool, with its direct heal, critical rules,
training and inline card text wired. Native menu captures cover the cards and
talent description; attended combat and Windows acceptance remain open.

Installed Regrowth rank-one spell 8936 has a direct healing
effect and a 21-second periodic heal with 3-second amplitude, a 2-second cast time,
40-yard range and 120 flat mana cost. Its direct value has a die range and level
term, so do not copy the base-point field as the final healing amount. The authored
Survivors conversion is documented in [the mechanics spec](../SURVIVORS.md#druid-regrowth-and-improved-regrowth).

## Existing healing talent integration

Installed Gift of Nature (828; spells 17104 and 24943–24946) increases all healing
by 2/4/6/8/10%. Moonglow (783; 16845–16847) explicitly includes Regrowth in its
3/6/9% mana-cost reduction. Both now include the acquired Regrowth path: Gift
scales direct and periodic healing; Moonglow uses the existing recovery conversion.
Improved Rejuvenation remains spell-specific. Coverage stays 28/47, since these
are existing definitions gaining a missing consumer, not new definitions.

Nature's Focus (823; 17063/17065–17068) names Healing Touch, Regrowth and Tranquility
in its installed pushback-protection rule. Its explicit +2%/rank healing conversion now includes Rejuvenation and Regrowth,
including both direct and periodic healing; Gift of Nature multiplies it. Healing
Touch is now acquired direct healing with the same conversion; Tranquility remains
separate work. The authored healing bonus
is not the installed 14/28/42/56/70% pushback-protection chance.

## Healing Touch dependency resolved

Acquired Healing Touch now uses spell 5185, direct self-healing and fixed 150%
critical healing. Installed Improved Healing Touch (824) names that spell with
0.1 sec/rank cast reduction; Tranquil Spirit (843) names Healing Touch/Tranquility
with 2%/rank mana reduction; Moonglow (783) includes Healing Touch at 3%/rank.
Their existing explicit recovery conversions now reach this ability, together with
Nature's Swiftness. Gift of Nature and Nature's Focus scale its healing. Existing
Rejuvenation benefits remain deliberate authored rules. Values and acceptance
limits are in SURVIVORS.md and the implementation plan; no coverage increase.

## Verified form models — 2026-10-30

The faction display mapping was checked against vMaNGOS's
[`GetShapeshiftDisplayInfo`](https://github.com/vmangos/core/blob/development/src/game/Spells/SpellAuras.cpp)
(read 2026-10-30). This is server-emulator reference evidence, not a field supplied
by the installed 14-column `SpellShapeshiftForm.dbc`. No external code was copied.
The local creature catalog and mesh/texture decoder independently verified all six
models from this checkout's `wow-data/Data`:

| Form | Night Elf display | Tauren display | Triangles (NE / Tauren) |
| --- | ---: | ---: | ---: |
| Cat | 892 | 8571 | 742 / 738 |
| Bear / Dire Bear | 2281 | 2289 | 673 / 735 |
| Moonkin | 15374 | 15375 | 1032 / 980 |

The catalog supplies distinct cat, bear and owlbear paths/skins for both factions;
all ten unique mesh textures decode. Reproduce with:

```sh
target/debug/survivors-audit wow-data/Data --creature-visuals 892,8571,2281,2289,15374,15375
```

Using the existing developer-only `WOW_SURVIVORS_MODEL_PREVIEW` with those IDs,
a native macOS menu-only capture at 1280×720 verifies each textured body.
Left to right: Night Elf Cat, Tauren Cat, Night Elf Bear, Tauren Bear, Night Elf
Moonkin, Tauren Moonkin. The settled preview report confirms an active animation
clip on all six; it does not establish every combat animation. The preview uses
creature display scaling, not final player transformation scaling or collision.
No combat was started; the app was closed and its temporary launcher restored.

![Faction-specific Druid form specimens](../images/screenshots/2026-10-30-druid-forms/faction-pairs.png)

This clears the base model/skin discovery prerequisite. Form acquisition and
selection, non-vanilla race policy, equipment hide/restore, player scale/collision,
camera bounds, attack consumers and Windows acceptance remain open. The user has
been asked whether forms are selected on the starter, acquired as run cards, or
manually switched during play; that decision has not been inferred from the
model evidence. No talent definition coverage changes in this checkpoint.
