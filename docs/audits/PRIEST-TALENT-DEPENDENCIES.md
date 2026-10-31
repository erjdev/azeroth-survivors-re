# Remaining Priest mechanics

At `4f8a658`, 31 of 47 Priest nodes have named definitions; sixteen retain the
explicit provisional bonus. The following dependency audit uses this checkout's
1.12.1 data and current Survivors consumers. It does not authorize replacing the
named mechanics with unrelated global bonuses or count these nodes as complete.

Reproduce with `cargo run -p survivors-audit -- wow-data/Data --talent-gaps` and
`--talent-evidence <node>`. Tooltip references such as `$27827d` require inspecting
the referenced spell with `--spell-evidence`; they are not literal durations.

| Nodes | Installed behavior | Missing prerequisite |
| --- | --- | --- |
| 342 Unbreakable Will | Resist stun, fear and silence | Incoming hero control applications and separate resistance outcomes; existing enemy control timers are insufficient |
| 345 Wand Specialization | Increase wand damage | An actual wand attack with equipped-weapon identity; a wand appearance or Smite is not that attack |
| 352 Silent Resolve; 466 Shadow Affinity | Reduce spell threat / Shadow spell threat | Source-owned enemy attention and a threat policy; ordinary pursuit is not spell threat |
| 321 Martyrdom; 410 Healing Focus | Critical-hit-triggered Focused Casting / healing pushback protection | Incoming critical outcomes and interruptible casts; automatic instant recovery is not lost casting time |
| 347 Meditation; 1201 Mental Strength | Casting-time mana regeneration / maximum mana | Explicit Survivors conversion consistent with the no-mana design; do not add a resource bar just to make the talents applicable |
| 350 Improved Mana Burn | Shorter Mana Burn cast time | Mana Burn, its target-resource policy and an explicit automatic-cast conversion |
| 351 Divine Spirit; 402 Spiritual Guidance | Spirit buff / damage and healing derived from total Spirit | Primary Spirit accounting, buff ownership and derived damage/healing; existing health regeneration is not Spirit |
| 413 Improved Prayer of Healing | Reduce Prayer of Healing mana cost | Prayer of Healing and eligible friendly recipients, then an explicit recovery conversion; Greater Heal is a different spell |
| 1561 Spirit of Redemption | Restricted healing-only state after lethal damage, followed by death | A lethal-hit state transition, recipient selection, movement/attack/targeting restrictions and exactly-once final death; normal regeneration must still never revive a dead hero |
| 1637 Lightwell | Placed, charged healing object; damage cancels its heal | Object lifetime, recipient interaction/acquisition policy, charge ownership and interruptible source-owned healing; the Holy-30 Nova pulse is not a Lightwell |
| 541 Silence | Prevent target spellcasting | Enemy cast state and interruption/silence rules; stopping contact movement or attacks is not silence |
| 521 Shadowform | Shadow damage bonus, physical mitigation, no Holy casting | Persistent form state, school-aware incoming damage and cast eligibility; the Shadow-30 Scream shield is not a form |

## Integration order

Choose and document acquisition/activation rules before introducing new abilities
or forms. Implement shared state and actual consumers together; keep nodes pending
until their named effects reach those consumers. The current `Run` stats contain
derived combat values, not Spirit or mana. `hero_vitals` immediately enters the
death path at zero health, and regeneration explicitly excludes dead heroes;
Redemption must preserve those invariants outside its deliberate state transition.

Prioritize shared enemy casting/control and attack identity, then scoped talents;
develop primary attributes and healing objects as separate coherent systems.
Keep no-mana conversions explicit and distinguish individual nodes from the
existing authored 10/20/30-point milestones. Do not infer implementation from a
matching milestone name.

Each completed group needs rank/prerequisite and saved-spec checks, actual event
consumers, pause/death/expiry handling and scope isolation. UI changes need native
wide/compact screenshots with green inline deltas. Attended combat and Windows
acceptance remain separate. This audit changes no coverage count or game rule.

## Silence follow-up — 2026-10-30

[Node 541 and its acquired spell](PRIEST-SILENCE.md) now have actual enemy-cast
cancellation and silence consumers, with runtime and native UI evidence. This
supersedes the missing-consumer note for Silence above. Priest coverage is now
32/47 and overall coverage is 313/432; the other fifteen Priest nodes remain
provisional. Attended combat and Windows acceptance are still required.
