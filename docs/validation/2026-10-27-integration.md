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
