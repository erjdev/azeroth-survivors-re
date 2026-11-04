# Feral Charge dependency audit — 2026-11-04

This audit prepares node 804; it does not count Feral Charge as implemented.
Only checkout `wow-data/Data` was consulted, using `survivors-audit` with
`--talent-evidence 804`, `--spell-evidence 16979` and `--spell-evidence 19675`.

## Reference and required adaptation

The talent has one rank, row 2/column 1, no prerequisite node, and teaches 16979.
That spell requires Bear/Dire Bear (mask 0x90), has 8–25-yard range and 15-second
recovery, and triggers 19675. The triggered effect has four-second duration,
interrupt effect 68 and root aura 26. Its raw Rage cost is 50; Survivors must not
introduce a Rage pool. A root is not a stun and does not inherently stop casting.

Input is the remaining player-facing choice: replacing the existing manual Dash
on the mobility button, or adding an automatic third starting ability that moves
the hero. The first preserves movement control and the existing mobility slot;
the second must obey the three-start/six-total limit. A clarification is pending.
Neither interpretation is silently treated as approved.

## Existing paths and implementation work

- `mobility::cast_mobility` has Warrior Charge selection, cursor targeting, map
  leash clamping and `Floor::travel` body/floor sweeps. It applies control only
  after landing within two yards with a clear line to the target. The existing
  no-target lunge and spent cooldown on blocked movement are Warrior behavior,
  not automatically a Feral Charge requirement.
- Feral Charge needs its own minimum-range check and form/talent gate. Use the
  selected run snapshot, including refunds/spec changes at the next run. Do not
  turn a skipped target into an uncontrolled forward lunge.
- Use the body sweep even when visual line of sight is clear: a low fence can
  leave the target visible while blocking the player's capsule. Check the
  reached target after movement, never apply root/interrupt across an obstacle.
- Root duration and active-cast interruption are separate. Call the existing
  `Caster.state.interrupt` with a four-second school lock only after a valid
  arrival; respect interrupt immunity and cast start/completion times. Ordinary
  roots use four seconds; the existing boss root reduction yields 1.2 seconds,
  while bosses retain the existing interrupt immunity policy.
- Keep other ability and automatic-attack timers running independently. Recovery
  should follow the selected implementation's haste/cooldown rules, with no
  shared spell lock or resource system. Area should scale maximum reach; do not
  multiply player displacement for extra targets.
- If it replaces Dash, update the resolved mobility description/icon and the
  level-five learned banner: that banner currently reads the class default
  rather than `Run::mobility`. The implementation contract should make a tree-learned replacement available
  from run start, consistent with pre-run talent grants; untrained Dash keeps its
  existing level-five unlock.
- If it is slotted instead, add the ability definition, tree starter grant,
  acquired-rank upgrades and runtime form guard through the shared paths.

## Verification boundary

A new ECS regression exercises the existing manual Warrior Charge against an
unobstructed target, a visible target behind a low fence, a tall wall and missing
floor. It checks reached-only control, no health damage, and untouched slotted
and automatic-attack timers. All four cases pass. The ten-package scoped format/Clippy/test gate passes
with 31 optional addon-corpus skips; build output is 62.54 GiB. Production code
is unchanged, so no new native build or UI screenshot is claimed.
This is synthetic collision evidence, not attended Elwynn acceptance or proof
that Feral Charge has been implemented.

Feral-specific acceptance still needs form/refund/spec/start rules, minimum and
area-scaled maximum range, missing/dead/blocked targets, root and interrupt
settlement (including immune/too-late casts), pause/loading/cooldown behavior,
and native wide/compact screenshots plus saved restart once the UI changes.
