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
            text(side, font, format!("{} of {} achievements", rewards::earned_count(account.earned_ids()), rewards::catalog_len()), 12.0, DIM);
            let pending = !armory.reward_draft.is_empty();
            if pending {
                text(side, font, format!("{} pending {}", armory.reward_draft.codes().len(), if armory.reward_draft.codes().len() == 1 { "change" } else { "changes" }), 13.0, hud::card_growth::GAIN);
            }
            side.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), ..default() }).with_children(|buttons| {
                for (label, act, selected) in [("Confirm", Act::ConfirmRewards, true), ("Undo", Act::UndoRewards, false)] {
                    let node = Node { min_height: Val::Px(24.0), padding: UiRect::horizontal(Val::Px(7.0)), justify_content: JustifyContent::Center, ..default() };
                    if pending { super::controls::button(buttons, font, label, act, selected, node); }
                    else { super::controls::disabled(buttons, font, label, node); }
                }
            });
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
    panel.spawn(Node { width: Val::Percent(100.0), flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), ..default() }).with_children(|grid| {
        for code in [13, 6, 7, 8, 9, 10, 11] {
            let pets = &account.pets;
            let target = account.instinct_target();
            let current = target.map_or(0, |slot| pets.instinct[slot]);
            let (name, icon, rank, max) = match code {
                13 => ("Companion", "INV_Box_PetCarrier_01", u8::from(pets.owned > 0), 1),
                6 => ("Long Leash", "INV_Misc_Key_09", pets.leash.min(3), 3),
                7 => ("Quick Paws", "Ability_Druid_Dash", pets.paws.min(3), 3),
                8 => ("Gold-sniffer", "INV_Misc_Coin_01", u8::from(current == 1), 1),
                9 => ("Wisp-herder", "Spell_Nature_WispSplode", u8::from(current == 2), 1),
                10 => ("Rune-hound", "INV_Misc_Rune_01", u8::from(current == 3), 1),
                _ => ("Second Companion", "INV_Box_PetCarrier_01", u8::from(pets.owned >= 2), 1),
            };
            let cost = if rank == max || code == 13 { None } else { Some(match code { 6 | 7 => [15, 25, 35][usize::from(rank)], 11 => 40, _ => 30 }) };
            let now = trip(0.0, pets.leash, pets.paws, current);
            let next = trip(0.0, if code == 6 { rank.saturating_add(1) } else { pets.leash }, if code == 7 { rank.saturating_add(1) } else { pets.paws }, if (8..=10).contains(&code) { code - 7 } else { current });
            let parts = match code {
                13 => vec![(format!("A free companion follows every hero, fetching coins and experience up to {} yd beyond your pickup radius. It sweeps a quarter of your pickup radius (at least {} yd), resting {} sec between trips before upgrades. Chests and healing drops are left for you.", trip(0.0, 0, 0, 0).reach, trip(0.0, 0, 0, 0).grasp, trip(0.0, 0, 0, 0).rest), DIM)],
                6 => vec![("Fetch coins and experience up to ".into(), DIM), (comparison(now.reach, next.reach, " yd"), hud::card_growth::GAIN), (" beyond your pickup radius.".into(), DIM)],
                7 => vec![("Run ".into(), DIM), (comparison((now.speed / trip(0.0, 0, 0, 0).speed - 1.0) * 100.0, (next.speed / trip(0.0, 0, 0, 0).speed - 1.0) * 100.0, "%"), hud::card_growth::GAIN), (" faster and rest ".into(), DIM), (comparison(now.rest, next.rest, " sec"), hud::card_growth::GAIN), (" between trips.".into(), DIM)],
                8 => vec![("Prioritize coins, worth ".into(), DIM), (comparison((now.gold_bonus - 1.0) * 100.0, (next.gold_bonus - 1.0) * 100.0, "% more"), hud::card_growth::GAIN), (" when fetched.".into(), DIM)],
                9 => vec![("Prioritize experience wisps with ".into(), DIM), (comparison(now.wisp_reach / now.reach, next.wisp_reach / next.reach, "× reach"), hud::card_growth::GAIN), (".".into(), DIM)],
                10 => vec![("Prioritize and fetch battleground runes. Rune spawning is not yet available.".into(), DIM)],
                _ => vec![("Travel with ".into(), DIM), (comparison(f32::from(pets.owned.min(2)), 2.0, " companions"), hud::card_growth::GAIN), (", each with its own instinct.".into(), DIM)],
            };
            grid.spawn((Node { flex_basis: Val::Percent(48.0), flex_grow: 1.0, min_width: Val::Px(0.0), padding: UiRect::all(Val::Px(10.0)), column_gap: Val::Px(10.0), border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.035, 0.027, 0.014)), BorderColor::all(FRAME))).with_children(|card| {
                card.spawn((super::talent_page::Backdrop(format!("Interface\\Icons\\{icon}.blp")), ImageNode::default(), Node { width: Val::Px(34.0), height: Val::Px(34.0), flex_shrink: 0.0, ..default() }));
                card.spawn(Node { flex_grow: 1.0, flex_basis: Val::Px(0.0), min_width: Val::Px(0.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), ..default() }).with_children(|body| {
                    text(body, font, name, 16.0, GOLD);
                    body.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|pips| {
                        for n in 0..max { pips.spawn((Node { width: Val::Px(7.0), height: Val::Px(7.0), border_radius: BorderRadius::MAX, border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor::all(FRAME), BackgroundColor(if n < rank { GOLD } else { Color::BLACK }))); }
                    });
                    body.spawn(hud::text_bundle(font, "", 12.0, DIM)).with_children(|line| {
                        for (part, color) in parts { line.spawn((TextSpan::new(part), TextColor(color), TextFont { font: font.clone(), font_size: 12.0, ..default() })); }
                    });
                    if (8..=10).contains(&code) {
                        text(body, font, target.map_or_else(|| "Requires a companion".into(), |slot| format!("{} companion · {}", if slot == 0 { "First" } else { "Second" }, if rank == max { "instinct learned".into() } else if current == 0 { "learn instinct".into() } else { format!("replaces {}", super::instinct_name(current)) })), 12.0, DIM);
                    }
                    remove_pending(body, font, armory, code);
                    if let Some(cost) = cost {
                        if !(8..=10).contains(&code) || target.is_some() {
                            body.spawn(Node::default()).with_children(|buy| chip(buy, font, &format!("Buy · {cost} points"), Act::Reward(code), false));
                            if account.unspent() < cost { text(body, font, "Not enough points", 12.0, DIM); }
                        }
                    } else { text(body, font, if code == 13 { if pets.owned > 0 { "Included" } else { "Unavailable" } } else if (8..=10).contains(&code) { "Learned" } else { "Max rank" }, 12.0, GOLD); }
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
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            for (index, look) in super::super::companions::LOOKS.iter().enumerate() {
                let selected = worn == index as u8;
                let unlocked = account.look_unlocked(index as u8);
                grid.spawn((
                    Button,
                    Act::WearLook(index as u8),
                    Node {
                        flex_basis: Val::Px(180.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(if selected {
                        Color::srgb(0.14, 0.10, 0.035)
                    } else {
                        Color::srgb(0.025, 0.02, 0.012)
                    }),
                    BorderColor::all(if selected { GOLD } else { FRAME }),
                ))
                .with_children(|card| {
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
                });
            }
        });
}

/// Account cosmetics use the same illustrated selection treatment as Companion looks.
fn tabards(panel: &mut ChildSpawnerCommands, font: &Handle<Font>, account: &Account) {
    text(panel, font, "Tabards are shared by every hero. They change your appearance and grant no stats. Select a look to wear it immediately.", 13.0, DIM);
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_wrap: FlexWrap::Wrap,
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
                grid.spawn((
                    Button,
                    Act::Tabard(index as u8),
                    Node {
                        flex_basis: Val::Px(180.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        column_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(if selected {
                        Color::srgb(0.14, 0.10, 0.035)
                    } else {
                        Color::srgb(0.025, 0.02, 0.012)
                    }),
                    BorderColor::all(if selected { GOLD } else { FRAME }),
                ))
                .with_children(|card| {
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
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

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
