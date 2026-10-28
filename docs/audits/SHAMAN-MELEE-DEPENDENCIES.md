# Shaman melee dependencies — 2026-10-28

The installed catalog still reports 31/46 Shaman definitions (296/432 overall).
Elemental Devastation cannot be represented by unconditional critical chance:
the current Shaman pool contains only lightning, shocks, totems and a shield.
There is no Shaman melee cast or automatic weapon-swing path to receive its bonus.

## Installed evidence

Read only from this checkout's `wow-data/Data`, using `survivors-audit`:

```sh
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 1645
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 30165
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 29177
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 29178
cargo run -p survivors-audit -- wow-data/Data --talent-evidence 901
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 17364
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 8680
cargo run -p survivors-audit -- wow-data/Data --spell-evidence 2818
```

- Elemental Devastation node 1645: three ranks, offensive spell critical trigger.
  Triggered buffs 30165/29177/29178 have aura 52, raw base points 2/5/8 plus
  one die-side point, and 10000 ms duration: +3/6/9 percentage points melee
  critical chance for ten seconds. It does not increase spell critical chance.
- Stormstrike node 901: Enhancement row 6, prerequisite Elemental Weapons 611
  at rank 3. Spell 17364 supplies an extra attack and two Nature-damage charges
  at +20%, lasting twelve seconds. Its range is five yards and recovery is
  20000 ms; start recovery is 1500 ms. Raw aura 87 has base points 19 plus one,
  and school mask 8. These rows do not supply a universal weapon-damage formula.

## Implemented foundation

`survivors-core/src/stormstrike.rs` provides target-owned vulnerability state:
application refreshes two charges and twelve seconds, never adds potency or
charges. Positive connected Nature-hit settlement consumes one charge and gets
1.2× damage. Invalid damage/time leaves valid charges intact; independent targets
remain independent. Active gameplay time supplies pause behavior. A despawned
victim must lose its component rather than transfer charges to another victim.

All 276 core tests and core all-target Clippy pass. The damage pipeline now
consumes target-owned charges for catalog-confirmed Nature damage: direct hits,
periodic effects, actual weapon poison procs and reflection. Poison damage uses
spells 8680/2818 rather than the physical coating spells 8679/2823. Reflection
uses the actual equipped reflection spell rather than its shared meter label.
Unknown spell data leaves charges untouched. Catch-up ticks use settlement time,
matching the existing damage pipeline; historical sub-frame timing is not reconstructed.
The successful Stormstrike attack still needs to apply this component to enemies. All 528 Survivors app tests and feature-enabled all-target Clippy pass.
No talent definition, offer, player UI or coverage count is
changed; no screenshot or combat acceptance is claimed.

## Required next implementation

1. Establish the real Shaman melee attack and its weapon-damage input. An equipped
   model or a spell tagged ranged=false is not evidence of a weapon attack.
   Document any authored Survivors conversion and its rank/gear scaling.
2. Integrate Stormstrike application after a successful attack and consumption
   at per-victim Nature-hit settlement (consumption implemented; application pending). Define school ownership for lightning,
   shocks, shields, periodic damage and totems explicitly. Authored multistrike
   echoes share the original event's charge; they do not spend extra charges.
   Exclude misses, immune/zero-damage events and non-Nature damage.
3. Add Elemental Devastation from actual offensive spell critical outcomes to
   actual melee critical rolls. Refresh one ten-second buff; exclude reflected,
   periodic and pet events unless explicitly eligible. Preserve source identity
   and cast/impact timing, rather than applying a global critical bonus.
4. Validate legal prerequisite purchase/refund, all ranks, saved specs, paused
   clocks, death cleanup, target isolation, critical/multistrike interactions,
   normal offers, green inline text and wide/compact native screenshots.

The original named-talent requirements remain open until these runtime paths work.
