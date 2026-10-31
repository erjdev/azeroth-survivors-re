# Warlock demon implementation contract — 2026-10-29

Warlock has 29/50 matching talent definitions. The following twelve nodes remain
provisional; this audit does not enable them or count them as implemented. The
starter/ability-slot decision is pending user input: choose a combat demon before
the run, or acquire a summon as one of the six abilities. Cosmetic pickup companions
remain independent in either case.

## Reproduce installed evidence

Use only this checkout's `wow-data/Data`:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-gaps
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1222
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 3110
```

Repeat `--talent-evidence` for each ID below. `--spell-evidence` now prints the
resolved raw cast-time row, signed power type, cost terms, equipment restrictions
and nonzero reagents as well as effects, ranges and cooldowns. These are inputs,
not a claim of final damage, resource cost, level scaling or Survivors behavior.
For example, `power_type=-2` on Life Tap (1454) means health, not mana; percentage
costs are percentages of base mana, not maximum mana.

## Talent boundaries

| Node | Installed rule | Required behavior before enabling |
| --- | --- | --- |
| 1222 Improved Imp | +10/20/30% effect for Firebolt, Fire Shield and Blood Pact | Ranged Imp casting, reflected Fire Shield damage and the living Imp's owner aura; all three effects must scale |
| 1225 Improved Voidwalker | +10/20/30% Torment, Consume Shadows, Sacrifice and Suffering effectiveness | Target attention/threat, pet healing and a sacrifice shield with correct ownership |
| 1243 Improved Succubus | +10/20/30% Lash of Pain/Soothing Kiss effect and Seduction/Lesser Invisibility duration | Distinct damage, threat, control and concealment paths; not a generic damage multiplier |
| 1242 Fel Intellect | +3/6/9/12/15% demon maximum mana | Pet-owned mana, costs and regeneration, or an explicitly documented Survivors resource conversion |
| 1241 Fel Stamina | +3/6/9/12/15% demon maximum health | Living demon health, damage, healing, defeat and recovery; no player-health bonus |
| 1262 Unholy Power | +4/8/12/16/20% Voidwalker/Succubus/Felhunter melee damage | Demon melee source ownership; excludes Imp Firebolt and player attacks |
| 1244 Master Demonologist | Living-demon-dependent effects on owner and demon | Imp threat reduction, Voidwalker physical reduction, Succubus damage, Felhunter resistances; inspect each triggered rank spell before implementing |
| 983 Improved Firebolt | Firebolt cast time −0.5/1 sec | Real cast start/completion/interruption; not an unrelated player cooldown bonus |
| 984 Improved Lash of Pain | Lash of Pain cooldown −3/6 sec | Separate Succubus special-attack recovery, not faster basic melee |
| 1227 Master Summoner | Summon cast time −2/4 sec and mana cost −20/40% | Summon lifecycle and resource policy; requires Fel Domination rank 1 |
| 1226 Fel Domination | Next summon cast time −5.5 sec and mana cost −50%; 15 sec window | One-use consumption and expiration in active run time, not a permanent pet modifier |
| 1281 Demonic Sacrifice | Sacrifices demon for species-specific owner effect; another summon cancels it | Distinguish sacrifice from defeat, prevent automatic recovery from undoing deliberate sacrifice; inspect triggered effects before enabling |

Positive percentage magnitudes above use raw base points plus the die-side term.
Negative millisecond modifiers also include that term: −501 + 1 = −500 ms.
Master Demonologist and Demonic Sacrifice embed references to other spells; their
outer row is insufficient to establish every magnitude and duration.

## Base summon and Imp facts

| Summon spell | Creature template (not display ID) | Cast time | Base-mana cost | Reagent |
| --- | ---: | ---: | ---: | --- |
| 688 Imp | 416 | 10 sec | 80% | none |
| 697 Voidwalker | 1860 | 10 sec | 100% | item 6265 ×1 |
| 712 Succubus | 1863 | 10 sec | 100% | item 6265 ×1 |
| 691 Felhunter | 417 | 10 sec | 100% | item 6265 ×1 |

Firebolt rank 1 (3110) is Fire school, has a 2-second cast, 30-yard maximum
range, projectile speed 16, and costs 10 mana. Its variable damage and level term
must not be mistaken for one universal constant. Blood Pact (6307) modifies
Stamina; representing it as flat health requires an explicit conversion because
Survivors currently has no primary-attribute model. Fire Shield (2947) is a
separate reflected-damage effect, not Firebolt splash.

Summon spell creature templates do not prove model/display identity. Verify the
selected creature rows, every referenced model and texture against the checkout
before rendering. Never use the installed Applications game as evidence.

## Integration work, in order

1. Resolve saved pre-run choice versus acquired summon. Define stable species keys,
   default/migration behavior, unlock rules and interaction with the six-slot limit.
2. Extract reusable combat-pet ownership, health, defeat and lifecycle handling
   while preserving the Hunter wolf. Enemy attention currently queries `HunterPet`
   directly; nearby attention and `PetTaunt` are not a general threat table.
3. Add verified demon specimens and a noncombat preview. Keep species combat behavior
   separate from pickup-companion collection/instincts. Use the existing navigation,
   collision and line-of-sight interfaces; do not substitute wall-penetrating attacks.
4. Implement one complete Imp slice: casted Firebolt, owner aura, reflected shield,
   damage attribution, defeat/recovery and the chosen starter/acquisition flow.
   A wolf with an Imp model does not satisfy this step.
5. Wire Fel Stamina, Improved Imp and Improved Firebolt only once all described
   consumers exist. Implement other species and their consumers before their talents.
6. Finish mana/threat/resistance/sacrifice dependencies and the remaining nodes.
   Missing mechanics stay explicitly pending, without claiming definition coverage.

The hit pipeline now carries explicit `Hero`/`CombatPet` origin on direct hits and
periodic records; the Hunter wolf emits `CombatPet`. Origin survives splash and
queued settlement, replacing the old `Hunter Pet` string exception. Pet hits bypass
owner creature-slaying bonuses, ranged mark power, weapon poisons and owner talent
triggers (including Seal Fate, Ignite, Combustion and periodic Nightfall). They retain
existing target-vulnerability, multistrike, leech, meter and loot settlement.
Future demons must supply their own spell metadata and concrete caster/lifecycle
state; the origin tag does not implement demons, casts or pet resource policies. Decide which owner stats are
intentionally inherited and test that list; do not inherit everything by accident.
Hunter frenzy and Spirit Bond remain Hunter-specific. Pet damage retains normal
victim death, meter, loot and leech settlement without duplicating kill credit.

## Acceptance

Cover each species/rank and unrelated classes; summoned versus absent/dead/sacrificed
states; pause, restart and new-run reset; target loss, blocked shots, floor changes,
leash and invalid spawn positions; cast cancellation and at-most-once impacts;
owner-versus-pet modifiers, damage schools and lethal settlement. Save migration,
selected hero/spec, refund and reload must preserve the chosen policy. Verify real
native wide/compact choice and green inline talent descriptions, then attended
combat and Windows behavior. No demon runtime or new talent is accepted by this
audit alone.

Validation: fifteen installed spell reports exercise casted/instant spells, signed
health cost, base-mana percentage, reagent and weapon restriction fields. The audit
build and eight-package gate pass (formatting, Clippy, tests; 31 optional-addon
skips). This is evidence/tooling validation, not demon gameplay acceptance.

Hit-origin checkpoint: 584 focused runtime tests pass, including deliberately
colliding hero/pet labels, owner poison/Seal Fate, Ignite/Combustion/Scorch, periodic
Nightfall, splash/periodic propagation and one-time lethal loot/leech settlement.
The native play build passes in 3m11s. The eight-package formatting/Clippy/test gate
also passes (31 optional-addon skips). No demon model or interface is enabled.


## Verified model identities — 2026-10-29

The hash-checked MaNGOS Zero creature catalog resolves the summon entries to the
following display IDs. [Identity output](demon-identities.csv) is reproducible with
`python3 docs/audits/demon-identities.py target/survivors-validation`; it reuses the
pinned catalog checksum from [the equipment audit](ENEMY-EQUIPMENT.md).

| Demon | Template | Display | Model scale | Render triangles |
| --- | ---: | ---: | ---: | ---: |
| Imp | 416 | 4449 | 0.5 | 622 |
| Voidwalker | 1860 | 1132 | 0.8 | 1,414 |
| Succubus | 1863 | 4162 | 1.0 | 2,338 |
| Felhunter | 417 | 850 | 0.5 | 843 |

`cargo run -p survivors-audit -- wow-data/Data --creature-visuals 4449,1132,4162,850`
parses each model using its CreatureDisplayInfo skin variations, rejects unresolved
skin slots and empty geometry, and decodes all eight distinct mesh textures.
The Succubus resolves body, wings and hair separately. Paths are deduplicated without
case sensitivity, as in the MPQ lookup. Character-composed NPCs are explicitly
rejected by this plain-creature audit instead of claiming their body atlas is checked.
Triangle counts include render batches and are not unique geometric polygon counts.

This verifies mesh assets only. Particle textures, animation suitability, scale in
scene, native screenshots and combat behavior remain open. No install assets are
copied into the repository, and no demon is enabled by this checkpoint.

Validation: all four displays pass the installed-data audit; empty/malformed lists
and nonexistent displays fail. The eight-package formatting/Clippy/test gate passes
with 31 optional-addon skips. No native render is claimed.


## Native specimen acceptance — 2026-10-29

Set `WOW_SURVIVORS_DEMON_PREVIEW=1` for a development build to replace the menu
view with four labeled specimens. Use an isolated `BENILLA_HOME`, checkout
`WOW_DATA`, `WOW_UNATTENDED=1` and `WOW_NOSOUND=1`; do not start a run.
The preview positions each specimen on a resolved floor and uses the regular unit
model path, with no enemy or combat-pet component. The default model scale is retained.

The native macOS 1280×720 render shows all four models and their skins, including
Imp glow, Voidwalker glow/vortex, Succubus wings and Felhunter appendages. The build
finished in 1m34s. The log confirms a network-off visual session, with no asset-load
error or panic; startup frame hitches were present. This capture establishes visible
appearance, not animation timing, performance, Windows, spells or combat behavior.

![Native demon specimens](../images/screenshots/2026-10-29-demon-specimens/lineup.png)

The eight-package formatting/Clippy/test gate passes (31 optional-addon skips),
including 4,819 passing app unit tests. Windows visual acceptance remains open.


## Shared health prerequisite — 2026-10-29

The Hunter wolf now uses `survivors_core::combat_pet::Vitality`. It owns health,
maximum health, a configured recovery interval and the first lethal-hit deadline.
Healing cannot resurrect it, repeated lethal hits cannot postpone recovery, and
maximum-health changes retain its health fraction. Hunter frenzy and Spirit Bond
remain separate. The runtime still controls corpse visuals, valid respawn placement,
attention and active run time. Enemy attention still queries `HunterPet` directly.

This is a reusable health primitive, not full demon lifecycle acceptance. Demons
must explicitly distinguish sacrifice from defeat and define their summon/recovery
policy before consuming these rules; do not copy automatic wolf recovery blindly.

Shared-health validation: 304 core and 584 runtime tests pass; the native play
build succeeds in 1m58s. No new screenshot is implied by this rules-only change.

The final eight-package formatting/Clippy/test gate passes (31 optional-addon skips).
