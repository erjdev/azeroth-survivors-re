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
resolve its actual threat/targeting policy before implementing that consumer.

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
