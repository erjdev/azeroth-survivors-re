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

pub(super) fn build(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let class = picked_class(account, selection);
    let pieces = &account.classes[class].pieces;
    label(
        ui,
        font,
        format!(
            "{} · {} of {} pieces collected",
            super::CLASSES[class].name,
            campaign::owned_pieces(pieces),
            campaign::PIECES
        ),
        layout.body_size,
        GOLD,
    );
    ui.spawn(Node { width: Val::Percent(100.0), flex_grow: 1.0, min_height: Val::Px(0.0), column_gap: Val::Px(layout.gap), ..default() }).with_children(|row| {
        collection::column(row, layout.left_width, layout, |body| {
            character::body_controls(body, art, font, account, selection, layout);
            collection::equipment(body, font, armory, account, selection, layout);
        });
        collection::column(row, (100.0-layout.left_width-layout.right_width)*0.30, layout, |body| {
            for (index, zone) in campaign::ZONES.iter().enumerate() {
                let drops: Vec<_> = campaign::DROPS.iter().filter(|d| d.zone == index).collect();
                let owned = drops.iter().filter(|d| pieces[d.band*9+d.slot] != 0).count();
                body.spawn((Button, Act::BrowseDrops(index), Node {
                    width: Val::Percent(100.0), flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0, padding: UiRect::all(Val::Px(layout.gap)),
                    border: UiRect::all(Val::Px(1.0)), ..default()
                }, BackgroundColor(if armory.drop_zone == index { Color::srgb(0.16,0.12,0.04) } else { Color::srgb(0.03,0.025,0.015) }),
                BorderColor::all(if armory.drop_zone == index { GOLD } else { DIM }))).with_children(|button| {
                    label(button, font, zone.name, layout.body_size, GOLD);
                    let summary = if drops.is_empty() { "No table yet".into() } else { format!("{owned}/{} collected", drops.len()) };
                    label(button, font, summary, layout.body_size-2.0, DIM);
                });
            }
        });
        collection::column(row, (100.0-layout.left_width-layout.right_width)*0.70, layout, |body| {
            let zone = &campaign::ZONES[armory.drop_zone];
            label(body, font, zone.name, layout.title_size, GOLD);
            label(body, font, "Each piece rolls independently. Duplicates add no stats. Chances below are base values; Keen Eye improves boss rolls.", layout.body_size, DIM);
            let rows: Vec<_> = campaign::DROPS.iter().filter(|d| d.zone == armory.drop_zone).collect();
            if rows.is_empty() {
                label(body, font, "This zone has no implemented drop table yet.", layout.body_size, DIM);
            }
            for drop in rows {
                let piece = drop.band*9+drop.slot;
                let owned = pieces[piece] != 0;
                body.spawn((Button, Act::InspectPiece(piece), Node {
                    width: Val::Percent(100.0), flex_shrink: 0.0, align_items: AlignItems::Center,
                    column_gap: Val::Px(layout.gap), padding: UiRect::all(Val::Px(layout.padding)),
                    border: UiRect::all(Val::Px(1.0)), ..default()
                }, BackgroundColor(Color::srgb(0.035,0.025,0.015)),
                BorderColor::all(if armory.piece == piece { GOLD } else { DIM }))).with_children(|entry| {
                    super::super::gear_icons::spawn_cell(entry, class, drop.slot, drop.band, layout.body_size*3.0, owned);
                    entry.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, min_width: Val::Px(0.0), ..default() }).with_children(|text| {
                        label(text, font, drop.creature, layout.body_size, GOLD);
                        label(text, font, format!("{}:{:02}{}", drop.at_secs/60, drop.at_secs%60, if drop.creature == zone.boss { " · final boss" } else { "" }), layout.body_size-2.0, DIM);
                        label(text, font, campaign::piece_name(class, drop.slot, drop.band).unwrap_or("Item content pending"), layout.body_size, door::band_color(drop.band));
                        label(text, font, format!("{}% base · {}", drop.chance, campaign::SLOTS[drop.slot]), layout.body_size-2.0, DIM);
                        label(text, font, if owned { "Collected" } else { "Not collected" }, layout.body_size-2.0, if owned { GREEN } else { DIM });
                    });
                });
            }
        });
        collection::column(row, layout.right_width, layout, |body| collection::detail(body, font, armory, account, selection, layout));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_zone_exposes_only_its_catalog_sources_without_run_selection_actions() {
        for zone in 0..campaign::ZONES.len() {
            let mut world = World::new();
            let mut armory = Armory::default();
            armory.drop_zone = zone;
            world.commands().spawn(Node::default()).with_children(|ui| {
                build(
                    ui,
                    &GlueArt::default(),
                    &Handle::default(),
                    &armory,
                    &super::super::super::account::blank(),
                    &Selection::default(),
                    &Layout::default(),
                );
            });
            world.flush();
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
