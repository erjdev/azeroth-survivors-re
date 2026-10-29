# Enemy equipment evidence — 2026-10-29

Disarm requires an explicit weapon-attack model. The current contact path treats
every enemy contact as physical damage; creature type and appearance do not prove
that it is a weapon attack. Riposte also needs equipped-player weapon damage.

The [reproducible join](enemy-equipment.py) covers all 476 unique authored
name/display identities (504 roster constructors), using the existing checked
template/type mappings. Its [report](enemy-equipment.csv) records three equipment
slots per identity, NPC item classifications, and the raw mechanic
immunity mask. It does **not** label an enemy disarmable merely because it has equipment.

Sources are the MaNGOS Zero database revision
`325948a78015794e6cc626212bc19fee70c805b2`:
[creatures](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_template.sql),
[equipment](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_equip_template.sql),
[NPC items](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_item_template.sql), and
[legacy raw equipment](https://github.com/mangoszero/database/blob/325948a78015794e6cc626212bc19fee70c805b2/World/Setup/FullDB/creature_equip_template_raw.sql).
The script pins all four SHA-256 hashes and checks schema, duplicate IDs and
complete roster identity coverage. Downloads remain in ignored validation storage.

```sh
python3 docs/audits/enemy-equipment.py target/survivors-validation > /tmp/enemy-equipment.csv
cmp /tmp/enemy-equipment.csv docs/audits/enemy-equipment.csv
```

## Findings

Of 1,428 slot records, 825 belong to enemies with no equipment template, 326
are empty modern slots, 268 resolve through NPC item metadata, and nine resolve
through legacy raw equipment. There are 271 nonzero item displays. No equipment
or NPC-item references remain unresolved in this roster.

The first checkpoint joined the player `item_template` table and reported 98
missing IDs plus three missing templates. That was an incomplete lookup, corrected
here: [Creature::LoadEquipment and SetVirtualItem](https://github.com/mangoszero/server/blob/b7fface6b1bd3b036211dcde1bff39cc6da503e4/src/game/Object/Creature.cpp)
use `creature_item_template`, with `creature_equip_template_raw` only when the
modern equipment template is absent. Lord Azrethoc (1260), Scarlet Torturer (805),
and Skarr the Unbreakable (2430) all use that raw fallback. Missing NPC items in a
present modern template must not borrow legacy data.

Raw metadata follows [Creature.h's byte offsets](https://github.com/mangoszero/server/blob/b7fface6b1bd3b036211dcde1bff39cc6da503e4/src/game/Object/Creature.h):
class, subclass, material and inventory type occupy successive bytes of `equipinfo`;
the low byte of `equipslot` holds sheath type. The report preserves these factual
fields and the display ID independently of gameplay damage or eligibility.
Six unit regressions cover lookup precedence, empty slots, packed metadata,
missing NPC items, missing equipment and zero-template behavior. Full report
reproduction passes. Runtime attachment alignment and visual validation remain open.

## Implementation sequence

1. NPC virtual-item metadata and server lookup interpretation are resolved. Next,
   validate these display/model references against the explicit checkout data
   before attaching them to rendered enemies. Preserve the identity audit's
   authored-archetype and legacy-appearance distinctions.
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

## Installed asset validation

```sh
python3 docs/audits/enemy-equipment.py target/survivors-validation --visuals > /tmp/enemy-equipment-visuals.txt
cargo run -p survivors-audit -- wow-data/Data --equipment-visuals /tmp/enemy-equipment-visuals.txt
```

The explicit checkout data passes all 114 display/inventory pairs: 98 unique M2
models parse to renderable triangles and 119 referenced textures decode to nonempty
images. The audit uses the renderer's Shield directory for inventory type 14 and
Weapon directory otherwise, with independently named display textures and embedded
model textures. It never guesses a texture filename from the model basename.

The three legacy templates carry inventory type 1 despite being held items. The
raw server path preserves that value; the audit follows the renderer rather than
silently rewriting metadata. Native attachment/pose review must cover these cases.
The manifest rejects malformed/empty inputs and out-of-range values, deduplicates
identical pairs, and preserves different inventory values for the same display.
This verifies asset decoding, not live enemy attachments or disarm behavior.

Verification: six Python lookup regressions, two Rust manifest regressions,
formatting, Clippy and the eight-package scoped test gate pass (31 optional-addon
skips). No native screenshot is claimed for this headless asset audit.

## Runtime descriptors and native specimens

`survivors_core::enemy_equipment::for_template` now supplies immutable virtual-item
slots to enemy spawning. All 456 template identities resolve explicitly; unknown
IDs remain distinct from verified empty equipment. Existing creature identity and
legacy appearance exceptions remain unchanged. Roster-wide core/runtime tests
check unique slots and exact display/class/subclass/material/inventory/sheath
preservation. Equipment is drawn in melee sheath state; ranged items remain
stowed until a future ranged-attack channel requests them. Damage is unchanged.

Regenerate the catalog from pinned evidence, then format it:

```sh
python3 docs/audits/enemy-equipment.py target/survivors-validation --rust > crates/survivors-core/src/enemy_equipment.rs
cargo fmt -p survivors-core
```

With an isolated selected-hero fixture, a development build accepts
`WOW_SURVIVORS_EQUIPMENT_PREVIEW=116` (or 98/4306). It stays in Menu, hides the menu
UI, and spawns an independent specimen with no combat component. The fixed camera
and separate floor-resolved position prevent the menu hero's attachment parts
from overlapping the specimen. Exit the app to end the preview. Player builds
ignore the switch. Use `WOW_UNATTENDED=1` and `WOW_NOSOUND=1` for agent previews.

Native macOS captures verify visible attachments for Defias Bandit, Riverpaw
Taskmaster and Scarlet Torturer. They are saved inline in CHANGELOG. These static
specimens do not establish attack-animation alignment for every model, live
combat acceptance, Windows rendering, or disarm mechanics. Core tests: 298 pass;
Survivors runtime tests: 576 pass. Native incremental builds: 2m45s initially,
57 seconds after preview isolation correction. Generated catalog reproduction and
six Python lookup regressions pass.

The eight-package scoped gate and final Clippy including tests pass (31 optional-
addon skips). The preview isolation correction was rebuilt and validated natively.
