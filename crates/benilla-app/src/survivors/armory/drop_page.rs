//! Browse source encounters independently of the zone selected for the next run.
use super::{
    campaign, character, collection, door, hud, picked_class, Account, Act, Armory, Selection, GOLD,
};
use crate::glue::art::GlueArt;
use bevy::prelude::*;
use survivors_core::layout::Layout;

const DIM: Color = Color::srgb(0.65, 0.60, 0.44);
const GREEN: Color = Color::srgb(0.45, 0.9, 0.4);

fn label(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, text, size, color));
}

/// Keep source labels consistent across Drop Tables, Collection and item details.
pub(super) fn chance_label(drop: &campaign::DropRow, account: &Account) -> String {
    let chance = campaign::drop_chance(
        drop.chance,
        super::super::encounters::drop_is_boss(drop),
        account.perks.keen_eye,
    );
    let percent = (chance * 100.0).round() as u8;
    if percent > drop.chance {
        format!("{percent}% ({}% base + Keen Eye)", drop.chance)
    } else {
        format!("{percent}%")
    }
}

pub(super) fn build(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
    compact: bool,
) {
    if compact {
        ui.spawn((
            collection::CollectionScroll,
            collection::DropCompactScroll,
            bevy::ui::RelativeCursorPosition::default(),
            ScrollPosition(armory.drop_scroll),
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
            collection::section(body, layout, |body| {
                zones(body, font, armory, account, selection, layout, compact)
            });
            collection::section(body, layout, |body| {
                sources(body, font, armory, account, selection, layout)
            });
            collection::section(body, layout, |body| {
                collection::detail(body, font, armory, account, selection, layout)
            });
            collection::section(body, layout, |body| {
                character::body_controls(body, art, font, account, selection, layout);
                collection::equipment(body, font, armory, account, selection, layout);
            });
        });
        return;
    }
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        min_height: Val::Px(0.0),
        column_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|row| {
        collection::column(row, layout.left_width, layout, |body| {
            character::body_controls(body, art, font, account, selection, layout);
            collection::equipment(body, font, armory, account, selection, layout);
        });
        collection::column(
            row,
            (100.0 - layout.left_width - layout.right_width) * 0.30,
            layout,
            |body| zones(body, font, armory, account, selection, layout, compact),
        );
        collection::column(
            row,
            (100.0 - layout.left_width - layout.right_width) * 0.70,
            layout,
            |body| sources(body, font, armory, account, selection, layout),
        );
        collection::column(row, layout.right_width, layout, |body| {
            collection::detail(body, font, armory, account, selection, layout)
        });
    });
}

fn zones(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
    compact: bool,
) {
    let class = picked_class(account, selection);
    let pieces = &account.classes[class].pieces;
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(layout.gap),
        row_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|body| {
        for (index, zone) in campaign::ZONES.iter().enumerate() {
            let drops: Vec<_> = campaign::DROPS.iter().filter(|d| d.zone == index).collect();
            let owned = drops
                .iter()
                .filter(|d| pieces[d.band * 9 + d.slot] != 0)
                .count();
            body.spawn((
                Button,
                Act::BrowseDrops(index),
                Node {
                    width: if compact {
                        Val::Auto
                    } else {
                        Val::Percent(100.0)
                    },
                    flex_basis: if compact { Val::Px(150.0) } else { Val::Auto },
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    padding: UiRect::all(Val::Px(layout.gap)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(if armory.drop_zone == index {
                    Color::srgb(0.16, 0.12, 0.04)
                } else {
                    Color::srgb(0.03, 0.025, 0.015)
                }),
                BorderColor::all(if armory.drop_zone == index { GOLD } else { DIM }),
            ))
            .with_children(|button| {
                label(button, font, zone.name, layout.body_size, GOLD);
                let summary = if drops.is_empty() {
                    "No table yet".into()
                } else {
                    format!(
                        "{} · {owned}/{}",
                        campaign::BANDS_NAMED[drops[0].band],
                        drops.len()
                    )
                };
                label(
                    button,
                    font,
                    summary,
                    layout.body_size - 2.0,
                    drops
                        .first()
                        .map_or(DIM, |drop| door::band_color(drop.band)),
                );
            });
        }
    });
}

fn sources(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let class = picked_class(account, selection);
    let pieces = &account.classes[class].pieces;
    let zone = &campaign::ZONES[armory.drop_zone];
    label(body, font, zone.name, layout.title_size, GOLD);
    let (level, clears) = super::hero_gate(account, selection);
    if let Some(reason) = campaign::zone_lock_reason(armory.drop_zone, level, &clears) {
        label(body, font, reason, layout.body_size, GOLD);
    }
    label(body, font, "Each piece rolls independently. Duplicates add no stats. Chances include your Keen Eye bonus for scheduled bosses.", layout.body_size, DIM);
    let rows: Vec<_> = campaign::DROPS
        .iter()
        .filter(|d| d.zone == armory.drop_zone)
        .collect();
    if rows.is_empty() {
        label(
            body,
            font,
            "This zone has no implemented drop table yet.",
            layout.body_size,
            DIM,
        );
    }
    for drop in rows {
        let piece = drop.band * 9 + drop.slot;
        let owned = pieces[piece] != 0;
        body.spawn((
            Button,
            Act::InspectPiece(piece),
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: Val::Px(layout.gap),
                padding: UiRect::all(Val::Px(layout.padding)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.025, 0.015)),
            BorderColor::all(if armory.piece == piece { GOLD } else { DIM }),
        ))
        .with_children(|entry| {
            entry
                .spawn(Node {
                    flex_basis: Val::Percent(38.0),
                    flex_shrink: 0.0,
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|source| {
                    label(source, font, drop.creature, layout.body_size, GOLD);
                    label(
                        source,
                        font,
                        format!(
                            "{}:{:02}{}",
                            drop.at_secs / 60,
                            drop.at_secs % 60,
                            if super::super::encounters::drop_is_final_boss(drop) {
                                " · final boss"
                            } else if super::super::encounters::drop_is_boss(drop) {
                                " · boss"
                            } else {
                                ""
                            }
                        ),
                        layout.body_size - 2.0,
                        DIM,
                    );
                });
            super::super::gear_icons::spawn_cell(
                entry,
                class,
                drop.slot,
                drop.band,
                layout.body_size * 3.0,
                owned,
            );
            entry
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    ..default()
                })
                .with_children(|text| {
                    label(
                        text,
                        font,
                        campaign::piece_name(class, drop.slot, drop.band)
                            .unwrap_or("Item content pending"),
                        layout.body_size,
                        door::band_color(drop.band),
                    );
                    label(
                        text,
                        font,
                        format!(
                            "{} · {}",
                            chance_label(drop, account),
                            campaign::SLOTS[drop.slot]
                        ),
                        layout.body_size - 2.0,
                        DIM,
                    );
                    label(
                        text,
                        font,
                        if owned { "Collected" } else { "Not collected" },
                        layout.body_size - 2.0,
                        if owned { GREEN } else { DIM },
                    );
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_labels_include_the_accounts_effective_keen_eye_rate() {
        let mut account = super::super::super::account::blank();
        let source = &campaign::DROPS[0];
        assert_eq!(chance_label(source, &account), "60%");
        account.perks.keen_eye = 1;
        assert_eq!(chance_label(source, &account), "65% (60% base + Keen Eye)");
        account.perks.keen_eye = 2;
        assert_eq!(chance_label(source, &account), "70% (60% base + Keen Eye)");
    }

    #[test]
    fn every_zone_exposes_only_its_catalog_sources_without_run_selection_actions() {
        for (zone, compact) in
            (0..campaign::ZONES.len()).flat_map(|zone| [(zone, false), (zone, true)])
        {
            let mut world = World::new();
            let armory = Armory {
                drop_zone: zone,
                ..default()
            };
            world.commands().spawn(Node::default()).with_children(|ui| {
                build(
                    ui,
                    &GlueArt::default(),
                    &Handle::default(),
                    &armory,
                    &super::super::super::account::blank(),
                    &Selection::default(),
                    &Layout::default(),
                    compact,
                );
            });
            world.flush();
            assert_eq!(
                world
                    .query::<&collection::CollectionScroll>()
                    .iter(&world)
                    .count(),
                if compact { 1 } else { 4 }
            );
            let actions: Vec<_> = world.query::<&Act>().iter(&world).collect();
            assert!(!actions
                .iter()
                .any(|a| matches!(a, Act::Zone(_) | Act::Enter)));
            assert_eq!(
                actions
                    .iter()
                    .filter(|a| matches!(a, Act::BrowseDrops(_)))
                    .count(),
                campaign::ZONES.len()
            );
            // Nine loadout actions are shared with Character; the rest are source rows.
            let inspected: Vec<_> = actions
                .iter()
                .filter_map(|a| {
                    if let Act::InspectPiece(i) = a {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                inspected.len(),
                9 + campaign::DROPS.iter().filter(|d| d.zone == zone).count()
            );
            for row in campaign::DROPS.iter().filter(|d| d.zone == zone) {
                assert!(inspected.contains(&(row.band * 9 + row.slot)));
            }
        }
    }
}
