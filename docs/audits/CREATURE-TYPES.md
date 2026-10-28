# Creature-type roster audit — 2026-10-28

This records explicit runtime identity metadata for the pending
Murder/Monster Slaying/Humanoid Slaying work. It is not an implemented damage
bonus or zone acceptance result.

Input: the [pinned MaNGOS Zero creature catalog](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql)
already used by [the location audit](../SURVIVORS-LOCATIONS.md).
The script checks its SHA-256 before parsing. The roster snapshot is
`crates/survivors-core/src/data.rs` at `1c4fd1c`, SHA-256
`c30a747cb4d8fe8c8657a041b144a1387bfd553d6f3185411fe8a55a830df8b9`.

The [CSV](creature-types.csv) covers 476 unique authored name/display pairs:

- 434 match one catalog entry by both name and one of its four displays.
- 18 authored Warsong/Arathi bosses use explicit archetype entries documented
  in the location audit. Their custom names are not vanilla boss claims.
- 24 preserve legacy appearances while explicitly assigning the intended
  creature's category. The mapping is listed in the audit script's `LEGACY`
  table; these displays are not asserted to belong to their assigned templates.

The initial audit at `d00f566` left those 24 rows unresolved. The category
decision now follows the named creature's template for 22 rows, while leaving
the display unchanged. Inspect the models before changing their appearances. In
particular, scaled-up creatures are not necessarily Giants, and the Warsong
Shredder Foreman is Mechanical despite its boss role and faction association.

Two additional rows use explicit authored archetypes: display
11178 is shared by Crypt Walker (8556) and Necrofiend (11551), both Undead;
neither is named Crypt Fiend. Survivors uses Crypt Walker's Undead category for
its authored Crypt Fiend. Hederine Slayer (7463) is a Demon with display
9018, which differs from the authored The Hederine Slayer display 10729.
Survivors uses that Demon's identity for its named encounter. These are explicit
authoring decisions, not automatic name aliases or model replacements.

Reproduce from the repository root using the existing checkout-local catalog:

```sh
python3 docs/audits/creature-types.py target/survivors-validation/vanilla-creature-template.sql > /tmp/creature-types.csv
diff -u docs/audits/creature-types.csv /tmp/creature-types.csv
```

This offline audit validates every source annotation against the pinned template
and the explicit authoring decisions. All 504 roster constructors (476 unique
pairs) carry template/type metadata through ladder generation. Spawned enemies
retain their full `EnemyDef`; no runtime name or model lookup is used.
Unknown types match no creature mask, even an all-bits mask. Core tests exercise
all generated rosters and the vanilla masks, including Murder's Critter bit.
Next steps are settling the critical damage and pet-inheritance rules, then
implementing and testing per-victim damage settlement. Talent definition
coverage remains 291/432. Legacy artwork mismatches still need visual review.
