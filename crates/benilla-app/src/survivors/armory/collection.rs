//! Three collection columns share the same loadout used by the run's stat sheet.

use super::super::equipment::Look;
use super::{campaign, chip, hud, picked_class, Account, Act, Armory, Selection, GOLD};
use crate::glue::art::GlueArt;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use survivors_core::layout::Layout;

const DIM: Color = Color::srgb(0.65, 0.60, 0.44);
const BONUS: Color = Color::srgb(0.45, 0.9, 0.4);

#[derive(Component)]
pub(in crate::survivors) struct CollectionScroll;

#[derive(Component)]
pub(in crate::survivors) enum CompactScroll {
    Collection,
    Drops,
    Character,
    Starter,
}

pub(in crate::survivors) fn remember_scroll(
    mut armory: ResMut<Armory>,
    surfaces: Query<(&ScrollPosition, &CompactScroll)>,
) {
    for (position, page) in &surfaces {
        if position.0.is_finite() {
            let saved = match page {
                CompactScroll::Collection => &mut armory.collection_scroll,
                CompactScroll::Drops => &mut armory.drop_scroll,
                CompactScroll::Character => &mut armory.character_scroll,
                CompactScroll::Starter => &mut armory.starter_scroll,
            };
            if *saved != position.0 {
                *saved = position.0;
            }
        }
    }
}

pub(super) fn section(
    ui: &mut ChildSpawnerCommands,
    layout: &Layout,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            padding: UiRect::all(Val::Px(layout.padding)),
            row_gap: Val::Px(layout.gap * 0.5),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.028, 0.015, 0.96)),
        BorderColor::all(Color::srgb(0.5, 0.4, 0.14)),
    ))
    .with_children(body);
}

#[cfg(test)]
mod scroll_restore_tests {
    use super::*;

    #[test]
    fn compact_pages_remember_independent_scroll_positions() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .add_systems(Update, remember_scroll);
        app.world_mut().spawn((
            CompactScroll::Collection,
            ScrollPosition(Vec2::new(0.0, 120.0)),
        ));
        let drops = app
            .world_mut()
            .spawn((CompactScroll::Drops, ScrollPosition(Vec2::new(0.0, 460.0))))
            .id();
        app.world_mut().spawn((
            CompactScroll::Character,
            ScrollPosition(Vec2::new(0.0, 730.0)),
        ));
        app.update();
        assert_eq!(app.world().resource::<Armory>().character_scroll.y, 730.0);
        assert_eq!(app.world().resource::<Armory>().collection_scroll.y, 120.0);
        assert_eq!(app.world().resource::<Armory>().drop_scroll.y, 460.0);
        app.world_mut()
            .get_mut::<ScrollPosition>(drops)
            .unwrap()
            .0
            .y = f32::NAN;
        app.update();
        assert_eq!(app.world().resource::<Armory>().drop_scroll.y, 460.0);
    }

    #[test]
    fn restored_scroll_waits_for_layout_then_clamps_to_actual_content() {
        let mut app = App::new();
        app.add_message::<MouseWheel>().add_systems(Update, scroll);
        let entity = app
            .world_mut()
            .spawn((
                CollectionScroll,
                RelativeCursorPosition::default(),
                ComputedNode::default(),
                ScrollPosition(Vec2::new(0., 250.)),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(entity).unwrap().0.y, 250.);
        {
            let mut node = app.world_mut().get_mut::<ComputedNode>(entity).unwrap();
            node.size = Vec2::new(100., 100.);
            node.content_size = Vec2::new(100., 200.);
            node.inverse_scale_factor = 1.;
        }
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(entity).unwrap().0.y, 100.);
    }
}

pub(in crate::survivors) fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    mut panels: Query<
        (&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition),
        With<CollectionScroll>,
    >,
) {
    let dy: f32 = wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y * 24.0,
            MouseScrollUnit::Pixel => event.y,
        })
        .sum();
    for (cursor, node, mut pos) in &mut panels {
        // Newly rebuilt or hidden nodes have no layout yet. Do not erase a restored offset.
        if node.size.y <= 0.0 {
            continue;
        }
        let max = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.0);
        // Relative position also works over the child buttons, which consume Interaction.
        let hovered = cursor
            .normalized
            .is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5);
        let target = pos.0.y - if hovered { dy } else { 0.0 };
        pos.0.y = target.clamp(0.0, max);
    }
}

fn text(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, value, size, color));
}

pub(super) fn column(
    ui: &mut ChildSpawnerCommands,
    weight: f32,
    layout: &Layout,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            flex_grow: weight,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.028, 0.015, 0.96)),
        BorderColor::all(Color::srgb(0.5, 0.4, 0.14)),
    ))
    .with_children(|frame| {
        frame
            .spawn((
                CollectionScroll,
                RelativeCursorPosition::default(),
                ScrollPosition::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(layout.padding)),
                    row_gap: Val::Px(layout.gap * 0.5),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(body);
    });
}

pub(super) fn build(
    panel: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
    compact: bool,
) {
    if compact {
        panel
            .spawn((
                CollectionScroll,
                CompactScroll::Collection,
                RelativeCursorPosition::default(),
                ScrollPosition(armory.collection_scroll),
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(layout.gap),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|body| {
                section(body, layout, |body| {
                    grid(body, font, armory, account, selection, layout, true);
                    text(body, font, "Select a piece; its details follow the collection. E: worn · L: shown as a look", layout.body_size, DIM);
                });
                section(body, layout, |body| {
                    detail(body, font, armory, account, selection, layout)
                });
                section(body, layout, |body| {
                    super::character::body_controls(body, art, font, account, selection, layout);
                    equipment(body, font, armory, account, selection, layout);
                });
                section(body, layout, |body| {
                    missing(body, font, armory, account, selection, layout)
                });
            });
        return;
    }
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            column_gap: Val::Px(layout.gap),
            ..default()
        })
        .with_children(|row| {
            column(row, layout.left_width, layout, |body| {
                super::character::body_controls(body, art, font, account, selection, layout);
                equipment(body, font, armory, account, selection, layout)
            });
            column(
                row,
                100.0 - layout.left_width - layout.right_width,
                layout,
                |body| {
                    grid(body, font, armory, account, selection, layout, false);
                    missing(body, font, armory, account, selection, layout);
                    text(
                        body,
                        font,
                        "E: worn · L: shown as a look",
                        layout.body_size,
                        DIM,
                    );
                    let found = campaign::owned_pieces(&progress.pieces);
                    text(
                        body,
                        font,
                        format!("{found} / {} collected", campaign::PIECES),
                        layout.body_size,
                        GOLD,
                    );
                    body.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(5.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.10, 0.04)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Percent(100.0 * found as f32 / campaign::PIECES as f32),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(GOLD),
                        ));
                    });
                },
            );
            column(row, layout.right_width, layout, |body| {
                detail(body, font, armory, account, selection, layout)
            });
        });
}

pub(super) fn equipment(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    super::typography::heading(ui, font, "Equipment", layout.title_size, GOLD);
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    for (slot, label) in campaign::SLOTS
        .iter()
        .take(campaign::GRID_SLOTS)
        .enumerate()
    {
        let band = progress.equipment.worn[slot].map(usize::from);
        let name = band
            .and_then(|band| campaign::piece_name(class, slot, band))
            .unwrap_or("Empty");
        let color = band.map(super::door::band_color).unwrap_or(DIM);
        let index = band.unwrap_or(0) * campaign::GRID_SLOTS + slot;
        let selected = armory.piece == index;
        ui.spawn((
            Button,
            Act::InspectPiece(index),
            Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(layout.gap),
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(if selected {
                GOLD
            } else {
                Color::srgb(0.20, 0.17, 0.09)
            }),
            BackgroundColor(if selected {
                Color::srgb(0.15, 0.11, 0.045)
            } else {
                Color::NONE
            }),
        ))
        .with_children(|entry| {
            let size = layout.body_size * 2.4;
            if let Some(band) = band {
                super::super::gear_icons::spawn(entry, class, slot, band, size);
            } else {
                entry.spawn((
                    Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        flex_shrink: 0.0,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.20, 0.17, 0.09)),
                    BackgroundColor(Color::srgb(0.025, 0.02, 0.01)),
                ));
            }
            entry
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    ..default()
                })
                .with_children(|labels| {
                    text(labels, font, *label, layout.body_size - 2.0, DIM);
                    text(labels, font, name, layout.body_size, color);
                });
        });
    }
    super::typography::heading(ui, font, "Equipment bonuses", layout.title_size, GOLD);
    let worn = progress.equipment.worn_pieces(&progress.pieces);
    text(
        ui,
        font,
        super::door::bonus_line(campaign::equipped_bonus(&worn, None)),
        layout.body_size,
        BONUS,
    );
}

fn grid(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
    compact: bool,
) {
    super::typography::heading(ui, font, "Collection Log", layout.title_size, GOLD);
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    if !compact {
        ui.spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: if compact {
                    Val::Percent(100.0)
                } else {
                    Val::Px(layout.body_size * 7.0)
                },
                flex_shrink: 0.0,
                ..default()
            });
            for label in campaign::SLOTS.iter().take(campaign::GRID_SLOTS) {
                row.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    ..default()
                })
                .with_children(|cell| {
                    cell.spawn((
                        hud::centered_bundle(
                            font,
                            *label,
                            (layout.body_size * 0.75).min(10.0),
                            DIM,
                        ),
                        Node {
                            width: Val::Percent(100.0),
                            min_width: Val::Px(0.0),
                            ..default()
                        },
                    ));
                });
            }
        });
    }
    for band in 0..campaign::BANDS {
        let found = progress.pieces[band * campaign::GRID_SLOTS..(band + 1) * campaign::GRID_SLOTS]
            .iter()
            .filter(|n| **n > 0)
            .count();
        ui.spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(3.0),
            row_gap: Val::Px(6.0),
            flex_wrap: if compact {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: if compact {
                    Val::Percent(100.0)
                } else {
                    Val::Px(layout.body_size * 7.0)
                },
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|label| {
                text(
                    label,
                    font,
                    campaign::BANDS_NAMED[band],
                    layout.body_size,
                    super::door::band_color(band),
                );
                text(
                    label,
                    font,
                    format!("{found}/9"),
                    layout.body_size - 2.0,
                    DIM,
                );
                let mut zones = Vec::new();
                for drop in campaign::DROPS.iter().filter(|drop| drop.band == band) {
                    let name = campaign::ZONES[drop.zone].name;
                    if !zones.contains(&name) {
                        zones.push(name);
                    }
                }
                if !zones.is_empty() {
                    text(label, font, zones.join(" · "), layout.body_size - 3.0, DIM);
                }
            });
            for slot in 0..campaign::GRID_SLOTS {
                let index = band * campaign::GRID_SLOTS + slot;
                let owned = progress.pieces[index] > 0;
                let equipped = progress.equipment.worn[slot] == Some(band as u8);
                let shown = progress.equipment.looks[slot] == Look::Piece(band as u8);
                row.spawn((
                    Button,
                    Act::InspectPiece(index),
                    Node {
                        flex_grow: if compact { 0.0 } else { 1.0 },
                        flex_basis: if compact {
                            Val::Percent(10.0)
                        } else {
                            Val::Px(0.0)
                        },
                        min_width: Val::Px(if compact { 54.0 } else { 0.0 }),
                        height: Val::Px(layout.cell_height + if compact { 20.0 } else { 0.0 }),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(if armory.piece == index { 2.0 } else { 1.0 })),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(if armory.piece == index {
                        GOLD
                    } else if owned {
                        super::door::band_color(band)
                    } else {
                        Color::srgb(0.20, 0.18, 0.12)
                    }),
                    BackgroundColor(Color::srgb(0.07, 0.06, 0.04)),
                ))
                .with_children(|cell| {
                    cell.spawn(Node {
                        width: Val::Percent(100.0),
                        height: if compact {
                            Val::Px(layout.cell_height)
                        } else {
                            Val::Percent(100.0)
                        },
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_shrink: 0.0,
                        ..default()
                    })
                    .with_children(|cell| {
                        let has_icon = super::super::gear_icons::spawn_cell(
                            cell,
                            class,
                            slot,
                            band,
                            layout.cell_height * 0.86,
                            owned,
                        );
                        if !owned || !has_icon {
                            cell.spawn((
                                hud::text_bundle(
                                    font,
                                    if owned { "•" } else { "?" },
                                    layout.body_size,
                                    DIM,
                                ),
                                Node {
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                ZIndex(1),
                            ));
                        }
                        if equipped || shown {
                            let mark = match (equipped, shown) {
                                (true, true) => "EL",
                                (true, false) => "E",
                                _ => "L",
                            };
                            cell.spawn((
                                hud::text_bundle(font, mark, layout.body_size * 0.8, Color::BLACK),
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    padding: UiRect::horizontal(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(layout.body_size)),
                                    ..default()
                                },
                                BackgroundColor(GOLD),
                                ZIndex(2),
                            ))
                            .remove::<TextShadow>();
                        }
                    });
                    if compact {
                        cell.spawn(hud::centered_bundle(font, campaign::SLOTS[slot], 9.0, DIM));
                    }
                });
            }
        });
    }
}

pub(super) fn detail(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    let selected = armory.piece.min(campaign::PIECES - 1);
    let slot = selected % campaign::GRID_SLOTS;
    let band = selected / campaign::GRID_SLOTS;
    let name = campaign::piece_name(class, slot, band);
    let owned = progress.pieces[selected] > 0;
    super::super::gear_icons::spawn(ui, class, slot, band, layout.title_size * 3.0);
    text(
        ui,
        font,
        name.unwrap_or("Item content not yet available"),
        layout.title_size,
        super::door::band_color(band),
    );
    text(
        ui,
        font,
        format!(
            "{} · {}",
            campaign::BANDS_NAMED[band],
            campaign::SLOTS[slot]
        ),
        layout.body_size,
        DIM,
    );
    text(
        ui,
        font,
        if owned {
            "In your collection"
        } else {
            "Not collected"
        },
        layout.body_size,
        if owned { BONUS } else { DIM },
    );
    if name.is_some() {
        text(
            ui,
            font,
            super::door::bonus_line(campaign::piece_bonus(band, slot)),
            layout.body_size,
            BONUS,
        );
    }
    super::typography::heading(ui, font, "Drops from", layout.title_size, GOLD);
    let mut sources = 0;
    for drop in campaign::DROPS
        .iter()
        .filter(|d| d.band == band && d.slot == slot)
    {
        sources += 1;
        text(
            ui,
            font,
            format!(
                "{} · {} chance",
                drop.creature,
                super::drop_page::chance_label(drop, account)
            ),
            layout.body_size,
            GOLD,
        );
        text(
            ui,
            font,
            format!(
                "{} · {}:{:02}",
                campaign::ZONES[drop.zone].name,
                drop.at_secs / 60,
                drop.at_secs % 60
            ),
            layout.body_size,
            DIM,
        );
    }
    if sources == 0 {
        text(ui, font, "Source not yet available", layout.body_size, DIM);
    }
    if owned {
        let equipped = progress.equipment.worn[slot] == Some(band as u8);
        chip(
            ui,
            font,
            if equipped { "Unequip" } else { "Equip" },
            Act::WearPiece(slot, if equipped { None } else { Some(band as u8) }),
            equipped,
        );
    }
    text(
        ui,
        font,
        "Right-click a collected piece to equip or unequip it. Shift-right-click chooses its look.",
        layout.body_size,
        DIM,
    );
    text(
        ui,
        font,
        format!("Look · {}", campaign::SLOTS[slot]),
        layout.title_size,
        GOLD,
    );
    text(
        ui,
        font,
        "Only worn pieces add bonuses.",
        layout.body_size,
        DIM,
    );
    ui.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        ..default()
    })
    .with_children(|actions| {
        for (label, look) in [
            ("Worn piece", Look::Worn),
            ("This piece", Look::Piece(band as u8)),
            ("Hidden", Look::Hidden),
        ] {
            if owned || !matches!(look, Look::Piece(_)) {
                chip(
                    actions,
                    font,
                    label,
                    Act::ShowPiece(slot, look),
                    progress.equipment.looks[slot] == look,
                );
            }
        }
    });
}

fn missing(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    let (level, clears) = super::hero_gate(account, selection);
    let missing: Vec<_> = campaign::DROPS
        .iter()
        .filter(|drop| {
            campaign::zone_open(drop.zone, level, &clears)
                && campaign::piece_index(drop.band, drop.slot)
                    .is_some_and(|index| progress.pieces[index] == 0)
                && campaign::piece_name(class, drop.slot, drop.band).is_some()
        })
        .collect();
    text(
        ui,
        font,
        format!(
            "Still to find in your open zones · {} sources",
            missing.len()
        ),
        layout.body_size,
        GOLD,
    );
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(6.0),
        row_gap: Val::Px(4.0),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|list| {
        for drop in missing {
            let index = drop.band * campaign::GRID_SLOTS + drop.slot;
            list.spawn((
                Button,
                Act::InspectPiece(index),
                Node {
                    width: Val::Percent(48.0),
                    min_width: Val::Px(0.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(layout.gap),
                    padding: UiRect::all(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(if armory.piece == index {
                    GOLD
                } else {
                    Color::NONE
                }),
            ))
            .with_children(|entry| {
                let size = layout.body_size * 2.5;
                entry
                    .spawn(Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        flex_shrink: 0.0,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|icon| {
                        super::super::gear_icons::spawn_cell(
                            icon, class, drop.slot, drop.band, size, false,
                        );
                    });
                entry
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        min_width: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|labels| {
                        text(
                            labels,
                            font,
                            campaign::piece_name(class, drop.slot, drop.band).unwrap(),
                            layout.body_size,
                            super::door::band_color(drop.band),
                        );
                        text(
                            labels,
                            font,
                            format!(
                                "{} · {} · {}",
                                drop.creature,
                                campaign::ZONES[drop.zone].name,
                                super::drop_page::chance_label(drop, account)
                            ),
                            layout.body_size - 2.0,
                            DIM,
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
    fn empty_and_full_class_collections_build_without_using_the_legacy_off_hand() {
        for class in 0..super::super::CLASSES.len() {
            for (owned, compact) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut account = super::super::super::account::blank();
                if owned {
                    account.classes[class].pieces.fill(1);
                    account.classes[class].equipment.worn.fill(Some(0));
                }
                let selection = Selection {
                    class: Some(class),
                    ..default()
                };
                let mut world = World::new();
                world
                    .commands()
                    .spawn(Node::default())
                    .with_children(|panel| {
                        build(
                            panel,
                            &GlueArt::default(),
                            &Handle::default(),
                            &Armory::default(),
                            &account,
                            &selection,
                            &Layout::default(),
                            compact,
                        );
                    });
                world.flush();
                let actions: Vec<_> = world.query::<&Act>().iter(&world).collect();
                for action in &actions {
                    if let Act::InspectPiece(index) = action {
                        assert!(*index < campaign::PIECES);
                    }
                }
                assert!(actions
                    .iter()
                    .any(|act| matches!(act, Act::InspectPiece(53))));
                for index in 0..campaign::PIECES {
                    assert!(actions
                        .iter()
                        .any(|act| matches!(act, Act::InspectPiece(found) if *found == index)));
                }
                let panels = world.query::<&CollectionScroll>().iter(&world).count();
                assert_eq!(panels, if compact { 1 } else { 3 });
            }
        }
    }
}
