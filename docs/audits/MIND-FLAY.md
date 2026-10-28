# Mind Flay implementation — 2026-10-28

Priest node 501 now has a local acquisition unlock and channelled attack implementation.
Native talent purchase/save and wide/compact UI previews are verified. Rendered
combat acceptance remains open; this document is not a completion claim.

## Current acquired implementation

The talent unlocks Mind Flay in ability choices; it does not fill a slot until the
player acquires it. The authored Survivors base is 25 Shadow damage per tick,
three one-second ticks, 20-yard range and four-second recovery. Standard ability
ranks add 30% of base damage per rank; recovery cannot fall below the three-second
channel. Ticks do not crit. Extra single targets follow the existing rank and
+target rules. Movement and other automatic abilities are allowed; a channel
cannot overlap itself or replace lost victims mid-cast.

Shadow Focus, Shadow Reach and the existing Priest Shadow talent paths include
Mind Flay. The card shows inline green tick-damage growth. Independent beam-only
anchors use the installed channel kit without replaying its sound per victim;
the hero owns the pose and sound. Pause preserves anchors; run exit reaps them.
Installed `benilla-extract ... spellvis 15407` evidence: visual 277, channel kit
430, animation 124, sound 3012, persistent chain effect 4, ManaBeam texture.

## Installed evidence

Read this checkout's data with:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 501
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 15407
```

Node 501 is Shadow row 2, column 2, one rank, with no explicit prerequisite node.
Spell 15407 is Shadow school (ordinal 5), range 20 yards and duration 3000 ms.
Its periodic aura has a 1000 ms amplitude and base points 24 plus one die-side
point; the second aura slows movement by 50%. There is no individual recovery;
the start-recovery field is 1500 ms. These fields do not define Survivors rank
scaling, channel interruption policy or acquisition behavior.

## Implemented timing foundation

`survivors-core/src/channel.rs` fixes a deduplicated target list at cast time,
delays the first pulse by one interval, and emits each scheduled pulse once.
Catch-up pulses retain their scheduled timestamps. The caller validates targets
on every update; losing a target removes it permanently from that cast, without
retargeting. Caster death and interruption cancel all unpaid pulses. Repeated
active-time values produce no duplicate pulses; invalid/backwards time is ignored.

All 279 core tests and core all-target Clippy pass. The six-package scoped
checks passed for the timing foundation, with 31 missing-addon skips.
The runtime channel adapter now consumes this state before enemy movement,
checks caster/target life, range, concealment and solid-world obstruction, and
emits non-critical damage pulses. A separate channel slow composes with timed
snares without overwriting them. Run reset clears channels.
Acquisition, talent and visual code are wired. Channel pulses now retain their
occurrence times and merge chronologically with queued missiles and current-frame
hits. Historical timed-modifier settlement is not integrated. Initial selection checks range and concealment;
solid-world obstruction is now also checked before a cast starts a channel.
Blocked victims do not fill target slots: selection continues to farther clear
victims. If none qualify, the usual 200ms no-target retry applies without spending
the spell recovery or triggering successful-cast bonuses. Missing collision state
also prevents channel creation. Active channels still recheck obstruction each update.

All 540 Survivors app tests pass, including acquisition gating, obstruction before
selection, no immediate hit, no overlapping cast, inline upgrades and independent
beam lifecycle. The cast snapshot test covers Power Infusion expiry, Shadow Reach,
pause/resume and three exactly-once catch-up ticks. Paid-tick settlement tests cover Shadow Weaving, Blackout,
multistrike and Vampiric Embrace capped to removed health. Feature-enabled Clippy
passes; the targeting-fix native build took 1m13s. Six-package scoped checks pass
with 31 missing-addon skips; full pre-merge gates remain required. The isolated Priest
profile purchased node 501 through the UI and persisted `2:501:1`; native captures
cover 1100×720 and 640×720 talent/card layouts. Definition coverage is 297/432
(Priest 30/47). Screenshots are embedded in the changelog.

The independent anchors and channel fields are tested, but their rendered beams,
pose during other autocasts, sounds and attended Windows gameplay remain unverified.

## Remaining event-time settlement work

The core and adapter retain scheduled pulse times in `Hit` messages. The shared
queue now orders due missiles, channel pulses and ordinary hits, with stable
ties (previously queued before newly arriving hits). Future events remain queued;
invalid occurrence times are discarded. Splash and generated sword strikes
inherit the parent time. All 540 app tests pass, including a fine-frame/batched lethal-hit comparison;
the native build passed in 2m01s. Feature Clippy and six-package scoped checks pass
with 31 missing-addon skips. Full pre-merge gates remain required.

The shared consumer still expands proc/splash work before damage settlement and
handles periodic effects separately, while enemy debuff timers are advanced by
a separate system. Sorting direct hits does not reconstruct which vulnerability
was active at an earlier tick or complete ordering across all effect types.

Complete this as a shared damage-timeline change: preserve occurrence times for
all queued damage, order simultaneous/catch-up events deterministically, and
advance target modifier expiry/refresh to each event before settlement. Apply
new procs at the event time, then advance remaining timers to the current run
time exactly once. Keep caster damage snapshots separate from target modifiers.
Fixtures must compare fine-frame and long-frame results across expiry boundaries,
interleaved sources, death/overkill, refresh, and paused active time. Until then,
cast snapshots and exactly-once catch-up are verified independently of historical
target-modifier fidelity.

## Runtime and interface acceptance

1. Add an acquired channel kind and explicit Survivors rank/cooldown rules.
   Preserve single-target +target rules and independent selected victims. Do not
   turn the channel into several area locations or an instant damage burst.
2. Snapshot damage at cast time. Validate caster life, target identity, range and
   visibility while active. Do not inherit another cast's victims or survive run
   reset. Define movement and other automatic casts explicitly in the card prose.
3. Apply the slow immediately and only while connected. Remove only this channel's
   slow on cancellation; preserve unrelated snares, roots and other casters.
   Enemy movement currently reads a shared `Enemy.slow`/`slow_t`; overwriting
   those fields would discard another effect. Compose a separate channel slow
   with existing movement effects and remove it when the connection ends.
4. Set and clear the native channel fields, rather than repeatedly playing a
   normal cast. Existing engine code polls `UNIT_CHANNEL_SPELL` (144) and targets
   `UNIT_FIELD_CHANNEL_OBJECT` (20/21); its persistent beam/pose lifecycle already
   has regression tests in `creature_anim/spell_visual/tests.rs`. Confirm the
   actual installed Mind Flay visual before claiming rendered acceptance. One
   channel-object field can identify only one target: additional +target victims
   need their own beam representation without overwriting the primary target.
   Existing `ChainHops` forms a victim-to-victim polyline; it is not a substitute
   for independent caster-to-victim beams.
   The existing `Fx::cast` emits a normal Go event, not channel start/stop state.
5. Connect Shadow talents, periodic damage accounting and healing to paid ticks.
   Explicitly test Shadow Weaving, Blackout, Vampiric Embrace, Power Infusion,
   Shadow Reach, overkill, multistrike, target death and pause/resume. Retain pulse
   timestamps where timed damage modifiers require them.
6. Add legal talent purchase/refund and saved-spec tests, real offers, codex,
   green inline upgrade text and native wide/compact screenshots. Embed captures
   in the changelog. Attended Windows gameplay remains a separate acceptance step.
