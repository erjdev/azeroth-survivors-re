# Faerie Fire (Feral) implementation plan — 2026-11-04

Node 1162 grants spell 16857 in the checkout data: Cat/Bear/Dire Bear form mask
0x91, 30-yard range, six-second category recovery, 40-second duration and 175
armor reduction. Stealth/invisibility suppression is reference evidence only;
the approved Survivors policy excludes those mechanics.

## Planned Survivors consumer

Use a target-owned physical vulnerability rather than inventing enemy equipment
or armor values. Initial authored tuning: +10% physical damage taken for ten
seconds, six-second base recovery and 30-yard range. This is not a flat 175 armor
simulation. Rank upgrades can scale duration/range/recovery; display those
changes inline and retain a non-damaging description.

Learning the talent adds Faerie Fire to the starting kit only while Cat or Bear
is selected: Wrath, Rake/Swipe and Faerie Fire, exactly three slotted abilities.
Claw/Maul remain outside the slots. No benefit or cast in other forms. Preserve
the spell pool and enforce the six-slot acquisition limit. If another tree grant
would make a fourth starter, resolve the interaction before enabling it.

It is an independent moving single-target cast. Extra-target stats may affect
its target count; area stats scale range. Require visible targets and clear
line of sight, and prefer targets lacking the debuff to avoid repeatedly marking
one enemy while others remain untouched. Bosses can receive it.

## Integration checks before counting node 1162

- Introduce a core expiry/refresh rule and an enemy-owned runtime component.
  Reapplication refreshes without stacking; malformed/stale applications fail
  closed. Expiry follows simulation time, so pause cannot consume it.
- Apply only to physical direct hits and physical periodic damage, including
  automatic attacks and authored pet bites. Share the physical-source classifier
  with Hemorrhage; do not infer school from UI color. Test absent-catalog behavior.
- Preserve Hemorrhage charges, curses, Roar and independent vulnerability effects;
  define multiplication order explicitly and cover splash/triggered paths once.
- Verify form and talent gating, complete starter combinations, refunds, spec
  switching, import clamping, death cleanup and restart. Capture actual native
  purchase/refund/spec/restart and wide/compact card/talent views.

This document is a planned conversion. Node 1162 remains provisional until the
consumer and visual evidence pass; no Faerie Fire runtime behavior is claimed.
