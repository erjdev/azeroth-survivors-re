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
    StarterPanel(usize),
    CollectionColumn(usize),
    DropColumn(usize),
    CharacterColumn(usize),
    CodexColumn(usize),
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
                CompactScroll::StarterPanel(index) => &mut armory.starter_panel_scroll[*index],
                CompactScroll::CollectionColumn(index) => &mut armory.collection_columns[*index],
                CompactScroll::DropColumn(index) => &mut armory.drop_columns[*index],
                CompactScroll::CharacterColumn(index) => &mut armory.character_columns[*index],
                CompactScroll::CodexColumn(index) => &mut armory.codex_columns[*index],
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
        app.world_mut().spawn((
            CompactScroll::Starter,
            ScrollPosition(Vec2::new(0.0, 900.0)),
        ));
        for index in 0..3 {
            app.world_mut().spawn((
                CompactScroll::StarterPanel(index),
                ScrollPosition(Vec2::new(0.0, 100.0 * (index + 1) as f32)),
            ));
        }
        app.update();
        assert_eq!(app.world().resource::<Armory>().starter_scroll.y, 900.0);
        assert_eq!(
            app.world()
                .resource::<Armory>()
                .starter_panel_scroll
                .map(|p| p.y),
            [100.0, 200.0, 300.0],
        );
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
    fn wide_columns_survive_rebuilds_without_overwriting_compact_offsets() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .add_systems(Update, remember_scroll);
        for (marker, y) in [
            (CompactScroll::CollectionColumn(0), 80.0),
            (CompactScroll::CollectionColumn(1), 140.0),
            (CompactScroll::CollectionColumn(2), 200.0),
            (CompactScroll::DropColumn(3), 260.0),
            (CompactScroll::CharacterColumn(2), 320.0),
            (CompactScroll::CodexColumn(1), 380.0),
            (CompactScroll::Collection, 900.0),
        ] {
            app.world_mut()
                .spawn((marker, ScrollPosition(Vec2::new(0., y))));
        }
        app.update();
        let armory = app.world().resource::<Armory>();
        assert_eq!(armory.collection_columns.map(|p| p.y), [80., 140., 200.]);
        assert_eq!(armory.drop_columns.map(|p| p.y), [0., 0., 0., 260.]);
        assert_eq!(armory.character_columns.map(|p| p.y), [0., 0., 320.]);
        assert_eq!(armory.codex_columns.map(|p| p.y), [0., 380.]);
        assert_eq!(armory.collection_scroll.y, 900.);
        let saved = armory.collection_columns[1];
        app.world_mut().clear_entities();
        {
            let mut commands = app.world_mut().commands();
            commands.spawn_empty().with_children(|root| {
                column(
                    root,
                    1.,
                    &Layout::default(),
                    (CompactScroll::CollectionColumn(1), saved),
                    |_| {},
                );
            });
        }
        app.world_mut().flush();
        app.update();
        let position = app
            .world_mut()
            .query_filtered::<&ScrollPosition, With<CollectionScroll>>()
            .single(app.world())
            .unwrap();
        assert_eq!(position.0, saved);
        assert_eq!(app.world().resource::<Armory>().collection_scroll.y, 900.);
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
    memory: (CompactScroll, Vec2),
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            flex_grow: weight,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            min_height: Val::Px(0.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.028, 0.015, 0.96)),
        BorderColor::all(Color::srgb(0.5, 0.4, 0.14)),
    ))
    .with_children(|frame| {
        let viewport = frame
            .spawn((
                CollectionScroll,
                RelativeCursorPosition::default(),
                memory.0,
                ScrollPosition(memory.1),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect {
                        right: Val::Px(layout.padding.max(16.0)),
                        ..UiRect::all(Val::Px(layout.padding))
                    },
                    row_gap: Val::Px(layout.gap * 0.5),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(body)
            .id();
        super::scrollbar::spawn(frame, viewport);
    });
}

/// Keep the scrollbar outside the moving content and retain each page's offset.
pub(super) fn compact_column(
    ui: &mut ChildSpawnerCommands,
    marker: CompactScroll,
    offset: Vec2,
    layout: &Layout,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        flex_basis: Val::Px(0.0),
        min_height: Val::Px(0.0),
        ..default()
    })
    .with_children(|frame| {
        let viewport = frame
            .spawn((
                CollectionScroll,
                marker,
                RelativeCursorPosition::default(),
                ScrollPosition(offset),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::right(Val::Px(16.0)),
                    row_gap: Val::Px(layout.gap),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(body)
            .id();
        super::scrollbar::spawn(frame, viewport);
    });
}

pub(super) fn build(
    strings: &crate::survivors::localization::Strings,
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
        compact_column(
            panel,
            CompactScroll::Collection,
            armory.collection_scroll,
            layout,
            |body| {
                section(body, layout, |body| {
                    grid(body, font, armory, account, selection, layout, true);
                    text(body, font, "Select a piece; its details follow the collection. E: worn · L: shown as a look", layout.body_size, DIM);
                });
                section(body, layout, |body| {
                    detail(strings, body, font, armory, account, selection, layout)
                });
                section(body, layout, |body| {
                    super::character::body_controls(body, art, font, account, selection, layout);
                    equipment(strings, body, font, armory, account, selection, layout);
                });
                section(body, layout, |body| {
                    missing(body, font, armory, account, selection, layout)
                });
            },
        );
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
            column(
                row,
                layout.left_width,
                layout,
                (
                    CompactScroll::CollectionColumn(0),
                    armory.collection_columns[0],
                ),
                |body| {
                    super::character::body_controls(body, art, font, account, selection, layout);
                    equipment(strings, body, font, armory, account, selection, layout)
                },
            );
            column(
                row,
                100.0 - layout.left_width - layout.right_width,
                layout,
                (
                    CompactScroll::CollectionColumn(1),
                    armory.collection_columns[1],
                ),
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
                        format!("Common–Legendary: {found} / {} collected", campaign::PIECES),
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
            column(
                row,
                layout.right_width,
                layout,
                (
                    CompactScroll::CollectionColumn(2),
                    armory.collection_columns[2],
                ),
                |body| detail(strings, body, font, armory, account, selection, layout),
            );
        });
}

pub(super) fn equipment(
    strings: &crate::survivors::localization::Strings,
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    super::typography::section(ui, font, "Equipment", layout.title_size, GOLD);
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    for (slot, label) in campaign::SLOTS
        .iter()
        .take(campaign::GRID_SLOTS)
        .enumerate()
    {
        let band = progress.equipment.worn[slot].map(usize::from);
        let resolved =
            progress
                .equipment
                .worn(class, &progress.pieces, &account.supplemental, slot);
        let name = resolved.as_ref().map(|piece| piece.name).unwrap_or("Empty");
        let key = progress.equipment.supplemental_worn()[slot].as_ref();
        let color = band
            .or_else(|| {
                key.and_then(|key| survivors_core::supplemental_gear::by_key(key))
                    .filter(|piece| piece.tier == survivors_core::gear_tier::GearTier::RaidEpic)
                    .map(|_| 3)
            })
            .map(super::door::band_color)
            .unwrap_or(Color::srgb(0.65, 0.65, 0.65));
        let index = band.unwrap_or(0) * campaign::GRID_SLOTS + slot;
        let selected = key.map_or(
            armory.supplemental_piece.is_none() && armory.piece == index,
            |key| armory.supplemental_piece.as_ref() == Some(key),
        );
        ui.spawn((
            Button,
            key.map_or(Act::InspectPiece(index), |key| {
                Act::InspectSupplemental(key.clone())
            }),
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
            if let Some(piece) = resolved {
                super::super::gear_icons::spawn_visual(entry, piece.visual, size);
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
    text(
        ui,
        font,
        super::door::bonus_line(
            strings,
            progress
                .equipment
                .bonus(class, &progress.pieces, &account.supplemental),
        ),
        layout.body_size,
        BONUS,
    );
    super::cloak::build(strings, ui, font, account, selection, layout);
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
    super::typography::section(ui, font, "Collection Log", layout.title_size, GOLD);
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
    use survivors_core::gear_tier::GearTier;
    for tier in [
        GearTier::Poor,
        GearTier::Common,
        GearTier::Uncommon,
        GearTier::Rare,
        GearTier::Epic,
        GearTier::RaidEpic,
        GearTier::Pvp,
        GearTier::Legendary,
    ] {
        let band = tier.legacy_band();
        let raid = tier == GearTier::RaidEpic;
        let cells: Vec<_> = (0..campaign::GRID_SLOTS)
            .filter_map(|slot| {
                survivors_core::supplemental_gear::piece(
                    class,
                    slot,
                    if raid {
                        GearTier::RaidEpic
                    } else {
                        GearTier::Poor
                    },
                )
            })
            .collect();
        let color = if raid {
            super::door::band_color(3)
        } else {
            band.map(super::door::band_color)
                .unwrap_or(Color::srgb(0.65, 0.65, 0.65))
        };
        let found = band.map_or_else(
            || {
                cells
                    .iter()
                    .filter(|piece| account.supplemental.owns(class, &piece.key))
                    .count()
            },
            |band| {
                progress.pieces[band * campaign::GRID_SLOTS..(band + 1) * campaign::GRID_SLOTS]
                    .iter()
                    .filter(|n| **n > 0)
                    .count()
            },
        );
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
                    band.map(|band| campaign::BANDS_NAMED[band])
                        .unwrap_or(if raid { "Raid Epic" } else { "Poor" }),
                    layout.body_size,
                    color,
                );
                text(
                    label,
                    font,
                    format!("{found}/{}", if raid { 8 } else { 9 }),
                    layout.body_size - 2.0,
                    DIM,
                );
                if raid {
                    text(label, font, "Sources pending", layout.body_size - 3.0, DIM);
                }
                let mut zones = Vec::new();
                for drop in campaign::DROPS
                    .iter()
                    .filter(|drop| Some(drop.band) == band)
                {
                    let name = campaign::ZONES[drop.zone].name;
                    if !zones.contains(&name) {
                        zones.push(name);
                    }
                }
                if !zones.is_empty() {
                    text(label, font, zones.join(" · "), layout.body_size - 3.0, DIM);
                }
            });
            for (slot, piece) in cells.iter().enumerate() {
                let index = band.map(|band| band * campaign::GRID_SLOTS + slot);
                let owned = index.map_or_else(
                    || account.supplemental.owns(class, &piece.key),
                    |i| progress.pieces[i] > 0,
                );
                let equipped = band.map_or_else(
                    || progress.equipment.supplemental_worn()[slot].as_deref() == Some(&piece.key),
                    |band| progress.equipment.worn[slot] == Some(band as u8),
                );
                let shown = band.map_or_else(
                    || progress.equipment.supplemental_looks()[slot].as_deref() == Some(&piece.key),
                    |band| progress.equipment.looks[slot] == Look::Piece(band as u8),
                );
                let selected = index.map_or_else(
                    || armory.supplemental_piece.as_deref() == Some(&piece.key),
                    |index| armory.supplemental_piece.is_none() && armory.piece == index,
                );
                row.spawn((
                    Button,
                    super::controller::CollectionGridItem,
                    index
                        .map(Act::InspectPiece)
                        .unwrap_or_else(|| Act::InspectSupplemental(piece.key.clone())),
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
                        border: UiRect::all(Val::Px(if selected { 2.0 } else { 1.0 })),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(if selected {
                        GOLD
                    } else if owned {
                        color
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
                        let has_icon = if let Some(band) = band {
                            super::super::gear_icons::spawn_cell(
                                cell,
                                class,
                                slot,
                                band,
                                layout.cell_height * 0.86,
                                owned,
                            )
                        } else {
                            super::super::gear_icons::spawn_visual_cell(
                                cell,
                                piece.visual,
                                layout.cell_height * 0.86,
                                owned,
                            );
                            true
                        };
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
            if raid {
                row.spawn(Node {
                    flex_grow: if compact { 0.0 } else { 1.0 },
                    flex_basis: if compact {
                        Val::Percent(10.0)
                    } else {
                        Val::Px(0.0)
                    },
                    min_width: Val::Px(if compact { 54.0 } else { 0.0 }),
                    ..default()
                });
            }
        });
    }
    super::weapon_alternatives::entries(ui, font, class, account, armory, layout);
}

struct ItemHeader<'a> {
    name: &'a str,
    quality: &'a str,
    slot: usize,
    owned: bool,
    color: Color,
}

fn item_header(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    layout: &Layout,
    item: ItemHeader<'_>,
    draw_icon: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: Val::Px(layout.gap),
            padding: UiRect::all(Val::Px(layout.padding * 0.7)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.13, 0.12, 0.10)),
        BorderColor::all(item.color.with_alpha(0.65)),
    ))
    .with_children(|card| {
        card.spawn(Node {
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|icon| {
            draw_icon(icon);
        });
        card.spawn(Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(layout.gap * 0.25),
            ..default()
        })
        .with_children(|labels| {
            text(labels, font, item.name, layout.body_size * 1.15, item.color);
            text(
                labels,
                font,
                format!("{} · {}", item.quality, campaign::SLOTS[item.slot]),
                layout.body_size,
                DIM,
            );
            text(
                labels,
                font,
                if item.owned {
                    "In your collection"
                } else {
                    "Not collected"
                },
                layout.body_size,
                if item.owned { BONUS } else { DIM },
            );
        });
    });
}

pub(super) fn detail(
    strings: &crate::survivors::localization::Strings,
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    if let Some(piece) = armory
        .supplemental_piece
        .as_deref()
        .and_then(survivors_core::supplemental_gear::by_key)
        .filter(|p| p.class == class)
    {
        let owned = account.supplemental.owns(class, &piece.key);
        item_header(
            ui,
            font,
            layout,
            ItemHeader {
                name: piece.name,
                quality: piece
                    .tier
                    .legacy_band()
                    .map(|band| campaign::BANDS_NAMED[band])
                    .unwrap_or(
                        if piece.tier == survivors_core::gear_tier::GearTier::RaidEpic {
                            "Raid Epic"
                        } else {
                            "Poor"
                        },
                    ),
                slot: piece.slot,
                owned,
                color: if piece.tier == survivors_core::gear_tier::GearTier::RaidEpic {
                    super::door::band_color(3)
                } else {
                    piece
                        .tier
                        .legacy_band()
                        .map(super::door::band_color)
                        .unwrap_or(Color::srgb(0.65, 0.65, 0.65))
                },
            },
            |icon| {
                super::super::gear_icons::spawn_visual(icon, piece.visual, layout.title_size * 2.6);
            },
        );
        text(
            ui,
            font,
            super::door::bonus_line(strings, piece.tier.piece_bonus(piece.slot)),
            layout.body_size,
            BONUS,
        );
        if piece.tier == survivors_core::gear_tier::GearTier::RaidEpic {
            super::typography::heading(
                ui,
                font,
                survivors_core::raid_sets::NAMES[class],
                layout.title_size,
                GOLD,
            );
            text(
                ui,
                font,
                "Cumulative set bonuses from equipped Raid Epic armor only:",
                layout.body_size,
                DIM,
            );
            let equipped_count = progress.equipment.raid_pieces(class, &account.supplemental);
            text(
                ui,
                font,
                format!("{equipped_count}/8 raid armor equipped"),
                layout.body_size,
                GOLD,
            );
            for count in [2, 4, 6] {
                text(
                    ui,
                    font,
                    format!(
                        "{count} pieces · {}: {}",
                        if equipped_count >= count {
                            "Active"
                        } else {
                            "Inactive"
                        },
                        super::door::bonus_line(
                            strings,
                            survivors_core::raid_sets::bonus(class, count)
                        )
                    ),
                    layout.body_size,
                    if equipped_count >= count { BONUS } else { DIM },
                );
            }
        }
        super::typography::heading(ui, font, "Drops from", layout.title_size, GOLD);
        text(
            ui,
            font,
            if piece.tier == survivors_core::gear_tier::GearTier::RaidEpic {
                survivors_core::supplemental_gear::raid_source_label()
            } else if survivors_core::weapon_alternatives::by_key(&piece.key).is_some() {
                survivors_core::weapon_alternatives::source_label(piece.class)
            } else {
                survivors_core::supplemental_gear::poor_source_label()
            },
            layout.body_size,
            DIM,
        );
        if piece.tier == survivors_core::gear_tier::GearTier::Poor {
            for zone in survivors_core::supplemental_gear::POOR_DROP_ZONES {
                source_button(ui, font, armory, zone);
            }
        }
        if piece.tier == survivors_core::gear_tier::GearTier::RaidEpic {
            source_button(
                ui,
                font,
                armory,
                survivors_core::supplemental_gear::RAID_DROP_ZONE,
            );
        }
        if survivors_core::weapon_alternatives::by_key(&piece.key).is_some() {
            source_button(ui, font, armory, survivors_core::weapon_alternatives::ZONE);
        }
        if owned {
            let equipped =
                progress.equipment.supplemental_worn()[piece.slot].as_deref() == Some(&piece.key);
            let mut preview = progress.equipment.clone();
            if equipped {
                preview.wear(&progress.pieces, piece.slot, None);
            } else {
                preview.wear_supplemental(class, &account.supplemental, piece.slot, &piece.key);
            }
            text(
                ui,
                font,
                if equipped {
                    "Equipment bonuses if unequipped"
                } else {
                    "Equipment bonuses if equipped"
                },
                layout.body_size,
                GOLD,
            );
            for (line, color) in set_changes(
                strings,
                class,
                &progress.pieces,
                &account.supplemental,
                &progress.equipment,
                &preview,
            ) {
                text(ui, font, line, layout.body_size, color);
            }
            for (line, color) in hud::card_growth::bonus_changes(
                progress
                    .equipment
                    .bonus(class, &progress.pieces, &account.supplemental),
                preview.bonus(class, &progress.pieces, &account.supplemental),
                "No change to equipment bonuses",
            ) {
                text(ui, font, line, layout.body_size, color);
            }
            chip(
                ui,
                font,
                if equipped { "Unequip" } else { "Equip" },
                if equipped {
                    Act::WearPiece(piece.slot, None)
                } else {
                    Act::WearSupplemental(piece.key.clone())
                },
                equipped,
            );
        }
        text(ui, font, "Right-click or focus a collected piece and press Backspace / controller west to equip or unequip. Shift-right-click chooses its look.", layout.body_size, DIM);
        super::typography::heading(
            ui,
            font,
            format!("Look · {}", campaign::SLOTS[piece.slot]),
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
            chip(
                actions,
                font,
                "Worn piece",
                Act::ShowPiece(piece.slot, Look::Worn),
                progress.equipment.supplemental_looks()[piece.slot].is_none()
                    && progress.equipment.looks[piece.slot] == Look::Worn,
            );
            if owned {
                chip(
                    actions,
                    font,
                    "This piece",
                    Act::ShowSupplemental(piece.key.clone()),
                    progress.equipment.supplemental_looks()[piece.slot].as_deref()
                        == Some(&piece.key),
                );
            }
            chip(
                actions,
                font,
                "Hidden",
                Act::ShowPiece(piece.slot, Look::Hidden),
                progress.equipment.supplemental_looks()[piece.slot].is_none()
                    && progress.equipment.looks[piece.slot] == Look::Hidden,
            );
        });
        return;
    }
    let selected = armory.piece.min(campaign::PIECES - 1);
    let slot = selected % campaign::GRID_SLOTS;
    let band = selected / campaign::GRID_SLOTS;
    let name = campaign::piece_name(class, slot, band);
    let owned = progress.pieces[selected] > 0;
    item_header(
        ui,
        font,
        layout,
        ItemHeader {
            name: name.unwrap_or("Item content not yet available"),
            quality: campaign::BANDS_NAMED[band],
            slot,
            owned,
            color: super::door::band_color(band),
        },
        |icon| {
            super::super::gear_icons::spawn(icon, class, slot, band, layout.title_size * 2.6);
        },
    );
    if name.is_some() {
        text(
            ui,
            font,
            super::door::bonus_line(strings, campaign::piece_bonus(band, slot)),
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
        source_button(ui, font, armory, drop.zone);
    }
    if sources == 0 {
        text(ui, font, "Source not yet available", layout.body_size, DIM);
    }
    if owned {
        let equipped = progress.equipment.worn[slot] == Some(band as u8);
        let target = if equipped { None } else { Some(band as u8) };
        let mut preview = progress.equipment.clone();
        if preview.wear(&progress.pieces, slot, target) {
            let before = progress
                .equipment
                .bonus(class, &progress.pieces, &account.supplemental);
            let after = preview.bonus(class, &progress.pieces, &account.supplemental);
            text(
                ui,
                font,
                if equipped {
                    "Equipment bonuses if unequipped"
                } else {
                    "Equipment bonuses if equipped"
                },
                layout.body_size,
                GOLD,
            );
            for (line, color) in set_changes(
                strings,
                class,
                &progress.pieces,
                &account.supplemental,
                &progress.equipment,
                &preview,
            ) {
                text(ui, font, line, layout.body_size, color);
            }
            for (line, color) in
                hud::card_growth::bonus_changes(before, after, "No change to equipment bonuses")
            {
                text(ui, font, line, layout.body_size, color);
            }
        }
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
        "Right-click or focus a collected piece and press Backspace / controller west to equip or unequip. Shift-right-click chooses its look.",
        layout.body_size,
        DIM,
    );
    look_controls(ui, font, armory, account, selection, layout);
}

/// Explain threshold effects separately from the aggregate equipment stat comparison.
fn set_changes(
    strings: &crate::survivors::localization::Strings,
    class: usize,
    pieces: &[u8],
    owned: &survivors_core::supplemental_gear::Inventory,
    before: &survivors_core::gear_loadout::Loadout,
    after: &survivors_core::gear_loadout::Loadout,
) -> Vec<(String, Color)> {
    let mut lines = Vec::new();
    let complete = |loadout: &survivors_core::gear_loadout::Loadout| {
        (0..campaign::GRID_SLOTS).all(|slot| loadout.worn(class, pieces, owned, slot).is_some())
    };
    let transition = |active| {
        if active {
            ("gained", hud::card_growth::GAIN)
        } else {
            ("lost", Color::srgb(1.0, 0.4, 0.35))
        }
    };
    if complete(before) != complete(after) {
        let (verb, color) = transition(complete(after));
        lines.push((
            format!(
                "Full loadout bonus {verb} (all 9 slots): {}",
                super::door::bonus_line(strings, campaign::set_bonus())
            ),
            color,
        ));
    }
    let old_count = before.raid_pieces(class, owned);
    let new_count = after.raid_pieces(class, owned);
    if let Some(name) = survivors_core::raid_sets::NAMES.get(class) {
        for threshold in [2, 4, 6] {
            if (old_count >= threshold) != (new_count >= threshold) {
                let (verb, color) = transition(new_count >= threshold);
                lines.push((format!("{name} {threshold}-piece bonus {verb} ({old_count} » {new_count} raid armor equipped)"), color));
            }
        }
    }
    lines
}

fn source_button(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, armory: &Armory, zone: usize) {
    if armory.page != super::Page::DropTables || armory.drop_zone != zone {
        chip(
            ui,
            font,
            &format!("View {} drops", campaign::ZONES[zone].name),
            Act::OpenDropSource(zone),
            false,
        );
    }
}

pub(super) fn look_controls(
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
    let supplemental = armory
        .supplemental_piece
        .as_deref()
        .and_then(survivors_core::supplemental_gear::by_key)
        .filter(|piece| piece.class == class);
    let slot = supplemental
        .as_ref()
        .map_or(selected % campaign::GRID_SLOTS, |piece| piece.slot);
    let band = selected / campaign::GRID_SLOTS;
    let owned = supplemental
        .as_ref()
        .map_or(progress.pieces[selected] > 0, |piece| {
            account.supplemental.owns(class, &piece.key)
        });
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
                let (action, active) = if let Some(piece) = supplemental
                    .as_ref()
                    .filter(|_| matches!(look, Look::Piece(_)))
                {
                    (
                        Act::ShowSupplemental(piece.key.clone()),
                        progress.equipment.supplemental_looks()[slot].as_deref()
                            == Some(piece.key.as_str()),
                    )
                } else {
                    (
                        Act::ShowPiece(slot, look),
                        progress.equipment.supplemental_looks()[slot].is_none()
                            && progress.equipment.looks[slot] == look,
                    )
                };
                chip(actions, font, label, action, active);
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
                BorderColor::all(
                    if armory.supplemental_piece.is_none() && armory.piece == index {
                        GOLD
                    } else {
                        Color::NONE
                    },
                ),
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
    fn character_look_controls_follow_supplemental_slot_and_ownership() {
        for (key, slot) in [
            ("weapon:warrior:worn-mace", 8),
            ("weapon:warrior:short-spear", 8),
            ("poor:warrior:hands", 7),
        ] {
            for owned in [false, true] {
                let mut account = super::super::super::account::blank();
                if owned {
                    assert!(account.supplemental.collect(0, key));
                }
                let armory = Armory {
                    piece: 0,
                    supplemental_piece: Some(key.to_owned()),
                    ..default()
                };
                let selection = Selection {
                    class: Some(0),
                    ..default()
                };
                let mut world = World::new();
                world.commands().spawn(Node::default()).with_children(|ui| {
                    look_controls(
                        ui,
                        &Handle::default(),
                        &armory,
                        &account,
                        &selection,
                        &Layout::default(),
                    );
                });
                world.flush();
                let acts: Vec<_> = world.query::<&Act>().iter(&world).collect();
                assert_eq!(acts.len(), if owned { 3 } else { 2 });
                assert!(acts
                    .iter()
                    .any(|act| matches!(act, Act::ShowPiece(s, Look::Worn) if *s == slot)));
                assert!(acts
                    .iter()
                    .any(|act| matches!(act, Act::ShowPiece(s, Look::Hidden) if *s == slot)));
                assert_eq!(
                    acts.iter()
                        .filter(|act| matches!(act, Act::ShowSupplemental(k) if k == key))
                        .count(),
                    usize::from(owned)
                );
                let heading = format!("Look · {}", campaign::SLOTS[slot]);
                assert!(world
                    .query::<&Text>()
                    .iter(&world)
                    .any(|text| text.0 == heading));
            }
        }
    }

    #[test]
    fn set_previews_explain_full_loadout_and_raid_thresholds_in_both_item_paths() {
        use survivors_core::{gear_tier::GearTier, supplemental_gear};
        for class in 0..9 {
            let mut account = super::super::super::account::blank();
            account.classes[class].pieces.fill(1);
            for slot in 0..9 {
                account.classes[class]
                    .equipment
                    .wear(&[1; campaign::PIECES], slot, Some(0));
            }
            // Replacing a worn piece preserves the complete loadout bonus.
            let original = account.classes[class].equipment.clone();
            let mut replacement = original.clone();
            replacement.wear(&[1; campaign::PIECES], 0, Some(1));
            assert!(set_changes(
                &crate::survivors::localization::Strings::default(),
                class,
                &[1; campaign::PIECES],
                &account.supplemental,
                &original,
                &replacement
            )
            .is_empty());
            for raid_count in 0..=6 {
                if raid_count > 0 {
                    let piece = supplemental_gear::piece(class, raid_count - 1, GearTier::RaidEpic)
                        .unwrap();
                    account.supplemental.collect(class, &piece.key);
                    account.classes[class].equipment.wear_supplemental(
                        class,
                        &account.supplemental,
                        piece.slot,
                        &piece.key,
                    );
                }
                let armory = Armory {
                    piece: 0,
                    supplemental_piece: (raid_count > 0).then(|| {
                        supplemental_gear::piece(class, raid_count - 1, GearTier::RaidEpic)
                            .unwrap()
                            .key
                    }),
                    ..default()
                };
                let mut world = World::new();
                world
                    .commands()
                    .spawn(Node::default())
                    .with_children(|panel| {
                        detail(
                            &crate::survivors::localization::Strings::default(),
                            panel,
                            &Handle::default(),
                            &armory,
                            &account,
                            &Selection {
                                class: Some(class),
                                ..default()
                            },
                            &Layout::default(),
                        );
                    });
                world.flush();
                let lines: Vec<_> = world
                    .query::<&Text>()
                    .iter(&world)
                    .map(|text| text.0.clone())
                    .collect();
                assert!(
                    lines
                        .iter()
                        .any(|line| line.starts_with("Full loadout bonus lost (all 9 slots):")),
                    "{class}: {raid_count}: {lines:?}"
                );
                let raid_lines: Vec<_> = lines
                    .iter()
                    .filter(|line| line.contains("raid armor equipped)"))
                    .collect();
                if [2, 4, 6].contains(&raid_count) {
                    assert_eq!(raid_lines.len(), 1);
                    assert!(raid_lines[0].contains(&format!("{raid_count}-piece bonus lost")));
                } else {
                    assert!(raid_lines.is_empty());
                }
                let before = &account.classes[class].equipment;
                let mut removed = before.clone();
                removed.wear(&[1; campaign::PIECES], raid_count.saturating_sub(1), None);
                let restored = set_changes(
                    &crate::survivors::localization::Strings::default(),
                    class,
                    &[1; campaign::PIECES],
                    &account.supplemental,
                    &removed,
                    before,
                );
                assert!(restored.iter().all(
                    |(line, color)| line.contains("gained") && *color == hud::card_growth::GAIN
                ));
                assert_eq!(
                    restored.len(),
                    1 + usize::from([2, 4, 6].contains(&raid_count))
                );
            }
        }
    }

    #[test]
    fn supplemental_sources_link_only_obtainable_zones_and_skip_current_destination() {
        use survivors_core::{gear_tier::GearTier, supplemental_gear};
        let account = super::super::super::account::blank();
        for class in 0..9 {
            for tier in [GearTier::Poor, GearTier::RaidEpic] {
                for slot in 0..9 {
                    let Some(piece) = supplemental_gear::piece(class, slot, tier) else {
                        continue;
                    };
                    for current in [None, Some(0), Some(1)] {
                        let armory = Armory {
                            page: if current.is_some() {
                                super::super::Page::DropTables
                            } else {
                                super::super::Page::Collection
                            },
                            drop_zone: current.unwrap_or(0),
                            supplemental_piece: Some(piece.key.clone()),
                            ..default()
                        };
                        let selection = Selection {
                            class: Some(class),
                            ..default()
                        };
                        let mut world = World::new();
                        world
                            .commands()
                            .spawn(Node::default())
                            .with_children(|panel| {
                                detail(
                                    &crate::survivors::localization::Strings::default(),
                                    panel,
                                    &Handle::default(),
                                    &armory,
                                    &account,
                                    &selection,
                                    &Layout::default(),
                                );
                            });
                        world.flush();
                        let actual: Vec<_> = world
                            .query::<&Act>()
                            .iter(&world)
                            .filter_map(|act| {
                                if let Act::OpenDropSource(zone) = act {
                                    Some(*zone)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let zones = if tier == GearTier::Poor {
                            supplemental_gear::POOR_DROP_ZONES.to_vec()
                        } else {
                            vec![supplemental_gear::RAID_DROP_ZONE]
                        };
                        let expected: Vec<_> = zones
                            .into_iter()
                            .filter(|&zone| Some(zone) != current)
                            .collect();
                        assert_eq!(actual, expected, "{} at {current:?}", piece.key);
                    }
                }
            }
        }
    }

    #[test]
    fn legacy_item_sources_offer_navigation_to_their_actual_zones() {
        let account = super::super::super::account::blank();
        let selection = Selection {
            class: Some(0),
            ..default()
        };
        for piece in 0..campaign::PIECES {
            let armory = Armory {
                page: super::super::Page::Collection,
                piece,
                ..default()
            };
            let mut world = World::new();
            world
                .commands()
                .spawn(Node::default())
                .with_children(|panel| {
                    detail(
                        &crate::survivors::localization::Strings::default(),
                        panel,
                        &Handle::default(),
                        &armory,
                        &account,
                        &selection,
                        &Layout::default(),
                    );
                });
            world.flush();
            let actual: Vec<_> = world
                .query::<&Act>()
                .iter(&world)
                .filter_map(|act| {
                    if let Act::OpenDropSource(zone) = act {
                        Some(*zone)
                    } else {
                        None
                    }
                })
                .collect();
            let expected: Vec<_> = campaign::DROPS
                .iter()
                .filter(|drop| {
                    drop.band == piece / campaign::GRID_SLOTS
                        && drop.slot == piece % campaign::GRID_SLOTS
                })
                .map(|drop| drop.zone)
                .collect();
            assert_eq!(actual, expected, "piece {piece}");
        }
    }

    #[test]
    fn poor_inspector_only_offers_owned_equipment_and_appearance_actions() {
        for owned in [false, true] {
            let mut account = super::super::super::account::blank();
            let key = "poor:hunter:hands";
            if owned {
                account.supplemental.collect(2, key);
            }
            let armory = Armory {
                supplemental_piece: Some(key.into()),
                ..default()
            };
            let selection = Selection {
                class: Some(2),
                ..default()
            };
            let mut world = World::new();
            world
                .commands()
                .spawn(Node::default())
                .with_children(|panel| {
                    detail(
                        &crate::survivors::localization::Strings::default(),
                        panel,
                        &Handle::default(),
                        &armory,
                        &account,
                        &selection,
                        &Layout::default(),
                    );
                });
            world.flush();
            let actions: Vec<_> = world.query::<&Act>().iter(&world).collect();
            assert_eq!(
                actions
                    .iter()
                    .any(|a| matches!(a, Act::WearSupplemental(k) if k == key)),
                owned
            );
            assert_eq!(
                actions
                    .iter()
                    .any(|a| matches!(a, Act::ShowSupplemental(k) if k == key)),
                owned
            );
            assert!(world
                .query::<&Text>()
                .iter(&world)
                .any(|t| t.0 == "Double Mail Gloves"));
            assert!(world.query::<&Text>().iter(&world).any(|t| t.0
                == if owned {
                    "In your collection"
                } else {
                    "Not collected"
                }));
        }
    }

    #[test]
    fn empty_and_full_class_collections_build_without_using_the_legacy_off_hand() {
        for class in 0..super::super::CLASSES.len() {
            for (owned, compact) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut account = super::super::super::account::blank();
                if owned {
                    account.classes[class].pieces.fill(1);
                    for slot in 0..campaign::GRID_SLOTS {
                        account.classes[class].equipment.wear(
                            &[1; campaign::PIECES],
                            slot,
                            Some(0),
                        );
                    }
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
                            &crate::survivors::localization::Strings::default(),
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
                let raid: Vec<_> = actions
                    .iter()
                    .filter_map(|action| match action {
                        Act::InspectSupplemental(key) if key.starts_with("raid-epic:") => {
                            survivors_core::supplemental_gear::by_key(key)
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(raid.len(), 8);
                assert!(raid
                    .iter()
                    .all(|piece| piece.class == class && piece.slot < 8));
                for index in 0..campaign::PIECES {
                    assert!(actions
                        .iter()
                        .any(|act| matches!(act, Act::InspectPiece(found) if *found == index)));
                }
                for slot in 0..campaign::GRID_SLOTS {
                    let piece = survivors_core::supplemental_gear::piece(
                        class,
                        slot,
                        survivors_core::gear_tier::GearTier::Poor,
                    )
                    .unwrap();
                    assert!(actions
                        .iter()
                        .any(|a| matches!(a, Act::InspectSupplemental(key) if key == &piece.key)));
                }
                assert!(world
                    .query::<&Text>()
                    .iter(&world)
                    .any(|text| text.0 == "Raid Epic"));
                let panels = world.query::<&CollectionScroll>().iter(&world).count();
                assert_eq!(panels, if compact { 1 } else { 3 });
            }
        }
    }

    #[test]
    fn raid_inspector_counts_equipped_armor_and_marks_thresholds() {
        use survivors_core::{gear_tier::GearTier, supplemental_gear};
        for count in [0, 1, 2, 3, 4, 5, 6, 8] {
            let mut account = super::super::super::account::blank();
            for slot in 0..8 {
                let piece = supplemental_gear::piece(0, slot, GearTier::RaidEpic).unwrap();
                account.supplemental.collect(0, &piece.key);
                account.select_supplemental(0, &piece.key, true);
                if slot < count {
                    account.select_supplemental(0, &piece.key, false);
                }
            }
            let armory = Armory {
                supplemental_piece: Some("raid-epic:warrior:head".into()),
                ..default()
            };
            let selection = Selection {
                class: Some(0),
                ..default()
            };
            let mut world = World::new();
            world
                .commands()
                .spawn(Node::default())
                .with_children(|panel| {
                    detail(
                        &crate::survivors::localization::Strings::default(),
                        panel,
                        &Handle::default(),
                        &armory,
                        &account,
                        &selection,
                        &Layout::default(),
                    );
                });
            world.flush();
            let texts: Vec<_> = world.query::<(&Text, &TextColor)>().iter(&world).collect();
            assert!(texts
                .iter()
                .any(|(text, _)| text.0 == format!("{count}/8 raid armor equipped")));
            for threshold in [2, 4, 6] {
                let active = count >= threshold;
                let prefix = format!(
                    "{threshold} pieces · {}:",
                    if active { "Active" } else { "Inactive" }
                );
                let (_, color) = texts
                    .iter()
                    .find(|(text, _)| text.0.starts_with(&prefix))
                    .unwrap();
                assert_eq!(color.0, if active { BONUS } else { DIM });
            }
        }
    }
}
