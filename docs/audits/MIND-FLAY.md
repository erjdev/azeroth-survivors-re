# Mind Flay implementation — 2026-10-28

Priest node 501 remains pending. The current Priest pool has no channelled attack;
an ordinary damage-over-time application does not establish a channel lifecycle.

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
Acquisition, talent, offer and visual acceptance remain open. The runtime currently
settles catch-up damage using the existing hit pipeline time; although the core
retains scheduled timestamps, historical timed-modifier settlement is not integrated.
All 532 Survivors app tests pass, including the runtime system with wall, range,
concealment and missing-caster interruptions. The final native build passed in 58s.
Feature-enabled Clippy and six-package scoped checks pass (31 missing-addon skips).
Coverage remains 296/432. This is not a completed Mind Flay implementation.

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
