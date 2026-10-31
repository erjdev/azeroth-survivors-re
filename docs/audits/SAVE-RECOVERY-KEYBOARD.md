# Native keyboard save recovery — 2026-10-30

Verified on macOS at 1280×720 / 100%, using the existing native build with the
Paladin weapon-specialization implementation recorded at `4be2d99`. No runtime
code changed or rebuild was needed for this acceptance pass.

## Procedure and evidence

A copy of the synthetic Paladin keyboard fixture was isolated under
`target/ui-fixtures/paladin-save-keyboard`. An empty directory at
`benilla-config/survivors-account.tmp` blocked the atomic temporary-file write.
The real player profile was untouched.

1. Arrow keys selected One-Handed Weapon Specialization; Enter purchased 4/5 → 5/5.
   The warning appeared and the new rank remained in memory. The saved account
   remained byte-identical to the baseline.
2. Arrows reached Retry save (from Done: Down to Spec 2, Right to Spec 3, Up).
   Enter retried while the obstruction remained. Warning and keyboard focus
   persisted; the saved account was still byte-identical.
3. Removed only the empty test obstruction. Enter on Retry save cleared the warning.
   A byte comparison proved the sole persisted change was `1:1429:4` → `1:1429:5`.
4. Closed and relaunched the preview. Rank 5/5, 30 Protection points and 21 unspent
   points persisted, without a warning. The account still matched that exact change.
   Closed the preview and restored its launcher environment.

All purchases and retry actions used the keyboard. Window closing used the native
close control. Captures below are original JPEG screenshot bytes.

![Failed keyboard retry retains warning and focus](../images/screenshots/2026-10-30-save-keyboard/failed-retry.jpg)

![Successful keyboard retry clears warning](../images/screenshots/2026-10-30-save-keyboard/recovered.jpg)

![Recovered rank survives restart](../images/screenshots/2026-10-30-save-keyboard/restarted.jpg)

## Scope and limits

This closes the wide native keyboard save-recovery case in phase 1. It extends the
existing Armory layout without a design change; the talent artwork remains the
reference-aligned layout already reviewed in the class matrix. Compact keyboard,
physical controller, Windows and browser-storage failures remain unverified.
The prior full gate covers unchanged runtime code; this round checks documentation
and captures only. It does not prove all progression or platform acceptance.
