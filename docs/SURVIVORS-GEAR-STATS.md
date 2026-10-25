# Gear stat profiles

Initial Survivors tuning, shared by all classes. These are authored gameplay bonuses,
not the vanilla stats of the items used for names and appearances. Runtime definitions
live in `crates/survivors-core/src/gear_stats.rs`; gloves retain the requested examples
in `campaign::glove_bonus`.

Common grants the first two stats below. Each subsequent current grid band adds the
next stat and increases every existing stat. The existing bands are Common, Uncommon,
Rare, Epic, PvP and Legendary. This preserves saved collection indices; it does not
establish PvP as a final quality tier. Poor drops, distinct raid epics, their additional
stat and build-specific raid sets still require explicit content/source identities.

| Slot | Common | Uncommon adds | Rare adds | Epic adds | PvP adds | Legendary adds |
|---|---|---|---|---|---|---|
| Head | Armor, damage | Critical chance | Multistrike | Movement | Damage reduction | Critical damage |
| Shoulder | Armor, critical chance | Critical damage | Damage | Damage reduction | Area | Cooldown reduction |
| Chest | Health, armor | Damage reduction | Pickup | Damage | Critical chance | Multistrike |
| Waist | Health, pickup | Gold | Experience | Area | Damage reduction | Cooldown reduction |
| Legs | Armor, movement | Health | Area | Damage reduction | Damage | Cooldown reduction |
| Feet | Movement, pickup | Health | Gold | Area | Damage reduction | Damage |
| Wrist | Critical chance, cooldown reduction | Damage | Critical damage | Area | Health | Multistrike |
| Hands | Health, pickup | Area | Damage reduction | Damage | Cooldown reduction | Critical damage |
| Main hand | Damage, critical chance | Critical damage | Area | Multistrike | Cooldown reduction | Health |

For example, Common chest grants +10 health and +6 armor; Epic chest grants +25
health, +14 armor, +2% damage reduction, +18% pickup range and +6% damage. Common
boots grant +1% movement and +8% pickup range. Common wrists grant +1% critical
chance and +0.5% cooldown reduction. Gloves retain the user's specified coefficients.

Only the selected owned item in each slot contributes. Additional copies add no
power. Selecting a lower quality removes the higher quality's extra stats; cosmetic
choices do not affect bonuses. Existing full-set bonuses remain unchanged. This
changes the power of previously collected non-glove slots without changing ownership
or requiring a save migration. Combat balance and class-specific itemization remain
open; these values are not final release tuning.

Verification covers every slot/band's stat count and strictly increasing retained
stats, duplicate immunity, explicit downgrades, and propagation of mixed equipped
items into the actual run stat sheet. UI captures use isolated synthetic accounts;
they demonstrate presentation, not encounter balance or earned progression.
