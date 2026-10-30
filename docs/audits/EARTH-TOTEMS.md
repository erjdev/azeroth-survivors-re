# Earth totem prerequisites

## Checkout evidence

Read from `wow-data/Data` with `survivors-audit --spell-evidence`:

| Spell | Installed rule |
| --- | --- |
| 2484 Earthbind Totem | Summons entry 2630, 5 health, 45-second lifetime, 15-second recovery |
| 6474 Earthbind Totem Passive | Periodic trigger 3600 every 3000 ms |
| 3600 Earthbind | 10-yard radius, 50% movement slow for 5 seconds |
| 5730 Stoneclaw Totem rank one | Summons entry 3579, 50 base health plus level scaling, 15-second lifetime, category recovery 30 seconds |
| 5729 Stoneclaw Totem Effect rank one | 8-yard threat effect; raw effect 63, base points 21 plus one |

Talent evidence for Earth's Grasp 572 resolves to +25/50% Stoneclaw health
and +10/20% Earthbind radius. Implementing Earthbind alone does not complete it.
Stoneclaw's effect row does not establish an unconditional fixation duration;
resolve its actual threat/targeting policy before implementing that consumer.

Display audit: 4588 decodes `Creature/Spells/EarthElementalTotem.mdx` with
11 batches and 268 triangles. All referenced textures decode. This verifies an
earth-totem model, not the creature-entry mapping, race variants, particles or
native appearance. Those remain visual integration work; no assets are copied.

## Shared Earthbind lifecycle

`survivors-core::earthbind` implements five health, destruction, the 45-second
lifetime and three-second pulse cadence. Survivors schedules the first pulse
three seconds after deployment and excludes the exact expiry boundary; these
are explicit lifecycle choices, not inferred guarantees from the DBC interval.
The caller must replace its previous Earthbind on redeployment.

Pulses return their scheduled times so catch-up cannot make an expired slow
fresh. Empty pulses are consumed. Invalid or backwards clocks cannot duplicate
pulses or damage the totem; zero/invalid damage cannot heal it. Fatal damage
stops future pulses, and replacement creates a fresh independent state.

Four core regressions cover fine/coarse frame partitions, destruction, replacement,
invalid clocks/damage and integration with the existing source-owned slow state.
No card, runtime entity, health target or new talent definition is enabled yet.

## Remaining integration, in order

1. Deploy a grounded, separately owned Earthbind entity from a Shaman offer.
   Replace the previous Earthbind without removing unrelated fire totems.
   Define rank growth and show radius/recovery changes green and inline.
2. Let enemy contacts damage its real five-health state, with a documented
   targeting policy and the existing collision-safe approach. Destruction and
   expiry must stop pulses and remove the model; enemy hits must not redirect
   excess damage into the hero or activate hero-only defenses.
3. Apply zero-damage slows within the current totem radius using original pulse
   times. Extra targets must not cap the area or duplicate locations. Preserve
   stronger independent slows; leaving the radius allows the last slow to expire.
4. Verify actual deployment, replacement, target loss, damage/destruction, pause,
   death/reset and long-frame outcomes. Do not turn this into an unattended combat
   probe; use headless system tests and a menu-only visual fixture.
5. Verify the model and card at wide/compact sizes, capture native screenshots
   inline in CHANGELOG/README, then retain attended gameplay and Windows checks.
6. Add Stoneclaw's full health/threat behavior before enabling Earth's Grasp.

This is a shared-rule foundation, not a playable Earthbind or completed talent.
Coverage remains 308/432. No new rendered screenshot is claimed.

Validation: the eight-package scoped formatting, Clippy and test gate passes
with checkout data (31 optional-addon skips), including all four Earthbind tests.
The initial focused four-test run also passed. No native runtime behavior is
claimed from these shared-rule tests.
