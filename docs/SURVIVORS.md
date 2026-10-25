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

These are Survivors mechanics, not vanilla racial spell effects. Other races'
bonuses require authored rules and currently grant none; the starter labels them
as not yet defined. The selected body's race is captured when a run starts.

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

Account-wide. The summary shows points earned against the whole catalog, recently earned,
and five bars: Zones, Classes, Collection, Feats, Lifetime. Each category has its own page.
A row shows its reward in points and, where it is a count, how far along it is.

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
