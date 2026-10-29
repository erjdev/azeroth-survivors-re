# Druid talent dependencies — 2026-10-29

The checkout-data audit finds 27/47 matching Druid definitions; all twenty nodes
below remain provisional. Overall coverage remains 300/432. This document records
implementation dependencies, not additional completed talents.

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
| 825 Improved Regrowth | Regrowth critical chance +10 percentage points/rank | Regrowth direct/periodic healing and scoped healing criticals |

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
