# Earth totem prerequisites

## Checkout evidence

Read from `wow-data/Data` with `survivors-audit --spell-evidence`:

| Spell | Installed rule |
| --- | --- |
| 2484 Earthbind Totem | Summons entry 2630, 5 health, 45-second lifetime, 15-second recovery |
| 6474 Earthbind Totem Passive | Periodic trigger 3600 every 3000 ms |
| 3600 Earthbind | 10-yard radius, 50% movement slow for 5 seconds |
| 5730 Stoneclaw Totem rank one | Summons entry 3579, 50 base health plus level scaling, 15-second lifetime, category recovery 30 seconds |
| 5729 Stoneclaw Totem Effect rank one | 8-yard threat effect; raw effect 63, base points 21 plus one |

Talent evidence for Earth's Grasp 572 resolves to +25/50% Stoneclaw health
and +10/20% Earthbind radius. Implementing Earthbind alone does not complete it.
Stoneclaw's effect row does not establish an unconditional fixation duration;
the user selected forced targeting while the totem lives (2026-10-29). This is
an explicit Survivors adaptation, not an interpretation of the threat row.

Display audit: 4588 decodes `Creature/Spells/EarthElementalTotem.mdx` with
11 batches and 268 triangles. All referenced textures decode. This verifies an
earth-totem model, not the creature-entry mapping, race variants, particles or
native appearance. Those remain visual integration work; no assets are copied.

## Shared Earthbind lifecycle

`survivors-core::earthbind` implements five health, destruction, the 45-second
lifetime and three-second pulse cadence. Survivors schedules the first pulse
three seconds after deployment and excludes the exact expiry boundary; these
are explicit lifecycle choices, not inferred guarantees from the DBC interval.
The caller must replace its previous Earthbind on redeployment.

Pulses return their scheduled times so catch-up cannot make an expired slow
fresh. Empty pulses are consumed. Invalid or backwards clocks cannot duplicate
pulses or damage the totem; zero/invalid damage cannot heal it. Fatal damage
stops future pulses, and replacement creates a fresh independent state.

Four core regressions cover fine/coarse frame partitions, destruction, replacement,
invalid clocks/damage and integration with the existing source-owned slow state.

## Runtime integration

Earthbind is now a Shaman offer. It deploys at the hero's grounded feet only when
supported, clear placement is available and enemies are nearby. Recasting removes
only the previous Earthbind. The actual entity uses the decoded earth-totem
model and five health; ordinary foes may attack it when it is closer than the
hero, within four yards and visible. Existing pet attention takes priority;
bosses continue targeting the hero. These are authored local-attention rules,
not a global threat table or Stoneclaw taunt implementation.

The independent area affects every live enemy in its spherical radius, without
an extra-target cap or damage messages. It uses original pulse times, retains
stronger independent slows, and leaves the last slow to expire after a target
moves away. Bosses use the existing 25% slow cap. Fatal contact removes only
totem health, with no hero defenses, reflection or spillover damage. Destruction,
expiry and hero death remove the active component and fade the model. The system
runs under the existing Playing/loading-screen gates; run cleanup owns its entity.

Ranks increase radius and shorten recovery through the shared upgrade curve;
health, lifetime, pulse interval and slow strength stay fixed. Fire-totem-specific
training retains its documented scope. Earth's Grasp remains pending until
Stoneclaw has a real health/threat consumer; coverage remains 308/432.

Six runtime regressions cover grounded deployment/replacement, zero-damage slow
pulses and departure/expiry, actual enemy destruction without hero reflection,
rank scaling, independent slow overlap/boss reduction, and Shaman-only offers.
All six pass in the final scoped gate.

Native macOS card previews pass at 1280×720 and 640×720 / 125%, including scrolling
to the third compact card. Green radius/recovery values stay inline and units
appear once. Three screenshots under `docs/images/screenshots/2026-10-29-earthbind/`
are embedded in CHANGELOG; README includes the wide view. The preview disables
picks, so screenshots do not establish live acquisition or combat acceptance.
The incremental play build passes in 1m50s with the existing unwind/linker warning.

## Remaining acceptance

- Rendered deployed-model/particle acceptance, attended gameplay and Windows.
- Explicit pause/death/reset, blocked placement and pet/boss attention scenarios
  beyond the current tests; class offer inclusion is not a live acquisition test.
- Stoneclaw's complete threat/health behavior, then both Earth's Grasp effects.

The earlier core-only scoped gate passed with 31 optional-addon skips. This
runtime checkpoint supersedes the previous not-selectable foundation status.
The runtime eight-package formatting, Clippy and test gate also passes with
31 optional-addon skips. An initial class-offer test compile error was corrected
before the successful rerun. Native card rendering and the play build pass;
the remaining acceptance items above are not closed by those results.

## Initial native model investigation — 2026-10-29

Superseded by the settled checks below. This initial capture did not establish a
persistent rendering failure.

`WOW_SURVIVORS_MODEL_PREVIEW=4588` renders a menu-only specimen at native scale,
grounded with the same floor query used by other model specimens. At 1280×720,
the current display shows a green effect without a clearly visible solid body.
This is a failed appearance checkpoint, not proof of a correct deployed totem.
The capture is embedded in CHANGELOG and README. No combat was started.

The repeated checkout decode confirms 11 batches / 268 triangles and four
textures: DemonRune1, green_glow3, green_glow2 and FirePlaceTex01. Successful
mesh/texture decoding does not establish the intended creature-entry mapping
or final appearance. Next: resolve the intended solid-body display and inspect
its animations/materials before changing the gameplay model. Race variants,
actual deployment, particles and Windows acceptance remain open.

The generic preview accepts one to eight nonzero comma-separated display IDs;
it preserves the existing demon preview flag and is development-only. It spawns
ordinary visual specimens with no Earthbind, enemy or combat-pet components.

Validation: native incremental build passed in 1m42s; the eight-package scoped
formatting, Clippy and test gate passed with 31 optional-addon skips.

## Settled native model verification — 2026-10-29

The same display 4588 renders a solid body and green effects after loading.
A second launch with displays 4588 and 4589 renders both Earth and Fire Totems.
Screenshots `model-settled.png` and `model-pair.png` are embedded in CHANGELOG and
the pair in README. No model identity, gameplay or material workaround was made.
The initial capture is retained as investigation history, not a proven defect.
Its transient cause was not measured; do not infer a load-time guarantee from it.

Read-only evidence from the checkout:

- `m2seq`: file slot 0 = Spawn 127 (1.867s), slot 1 = Stand 0 (1.5s),
  slot 2 = Death 1 (1.2s).
- `m2batch`: body batch 5 is opaque, 227 vertices, with FirePlaceTex01 texture.
- Extended `survivors-audit --creature-visuals 4588` samples baked alpha at
  0/0.5/1 seconds. Body batch 5 is 1/1/1 for Spawn and Stand; 1/1/0 for Death.
- Live preview report: slot 1, seek ~0.376s, four descendant material components
  at alpha 1 with inherited visibility true. Root position (-95,58.88147,9548.5)
  matches the independent terrain probe's floor height 58.88147.
- Repeat launch: both models in slot 1 at ~0.391s; four descendant materials
  each at alpha 1 and visible. World-root billboard cards are outside this
  descendant-only report; the screenshot independently shows their effects.

The preview logs once after five seconds with an animated specimen present.
This provides runtime context for captures, not a blanket readiness assertion.
The native incremental build passed in 1m10s. Actual ability deployment,
destruction timing, particle fidelity, racial variants and Windows remain open.

Final diagnostic revision reports all active clips directly as
`(animation ID, file slot, seek seconds, weight)`, avoiding an unpublished engine
helper caught by the API-wall gate. The final native build passed in 43.50s;
a third launch reports `(0, 1, ~0.205, 1.0)` for both specimens. The pair capture
was refreshed from this final build. No engine API allowance was expanded.

The final eight-package scoped gate passes formatting, Clippy and tests, including
the API-wall regression, with 31 optional-addon skips.

## Stoneclaw lifecycle foundation — 2026-10-29

`survivors-core::stoneclaw` now owns a destructible 15-second lifetime in active
run time. It accepts the caller's final health, retaining maximum health for
presentation. Damage removes only the remaining totem health; overkill does not
spill into the hero. Destruction and observed expiry cannot be reversed by stale
timestamps. Recasting requires a fresh instance and replacement of the old entity.
Paused run clocks retain the lifetime. Invalid health, clocks and damage are
rejected; fractional cast times retain the expiry boundary within float precision.

Four focused core tests pass: frame partitions/pause/terminal expiry, fractional
cast times, configured health/destruction/replacement, and invalid/stale inputs.
Health fixtures include 50, 62.5 and 75, but this does **not** enable Earth's Grasp.
The caller still needs to supply the implemented level/rank/talent scaling.

The exported constants record installed rank-one health 50, lifetime 15 seconds,
radius 8 yards, category recovery 30 seconds and effect threat 22. No pulse cadence
or targeting decision is inferred from those constants. The user decision between
accumulating threat overcome by damage and forced targeting remains pending.

Stoneclaw is not a playable offer yet. Remaining work: targeting policy and pulse
cadence, enemy attention/navigation and damage integration, grounded deployment,
replacement/cleanup, rank and talent scaling, inline card text, and native visual
and attended gameplay acceptance. This checkpoint has no UI change or screenshot.

Integration map for the targeting decision: `abilities.rs` carries `HitOrigin`
and applies direct, proc and periodic damage; threat must use actual health removed
at those consumers, not pre-mitigation `Hit.damage`, and must not count a proc twice.
`enemies.rs::move_enemies` currently prioritizes forced/local pet attention, then
nearby Earthbind, then the hero; Stoneclaw needs explicit precedence and live-target
validation there, including expiry/replacement. Existing navigation must retain
its blocked-route state when the target changes. A fixed taunt is not equivalent
to spell 5729's raw threat amount; either chosen adaptation must be documented.

Stoneclaw lifecycle validation: all four focused tests and the eight-package
scoped formatting, Clippy and test gate pass (31 optional-addon skips).
No native deployment or rendered evidence is claimed for this core-only change.

## Stoneclaw forced-target runtime — 2026-10-29

The user's decision supersedes the pending threat policy above. Stoneclaw is a
Shaman offer: spell 5730, verified Earth display 4588, 50 health, 15 seconds,
30-second base recovery and eight-yard base acquisition radius. Recasting
replaces only the previous Stoneclaw. Ground support and clear placement are
required. The authored health stays fixed across ability ranks; radius and
recovery use the shared upgrade curve. Earth's Grasp remains pending.

Visible ordinary enemies entering the spherical radius are captured until this
individual deployment dies, expires or is replaced. Hero damage does not break
attention; moving outside the radius or losing sight does not release it.
Stoneclaw wins over forced/local pet attention and Earthbind attention. Bosses
are immune. Hero vanish does not freeze enemies engaged with Stoneclaw. Movement
still uses existing navigation, contact still requires line of sight, and damage
uses enemy outgoing modifiers without hero defenses/reflection or overkill spill.
The active-run clock handles pause; hero death removes the active totem.

The raw 22-threat value is not used. No global threat table, new pulse cadence,
server authority, race variant mapping or live-combat visual acceptance is claimed.

Native macOS menu-only previews verify the full description at 1280×720 and
640×720 / 125%, including End navigation to the third card. Screenshots are
embedded in CHANGELOG and README uses the wide view. The acquisition card follows
the established future-growth convention (rank one to two); upgrade cards show
the actual next rank. No live combat or Windows check was performed. The play
build passed in 1m43s with the existing linker warning. Capture timeouts recovered
by raising the same live window; the launcher was restored after closing it.

Seven runtime regressions pass: forced pursuit/retention/expiry; boss and range
exclusion; fatal damage without hero spill/reflection; grounded placement,
replacement and hero death; Shaman-only offers/fixed lifetime; pet-taunt and
vanish precedence; visibility on acquisition with retention after occlusion.

The eight-package formatting, Clippy and test gate is green, with 31 expected
optional-addon corpus skips. No new test failure remains.

## Earth's Grasp integration — 2026-10-29

Node 572 is now defined at two ranks. Checkout spell rows 16043/16130 specify
+25/50% Stoneclaw health and +10/20% Earthbind radius. The shared talent snapshot
produces 50/62.5/75 health at ranks 0/1/2; deployment passes that value into the
actual lifecycle. Earthbind's cast radius is multiplied by 1/1.1/1.2, so its
runtime pulse and upgrade description use the same radius. Ability rank, equipment
area and the talent compose through the existing cast parameters. Other totems,
Stoneclaw acquisition radius, recovery, lifetime and slow strength are unchanged.
The talent replaces the provisional 1% generic damage per spent point.

Two shared-rule tests cover capped ranks, class/scope isolation, resetting and
both colored inline values. Two runtime tests cover deployed health and fatal
contact at every rank, and actual Earthbind pulse reach at 11.5 yards. With no
other modifiers, only rank two reaches that target, with the original 50% slow.

Native macOS preview: 1280×720 and 640×720 / 125% show both green bonuses without
clipping. Purchased ranks 0→1→2, refunded to one, confirmed the saved DBC record,
then relaunched into selected-profile cards. Stoneclaw shows 62.5 health; the
Earthbind card includes both saved talent and equipment radius. All captures are
under `docs/images/screenshots/2026-10-29-earths-grasp/`; CHANGELOG embeds them and
README uses the compact refund view. Presets are under `docs/ui-previews/`.
Incremental play build: 18.71 seconds, with the existing linker warning.

This supersedes the earlier incomplete Earth's Grasp consumer status. It does
not close live combat visuals, race-specific totem appearance, Windows validation
or the remaining talent definitions. No combat session was started.

Fresh `survivors-audit --talent-coverage` matches Shaman 33/46 and total 309/432
against checkout DBC identities and rank caps. This is definition coverage only.

Final eight-package formatting, Clippy and test gate: green, 31 optional-addon
corpus skips. Both new runtime tests and the shared-rule tests pass.

## Distant captured enemies — 2026-10-30

The actual movement-system regression reproduced an early-exit bug: once the
hero moved more than 75 yards away, recycling stopped a captured enemy's
approach before Stoneclaw targeting was evaluated. On streamed terrain that
same path could relocate it near the hero. Target resolution now precedes
recycling; a living Stoneclaw prevents recycling of its ordinary captives.
Expiry, destruction and replacement still release the target normally.

The regression captures an ordinary enemy, moves the hero 100 yards away,
checks continued movement toward the totem without teleporting, then expires
the totem. In this fixture no supported distant replacement exists, so the
normal recycling path parks the released enemy until terrain is available.
This is headless runtime evidence, not attended combat or Windows acceptance.

Validation: all nine Stoneclaw runtime tests pass; incremental native build
passes in 14.75 seconds. The eight-package scoped formatting, Clippy and test
gate passes with 31 optional-addon-data skips. No UI change or new visual
acceptance is claimed.
