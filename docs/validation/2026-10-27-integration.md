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
