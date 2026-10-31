# Remaining Mage talent dependencies

Evidence: this checkout's `wow-data/Data`, read using `survivors-audit
--talent-evidence NODE` for the eleven nodes below and `--spell-evidence ID`
for their prerequisite spells. Raw installed rows establish reference mechanics,
not Survivors balance or implementation. The audit binary's coverage summary is
stale; current source coverage remains 38/49 Mage definitions, 308/432 overall.

| Node | Installed mechanic | Required consumer or decision |
| --- | --- | --- |
| 78 Wand Specialization | +13/25% wand damage | Worn-wand attack identity and damage; cosmetics and ordinary spells must not qualify |
| 75 Arcane Concentration | 2/4/6/8/10% on damaging spell hits; Clearcasting makes the next damaging spell free | One-use proc state and a confirmed no-mana conversion; half recovery for the next damaging spell is proposed, awaiting the user |
| 82 Magic Attunement | Amplify/Dampen Magic effectiveness +25/50% | Actual recipient-owned spell-damage/healing modifiers for both buffs |
| 85 Arcane Resilience | Armor equal to 50% of Intellect | A real Intellect quantity or an explicitly authored replacement |
| 88 Improved Counterspell | 50/100% chance to silence for 4 seconds | Counterspell acquisition, enemy casting, interruption, school lockout and silence |
| 1142 Arcane Meditation | 5/10/15% mana regeneration continues while casting | Explicit no-mana conversion; no existing mana regeneration consumer |
| 77 Arcane Mind | Maximum mana +2/4/6/8/10%; requires Arcane Resilience | Explicit no-mana conversion; installed text does not say Intellect |
| 23 Burning Soul | 35/70% resistance to Fire casting pushback; Fire threat −15/30% | Interruptible Fire casts and spell-specific threat accounting |
| 24 Improved Fire Ward | 10/20% chance to reflect Fire spells while Fire Ward is active | School-aware incoming spell events, ward ownership and nonrecursive reflection |
| 1639 Master of Elements | Fire/Frost criticals refund 10/20/30% of base mana cost | Explicit recovery/resource conversion, with cast identity and a defined multi-hit refund policy |
| 70 Frost Warding | Frost/Ice Armor armor and resistance +15/30%; Frost Ward reflection 10/20% | Separate armor buff contributions, Frost resistance and school-aware ward reflection |

## Prerequisite spell evidence

- Clearcasting 12536: 15 seconds, one charge. Its raw cost modifier is −1000
  with the talent tooltip's division by ten, meaning a 100% cost reduction.
- Counterspell 2139: 30-yard range, 30-second category recovery, interrupt effect
  and 10-second school lockout. Trigger 18469 is a separate 4-second silence.
- Fire Ward 543 / Frost Ward 6143 rank one: 165 school-specific absorption,
  30-second duration and shared category 56 with 30-second recovery. These are
  distinct from Mana Shield.
- Frost Armor 168 rank one: 30 armor, 30-minute duration and contact trigger
  6136. Ice Armor 7302 rank one: 290 armor, 6 Frost resistance, 30-minute
  duration and contact trigger 7321. Their contact triggers still need inspection.
- Amplify Magic 1008 rank one: +15 incoming spell damage and +30 incoming
  healing for 10 minutes. Dampen Magic 604 rank one: −10 incoming spell damage
  and −20 incoming healing for 10 minutes. Both target a party member.

These installed ranks do not establish Survivors acquisition, duration or scaling.

## Current code and implementation order

The Mage pool in `crates/survivors-core/src/data.rs` contains thirteen attacks
and defensive abilities, including Mana Shield, but no wards, armor spells or
Counterspell. The existing **Arcane Intellect** passive grants haste, and
**Amplify Magic** grants area. Their names are not evidence that the installed
primary stat or recipient buff exists. Do not multiply those substitutes to mark
Arcane Resilience or Magic Attunement complete.

Incoming contact in `crates/benilla-app/src/survivors/defense.rs` carries attacker
and raw damage, without spell-school metadata. Its contact reflection is not
ward reflection. Outgoing `Hit` in `abilities.rs` carries hero/pet ownership,
source, critical result and occurrence time, but no unique cast identifier.
Settlement already distinguishes original hits and verifies effective health
removal; a new proc must use those facts rather than an attempted cast or raw
damage number. Multistrike echoes, splash and periodic events need an explicit
eligibility policy before they can award additional charges or refunds.

1. Resolve the proposed Clearcasting conversion, then implement its state,
   successful-hit trigger and successful-cast consumption together. Preserve
   the one-use, 15-second window; define refresh and multi-target behavior.
2. Establish incoming spell identity and enemy casting before Counterspell or
   wards. Keep school lockout distinct from all-school silence and prohibit
   reflected damage from recursively reflecting.
3. Add actual armor/buff and wand consumers before their dependent talents.
   Resolve the existing same-name passive semantics explicitly.
4. Resolve remaining mana/Intellect/threat conversions before claiming those
   nodes have named mechanics. Illumination's approved half-heal-recovery rule
   does not approve every other resource conversion.

## Acceptance

Each implementation needs legal purchase/save/reload/spec/refund checks and
actual cast or incoming-event outcomes for every rank. Proc checks must cover
ownership, school, effective damage, invalid/dead targets, pause/expiry, run
reset, failed casts, multi-target hits and echoes. Only a successful eligible
cast may consume a next-cast charge. Tooltip growth stays green and inline,
with units once; native wide/compact screenshots belong inline in CHANGELOG.

This checkpoint only reads installed rows and source. No runtime behavior,
coverage increase, combat acceptance or new rendered screenshot is claimed.

## Counterspell follow-up — 2026-10-30

[Counterspell and node 88](COUNTERSPELL.md) now have acquisition, enemy-cast
interruption and independent silence consumers, with runtime and native UI
evidence. This supersedes their missing-consumer notes above. Fresh coverage is
39/49 Mage and 312/432 overall; the other ten Mage definitions remain pending.
