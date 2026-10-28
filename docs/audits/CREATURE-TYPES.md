# Creature-type roster audit — 2026-10-28

This is evidence for the pending Murder/Monster Slaying/Humanoid Slaying work,
not an implemented damage bonus or zone acceptance result.

Input: the [pinned MaNGOS Zero creature catalog](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql)
already used by [the location audit](../SURVIVORS-LOCATIONS.md).
The script checks its SHA-256 before parsing. The roster snapshot is
`crates/survivors-core/src/data.rs` at `1c4fd1c`, SHA-256
`c30a747cb4d8fe8c8657a041b144a1387bfd553d6f3185411fe8a55a830df8b9`.

The [CSV](creature-types.csv) covers 476 unique authored name/display pairs:

- 434 match one catalog entry by both name and one of its four displays.
- 18 authored Warsong/Arathi bosses use explicit archetype entries documented
  in the location audit. Their custom names are not vanilla boss claims.
- 24 remain unresolved: 22 names have catalog entries with different displays;
  Crypt Fiend and The Hederine Slayer have no exact-name entry.

Candidate entries/displays on unresolved rows are investigation leads, not
approved substitutions. No type is assigned to these rows. Inspect those models
and their existing authored purpose before changing runtime content. In
particular, scaled-up creatures are not necessarily Giants, and the Warsong
Shredder Foreman is Mechanical despite its boss role and faction association.

Two additional leads require an explicit authored identity decision: display
11178 is shared by Crypt Walker (8556) and Necrofiend (11551), both Undead;
neither is named Crypt Fiend. Hederine Slayer (7463) is a Demon with display
9018, which differs from the authored The Hederine Slayer display 10729.
These are not automatic name aliases or approved model replacements.

Reproduce from the repository root using the existing checkout-local catalog:

```sh
python3 docs/audits/creature-types.py target/survivors-validation/vanilla-creature-template.sql > /tmp/creature-types.csv
diff -u docs/audits/creature-types.csv /tmp/creature-types.csv
```

This is an offline authoring audit. Runtime must carry explicit template/type
metadata through roster construction and spawning, rather than match names or
models. Unresolved types must not receive creature-specific bonuses. Next steps
are resolving the 24 identities, adding that metadata, settling the critical
damage and pet-inheritance rules, then implementing and testing per-victim
damage settlement. Talent definition coverage remains 291/432.
