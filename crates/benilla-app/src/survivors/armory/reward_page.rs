//! Account rewards, arranged around the intention's currency rail and illustrated cards.
use super::{chip, hud, rewards, Account, Act, Armory, ChipParent, DIM, GOLD};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

/// The rendered category owns the offset even when an action changes the selected category.
#[derive(Component)]
pub(in crate::survivors) struct ScrollCategory(usize);

pub(in crate::survivors) fn remember_scroll(
    mut armory: ResMut<Armory>,
    surfaces: Query<(&ScrollCategory, &ScrollPosition)>,
) {
    for (category, position) in &surfaces {
        if position.0.is_finite()
            && armory
                .reward_scroll
                .get(category.0)
                .is_some_and(|old| *old != position.0)
        {
            armory.reward_scroll[category.0] = position.0;
        }
    }
}

#[derive(Component)]
pub(in crate::survivors) struct SidebarActions;
#[derive(Component)]
pub(in crate::survivors) struct PendingActions;

/// Pending purchases must remain actionable when the reference rail scrolls away.
pub(in crate::survivors) fn keep_actions_visible(
    armory: Res<Armory>,
    viewport: Query<(&ComputedNode, &UiGlobalTransform), With<ScrollCategory>>,
    sidebar: Query<(&ComputedNode, &UiGlobalTransform), With<SidebarActions>>,
    mut pending: Query<&mut Node, With<PendingActions>>,
) {
    let Ok((view, transform)) = viewport.single() else {
        return;
    };
    if view.size().min_element() <= 0.0 {
        return;
    }
    let bounds = Rect::from_center_size(transform.translation, view.size());
    let visible = sidebar.single().is_ok_and(|(node, transform)| {
        let rect = Rect::from_center_size(transform.translation, node.size());
        node.size().min_element() > 0.0 && bounds.contains(rect.min) && bounds.contains(rect.max)
    });
    let display = if !armory.reward_draft.is_empty() && !visible {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut pending {
        if node.display != display {
            node.display = display;
        }
    }
}

fn purchase_actions(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, pending: bool) {
    for (label, act, selected) in [
        ("Confirm", Act::ConfirmRewards, true),
        ("Undo", Act::UndoRewards, false),
    ] {
        let node = Node {
            min_height: Val::Px(24.0),
            padding: UiRect::horizontal(Val::Px(7.0)),
            justify_content: JustifyContent::Center,
            ..default()
        };
        if pending {
            super::controls::button(ui, font, label, act, selected, node);
        } else {
            super::controls::disabled(ui, font, label, node);
        }
    }
}

#[derive(Component)]
pub(in crate::survivors) struct CompanionGrid;

/// Reserve reading space after the icon and purchase controls, using the pane's
/// logical width so stacked layouts and UI scaling share the same breakpoint.
pub(in crate::survivors) fn layout_companions(
    mut grids: Query<(&ComputedNode, &mut Node), With<CompanionGrid>>,
) {
    for (computed, mut node) in &mut grids {
        let width = computed.size().x * computed.inverse_scale_factor();
        if width <= 0.0 {
            continue;
        }
        let columns = if width < 648.0 { 1 } else { 2 };
        let tracks = vec![RepeatedGridTrack::minmax(
            columns,
            MinTrackSizingFunction::Px(0.0),
            MaxTrackSizingFunction::Fraction(1.0),
        )];
        if node.grid_template_columns != tracks {
            node.grid_template_columns = tracks;
        }
    }
}

const FRAME: Color = Color::srgb(0.48, 0.37, 0.12);

fn text(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, value, size, color));
}

pub(super) fn frame(
    ui: &mut ChildSpawnerCommands,
    basis: f32,
    grow: f32,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            flex_basis: Val::Px(basis),
            flex_grow: grow,
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.027, 0.014)),
        BorderColor::all(FRAME),
    ))
    .with_children(body);
}

pub(super) fn build(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let preview = armory.reward_draft.preview(account);
    let account = preview.as_ref().unwrap_or(account);
    let category = usize::from(armory.reward_page.min(2));
    panel.spawn((super::collection::CollectionScroll, ScrollCategory(category), RelativeCursorPosition::default(), ScrollPosition(armory.reward_scroll[category]), Node {
        width: Val::Percent(100.0), flex_grow: 1.0, flex_basis: Val::Px(0.0), min_height: Val::Px(0.0),
        flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), row_gap: Val::Px(12.0),
        overflow: Overflow::scroll_y(), align_content: AlignContent::Start, ..default()
    })).with_children(|row| {
        frame(row, 190.0, 1.0, |side| {
            super::typography::section(side, font, "Rewards", 22.0, GOLD);
            side.spawn(Node { flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Baseline, column_gap: Val::Px(6.0), ..default() }).with_children(|currency| {
                text(currency, font, account.unspent().to_string(), 36.0, GOLD);
                text(currency, font, "points to spend", 13.0, DIM);
            });
            let earned = rewards::points_of(account.earned_ids());
            text(side, font, format!("{earned} earned · {} spent", earned.saturating_sub(account.unspent())), 12.0, DIM);
            text(side, font, format!("{} of {} available achievements", rewards::earned_count(account.earned_ids()), rewards::available_count()), 12.0, DIM);
            let pending = !armory.reward_draft.is_empty();
            if pending {
                text(side, font, format!("{} pending {}", armory.reward_draft.codes().len(), if armory.reward_draft.codes().len() == 1 { "change" } else { "changes" }), 13.0, hud::card_growth::GAIN);
            }
            side.spawn((SidebarActions, Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), ..default() }))
                .with_children(|buttons| purchase_actions(buttons, font, pending));
            side.spawn(Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() }).with_children(|categories| {
                for (i, name) in ["Perks", "Companions", "Tabards"].iter().enumerate() {
                    super::controls::button(categories, font, name, Act::RewardPage(i as u8), armory.reward_page == i as u8,
                        Node { width: Val::Percent(100.0), min_height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)), justify_content: JustifyContent::FlexStart, ..default() });
                }
            });
            super::typography::section(side, font, "Reset", 18.0, GOLD);
            text(side, font, format!("Refund every point spent on perks. Companion purchases and tabards are kept. Resets remaining: {}.", account.resets_left()), 12.0, DIM);
            side.spawn(Node::default()).with_children(|slot| chip(slot, font, "Reset perks", Act::Reward(12), false));
            remove_pending(side, font, armory, 12);
            text(side, font, "Purchases and resets are previews until Confirm. Undo discards all pending changes. Closing the app discards them too. Looks apply immediately.", 12.0, DIM);
            if let Some(next) = account.next_locked() { text(side, font, next, 12.0, GOLD); }
        });
        frame(row, 540.0, 4.0, |body| {
            super::typography::section(body, font, match armory.reward_page { 1 => "Companions", 2 => "Tabards", _ => "Perks" }, 22.0, GOLD);
            match armory.reward_page {
                1 => super::companion_page(body, font, armory, account),
                2 => tabards(body, font, account),
                _ => perks(body, font, armory, account),
            }
        });
    });
    panel
        .spawn((
            PendingActions,
            Node {
                display: Display::None,
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.027, 0.014)),
            BorderColor::all(FRAME),
        ))
        .with_children(|bar| {
            text(
                bar,
                font,
                format!(
                    "{} pending · {} points remaining",
                    armory.reward_draft.codes().len(),
                    account.unspent()
                ),
                13.0,
                GOLD,
            );
            purchase_actions(bar, font, !armory.reward_draft.is_empty());
        });
}

fn remove_pending(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, armory: &Armory, code: u8) {
    let count = armory
        .reward_draft
        .codes()
        .iter()
        .filter(|&&pending| pending == code)
        .count();
    if count > 0 {
        text(
            ui,
            font,
            format!("{count} pending"),
            12.0,
            hud::card_growth::GAIN,
        );
        ui.spawn(Node::default()).with_children(|buttons| {
            chip(
                buttons,
                font,
                if code == 12 {
                    "− Undo reset"
                } else {
                    "− Undo upgrade"
                },
                Act::RemoveReward(code),
                false,
            );
        });
    }
}

fn perks(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, armory: &Armory, account: &Account) {
    let icons = [
        "INV_Misc_PocketWatch_01",
        "INV_Misc_Coin_01",
        "INV_Misc_Book_09",
        "Spell_Shadow_Cripple",
        "INV_Misc_Bag_10",
        "INV_Misc_Spyglass_03",
    ];
    text(
        ui,
        font,
        "Use + to preview a rank and − to remove a pending rank. Confirm applies your purchases.",
        12.0,
        DIM,
    );
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(8.0),
        row_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|grid| {
        for code in [0, 3, 1, 4, 2, 5] {
            let perk = rewards::PERKS[code];
            let rank = account.perks.rank(perk);
            let max = rewards::perk_max_rank(perk);
            let cost = rewards::perk_cost(perk, rank);
            let pending = armory
                .reward_draft
                .codes()
                .iter()
                .filter(|&&value| usize::from(value) == code)
                .count();
            grid.spawn((
                Node {
                    min_width: Val::Px(300.0),
                    flex_basis: Val::Percent(48.0),
                    flex_grow: 1.0,
                    padding: UiRect::all(Val::Px(8.0)),
                    column_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(if rank > 0 {
                    Color::srgb(0.14, 0.10, 0.035)
                } else {
                    Color::srgb(0.025, 0.02, 0.012)
                }),
                BorderColor::all(FRAME),
            ))
            .with_children(|card| {
                card.spawn((
                    super::talent_page::Backdrop(format!("Interface\\Icons\\{}.blp", icons[code])),
                    ImageNode::default(),
                    Node {
                        width: Val::Px(34.0),
                        height: Val::Px(34.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));
                card.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|content| {
                    content
                        .spawn(Node {
                            flex_wrap: FlexWrap::Wrap,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(3.0),
                            ..default()
                        })
                        .with_children(|heading| {
                            text(heading, font, rewards::perk_name(perk), 15.0, GOLD);
                            heading
                                .spawn(Node {
                                    column_gap: Val::Px(4.0),
                                    ..default()
                                })
                                .with_children(|pips| {
                                    for n in 0..max {
                                        pips.spawn((
                                            Node {
                                                width: Val::Px(7.0),
                                                height: Val::Px(7.0),
                                                border_radius: BorderRadius::MAX,
                                                border: UiRect::all(Val::Px(1.0)),
                                                ..default()
                                            },
                                            BorderColor::all(FRAME),
                                            BackgroundColor(if n < rank {
                                                GOLD
                                            } else {
                                                Color::BLACK
                                            }),
                                        ));
                                    }
                                });
                        });
                    let (before, value, after) = rewards::perk_preview(perk, rank);
                    content
                        .spawn(hud::text_bundle(font, "", 12.0, DIM))
                        .with_children(|sentence| {
                            for (part, color) in [
                                (before.to_string(), DIM),
                                (value, hud::card_growth::GAIN),
                                (after.to_string(), DIM),
                            ] {
                                sentence.spawn((
                                    TextSpan::new(part),
                                    TextColor(color),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                ));
                            }
                        });
                });
                card.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    width: Val::Px(78.0),
                    flex_shrink: 0.0,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|actions| {
                    text(
                        actions,
                        font,
                        cost.map_or_else(|| "Max rank".into(), |value| format!("{value} points")),
                        12.0,
                        DIM,
                    );
                    actions
                        .spawn(Node {
                            column_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            perk_step(
                                buttons,
                                font,
                                "−",
                                (pending > 0).then_some(Act::RemoveReward(code as u8)),
                            );
                            perk_step(
                                buttons,
                                font,
                                "+",
                                cost.filter(|&value| account.unspent() >= value)
                                    .map(|_| Act::Reward(code as u8)),
                            );
                        });
                    if pending > 0 {
                        text(
                            actions,
                            font,
                            format!("{pending} pending"),
                            11.0,
                            hud::card_growth::GAIN,
                        );
                    }
                    if cost.is_some_and(|value| account.unspent() < value) {
                        text(actions, font, "Need points", 11.0, DIM);
                    }
                });
            });
        }
    });
}

fn perk_step(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str, act: Option<Act>) {
    let node = Node {
        width: Val::Px(22.0),
        height: Val::Px(22.0),
        justify_content: JustifyContent::Center,
        ..default()
    };
    if let Some(act) = act {
        super::controls::button(ui, font, label, act, false, node);
    } else {
        super::controls::disabled(ui, font, label, node);
    }
}

// Values come from the same trip calculation that drives the fetchers.
fn comparison(current: f32, next: f32, unit: &str) -> String {
    let number = |value: f32| {
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    };
    if (current - next).abs() < 0.001 {
        format!("{}{unit}", number(current))
    } else {
        format!("{} » {}{unit}", number(current), number(next))
    }
}

pub(super) fn companions(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    use super::super::companions::trip;
    panel.spawn(hud::text_bundle(font, "Instincts fill the first companion, then the second. Further purchases replace the first companion's instinct. Appearance selection does not change this order.", 12.0, DIM));
    panel.spawn((CompanionGrid, Node { width: Val::Percent(100.0), display: Display::Grid, grid_template_columns: vec![RepeatedGridTrack::minmax(1, MinTrackSizingFunction::Px(0.0), MaxTrackSizingFunction::Fraction(1.0))], column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), ..default() })).with_children(|grid| {
        for code in [13, 6, 7, 8, 9, 10, 11] {
            let pets = &account.pets;
            let target = account.instinct_target();
            let current = target.map_or(0, |slot| pets.instinct[slot]);
            let (name, icon, rank, max) = match code {
                13 => ("Companion", "INV_Box_PetCarrier_01", u8::from(pets.owned > 0), 1),
                6 => ("Long Leash", "Ability_Hunter_BeastCall", pets.leash.min(3), 3),
                7 => ("Quick Paws", "Ability_Hunter_Pet_Cat", pets.paws.min(3), 3),
                8 => ("Gold-sniffer", "INV_Misc_Coin_01", u8::from(current == 1), 1),
                9 => ("Wisp-herder", "Spell_Nature_WispSplode", u8::from(current == 2), 1),
                10 => ("Rune-hound", "INV_Misc_Rune_01", u8::from(current == 3), 1),
                _ => ("Second Companion", "INV_Box_PetCarrier_01", u8::from(pets.owned >= 2), 1),
            };
            let available = !(8..=10).contains(&code) || super::super::companions::instinct_available(code - 7);
            let cost = if !available || rank == max || code == 13 { None } else { Some(match code { 6 | 7 => [15, 25, 35][usize::from(rank)], 11 => 40, _ => 30 }) };
            let now = trip(0.0, pets.leash, pets.paws, current);
            let next = trip(0.0, if code == 6 { rank.saturating_add(1) } else { pets.leash }, if code == 7 { rank.saturating_add(1) } else { pets.paws }, if (8..=10).contains(&code) { code - 7 } else { current });
            let parts = match code {
                13 => vec![(format!("A free companion follows every hero, fetching coins and experience up to {} yd beyond your pickup radius. It sweeps a quarter of your pickup radius (at least {} yd), resting {} sec between trips before upgrades. Chests and healing drops are left for you.", trip(0.0, 0, 0, 0).reach, trip(0.0, 0, 0, 0).grasp, trip(0.0, 0, 0, 0).rest), DIM)],
                6 => vec![("Fetch coins and experience up to ".into(), DIM), (comparison(now.reach, next.reach, " yd"), hud::card_growth::GAIN), (" beyond your pickup radius.".into(), DIM)],
                7 => vec![("Run ".into(), DIM), (comparison((now.speed / trip(0.0, 0, 0, 0).speed - 1.0) * 100.0, (next.speed / trip(0.0, 0, 0, 0).speed - 1.0) * 100.0, "%"), hud::card_growth::GAIN), (" faster and rest ".into(), DIM), (comparison(now.rest, next.rest, " sec"), hud::card_growth::GAIN), (" between trips.".into(), DIM)],
                8 => vec![("Prioritize coins, worth ".into(), DIM), (comparison((now.gold_bonus - 1.0) * 100.0, (next.gold_bonus - 1.0) * 100.0, "% more"), hud::card_growth::GAIN), (" when fetched.".into(), DIM)],
                9 => vec![("Prioritize experience wisps with ".into(), DIM), (comparison(now.wisp_reach / now.reach, next.wisp_reach / next.reach, "× reach"), hud::card_growth::GAIN), (".".into(), DIM)],
                10 => vec![("Unavailable: battleground runes are not implemented. Existing saved instincts are retained.".into(), DIM)],
                _ => vec![("Travel with ".into(), DIM), (comparison(f32::from(pets.owned.min(2)), 2.0, " companions"), hud::card_growth::GAIN), (", each with its own instinct.".into(), DIM)],
            };
            let pending = armory.reward_draft.codes().iter().filter(|&&value| value == code).count();
            grid.spawn((Node { min_width: Val::Px(0.0), padding: UiRect::all(Val::Px(8.0)), column_gap: Val::Px(8.0), border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(if rank > 0 { Color::srgb(0.14, 0.10, 0.035) } else { Color::srgb(0.025, 0.02, 0.012) }), BorderColor::all(FRAME))).with_children(|card| {
                card.spawn((super::talent_page::Backdrop(format!("Interface\\Icons\\{icon}.blp")), ImageNode::default(), Node { width: Val::Px(34.0), height: Val::Px(34.0), flex_shrink: 0.0, ..default() }));
                card.spawn(Node { flex_grow: 1.0, flex_basis: Val::Px(0.0), min_width: Val::Px(0.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() }).with_children(|body| {
                    body.spawn(Node { flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: Val::Px(8.0), row_gap: Val::Px(3.0), ..default() }).with_children(|heading| {
                        text(heading, font, name, 15.0, GOLD);
                        heading.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|pips| {
                            for n in 0..max { pips.spawn((Node { width: Val::Px(7.0), height: Val::Px(7.0), border_radius: BorderRadius::MAX, border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor::all(FRAME), BackgroundColor(if n < rank { GOLD } else { Color::BLACK }))); }
                        });
                    });
                    body.spawn(hud::text_bundle(font, "", 12.0, DIM)).with_children(|line| {
                        for (part, color) in parts { line.spawn((TextSpan::new(part), TextColor(color), TextFont { font: font.clone(), font_size: 12.0, ..default() })); }
                    });
                    if (8..=10).contains(&code) {
                        text(body, font, target.map_or_else(|| "Requires a companion".into(), |slot| format!("{} companion · {}", if slot == 0 { "First" } else { "Second" }, if rank == max { "instinct learned".into() } else if current == 0 { "learn instinct".into() } else { format!("replaces {}", super::instinct_name(current)) })), 12.0, DIM);
                    }
                });
                card.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexEnd, width: Val::Px(78.0), flex_shrink: 0.0, row_gap: Val::Px(4.0), ..default() }).with_children(|actions| {
                    text(actions, font, cost.map_or_else(|| if !available { "Unavailable" } else if code == 13 { "Included" } else if (8..=10).contains(&code) { "Learned" } else { "Max rank" }.into(), |value| format!("{value} points")), 12.0, DIM);
                    actions.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|buttons| {
                        perk_step(buttons, font, "−", (pending > 0).then_some(Act::RemoveReward(code)));
                        perk_step(buttons, font, "+", cost.filter(|&value| account.unspent() >= value && (!(8..=10).contains(&code) || target.is_some())).map(|_| Act::Reward(code)));
                    });
                    if pending > 0 { text(actions, font, format!("{pending} pending"), 11.0, hud::card_growth::GAIN); }
                    if cost.is_some_and(|value| account.unspent() < value) { text(actions, font, "Need points", 11.0, DIM); }
                });
            });
        }
    });
    super::typography::section(panel, font, "Looks", 18.0, GOLD);
}

/// Cosmetic choices act on the selected slot; they never choose an instinct destination.
pub(super) fn looks(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let worn = account.pets.look[usize::from(armory.pet_slot)];
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::minmax(
                GridTrackRepetition::AutoFit,
                MinTrackSizingFunction::Px(200.0),
                MaxTrackSizingFunction::Fraction(1.0),
            )],
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            for (index, look) in super::super::companions::LOOKS.iter().enumerate() {
                let selected = worn == index as u8;
                let unlocked = account.look_unlocked(index as u8);
                let node = Node {
                    min_width: Val::Px(0.0),
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                };
                let content = |card: &mut ChildSpawnerCommands| {
                    card.spawn((
                        super::talent_page::Backdrop(format!(
                            "Interface\\Icons\\{}.blp",
                            look.icon
                        )),
                        ImageNode::default(),
                        Node {
                            width: Val::Px(30.0),
                            height: Val::Px(30.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                    card.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        flex_basis: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|label| {
                        text(
                            label,
                            font,
                            look.name,
                            13.0,
                            if unlocked { GOLD } else { DIM },
                        );
                        text(
                            label,
                            font,
                            if selected {
                                "Worn"
                            } else if unlocked {
                                "Wear"
                            } else {
                                look.locked_label
                            },
                            12.0,
                            if selected {
                                hud::card_growth::GAIN
                            } else {
                                DIM
                            },
                        );
                    });
                };
                if unlocked {
                    super::controls::content_button(
                        grid,
                        Act::WearLook(index as u8),
                        selected,
                        node,
                        content,
                    );
                } else {
                    super::controls::disabled_content(grid, node, content);
                }
            }
        });
}

/// Account cosmetics use the same illustrated selection treatment as Companion looks.
fn tabards(panel: &mut ChildSpawnerCommands, font: &Handle<Font>, account: &Account) {
    text(panel, font, "Tabards are shared by every hero. They change your appearance and grant no stats. Select a look to wear it immediately.", 13.0, DIM);
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::minmax(
                GridTrackRepetition::AutoFit,
                MinTrackSizingFunction::Px(200.0),
                MaxTrackSizingFunction::Fraction(1.0),
            )],
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            for (index, name) in std::iter::once("None")
                .chain(super::campaign::TABARDS.iter().map(|tabard| tabard.name))
                .enumerate()
            {
                let selected = account.tabard == index as u8;
                super::controls::content_button(
                    grid,
                    Act::Tabard(index as u8),
                    selected,
                    Node {
                        min_width: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    |card| {
                        if index != 0 {
                            card.spawn((
                                super::talent_page::Backdrop(
                                    "Interface\\Icons\\INV_Shirt_GuildTabard_01.blp".into(),
                                ),
                                ImageNode::default(),
                                Node {
                                    width: Val::Px(34.0),
                                    height: Val::Px(34.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ));
                        }
                        card.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            flex_basis: Val::Px(0.0),
                            min_width: Val::Px(0.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|label| {
                            text(label, font, name, 16.0, GOLD);
                            text(
                                label,
                                font,
                                if index == 0 {
                                    "Show your equipped chest armor."
                                } else {
                                    "Wear a tabard over your chest armor."
                                },
                                12.0,
                                DIM,
                            );
                            text(
                                label,
                                font,
                                if selected {
                                    if index == 0 {
                                        "No tabard worn"
                                    } else {
                                        "Worn"
                                    }
                                } else if index == 0 {
                                    "Remove tabard"
                                } else {
                                    "Wear"
                                },
                                12.0,
                                if selected {
                                    hud::card_growth::GAIN
                                } else {
                                    DIM
                                },
                            );
                        });
                    },
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_actions_remain_reachable_when_sidebar_is_clipped() {
        let mut account = crate::survivors::account::blank();
        for class in 0..9 {
            account.grant_zone(class, 0);
        }
        account.grant_new();
        let points = account.unspent();
        let mut armory = Armory::default();
        armory.reward_draft.stage(&account, 0).unwrap();
        let mut app = App::new();
        app.insert_resource(armory)
            .add_systems(Update, keep_actions_visible);
        app.world_mut().spawn((
            ScrollCategory(0),
            ComputedNode {
                size: Vec2::new(600., 400.),
                ..default()
            },
            UiGlobalTransform::from_xy(300., 300.),
        ));
        let sidebar = app
            .world_mut()
            .spawn((
                SidebarActions,
                ComputedNode {
                    size: Vec2::new(200., 40.),
                    ..default()
                },
                UiGlobalTransform::from_xy(150., 140.),
            ))
            .id();
        let bar = app
            .world_mut()
            .spawn((PendingActions, Node::default()))
            .id();
        for (y, expected) in [
            (140., Display::None),
            (90., Display::Flex),
            (110., Display::Flex),
            (490., Display::Flex),
            (460., Display::None),
        ] {
            app.world_mut()
                .entity_mut(sidebar)
                .insert(UiGlobalTransform::from_xy(150., y));
            app.update();
            assert_eq!(app.world().get::<Node>(bar).unwrap().display, expected);
            assert_eq!(
                account.unspent(),
                points,
                "visibility must not commit a draft"
            );
        }
        app.world_mut()
            .entity_mut(sidebar)
            .insert(UiGlobalTransform::from_xy(150., 600.));
        app.world_mut()
            .resource_mut::<Armory>()
            .reward_draft
            .clear();
        app.update();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().display,
            Display::None,
            "Undo removes the pending bar even with the rail offscreen"
        );
    }

    #[test]
    fn pending_bar_is_outside_the_catalog_scroll_and_uses_the_same_transaction_actions() {
        let mut account = crate::survivors::account::blank();
        for class in 0..9 {
            account.grant_zone(class, 0);
        }
        account.grant_new();
        let mut armory = Armory::default();
        armory.reward_draft.stage(&account, 0).unwrap();
        let mut world = World::new();
        let root = world
            .commands()
            .spawn(Node::default())
            .with_children(|ui| build(ui, &Handle::default(), &armory, &account))
            .id();
        world.flush();
        let bar = world
            .query_filtered::<Entity, With<PendingActions>>()
            .single(&world)
            .unwrap();
        assert_eq!(world.get::<ChildOf>(bar).unwrap().parent(), root);
        let actions: Vec<_> = world
            .get::<Children>(bar)
            .unwrap()
            .iter()
            .filter_map(|child| world.get::<Act>(child))
            .collect();
        assert_eq!(
            actions
                .iter()
                .filter(|a| matches!(a, Act::ConfirmRewards))
                .count(),
            1
        );
        assert_eq!(
            actions
                .iter()
                .filter(|a| matches!(a, Act::UndoRewards))
                .count(),
            1
        );
        assert!(world
            .get::<super::super::collection::CollectionScroll>(bar)
            .is_none());
        assert_eq!(
            world.get::<Node>(bar).unwrap().overflow,
            Overflow::visible()
        );
    }

    #[test]
    fn category_offsets_follow_the_rendered_page_and_reject_invalid_positions() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .add_systems(Update, remember_scroll);
        let old = app
            .world_mut()
            .spawn((ScrollCategory(0), ScrollPosition(Vec2::new(0., 280.))))
            .id();
        // A page action can change selection before the old rendered surface is replaced.
        app.world_mut().resource_mut::<Armory>().reward_page = 1;
        app.update();
        assert_eq!(
            app.world().resource::<Armory>().reward_scroll,
            [Vec2::new(0., 280.), Vec2::ZERO, Vec2::ZERO]
        );
        app.world_mut().despawn(old);
        let current = app
            .world_mut()
            .spawn((ScrollCategory(1), ScrollPosition(Vec2::new(0., 90.))))
            .id();
        app.update();
        app.world_mut()
            .entity_mut(current)
            .insert(ScrollPosition(Vec2::splat(f32::NAN)));
        app.update();
        let memory = app.world().resource::<Armory>().reward_scroll;
        assert_eq!(
            memory,
            [Vec2::new(0., 280.), Vec2::new(0., 90.), Vec2::ZERO]
        );
    }
}
