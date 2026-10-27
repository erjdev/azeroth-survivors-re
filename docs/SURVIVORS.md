# Survivors — the Armory

The pictures in `docs/intentions/` are the long-term game. The first draft proved the rules
and stored them. It did not become that game. A player still lands on the glue character
screen with a text panel over it. This file is the spec for what replaces that, and the
order to build it in. The pictures win when this file and the code disagree.

The in-run game stays: waves, level-up cards, chests, gold, the stat sheet, renown, and the
current enemy counts. The Armory is the account, opened before a run and closed with Done.

The implementation phases, architecture, validation requirements and acceptance criteria are
in [the implementation contract](SURVIVORS-IMPLEMENTATION.md).

## What the draft already keeps

These stay, and later phases read them instead of inventing a second copy.

- One stat sheet. Damage reduction after armor, capped. Cooldown reduction stacks with haste
  by multiplication and has a floor. Luck is one rating. Extra targets apply only to a
  single-target bolt, strike, or melee. Gold and experience are percent bonuses. A positive
  gold drop never rounds to zero. Multistrike and leech behave as the sheet says.
- Named heroes. Talent points live on the hero. Gear, zone clears, and seen legendary amulets
  live on the class. The save is `benilla-config/survivors-account.txt`.
- A run still spends a level-up hand, then any chest trades. Chests are not a magnet.
  Companions walk out, pick up coins and experience wisps, and rest. They do not open chests.
- Hunters also summon one separate combat wolf. Its melee bite deals 8 base damage
  at run level 1, +2 per additional level, multiplied by global damage, every 2 seconds.
  It follows the Hunter, seeks enemies within 18 yards of the Hunter, and uses body
  sweeps and local routes. Extra-target bonuses add nearby melee victims without increasing reach.
  It cannot bite through solid scenery or across floors.
  The wolf has 60 health plus 6 per additional run level. Non-boss enemies can
  attack a closer living wolf within 4 yards and solid-world line of sight; bosses
  keep targeting the hero. Defeat disables the wolf for a base 10 seconds of run time,
  then it returns beside the hero when that position has safe floor support.
  The HUD reports wolf health or recovery time. Endurance Training adds 3% wolf
  maximum health per rank (5 ranks). Level changes preserve its health percentage.
  It does not occupy a collectible companion slot. Further pet talents and balance
  acceptance remain unfinished.
  Beast Mastery milestones: 10 points grants +20% bite damage; 20 points shortens
  the bite cooldown by 20%; 30 points grants a further ×1.5 bite damage during
  seconds 0–10 of each 30-second run-time cycle. These automatic adaptations affect
  neither Hunter shots nor pickup companions.
  Individual training: Unleashed Fury adds 4% wolf damage per rank (5 ranks),
  Ferocity adds 2 percentage points wolf critical chance per rank (5 ranks),
  Frenzy grants a 20% chance per rank on wolf criticals for 30% bite haste for
  8 seconds (5 ranks; refreshes without stacking), and Bestial Swiftness grants
  30% wolf movement speed (1 rank). Bites use global critical chance/damage;
  Frenzy haste divides the milestone-adjusted bite cooldown by 1.3 while active.
- Achievement points buy perks and companion ranks. Two perk resets refund perk points and
  leave companion ranks. The first companion is free.
- Cloak quality follows feats: talent cap, raid clear, a finished collection, every legendary
  amulet seen. Nine epic cloaks are the legendary cloak.
- Restart launches this build again, or a local file named in `survivors-update.txt`. A web
  address is refused. Nothing is downloaded.

`cargo survivors` (or `scripts\survivors.cmd` from Cursor's terminal) is the native loop.
`--release` rebuilds the client crate from scratch and is not the daily build.

## The screen

One Armory frame, gold on black, using the client's own glue art and icons. Eight tabs, in
this order, plus Done:

Character, Collection, Drop Tables, Codex, Talents, Achievements, Rewards.

Rewards shows the unspent achievement points on the tab. The same frame is the front door,
before a run, and the Armory opened from that door. The glue race and class screen is where
a hero's body is chosen. It is not the menu.

The front door is three columns.

- Class: the nine classes, the selected class's fantasy in one line, health, the spell it
  starts with, how many zones are open, and its spell icons.
- Hero: name, race, sex, a random-name button, the worn equipment as icons, the summed
  bonuses in one block, and two doors, Armory and Talents. Under that, the race's permanent
  bonus.
- Zone: every campaign zone in order. A locked zone says why. The selected zone shows its
  title and its stakes. Enter World names that zone. Run Options sits beside it. Difficulty
  is Normal until a second difficulty exists.

Reference: `docs/intentions/survivors_main_menu.png`.

Human racial rules from the main-menu reference:
- Perception: +15% damage for 10 seconds every 30 seconds of gameplay. The first
  activation is at 30 seconds; windows are [30, 40), [60, 70), and so on. Paused
  gameplay does not advance this timer. It multiplies damage after permanent
  bonuses, including spell-specific legendary damage.
- The Human Spirit: +10 percentage points to run experience gain, additive with
  other experience bonuses before run tuning. This affects pickups, not account
  talent experience awarded at run end.
- Diplomacy: +20 percentage points to gold gain, additive with other gold bonuses.
  Existing per-award nearest-coin rounding still applies; a single coin can remain
  one coin. Companion gold multipliers apply afterward.

The user authorized lore-appropriate authored rules for every other race. Initial
values are tuning candidates; these are Survivors adaptations, not vanilla racial
spell behavior. The selected body's race is captured when a run starts.

| Race | Bonuses |
|---|---|
| Orc | Blood Fury: +20% multiplicative damage for 8 sec every 30 sec; Hardiness: 5 percentage points damage reduction; Warrior Spirit: +5 percentage points damage |
| Dwarf | Stoneform: 15 percentage points damage reduction for 8 sec every 30 sec; Mountain Resilience: 3 permanent points reduction; Treasure Sense: +10 percentage points gold |
| Night Elf | Quickness: +5% movement speed; Shadowmeld: 10 percentage points damage reduction for 6 sec every 30 sec; Elune's Grace: 0.3 health/sec regeneration |
| Undead | Unending Hunger: 0.5 health/sec regeneration; Will of the Forsaken: 5 percentage points damage reduction; Shadow Affinity: +0.10 critical damage multiplier |
| Tauren | Endurance: +10% maximum health after flat bonuses; Thick Hide: +5 armor; Earthmother's Reach: +15% pickup radius |
| Gnome | Expansive Mind: +10 percentage points experience; Nimble: +5% movement speed; Ingenious Design: +10 percentage points area-radius bonus |
| Troll | Berserking: +15 percentage points haste for 10 sec every 30 sec; Regeneration: 0.5 health/sec; Hunter's Instinct: +5 percentage points damage |

All timed racials first activate at 30 seconds of gameplay and use half-open
windows like Perception; pause stops their timer. Reduction adds to other reduction
bonuses under the existing combat cap. Regeneration follows the normal health cap.
These adaptations do not grant stealth, control immunity, creature-type bonuses,
or extra summoned units. Unknown race IDs grant no racial bonus.

The reference composition must scale proportionally across reasonable resolutions and aspect
ratios, with readable minimum sizes and deliberate reflow/scrolling. Fixed-pixel placement at
one resolution is not completion. Card sheen stays inside its card. Every interface surface
needs an actual checkout screenshot in the README gallery. The detailed acceptance matrix is
in [the implementation contract](SURVIVORS-IMPLEMENTATION.md#responsive-interface-and-bounded-effects).

Upgrade cards show changes inline in their description: `Deal 20 » 43 damage to all enemies
within 5 » 7yd of the player.` Both values, the arrow and their units/stat wording are green;
surrounding prose keeps its normal color. This replaces a separate change summary below the
description. See [upgrade readability](SURVIVORS-IMPLEMENTATION.md#upgrade-readability-checkpoint)
for scope and acceptance requirements.

World readability includes localized see-through scenery around the hero, independently
selectable red enemy and green player/friendly silhouettes behind objects, and a separate
“Auto-push camera in” setting for solid cave/instance boundaries. Solid tree trunks must block
movement; route finding around obstacles is separate from collision. See the
[camera and collision requirements](SURVIVORS-IMPLEMENTATION.md#camera-visibility-silhouettes-and-solid-world-push-in).

## Zones

The path on the front door is the path in the world. Twelve zones, in this order. Talent
gates still climb along the path. A zone opens when the previous one is cleared on that
class and the hero's talent level meets the gate. Elwynn needs neither. Maps that are not
on this list stay open.

| Zone | Band |
| --- | --- |
| Elwynn Forest | Common |
| Duskwood | Common |
| Warsong Gulch | Uncommon |
| Un'Goro Crater | Uncommon |
| Arathi Basin | Rare |
| Feralas | Rare |
| Gurubashi Arena | Epic |
| Scarlet Monastery | Epic |
| Alterac Valley | PvP |
| Blackrock Mountain | PvP |
| Gates of Ahn'Qiraj | Legendary |
| Naxxramas | Legendary |

Westfall and Western Plaguelands were the draft's path. They are not this path. Stranglethorn
in the draft was Gurubashi Arena under another name. Gurubashi stays. The elites and bosses
already named on a draft zone move to whichever row of this list uses that map. New rows
need their own map, elites, boss, and drop table before they can be entered.

## Gear

A class collects pieces. It wears one piece in each slot. Slots are helm, shoulders, chest,
waist, legs, feet, wrist, hands, and main hand. That is nine. The collection is those nine
slots across six qualities: common, uncommon, rare, epic, PvP, and legendary. 9 × 6 = 54.
The front door's "26 of 54" is that count.

A piece has a real item name, an icon, a quality, the zone and the creature that drop it,
and a stat line. Higher quality keeps the lower lines and adds the next, in larger numbers.
The worn set's lines add together and show under the paper doll. A full worn set of nine
adds the set bonus on top. A two-handed weapon fills the main hand only. It does not invent
an off-hand piece. The draft's tenth slot and its poor-through-legendary ladder were a
stand-in for this grid.

Quality progression increases the existing stat lines and adds another line at each step.
The user's examples are illustrative tuning: poor gloves may start with health; common adds
pickup range; uncommon adds area; rare adds damage reduction; epic adds damage. Each step
also strengthens the earlier lines. A raid epic is a separate progression tier beyond an
ordinary epic, with an additional stat and raid set bonuses that support specific builds.
The catalog must represent those tiers explicitly; the existing six-band grid is a foundation,
not a reason to collapse raid epics into ordinary epics. Final coefficients are tuning data.

Poor gear has an initial 2% aggregate drop chance from ordinary enemies in Elwynn
Forest and Duskwood; one of the nine slots is chosen uniformly. Scheduled item
sources, bosses and elites keep their own loot rules. First-time Poor drops fill
empty equipment slots only; occupied slots and independent looks are preserved.
Duplicates never add stats or re-equip an unequipped item. Like other permanent
equipment changes, collected power is applied when starting the next run.
Poor ownership counts toward general piece-count and nine-slot Full set achievements;
the Common–Legendary completion feat still requires its 54 cells.
This is tuning, not a vanilla drop-table claim.

The cloak's collection feat requires every catalog cell for one class (currently
54 pieces), not merely one item in each of the nine slots. The nine-slot Full set
achievement and worn-set combat bonus remain separate. Previously earned achievement
points stay earned after rule corrections; cloak power follows the current feats.

Looks are separate from stats. A collected piece can be worn as gear, shown on the model
while another piece is worn, or hidden. Only the worn piece adds bonuses.

Reference: `docs/intentions/survivors_character_tab.png`,
`docs/intentions/survivors_collection_tab.png`.

## Drop tables

Each zone lists the creatures that drop its pieces, the slot, the chance, and how often that
creature comes. A duplicate roll adds nothing: no extra copy and no stat increase. The table is the source of truth for what a run can award.
Clearing a zone is not, by itself, the grant.

Elwynn, as the picture has it: Hogger at 3:00 drops the helm, Mother Fang at 6:00 the
shoulders, Rhahk'Zor at 9:00 the chest, Mr. Smite at 12:00 the waist, Edwin VanCleef at
16:00 the main hand. Other zones follow the same shape: earlier creatures, then the last
boss. The names have to exist on that map before the row is real.

Reference: `docs/intentions/survivors_drop_tables_tab.png`.

## Codex

Every ring, trinket, and amulet a chest can offer, as icons. Legendary amulets are per
class, one for each of that class's spells, and they stay seen once dealt. Rings, trinkets,
and the shared amulets are offered to every class and last one run. The draft's catalogs
(9 rings, 13 trinkets, 17 shared amulets, one legendary amulet per ability) are the floor.
The picture is the ceiling: a grid large enough that a finished account reads well above
that floor, with unseen icons still drawn. Click an icon to read it.

Luck still biases a chest card toward jewelry, and a boss chest more so with Keen Eye.
Jewelry is never written into the class collection.

Divine Shield's legendary amulet grants 15 percentage points of cooldown reduction
to Divine Shield alone, using the existing cooldown cap and minimum timer. It does
not change immunity duration or grant damage, critical damage, or extra targets.
Repeated copies do not stack; replacing the legendary amulet removes this benefit.

Reference: `docs/intentions/survivors_codex_tab.png`.

## Talents

Three trees per class, vanilla names, 51 points across them. A row opens after 5 points
already spent in that tree. Milestones at 10, 20, and 30 points in one tree change how
spells play, and they are reachable, so a tree has enough nodes to hold 30. Ranks are the
vanilla mix of 5, 3, and 1, not six identical nodes of 5. Click spends, right-click takes
one back, Reset is free.

Three specs are stored on the hero. The points in the picture (0/11/40, 0/38/13, and an
empty third) are three saved spreads, not one array with a view on top. The run uses the
spec that is selected.

Paladin milestones, as named on the picture: Holy — Divine Favor, Daybreak, Light's Grace.
Protection — Hallowed Ground, Ardent Defender, Hammer of the Righteous. Retribution — Seal
of Command, The Art of War, Divine Storm. Other classes use their own vanilla trees and
name the 10, 20, and 30 the same way, as a real talent, not as "{tree} 10".

Talent experience is unchanged: the run pays it from time, kills, bosses, and a win, more
in a later zone. 100 experience is one point. Veteran's Lessons still raises that for a
class behind the account's best.

Reference: `docs/intentions/survivors_talent_tab.png`.

## Achievements

New grants record their first-earned UTC calendar date from the system clock.
Summary and category cards show that date; legacy achievements without recorded
dates remain labeled Earned. Date metadata never grants an achievement or points.
Recent ordering follows grant order, even if the system clock changes.

Account-wide. The summary shows points earned against the whole catalog, recently earned,
and five bars: Zones, Classes, Collection, Feats, Lifetime. Each category has its own page.
A row shows its reward in points and, where it is a count, how far along it is.

Jeweler’s Eye grants 10 points for discovering half the combined jewelry Codex,
rounded up: shared rings, trinkets and amulets plus every class-specific legendary
amulet (currently 56 of 112 entries). Each entry counts once across the account.
Existing awards from the earlier eight-legendary rule remain earned. Legendary
discoveries separately grants 15 points for 36 class legendary amulets.

Legendary grants 25 points for all nine legendary gear pieces of one class.
It tracks the best single-class collection; duplicate pieces and lower rarities do
not advance it. Each achievement ID contributes points and earned count only once,
including when a saved account contains repeated IDs.

The draft's catalog is the start (zone clears, a win on each class, talent cap, pieces,
the set, legendary amulets, cloaks, runs, wins). It grows when a zone, a piece, or a feat
from this file exists to be earned. Point totals on the pictures (645 of 1520, 40 of 73)
describe a finished catalog, not a number to hard-code first.

Reference: `docs/intentions/survivors_achievements_tab_summary.png`,
`docs/intentions/survivors_achievement_tab_zones.png`.

## Rewards

Perks and companions are bought with the same points. The pages match the pictures.

Perks: Second Thoughts, Skip, Veteran's Lessons, Banish, Opening Hand, Keen Eye. Ranks show
as pips. The effect text on the picture is the effect in the run. Where the draft used a
milder number (Veteran's Lessons at 20% a rank, Keen Eye as a jewelry chance), the picture's
wording wins: a class behind the account's best earns 40% more talent experience, and a
boss is 10% more likely to drop each piece.

Companions: one free companion, Long Leash, Quick Paws, Gold-sniffer, Wisp-herder,
Rune-hound, and a second companion with its own instinct. Fetch distance, grasp, rest, and
the "not a magnet" rule are unchanged. Looks are the critters on the picture, worn from
this page. Display ids still have to be ids the client actually ships. Snowshoe Rabbit
stays the default until another id is checked. Tabards are the third rewards page. They
are account cosmetics. No tabard grants a stat.

Reference: `docs/intentions/survivors_rewards_tab_perks.png`,
`docs/intentions/survivors_rewards_tab_companions.png`.

## Order of work

The draft built every rule one inch deep. The next work builds one surface to the picture,
then the data that surface is lying about.

1. **Front door.** The three columns, the tabs, Done, and Enter World, drawn with glue art.
   Tabs that do not have their phase yet open the draft page for that topic, inside this
   frame, not a second menu. Enter World uses the zone the door selected. A locked zone
   does not start.
2. **Zone path.** Replace the seven-zone list with the twelve above. A zone with no map and
   no boss is shown and locked, not enterable. Gates stay "previous clear, then talent level".
3. **Drop tables and the collection grid.** Pieces come from named creatures. The collection
   becomes nine slots by six qualities. The worn set and the set bonus read that grid.
4. **Talent trees and three specs.** Replace the six-node skeleton. Specs save separately.
5. **Codex icons, achievement categories, companion looks, tabards.** Each is a page on the
   frame from step 1, filled when its catalog is real.

Character appearance (race portraits, sex, skin, face, hair) stays the glue screen reached
from the hero column. A paper doll in the Armory comes after the worn set is real, because
a doll with nothing to show is the draft again.

## Out of scope until this file says otherwise

Shrinking the horde. Removing the level-up draft, renown, or the relic stash before gear
and talents replace what the player spends them on. An unverified download-and-run updater. The prompted, authenticated update design is in
[development and distribution](SURVIVORS-DEVELOPMENT.md). Inventing
item icons, creature display ids, or map files that are not in the 1.12.1 client.

### Battleground encounter format

Warsong Gulch, Arathi Basin and Alterac Valley keep horde survival with authored
faction-themed encounters and bosses. They do not require a separate PvP objective
mode. Use map-appropriate faction identities and verify spawn routes, collision,
boss pacing and reward sources before unlocking these zones.

### Rogue individual talent checkpoint

The following audited nodes replace the temporary +1% damage per spent rank;
they do not stack with it. Existing tree bonuses and row/prerequisite gates remain.

| Talent (DBC ID) | Ranks | Survivors effect per rank |
|---|---|---|
| Improved Slice and Dice (277) | 3 | Blade Flurry: 5% shorter cooldown per rank; automatic Slice and Dice conversion |
| Elusiveness (247) | 2 | Gouge and Kidney Shot: 10% shorter cooldown per rank; defensive cooldown conversion, no stealth required |
| Vigor (382) | 1 | Rogue spells: 10% shorter cooldown; energy-capacity conversion |
| Remorseless Attacks (272) | 2 | After a kill, +20 percentage points critical chance on the next successful Rogue melee cast; 20 sec window |
| Lightning Reflexes (186) | 5 | +1 percentage point damage reduction; dodge conversion |
| Deflection (187) | 5 | +3 armor; parry conversion |
| Camouflage (244) | 5 | +3% movement speed, always active without stealth |
| Sleight of Hand (1700) | 2 | +2 percentage points damage reduction; threat conversion |
| Improved Gouge (203) | 3 | +25% Gouge stun duration |
| Lethality (269) | 5 | +6 percentage points critical damage for Sinister Strike and Gouge |
| Aggression (1122) | 3 | +2% Sinister Strike and Eviscerate damage |
| Improved Eviscerate (276) | 3 | +5% Eviscerate damage |
| Malice (270) | 5 | +1 percentage point critical chance for Rogue spells |
| Improved Sinister Strike (201) | 2 | 5% shorter Sinister Strike cooldown; energy-cost conversion |
| Serrated Blades (1123) | 3 | +10% Rupture damage; no armor penetration |
| Deadliness (1702) | 5 | +2% Rogue spell damage; attack-power conversion |

Rogue spell membership is Sinister Strike, Eviscerate, Blade Flurry, Rupture,
Gouge and Kidney Shot. Other-class and neutral jewelry abilities are excluded.
Remorseless Attacks applies to Sinister Strike, Eviscerate, Gouge and Kidney Shot.
Any kill by the run refreshes its 20-second gameplay-time window without stacking.
It is consumed once by a successful eligible cast, including all targets of that
cast. Failed casts, Rupture and area spells do not consume it. It expires while
unspent, does not advance while paused, and cannot trigger or apply while dead.
Critical chance is capped at 100%; this proc adds no permanent sheet bonus.

Damage factors from different talents multiply. Talents improve acquired abilities;
they do not grant the spell. Other individual Rogue effects
remain unfinished and retain their explicitly labeled provisional behavior.

### Rogue tree milestones

These authored Survivors effects supplement the existing tree-stat bonuses.
They require an acquired ability, stay within Rogue spells, and stack with individual
talents. Cooldown multipliers multiply; critical chance is capped at 100%.

| Tree | Points | Milestone | Effect |
|---|---|---|---|
| Assassination | 10 | Malice | +5 percentage points critical chance for Rogue spells; periodic ticks do not crit |
| Assassination | 20 | Lethality | +20 percentage points critical damage for Rogue direct attacks |
| Assassination | 30 | Cold Blood | Eviscerate always critically strikes; automatic Survivors conversion |
| Combat | 10 | Improved Sinister Strike | Sinister Strike cooldown ×0.90 |
| Combat | 20 | Blade Flurry | Blade Flurry radius ×1.30, still one player-centered area |
| Combat | 30 | Adrenaline Rush | All Rogue spell cooldowns ×0.85; automatic Survivors conversion |
| Subtlety | 10 | Master of Deception | Successful Gouge grants a non-stacking 3% maximum-health absorb for 3 seconds |
| Subtlety | 20 | Preparation | Successful Kidney Shot advances Gouge cooldown by 2 seconds |
| Subtlety | 30 | Premeditation | Successful Gouge advances Eviscerate cooldown by 1 second |

Subtlety's authored conversions trigger once per successful cast, not per target.
Cooldown advances only affect acquired spells and stop at zero; they cannot
trigger while dead. A smaller absorb does not replace or extend a stronger one.

Milestones are independent of the
similarly named individual talent nodes and reset when the selected spec changes.

### Priest talent tuning (initial)

Nineteen individual Priest talents now replace their provisional general damage:

| Talent | Per-rank Survivors effect |
|---|---|
| Vampiric Embrace (1 rank) | Always-active self-healing for 20% of health removed by Priest Shadow damage, including Pain ticks; excludes overkill |
| Improved Vampiric Embrace (2 ranks) | +5 percentage points of healing per rank (25% / 30% total); requires the base talent |
| Shadow Weaving (5 ranks) | +20 percentage points chance on Priest Shadow damage to add 3% Shadow vulnerability for 15 seconds; five stacks maximum, refreshed after damage |
| Blackout (5 ranks) | +2 percentage points chance on Priest Shadow damage, including Pain ticks, to stun for 3 seconds (bosses: 0.9 seconds); refreshes without stacking |
| Improved Power Word: Fortitude (2 ranks) | +10 maximum health, always active; no maintained buff required |
| Improved Inner Fire (3 ranks) | +2 flat armor, always active; applied before percentage reduction |
| Spell Warding (5 ranks) | 2% reduced incoming damage; general reduction until incoming damage schools are modeled |
| Force of Will (5 ranks) | +1% Priest Holy/Shadow damage and +1 percentage point direct critical chance; periodic damage cannot crit, shields/foreign-class/jewelry spells excluded |
| Holy Reach (2 ranks) | +10% Smite/Holy Fire casting range and Holy Nova radius; Nova remains one area |
| Shadow Reach (3 ranks) | +6⅔% Shadow Word: Pain/Mind Blast casting range, totaling 20%; Psychic Scream unchanged |
| Spirit Tap (5 ranks) | +20 percentage points chance on kill to regenerate 1% maximum health/sec for 15 sec; refreshes without stacking |
| Improved Power Word: Shield (3 ranks) | +5% shield absorption; same duration |
| Holy Specialization (5) | +1% critical chance for Smite, Holy Fire and Holy Nova |
| Divine Fury (5) | 4% shorter Smite/Holy Fire cooldown; automatic-cast conversion |
| Searing Light (2) | +5% Smite/Holy Fire damage, including Holy Fire's burn |
| Improved Shadow Word: Pain (2) | +15% duration; unchanged tick damage |
| Improved Mind Blast (5) | 5% shorter Mind Blast cooldown |
| Improved Psychic Scream (2) | 10% shorter cooldown; unchanged fear duration |
| Darkness (5) | +2% Shadow Word: Pain, Mind Blast and Psychic Scream damage |

Spirit Tap converts spirit/mana recovery to health regeneration. Direct and periodic
kills can trigger it once per enemy; healing stops at maximum health and cannot revive.

These effects stay with the selected hero/spec and do not affect foreign-class or
jewelry spells. Duration/cooldown conversions use the existing Survivors spell
cadence and are initial balance values. Remaining individual Priest talents retain explicitly provisional behavior.

### Priest tree milestones (initial)

These automatic Survivors conversions supplement the existing tree-point bonuses.
They apply only to the named spells and the selected valid spec; they do not grant
unowned spells. Successful-cast rewards apply once, regardless of targets hit.

| Tree / points | Milestone | Effect |
|---|---|---|
| Discipline / 10 | Unbreakable Will | Power Word: Shield grants 1 sec immunity |
| Discipline / 20 | Mental Agility | Shield and Psychic Scream cooldowns ×0.85 |
| Discipline / 30 | Power Infusion | Shield advances Mind Blast cooldown by 2 sec |
| Holy / 10 | Holy Specialization | Smite, Holy Fire, Holy Nova gain 5 percentage points critical chance, additive with individual ranks |
| Holy / 20 | Spiritual Guidance | Holy Fire heals 3% maximum health per successful cast |
| Holy / 30 | Lightwell | Holy Nova heals 5% maximum health per successful cast; automatic healing pulse |
| Shadow / 10 | Shadow Affinity | Mind Blast gains one target; AoEs remain single areas |
| Shadow / 20 | Shadow Weaving | Mind Blast advances Shadow Word: Pain cooldown by 2 sec |
| Shadow / 30 | Shadowform | Psychic Scream grants a 5% maximum-health absorb for 3 sec; no stacking or replacement of a stronger shield |

Healing cannot exceed maximum health or revive a dead hero. Cooldown advances clamp
to zero. Lightwell is a cast-triggered pulse here, not a placed world object;
Shadowform is a defensive cast interaction, not a model transformation. Combat
balance and presentation refinement remain open.

### Elite-chest pacing

Special named-elite chests are limited to five per run. The first can drop after
60 seconds of gameplay; later drops require at least 90 seconds since the previous
one. Killing many elites together does not bypass this pacing. Boss rewards and
ordinary wooden chests use their own rules.

### Hunter pet sustain

Improved Revive Pet (two ranks) reduces the wolf’s ten-second recovery by 20%
per rank: 8 or 6 seconds, still requiring a supported spawn position. Spirit Bond
(two ranks) restores 1% of each recipient’s own maximum health per rank to the
Hunter and wolf every ten seconds while the wolf lives. Healing caps at maximum
health. Defeat resets the interval, and paused gameplay does not advance it.
These are authored Survivors adaptations; no pet resurrection cast or mana cost
is introduced.

Intimidation (one rank) empowers the next available wolf bite immediately and
then at most once every 60 seconds of run time. Only the nearest bite victim
receives a three-second stun and six seconds of forced attention toward the
living wolf, even when the hero is closer. Bosses receive the existing 30% stun
duration (0.9 seconds), but cannot be forced away from the hero. Recovery does
not reset the cooldown. Solid-world collision and route planning still apply.

The individual Bestial Wrath capstone upgrades the automatic 30-point Beast
Mastery milestone from +50% bite damage for 10 seconds to +100% for 18 seconds
of each 30-second run-time cycle. It replaces the base Wrath multiplier, while
still multiplying Unleashed Fury and trained bite damage. It provides no damage
or crowd-control immunity and does not affect Hunter spells or pickup companions.

### Initial Raid Epic armor rules

Raid Epic armor uses separate `raid-epic:<class>:<slot>` identities for eight armor
slots. These are empowered Survivors variants of the verified ordinary Epic art
and names, not imported vanilla set powers. Main hand is excluded from these sets.
Each piece has the six-stat Raid Epic budget; ordinary Epic remains five stats.
The dedicated Raid Epic Collection row tracks these eight armor slots separately.
Encounter acquisition is still pending, so these variants are not yet obtainable
during a normal run. The inspector reports equipped armor count and active versus
inactive cumulative thresholds; owning or displaying a piece does not activate it.

Bonuses activate cumulatively at 2/4/6 distinct owned, equipped armor slots of the
class set. Ownership alone, duplicates, hidden appearances and transmog cannot
add or remove set power. Replacing a raid piece with its ordinary Epic counterpart
reduces the set count even though the appearance may be identical. These bonuses
are initial Survivors tuning and affect the hero's shared stats:

| Class set | 2 pieces | 4 pieces | 6 pieces |
|---|---|---|---|
| Warrior — Wrath | +24 armor | +4 percentage points critical chance | +25 percentage points critical damage |
| Paladin — Judgement | +24 armor | +10% area | +4% damage reduction |
| Hunter — Dragonstalker | +4% movement speed | +8 percentage points multistrike | +25 percentage points critical damage |
| Rogue — Bloodfang | +4% movement speed | +4 percentage points critical chance | +25 percentage points critical damage |
| Priest — Transcendence | +30 health | +6% cooldown reduction | +4% damage reduction |
| Shaman — Ten Storms | +15% pickup range | +10% area | +8% cooldown reduction |
| Mage — Netherwind | +15% pickup range | +10% area | +8% cooldown reduction |
| Warlock — Nemesis | +15% pickup range | +2% life leech | +10% damage |
| Druid — Stormrage | +30 health | +6% cooldown reduction | +4% damage reduction |

The existing nine-slot loadout bonus remains separate. Supplemental raid ownership
does not fill any of the legacy 54 collection cells or grant their completion feat.

Raid armor counts once per distinct class/slot identity toward Armed and Hoarder,
regardless of equipment or cosmetic selection. Full set still requires ownership
in all nine slots of one class: eight raid armor pieces alone cannot earn it, but
a main hand of any supported quality from that same class completes it. A weapon
owned by another class cannot fill the gap. Raid armor does not advance Legendary,
which requires the nine legacy Legendary pieces. No separate raid-completion
point reward is currently defined; raid set power remains based on worn armor.
