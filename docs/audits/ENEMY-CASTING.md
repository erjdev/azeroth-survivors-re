# Enemy casting and interruption

## Shared lifecycle — 2026-10-30

`survivors-core::casting` now provides a per-caster lifecycle for six magic schools.
This is a prerequisite implementation, not a playable enemy spell or a completed
talent. Current named talent coverage remains 311/432.

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

Until those steps exist, the current enemy contact loop is unchanged. These core
rules alone do not satisfy Counterspell, Kick, Shield Bash, Silence, casting
pushback, spell-critical reflection or the remaining talent acceptance contract.
