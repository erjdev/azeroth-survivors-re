# Native integration checkpoint — 2026-10-27

The pre-sidebar-layout baseline (`dd6bb2e`, Rust changes through `876d69f`)
passes `WOW_DATA="$PWD/wow-data/Data" scripts/check.sh` on macOS. This is the
terminal result of the existing full run, not a new run per progress commit.

| Stage | Result | Seconds |
| --- | --- | ---: |
| Formatting | Pass | 7 |
| Workspace all-target Clippy | Pass | 91 |
| Workspace tests with checkout data | Pass | 751 |
| Workspace tests without an install | Pass | 37 |
| Documentation links | Pass | 15 |
| Render-pass lint | Pass | 2 |
| Player tests without developer features | Pass | 326 |

The whole chain took 1,323 seconds including setup. The app unit suite had
4,684 passes and 10 ignored tests. The data/player stages each reported 31
addon-corpus skips: 24 require the vanilla addon corpus and seven require Quiver.
The dedicated feature-enabled Survivors suite separately passed all 449 tests.
The strengthened documentation gate's three failure-handling regressions pass.

This does not establish Windows or browser execution, GPU scene acceptance,
addon compatibility without the missing corpus, combat balance, complete talent
coverage, signed distribution, or the full seven-phase goal. No live realm smoke
or unattended combat session was performed. Later UI edits need their own checks.

## Talent-sidebar follow-up

The same full command passes for `a0a6ca1`: formatting (6 s), workspace Clippy
(20 s), workspace tests (294 s), no-install tests (24 s), documentation links
(47 s), render-pass lint (1 s) and player tests (117 s). Total: 510 seconds.
The data/player stages still report the same 31 addon-corpus skips. Native
1600×900 and 800×600 visual checks are captured in the changelog; Windows visual
confirmation remains open.

## Worn-weapon talent integration

The full command passes for the code at `1cd935a` (later `ec98bf3` changes only
documentation and screenshots): formatting 7 s, workspace all-target Clippy 65 s,
workspace tests 564 s, no-install tests 39 s, documentation links 57 s, render-pass
lint 1 s and player tests without developer features 297 s. Total: 1,031 seconds
(17m11s), terminal exit 0. Both data/player stages report the same 31 addon-corpus
skips: 24 missing vanilla corpus and seven missing Quiver.

The separate feature-enabled Survivors suite passed 456 tests, and core rules
passed 230. Native selected-profile card screenshots at 1280×900 confirm the
equipped-weapon multiplier reaches displayed upgrade values. This checkpoint
does not establish Windows/browser execution, live realm smoke, live combat
balance, missing addon compatibility or completion of the seven-phase goal.
