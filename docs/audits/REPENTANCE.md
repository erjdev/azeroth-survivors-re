# Repentance implementation contract

Installed `--talent-evidence 1441` and `--spell-evidence 20066` establish the
Retribution capstone: Humanoids only, six-second incapacitation, damage wakes,
20-yard range and 60-second recovery. This is not an ordinary stun.

Survivors conversion: training unlocks card offers, without granting a spell slot.
After acquisition, automatically select the nearest non-concealed living non-boss
humanoids; no eligible target preserves readiness. Boss exclusion is an authored
policy. There is no mana cost in the no-mana game. Shared rank scaling adds 10%
base duration per rank, multiplies recovery by 0.93 per rank, and adds targets at
ranks 3/5/7. Equipment +targets applies; radius bonuses do not create areas.

Incapacitation stops movement and contact attacks, including attacks on pets.
Positive health damage wakes immediately and removes only the Repentance visual;
zero damage does not. Direct, generated, pet, periodic and poison damage must all
use the wake path. Unrelated roots, stuns and debuffs survive waking. Reapplication
preserves the longer duration; active gameplay alone advances the timer. Corpses
cannot receive the effect. Expiry and death remove the visual normally.

Acceptance: verify unlock/refund and saved specs, target eligibility and readiness,
rank scaling, effect application, every damage path, control/visual independence,
pause and expiry. Capture native wide/compact talent and card text with green
inline duration/recovery changes and units shown once. Attended combat and Windows
remain separate acceptance requirements. A definition alone is not completion.
