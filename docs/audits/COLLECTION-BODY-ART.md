# Collection body-art acceptance — 2026-10-30

The Collection audit now checks the armor overlays selected by the same
`equip_blits` plan used by character rendering, for both male and female bodies.
It checks each piece alone so other equipped layers cannot conceal a missing
asset. Resolution follows the compositor: first decodable unisex texture, then
the corresponding sex-specific texture. Missing or corrupt candidates are
reported with item, display, inventory, sex and layer; no substitute is invented.

```sh
cargo run -p survivors-audit -- wow-data/Data --collection
```

The checkout patch chain supplies 746 distinct decoded body textures and passes
645 of 646 unique display/inventory/sex checks. All nine classes retain their
54 named ordinary cells and 17 supplemental identities, but Priest has one body-art
review gap: **Virtuous Robe**, item 22083, display 34633, inventory 20, male lower arm.
The referenced `Robe_DungeonPriest_A_02Gold_Sleeve_AL_U.blp` and `_M.blp` are absent
from `Item/TextureComponents/ArmLowerTexture/`. The `_F.blp` exists and decodes.
The chain also lists both male and female upper-arm sleeves, plus a unisex bracer
texture; those are distinct assets, not approved replacement lower-arm sleeves.

This is an unresolved source-art asymmetry, **not proof of a rendering defect**.
The compositor already skips an unavailable layer, as it does for the original
source data. Native male/female inspection of this robe, alone and with matching
wrist/glove pieces, is required before deciding whether any Survivors override is
appropriate. No runtime art, save, equipment identity or appearance behavior changed.

The audit deliberately fails while this review gap remains. The code tests cover
candidate precedence, fallback after decode failure, total-failure diagnostics and
inventory-to-compositor slots. This does not validate head/shoulder/held meshes,
combined-layer ordering, body fit, attachment alignment or Windows rendering.
No new rendered screenshot is claimed for this asset-decoding checkpoint.

Validation: the eight-package scoped formatting, Clippy and test gate passes
with 31 optional-addon skips (`/tmp/collection-body-check.log`). After the final
variant-count deduplication, audit-only all-target Clippy and all 16 tests pass
again; the final collection run retains precisely the one gap above. These are
source/asset checks, not native or Windows visual acceptance.
