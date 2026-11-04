# Hunter's Mark dependencies

Read from this checkout's explicit `wow-data/Data` patch chain on 2026-10-28.
This is implementation evidence, not a completed ability or talent.

```sh
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 1130
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1343
```

The base spell describes a target-specific ranged attack power bonus, visibility
through stealth/invisibility, and minimap tracking. All four installed ranks have
a 120-second duration, 100-yard maximum range, no individual/category cooldown,
and a 1500-ms start recovery. These are vanilla values, not an accepted Survivors
automatic casting cadence or eight-rank progression curve.

| Spell | Installed rank | Ranged attack power bonus |
| --- | ---: | ---: |
| 1130 | 1 | 20 |
| 14323 | 2 | 45 |
| 14324 | 3 | 75 |
| 14325 | 4 | 110 |

The bonuses are effect 1's base points plus its one die side. Effect 0 has aura
68 and implicit target 25; effect 1 has aura 127 and implicit target 6. Do not
treat either raw target code or the absence of a maximum-target count as proof
that arbitrary simultaneous marks are allowed.

Improved Hunter's Mark (node 1343, spells 19421–19425) increases that bonus by
3/6/9/12/15%. It does not increase all ranged damage by those percentages. Rank-1
Mark with five talent ranks therefore supplies 23 ranged attack power, not a
15% damage multiplier.

## Attack classification

The inspector shows `ranged_attack=true` for Auto Shot (75), Multi-Shot (2643),
Aimed Shot (19434), and Arcane Shot (3044), but their effects differ:

| Spell | Effect type | School |
| --- | ---: | ---: |
| Auto Shot | 58 | 0 |
| Multi-Shot | 121 | 0 |
| Aimed Shot | 121 | 0 |
| Arcane Shot | 2 | 6 |

The ranged flag alone does not establish an attack-power coefficient. The DBC
rows do not supply the server damage formula. Survivors currently has authored
per-ability damage and a percentage-damage conversion for Trueshot Aura; it has
no ranged attack power stat or weapon-speed contribution to reuse.

Additional implementation evidence, inspected 2026-10-28:
[CMaNGOS's weapon-effect handler](https://github.com/cmangos/mangos-classic/blob/master/src/game/Spells/SpellEffects.cpp)
distinguishes normalized weapon effects, and
[its AP multiplier](https://github.com/cmangos/mangos-classic/blob/master/src/game/Entities/Unit.cpp)
uses a 2.8-second factor for normalized ranged weapons, versus attack time for
non-normalized attacks. This is an open-source server reference, not a Blizzard
formula specification; no source code is copied into this repository. It does not
justify adding the same bonus to Arcane Shot, stings, traps, Volley or pets.

Current engine integration: `survivors/minimap.rs::compose` includes living,
perceptible enemies inside the map radius. A mark still needs a distinctive icon
in addition to its reveal predicate. Enemy concealment uses a separate `Concealed`
component; authored encounters do not apply it yet. The `Marks` component controls eight visual aura
slots; it does not own a caster-specific gameplay debuff. Keep visual expiry and
gameplay mark expiry synchronized rather than deriving damage from aura slots.

## Implemented damage foundation

`marked_target::HunterMark` now owns the caster, snapshotted power, start and
expiry. `marking::prune` removes expired/dead/orphaned marks. Aimed Shot and
Multi-Shot snapshot a 0.2 damage-per-power coefficient with their cast modifiers;
settlement reads each victim's live mark at impact, including independent splash.
This explicit Survivors contribution does not add a general ranged-power gear
stat or Auto Shot. Tests cover replacement, pause-time expiry, delayed hits,
critical scaling, splash and caster/target loss. No acquisition or reveal UI is
exposed, and this foundation does not complete Improved Hunter's Mark.

The concealment integration now distinguishes gameplay concealment from camera
occlusion (515 app tests, Clippy, native build and scoped dependent-package
validation pass; rendered acceptance remains open):

- `abilities::cast_abilities` retains all living victims for area damage and
  filters direct targets through the caster's reveal predicate. Ground placement
  selects visible targets; an existing area still hits concealed victims.
- `hunter_pet::fight` filters both its primary target and additional bites through
  the owner's reveal predicate. Ranged totems use the same predicate.
- `minimap::compose` rebuilds its visible enemy-offset input before comparing
  its cache, so concealment/reveal changes invalidate stationary dots. A distinctive
  mark icon still needs its own cache state and visual validation.
- `perception::sync` hides enemy roots before visibility propagation, restoring
  their previous visibility after reveal or concealment removal. Camera cutaways
  are independent. Rendered acceptance and future silhouette integration remain open.

Verify these consumers with ordinary, concealed, marked, expired and orphaned
victims before enabling the acquired ability. The automatic cast cadence,
replacement policy and eight-rank power curve still need explicit Survivors rules.

## Required implementation and acceptance

- Establish an explicit ranged attack power contribution and per-attack scope.
  Keep ability rank, talent multiplier, own-amulet scaling, critical damage and
  source/target snapshots distinct. Do not infer eligibility from the ranged flag.
- Add target-owned mark state with caster identity, replacement/refresh policy,
  duration, death cleanup, and pause behavior. Settle marked versus unmarked
  targets independently for multiple shots, splash and delayed impacts.
- Connect mark display and minimap tracking. Stealth/invisibility reveal needs a
  shared enemy concealment state; camera occlusion silhouettes do not establish it.
- Add the acquired offer and its actual green inline values, then the talent's
  legal purchase/refund, saved-spec and run-snapshot behavior. Training alone must
  not grant the mark. Keep the existing generic bonus explicitly provisional
  until the named effect is implemented.
- Verify expiry, replacement, caster/target loss, target scope, each talent rank,
  save/reload, and wide/compact UI. Attended combat remains a separate check.

These requirements extend the [Hunter dependency audit](HUNTER-TALENT-DEPENDENCIES.md).
Neither a damage-only mark nor a tooltip-only tracking promise closes the node.


## Acquired integration — 2026-10-28

The local implementation now uses the explicit automatic-casting and eight-rank
rules in [the game specification](../SURVIVORS.md). Ability acquisition, training,
world aura synchronization and a distinct gold minimap diamond are wired in.
273 core and 521 app tests pass, with feature-enabled Clippy and the native build.
Wide/compact cards and talent screens were captured; native purchases persisted
Efficiency 5/5 and Improved Hunter's Mark 3/5 in an isolated profile. Scoped
integration passes for all six affected/dependent packages, with 31 missing-addon
skips. Full pre-merge gates remain required. Earlier sections record the dependency investigation;
acquisition and training are now exposed. Rendered world aura/minimap and attended
combat acceptance remain open.
