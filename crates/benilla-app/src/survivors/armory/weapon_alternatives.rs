//! Shared alternative-weapon entries for Collection and encounter sources.
use super::{hud, Account, Act, Armory, GOLD};
use bevy::prelude::*;
use survivors_core::{layout::Layout, weapon_alternatives as weapons};

pub(super) fn entries(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    class: usize,
    account: &Account,
    armory: &Armory,
    layout: &Layout,
) {
    if weapons::choices(class).is_empty() {
        return;
    }
    super::typography::heading(
        ui,
        font,
        "Common alternative weapons",
        layout.title_size,
        GOLD,
    );
    ui.spawn(hud::text_bundle(
        font,
        "Hogger · Elwynn Forest · separate from the 54-piece collection",
        layout.body_size,
        Color::srgb(0.65, 0.60, 0.44),
    ));
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(layout.gap),
        row_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|row| {
        for weapon in weapons::choices(class) {
            let owned = account.supplemental.owns(class, weapon.key);
            let gear = &account.classes[class].equipment;
            let worn = gear.supplemental_worn()[weapons::SLOT].as_deref() == Some(weapon.key);
            let shown = gear.supplemental_looks()[weapons::SLOT].as_deref() == Some(weapon.key);
            let selected = armory.supplemental_piece.as_deref() == Some(weapon.key);
            row.spawn((
                Button,
                super::controller::CollectionGridItem,
                Act::InspectSupplemental(weapon.key.to_owned()),
                Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Percent(45.0),
                    min_width: Val::Px(180.0),
                    padding: UiRect::all(Val::Px(layout.padding)),
                    column_gap: Val::Px(layout.gap),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.07, 0.06, 0.04)),
                BorderColor::all(if selected {
                    GOLD
                } else {
                    Color::srgb(0.35, 0.32, 0.25)
                }),
            ))
            .with_children(|entry| {
                super::super::gear_icons::spawn_visual_cell(
                    entry,
                    weapon.visual,
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
                        text.spawn(hud::text_bundle(
                            font,
                            weapon.name,
                            layout.body_size,
                            Color::WHITE,
                        ));
                        let kind = weapon.kind_label();
                        text.spawn(hud::text_bundle(font, kind, layout.body_size - 2.0, GOLD));
                        let state = match (owned, worn, shown) {
                            (_, true, true) => "Equipped · Appearance selected",
                            (_, true, false) => "Equipped",
                            (_, false, true) => "Appearance selected",
                            (true, false, false) => "Collected",
                            _ => "Not collected",
                        };
                        text.spawn(hud::text_bundle(
                            font,
                            state,
                            layout.body_size - 2.0,
                            if owned {
                                Color::srgb(0.45, 0.9, 0.4)
                            } else {
                                Color::srgb(0.65, 0.60, 0.44)
                            },
                        ));
                    });
            });
        }
    });
}
