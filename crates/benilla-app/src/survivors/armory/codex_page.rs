//! The discovered catalog and independently scrollable selected-entry details.

use super::{campaign, codex_jewel, codex_legend, collection, data, hud, picked_class};
use super::{Account, Act, Armory, Selection, CLASSES, GOLD};
use bevy::prelude::*;
use survivors_core::layout::Layout;

const DIM: Color = Color::srgb(0.65, 0.60, 0.44);
const GREEN: Color = Color::srgb(0.45, 0.9, 0.4);

fn label(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, value, size, color));
}

pub(super) fn build(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        min_height: Val::Px(0.0),
        column_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|panes| {
        collection::column(panes, 74.0, layout, |catalog| {
            label(catalog, font, "Codex", layout.title_size, GOLD);
            label(
                catalog,
                font,
                "Legendary amulets · each class has one for every spell",
                layout.body_size,
                DIM,
            );
            for (class_i, class) in CLASSES.iter().enumerate() {
                let (found, total) = account.legendary_progress(class_i);
                catalog
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_shrink: 0.0,
                        column_gap: Val::Px(layout.gap),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(Node {
                            width: Val::Percent(22.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|title| {
                            let color = Color::srgb_u8(
                                (class.color >> 16) as u8,
                                (class.color >> 8) as u8,
                                class.color as u8,
                            );
                            label(title, font, class.name, layout.body_size, color);
                            label(
                                title,
                                font,
                                format!(
                                    "{found}/{total}{}",
                                    if picked_class(account, selection) == class_i {
                                        " · your hero"
                                    } else {
                                        ""
                                    }
                                ),
                                layout.body_size - 2.0,
                                DIM,
                            );
                        });
                        row.spawn(Node {
                            flex_grow: 1.0,
                            flex_basis: Val::Px(0.0),
                            min_width: Val::Px(0.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(3.0),
                            row_gap: Val::Px(3.0),
                            ..default()
                        })
                        .with_children(|icons| {
                            for (slot, key) in class.ability_keys().iter().enumerate() {
                                let seen = account.classes.get(class_i).is_some_and(|p| {
                                    p.seen.contains(&campaign::legendary_amulet(key))
                                });
                                let dim = if seen { 1.0 } else { 0.35 };
                                icons.spawn((
                                    Button,
                                    Act::Codex(codex_legend(class_i, slot)),
                                    Node {
                                        width: Val::Px(layout.body_size * 2.5),
                                        height: Val::Px(layout.body_size * 2.5),
                                        flex_shrink: 0.0,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor::all(if seen { GOLD } else { DIM }),
                                    ImageNode {
                                        color: Color::srgba(1.0, 1.0, 1.0, dim),
                                        ..default()
                                    },
                                    hud::SpellIcon(data::ability(key).spell),
                                    hud::IconDim(dim),
                                ));
                            }
                        });
                    });
            }
            label(
                catalog,
                font,
                "All classes · offered to every hero",
                layout.body_size,
                GOLD,
            );
            for (kind, title, names) in [
                (2, "Rings", campaign::RING_NAMES.as_slice()),
                (3, "Trinkets", campaign::TRINKET_NAMES.as_slice()),
                (4, "Amulets", campaign::AMULET_NAMES.as_slice()),
            ] {
                let (found, total) = account.shared_jewel_progress(kind);
                label(
                    catalog,
                    font,
                    format!("{title} · {found}/{total}"),
                    layout.body_size,
                    if found == total { GREEN } else { GOLD },
                );
                catalog
                    .spawn(Node {
                        flex_shrink: 0.0,
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|row| {
                        for (index, name) in names.iter().enumerate() {
                            let seen = account.shared_jewel_seen(kind, index);
                            let tint = if seen {
                                Color::WHITE
                            } else {
                                Color::srgba(0.3, 0.3, 0.3, 0.65)
                            };
                            let mut cell = row.spawn((
                                Button,
                                Act::Codex(codex_jewel(kind, index)),
                                Node {
                                    width: Val::Px(layout.body_size * 3.0),
                                    height: Val::Px(layout.body_size * 3.0),
                                    margin: UiRect::all(Val::Px(2.0)),
                                    flex_shrink: 0.0,
                                    border: UiRect::all(Val::Px(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor::all(if seen { GOLD } else { DIM }),
                            ));
                            if let Some(display) = super::super::jewel_icons::display(name) {
                                cell.insert((
                                    super::super::gear_icons::GearIcon(display, tint),
                                    ImageNode {
                                        color: Color::NONE,
                                        ..default()
                                    },
                                ));
                            } else {
                                cell.with_children(|fallback| {
                                    label(fallback, font, "?", layout.title_size, DIM)
                                });
                            }
                        }
                    });
            }
            let (found, total) = discovery_total(account);
            label(
                catalog,
                font,
                format!("Discovered across all heroes · {found}/{total}"),
                layout.body_size,
                GOLD,
            );
            catalog
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(5.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.10, 0.06)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0 * found as f32 / total.max(1) as f32),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(GOLD),
                    ));
                });
        });
        collection::column(panes, 26.0, layout, |details| {
            label(details, font, "Details", layout.title_size, GOLD);
            details.spawn((
                DetailText,
                hud::text_bundle(font, detail_text(armory, account), layout.body_size, DIM),
            ));
        });
    });
}

const INTRO: &str = "Every ring, trinket and amulet offered to any hero is remembered, even if you do not take it. Dim icons have not been offered yet. Select an entry to read it.\n\nShared jewelry lasts one run. Legendary amulets belong to a class and enhance one spell.";

#[derive(Component)]
pub(in crate::survivors) struct DetailText;

fn detail_text(armory: &Armory, account: &Account) -> String {
    armory
        .codex_entry
        .map(|id| {
            let text = super::codex_text(account, id);
            let kind = (id >> 12) as u8;
            if (2..=4).contains(&kind) {
                format!(
                    "{text}\n\n{}",
                    if account.shared_jewel_seen(kind, usize::from(id & 0x0fff)) {
                        "Previously offered"
                    } else {
                        "Not offered yet"
                    }
                )
            } else {
                text
            }
        })
        .unwrap_or_else(|| INTRO.into())
}

/// Selection changes text only: keep the catalog entities and their scroll offset alive.
pub(in crate::survivors) fn refresh_details(
    armory: Res<Armory>,
    account: Res<Account>,
    mut details: Query<&mut Text, With<DetailText>>,
) {
    if details.is_empty() || (!armory.is_changed() && !account.is_changed()) {
        return;
    }
    let value = detail_text(&armory, &account);
    for mut text in &mut details {
        if text.0 != value {
            text.0.clone_from(&value);
        }
    }
}

fn discovery_total(account: &Account) -> (usize, usize) {
    (0..CLASSES.len())
        .map(|class| account.legendary_progress(class))
        .chain((2..=4).map(|kind| account.shared_jewel_progress(kind)))
        .fold((0, 0), |(found, total), (f, t)| (found + f, total + t))
}
