# Enemy equipment evidence — 2026-10-29

Disarm requires an explicit weapon-attack model. The current contact path treats
every enemy contact as physical damage; creature type and appearance do not prove
that it is a weapon attack. Riposte also needs equipped-player weapon damage.

The [reproducible join](enemy-equipment.py) covers all 476 unique authored
name/display identities (504 roster constructors), using the existing checked
template/type mappings. Its [report](enemy-equipment.csv) records three equipment
slots per identity, item classifications where available, and the raw mechanic
immunity mask. It does **not** label an unresolved enemy armed or disarmable.

Sources are the MaNGOS Zero database revision
`325948a78015794e6cc626212bc19fee70c805b2`:
[creatures](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql),
[equipment](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_equip_template.sql), and
[items](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/item_template.sql).
The script pins all three SHA-256 hashes and checks schema, duplicate IDs and
complete roster identity coverage. Downloads remain in ignored validation storage.

```sh
python3 docs/audits/enemy-equipment.py target/survivors-validation > /tmp/enemy-equipment.csv
cmp /tmp/enemy-equipment.csv docs/audits/enemy-equipment.csv
```

## Findings

Of 1,428 slot records, 825 belong to enemies with no equipment template, 326
are empty slots, 28 match item rows, 240 reference missing item entries, and nine
belong to missing equipment templates. The missing item references cover 98 IDs
and 182 enemy names. Missing templates are Lord Azrethoc (1260), Scarlet Torturer
(805), and Skarr the Unbreakable (2430). These are evidence gaps, not empty slots.

## Implementation sequence

1. Resolve NPC virtual-item metadata from the explicit checkout data and verify
   the server's equipment interpretation. Preserve unresolved entries; do not
   substitute player weapon data based on a matching name or model.
2. Define authored attack channels (weapon, natural, spell), explicit immunity
   policy and disarmed fallback damage. A zero equipment template alone does not
   prove that a model lacks a built-in weapon. Keep spell damage independent.
3. Carry weapon ownership and disarm expiry through enemy spawn/contact/visual
   state, with refresh, pause, death, immunity and pet-target regressions.
4. Add equipped-player weapon damage before Riposte's 150% hit, six-second
   disarm and parry-trigger acceptance. Keep Stormstrike's pending design decision
   separate. Replace Improved Disarm's current weakening-duration substitute only
   when actual disarm exists.

This is a data audit, not a gameplay change. No new screenshot is implied;
runtime disarm and weapon visuals still require implementation and validation.
