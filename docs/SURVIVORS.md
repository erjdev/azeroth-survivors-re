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
  Drain and life leech heal from enemy health actually removed, including periodic
  damage and multistrikes; overkill and already-dead targets grant no healing.
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
  Improved Mend Pet regenerates 1% of the living wolf’s maximum health per second
  per rank (2 ranks), using run time. It does not heal the Hunter, revive the wolf
  or alter recovery. This automatic Mend Pet conversion has no dispel.
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
  Bestial Discipline shortens bite cooldown by 10% per rank (2 ranks), multiplying
  the 20-point milestone. Frenzy haste divides that adjusted cooldown by 1.3 while active.
- Mage Cold Snap automatically resets owned Frostbolt, Frost Nova, Blizzard and
  Cone of Cold cooldowns every 90 seconds of run time, first at 90 seconds. Pauses
  do not advance its clock. Missed intervals do not bank charges; it does not grant
  spells or remove existing areas. This is the Survivors automatic conversion.
- Rogue Preparation automatically resets owned Sinister Strike, Eviscerate, Blade
  Flurry, Rupture, Gouge, Kidney Shot and learned Sprint cooldowns every 180
  seconds of active run time, first at 180 seconds. It grants no spells or stored
  charges, leaves existing effects intact and does not reset Cold Blood or
  Adrenaline Rush recovery. This is the Survivors automatic conversion.
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
  is Normal until a second difficulty exists. Run Options shows the selected zone,
  Normal difficulty and Normal (1×) speed, links to Settings, and returns with Done.
  Enter on this page must not start a run.

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
for scope and acceptance requirements. Provisional talent bonuses follow the same
inline comparison and single-unit formatting; their pending-effect notice stays
visible. At maximum rank, only the current bonus is shown.

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

Talent artwork keeps its 300:331 proportions. Give each tree enough width and
height for readable icons and separated rank labels; wrap columns and scroll
vertically on narrow windows instead of compressing all three trees. Keep tree
widths uniform within each row, including beside the spec/details panel. When
wrapping or scrolling puts full talent details off-screen, keep an inspection
summary visible so players can read an upgrade before spending a point.

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
amulet (rounded up from the current catalog; 59 of 117 entries at this checkpoint). Each entry counts once across the account.
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
| Improved Backstab (202) | 3 | +10 percentage points Backstab critical chance per rank; acquired attack, worn dagger and rear position required |
| Opportunity (261) | 5 | +4% Backstab damage per rank; Garrote/Ambush remain unavailable; does not grant attacks |
| Dagger Specialization (182) | 5 | +1 percentage point critical chance per rank for direct Rogue weapon attacks with a worn dagger; cosmetics do not qualify |
| Sword Specialization (242) | 5 | 1% chance per rank after an original direct Rogue weapon hit with a worn sword to repeat its direct damage on the same target; no copied control, area or bleed effects and no recursive procs |
| Fist Weapon Specialization (183) | 5 | +1 percentage point critical chance per rank for direct Rogue weapon attacks with a worn fist weapon; cosmetics do not qualify |
| Mace Specialization (184) | 5 | 1/2/3/4/6% chance to stun for 3 sec (bosses 0.9 sec), plus +1–5% direct damage with a worn mace; explicit weapon-skill conversion |
| Weapon Expertise (1703) | 2 | +3/+5% direct damage with a worn sword, dagger or fist weapon; explicit weapon-skill conversion; requires Blade Flurry |
| Adrenaline Rush (205) | 1 | First successful Rogue cast starts a 15 sec burst: casts assign 50% shorter cooldowns, multiplying other bonuses. Ready at run start; 300 sec recovery. Existing cooldowns, Sprint and Cold Blood recovery unchanged; energy-regeneration conversion |
| Endurance (204) | 2 | Sprint cooldown reduced by 10% per rank: 15 / 13.5 / 12 sec; speed, duration and level-five unlock unchanged |
| Improved Slice and Dice (277) | 3 | Blade Flurry: 5% shorter cooldown per rank; automatic Slice and Dice conversion |
| Elusiveness (247) | 2 | Gouge and Kidney Shot: 10% shorter cooldown per rank; defensive cooldown conversion, no stealth required |
| Vigor (382) | 1 | Rogue spells: 10% shorter cooldown; energy-capacity conversion |
| Preparation (284) | 1 | Reset owned Rogue ability and learned Sprint cooldowns every 180 sec of active run time; preserves active effects and excludes Cold Blood / Adrenaline Rush recovery |
| Cold Blood (280) | 1 | Next successful Backstab, Sinister Strike, Eviscerate, Gouge or Kidney Shot cast: +100 percentage points critical chance; ready at run start, 180 sec recovery after use |
| Seal Fate (283) | 5 | Critical original Backstab/Sinister Strike/Gouge hits: 20% chance per rank to advance owned Eviscerate, Rupture and Kidney Shot cooldowns by 1 sec; combo-point conversion. No splash, echo, generated-strike or queued-overkill procs |
| Remorseless Attacks (272) | 2 | After a kill, +20 percentage points critical chance on the next successful Rogue melee cast; 20 sec window |
| Lightning Reflexes (186) | 5 | +1 percentage point damage reduction; dodge conversion |
| Deflection (187) | 5 | +3 armor; parry conversion |
| Camouflage (244) | 5 | +3% movement speed, always active without stealth |
| Sleight of Hand (1700) | 2 | +2 percentage points damage reduction; threat conversion |
| Improved Gouge (203) | 3 | +25% Gouge stun duration |
| Lethality (269) | 5 | +6 percentage points critical damage for Backstab, Sinister Strike and Gouge |
| Aggression (1122) | 3 | +2% Backstab, Sinister Strike and Eviscerate damage |
| Improved Eviscerate (276) | 3 | +5% Eviscerate damage |
| Malice (270) | 5 | +1 percentage point critical chance for Rogue spells |
| Improved Sinister Strike (201) | 2 | 5% shorter Sinister Strike cooldown; energy-cost conversion |
| Serrated Blades (1123) | 3 | +10% Rupture damage; no armor penetration |
| Deadliness (1702) | 5 | +2% Rogue spell damage; attack-power conversion |
| Precision (181) | 5 | +1% Rogue direct and periodic damage; hit-chance conversion, no miss roll |

Weapon training applies to Sinister Strike, Eviscerate, Blade Flurry, Gouge and
Kidney Shot. Backstab also benefits from dagger training and Weapon Expertise.
Rupture ticks and neutral item abilities are excluded. Equipment
identity comes from the worn weapon, never its independent appearance.

Backstab is an acquired single-target melee attack (initial tuning: 30 damage,
1.3 sec recovery, 5.5 yd nominal reach). Offers and casts require a worn dagger;
a cosmetic dagger cannot qualify. Every target, including additional targets,
must face away from the hero. A failed positional attempt spends neither attack
recovery nor critical charges. Sinister Strike remains the starter attack.

Rogue spell membership is Backstab, Sinister Strike, Eviscerate, Blade Flurry, Rupture,
Gouge and Kidney Shot. Other-class and neutral jewelry abilities are excluded.
Remorseless Attacks applies to Backstab, Sinister Strike, Eviscerate, Gouge and Kidney Shot.
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

### Mage Arcane Power

Arcane Power (node 87, one rank) automatically starts on the next successful Mage
cast: +30% direct and periodic damage for 15 active seconds, with 180-second
recovery measured from activation. It is ready at run start; unsuccessful targeting
and foreign abilities cannot activate it. Later casts do not extend the window.
Each cast retains its damage multiplier for projectiles, attached burns and ground
areas; expiry does not weaken an existing effect. Pauses stop both timers, missed
windows do not accumulate charges, and new runs reset the clock. The HUD shows
remaining active seconds. This Survivors conversion has no mana surcharge and
stacks multiplicatively with existing damage talents and milestones. Cold Snap
resets ability cooldowns only, not this recovery clock.

### Paladin Divine Favor

Divine Favor (node 1433, one rank) automatically guarantees direct critical hits
for the next successful owned Holy Shock, including additional single-target hits.
It starts ready and recovers 120 seconds after use in active run time; missed
opportunities never accumulate charges. Other spells and periodic damage neither
benefit nor consume it. Holy Shock's cast healing is unchanged. The real DBC row
and Illumination prerequisite still apply; this talent does not grant Holy Shock.
The separate ten-point Holy milestone continues to add its ordinary critical chance.

Blessing of Sanctuary (1431, one rank) prevents 2 damage from every incoming
hit after armor and percentage reduction, before temporary shields. Damage cannot
fall below zero. This is an always-active self-only Survivors conversion; it
grants neither block chance nor retaliation damage.

### Mage Combustion (initial)

Combustion (36, one rank) starts ready with ten percentage points of additional
Fire critical chance. Each landed damaging direct Fire hit adds another ten
points, capped at 100%; the third direct critical hit ends the burst and starts
180 seconds of recovery in active run time. Fireball, Fire Blast, Pyroblast,
Blast Wave and Flamestrike impacts qualify. Area targets count separately;
periodic burns, foreign spells, invalid targets and multistrike echoes do not.
Direct casts snapshot critical chance when fired; ground impacts read it when
pulsing. Existing in-flight attacks keep their rolled result. The HUD shows
active chance and critical-hit progress. This automatic Survivors conversion
replaces the individual node’s provisional damage, independently of tree milestones.

### Scoped caster accuracy conversions

Shadow Focus (Priest 463, five ranks) adds 2% direct and periodic damage per
rank to Mind Blast and Shadow Word: Pain. Suppression (Warlock 1005, five ranks)
adds 2% per rank to Corruption, Curse of Agony and Death Coil. Nature’s Guidance
(Shaman 583, three ranks) adds 1% per rank to Lightning Bolt, Chain Lightning
and the three shocks. These explicitly labeled Survivors hit-chance conversions
replace provisional global damage; they do not introduce misses or affect other
spells, cooldowns, range, targets or critical chance. Refunds and inactive specs
remove their bonuses.

### Improved Aspect of the Hawk

Hunter Beast Mastery node 1382 has five ranks. Each successful shot cast has a
10% chance to grant 3% shot haste per rank for 12 seconds of gameplay time.
It refreshes without stacking. Subsequent shot casts snapshot their cooldown
as `cooldown / (1 + haste)`; the triggering cast is unchanged. It affects Arcane
Shot, Multi-Shot, Serpent Sting, Concussive Shot, Aimed Shot and Scorpid Sting.
Volley, traps, pets, periodic ticks and other-class spells neither trigger nor
benefit. Pauses stop its timer; failed casts do not roll. A HUD status shows the
active percentage and remaining time. This is an automatic Survivors aspect
conversion with no separate aspect toggle or auto-shot requirement.

### Hunter Trap Mastery

Trap Mastery (Survival node 1322, two ranks) adds 5% Explosive Trap direct and
periodic damage per rank, multiplying Clever Traps. This is the Survivors
trap-resistance conversion: there is no separate resistance roll. It does not
change radius, cooldown, target count, duration or Entrapment roots, and does
not affect shots, Volley or pet bites. Inactive specs and refunds remove it.

### Priest talent tuning (initial)

Inner Focus (one rank, node 348) adds 25 percentage points critical chance to the
next successful Smite, Holy Fire, Holy Nova or Mind Blast, for all direct targets.
It starts ready, recovers 180 seconds after use in active run time, and never banks
charges. Failed targeting and paused frames preserve it. Damage-over-time ticks,
shields, fear and foreign-class spells neither consume nor benefit. This is an
automatic Survivors conversion; there are no mana costs. The bonus adds to normal
critical chance, capped at 100%, rather than guaranteeing a critical hit.


Implemented individual Priest talents include these replacements for provisional general damage:

Inspiration (361, three ranks) grants 8% / 16% / 24% more armor for 15 seconds
when Holy Nova restores health. It multiplies armor after flat contributions,
refreshes without stacking, and ends on expiry or death. Full-health casts,
regeneration and other healing cannot trigger it. The player must acquire Holy
Nova and unlock its healing through training or the Holy milestone; Inspiration
does not grant either. This scaled Survivors conversion requires effective healing
rather than a healing critical.

Blessed Recovery (1636, three ranks) restores 7% / 14% / 21% of health damage
taken over six seconds after any nonlethal hit. Each new hit adds its healing to
the unpaid amount and restarts the six seconds; already delivered healing is
never repeated. Shields and immunity do not trigger it. Healing caps at maximum
health, expires even while full, pauses with the run and cannot revive a dead hero.
This explicit Survivors conversion uses all health-damaging hits until incoming
critical hits are modeled.

Improved Fade (483, two ranks) reduces the learned level-five Fade cooldown by
10% per rank: 18 / 16.2 / 14.4 seconds. Its four-second escape and 40% movement
speed bonus are unchanged. This is a scaled Survivors cooldown conversion.

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


### Elemental Mastery individual talent

Shaman DBC node 573 grants an automatic critical charge for the next successful
Lightning Bolt, Chain Lightning, Earth Shock, Flame Shock or Frost Shock cast.
It adds 100 percentage points critical chance to every direct target, including
Chain Lightning's delayed bounces. The charge starts ready and recovers 180 seconds
after use in run time. Empty target selections and paused frames cannot consume it;
missed time does not bank charges. Totems, Lightning Shield, periodic ticks and
other-class spells are excluded. It grants no mana benefit because Survivors has
no mana resource. Existing Elemental milestones remain independent.


### Hunter Trueshot Aura

The individual one-rank Marksmanship node (1361) passively multiplies the Hunter's
Arcane Shot, Multi-Shot, Serpent Sting, Concussive Shot and Aimed Shot direct and
periodic damage by 1.10. This is a self-only Survivors ranged-attack-power conversion;
Volley, traps, pets and other-class abilities are unaffected. It multiplies other
trained damage bonuses and the distinct 30-point milestone (Multi-Shot targets and
Aimed Shot damage). It changes neither targeting nor timing, grants no new spell,
and disappears when the node is refunded or another spec is selected.


### Priest Power Infusion

The individual Discipline node (322) automatically starts a 15-second window
on a successful damaging Priest cast, with 180 seconds of active-run recovery
from activation. It is ready at run start; failed targeting and pauses preserve
readiness. During the window, Priest direct/periodic cast damage and unlocked
Holy Nova healing multiply by 1.20. Cast damage retains the multiplier through
impact and attached periodic ticks; subsequent casts do not refresh the window.
Owned Renew healing checks the window dynamically, multiplying Improved Renew
and Spiritual Healing. It does not unlock Renew or Holy Nova healing, amplify
shields, other regeneration or other-class spells, or add a healing multiplier
to lifesteal/Vampiric Embrace (higher damage can still increase health drained).
This is a self-only automatic Survivors conversion, separate from Inner Focus
and the 30-point Discipline milestone. HUD status shows the active countdown.


### Priest Mind Flay

Shadow node 501 unlocks Mind Flay in run offers; training alone does not occupy an
ability slot. The acquired spell channels three non-critical ticks, one per second,
with no immediate hit. Base tick damage is 25, range is 20 yards and recovery is
four seconds. Ability ranks add 30% of base damage per rank; ordinary single-target
rank/+target rules apply. Recovery cannot be shorter than the three-second channel.

Each cast fixes its victims. Selection skips dead, concealed, out-of-range or
solid-world-obstructed enemies before filling target slots. No valid target uses
the ordinary 200ms retry without spending spell recovery. A connected victim is
slowed by 50%; the strongest active snare wins without overwriting other timers.
Lost victims never rejoin that cast. Death, loss of range/visibility or solid-world
obstruction cancels their unpaid ticks. Movement and other automatic abilities
remain available; the channel cannot overlap itself.

Shadow Focus and Power Infusion are included in the cast's tick-damage snapshot;
Infusion expiry does not weaken remaining ticks. Shadow Reach extends selection
and connection range. Shadow Weaving, Blackout, multistrike and Vampiric Embrace
settle on paid ticks; healing is capped to health actually removed. Pauses freeze
active run time and do not consume ticks. A late update emits each owed tick once.
Historical target-modifier timing across long frames is not yet implemented in
the shared hit pipeline; this remains an acceptance gap, not a different rule.
Native channel beams, pose during concurrent casts and Windows combat acceptance
are tracked in [the Mind Flay audit](audits/MIND-FLAY.md).


### Heart of the Wild individual talent

Druid node 808 grants 4% maximum health per rank, capped at 20%. This is an
explicit Survivors stamina conversion without a form requirement. It multiplies
the final health total after equipment, passive, renown and racial contributions;
recalculating stats does not heal current health. New runs start at full health.
The individual node is separate from the existing 20-point Feral milestone.
Its former placeholder damage is removed; refunds and spec changes remove the
health multiplier for subsequent runs without changing an active run's snapshot.


### Mage Mana Shield (initial)

Mana Shield is an acquirable Mage ability: rank one absorbs 40 damage for 10 seconds
with an 18-second cooldown. Ordinary ability upgrades scale absorption, duration
and cooldown. It uses the shared non-stacking shield pool. Automatic casts wait
while a stronger shield is active, or an equal shield has at least as much duration remaining;
unsuccessful refreshes retry after 0.2 seconds without spending the full cooldown.
A useful refresh replaces the absorb without adding the two amounts;
this mode has no mana resource or damage-school restriction. Its legendary amulet
and discovery entry follow the existing per-spell Codex rules. Appending it to the
Mage pool preserves existing Codex spell positions.

Improved Mana Shield (node 83, two ranks) reduces only its cooldown by 10% per rank,
using an explicitly labeled Survivors mana-cost conversion. It does not grant the
spell or increase absorption/duration. Arcane Instability, Arcane Power and offensive
school talents exclude Mana Shield. Initial values require live balance validation.


### Hunter Deterrence

The one-rank Survival talent (1308) automatically reduces incoming damage by 50%
for the first 10 seconds of a run and for 10 seconds every 180 seconds thereafter.
This is an explicit Survivors dodge/parry conversion, without random avoidance
rolls. It multiplies armor and other reduction before temporary shields absorb
the hit. The combat wolf and pickup companions receive no protection from it.
The HUD shows the remaining active duration. Gameplay time controls the windows,
so pauses and selection screens do not consume them; missed intervals do not
accumulate charges. An untrained or inactive spec grants no protection.


### Hunter Scorpid Sting

Scorpid Sting (3043) is a normal Hunter ability offer with its own legendary amulet
and Codex entry. Rank 1 hits one nearby enemy within 26 yd for 4 damage, reducing
that enemy's outgoing damage by 25% for 6 seconds, with a 4-second cooldown.
This is a Survivors strength/agility-reduction conversion. Normal ability ranks
scale hit damage, debuff duration and cooldown; additional targets apply because
it is a single-target attack. Shot talents, Efficiency and Hawk Eye include it.

Improved Scorpid Sting (1351, three ranks) increases only its debuff duration by
20% per rank, multiplying ability-rank scaling. This explicitly replaces the
original stamina-reduction effect; it does not increase damage reduction strength
or grant the spell. Bosses retain the full damage-reduction duration. A weaker
sting cannot replace or extend a stronger active reduction; equal-strength
refreshes retain the longer duration. Neither debuff stacks multiplicatively.


### Hunter melee and Savage Strikes

Raptor Strike (2973) and Mongoose Bite (1495) are normal Hunter offers with their
own Codex/legendary amulet entries. Rank 1 uses 4 yd targeting range: Raptor Strike
has 30 damage and a 3-second cooldown; Mongoose Bite has 22 damage and a 2.5-second
cooldown. Normal ranks scale damage/cooldown. These are draft Survivors values;
Mongoose Bite is an automatic cooldown attack, without a dodge prerequisite.
Single-target additions apply; shot, trap and pet-specific training does not.

Savage Strikes (1621, two ranks) grants these two abilities 10 percentage points
of critical chance per rank. It grants neither ability and does not increase
generic critical chance. Saved specs and refunds retain the normal talent rules.


### Hunter Wing Clip

Wing Clip (2974) is a normal Hunter melee offer, with an associated legendary
amulet/Codex entry. Rank 1 deals 5 damage within 4 yd, slows by 50% for 5 seconds,
and recharges in 4 seconds. Normal ranks scale damage, slow duration and cooldown;
additional targets remain within melee reach. Bosses retain 75% movement speed.
These are draft Survivors balance values. Shot bonuses and Savage Strikes exclude it.

Improved Wing Clip (1305, five ranks) grants a 4% root chance per rank on each
direct damaging Wing Clip hit. Roots last 5 seconds, or 1.5 seconds on bosses,
and preserve longer existing roots. The baseline slow remains. Roots do not count
as frost freezes; periodic ticks cannot trigger the talent. This talent grants
neither the ability nor extra damage, range or slow strength.


### Druid Insect Swarm talent

Insect Swarm (827, one rank) makes applications of the already-owned Insect Swarm
ability reduce enemy outgoing damage by 2% for a fixed 10 seconds, including
bosses. This is a Survivors hit-chance conversion; the mode has no miss roll.
Ability-rank damage/duration, cooldown and targeting are unchanged. Periodic
damage ticks do not refresh weakening. Stronger reductions retain their own
expiry; equal-strength applications retain the longer duration. This replaces
the provisional damage bonus and does not grant the ability.

### Rogue weapon coatings

Instant Poison (8679) and Deadly Poison (2823) are acquired abilities occupying
one of the six run slots. A run can select one coating, then upgrade its rank.
Coatings require a worn one-handed dagger, sword, mace or fist weapon; cosmetic
appearances do not qualify. The coating lasts for the run, explicitly replacing
reference consumable charges and a 30-minute duration. No off-hand slot is added.

An original damaging Backstab, Sinister Strike, Eviscerate, Blade Flurry, Gouge or
Kidney Shot hit rolls once while its victim survives. Additional actual targets
roll separately; splash, multistrike echoes, generated attacks, poison damage
and queued overkill never add applications. Coatings neither autocast nor crit,
and do not consume Cold Blood or Remorseless charges or activate Seal Fate.

Instant Poison has a 20% base application chance and 22 base Nature damage.
Deadly Poison has a 30% chance and deals 9 base Nature damage per stack every
3 seconds for 12 seconds, including its final tick. Each application adds one
stack, up to five, and refreshes duration without resetting the next tick.
The latest application snapshots per-stack damage for all stacks. New applications
start after existing clocks advance for the frame. Pause freezes ticking;
long frames settle due ticks until lethal damage, with one kill/loot settlement.
These damage values are initial Survivors tuning. Both poisons have separate meter
entries, use damage/leech bonuses, and benefit from active Kidney Shot exposure.
Rank upgrades improve damage; target, area, haste and cooldown bonuses do not
create extra poison rolls or change Deadly's timing. Precision, Deadliness,
Malice and Vigor affect Rogue attacks, excluding coatings.

Improved Poisons (268) adds 2/4/6/8/10 percentage points of application chance.
Vile Poisons (682) adds 4/8/12/16/20% poison damage. Neither talent grants a
coating. Enemy poison dispels are not implemented; Vile Poisons' reference
resistance-to-dispel component has no effect and is explicitly labeled pending.

### Creature-specific damage training

Murder (274) increases the Rogue's damage by 1/2% against Beasts, Dragonkin,
Giants, Humanoids and Critters. Critters follow Spell.dbc's mask 211; the adapted
description names them explicitly despite their omission from the reference
prose. Unknown creatures receive no category bonus.

Monster Slaying (1623) increases Hunter damage against Beasts, Dragonkin and
Giants by 1/2/3%; Humanoid Slaying (1301) does the same against Humanoids.
Acquired attack criticals multiply their final damage by the same factor again:
rank three gives a normal-hit factor of 1.03 and a critical-hit factor of 1.0609.
This follows the reference spell-critical path, rather than its separate
autoattack formula. The two target groups do not overlap.

These are owner bonuses, including acquired neutral abilities and damage over
time; the Hunter pet does not inherit them. Each victim is evaluated at damage
settlement. Splash, generated strikes and periodic snapshots retain their
unmodified input so the victim's bonus is applied once. Instant Poison and
Deadly Poison ticks use the same target rule without gaining critical hits.
Creature identity is authored independently of display, size and boss status.


### Incoming contact outcomes

Hero contact attacks retain the attacker and distinguish landed, immune, inactive
and invalid outcomes. A landed attack records shield absorption separately from
health removed. Full absorption is not a dodge or parry. Lethal damage removes
only remaining health; later attacks cannot trigger additional reflection from a
defeated hero. Non-finite and nonpositive damage is rejected without changing
health, shields or recovery buffs. Immunity retains the existing Thorns reflection
behavior. Defensive avoidance and Hunter aspects are still pending.


### Hunter Aspect of the Monkey

Aspect of the Monkey (13163) is an acquired, run-long Hunter ability. Rank one
adds an 8% chance to dodge each enemy contact attack. Each of the eight ability
ranks adds 2 percentage points after the first (22% at rank eight). This is an
authored Survivors rank curve; the base 8 points come from the installed spell.
Only one aspect can be acquired per run, and stale acquisition cards cannot stack it.

Improved Aspect of the Monkey (1381, five ranks) adds 1 percentage point per rank
while the aspect is acquired. It does not grant the ability. Its own legendary
amulet multiplies the combined dodge chance by 1.05; other damage, haste, critical,
area and target stats do not change it. The general chance ceiling is 95%.

A successful dodge happens before armor, shield consumption and damage-taken
recovery. It retains attacker identity, uses the dodge animation outcome, and
cannot reflect Thorns or activate Enrage/Blood Craze/Blessed Recovery. Invalid
hits, inactive heroes and immunity consume no dodge roll. Pets receive no aspect
protection. Current enemy contacts are eligible regardless of facing; ranged and
spell attack categories, parry and other Hunter aspects remain future work.


### Marked-target damage foundation (2026-10-28)

Hunter's Mark remains unavailable while acquisition and tracking/reveal are pending.
The target-state foundation stores one caster and attack-power snapshot, replacing
rather than adding repeated applications. Its timer uses active run time. Expiry,
caster loss, hero death and target death remove the gameplay benefit.

For Survivors' normalized Aimed Shot and Multi-Shot, marked-target attack power
contributes 0.2 damage per point before the cast's rank, damage and critical factors.
Those factors are captured when firing; the victim's mark is read when the hit
lands. Splash reads its own victim and applies its 70% factor to both base damage
and the mark contribution. Arcane Shot, stings, traps, pets and melee gain none of
this contribution. This is the explicit Survivors damage rule, not a claim to
reproduce every vanilla weapon/attack-power calculation. The base-spell evidence
and remaining acceptance are in [the Mark audit](audits/HUNTERS-MARK.md).


### Acquired Hunter's Mark

Hunter's Mark is an offered utility ability, not granted by training. Its authored
Survivors power curve is 20/30/45/60/75/90/110/130 ranged attack power over eight
ability ranks. Improved Hunter's Mark adds 3% of this power per talent rank, up
to 15%; only the mark's own legendary amulet then multiplies it. Global damage,
critical chance/damage and area bonuses do not increase mark power.

A base 1.5-second cycle selects the enemies with the most current health within
100 yards, using the shared concealment/reveal predicate. Haste and cooldown
reduction shorten this cycle. It marks one target plus eligible +target bonuses
(capped by the normal sheet rules); ability ranks do not add targets. Each
successful cycle replaces this caster's previous selection and refreshes the
selected marks to 120 seconds without stacking. A cycle with no candidate uses
the normal short target-search retry and leaves existing marks to expire.

Aimed Shot and Multi-Shot gain 0.2 damage per marked-target power point before
their cast damage/critical modifiers, sampled per victim at impact. Mark itself
deals no damage. Concealed victims are revealed to the caster and their wolf;
their minimap dot gets a gold diamond. The 1130 world state kit follows the
same gameplay mark lifetime, taking priority over the last ordinary visual
debuff slot if all eight slots are occupied. This affects visual slots only.
Mark duration uses active run time, pauses freeze it, and caster/target death
removes the benefit. These automatic-targeting and eight-rank rules are Survivors
conversions, not claims about vanilla spell automation or rank progression.
