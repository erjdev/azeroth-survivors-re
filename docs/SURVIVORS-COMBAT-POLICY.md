# Approved combat policy — 2026-11-01

User decisions, mapped by mechanic rather than reply numbering. This contract
supersedes conflicting proposals and historical dependency-audit requirements.
These are implementation requirements, not a claim that runtime changes are done.

| Mechanic | Approved behavior |
| --- | --- |
| Warlock demons | Pre-run choice through talents or a Warlock menu. Imp is the default for Affliction/Destruction; Demonology talents can add other demons. Pets spawn as additional gameplay entities. Do not require a summon ability card. Whether a talent replaces or adds a demon must be explicit in its design. Defeat/recovery and sacrifice details were not approved by this answer. |
| Mage Clearcasting | A proc fires an additional Arcane Missiles ability, with its own haste-reduced internal cooldown. Replace the proposed recovery-refund conversion. Exact proc/cooldown tuning remains implementation work; prevent recursive proc chains. |
| Ancestral Healing | Replace the existing conversion with 8/16/25% increased armor for 15 seconds after an effective critical heal; refresh rather than stack. |
| Melee damage | Fixed, normalized attack damage independent of weapon damage ranges. Equipment upgrades scale percentage stats; do not introduce item weapon damage/speed as the damage basis. Automatic attacks remain outside ability slots. |
| Disarm | Remove the mechanic and enemy-weapon dependency. Redesign associated talents/abilities using stuns, fears, paralysis, slows, damage-over-time or suitable debuffs. |
| Independent attacks | Never require standing still. Abilities run on independent timers and continue while other attacks/channels run. Procs have individual internal cooldowns, normally affected by haste. Apply to Tranquility and every other channel/cast. |
| Starting kit | At most three slotted starting abilities, six maximum during a run. Major talent milestones grant or replace abilities. Redesign any build that exceeds three; do not add an overflow selection menu as the solution. |
| Attribute/resource conversions | Intellect becomes raw damage percentage. Threat becomes recovery reduction or damage percentage. Mana, Energy, Rage and Spirit become recovery reduction. Document per-talent tuning and avoid new resource pools. |
| Feral/Rogue mechanics | No combo points, stealth or positional attack requirements. Redesign abilities that depend on them; do not implement their original prerequisite mechanics. Ability-specific range remains relevant, especially for melee; area-of-effect stats increase range. |

## Integration and acceptance

1. Audit existing consumers and descriptions as well as provisional nodes. Existing
   definition counts do not prove compliance with this revised contract.
2. Correct starter grants/replacements and demon selection in the starter/talent UI;
   keep all possible builds within three starting slots. Preserve saved builds or
   explicitly migrate replaced talent effects.
3. Implement normalized melee and resource/attribute conversions. Replace disarm,
   combo/stealth/position dependencies with concrete effects and matching green
   inline previews. Audit existing Rogue recovery conversions rather than treating
   every old finisher name as a requirement to add combo points.
4. Implement Clearcasting's extra cast and independent moving channels, with
   source attribution, per-proc cooldowns, haste, and recursion protection. Check
   range scaling without accidentally converting single-target attacks into AoE.
5. Verify actual runtime effects, refunds/spec switching/restart and maximum-slot
   invariants; capture changed UI at wide/compact sizes and embed milestone images
   in changelog, README and PR. Keep attended gameplay/platform limits explicit.

Routine tuning and conversions can proceed within these rules. Ask only for a
material unresolved player-facing choice; no blanket approval of earlier demon
recovery/sacrifice proposals is inferred. UI/progression remains the priority;
Box3D stays after PR merge.
