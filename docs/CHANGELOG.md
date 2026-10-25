# Changelog

Feature milestones, newest first. These are development checkpoints, not released versions.
Detailed changes remain in Git; unfinished work remains in the [implementation plan](SURVIVORS-IMPLEMENTATION.md).

## 2026-10-25 — Saved Talent builds: live acceptance

Verified the [Talent interface](intentions/survivors_talent_tab.png) with an isolated
Warrior/Mage account at 1280×720. Warrior specs retain distinct Arms/Fury allocations;
Mage uses its own third spec. Both heroes restore their selected build after an app
restart. Resetting Warrior spec 2 refunds its points without clearing spec 1 or Mage's
saved build. Save-file readback agrees with the interface. This is a validation
checkpoint, with no gameplay or balance changes.

![Warrior first build: three Arms points](images/screenshots/2026-10-25-talent-specs/warrior-spec1.jpg)

![Warrior second build: two Fury points](images/screenshots/2026-10-25-talent-specs/warrior-spec2.jpg)

![Mage third build: one Frost point](images/screenshots/2026-10-25-talent-specs/mage-spec3.jpg)

![Mage selected build restored after restart](images/screenshots/2026-10-25-talent-specs/mage-restarted.jpg)

![Warrior selected build restored after switching back](images/screenshots/2026-10-25-talent-specs/warrior-restored.jpg)

![Reset affects only Warrior's second spec](images/screenshots/2026-10-25-talent-specs/reset-spec2.jpg)

![First spec remains usable after resetting the second](images/screenshots/2026-10-25-talent-specs/spec1-after-reset.jpg)

All-class combat effects and deeper prerequisite/refund behavior still need their
own acceptance. A visual issue remains: Talent action messages carry into Character
when changing pages. Narrow per-spec scroll restoration has automated coverage but
was not exercised in this wide-screen session.

## 2026-10-25 — Responsive Character and saved appearances

Character now stacks its proportional hero preview, appearance controls, equipment
and item details on narrow screens. Wide screens retain the
[reference's](intentions/survivors_character_tab.png) three-pane composition.
Appearance edits and tab return preserve Character scrolling independently of
Collection and Drop Tables. Switching to a hero with a saved appearance no longer
shows the obsolete “Open Look” prompt.

![Wide Character composition](images/screenshots/2026-10-25-character-responsive/character-1280.jpg)

![Proportional Character preview at 640×720](images/screenshots/2026-10-25-character-responsive/character-640.jpg)

![Hero management at narrow width](images/screenshots/2026-10-25-character-responsive/manage-heroes-640.jpg)

![Saved appearance restored after hero switching and restart](images/screenshots/2026-10-25-character-responsive/appearance-restored-640.jpg)

![Appearance value and scroll retained after Collection tab return](images/screenshots/2026-10-25-character-responsive/appearance-tab-return-640.jpg)

![Character preview at 640×900](images/screenshots/2026-10-25-character-responsive/character-640x900.jpg)

![Appearance controls at 640×900](images/screenshots/2026-10-25-character-responsive/appearance-640x900.jpg)

Actual native captures use an isolated two-hero account. Live checks covered body
and feature edits, Warrior/Mage switching, tab return and restart persistence.
Validation: 181 Survivors app tests without default features, native incremental
build, workspace Clippy, formatting and render-pass lint pass. Final typography,
wide-column scrolling and per-hero/spec Talent restoration remain open.

## 2026-10-25 — Responsive Drop Tables

Drop Tables now wraps zone choices and stacks encounter sources, item details and
equipment at narrow widths. Item selection, equip actions and tab return retain the
page's independent scroll position. Wide encounter rows put the boss/time beside its
item, following the [reference](intentions/survivors_drop_tables_tab.png); zone choices
show quality and collection counts. Finale labels use the actual combat schedule.

![Wide encounter rows and quality counts](images/screenshots/2026-10-25-drop-tables-responsive/drops-1280.jpg)

![Wrapped zone choices at 640×720](images/screenshots/2026-10-25-drop-tables-responsive/drops-640.jpg)

![Lower source selection retains scroll and updates item details](images/screenshots/2026-10-25-drop-tables-responsive/selected-source-640.jpg)

![Equipped item and scroll survive a Collection round trip](images/screenshots/2026-10-25-drop-tables-responsive/equipped-tab-return-640.jpg)

![Unfinished zone tables remain explicit](images/screenshots/2026-10-25-drop-tables-responsive/unfinished-zone-640.jpg)

Live checks used an isolated synthetic Warrior account at 1280×720 and 640×720.
All twelve zone choices and each zone's source actions are covered in both layouts;
browsing does not select the next run's zone. Validation: 99 core tests, 181 Survivors
app tests without default features, native build, workspace Clippy, formatting and
render-pass lint pass. Missing zone content, final typography and wide-column scroll
restoration remain open; this checkpoint does not claim those are complete.

## 2026-10-25 — Distinct gear stat profiles

Gear slots now have distinct stat progressions: chest armor starts with health and
armor, boots with movement and pickup, wrists with critical chance and cooldown
reduction. Every higher current band improves retained stats and adds one; duplicate
copies still add no power. Gloves retain the requested examples. This changes existing
items' bonuses while preserving ownership and selected equipment. See the
[initial tuning and remaining raid/quality work](SURVIVORS-GEAR-STATS.md).

![Common chest: two stats](images/screenshots/2026-10-25-gear-profiles/common-chest-1280.jpg)

![Epic chest: five stats](images/screenshots/2026-10-25-gear-profiles/epic-chest-1280.jpg)

![Equipping Epic chest updates the worn item and totals](images/screenshots/2026-10-25-gear-profiles/equipped-epic-1280.jpg)

![Boots emphasize movement and pickup](images/screenshots/2026-10-25-gear-profiles/common-boots-1280.jpg)

![Wrists show critical chance and clearly labeled cooldown reduction](images/screenshots/2026-10-25-gear-profiles/common-wrist-1280.jpg)

Actual 1280×720 captures use an isolated synthetic account and retain the
[Collection reference's](intentions/survivors_collection_tab.png) green item details
and equipment totals. Typography, final balance, Poor drops, raid epics and distinct
raid set bonuses remain open. Validation: 98 core rules tests, 180 Survivors app
tests without default features (including mixed-equipment combat stats and downgrade),
native incremental build (1m32s), workspace Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Codex jewelry icons and selection

Legendary amulets now read as jewelry with an associated spell badge, following the
[Codex reference](intentions/survivors_codex_tab.png). Shared jewelry uses aligned
category/count rows. Selecting an entry highlights its cell and updates a larger
detail image without rebuilding or resetting the catalog. Legendary titles are
formatted as spell names instead of exposing internal underscores.

![Catalog with synthetic discovered and undiscovered entries](images/screenshots/2026-10-25-codex-icons/catalog-1280.jpg)

![Legendary selection and spell-specific bonuses](images/screenshots/2026-10-25-codex-icons/legendary-1280.jpg)

![Undiscovered legendary remains dim in the catalog](images/screenshots/2026-10-25-codex-icons/undiscovered-1280.jpg)

![Shared ring selection replaces the legendary badge](images/screenshots/2026-10-25-codex-icons/ring-1280.jpg)

![Lower rows remain selected and scrolled at 640×720](images/screenshots/2026-10-25-codex-icons/lower-selection-640.jpg)

Live checks used an isolated synthetic account with four discoveries, covering badge
clicks, unseen entries, shared jewelry and narrow scrolling. Legendary amulets share
one verified necklace silhouette for now; distinct artwork, catalog expansion,
Mark of the Champion's missing icon and reference typography remain open. These
captures demonstrate presentation, not completion of content or progression.
Validation: 183 Survivors app tests, native build, workspace test compilation,
Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Responsive Collection page

Collection now stacks its log, item details, equipment and missing drops on narrow
screens. Labeled tiles wrap without shrinking into overlapping headings or stretching
the final tile. Wide screens retain the [reference](intentions/survivors_collection_tab.png)
three-pane layout. Narrow scrolling survives item actions and tab round trips.

![Readable slots at 640×720](images/screenshots/2026-10-25-collection-responsive/collection-640.jpg)

![Wrapped tiles at 640×900 keep their size](images/screenshots/2026-10-25-collection-responsive/collection-640x900.jpg)

![Equipping retains the detail scroll position](images/screenshots/2026-10-25-collection-responsive/equipped-640.jpg)

![Independent appearance and scroll position survive a tab round trip](images/screenshots/2026-10-25-collection-responsive/tab-return-640.jpg)

![Wide layout and saved helm selection after restart](images/screenshots/2026-10-25-collection-responsive/collection-1280.jpg)

An isolated Warrior account verified selection, Equip, independent look selection,
scrolling through equipment, tab return and restart persistence. Empty/full catalogs
for all nine classes build every slot in both layouts. Validation: 182 Survivors app
tests, native build, workspace test compilation, Clippy, formatting and render-pass
lint pass. Reference typography, richer item stats and content acceptance remain open;
this checkpoint resolves the narrow Collection crowding recorded below.

## 2026-10-25 — Shared Armory header

Armory pages now share the reference's left-aligned title, wrapping navigation and
class-colored collection summary. Removed the duplicate Collection summary and
generic hero-selection hint. The starter retains its game title.

![Character with the shared Armory header](images/screenshots/2026-10-25-armory-header/character-1280.jpg)

![Collection without a duplicate class summary](images/screenshots/2026-10-25-armory-header/collection-1280.jpg)

![Switching classes updates the collection context](images/screenshots/2026-10-25-armory-header/codex-paladin-1280.jpg)

![Header wraps at 640×720; Collection slot labels still need reflow](images/screenshots/2026-10-25-armory-header/collection-640-open-layout-gap.jpg)

Live checks covered populated Warrior and empty Paladin collections, tab navigation,
and Done returning to the starter. The narrow capture records an **open Collection
layout defect**: crowded slot headings and compressed tiles; it is not whole-page
responsive acceptance. Typography and tab styling still need reference refinement.
Validation: 182 Survivors app tests, native build, workspace test compilation,
Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Undo individual pending Rewards

Perk and Companion cards now show pending counts and a minus control that removes
one pending upgrade while keeping the rest. Pending resets have their own Undo.
Removing a prerequisite cannot silently move an instinct to another companion or
invalidate later purchases: the draft stays intact until dependencies are undone.
Confirmed purchases retain the existing reset/refund rules.

![Three upgrades staged across two cards](images/screenshots/2026-10-25-reward-minus/three-pending-1280.jpg)

![Removing one rank returns 25 points and retains Banish](images/screenshots/2026-10-25-reward-minus/one-removed-1280.jpg)

![Per-card undo remains reachable at 640×720](images/screenshots/2026-10-25-reward-minus/pending-640.jpg)

![Companion upgrades use the same pending controls](images/screenshots/2026-10-25-reward-minus/companion-pending-1280.jpg)

![Pending resets can be undone independently](images/screenshots/2026-10-25-reward-minus/reset-pending-1280.jpg)

Isolated-account checks verified rank repricing, narrow-screen removal, confirmation
of only the retained 15-point perk, and undo of a reset and Long Leash without saving
either. Dependency and companion-destination cases have regression coverage.
Validation: 182 Survivors app tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass. Reference typography/control placement still
need refinement; these controls edit pending purchases only.

## 2026-10-25 — Tabard cosmetic gallery

Tabards now use illustrated selection tiles based on the Companion Looks reference,
with clear **Worn**, **Wear**, and **Remove tabard** states. Removed client display IDs
from the player-facing page. Selection stays immediate, account-wide and stat-free.

![Guild Tabard selected without spending points](images/screenshots/2026-10-25-tabards/worn-1280.jpg)

![Removing a tabard in the narrow layout](images/screenshots/2026-10-25-tabards/removed-640.jpg)

![The worn selection persists after restart at 640×720](images/screenshots/2026-10-25-tabards/restart-640.jpg)

An isolated account verified Wear/Remove save changes, unchanged reward balance,
narrow scrolling and restart persistence. The catalog still contains one verified
tabard; more cosmetics and character-model previews remain open. Validation: 179
Survivors app tests, native build, workspace test compilation, Clippy, formatting
and render-pass lint pass.

## 2026-10-25 — Illustrated achievement cards and progress

Rebuilt Achievements around the [summary](intentions/survivors_achievements_tab_summary.png)
and [zone](intentions/survivors_achievement_tab_zones.png) references: account points,
category navigation, recently earned cards, and ranked “Almost there” progress.
Cards explain their actual completion rules and known Companion rewards. Placeholder
feats remain impossible to earn. Each category retains its session scroll position.

![Achievement summary and closest goals](images/screenshots/2026-10-25-achievement-cards/summary-1280.jpg)

![Zone clears precede all-class champion goals](images/screenshots/2026-10-25-achievement-cards/zones-1280.jpg)

![Lower zone goals remain reachable after a Collection round trip](images/screenshots/2026-10-25-achievement-cards/zones-return-1280.jpg)

![Unachievable placeholder feats retain their planned rewards](images/screenshots/2026-10-25-achievement-cards/feats-1280.jpg)

![Navigation stacks at 640×720](images/screenshots/2026-10-25-achievement-cards/navigation-640.jpg)

![Narrow summary content remains reachable by scrolling](images/screenshots/2026-10-25-achievement-cards/summary-640.jpg)

Captures use an isolated synthetic account. Live checks covered category selection,
lower-card reachability, navigation scroll restoration and resizing without restart.
Category icons currently substitute for bespoke achievement art; the decorative crest
and typography still need refinement. Earned dates are not stored, so cards say “Earned”.
Validation: 97 core tests, 179 Survivors app tests, native build, workspace test
compilation, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Placeholder feats and locked Companion looks

Added **Deathless Hour** and **no u** under Feats, marked “Criteria pending”. As
requested, neither can currently be earned or award points. Panda Cub and Sapphiron
now appear in the [reference's Companion catalog](intentions/survivors_rewards_tab_companions.png),
locked behind those entries. Existing saves and selection indices remain compatible.

![Placeholder achievements under Feats](images/screenshots/2026-10-25-placeholder-feats/feats-1280.jpg)

![Panda Cub and Sapphiron remain locked](images/screenshots/2026-10-25-placeholder-feats/locked-looks-1280.jpg)

![All thirteen looks reflow at 640×720](images/screenshots/2026-10-25-placeholder-feats/locked-looks-640.jpg)

Live checks confirmed both locked tiles reject selection. Model/icon mappings resolve
from local data; Sapphiron's companion scale and animations still need validation
before its future unlock can be enabled. Achievement-page styling remains unfinished.
Validation: 97 core tests, 178 Survivors app tests, native build, workspace test
compilation, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Confirm and Undo reward purchases

Perk/Companion purchases and perk resets now preview together, following the
[Rewards reference](intentions/survivors_rewards_tab_companions.png). Confirm applies
the whole batch; Undo discards it. Pending choices survive tab changes, require
resolution before run entry, and are discarded on app close. Cosmetics stay immediate.

![Two perks pending, with updated ranks and remaining points](images/screenshots/2026-10-25-reward-confirm/pending-1280.jpg)

![Undo restores the original balance and ranks](images/screenshots/2026-10-25-reward-confirm/undo-1280.jpg)

![Confirmed purchases saved to the account](images/screenshots/2026-10-25-reward-confirm/confirmed-1280.jpg)

![Companion upgrades share the same pending purchase flow](images/screenshots/2026-10-25-reward-confirm/companions-pending-1280.jpg)

![Confirm and Undo remain reachable at 640×720](images/screenshots/2026-10-25-reward-confirm/pending-640.jpg)

![Restart retains confirmed perks and drops pending Companion upgrades](images/screenshots/2026-10-25-reward-confirm/restart-640.jpg)

Live synthetic-account checks verified an unchanged save while staging, Undo, the
30-point confirmed spend, and restart behavior. 177 Survivors tests, native build,
workspace test compilation, Clippy, formatting and render-pass lint pass. Per-card
minus controls and closer typography remain open.

## 2026-10-25 — Expanded Companion catalog and Veteran unlock

Added six looks from the [Companion reference](intentions/survivors_rewards_tab_companions.png):
Crimson Macaw, Great Horned Owl, Tree Frog, Sprite Darter, Zergling and Undercity Cockroach.
Existing selection indices stay stable. Corrected Black Whelpling's missing display
ID to the local pet-sized black whelpling. Icons remain generic for several critters.

![Eleven looks with Chicken locked before Veteran](images/screenshots/2026-10-25-companion-catalog/catalog-1280.jpg)

![New Crimson Macaw selected](images/screenshots/2026-10-25-companion-catalog/macaw-1280.jpg)

![Expanded catalog at 640×720](images/screenshots/2026-10-25-companion-catalog/catalog-640.jpg)

![Chicken wearable after the synthetic account reaches Veteran](images/screenshots/2026-10-25-companion-catalog/veteran-1280.jpg)

New accounts unlock Westfall Chicken with Veteran (100 runs). Existing saves retain
prior unrestricted access through a persisted migration flag; this grants no points.
Live isolated-fixture checks covered blocked selection, Macaw selection, narrow reflow
and the 100-run achievement unlock. These captures validate menus, not in-world pet
animation. Panda Cub and Sapphiron await definitions for their pictured achievement gates.

Validation: 174 Survivors tests, native build, workspace test compilation, Clippy,
formatting and render-pass lint pass.

## 2026-10-25 — Illustrated Companion appearances

Looks now uses illustrated, wrapping tiles with a gold selection border and green
“Worn” state. Added the free Companion card and restored the [reference's card order](intentions/survivors_rewards_tab_companions.png),
with carrier, leash and movement icons loaded from the local patch chain.

![Base companion and upgrade cards](images/screenshots/2026-10-25-companion-looks/companion-1280.jpg)

![First companion wearing Black Whelpling](images/screenshots/2026-10-25-companion-looks/first-whelpling-1280.jpg)

![Second companion independently wearing Squirrel](images/screenshots/2026-10-25-companion-looks/second-squirrel-1280.jpg)

![Saved looks after restart at 640×720](images/screenshots/2026-10-25-companion-looks/looks-640.jpg)

Live isolated-account checks: first-slot Whelpling, second-slot Squirrel then Kingsnake;
instincts and points remained unchanged, and both looks survived restart. Checked
1280×720 and 640×720 scrolling/reflow. Native build, workspace Clippy, formatting and
render-pass lint pass. The five existing looks are illustrated; expanding to the reference's
achievement-gated catalog, denser spacing and Confirm/Undo purchases remain unfinished.

## 2026-10-25 — Companion cards and upgrade previews

Companion upgrades now use illustrated two-column cards with rank pips, costs and
green inline comparisons derived from gameplay values. The [Companion intention](intentions/survivors_rewards_tab_companions.png)
remains the reference. Appearance selection stays separate from instinct assignment.

![Companion upgrade cards at 1280×720](images/screenshots/2026-10-25-companion-cards/companions-1280.jpg)

![Quick Paws purchase updates its rank and both effects](images/screenshots/2026-10-25-companion-cards/purchased-1280.jpg)

![Instincts assigned to first and second companions](images/screenshots/2026-10-25-companion-cards/instincts-1280.jpg)

![Narrow cards and appearance controls at 640×720](images/screenshots/2026-10-25-companion-cards/companions-640.jpg)

Checked live with an isolated collected account: Quick Paws charged 15 points,
second companion 40, and each instinct 30; Gold-sniffer filled the first slot and
Wisp-herder the second. Narrow scrolling reaches every card and the Looks controls.
Native build, workspace Clippy, formatting and render-pass lint pass. Captures are
Retina window images. Remaining design work: base-companion card, closer icon choices,
illustrated/expanded Looks catalog and staged purchases. Rune spawning remains unavailable.

## 2026-10-25 — Rewards scroll continuity

Rewards preserves its session scroll position when purchases rebuild the page and
when returning from another Armory tab. Perks, Companions and Tabards keep separate
positions; a category change cannot misattribute the previous page's offset.

![Scrolled Perks before purchase at 640×720](images/screenshots/2026-10-25-reward-scroll/before-purchase-640.jpg)

![Purchase updates the card without jumping to the top](images/screenshots/2026-10-25-reward-scroll/after-purchase-640.jpg)

![Returning from Collection restores the same position](images/screenshots/2026-10-25-reward-scroll/returned-640.jpg)

Live validation used an isolated collected Warrior at 640×720. Second Thoughts
charged 15 points and updated in place; a Collection round trip retained the view.
The layout remains based on the [Rewards intention](intentions/survivors_rewards_tab_perks.png).
Cross-category bookkeeping has an ECS regression test; live Companion/Tabard scroll
checks remain open. Positions reset when the application exits and are not save data.
All 172 Survivors app tests, workspace Clippy, formatting, render-pass lint and the
native build pass.

## 2026-10-25 — Inline Perk upgrade previews

All six Perk cards now show the current and next effect inline in green, with the
unit once: for example, “20 » 40% more talent experience” and “3 » 4 cards.” Capped
perks show only the current benefit. Skip explains its unlock; Keen Eye explicitly
uses percentage points. Preview values share the core reward mechanics. Reset-count
wording is also corrected. The [Rewards intention](intentions/survivors_rewards_tab_perks.png)
remains the layout reference; inline comparisons follow the requested presentation.

![Green inline Perk previews](images/screenshots/2026-10-25-reward-previews/perks-1280.jpg)

![Veteran's Lessons after the first purchase](images/screenshots/2026-10-25-reward-previews/rank-one-1280.jpg)

![Maximum rank and narrow preview wrapping](images/screenshots/2026-10-25-reward-previews/capped-640.jpg)

Live isolated-account purchases charged 20 then 40 points, updated the preview and
removed the arrow at the cap. Checked 1280×720 and a live resize to 640×720 (Retina
captures including title bar). All 96 core tests, workspace Clippy, formatting,
render-pass lint and native build pass. Companion previews, staged purchases and
Rewards scroll continuity remain unfinished.

## 2026-10-25 — Rewards sidebar and illustrated Perks

Rewards now uses the [intention's sidebar and two-column Perk cards](intentions/survivors_rewards_tab_perks.png):
account currency, achievement totals, category navigation and perk reset sit together;
cards show icons, earned rank pips, costs, effects and max-rank state. At narrower
widths the sidebar stacks above the cards in one scrollable page. Purchases still
apply immediately and are labelled as such; Confirm/Undo staging remains unfinished.

![Rewards and Perk cards at 1280×720](images/screenshots/2026-10-25-rewards-cards/perks-1280.jpg)

Live isolated-account check: Second Thoughts charged 15 points (275→260), filled
one pip and offered rank two for 25 points. Reset refunded all 15 points and consumed
one reset. Existing purchase/save logic is unchanged.

![Purchased rank and updated currency](images/screenshots/2026-10-25-rewards-cards/purchased-1280.jpg)

![Perks after reset](images/screenshots/2026-10-25-rewards-cards/reset-1280.jpg)

![Stacked Rewards layout after live resize to 640×720](images/screenshots/2026-10-25-rewards-cards/perks-640.jpg)

Native build, workspace Clippy, formatting and render-pass lint pass. Captures use
the synthetic collected Warrior account and include the Retina title bar. Companion
and Tabard content now share the frame but still need their illustrated layouts.
Green current→next effect previews, staged purchasing, scroll continuity across
purchases, the singular reset label and typography remain UI acceptance work.

## 2026-10-25 — Taller Talent presentation

Talent panels now follow the available page height, giving the illustrated trees
and seven talent rows more space in line with the
[Talent reference](intentions/survivors_talent_tab.png). Narrow windows retain wrapping
and one page-level scroll area. Tree totals now say “1 point” when appropriate.

![Full-height Talent panels at 1280×720](images/screenshots/2026-10-25-talent-height/talents-1280.jpg)

![One spent point and green preview](images/screenshots/2026-10-25-talent-height/one-point-1280.jpg)

![Wrapped lower panels at 640×720](images/screenshots/2026-10-25-talent-height/wrapped-640.jpg)

Verified with an isolated collected Warrior fixture: spending a point, live resizing,
and scrolling to the lower tree/spec panel. Native build, workspace Clippy, formatting
and render-pass lint pass. Header/milestone icons, typography and the black margins
around the tree artwork still differ from the reference; other-class visual checks
remain open. Captures include the Retina title bar.

## 2026-10-25 — Responsive Talents live validation

Validated the current native UI with an isolated collected Warrior account, against
[the Talent reference](intentions/survivors_talent_tab.png), without a gameplay run.
At 800×720, details wrap beneath three trees. Spending Deflection changes 51→50
available points; resetting restores 51 without losing scroll. Leaving for Collection
and returning to Talents also restores that position.

![Wrapped Talent page at 800×720](images/screenshots/2026-10-25-talent-responsive/trees-800.jpg)

![Spent point and green preview in the scrolled detail panel](images/screenshots/2026-10-25-talent-responsive/spent-scrolled-800.jpg)

![Scroll position after resetting and returning from Collection](images/screenshots/2026-10-25-talent-responsive/tab-restored-800.jpg)

Live preview resizing to 640×720 produces two columns. The lower Protection tree
and spec controls remain reachable. Left-click spending and right-click refunding
Shield Specialization update rank/totals and retain the scrolled view.

![Two-column Talent layout at 640×720](images/screenshots/2026-10-25-talent-responsive/trees-640.jpg)

![Spending in the wrapped Protection tree](images/screenshots/2026-10-25-talent-responsive/protection-spent-640.jpg)

![Refunding in place](images/screenshots/2026-10-25-talent-responsive/protection-refunded-640.jpg)

Screenshots are Retina captures including the title bar. No runtime code changed.
This closes the sampled reflow/scroll interaction checks, not all-class acceptance:
per-spec/per-hero restoration still needs live coverage. Reference header/milestone
icons, full-height tree art and typography remain unfinished; spending one point
also exposes the remaining “1 points” label.

## 2026-10-25 — Starter UI visual checkpoint

Captured build `4dcf72e` at 1280×720 logical resolution (Retina, title bar included),
using the isolated collected fixture: ten heroes, all gear, unspent talents and rewards.
Opened Character → Manage heroes → Talents → Rewards/Companions without a run.
These are current captures; older images below remain attached to their original milestones.

Compared with the [Character reference](intentions/survivors_character_tab.png),
the three-column layout and larger model are present; feature pips, ornamentation
and typography still need work. All ten heroes appear in management.

![Current Character page](images/screenshots/2026-10-25-starter-review/character-720.jpg)

![Ten-hero management view](images/screenshots/2026-10-25-starter-review/manage-heroes-720.jpg)

Talents displays all three trees and the spec panel at this size. Narrow wrapping,
scroll restoration and spend/refund interactions still need live checks.

![Current Talent page](images/screenshots/2026-10-25-starter-review/talents-720.jpg)

Companion destination, costs and effects are visible. The
[Rewards reference](intentions/survivors_rewards_tab_companions.png) remains the target:
illustrated cards, rank pips, appearance icons and a currency/sidebar layout are missing.
Purchases were not exercised in this capture pass.

![Current Companion page](images/screenshots/2026-10-25-starter-review/companions-720.jpg)

## 2026-10-25 — Companion purchase clarity

Rewards now names the companion receiving an instinct, any replaced instinct,
the cost and the effect. Current instincts and locked companion state are visible.
Initial purchases retain the reference's first-then-second assignment; appearance
selection stays separate. Invalid instincts cannot charge points, and locked slots
reject appearance edits. All 171 Survivors tests, Clippy and native build pass.
Initial rendered capture appears in the visual checkpoint above; purchase interactions remain unverified.

## 2026-10-25 — Talent scroll continuity

Talent pages restore their scroll position after rebuilds, with separate session
positions for each hero and spec. Restored offsets wait for layout before clamping
to content bounds. All 170 Survivors tests, workspace Clippy, formatting,
render-pass lint and the native build pass. Live scrolling, spending/refunding,
spec switching and resized-layout verification remain pending.

## 2026-10-25 — Talent panel reflow

Talent trees and the spec/detail panel now wrap at narrower widths instead of
shrinking indefinitely into one row. One page-level scroll area keeps wrapped
panels reachable without nested wheel scrolling. Workspace Clippy, formatting,
render-pass lint and the native build pass (21 seconds). Rendered wrapping,
interaction and screenshot checks remain pending.

## 2026-10-25 — Isolated UI account fixtures

Added fresh, partial and fully collected synthetic accounts for repeatable UI
checks without gameplay. A small script copies each fixture into a new isolated
state folder and prints its launch command; existing folders are never replaced.
The collected fixture includes ten heroes for roster checks. All 168 Survivors
tests, two script isolation tests and workspace Clippy pass. Rendered comparisons,
spent talent specs and purchased reward fixtures remain pending.

## 2026-10-25 — Character preview and hero management

Character now gives the dressed hero a larger central preview with a centered name
and class. Roster, creation, renown and relic controls move into Manage heroes;
selecting or creating a hero returns to the preview. Removed the roster's eight-hero
display limit. 167 Survivors tests, workspace Clippy, formatting, render-pass lint
and the native build pass. Rendered layout and interaction checks remain pending
because the Mac was locked; no new screenshot is claimed.

## 2026-10-25 — Live menu preview controls

Native development builds can now switch production Armory pages and resize the
window from an opt-in local file, without restarting or playing a run. Invalid
edits retain the current view; account progress and name drafts are preserved.
165 Survivors app tests, workspace Clippy and the native build pass. Startup logs
confirm the control was applied; visual switching/resizing remains unverified
because the Mac was locked. [Usage](SURVIVORS-DEVELOPMENT.md#starter-ui-iteration-priority).

## 2026-10-25 — Interface milestone and deferred physics scope

Recorded the current PR's interface/setup acceptance boundary: characters, talents,
gear, progression and achievements, with persistence, responsive pages and fast UI
previews. Box3D integration is a separate phase after this PR is complete and merged.
Documented the existing Avian backend and the separate future server architecture
decision; no physics dependencies or runtime behavior changed.

## 2026-10-25 — Conditional cutaway and starter-page previews

Cutaway activation now checks the camera-to-player sightline and fades reveal
strength over 500 ms of real time. Buildings behind or beside a visible player
stay opaque. Added direct development entry into starter/Armory pages; Collection
was verified without a run and captured in README. 95 core, 165 app and one camera
regression test pass, with Clippy and native build. The reported building scene's
visual fade check remains open. Starter UI takes priority; unlockable game-speed
options are recorded for later.

## 2026-10-25 — Initial enemy obstacle detours

Stalled enemies now plan and follow collision-checked local detours, sharing a
bounded search budget across frames. Routes reset when targets move substantially
or enemies leave play. All 164 app tests pass, including real-physics wall-detour
playback; Clippy and native build pass (78 seconds). Live foliage/indoor routing,
narrow passages and crowded-queue performance remain open.

## 2026-10-25 — Safe distant-enemy placement

Repositioned enemies now respect arena bounds, supported ground and body clearance,
just like new spawns. Missing or obstructed geometry defers repositioning. All 160
Survivors app tests, Clippy and native build pass. Routing around obstacles remains
in progress; this fixes a placement bypass, not pathfinding.

## 2026-10-25 — Blackrock floor investigation

Rejected seven unsupported interior arena candidates. The floor audit now separates
holes from other storeys and flags nearby unsupported samples, including gaps hidden
by a fully supported boundary ring. [Evidence and next layout requirements](SURVIVORS-LOCATIONS.md#blackrock-interior-floor-rejection--2026-10-25)
are recorded. Eight audit tests and Clippy pass; Blackrock remains pending a safe,
connected arena layout.

## 2026-10-25 — Alterac faction-survival candidate

Added ten wave entries and nine bosses ending with Drek'Thar, with PvP rewards
for every class and slot. [Gameplay and card captures](images/screenshots/2026-10-25-alterac-roster/survival-720.jpg)
are linked in README. 94 core and 160 app tests, Clippy and native build pass
(23 seconds). The data audit has three remaining gaps; Alterac stays locked pending
full encounters, traversal, attachments and camera validation.

![Gameplay and card captures](images/screenshots/2026-10-25-alterac-roster/survival-720.jpg)

## 2026-10-25 — Arathi faction-survival candidate

Added a surveyed stables-area arena, ten faction wave entries and nine authored
bosses with Rare rewards across all slots. Duplicate rewards shared with Feralas
preserve existing gear. [Live 720p capture](images/screenshots/2026-10-25-arathi-roster/survival-720.jpg)
is linked in README. 93 core and 159 app tests, Clippy and native build pass
(22 seconds). Arathi stays locked pending traversal, camera and full encounters;
the content audit now reports 85 remaining gaps.

![Live 720p capture](images/screenshots/2026-10-25-arathi-roster/survival-720.jpg)

## 2026-10-25 — Warsong faction-survival candidate

Added Warsong's map, ten wave entries, nine authored bosses and Uncommon drops for
all slots. Cross-zone duplicates preserve gear and appearances. [Live 720p capture](images/screenshots/2026-10-25-warsong-roster/survival-720.jpg)
is linked in README. 92 core and 158 app tests, Clippy and native build pass
(22 seconds). The content audit is down to 86 gaps; Warsong stays locked pending
full traversal, camera and encounter validation.

![Live 720p capture](images/screenshots/2026-10-25-warsong-roster/survival-720.jpg)

## 2026-10-25 — Mage school training and battleground direction

Arcane Instability now improves Mage spell damage and critical chance; Elemental
Precision improves Fire/Frost damage. Both replace generic stat placeholders,
with scoped effects and accurate previews. Battleground zones are confirmed as
horde survival with faction-themed encounters. Remaining Mage talents and
battleground content are still in progress. 91 core and 157 app tests, DBC audit,
Clippy and native build pass (14 seconds); live validation remains open.

## 2026-10-25 — Long upgrade-card layout validation

Added an opt-in card inspection mode that holds level-up and chest choices.
Frost Nova's full defensive description fits at 720p and 1024×768, with single-unit
green comparisons; selecting after resize grants rank two. [Screenshots](images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)
are in the README. Clippy and native build pass (42 seconds); live defensive timing remains open.

![Screenshots](images/screenshots/2026-10-25-mage-talents/frost-nova-card-720.jpg)

## 2026-10-25 — Mage Frost defenses and milestones

Frost milestones now add Frostbolt targeting, Frost critical damage and a Nova
barrier. Ice Block and Ice Barrier talents add brief Nova immunity and an absorb,
with no stacking or replacement of stronger defenses. Cards and tooltips explain
the benefits. 90 core and 156 app tests, DBC audit, Clippy and native build pass
(15 seconds). Live timing and new card-layout evidence remain open.

## 2026-10-25 — Menu performance evidence

Separated screenshot-associated Metal presentation stalls from quiet menu timing.
Saved a [controlled frame-journal comparison](validation/mage-menu-presentation.md).
Performance runs will exclude concurrent UI capture; no runtime fix or combat
performance clearance is claimed.

## 2026-10-25 — Winter's Chill

Mage Frost hits can now build a per-enemy critical-chance debuff with five-stack
cap, refresh and expiry. It combines with Shatter without freezing the target.
Verified spend/refund and green previews at 720p; [screenshots](images/screenshots/2026-10-25-mage-talents/winters-chill-preview-720.png)
are linked in the README. 89 core and 153 app tests, DBC audit, Clippy and native
build pass (95 seconds). Live combat acceptance remains open.

![screenshots](images/screenshots/2026-10-25-mage-talents/winters-chill-preview-720.png)

## 2026-10-25 — Mage Fire procs

Impact now gives fire hits a trained stun chance, with shorter boss stuns and no
Ignite recursion. Incinerate increases Fire Blast critical chance. Affected spell
tooltips show Impact in green. 87 core and 152 app tests, DBC audit, Clippy and native
build pass (19 seconds). Live combat acceptance remains open.

## 2026-10-25 — Mage reach talents

Flame Throwing and Arctic Reach now extend the appropriate targeting ranges or
Nova/Cone radii. Tooltips use effective range; ground spells retain one location
and unchanged area size. 86 core and 151 app tests, DBC audit, Clippy and native
build pass (21 seconds). Live range-boundary verification remains open.

## 2026-10-25 — Armory save fix and Mage UI milestone

Armory edits now save immediately, fixing lost talent changes when closing the window
directly from Talents. Permafrost previews compare both strength and duration.
Verified refund/spend, independent specs and close/relaunch persistence; added
[Mage milestone screenshots](images/screenshots/2026-10-25-mage-talents/permafrost-720.png) to the README.
85 core and 150 app tests, Clippy and native build pass; live combat verification remains open.

![Mage milestone screenshots](images/screenshots/2026-10-25-mage-talents/permafrost-720.png)

## 2026-10-25 — Mage frost slows

Permafrost strengthens and extends frost slows; Improved Blizzard adds slow strength.
Shared combat/card values now show slow-strength changes inline with one trailing unit.
84 core and 150 app tests, DBC audit, Clippy and native build pass (14 seconds).
Live visual verification remains open.

## 2026-10-25 — Mage Shatter

Mage spells gain rank-scaled critical chance against frozen enemies, including area
ticks. Frost Nova and Frostbite track freezing separately from ordinary roots, with
green conditional-chance tooltips. 83 core and 148 app tests, DBC audit, Clippy and
native build pass (65 seconds). Live combat and screenshot verification remain open.

## 2026-10-25 — Mage Frostbite

Frost damage can now freeze enemies with a rank-scaled chance. Boss roots are shorter,
longer roots are preserved, and trained spell tooltips show the chance in green.
82 core and 147 app tests, DBC audit, Clippy and native build pass (21 seconds).
Shatter and live visual verification remain open.

## 2026-10-25 — Mage Ignite

Critical fire hits now build one Ignite burn, carrying unpaid damage into refreshes
without postponing the next tick. Added green spell-tooltip details and regression
coverage for rank limits, school restrictions, refresh damage and recursion prevention.
Native build, DBC audit, Clippy, 81 core and 146 app tests pass; live visual evidence remains open.

## 2026-10-25 — Mage talent effects

Implemented 16 Mage talents across Arcane, Fire and Frost, with shared previews and
combat values. Fixed ground-area ticks ignoring talent critical bonuses. Remaining
Mage nodes and milestones are still in progress. Native build, DBC audit, 80 core and
144 app tests, and Clippy pass. Talent-page screenshots remain pending a UI input-tool issue.

## 2026-10-25 — Placed-tree camera regression

Added a real Elwynn trunk fixture for blocked/clear camera paths, sphere clearance and
hull removal/reload. This strengthens collision coverage; the reported close-camera
symptom still needs an exact live reproduction. All 141 Survivors tests and Clippy pass.

## 2026-10-25 — Camera obstruction diagnostics

Added optional position/zoom tracing to distinguish collision push-in from visual
occlusion. An Elwynn run kept all 98 sampled gameplay booms at 30 yards; the earlier
close-camera symptom remains unresolved. Native build, 140 Survivors tests and Clippy
pass. Evidence and next checks are in the plan.

## 2026-10-25 — Indoor minimap imagery

Fixed Scarlet's blank map by compositing authored building-group textures. Unchanged
backgrounds are reused while markers update. All 140 Survivors tests, Clippy and native
build pass; live Scarlet zoom and Elwynn terrain checks pass.
[Scarlet screenshot](images/screenshots/2026-10-25-indoor-minimap/scarlet.png).
Broader indoor alignment, camera/navigation and performance acceptance remain open.

![Scarlet screenshot](images/screenshots/2026-10-25-indoor-minimap/scarlet.png)

## 2026-10-25 — Scarlet Monastery candidate

Added the Cathedral courtyard, 18 verified wave identities and eight timed bosses.
Campaign encounters without collection loot now require their final boss defeat.
All 79 core tests, Clippy and native build pass.
[Early combat screenshot](images/screenshots/2026-10-25-scarlet-candidate/combat.png)
reveals wall-navigation and minimap work still needed; Scarlet remains locked.

![Early combat screenshot](images/screenshots/2026-10-25-scarlet-candidate/combat.png)

## 2026-10-25 — Ahn'Qiraj Legendary armor sources

Added eight timed outdoor encounters supplying Legendary armor for every class, with
verified Silithus creature identities. All 78 core and 135 Survivors tests pass, plus
Clippy and native build. A live probe saved three Legendary Warrior pieces.
[Milestone screenshot](images/screenshots/2026-10-25-aq-roster/level-up.png).
The zone remains locked for world acceptance; the content audit still reports 87 gaps.

![Milestone screenshot](images/screenshots/2026-10-25-aq-roster/level-up.png)

## 2026-10-25 — Foreground performance validation

Corrected the profiling setup: prior unattended/background FPS was not representative
of attended play. Foreground Gurubashi held about 90 FPS at a chest; six short combat
samples measured 45–75 FPS. Auto logs now count unfocused/unknown-focus frames; live output, all 134 Survivors
tests, Clippy and the native build pass.
The [development guide](SURVIVORS-DEVELOPMENT.md#measure-attended-performance-explicitly)
documents repeatable foreground measurements. Full performance acceptance remains open.

## 2026-10-25 — Cache repeated combat-text placement

Frozen damage numbers reuse identical overlap placements. A dense 160-label check keeps
every rectangle identical while reducing repeat placement from about 10 ms to 0.001 ms.
All 133 Survivors, 12 combat-text and 10 solver tests pass, plus Clippy and native build.
[Live chest evidence](images/screenshots/2026-10-25-combat-text-cache/chest.png) preserves
the numbers; selecting a reward resumes them. Overall Gurubashi performance remains open.

![Live chest evidence](images/screenshots/2026-10-25-combat-text-cache/chest.png)

## 2026-10-25 — Reuse unchanged minimap images

Paused scenes now reuse their minimap image until visible inputs change. Native build,
all 131 Survivors app tests and workspace Clippy pass; live chest selection confirms the
map updates after combat resumes. Gurubashi remains slow. A new stall sample points to
combat-text overlap placement as the next investigation target; no FPS gain is claimed.

## 2026-10-25 — Correct chest jewelry artwork

Chest jewelry now uses the verified item icons shared with the Codex, replacing generic
spell art. Added a held-chest inspection mode and [live trinket evidence](images/screenshots/2026-10-25-chest-item-icons/trinkets.png)
to README. All 131 app tests and Clippy pass. Mark of the Champion's identity remains
unresolved; reward mechanics and saves are unchanged.

![live trinket evidence](images/screenshots/2026-10-25-chest-item-icons/trinkets.png)

## 2026-10-25 — Spawn clearance and live Epic rewards

Spawn candidates now reject body overlaps with registered collision geometry and retry
other directions. Verified walls, ceilings, slopes and arena-edge spawning; all 130 app
tests and Clippy pass. Native build took 22 seconds. Added a [Gurubashi results milestone](images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png);
isolated accelerated attempts persisted three Epic pieces. Slow combat and generic chest
jewelry icons remain open; Gurubashi stays locked.

![Gurubashi results milestone](images/screenshots/2026-10-25-gurubashi-live/late-warrior-results.png)

## 2026-10-25 — Arena collision evidence and catalog screenshot

Verified Gurubashi's placed arena boundary with body/camera probes and 128 grounded
radial routes across the play area. Added the seeded [54/54 PvP menu milestone](images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png)
to README. Live streaming, camera rendering and crowd acceptance remain open;
Gurubashi stays locked.

![54/54 PvP menu milestone](images/screenshots/2026-10-25-pvp-catalog/equipped-menu.png)

## 2026-10-25 — Gurubashi Epic reward schedule

Added nine timed Epic reward encounters with verified outdoor Stranglethorn identities,
replacing the old mixed raid roster. The drop UI and combat schedule share all nine
slots; all-class save/reload coverage verifies equipment, appearances and duplicate
rewards. All 77 core and 127 app tests pass. Gurubashi remains locked pending world and
live encounter validation; the initial terrain footprint is flat and dry.

## 2026-10-25 — Complete collection identity catalog

Added the 81 PvP class/slot identities, completing all 54 named and mapped cells per
class. All 76 core and 126 app tests pass, including local display/icon reads; native
incremental build took 13 seconds. Acquisition remains incomplete (26 cells per class
lack drops), and full appearance/set-build validation remains open. The new menu capture
is pending because the Mac is locked; existing README screenshots remain historical.

## 2026-10-25 — Legendary armor identities

Added 72 verified class armor identities, bringing each collection to 45 of 54 named
cells. Existing Legendary weapons and numeric save slots remain unchanged. All 75
core and 126 app tests pass, including new display/icon reads; native build took 13 seconds.
The PvP row, remaining acquisition schedules, live appearances and raid-set powers remain open.

## 2026-10-25 — Complete Epic item identities

Added 81 verified Epic armor/weapon identities across all nine classes: each collection
now has 37 of 54 named cells. Existing rarity power and save slots are preserved. Added
an [equipped-menu milestone](images/screenshots/2026-10-25-epic-catalog/equipped-menu.png)
to the README. All 74 core and 126 app tests pass, including new display/icon reads.
Epic sources, full appearance checks and raid-epic/set mechanics remain open.

![equipped-menu milestone](images/screenshots/2026-10-25-epic-catalog/equipped-menu.png)

## 2026-10-25 — Rare rewards and full-health cards

Verified all 81 Rare class/slot rewards through save/reload and duplicate replay,
preserving selected gear, independent appearances, empty slots and class isolation.
Healthstone now clearly says when health is already full; actual heals retain green
inline comparisons. All 126 Survivors app tests pass. Remaining live Rare encounters
and appearance checks stay open.

## 2026-10-25 — Feralas live validation and useful combat metrics

The accelerated Warrior fixture reached 219 enemies and persisted four Rare armor slots
in an isolated save. A [live level-up milestone](images/screenshots/2026-10-25-feralas-live/late-warrior.png)
is linked from the README. Autopilot logs now separate combat FPS/time and hitches from
menu and level-up frames; the new fields were checked in a native run. All 124 Survivors
app tests pass. Early loading hitches and full zone acceptance remain open; Feralas stays locked.

![live level-up milestone](images/screenshots/2026-10-25-feralas-live/late-warrior.png)

## 2026-10-25 — Faster horde collision

Survivors now slides along planes found by successive sweeps, avoiding repeated nearby-face
collection. The 220-body Feralas probe fell from 9.80 to 5.77 ms p95 (about 41%), with
more advancing steps and passing root, slope and two-wall corner checks. All 122
Survivors app tests pass; the native incremental build took 19 seconds. Live horde
profiling and broader world acceptance remain open.

## 2026-10-25 — Horde collision profiling

Added a repeatable, opt-in Feralas collision timing probe at up to 220 bodies. It exposes
roughly 9.8 ms p95 movement cost on this Mac; final-position overlap checks account for
much less in an independent replay. Solver optimization and live gameplay profiling
remain open. Measurements and the command are in the implementation plan. All 121
Survivors app tests with checkout data and workspace Clippy pass; runtime is unchanged.

## 2026-10-25 — Grounded movement across slopes and roots

Walking follows gentle terrain without wedging, recovers missed floor probes at triangle
edges, and rejects ground adjustments that put the character inside a root. The actual
Feralas terrain/root test now advances 39 yards before contact without penetration;
airborne clearance and existing collision checks pass. Native incremental build: 50 seconds;
121 Survivors app tests with checkout data pass. Live horde performance and full Feralas
world acceptance remain open; the arena stays locked.

## 2026-10-25 — Feralas geometry checkpoint

Added a [candidate world view](images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)
to the README and identified nearby root/tree collision hulls. A placed-root regression
covers capsule contact and collider removal/reload. Grounded traversal, gameplay camera
behavior and live encounters remain open; Feralas stays locked.
All 120 Survivors app tests with checkout data, workspace Clippy, formatting and
render-pass lint pass.

![candidate world view](images/screenshots/2026-10-25-feralas-candidate/world-preview-720.png)

## 2026-10-25 — Feralas world and encounter foundation

Added a provisional Feralas arena, 15 verified enemy identities and nine timed bosses.
Its drop schedule covers all Rare slots and waits for the final boss before victory.
Terrain sampling found a dry supported candidate; Feralas remains locked pending full
world, camera, combat and reward validation. Existing saved map indices are preserved.
Native build, 73 core tests, 119 app tests, Clippy and formatting pass; Feralas model
files resolve. The full content audit still reports 476 unfinished-content gaps.

## 2026-10-25 — Rare weapon identities

Added nine verified class weapon identities, completing the Rare tier's nine slots.
Each class now has 28 of 54 named cells; 26 remain unnamed. Weapons retain Survivors
stats. Rare acquisition sources and live weapon attachment/pose checks remain open.
Native build, 72 core tests, 119 app tests with data (including new display/icon reads),
workspace Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Rare armor identities

Added 72 verified armor identities across all nine classes, bringing each collection to
27 of 54 named cells. Rare items use their own icons and appearance metadata; stats remain
Survivors rules. Rare weapons, acquisition sources and live appearance checks remain open.
Added [collection](images/screenshots/2026-10-25-rare-armor/collection-720.png) and
[Warrior preview](images/screenshots/2026-10-25-rare-armor/character-720.png) milestones
from a seeded validation account. Native build, 72 core tests, 119 app tests with data,
workspace Clippy, formatting and render-pass lint pass.

![collection](images/screenshots/2026-10-25-rare-armor/collection-720.png)

## 2026-10-25 — Complete uncommon source table

Clutchmother Zavas, Gruff and U'cha now supply uncommon wrists, hands and weapons at
4:30, 7:30 and 15:00. All nine slots have named sources for every class. Victory requires
all nine scheduled bosses and the 16-minute clock. Un'Goro stays locked pending world
acceptance; live encounters, rewards and balancing remain open.
Native build, 71 core tests, 118 Survivors app tests with checkout data, formatting,
workspace Clippy and render-pass lint pass.
Follow-up: all nine classes now have reward/save replay coverage, including duplicate
drops, selected gear, independent appearances and class isolation; 119 app tests pass.

## 2026-10-25 — Uncommon weapons

Added nine verified weapon identities, completing the uncommon tier's nine slots for
every class. Collections now have 19 of 54 named cells per class; 35 remain unnamed.
Icons and attachments use pinned vanilla identities; rarity stats remain Survivors rules.
Weapon acquisition sources and live attachment validation remain open. Native build,
71 core tests, 118 Survivors app tests (including display/icon reads) and workspace Clippy pass.

## 2026-10-25 — Live card and chest validation

Verified keyboard banish, preserved neighboring cards, charge consumption and mouse
selection at 720p. A [single-card chest](images/screenshots/2026-10-25-banish/single-chest-720.png)
remains centered and usable, including after a jewelry replacement. README evidence added.
Chest instructions now say “Choose your chest reward” to cover both spells and jewelry.

![single-card chest](images/screenshots/2026-10-25-banish/single-chest-720.png)

## 2026-10-25 — Banish replacement safety

Banish now excludes every option in the current hand, including the card being replaced,
and keeps the replacement in its original position. If no distinct reward rolls, the hand
and charge are preserved without granting a free reroll. The banish key no longer falls
through into selecting a card. Native build, 70 core tests, 118 Survivors app tests and
workspace Clippy pass; live input/visual acceptance remains open.

## 2026-10-25 — Distinct chest rewards

New chest hands no longer repeat the same spell upgrade or jewelry piece. With fewer
eligible upgrades, hands can contain fewer than three cards; exhausted upgrade rolls do
not turn into free jewelry. Luck and legendary category thresholds are unchanged.
The two-spell duplicate seen during Un'Goro validation is covered by a regression test.
Native build, 69 core tests and 115 Survivors app tests pass. Reduced-hand live rendering
and duplicate prevention during banish replacement remain follow-up work.

## 2026-10-25 — Un'Goro gameplay investigation

Captured [early Hunter combat](images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)
on the southern candidate and refreshed the README. Moving away from the central tree
opens the view; naming, full traversal, camera, performance and boss/reward checks remain
open. Preview overrides were restored and the zone remains locked.

![early Hunter combat](images/screenshots/2026-10-25-ungoro-gameplay/combat-720.png)

## 2026-10-25 — Un'Goro encounter sources

Six timed bosses now roll uncommon armor from 3:00 through King Mosh at 16:00, using the
same table for combat and item sources. Creature identities and zone placements are
verified against pinned data. The [drop browser](images/screenshots/2026-10-25-ungoro-drops/sources-720.png)
explains why the zone remains locked; arena, camera and live encounter acceptance remain
open. Native build, 68 core tests, 115 Survivors tests and lint checks pass. README evidence updated.

![drop browser](images/screenshots/2026-10-25-ungoro-drops/sources-720.png)

## 2026-10-25 — Collection heading spacing

Centered slot headings within their proportional columns and adjusted text sizing to
separate the crowded Shoulders/Chest labels. Verified selection and resizing at
[1024×720](images/screenshots/2026-10-25-collection-headings/collection-1024.png) and
[1280×720](images/screenshots/2026-10-25-collection-headings/collection-720.png).
Native build and formatting pass; README screenshots refreshed.

![1024×720](images/screenshots/2026-10-25-collection-headings/collection-1024.png)

## 2026-10-25 — Uncommon armor catalog

Added 72 named armor appearances across all nine classes. Each class now has 18 of 54
named cells; new drop sources and the remaining 36 names are still pending. Existing
rarity stats and ownership are preserved. Native build, 67 core tests and 115 Survivors
tests pass, including display/icon checks. Hunter collection details were checked at
[720p](images/screenshots/2026-10-25-uncommon-armor/hunter-720.png) and
[1024×720](images/screenshots/2026-10-25-uncommon-armor/hunter-1024.png); narrow slot-heading
spacing and full armor-on-character validation remain open. README gallery updated.

![720p](images/screenshots/2026-10-25-uncommon-armor/hunter-720.png)

## 2026-10-25 — Class-specific Atiesh appearances

Priest, Mage, Warlock and Druid Atiesh cells now use their verified class reward identities
for collection icons and equipment rendering. Ambiguous name-only lookups stay unresolved.
Native build, 67 core tests and 115 Survivors tests pass, including local display/icon checks.
Live staff attachment checks, acquisition paths and unnamed collection cells remain open.

## 2026-10-25 — Hunter control talents

Scatter Shot and Wyvern Sting add brief shot stuns; Entrapment roots enemies hit by
Explosive Trap. Boss durations are reduced and longer active controls are preserved.
Ability tooltips show both durations. Hunter now has sixteen individual effects; remaining
class work and live control validation remain open. Native build, 66 core tests, 115
Survivors app tests, catalog audit, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Hunter Marksmanship milestones

Efficiency shortens shot cooldowns; Mortal Shots adds direct critical damage; Trueshot Aura
adds two Multi-Shot targets and strengthens Aimed Shot. These compose with individual ranks
and the existing baseline tree bonuses. Volley and trap areas are unchanged. Other Hunter
milestones, remaining talents, pets and live milestone validation remain open. Native build,
65 core tests, 113 Survivors app tests, workspace test compilation, Clippy, formatting and
render-pass lint pass.

## 2026-10-25 — Hunter talent UI evidence

Verified rank refund/spend, independent spec reset and saved allocation after relaunch.
The three-tree page and green descriptions fit 1280×720 and a live resize to 1024×720.
Added [milestone screenshots](images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg)
to the README. No runtime errors or panics appeared; broader visual acceptance remains open.

![milestone screenshots](images/screenshots/2026-10-25-hunter-talents/mortal-shots-720.jpg)

## 2026-10-25 — Hunter Marksmanship bonuses

Barrage, Lethal Shots, Mortal Shots and Ranged Weapon Specialization now affect explicit
ability groups. Shot critical bonuses compose with equipment; traps and Volley receive
no shot-only critical bonus. Barrage improves Volley damage without adding area locations.
Hunter now has thirteen individual effects; remaining talents, pets and milestones are open.
Native build, 64 core tests, 111 Survivors tests, catalog audit, Clippy, formatting and
render-pass lint pass. Live visual acceptance remains open.

## 2026-10-25 — Hunter training foundation

Nine Hunter talents now affect shots, traps, armor, movement, damage reduction or critical
chance instead of generic damage. Tooltips identify Survivors conversions. Rank caps,
selected-spec reset and single-area targeting are preserved. Pets, remaining talents,
milestones and visual acceptance remain unfinished. Native build, 63 core tests,
110 Survivors app tests, catalog audit, Clippy, formatting and render-pass lint pass.

## 2026-10-25 — Grounded tree collision

Fixed a reproduced Un'Goro trunk penetration: steep surfaces now constrain horizontal
sliding instead of lifting actors before grounding. Added a real placed-tree/terrain
fixture covering initial clearance, 80 grounded steps and collider removal. Native build,
workspace test compilation, Clippy, formatting and 109 Survivors tests pass (including
12 floor tests). Full zone traversal,
streaming and navigation validation remain open; Un'Goro stays locked.

## 2026-10-25 — Upgrade comparison units

Upgrade comparisons use one trailing unit: `4 » 3.7 sec`, `5 » 7 yd`.
The inline card text already follows this rule; the remaining ability-summary formatter
now follows it too.

## 2026-10-25 — Un'Goro candidate visual evidence

Added a 720p world-viewer capture to the README and recorded nearby tree-hull probes.
The southern candidate renders jungle terrain with a large trunk near its center; player
clearance, grounded traversal and gameplay-camera checks remain open. No runtime errors
or panics appeared. Validation launch overrides were restored; the zone remains locked.

## 2026-10-25 — Recover terrain boundary heights

Survivors now checks nearby emitted terrain triangles when the fast height lookup misses.
The reproduced Un'Goro boundary resolves correctly; successful queries and real holes
retain their behavior. The change is opt-in for Survivors and its audit. Arena placement
and zone readiness remain unapproved pending collision, visual and content validation.
The southern candidate now resolves all 1,257 survey heights. Native build, workspace
test compilation, Clippy, formatting and 122 scoped tests pass; the feature-off format
build also checks.

## 2026-10-25 — Un'Goro arena investigation

The current summit footprint contains submerged and steep terrain. Surveyed nearby
alternatives and found a height-lookup discrepancy: 17 apparent gaps at a southern
candidate have rendered-mesh support. The audit now reports these discrepancies and
sample coordinates. Location evidence records the reproduction; Un'Goro remains locked.
Audit build, seven tests, scoped Clippy and workspace formatting pass.

## 2026-10-25 — Accurate critical-hit descriptions

Ability tooltips now show the effective critical damage percentage, including talents and
gear, instead of always promising double damage. Critical-chance passive cards/tooltips
now describe chance alone; inline green comparisons still show the unit once. Native build,
workspace test compilation, Clippy, formatting, render-pass lint and 12 HUD tests pass.
Live tooltip validation remains open.

## 2026-10-25 — Warrior strike training

Improved Overpower adds Heroic Strike critical chance; Improved Slam shortens Mortal
Strike's cooldown. Both use the selected spec and existing cast rules, with explicit
Survivors conversion text. They do not grant abilities. Warrior now has 34 supported nodes;
remaining talents and class completion are still open. Native build, workspace test
compilation, Clippy, formatting, 62 core tests, 107 Survivors tests and the local talent
catalog audit pass. Live talent-panel/combat validation remains pending.

## 2026-10-25 — Cutaway scales with the viewport

The foliage reveal now follows camera zoom, perspective and aspect ratio, occupying 35%
of the shorter viewport side. Its center shares the camera's shoulder-height focus.
Native macOS build, workspace Clippy and 61 core tests pass, including projection checks
at near/default/far zoom and landscape/portrait aspects. Live retained-renderer 720p on/off and farther-zoom captures now appear in the README;
the hero and nearby enemies remain visible through the canopy. Resize, other renderer
zoom checks and the broader visibility matrix remain pending.

## 2026-10-25 — Foliage visibility cutaway

Added a saved Scenery cutaway option that reveals the hero and nearby combat through
foliage in both scenery renderers, independently of camera auto-push. Live 720p on/off
comparisons and restart persistence verified; README screenshots added. Collision is unchanged.

Native build, formatting, workspace Clippy, workspace test compilation and focused suites
passed: 106 Survivors, 563 world (6 ignored), 103 assets and 59 core tests. Documentation-link and
render-pass lints also pass. Cleaned up earlier test-style lint failures. Interior visibility,
colored silhouettes, camera motion/zoom and the wider resolution matrix remain unfinished.

![Foliage reveal](images/screenshots/2026-10-25-cutaway/retained-gameplay.jpg)

## 2026-10-24 — Cutaway geometry foundation

Added a tested camera-to-hero reveal calculation with a depth-scaled circular footprint,
soft edge and protection for geometry behind the hero. All 59 core tests pass. This is the
CPU reference only: shader integration, scenery filtering and visibility controls remain
unfinished, so there is no visible cutaway effect yet.

## 2026-10-24 — Live tree contact and visibility

Captured a nearby Elwynn trunk during forward contact and a successful sidestep; links are
in the README. This approach appears blocked, while the trunk obscures the hero and combat.
The reported collision issue remains open across other trees/approaches, as do cutaway and
silhouette rendering. Temporary XP tuning and launch settings were restored.

## 2026-10-24 — Real-placement tree fixture

Added tree 12580 with its actual transform and terrain tile to collision regression coverage.
It reproduces both runtime endpoints without a detected center-ray face crossing. Added
source winding diagnostics to investigate starts inside open hulls. All 106 focused
Survivors tests and the audit-tool tests pass. Rendered trunk confirmation remains open;
no gameplay collision policy changed.

## 2026-10-24 — Repeated tree-walking regression

Extended the six-variant Elwynn hull regression to 2,304 grounded movement segments.
Each approach must respond to the hull; independent center-ray checks reject solid-face
crossings while allowing sliding around trunks. Workspace test compilation and all 105
focused Survivors tests pass. The rotated placement/terrain reproduction remains open.

## 2026-10-24 — Surface-aligned collision diagnostics

Added repeated walking queries and saved reproducible tree-surface points. Canopy tree
12580 allows one approach but blocks the reverse; recorded exact starts, height-sample
failures and lateral movement in [surface evidence](validation/elwynn-tree-surface-queries.txt).
Native build passes. This is a narrowed reproduction candidate, not a collision fix.

## 2026-10-24 — Elwynn collision evidence

Audited source hulls and queried twelve nearby tree placements in the streamed world.
Several trunks block movement queries; canopy-origin paths need exact trunk alignment
before diagnosing a failure. Saved [query evidence](validation/elwynn-tree-queries.txt)
and the next reproduction steps. The reported tree-collision issue remains open.

## 2026-10-24 — Responsive results

Results use proportional widths, bounded controls and wrapping stat tiles/text. Live resize
checks passed at [1280×720](images/screenshots/2026-10-24-results-layout/wide-720.jpg),
[640×720](images/screenshots/2026-10-24-results-layout/narrow-720.jpg) and
[480×720](images/screenshots/2026-10-24-results-layout/wrapped-720.jpg).
Native incremental build passed in 30 seconds. Temporary validation tuning was restored;
these captures verify layout, not balance or the rest of the interface at narrow sizes.

![1280×720](images/screenshots/2026-10-24-results-layout/wide-720.jpg)

## 2026-10-24 — Defeat screen evidence

Added [720p defeat results](images/screenshots/2026-10-24-defeat/results-720.jpg) to the README.
An isolated save and temporary development tuning exercised the death-to-results flow;
normal tuning was restored. A follow-up verified Play again returns to hero selection and
starts a fresh run with full health, a restarted timer and zero damage; see the
[paused replay](images/screenshots/2026-10-24-defeat/replay-720.jpg). Corrected “1 marks”
to “1 mark.” Victory and normal-balance acceptance remain open.

![720p defeat results](images/screenshots/2026-10-24-defeat/results-720.jpg)

## 2026-10-24 — Consistent lethal-hit handling

Spell healing, lifesteal and pickups can no longer revive a hero after a lethal hit but
before the death animation. Pickups also cannot open a level-up choice in that interval.
New casts, chest collection and stale card selections require a living hero. Existing
in-flight attacks still resolve. Manual death/result-screen acceptance remains open.

Native build, workspace test compilation and 105 focused Survivors tests pass.

## 2026-10-24 — Charge training and paused mobility

Improved Charge lengthens Charge's stun; Improved Intercept shortens its cooldown. The
cast, cooldown sweep and tooltip share the trained values. Warrior has 32 supported nodes.
Mobility cooldowns and active speed/vanish effects now freeze during pause, level-up and
loading. Values remain provisional; fresh screenshots and manual Charge checks remain open.

Native build, workspace test compilation, 56 core tests, 102 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Reactive talent HUD

Enrage, Flurry and Blood Craze now show green effect values and remaining seconds beneath
the clock. Lines disappear on expiry or death. Live 720p checks verified activation,
multiple active effects and clearing; screenshots show [active Flurry](images/screenshots/2026-10-24-reactive-buffs/flurry-720.jpg)
and the [cleared HUD](images/screenshots/2026-10-24-reactive-buffs/expired-720.jpg).

Native build, workspace test compilation and 100 focused Survivors tests pass. Dedicated
buff icons, broader viewport checks and full combat acceptance remain open.

![active Flurry](images/screenshots/2026-10-24-reactive-buffs/flurry-720.jpg)

## 2026-10-24 — Flurry training

Direct Warrior criticals can now trigger a three-second Flurry haste bonus. Further
criticals refresh without stacking; bleeds, area ticks and reflection do not trigger it.
The bonus follows existing haste cooldown rules. Warrior now has 30 supported nodes,
and the run-only talent snapshot has room for complete class catalogs without save changes.
Balance, dedicated buff presentation and manual combat acceptance remain pending.

Native build, workspace test compilation, 56 core tests, 99 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Blood Craze and lethal-hit healing

Blood Craze heals over six seconds after health damage; repeated hits refresh without
stacking. Immunity and full absorbs do not trigger it, and spell healing stays independent.
Regeneration can no longer undo a lethal hit before the death check. Warrior now has 29
supported nodes. Values remain provisional; buff presentation and manual combat checks
remain pending.

Native build, workspace test compilation, 55 core tests, 98 focused Survivors tests and
the local talent-data audit pass.

## 2026-10-24 — Inline card visual checkpoint

Verified single-unit green comparisons and removed prose rank labels in the live checkout.
The same hand fits [720p](images/screenshots/2026-10-24-inline-units/cards-720.jpg),
[1280×800](images/screenshots/2026-10-24-inline-units/cards-800.jpg) and
[1024×768](images/screenshots/2026-10-24-inline-units/cards-1024.jpg); selecting Cleave after
resizing grants rank two. A [second hand](images/screenshots/2026-10-24-inline-units/area-1024.jpg)
shows seconds, yards and percentages without repeated units. README evidence refreshed.
Chest/jewelry captures and the complete resolution/UI-scale matrix remain pending.

![720p](images/screenshots/2026-10-24-inline-units/cards-720.jpg)

## 2026-10-24 — Warrior utility training

Death Wish extends Retaliation, including its active Fury risk. Improved Disarm extends
Demoralizing Shout's weakening; Improved Taunt shortens its cooldown. These Survivors
adaptations bring Warrior to 28 supported nodes; values remain provisional.

Native build, workspace test compilation, 54 core tests and 97 focused Survivors tests
pass; the local talent audit verifies node identities and rank caps. Fresh screenshots and manual combat validation remain pending.

## 2026-10-24 — Inline green upgrade descriptions

Cards now embed before/after values and units directly in their descriptions, with changes
green and surrounding prose neutral. This replaces the separate summary. Rank/new-choice
context appears only beneath the spell name; jewelry replacement losses remain red. Tooltips
share the ability prose. Comparison units appear once, after the new value.

Live captures: [new spell/passive previews](images/screenshots/2026-10-24-inline-upgrades/new-720.jpg),
[rank-up and area comparisons at 720p](images/screenshots/2026-10-24-inline-upgrades/upgrade-720.jpg),
[same hand after resizing to 1280×800](images/screenshots/2026-10-24-inline-upgrades/upgrade-800.jpg).

Native build, workspace test compilation and 96 focused Survivors tests pass. Fresh chest
and jewelry captures, plus the full resolution matrix, remain pending.

![new spell/passive previews](images/screenshots/2026-10-24-inline-upgrades/new-720.jpg)

## 2026-10-24 — Inline upgrade presentation planned

Updated the design contract: embed green before/after values, arrows and units directly in
card descriptions, replacing the separate change summary. Implementation and fresh screenshots
are pending; existing summary-style cards are not acceptance evidence for this requirement.

## 2026-10-24 — Card lighting stays on the card

Removed the oversized animated rarity shadow that left a hard rectangular edge across
the backdrop. Hover now brightens the rounded border while retaining lift and tilt;
the card-face lighting and landing sheen remain inside the card.

Native build passes. Verified hover and readable upgrade text at
[1280×720](images/screenshots/2026-10-24-card-lighting/hover-720.jpg) and after a live resize to
[1280×800](images/screenshots/2026-10-24-card-lighting/hover-800.jpg); selecting the resized
card correctly grants its spell. Full rarity/animation/scale coverage remains open.

![1280×720](images/screenshots/2026-10-24-card-lighting/hover-720.jpg)

## 2026-10-24 — Front-door talent activation and Enrage status

Fixed the front-door Enter action dropping the selected hero's active talent snapshot.
Permanent sheet stats were present, but spell-specific ranks and milestones could be missing
in the run. The entry path now copies them just like the other launch paths. Enrage also
has a green HUD line beneath the clock showing its damage bonus and seconds remaining.
Ability descriptions now use the same rank- and talent-scaled effect durations as their
green upgrade previews. [Live Enrage HUD](images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg).
[Trained shout card](images/screenshots/2026-10-24-warrior-talents/trained-shout-card-720.jpg).

Native build, workspace test compilation and 92 focused Survivors tests pass. Live 720p
checks confirm talent activation, Enrage's HUD line and the corrected duration description.

![Live Enrage HUD](images/screenshots/2026-10-24-warrior-talents/enrage-hud-720.jpg)

## 2026-10-24 — Reactive Warrior Enrage

Enrage now grants a four-second damage bonus after a hit removes health. Further hits
refresh the effect without stacking; immunity and full absorbs cannot trigger it.
The green talent preview shows the conditional damage increase. Twenty-five Warrior
nodes now have effects. Dedicated buff presentation and manual combat validation remain open.

Native build, workspace test compilation, 54 core tests, 90 focused Survivors tests and
the local talent-data audit pass. Live combat and fresh description captures remain pending.


## 2026-10-24 — Warrior absorb talents

Shield Specialization and Shield Slam now strengthen Heroic Strike's absorb; Improved
Shield Block extends its duration. Their effects combine with the Protection milestone
without stacking shields or replacing a stronger active shield. Green previews show shield
strength and added seconds. Twenty-four Warrior nodes now have effects; balancing and
manual combat acceptance remain pending.

Native build, workspace test compilation, 54 core tests, 89 focused Survivors tests and
the local talent-data audit pass. Fresh shield-description captures and manual combat
acceptance remain pending.


## 2026-10-24 — Warrior control talents

Piercing Howl adds slow duration to Demoralizing Shout; Concussion Blow adds stun duration
to Heroic Strike. Both compose with their tree milestones, preserve boss resistance and
show green rank comparisons in seconds. Twenty-one Warrior nodes now have effects.
Manual combat validation and balancing remain pending.

Native build, workspace test compilation, 53 core tests, 88 focused Survivors tests and
the local talent-data audit pass. Both green previews fit at 1280×720 on an empty validation
spec. These checks do not establish full manual combat acceptance.
[Piercing Howl](images/screenshots/2026-10-24-warrior-talents/piercing-howl-node-720.jpg) ·
[Concussion Blow](images/screenshots/2026-10-24-warrior-talents/concussion-blow-node-720.jpg).

![Piercing Howl](images/screenshots/2026-10-24-warrior-talents/piercing-howl-node-720.jpg)

## 2026-10-24 — Warrior bleed talents

Deep Wounds now adds bleed damage per talent rank; Improved Rend increases that bleed.
Both replace placeholder bonuses and show green current/next-rank values. Bleeds retain
three-second, non-stacking refresh behavior and cannot trigger themselves. Nineteen
Warrior nodes now have implemented effects; other nodes and manual combat acceptance
remain unfinished. Coefficients are provisional pending balance work.
Native build, workspace test compilation, 52 core tests, 88 focused Survivors tests and
the local talent identity/rank audit pass. Live 1280×720 green descriptions verified:
[Deep Wounds](images/screenshots/2026-10-24-warrior-talents/deep-wounds-ranks-720.jpg) ·
[Improved Rend](images/screenshots/2026-10-24-warrior-talents/improved-rend-720.jpg).
These captures use an empty validation spec; manual combat acceptance remains pending.

![Deep Wounds](images/screenshots/2026-10-24-warrior-talents/deep-wounds-ranks-720.jpg)

## 2026-10-24 — Placed obstacle regression

Added a real-data Warsong stump regression covering player-capsule contact and collider
removal/re-registration. It reproduces the documented ray hit using the actual placement.
[Evidence and limits](SURVIVORS-LOCATIONS.md#placed-stump-capsule-regression).
Workspace test compilation and 87 focused Survivors tests pass with checkout data.
Live streamed collision and navigation remain pending; this is validation, not a collision fix.

## 2026-10-24 — Developer instance preview

The development launcher can now open the engine's world viewer with `--world-preview`.
Verified Warsong terrain and obstacles in a live 720p render; campaign unlocks are unchanged.
[Capture](images/screenshots/2026-10-24-world-preview/warsong-720.jpg) ·
[Launch instructions](SURVIVORS-LOCATIONS.md#developer-instance-preview).
Native build passes. Player collision, navigation and gameplay camera checks remain pending.

![Capture](images/screenshots/2026-10-24-world-preview/warsong-720.jpg)

## 2026-10-24 — Warsong obstacle evidence

Model audits now report transformed collision bounds and rays from the candidate center.
Warsong's proposed arena contains solid stump obstacles; a documented 8.9-yard hit supplies
a fixture for runtime collision and routing checks. [Evidence](SURVIVORS-LOCATIONS.md#warsong-source-model-obstacles).
Six audit tests and Clippy pass. Runtime collision and world rendering remain unverified.

## 2026-10-24 — Arena terrain surveys

Added `--terrain` to measure sampled support, water, impassable terrain and slopes across
candidate arenas, including tile boundaries. Six local surveys reject Arathi's wet midpoint
and narrow the next world checks to a 60-yard Warsong arena. [Evidence and limits](SURVIVORS-LOCATIONS.md#arena-terrain-surveys).
Five audit tests and Clippy pass. No new zone is unlocked by these geometry checks.

## 2026-10-24 — Campaign location evidence

Added a read-only `--locations` audit using local map/area metadata and terrain. Documented
three battleground candidates and the remaining content gaps in the
[location report](SURVIVORS-LOCATIONS.md). Zones remain locked pending traversal, encounter
and reward completion. Validation: four audit tests, local metadata report and three WMO
floor probes pass; these are evidence checks, not playability acceptance.

## 2026-10-24 — Control duration scaling

Slow, root, stun, fear and weakening durations now grow by 10% of base duration per ability
rank after the first. Duration talents compose with rank growth, and green upgrade previews
show effect duration separately from area/buff lifetime. Boss resistance remains in place.
Improved Demoralizing Shout now increases weakening duration instead of placeholder damage.

[Shout duration talent at 720p](images/screenshots/2026-10-24-warrior-talents/shout-duration-720.jpg).
These changes affect combat balance; coefficients remain provisional and full manual combat
validation is still pending.
**Validation:** native build, workspace test compilation, 86 Survivors tests, 51 core tests
and the local talent-data audit pass. Live talent description verified at 720p.

![Shout duration talent at 720p](images/screenshots/2026-10-24-warrior-talents/shout-duration-720.jpg)

## 2026-10-24 — Green talent upgrade previews

Supported talent effects now appear in green beneath white name/rank headings. Each node
shows its current contribution and next-rank value, or its final value at maximum rank.
Milestone effects also use green; unfinished effects retain a clear pending notice.
Comparisons use font-compatible text and distinguish node bonuses from total character stats.

[Green rank preview at 720p](images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg).
**Validation:** native build, workspace test compilation, 85 Survivors tests and 51 core
tests pass. Live 720p checks confirm the comparison, green effect text, white heading and
color reset for an unfinished talent.

![Green rank preview at 720p](images/screenshots/2026-10-24-warrior-talents/green-rank-preview-720.jpg)

## 2026-10-24 — Warrior defensive talents

Four more individual talents replace placeholder damage: Improved Revenge increases
Retaliation damage, Improved Shield Wall extends it, and Last Stand/Bloodthirst add capped
healing to Retaliation/Execute. Node healing combines with milestone healing once per cast.
A longer Retaliation also extends Death Wish's incoming-damage penalty.

[Defensive talent details at 720p](images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg).
Coefficients remain provisional; remaining talents and manual combat validation are pending.
**Validation:** native build, workspace test compilation, 85 Survivors tests, 50 core tests,
and the local talent-data audit pass. Improved Shield Wall details verified at 720p.

![Defensive talent details at 720p](images/screenshots/2026-10-24-warrior-talents/defensive-talents-720.jpg)

## 2026-10-24 — Protection milestones

Last Stand heals on Retaliation, Concussion Blow adds a brief Heroic Strike stun, and
Shield Slam grants a small absorb once per Heroic Strike cast. Healing caps at maximum
health; absorbs do not stack or refresh a stronger shield. All nine Warrior milestone
behaviors are implemented; remaining talents and manual combat validation are unfinished.

[Protection details at 720p](images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 84 Survivors tests and 50 core
tests pass. Live 720p description verified. Coefficients remain provisional.

![Protection details at 720p](images/screenshots/2026-10-24-warrior-talents/protection-milestones-720.jpg)

## 2026-10-24 — Fury milestones

Piercing Howl adds a slow to Demoralizing Shout. Death Wish boosts Retaliation with an
incoming-damage tradeoff while active. Bloodthirst heals once per successful Execute cast.
The implementation plan records the provisional coefficients and how these compose with
existing effects.

[Fury details at 720p](images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 83 Survivors tests and 49 core
tests pass; the Death Wish description fits at 720p. A full manual Fury encounter is still
unverified. Protection milestones and remaining individual talents are unfinished.

![Fury details at 720p](images/screenshots/2026-10-24-warrior-talents/fury-milestones-720.jpg)

## 2026-10-24 — Arms milestones and reliable DoT ticks

Warrior Arms now gains Deep Wounds at 10 points, a larger Sweeping Strikes aura at 20,
and stronger Mortal Strike at 30. The bleed refreshes without stacking or postponing its
next tick. Damage-over-time catches up across long frames and stops at expiry.

[Arms milestone details at 720p](images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg).
**Validation:** native build, workspace test compilation, 82 Survivors tests and 48 core
tests pass. Live 720p description checked; a full manual Arms encounter remains unverified.
Fury/Protection milestones and remaining individual talents are still in progress.

![Arms milestone details at 720p](images/screenshots/2026-10-24-warrior-talents/arms-milestones-720.jpg)

## 2026-10-24 — Warrior talent effects

Twelve audited Warrior talents now improve specific attacks, shout radius, critical stats,
armor or damage reduction. Their effects replace placeholder damage, follow the selected
hero/spec, and appear in talent details. Other Warrior nodes and milestone behaviors remain
unfinished. The save format is unchanged.

**Validation:** native build, workspace test compilation, 79 Survivors tests, 47 core rules
tests, and the checkout talent-data audit pass. Live Heroic Strike and Booming Voice details
fit at 720p. [Milestone screenshot](images/screenshots/2026-10-24-warrior-talents/talents-720.jpg).

![Milestone screenshot](images/screenshots/2026-10-24-warrior-talents/talents-720.jpg)

## 2026-10-24 — Spell-specific legendary bonuses

Legendary amulets now enhance only their associated spell. Green comparisons name that
spell; replacements show lost bonuses in red. Extra targets remain limited to eligible
single-target attacks, with no additional AoE locations.

[Single-target bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-direct-720.jpg) ·
[Area bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-area-720.jpg).
Live 720p checks confirm both descriptions. Native build, workspace test compilation,
77 Survivors tests and 45 core rules tests pass. Bonus values remain provisional.

![Single-target bonuses](images/screenshots/2026-10-24-codex-discovery/legendary-direct-720.jpg)

## 2026-10-24 — Codex effect descriptions

Selected jewelry shows its implemented standalone bonuses in green, using the same rules
as chest comparisons. This checkpoint exposed global legendary bonuses; the later scope correction above fixes them.
**Validation:** native build, workspace test compilation and 75 Survivors tests pass.
[Live 720p capture](images/screenshots/2026-10-24-codex-discovery/codex-effects-720.jpg)
confirms green ring bonuses match the rules.

![Live 720p capture](images/screenshots/2026-10-24-codex-discovery/codex-effects-720.jpg)

## 2026-10-24 — Shared jewelry artwork

Codex rings, trinkets and amulets now use compact icon cells, dimmed until offered.
Thirty-eight items have verified display mappings from the pinned vanilla catalog. Mark of the Champion
remains a selectable question mark because its name identifies two different displays.
**Validation:** native build, workspace test compilation and 74 Survivors tests pass,
including all mapped icon files in the explicit data chain. The
[720p capture](images/screenshots/2026-10-24-codex-discovery/codex-jewelry-720.jpg) shows
the compact grid; mapped and unresolved cells both open their details.

![720p capture](images/screenshots/2026-10-24-codex-discovery/codex-jewelry-720.jpg)

## 2026-10-24 — Codex browsing

Selecting an entry updates the details text in place, preserving the catalog's scroll
position. Native build and a live 720p check with two amulet selections pass.
Workspace test compilation and all 73 Survivors tests pass.

## 2026-10-24 — Codex layout

The catalog and selected-entry details now use proportional, independently scrolling panes.
Class rows use class colors and the footer totals discoveries across heroes. Shared entries
show whether they have been offered. Jewelry artwork and legendary amulet badges remain
unfinished; the current rows retain names and spell icons.

**Validation:** app library check, Clippy and native build passed (1m08s).
[720p capture](images/screenshots/2026-10-24-codex-discovery/codex-layout-720.jpg) verifies
amulet selection/details and reachable footer. Selection still resets catalog scrolling.

![720p capture](images/screenshots/2026-10-24-codex-discovery/codex-layout-720.jpg)

## 2026-10-24 — Shared Codex discovery

Rings, trinkets and shared amulets now keep account-wide discovery when their cards are
dealt. Legendary discovery follows the actual hand too, so discarded banish candidates
no longer count. Shared rows show discovered totals instead of always showing completion.
Jewelry effects remain run-only; old saves begin with no shared discoveries recorded.

**Validation:** workspace test rebuild, 73 Survivors tests and workspace Clippy pass.
The native build passed. A 1280×720 live run confirms green level-up and chest scaling
previews ([level up](images/screenshots/2026-10-24-codex-discovery/level-up-720.jpg),
[chest](images/screenshots/2026-10-24-codex-discovery/chest-720.jpg)). Jewelry offer/restart
validation and the Codex icon/detail layout remain unfinished; shared amulets clip at 720p.

![level up](images/screenshots/2026-10-24-codex-discovery/level-up-720.jpg)

## 2026-10-24 — Validation follow-up

Corrected another US-locale test assumption while retaining checks for base string tables
and wrapped localization assignments. The audit now identifies duplicated Survivors UI
labels that need installed-string lookups. The complete workspace run finished with
7,917 passed, five failed and 18 ignored across 143 reported suites. Remaining failures:
reference strings, UI VM dependencies, web assets and the world API boundary. The full
repository gate is not green; the optional addon corpus is absent.

## 2026-10-24 — Class Codex completion

Class amulet completion now uses that class’s actual spell catalog for the Codex, cloaks
and achievement totals. Duplicate save rows no longer inflate progress. Unknown legacy
names are preserved but do not count. Recognized legacy names are normalized to restore
ability-key underscores after reload. Shared jewelry discovery and the Codex layout remain open.

**Validation:** 45 core and 71 Survivors app tests pass, including all-class completion,
legacy save reload and duplicate-count protection. Formatting and workspace Clippy pass;
the full gate is still running.

## 2026-10-24 — Portrait input during loading

Portrait rotation now stops under the loading cover and cancels held gestures. OS window
events remain available, including close requests. Added the missing input-audit decisions
for the portrait and browser heartbeat.

**Validation:** workspace test rebuild and nine focused portrait/loading-cover checks pass.
Formatting and workspace Clippy also pass. The full gate passed the cover audit, then
found a stale launcher example; its missing Lua patch and fork URLs are corrected, with
both launcher checks passing. Later gate stages and live interaction remain unverified.

## 2026-10-24 — Locale-aware validation

Corrected UK-only test assumptions for US client data. Glue strings are checked against
Lua execution, both clock formats are tested, and the verified US locale symbol is recorded
in the API reference without allowing arbitrary locale globals. Gameplay is unchanged.

**Validation:** 15 focused tests and the workspace test rebuild pass. The full gate passes
formatting, Clippy and the app unit suite, then stops at the loading-cover input audit:
portrait window events and browser wakeup types need explicit handling/classification.
Later gate stages remain unverified.

## 2026-10-24 — Loading and development integration

Added a loading-cover warmup draw for ability cooldown overlays. Moved native tuning/layout
reload, F12 captures and collision probes behind the existing development-only entry point;
player builds retain embedded defaults.

**Validation:** workspace test rebuild, 79 focused tests, formatting and workspace Clippy
pass. The player-only check passes with five warnings (four unused imports and an
unfulfilled rune lint expectation). The three remaining locale failures are confirmed enGB
test/reference assumptions against this checkout’s enUS assets. No UI layout changed.

## 2026-10-24 — Validation baseline

Normalized formatter drift in 22 files, simplified two core test setups, and named the
arena-scout candidate structure. No gameplay or visual changes.

Named repeated Bevy query shapes, restricted test-only helpers to test builds, and cleaned
up test initializers and placement. The reserved rune pickup rule keeps a specific warning
expectation until its spawning exists; no global warning suppression was added.

**Validation:** workspace formatting and workspace/all-target Clippy with warnings denied
now pass. The workspace run reached the app suite: 4,294 passed, six failed and ten were
ignored. The mock-network failure passes with loopback permission outside the sandbox;
five failures reproduce (material warmup, dev boundary, two localization checks and the
minimap clock format). Later gate stages have not run. The optional addon corpus is absent.
Earlier core validation passed 45 tests. The earlier
terrain-only diagnosis was incomplete because the gate prints a log tail.

## 2026-10-24 — Drop Tables browser

Replaced the text list with zone browsing, encounter/item rows, collection counts and shared
appearance, equipment and item-detail controls. Browsing leaves the run destination intact.
Missing tables are labeled explicitly. Drop Tables, Collection sources and item details
now include Keen Eye using combat’s shared chance calculation. Scheduled encounters are
labeled as bosses rather than deriving a final-boss claim from the zone-summary name.

**Validation:** native build and 72 focused tests passed. Live 1280×720 checks covered
Elwynn/Duskwood sources, item selection, missing content and the unchanged run destination.
The full resolution matrix and missing content remain open. Existing terrain formatting
still blocks the repository gate.

Keen Eye follow-up: native build, 45 core tests and 73 focused app tests passed; live rank-two Drop Tables and Collection labels
showed 70% from a 60% base at 1280×720. Combat odds are unchanged.

![Drop Tables at 1280×720](images/screenshots/2026-10-24-drop-tables/keen-eye-720.jpg)

## 2026-10-24 — Shared Collection appearance

Collection now shares Character’s faction portraits and race/sex controls, with class
restrictions and per-hero saving. Equipment remains directly below the controls.

**Validation:** native build and 71 focused tests passed. Live 1280×720 checks covered
race/sex changes, the matching Character model, and visibility of all nine empty slots.
The broader resolution/account matrix remains open; existing terrain formatting still
blocks the repository gate.

![Collection appearance at 1280×720](images/screenshots/2026-10-24-collection-appearance/collection-720.jpg)

## 2026-10-24 — Collection artwork

Collection cells now show implemented item artwork, dim unowned pieces, and keep E/L badges
in the corner. Missing-item source entries share the dimmed artwork, and rarity rows
list zones from implemented drop tables. Content without verified artwork retains a placeholder.

**Validation:** native build and 71 focused tests passed. Live checks covered owned icons,
combined E/L badges and legendary selection at 1440×900, plus unowned artwork and clickable
missing-item sources at 1280×720.
Existing terrain formatting still blocks the full repository check.

![Collection artwork](images/screenshots/2026-10-24-collection-icons/collection.jpg)
[Missing-item sources at 1280×720](images/screenshots/2026-10-24-collection-icons/sources-720.jpg)

## 2026-10-24 — Equipment rows

Character and Collection now show equipped item icons beside rarity-colored names, with
an explicit selection highlight and aligned empty slots. Icon sizes follow the UI text scale.

**Validation:** native build and 71 focused tests passed. Live 1440×900 checks covered
all nine equipped icons, selection/details, unequip/re-equip and the shared Collection rows.
Existing terrain formatting still blocks the full repository check.

![Equipment rows and dressed preview](images/screenshots/2026-10-24-equipment-rows/character.jpg)

## 2026-10-24 — Character appearance controls

Race and sex can now be selected above Character equipment. The preview updates immediately
and each named hero saves its own choice. Every race in each class's existing catalog now
offers both sexes, filling missing bodies with verified same-race trainer presets while
preserving previous appearances. Skin, face, hair and race-specific features now have live
controls and randomization, saved independently for each hero. Race portraits are grouped
by faction, follow the selected sex, and dim unavailable class choices. Feature edits update the counters and model in place, preserving the
Character screen and preview rotation. Short mouse drags now rotate the model reliably,
with sensitivity proportional to the preview width.

**Validation:** native build and 71 focused tests passed, including local DBC
identities and readable model files for every body. Live 1440×900 checks covered human,
night elf, newly available female orc/dwarf warriors, separate heroes and restart persistence.
Feature edits, hero switching and restart persistence also passed at 1440×900 / 1280×720.
Repeated feature edits and retained preview rotation were checked at 1280×720. Transient
missing button glyphs remain open, along with the full race/class/gear
visual matrix. Existing terrain formatting errors
still block the repository gate.

Short/reverse drags and rejection of presses outside the preview passed live at 1280×720.
Portrait selection, sex changes and disabled Paladin races also passed at 1280×720.
Feature controls now appear when a new hero’s body becomes ready; creation, immediate hair
edits and retained preview rotation passed live at 1280×720.

![Character race portraits, 1280×720](images/screenshots/2026-10-24-character-features/character-portraits.jpg)
[Independent hero appearance](images/screenshots/2026-10-24-character-appearance/character.jpg)
· [Drag rotation](images/screenshots/2026-10-24-character-features/character-drag.jpg)

## 2026-10-24 — Readable upgrade benefits

Level-up and chest choices show stat changes in green. New spells and passives preview
their next rank; jewelry compares the resulting loadout, with replacement losses in red.
Next-rank tooltips use the same green. Gameplay values are unchanged.

**Validation:** native build and 65 Survivors/preview tests passed, including replacement
losses and area/defensive labels. Live level-up cards checked at 1440×900; jewelry comparison
is covered by rules tests, not yet a live chest screenshot. Existing terrain formatting
errors still block the repository-wide gate.

![Green upgrade and next-rank previews, 1440×900](images/screenshots/2026-10-24-upgrade-benefits/level-up.jpg)

## 2026-10-24 — Dressed heroes and independent cosmetics

Selected armor and weapons now dress the world hero and Character preview. Hidden pieces
and appearance overrides leave equipped bonuses unchanged. The preview backs away to fit
large attachments; newly loaded weapon appearances refresh without a second click.
Weapon poses and the broader race/sex/weapon matrix remain unfinished.

**Validation:** native build, 44 core tests and 62 Survivors/preview tests passed. Seeded
accounts verified plate armor, hiding helmet/chest, independent weapon appearance and
world entry; weapon framing checked at 1280×720 and 1440×900, with the fixed first-load
replacement verified at 1280×720.
Repository checks still stop on two pre-existing terrain formatting errors.

![Dressed Character preview](images/screenshots/2026-10-24-dressed/character.jpg)
[Hidden helmet](images/screenshots/2026-10-24-dressed/hidden-helm.jpg) ·
[Large weapon](images/screenshots/2026-10-24-equipment-framing/weapon-appearance.jpg) ·
[1280×720 layout](images/screenshots/2026-10-24-equipment-framing/character-720.jpg)

## 2026-10-24 — Item visual identities

Collection and Character item details now show verified vanilla icons. Added 91 named
item mappings and corrected two item labels without changing saves. Atiesh variants and
unnamed rarity cells remain unresolved; dressing the hero in selected gear is still pending.

**Validation:** native build, 43 core tests and 58 Survivors tests passed, including local
DBC/icon availability. Helmet → axe → unresolved-item switching checked at 1440×900.

![Collection item detail](images/screenshots/2026-10-24-gear-icons/collection.jpg)

## 2026-10-24 — Character loadout layout

Character now has proportional, independently scrollable equipment, hero and item-detail
columns, plus a full-body current-hero preview with turn buttons. Named heroes retain their
own race and sex; older saves migrate their existing choice. Slot inspection and
appearance choices share Collection's loadout state. Native build and 57 Survivors tests
passed; framing, turn
buttons and tab transitions checked at 1440×900. Drag validation, selected-gear dressing,
inline features, populated-account checks and the resolution matrix remain unfinished.

![Character preview](images/screenshots/2026-10-24-preview/character.jpg)

## 2026-10-24 — Armory, progression and world interaction

- **UI:** three-column front door and Collection, illustrated talent trees, and a 14-screen
  README gallery. Upgrade cards wrap and scroll on narrower windows; shine stays on the
  card face and the oversized background rays are removed.
- **Progression:** selected heroes persist across restarts; worn equipment is separate from
  collected items. Duplicate drops add no power; rarity improves stats. Talent refunds
  preserve row requirements, with Paladin stat, spell, healing and shield effects implemented.
- **World:** movement sweeps against world geometry, nearby-floor placement for interiors,
  terrain-following mobility on gentle slopes, and a saved camera push-in option with
  smooth zoom restoration. Missing campaign content
  stays locked. Source-versus-live Elwynn probes expose offset canopy hulls; the reported
  tree-collision issue still needs visual reproduction.
- **Development:** incremental native builds, an engine-independent rules crate, live tuning
  and layout reloads, plus opt-in F12 screenshots. Failed replacement launches keep the
  current game open; validation shutdown avoids contacting the terminated macOS app.

**Validation:** native builds pass; recent Survivors regression suite: 54 tests passed.
Live Elwynn mobility probes traverse all four 12-yard open-ground routes.
Card reflow was exercised at 1000×800 and the gallery at 1440×900 logical resolution.
The repository check still stops at two existing terrain-stream formatting failures.

**Still in progress:** remaining Armory layouts, full gear/zone catalogs, other class talents,
raid sets, tree-collision investigation, foliage cutaway and colored silhouettes, the full
resolution matrix, and the authenticated updater. Screenshots show current UI, not completion
of every underlying feature.

| Front door | Collection |
| --- | --- |
| ![Front door](images/screenshots/2026-10-24/front-door.jpg) | ![Collection](images/screenshots/2026-10-24/collection.jpg) |
| Talent-tree presentation; effects incomplete | Responsive upgrade cards |
| ![Talent trees](images/screenshots/2026-10-24/talents.jpg) | ![Upgrade cards](images/screenshots/2026-10-24/level-up.jpg) |

[Full interface gallery](../README.md#interface-gallery) ·
[Build and capture instructions](SURVIVORS-DEVELOPMENT.md)
