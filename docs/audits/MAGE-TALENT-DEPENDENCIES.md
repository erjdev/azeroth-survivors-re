# Remaining Mage talent dependencies

**Policy update:** [Approved combat rules](../SURVIVORS-COMBAT-POLICY.md)
supersede conflicting design requirements below. Installed-data evidence remains
reference material; it does not require reproducing rejected vanilla mechanics.

Evidence: this checkout's `wow-data/Data`, read using `survivors-audit
--talent-evidence NODE` for the eleven nodes below and `--spell-evidence ID`
for their prerequisite spells. Raw installed rows establish reference mechanics,
not Survivors balance or implementation. The audit binary's coverage summary is
stale; current source coverage remains 38/49 Mage definitions, 308/432 overall.

| Node | Installed mechanic | Required consumer or decision |
| --- | --- | --- |
| 78 Wand Specialization | +13/25% wand damage | Worn-wand attack identity and damage; cosmetics and ordinary spells must not qualify |
| 75 Arcane Concentration | 2/4/6/8/10% on damaging spell hits; Clearcasting makes the next damaging spell free | Approved: proc an additional Arcane Missiles cast with an independent haste-reduced internal cooldown; no next-cast charge or recovery refund |
| 82 Magic Attunement | Amplify/Dampen Magic effectiveness +25/50% | Actual recipient-owned spell-damage/healing modifiers for both buffs |
| 85 Arcane Resilience | Armor equal to 50% of Intellect | Approved Intellect-to-damage-percentage conversion; author and document its rank values |
| 88 Improved Counterspell | 50/100% chance to silence for 4 seconds | Counterspell acquisition, enemy casting, interruption, school lockout and silence |
| 1142 Arcane Meditation | 5/10/15% mana regeneration continues while casting | Approved mana-to-recovery conversion; author and document its rank values |
| 77 Arcane Mind | Maximum mana +2/4/6/8/10%; requires Arcane Resilience | Approved mana-to-recovery conversion; installed text does not say Intellect |
| 23 Burning Soul | 35/70% resistance to Fire casting pushback; Fire threat −15/30% | Redesign pushback protection for attacks that continue while moving; convert threat to recovery reduction or damage percentage |
| 24 Improved Fire Ward | 10/20% chance to reflect Fire spells while Fire Ward is active | School-aware incoming spell events, ward ownership and nonrecursive reflection |
| 1639 Master of Elements | Fire/Frost criticals refund 10/20/30% of base mana cost | Recovery conversion with cast identity and an explicit multi-hit proc policy; no resource pool |
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

1. Implement the approved extra Arcane Missiles proc on eligible effective spell
   hits, with a separate haste-reduced internal cooldown and recursion protection.
   Document rank chance, cooldown and multi-hit eligibility. Do not carry over
   vanilla's one-use charge or 15-second consumption window into this conversion.
2. Establish incoming spell identity and enemy casting before Counterspell or
   wards. Keep school lockout distinct from all-school silence and prohibit
   reflected damage from recursively reflecting.
3. Add actual armor/buff and wand consumers before their dependent talents.
   Resolve the existing same-name passive semantics explicitly.
4. Apply the approved conversions: mana to recovery reduction, Intellect to
   damage percentage, threat to recovery reduction or damage percentage. Exact
   tuning is implementation work; Illumination's half-recovery amount is not a
   universal amount for these talents.

## Acceptance

Each implementation needs legal purchase/save/reload/spec/refund checks and
actual cast or incoming-event outcomes for every rank. Proc checks must cover
ownership, school, effective damage, invalid/dead targets, pause/expiry, run
reset, failed casts, multi-target hits and echoes. Clearcasting must not consume
an ability slot, recurse from its own missiles, or reset the ordinary Arcane
Missiles ability timer. Tooltip growth stays green and inline, with units once;
native wide/compact screenshots belong inline in CHANGELOG.

This checkpoint only reads installed rows and source. No runtime behavior,
coverage increase, combat acceptance or new rendered screenshot is claimed.

## Counterspell follow-up — 2026-10-30

[Counterspell and node 88](COUNTERSPELL.md) now have acquisition, enemy-cast
interruption and independent silence consumers, with runtime and native UI
evidence. This supersedes their missing-consumer notes above. Fresh coverage is
39/49 Mage and 312/432 overall; the other ten Mage definitions remain pending.


## Approved extra-cast implementation — 2026-11-02

[Clearcasting's implementation audit](MAGE-CLEARCASTING.md) records node 75's
extra-cast path, independent haste cooldown and provenance rules. Its acceptance
status supersedes the pending consumer above; no other Mage gap is closed here.


## Intellect, mana and threat conversions — 2026-11-02

[The resource-conversion audit](MAGE-RESOURCE-CONVERSIONS.md) tracks Arcane
Resilience, Arcane Meditation, Arcane Mind and Burning Soul's approved damage/
recovery replacements and acceptance. These effects do not require Intellect,
mana or threat consumers; the remaining ward, wand and buff dependencies are
separate work.


## Master of Elements follow-up — 2026-11-02

[Node 1639](MAGE-MASTER-OF-ELEMENTS.md) now has a remaining-recovery refund
consumer, scoped to original Fire/Frost criticals with a shared haste-scaled
internal cooldown. The explicit time-based multi-hit policy supersedes the
old cast-identity proposal above. Remaining Mage definitions: 78, 82, 24 and 70.
