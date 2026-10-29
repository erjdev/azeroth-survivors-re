# Scorch and Improved Scorch

## Installed evidence

Read only from this checkout's `wow-data/Data`:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 25
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 2948
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 22959
```

Mage Fire node 25 has three ranks (11095, 12872, 12873), no prerequisite
node, and needs 15 earlier-tree points. Its proc probabilities are 33%, 66%,
and 100%, not three linear increments of 33%. Spell 22959 applies 3% Fire
vulnerability for 30 seconds, capped at five stacks. Scorch 2948 is a
single-target Fire spell with a 30-yard range and no projectile travel speed.
The evidence command now reports stack limits for base spells as well as talents.

## Survivors behavior

Scorch is acquired independently from the Mage ability pool. Authored rank-one
balance is 16 damage, 1.5-second recovery and 30-yard range; ordinary rank,
gear, critical and Fire-talent scaling applies. Its definition and pool entry
are appended so older spell/Codex indices keep their meanings. Extra targets
select additional individual victims; this is not an area spell.

Improved Scorch rolls once after each original positive Scorch hit on a surviving
enemy. The triggering damage uses the old stack count. Refresh replaces the
30-second expiry and adds one stack up to five. Splash copies, multistrike echoes,
periodic ticks, other spells, zero damage and dead victims cannot add stacks.
Only actual installed Fire-school spells benefit, including Ignite and Fire
reflection; an absent spell catalog does not guess a school. Other classes' Fire
damage can benefit from the target debuff without triggering it.

Stacks live on the enemy and disappear on death/despawn. Active run time governs
expiry and pause behavior. Ordered direct hits use their occurrence timestamps;
periodic catch-up uses the existing settlement-time modifiers. Historical
sub-frame periodic modifiers remain a shared damage-timeline limitation.

## Acceptance

Core tests cover exact rank probabilities, scope, stacking, refresh, cap, expiry,
invalid clocks and inline green value spans. Runtime tests exercise real hit
settlement, periodic Fire damage, missing-data behavior, target isolation,
duplicate-hit exclusions, ordered timestamps and lethal cleanup. All 289 core
and 557 Survivors runtime tests pass. Native macOS checks cover legal purchases,
refunds, independent specs, save/reload, and talent/card previews at 1280×720 and
640×720. Screenshots are embedded in the changelog.
Installed definition coverage is now 298/432 nodes (Mage 38/49); this is not
a claim of complete combat acceptance. Feature Clippy and eight-package scoped
checks pass (31 missing-addon skips).
Attended combat, Windows visuals and balance remain open.
