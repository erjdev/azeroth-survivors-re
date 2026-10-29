# Remaining Paladin mechanics

At `fdabe73`, 26/44 Paladin nodes have named definitions; eighteen retain the
explicit provisional bonus. This audit reads all eighteen nodes and their ranks
from the checkout's 1.12.1 data. It changes neither coverage nor game rules.

Reproduce with `target/debug/survivors-audit wow-data/Data --talent-gaps`,
`--talent-evidence <node>` and `--spell-evidence <spell>`. Read referenced spells,
not just tooltip templates: Vindication points to 67/26017/26018, Seal of Command
to 20424/20467, Reckoning to 20178 and Illumination to dummy trigger 18350.
DBC proc fields alone do not establish server-side proc policy.

| Nodes | Installed behavior | Required consumer or policy |
| --- | --- | --- |
| 1432 Spiritual Focus | 14/28/42/56/70% pushback protection for Flash of Light and Holy Light | Those healing spells and interruptible casts; recovery reduction is not protection against losing cast time |
| 1628 Unyielding Faith | +5/10 percentage points resistance to fear and disorient | Incoming hero-control applications with distinct resistance outcomes |
| 1461 Illumination | Critical Flash of Light, Holy Light or Holy Shock healing can refund base mana cost; 20–100% proc chance | Effective Paladin healing-critical events and an explicit no-mana conversion; damaging Holy Shock criticals are not eligible healing |
| 1446 Improved Blessing of Wisdom | +10/20% to Blessing of Wisdom's effect | An authored no-mana Wisdom effect and blessing ownership; global regeneration does not establish this spell |
| 1465 Lasting Judgement | +10/20/30 seconds to Judgement of Light and Wisdom | Those persistent target debuffs and their healing/resource proc consumers; current damaging Judgement has no such duration |
| 1442 Blessing of Kings | +10% total primary attributes, five-minute blessing | Primary-attribute accounting or an explicit conversion, plus blessing activation/ownership; multiplying all derived stats also incorrectly affects unrelated ratings |
| 1501 Improved Righteous Fury | +16/33/50% to Righteous Fury's threat effect | Holy-damage threat ownership and enemy attention; contact pursuit alone is not threat |
| 1424 Shield Specialization | +10/20/30% shield block amount; requires five ranks of Redoubt | Equipped shield eligibility, a block outcome and block value; temporary absorb shields are a different mechanic |
| 1626 Improved Concentration Aura | +5/10/15 points of aura effect and additional silence/interrupt resistance | Concentration Aura, interruptible casts and incoming control resistance |
| 1426 Reckoning | Critical strikes received can grant an extra attack | Incoming critical outcomes and a real physical weapon attack; never recursively generate further extra attacks |
| 1429 One-Handed / 1410 Two-Handed Weapon Specialization | +2% physical weapon damage per rank; five/three ranks | Selected worn weapon identity and actual Paladin melee attack; see [weapon dependency](PALADIN-WEAPON-TALENTS.md) |
| 1430 Holy Shield | +30 points block chance for 10 seconds; four charges; 65 Holy damage per blocked attack at rank one | Shield eligibility, block events, charged state and attacker-owned retaliation; block charges must not be spent on dodges, parries or ordinary absorption |
| 1464 Improved Seal of the Crusader | +5/10/15% seal attack-power and judged Holy-vulnerability effects | Seal/ judgement state, physical melee contribution and scoped target Holy vulnerability; unrelated Holy attacks cannot stand in for both effects |
| 1633 Vindication | Damaging melee attacks can reduce target Strength and Agility by 5/10/15% for 10 seconds | Paladin melee events and target attributes or an explicit conversion; reducing outgoing damage alone is not both attributes |
| 1481 Seal of Command | Weapon-linked Holy proc (70% weapon damage); separate judgement with a stunned/incapacitated condition | Physical weapon damage source, seal exclusivity, proc policy and judgement consumption; Repentance supplies only the incapacitated target state |
| 1632 Eye for an Eye | Reflect 15/30% of incoming spell-critical damage, capped at 50% maximum health | Source-owned incoming spell criticals and nonrecursive retaliation; contact damage and outgoing player criticals do not qualify |
| 1405 Improved Retribution Aura | +25/50% Retribution Aura damage | A distinct acquired aura and its contact-retaliation source; global Thorns scaling would also alter other classes |

## Current runtime boundaries

`survivors/defense.rs` distinguishes invalid, inactive, immune, dodge, parry and
landed contact outcomes. It has no block or incoming critical outcome. The
`enemies.rs` contact consumer retains the attacker, but `Strike` has no incoming
spell/school/critical metadata. Its existing thorns response is shared with other
abilities and uses the source name `Thorns`.

`Stats` in `survivors/mod.rs` contains derived damage, health, armor and ratings,
not Strength/Agility/Intellect/Spirit. The configured Paladin pool contains Holy
attacks and defensive/control spells, not Holy Light, Flash of Light, a physical
basic swing, or Retribution Aura. The general instant-healing path can support new
heals but does not establish casting pushback. Existing milestone conversions
remain distinct from these unfinished individual nodes.

## Integration order and acceptance

1. Retribution Aura can reuse the existing attacker-owned contact boundary. Define
   acquisition, active-aura ownership and exclusive/shared behavior first; give its
   retaliation an explicit spell source so Improved Retribution Aura cannot boost
   Warrior Retaliation or Druid Thorns. Keep dodge/parry, immunity and absorbed-hit
   policy explicit. Verify actual emitted hits and settlement, not a stat alone.
2. Add Paladin healing and its critical-event consumers as a separate feature.
   Keep no-mana conversions explicit, scoped to successful eligible healing and
   protected from overheal/proc loops. Do not present these as cast-pushback support.
3. Resolve the existing physical-melee acquisition/damage decision before seals,
   Vindication, Reckoning and weapon specializations. Equipped appearance never
   supplies weapon power. Proc attacks need provenance and recursion guards.
4. Add incoming block, spell-critical and control outcomes with real producers,
   then their corresponding talents. Holy Shield needs actual shield equipment;
   flat absorption and avoidance cannot satisfy its requirements.
5. Develop primary attributes, blessing ownership and threat as coherent systems,
   or document deliberate Survivors conversions before enabling affected nodes.

Every group requires rank/prerequisite gates, saved-spec purchase/refund/restart,
actual event consumers, pause/death/expiry behavior and scope isolation. New UI
needs green inline changes and native wide/compact screenshots. Windows and
attended-combat acceptance remain separate. Do not mark an individual node done
because a similarly named milestone or generic provisional bonus exists.

Validation: the eight-package formatting/Clippy/test gate passes with 31
optional-addon skips. The first sandboxed run failed only at the existing
mock-login listener; the verified rerun had loopback permission. Rustdoc link
validation passes with 150 benign warnings. No new screenshot is claimed because
this checkpoint changes documentation only.

Retaliation foundation follow-up: contact reflection now emits the actual ability
key and cast-scaled damage once. A queued hit retains that identity if the active
buff changes. Retribution Aura still requires its own acquisition/active-state
policy and definition; this source fix does not enable node 1405.


Retribution Aura follow-up: spell 7294 is now an ordinary acquired Paladin ability,
using one slot and persisting for the run. Rank-one damage is 5 Holy (installed
base points 4 + die sides 1); ability ranks use the existing +30% base-damage
progression. Node 1405 adds 25/50% only to this spell. Damage is evaluated at each
contact using current Might, legendary and scoped Holy modifiers. It cannot crit,
spread to extra targets or gain a radius/cooldown. It does not trigger cast-only
milestones. Avoided/invalid/inactive contact emits nothing; absorbed and immune
contacts respond, consistently with existing temporary reflection. Only the hero
is protected, not pets or party members. This authored self-aura conversion
coexists with passive Sanctity Aura and temporary Thorns; party aura exclusivity
is not represented. Native macOS pointer purchase/refund/restart and independent-spec acceptance
now pass, with a catalog-backed save regression for both ranks. Physical-controller
and attended Windows combat acceptance remain open.


Holy Light follow-up: spell 635 now has an acquired automatic direct-self-heal
conversion (40 health / 12s recovery below 85%). Holy Power includes its healing
criticals. This creates a healing-event producer, but does not enable Spiritual
Focus or Illumination: it has no interrupted cast and no mana to refund. Flash of
Light and a revised Healing Light consumer remain unfinished. Existing Divine
Favor and Holy Shock cast-heal behavior are preserved pending that work.


Flash of Light / Healing Light follow-up: both automatic direct heals now exist.
Node 1444 applies 4/8/12% healing to them, with the previous Holy Shock cast-heal
conversion preserved independently. Holy Power includes both healing-critical
rolls. Illumination still needs an explicit no-mana policy and an effective-heal
critical consumer; mere cast or damage-critical events cannot satisfy it.
