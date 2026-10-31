# Enemy casting and interruption

## Shared lifecycle — 2026-10-30

`survivors-core::casting` now provides a per-caster lifecycle for six magic schools.
At `a73d9ae`, this was a prerequisite implementation without a playable enemy
spell. The follow-up below connects the first caster. Named talent coverage
remains 311/432.

The installed Counterspell evidence in the [Mage audit](MAGE-TALENT-DEPENDENCIES.md)
requires distinct interruption, school lockout and silence. The shared rules accept
caller-supplied timings and spell IDs; they do not assign enemy spells, damage,
resistance, cooldowns or acquisition rules.

- Starting a cast preserves its target, school, start and completion time. Another
  start cannot overwrite it. Instant spells complete on the first poll.
- Polling with active-run time emits one completion, retaining its scheduled time
  even after a long frame. Unchanged time naturally pauses a timed cast. Losing
  caster/target validity, range or visibility must be supplied by the caller and
  cancels that cast permanently. Explicit cancellation discards unpaid work.
- A successful interrupt cancels the cast and locks only that school. An idle or
  interruption-immune target grants no lockout. Zero-duration lockout is supported.
- Silence blocks every magic school even while idle, and cancels an unfinished
  cast independently of its interruption immunity. The caller must first resolve
  silence immunity/resistance. Shorter reapplications cannot shorten silence.
- At the exact completion deadline, an interrupt is too late. Silence still blocks
  subsequent casts but cannot erase an already completed pending spell. This is
  an explicit Survivors event-order policy, not a claim about server packet order.
- Nonfinite/backwards time and invalid durations cannot erase work, resurrect a
  canceled cast or replay a completion. A new run needs fresh state.

Nine focused regressions cover completion, school scope, silence/interrupt
independence, immunity, deadline ties, lost targets, stale operations and instant
casts. All 345 shared-rule tests pass. The six-package scoped formatting/Clippy/test gate also passes with local
mock-login socket access (31 optional-addon skips). Build storage passes at
63.83 GiB. There is no UI change or new screenshot.

## First roster candidate

The existing checksum-pinned creature template used by the
[equipment audit](ENEMY-EQUIPMENT.md) maps Darkeye Bonecaster (template 1522,
already authored in the roster) to `SpellListId=15220`. This is an explicit lookup
lead, not a spell identity. Read that list and its installed spell rows before
selecting the first cast; no spell is assigned by this checkpoint.

## Required runtime integration

1. Author an initial enemy spell roster with explicit installed spell evidence,
   cast/range/recovery values and encounter placement. Do not infer casting from
   an NPC name alone or enable arbitrary spells from unrelated creature records.
2. Own `Casting<Entity>` on each caster. Observe it with active-run time, validate
   caster and target life, range and line of sight on every update, and cancel on
   despawn/death/mode exit. Keep Stoneclaw's forced targeting and pet targets intact.
3. Settle each completion through a source-owned incoming spell event carrying
   school and critical outcome. Keep melee avoidance separate; do not route spells
   through contact-only retaliation. Then add school-aware resistance/wards.
4. Add acquired Counterspell and other interrupt abilities, distinguish successful
   interrupts from silence procs, and consume actual cast events. Preserve the
   installed school lockout and four-second Improved Counterspell silence.
5. Test actual ECS producers/consumers, target loss, pauses, boss immunities,
   death, proc recursion and restart. Add cast feedback and native screenshots;
   attended gameplay and Windows acceptance must follow before claiming completion.

At the initial `a73d9ae` checkpoint, the enemy contact loop was unchanged. Those core
rules alone do not satisfy Counterspell, Kick, Shield Bash, Silence, casting
pushback, spell-critical reflection or the remaining talent acceptance contract.


## Darkeye Bonecaster integration — 2026-10-30

The first producer now resolves creature template 1522 → spell list 15220 →
Frostbolt 13322 using the same pinned database revision as the equipment audit.
[Source spell list](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_spells.sql)
slot one specifies spell 13322, probability 100, target selector 1, cast flags 8,
zero initial delay and repeat delay 2–2. These raw fields establish the identity;
the following Survivors policies are explicit rather than a full server-AI port.

Reproduce the installed spell evidence with:

```sh
target/debug/survivors-audit wow-data/Data --spell-evidence 13322
```

The checkout reports Frost school (4), 3,000 ms cast time, 40-yard range, a
2,000 ms slow aura with base points −51 plus one, and a damage effect with raw
base points 47 / die sides 17. Survivors uses the cast time, range and 50% slow
for two seconds. Damage is authored as three times this enemy's existing scaled
contact damage, including its current percentage weakening. It does not import
vanilla mana costs or raw damage. No spell critical roll is added in this step.

The caster waits in range during casting/recovery and starts again two seconds
after completion or cancellation. It does not accumulate catch-up casts after a
long frame. School lockout/silence allows its existing melee fallback. Roots do
not interrupt casting; stun, fear, incapacitation, lost attention, death, range
or line-of-sight loss do. Recycled enemies discard their unfinished cast.

Existing attention ordering remains Stoneclaw, pet, nearby utility totem, hero.
Changing that selected target cancels the old cast rather than redirecting its
payload. Boss attention exclusions remain intact. The caster emits start, failure
and release animation events with the installed spell identity. Damage settles
at completion using the existing immediate-damage visual convention; missile
travel time is not yet a separate settlement clock.

Hero spell settlement retains attacker, spell, school and critical metadata and
bypasses melee dodge/parry, flat armor, Counterattack and contact retaliation.
General reduction, absorption and existing on-damage effects still apply. Frost
slow applies to a surviving landed hero hit even if fully absorbed; immunity
prevents it. Pet damage/slow stays on the pet; immobile totems only take damage.
The strongest incoming slow multiplies actual hero/pet movement, and expires in
active-run time. Fresh run/pet state clears it. Physical Curse of Weakness does
not reduce Frost damage; existing percentage weakening does.

Seven actual-runtime tests pass: cast/recovery timing and one-time settlement,
armor bypass without melee retaliation, eight cancellation cases, immunity/
absorption/pause, school-lockout melee fallback, Stoneclaw target switching,
pet-only delivery and missing-hero cancellation. Test build: 2m35s; execution: 0.06s.
The six-package scoped formatting/Clippy/test gate passes with 31 optional-addon
skips, and the documentation-link check passes. The native play build passes in
2m40s, reporting a linker warning that `__eh_frame` exceeds the compact-unwind
offset encoding limit (possible exception-handling performance impact), plus the
existing `block` future-compatibility warning. Build storage passes at 66.25 GiB.
This does not implement acquired player interrupts, incoming spell criticals,
resistance/ward policies or other enemy spells. Named talent coverage remains
311/432. Attended combat, cast animation readability and Windows verification
remain open; no unattended combat or new in-map screenshot is claimed.
