# Remaining Hunter mechanics — 2026-10-28

At `a3dcd9e`, Hunter has 39 of 46 named talent definitions. The remaining seven
still award the explicitly provisional generic bonus. This audit does not count
them as implemented or approve replacements for their named effects.

Implementation follow-up: node 1381 now has an acquired Monkey aspect and real
contact dodge outcomes, with tests for training and saved specs. Native wide/compact previews and rank-5 purchase/save are verified;
full native integration passed in 15m17s (31 missing-addon skips per data/player run). Node 1343 now also has acquired mark power, caster-owned target selection, ranged
impact integration, concealment reveal and tracking state. Core/app regressions
and native talent purchase/save previews pass; rendered world/minimap acceptance
remains open. The following reactive checkpoint brings this to four pending definitions (42/46 defined). The table
below preserves the original installed rules and dependency assessment.

## Parry and Counterattack — 2026-10-29

Installed node 1308 / spell 19263 grants **25 percentage points each** of dodge
and parry for 10 seconds. Node 1311 / spells 19295, 19297, 19298, 19301 and 19300
grants 1/2/3/4/5 percentage points parry. These now replace the earlier generic
50% Deterrence and 1%-per-rank Deflection damage-reduction conversions.

The Hunter contact table uses one roll: Monkey plus Deterrence dodge first,
then Deflection plus Deterrence parry, then a landed hit. Outcomes are exclusive;
parries consume no shield, health, reflection or on-damage recovery. Invalid,
immune and dead contacts do not roll. Deterrence retains automatic 10-second
windows at run start and every 180 seconds of active gameplay. Deflection supplies
parry outside those windows. Neither protects a pet or non-contact damage.

Counterattack node 1312 requires Deterrence. The Survivors automatic conversion
immediately strikes the actual parried attacker for 40 base damage and roots it
for 5 seconds (existing boss-control rule: 1.5 seconds). Damage bonuses and the
shared multistrike settlement apply; this automatic response does not roll a
critical hit. It uses spell 19306 visuals and has the installed 5-second category
recovery. No charge or later activation window is invented: a parry during recovery
is still avoided but cannot queue a future response. Pauses leave the run clock
unchanged. Death prevents queued Counterattack settlement; new runs reset recovery.
It uses the direct hit path, without a block/dodge/parry roll for the victim.

This completes the named reactive dependency in code, not attended combat or
Windows visual acceptance. All 292 core and 570 runtime tests pass with checkout data; Clippy including tests
and eight-package scoped checks pass (31 optional-addon skips). The first sandboxed
scoped run could not bind its existing loopback mock login server; the permitted
rerun passed. Native purchase/refund, prerequisite guard, spec isolation, save/relaunch and
wide/compact descriptions pass. Four screenshots are embedded in the changelog.
The incremental native macOS build took 1m11s. The remaining four Hunter nodes require pet vision,
incoming hit/impairment resistance, Feign Death and primary attributes.

## Reproduce the evidence

Run against the checkout's explicit data directory:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1312
```

Replace `1312` with a node ID below. The command reads Talent, TalentTab, Spell
and SpellDuration from that patch chain. It reports every rank, prerequisite,
description template, raw effect fields, duration and current Survivors definition.
No application installation discovery or data modification is involved.
Raw base points are not final magnitudes: these ranks use base points plus one
where their die-side field is one. Millisecond duration modifiers require conversion
to seconds. The command deliberately leaves those fields raw rather than claiming
a general spell interpreter.

## Required behavior and missing dependencies

| Node | Installed rank rules | Dependency before the named effect can be complete |
| --- | --- | --- |
| 1624 Improved Eyes of the Beast | Spells 19557–19558; +30/60 seconds to Eyes of the Beast | An actual pet-vision state, its activation, duration and return to the hero camera. The existing wolf's follow/attack behavior is not pet vision. |
| 1381 Improved Aspect of the Monkey | 19549–19551, 24386–24387; +1/2/3/4/5 percentage points to that aspect's dodge bonus | An acquired aspect, mutually exclusive aspect state, and a dodge outcome. An unconditional reduction would discard the aspect requirement. |
| 1343 Improved Hunter's Mark | 19421–19425; +3/6/9/12/15% to the mark's ranged attack power bonus | A per-target Hunter's Mark and a defined ranged attack power contribution. This scales the mark's bonus, not all damage by that percentage. |
| 1310 Surefooted | 19290, 19294, 24283; +1/2/3 percentage points hit chance and +5/10/15 percentage points movement-impairment resistance | Attack hit outcomes and incoming movement-impairment applications, including distinct root and snare resistance. Enemy slow/root timers are not hero resistance. |
| 1309 Improved Feign Death | 19286–19287; reduces Feign Death resistance by 2/4 percentage points | A Feign Death state and enemy-specific resistance/attention handling. Pet taunt alone does not implement feigning. |
| 1312 Counterattack | 19306; 40 damage and a 5-second root after a parry; requires Deterrence rank 1 | A parry event with attacker identity, a reactive activation window, and an attack that cannot be blocked/dodged/parried. Periodic automatic damage would lose the reactive behavior. |
| 1303 Lightning Reflexes | 19168, 19180–19181, 24296–24297; +3/6/9/12/15% Agility | An Agility value and its class-dependent derived stats, composed with gear and saved-spec snapshots. Existing damage/crit fields alone do not establish an Agility stat. |

The audit establishes rank values and prerequisites, not unspecified proc windows,
attack tables, level scaling or balance. Those need separate primary evidence and
explicit Survivors rules before implementation.

The same install's base Aspect of the Monkey is spell 13163: aura 49, base points
7, one die with one side, permanent duration row 21 (8 percentage points dodge).
Spell 13164 is a teaching spell, not the aura. Counterattack 19306 requires caster
aura state 7 and has a 5000 ms category recovery even though its individual recovery
field is zero. Its 5-second root duration does not establish the reactive window.
These fields were checked directly in Spell.dbc; future implementation must preserve
the distinction between teaching rows, effects, category cooldown and eligibility.

The [Hunter's Mark follow-up](HUNTERS-MARK.md) records the base ranks, tracking
requirements and why the ranged flag alone cannot select damage coefficients.

## Implementation order and acceptance

Start with defensive attack outcomes and aspect state, then Improved Aspect of the
Monkey and Counterattack. Both must consume the actual outcome, with source identity
retained; do not infer a parry from zero health loss (immunity, armor and shields
can also prevent damage). The earlier Deterrence conversion did not produce parries. The reactive checkpoint
above supersedes that conversion with installed avoidance values and verified contact outcomes.

Follow with marked-target ranged damage, incoming impairment resistance, Feign
Death, pet vision and primary attributes. Shared systems should also support the
other classes' pending talents rather than become Hunter-only substitutes.

For each completed node, verify every rank and refund, legal prerequisites,
save/reload and spec isolation, run snapshots, pause timing, source/target scope,
death cleanup, and interaction with existing protection. Capture native green
inline upgrade text at wide and compact sizes and embed it in the changelog.
Attended Windows combat and balance acceptance remain separate from unit tests.

The original incoming-hit follow-up introduced `survivors/defense.rs::contact`: its result
retains the attacker and distinguishes landed, immune, inactive and invalid hits.
Landed outcomes report absorbed damage and actual health removed separately.
This removed the ambiguous scalar boundary. The aspect and reactive checkpoints
subsequently added real dodge/parry outcomes and immediate Counterattack responses. `survivors/mod.rs::Stats` still has
derived combat stats but no primary attributes; `survivors/hunter_pet.rs` supplies
the combat wolf and taunt. This is a dependency assessment of Survivors, not a claim
that upstream's networked WoW client lacks these concepts.
