# Tree-granted starting abilities — 2026-11-01

Implements the approved starting-kit rule for the three existing explicit
ability-unlock talents: Priest Mind Flay/Silence and Paladin Repentance. Training
now grants rank one at run start alongside the class default, instead of merely
unlocking a future random card. A fully trained Priest starts with Smite, Silence
and Mind Flay; Paladin starts with Hammer of Wrath and Repentance. Automatic
weapon attacks remain separate and use no ability slot.

## Shared behavior

`Milestones::starting_abilities` supplies both the starter's “Starts with” list
and `Run::finish_start`. Mouse entry, keyboard entry and the production selected-run
preview already call that common initialization after obtaining the selected
spec snapshot. Each newly granted spell starts at rank one with the existing
0.5-second initial recovery. Existing owned entries are not duplicated. Foreign
class snapshots cannot grant spells. Refund/spec changes affect subsequent runs;
existing run snapshots are not rewritten. No save-format change is required.

The catalog regression checks all classes and all currently defined unlocks fit
three starting slots. Future additions that exceed this limit must implement an
explicit choice/replacement policy; do not truncate learned spells in catalog
order. Level-up offers upgrade owned spells and cannot acquire a seventh ability.
Other talent conversions, new feral abilities and broader melee automatic attacks
remain work. Definition coverage stays 319/432 plus three authored form nodes.
The bulk design recommendations are still unanswered, not approvals.

## Evidence and limits

Shared tests cover empty/trained/refunded kits, wrong classes, uniqueness and the
three-slot bound. App tests cover startup, initial rank/recovery, duplicate
prevention, independent subsequent specs, and upgrade offers at the six-slot cap.

Native macOS production preview uses isolated fixture
`target/ui-fixtures/starting-talents-20261101`, seeded before launch. The three-spell
Priest kit is readable at 1280×720 / 100% and 640×720 / 125%. Native right-click
refund of Mind Flay removes it from the starter; compact restart preserves the
remaining Smite/Silence kit. The revised talent description highlights Mind Flay
in green. No gameplay was launched. Paladin runtime initialization is regression
tested; its native presentation, Windows and attended combat remain open.

![Three starting abilities, wide](../images/screenshots/2026-11-01-starting-talents/priest-wide.jpg)

![Three starting abilities, compact](../images/screenshots/2026-11-01-starting-talents/priest-compact.jpg)

![Mind Flay explicitly grants a starting ability](../images/screenshots/2026-11-01-starting-talents/mind-flay-talent.jpg)

![Refund removes Mind Flay from the starter](../images/screenshots/2026-11-01-starting-talents/priest-refunded-wide.jpg)

![Refund retained after compact restart](../images/screenshots/2026-11-01-starting-talents/priest-refunded-restart.jpg)

Validation: 360 shared-rule tests, the ten-package scoped gate and final
same-scope all-target Clippy/format checks pass. There are 31 optional-addon
corpus skips. Native play build (45.22 seconds) and WebGPU check pass with the
existing linker unwind warning and six WebGPU warnings. Generated storage:
57.05 GiB; caches preserved. Full landing gates remain required before merge.
