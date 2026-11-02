# Collection body-art acceptance — 2026-10-30

The Collection audit now checks the armor overlays selected by the same
`equip_blits` plan used by character rendering, for both male and female bodies.
It checks each piece alone so other equipped layers cannot conceal a missing
asset. Resolution follows the compositor: first decodable unisex texture, then
the corresponding sex-specific texture. Missing or corrupt candidates are
reported with item, display, inventory, sex and layer. The repair below is explicit.

```sh
cargo run -p survivors-audit -- wow-data/Data --collection
```

The initial raw-source audit supplied 746 distinct decoded body textures and
passed 645 of 646 unique display/inventory/sex checks. All nine classes retain their
54 named ordinary cells and 17 supplemental identities; Priest initially had one
body-art review gap: **Virtuous Robe**, item 22083, display 34633, inventory 20, male lower arm.
The referenced `Robe_DungeonPriest_A_02Gold_Sleeve_AL_U.blp` and `_M.blp` are absent
from `Item/TextureComponents/ArmLowerTexture/`. The `_F.blp` exists and decodes.
The chain also lists both male and female upper-arm sleeves, plus a unisex bracer
texture. The native comparison below establishes the chosen sleeve fallback.

The initial audit established a source-art asymmetry, **not yet proof of a rendering defect**.
The compositor already skips an unavailable layer, as it does for the original
source data. The native follow-up below checks this robe alone and with matching wrist/glove
pieces before applying a Survivors-specific repair.

The initial audit deliberately failed while this review gap remained. The code tests cover
candidate precedence, fallback after decode failure, total-failure diagnostics and
inventory-to-compositor slots. This does not validate head/shoulder/held meshes,
combined-layer ordering, body fit, attachment alignment or Windows rendering.
No new rendered screenshot is claimed for this asset-decoding checkpoint.

Initial audit checkpoint validation: the eight-package scoped formatting, Clippy and test gate passes
with 31 optional-addon skips (`/tmp/collection-body-check.log`). After the final
variant-count deduplication, audit-only all-target Clippy and all 16 tests pass
again; the final collection run retains precisely the one gap above. These are
source/asset checks, not native or Windows visual acceptance.

## Native comparison and scoped repair

The production Armory page at 1280×720 shows the male robe's flared lower cuffs
painted as skin when the robe is worn alone. The female version has fabric cuffs.
Equipping Virtuous Bracers and Gloves covers the male gap, which explains why
full outfits can conceal the missing sleeve asset.

Survivors now tries the robe's existing female lower-sleeve texture only after
its exact male candidate fails to read/decode. Existing unisex and male textures
retain priority. No other item path or female candidate is substituted. This is
an authored appearance repair, not a claim that the original client uses this
fallback. No MPQ data is modified or bundled. The shared native body-compositing
path uses it only with Survivors active and the Survivors UI feature enabled;
ordinary-client runs keep their original behavior. The repaired sleeve follows
the existing layer ordering over bracers; hands still use the selected gloves.

Before, male robe alone:

![Male robe with skin-colored cuffs](../images/screenshots/2026-10-30-virtuous-robe/male-robe.png)

After, same male robe alone:

![Male robe with repaired fabric cuffs](../images/screenshots/2026-10-30-virtuous-robe/male-robe-fixed.png)

Female control before and after:

![Original female robe](../images/screenshots/2026-10-30-virtuous-robe/female-robe.png)

![Female robe after the scoped repair](../images/screenshots/2026-10-30-virtuous-robe/female-robe-control.png)

Matching pieces, before the repair (male and female):

![Male robe with matching bracers and gloves before repair](../images/screenshots/2026-10-30-virtuous-robe/male-matching-pieces.png)

![Female robe with matching bracers and gloves](../images/screenshots/2026-10-30-virtuous-robe/female-matching-pieces.png)

Matching pieces after the repair:

![Male repaired robe with bracers and gloves](../images/screenshots/2026-10-30-virtuous-robe/male-matching-fixed.png)

The final collection command passes **646/646 body variants**, 746 decoded body
textures and 498 decoded icons. Its output explicitly identifies this single
Survivors fallback rather than claiming the missing male asset exists. Three
new pure-rule tests cover original priority, successful/failed fallback, disabled
mode and unrelated/female/unisex paths. The native play build took 41.34 seconds.
Native evidence is Human male/female on macOS, menu only, using an isolated fixture
and checkout data. Other races, Windows and attended gameplay remain unverified.

Reproduction uses the `collected` UI fixture with hero 5 (Human Priest), the
`heroes` preview page at 1280×720, and isolated account row
`equipment 4 002000000 000000000` for the robe alone. Change to
`equipment 4 002000220 000000000` for matching bracers and gloves, or equip them
through Collection. Use the page's Male/Female controls for the comparison.
Prepare a new fixture directory and edit its account only while the app is closed;
never replace a player account. The live preview request itself can be edited
while the app runs. The fixture selector's `look` page is the legacy world menu;
`heroes` selects the modern Armory character appearance page used here.

Final repair validation: all eight scoped packages pass formatting, Clippy and
tests, with 31 optional-addon skips (`/tmp/robe-fallback-check-repaired.log`).
The first attempt exhausted disk space; the next exposed incomplete test-object
linkage. Removing obsolete quarantined artifacts and rebuilding only the affected
`benilla-app` test incremental cache resolved both. Shared dependency caches and
the 41-second native play build were retained. No source changes were needed for
that environment repair.

## Attachment asset coverage — 2026-10-30

The same `--collection` command now also checks head, shoulder and held mesh
assets for every ordinary/supplemental collection identity and alternative weapon.
It mirrors `entities/equipment/mod.rs::ensure_item_model`: independent model and
skin basenames; both authored shoulder columns; Weapon/Shield folders; and the
16 helmet suffixes (`Hu`, `Or`, `Dw`, `Ni`, `Sc`, `Ta`, `Gn`, `Tr`, each M/F).
An absent model column is deliberately model-less, including one-sided shoulders.
Clothing remains covered by the separate body-overlay audit.

The checkout passes **1,055 display/inventory attachment variants**, **869 unique
parsed models with render triangles**, and **221 selected decoded textures**.
The model loader supplies embedded texture references; type-2 object batches
require their display-selected skin. Reusing one model never skips another
item's skin. Unused display skin names do not establish a rendered dependency.
Missing geometry, required skins, unreadable files or invalid textures fail the
collection command with item/display context. No game data is modified or copied.

Three regressions cover race/body suffixes, independent shoulder columns and
skin names, inventory directories, legitimate model-less/clothing cases, empty
geometry, missing required skins and different skins on a shared model. All 19
audit tests pass. The retained native build completes in 3.13 seconds.

This closes source-asset coverage for these attachments, **not** attachment
alignment, combined outfits, particles, animation, native race/body previews or
Windows acceptance. Runtime/UI rendering is unchanged; no new screenshot is
claimed. The native screenshots above remain evidence only for their shown cases.

Final verification: the eight-package formatting/Clippy/test gate passes with
31 optional-addon corpus skips. The collection audit has no reported gaps.

## Native Gnome/Tauren plate checks — 2026-10-30

Native macOS menu-only validation of the `a7be32e` runtime (unchanged by the
`3f95c1c` audit) uses a disposable collected fixture, hero 1, Warrior equipment
`666666666`, initially all worn appearances `000000000`: full Dreadnaught armor
and Thunderfury. Race/body and look changes thereafter use native pointer input.
The primary layout reference remains `intentions/survivors_character_tab.png`.

Both Gnome bodies render the helmet, shoulders and held weapon at 1280×720.
Female rotation follows the turn buttons. Hiding only the weapon appearance
reveals the armor and allows the existing camera fit to move closer; equipment
bonus text remains unchanged. Both Tauren bodies show the full armor with the
weapon hidden, and the male also shows the restored weapon. These captures prove
visible assembly and containment for the shown angles, not every animation or
absence of clipping from every viewpoint. Thunderfury obscures the small body's
torso from the default angle; the hidden-weapon view is the stronger armor evidence.

After restoring then hiding the Tauren male weapon through UI, restart at
640×720 retains the body and hidden look. Disk agrees: `body 1 6 0`,
`equipment 0 666666666 000000001`. The compact preview contains the full figure
and rotation controls. No save edits occurred between interaction and restart.
The normal launcher is restored; no combat was started. No source changed, so
the preceding green scoped gate remains applicable. Windows, other outfits,
other race/body combinations and combat animation remain open.

![Gnome male, full outfit](../images/screenshots/2026-10-30-plate-bodies/gnome-male.png)

![Gnome female, full outfit](../images/screenshots/2026-10-30-plate-bodies/gnome-female.png)

![Gnome female, rotated](../images/screenshots/2026-10-30-plate-bodies/gnome-female-turned.png)

![Gnome female, weapon appearance hidden](../images/screenshots/2026-10-30-plate-bodies/gnome-female-weapon-hidden.png)

![Tauren female, weapon appearance hidden](../images/screenshots/2026-10-30-plate-bodies/tauren-female-weapon-hidden.png)

![Tauren male, weapon appearance hidden](../images/screenshots/2026-10-30-plate-bodies/tauren-male-weapon-hidden.png)

![Tauren male, full outfit](../images/screenshots/2026-10-30-plate-bodies/tauren-male.png)

![Tauren male, compact restart](../images/screenshots/2026-10-30-plate-bodies/tauren-male-restarted-compact.png)

## Native remaining-race plate matrix — 2026-11-01

Native macOS menu captures at `b3bdd4b` add Human, Dwarf, Night Elf, Orc,
Undead and Troll, male and female, at 1280×720 / 100%. Together with the
Gnome/Tauren checkpoint above, all sixteen race/body combinations now have
Dreadnaught outfit evidence. The intention reference remains
`docs/intentions/survivors_character_tab.png`: appearance/equipment at left,
hero in the center, features and independent looks at right.

The isolated `collected` fixture uses hero 1 and equipment row
`equipment 0 666666666 000000001`: full Dreadnaught with Thunderfury equipped
but its appearance hidden to expose the torso. Race/body changes use the native
page controls. All twelve added views contain the full figure, helmet and both
shoulders. These are front/idle assembly and framing checks, not proof of
clipping-free animation, every outfit, compact layouts or Windows rendering.

At this checkpoint Troll footwear was an art-rule review item (resolved by the
correction below): the worn look covered the feet.
The additional male hidden/restored comparison verifies that hiding footwear
reveals bare feet and restoring it restores the armor, with unchanged equipment
bonus text. This does not establish whether the worn Troll texture/geoset matches
vanilla's race-specific footwear rules; do not mark that correspondence accepted.

No production source changed. The warm native build completed in 0.46 seconds;
prior code gates remain applicable. Generated output is 46.62 GiB, within budget.
The preview exited and its normal launcher was restored. No combat was started.

![Human male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/human-male.jpg)

![Human female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/human-female.jpg)

![Dwarf male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/dwarf-male.jpg)

![Dwarf female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/dwarf-female.jpg)

![Night Elf male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/night-elf-male.jpg)

![Night Elf female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/night-elf-female.jpg)

![Orc male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/orc-male.jpg)

![Orc female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/orc-female.jpg)

![Undead male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/undead-male.jpg)

![Undead female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/undead-female.jpg)

![Troll male, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/troll-male.jpg)

![Troll female, Dreadnaught](../images/screenshots/2026-11-01-plate-matrix/troll-female.jpg)

![Troll male footwear hidden](../images/screenshots/2026-11-01-plate-matrix/troll-male-feet-hidden.jpg)

![Troll male footwear restored](../images/screenshots/2026-11-01-plate-matrix/troll-male-feet-restored.jpg)

## Barefoot race texture correction — 2026-11-01

The preceding native Troll screenshots exposed an actual compositor omission.
Checkout `patch.MPQ`'s `ChrRaces.dbc` has nine rows, 29 fields and 116-byte
records. Column 1 is `0x0e` for Tauren (6) and Troll (8), `0x0c` for Human,
Orc, Dwarf, Undead and Gnome, `0x04` for Night Elf, and `0x01` for Goblin.
Flag `0x02` is "Do Not Component Feet" in [TrinityCore's field documentation](https://trinitycore.info/files/DBC/335/chrraces).
[WoW Model Viewer's equipment compositor](https://wowmodelviewer.github.io/wowmodelviewer/WoWItem_8cpp_source.html)
also suppresses the foot texture when showing bare feet while retaining the
lower-leg layer. These external implementations corroborate the flag meaning;
the race values come from this checkout's 1.12.1 patch chain.

Survivors now loads the installed flags once per skin table and filters only the
foot tile from a flagged race's composition plan. It retains the base skin,
all other overlays in order, and existing boot geoset selection. Equipment,
bonuses and saved appearances do not change. The carry is gated by the existing
Survivors feature and active mode; the normal client's composition is unchanged.
Missing race data logs a warning and retains the previous rendering behavior.

Regression checks cover installed race flags and exact preservation of every
non-foot layer, including idempotence. Native macOS verifies the same saved Troll male worn-boot state before/after,
both Troll bodies, both Tauren bodies, and unchanged covered boots on Human male.
Lower-leg armor remains visible in all five new captures. The fixture remains
`plate-matrix-20261101` with equipment `666666666` and looks `000000001`;
no account edits were made for this follow-up. The app exited and the normal
launcher was restored. Windows, other outfits and gameplay animation remain open.

The first development launch exposed a recursive archive lock in the new loader.
The final loader consumes startup's existing mutable chain guard; it does not
lock again in `SkinSections::new`. The corrected native startup and all five
preview interactions complete without a panic or race-table warning. Initial
native rebuild: two minutes; final correction: 19.31 seconds. The linker warns
that its unwind section exceeds compact-unwind offset capacity; the build succeeds.


![Troll Male, race-correct footwear](../images/screenshots/2026-11-01-barefoot/troll-male.jpg)

![Troll Female, race-correct footwear](../images/screenshots/2026-11-01-barefoot/troll-female.jpg)

![Tauren Male, race-correct footwear](../images/screenshots/2026-11-01-barefoot/tauren-male.jpg)

![Tauren Female, race-correct footwear](../images/screenshots/2026-11-01-barefoot/tauren-female.jpg)

![Human Male, race-correct footwear](../images/screenshots/2026-11-01-barefoot/human-male.jpg)


Final verification: the ten-package scoped formatting/Clippy/test gate passes,
including both new regressions, with 31 optional addon-corpus skips. The WebGPU
check passes with six existing warnings outside this change. Build output is
53.71 GiB, within budget. No full workspace landing gate or Windows runtime
acceptance is claimed.
