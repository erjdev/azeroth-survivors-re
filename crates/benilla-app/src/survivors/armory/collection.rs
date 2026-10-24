//! Collection choices share the same loadout used by the run's stat sheet.

use super::super::equipment::Look;
use super::{campaign, chip, hud, picked_class, Account, Act, Armory, Selection, GOLD};
use bevy::prelude::*;

pub(super) fn build(
    panel: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
) {
    let class = picked_class(account, selection);
    let progress = &account.classes[class];
    let selected = armory.piece.min(campaign::PIECES - 1);
    let slot = selected % campaign::GRID_SLOTS;
    let band = selected / campaign::GRID_SLOTS;
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "Collection — {} / {} pieces",
            campaign::owned_pieces(&progress.pieces),
            campaign::PIECES
        ),
        20.0,
        GOLD,
    ));
    panel.spawn(hud::text_bundle(
        font,
        "Select a piece to inspect. Right-click equips; Shift-right-click selects its appearance.",
        13.0,
        Color::WHITE,
    ));
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        })
        .with_children(|grid| {
            for quality in 0..campaign::BANDS {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|row| {
                    let found = progress.pieces
                        [quality * campaign::GRID_SLOTS..(quality + 1) * campaign::GRID_SLOTS]
                        .iter()
                        .filter(|count| **count > 0)
                        .count();
                    row.spawn((Node {
                        width: Val::Px(100.0),
                        ..default()
                    },))
                        .with_children(|label| {
                            label.spawn(hud::text_bundle(
                                font,
                                format!(
                                    "{} {found}/{}",
                                    campaign::BANDS_NAMED[quality],
                                    campaign::GRID_SLOTS
                                ),
                                14.0,
                                super::door::band_color(quality),
                            ));
                        });
                    for equipment_slot in 0..campaign::GRID_SLOTS {
                        let index = quality * campaign::GRID_SLOTS + equipment_slot;
                        let owned = progress.pieces[index] > 0;
                        let equipped =
                            progress.equipment.worn[equipment_slot] == Some(quality as u8);
                        let shown =
                            progress.equipment.looks[equipment_slot] == Look::Piece(quality as u8);
                        let label = format!(
                            "{}{}{}{}",
                            campaign::SLOTS[equipment_slot],
                            if equipped { " · E" } else { "" },
                            if shown { " · L" } else { "" },
                            if owned { "" } else { " ?" }
                        );
                        chip(
                            row,
                            font,
                            &label,
                            Act::InspectPiece(index),
                            selected == index,
                        );
                    }
                });
            }
        });
    let name = campaign::piece_name(class, slot, band).unwrap_or("Item content not yet available");
    let owned = progress.pieces[selected] > 0;
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "{} — {} · {}",
            name,
            campaign::BANDS_NAMED[band],
            campaign::SLOTS[slot]
        ),
        18.0,
        super::door::band_color(band),
    ));
    panel.spawn(hud::text_bundle(
        font,
        if owned {
            "In your collection"
        } else {
            "Not collected"
        },
        14.0,
        Color::WHITE,
    ));
    for drop in campaign::DROPS
        .iter()
        .filter(|row| row.band == band && row.slot == slot)
    {
        panel.spawn(hud::text_bundle(
            font,
            format!(
                "{} · {} · {}:{:02} · {}% base chance",
                campaign::ZONES[drop.zone].name,
                drop.creature,
                drop.at_secs / 60,
                drop.at_secs % 60,
                drop.chance
            ),
            14.0,
            GOLD,
        ));
    }
    if campaign::piece_name(class, slot, band).is_some() {
        panel.spawn(hud::text_bundle(
            font,
            super::door::bonus_line(campaign::piece_bonus(band, slot)),
            14.0,
            Color::srgb(0.45, 0.9, 0.4),
        ));
    }
    let worn = progress.equipment.worn_pieces(&progress.pieces);
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "Worn equipment: {}",
            super::door::bonus_line(campaign::equipped_bonus(&worn, None))
        ),
        14.0,
        Color::srgb(0.45, 0.9, 0.4),
    ));
    if owned {
        let equipped = progress.equipment.worn[slot] == Some(band as u8);
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|actions| {
                chip(
                    actions,
                    font,
                    if equipped { "Unequip" } else { "Equip" },
                    Act::WearPiece(slot, if equipped { None } else { Some(band as u8) }),
                    equipped,
                );
                for (label, look) in [
                    ("Worn appearance", Look::Worn),
                    ("This appearance", Look::Piece(band as u8)),
                    ("Hidden", Look::Hidden),
                ] {
                    chip(
                        actions,
                        font,
                        label,
                        Act::ShowPiece(slot, look),
                        progress.equipment.looks[slot] == look,
                    );
                }
            });
    }
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
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "Still to find in your open zones — {} sources",
            missing.len()
        ),
        16.0,
        GOLD,
    ));
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|list| {
            for drop in missing {
                let index = drop.band * campaign::GRID_SLOTS + drop.slot;
                let name = campaign::piece_name(class, drop.slot, drop.band).unwrap();
                list.spawn(Node {
                    width: Val::Percent(48.0),
                    ..default()
                })
                .with_children(|entry| {
                    chip(
                        entry,
                        font,
                        &format!(
                            "{name} · {} · {} · {}%",
                            drop.creature,
                            campaign::ZONES[drop.zone].name,
                            drop.chance
                        ),
                        Act::InspectPiece(index),
                        selected == index,
                    );
                });
            }
        });
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(8.0),
            ..default()
        })
        .insert(BackgroundColor(Color::srgb(0.10, 0.08, 0.04)))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(
                        100.0 * campaign::owned_pieces(&progress.pieces) as f32
                            / campaign::PIECES as f32,
                    ),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(GOLD),
            ));
        });
    panel.spawn(hud::text_bundle(
        font,
        "E: equipped · L: shown as a look. Only worn pieces add bonuses.",
        13.0,
        Color::WHITE,
    ));
}
