//! The discovered catalog and independently scrollable selected-entry details.

use super::{campaign, codex_jewel, codex_legend, collection, data, hud, picked_class};
use super::{Account, Act, Armory, Selection, CLASSES, GOLD};
use bevy::prelude::*;
use survivors_core::layout::Layout;

const DIM: Color = Color::srgb(0.65, 0.60, 0.44);
const GREEN: Color = Color::srgb(0.45, 0.9, 0.4);

// Legendary amulets have generated names, not vanilla item identities. Use one
// verified necklace silhouette with the associated spell badge until bespoke art exists.
const LEGEND_DISPLAY: u32 = 9857;

#[derive(Component)]
pub(in crate::survivors) struct CatalogCell {
    id: u16,
    seen: bool,
}

#[derive(Component)]
pub(in crate::survivors) struct DetailArt(Option<u16>);

fn entry_icon(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, id: u16, size: f32, seen: bool) {
    let index = usize::from(id & 0x0fff);
    let legend = if id >> 12 == 1 {
        CLASSES
            .get(index >> 6)
            .and_then(|class| class.ability_keys().get(index & 0x3f).copied())
    } else {
        None
    };
    let name = match id >> 12 {
        2 => campaign::RING_NAMES.get(index),
        3 => campaign::TRINKET_NAMES.get(index),
        4 => campaign::AMULET_NAMES.get(index),
        _ => None,
    };
    let display = legend
        .map(|_| LEGEND_DISPLAY)
        .or_else(|| name.and_then(|name| super::super::jewel_icons::display(name)));
    let tint = if seen {
        Color::WHITE
    } else {
        Color::srgba(0.3, 0.3, 0.3, 0.65)
    };
    ui.spawn(Node {
        width: Val::Px(size),
        height: Val::Px(size),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|icon| {
        if let Some(display) = display {
            icon.spawn((
                super::super::gear_icons::GearIcon(display, tint),
                ImageNode {
                    color: Color::NONE,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        } else {
            label(icon, font, "?", size * 0.55, DIM);
        }
        if let Some(key) = legend {
            icon.spawn((
                hud::SpellIcon(data::ability(key).spell),
                ImageNode::default(),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(size * 0.4),
                    height: Val::Px(size * 0.4),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(GOLD),
                ZIndex(1),
            ));
        }
    });
}

fn entry_cell(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    id: u16,
    size: f32,
    seen: bool,
    selected: bool,
) {
    ui.spawn((
        Button,
        Act::Codex(id),
        CatalogCell { id, seen },
        Node {
            width: Val::Px(size + 4.0),
            height: Val::Px(size + 4.0),
            flex_shrink: 0.0,
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(if selected {
            GOLD
        } else if seen {
            Color::srgb(0.6, 0.6, 0.6)
        } else {
            Color::srgb(0.17, 0.15, 0.10)
        }),
    ))
    .with_children(|cell| entry_icon(cell, font, id, size, seen));
}

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
                                let id = codex_legend(class_i, slot);
                                entry_cell(
                                    icons,
                                    font,
                                    id,
                                    layout.body_size * 2.7,
                                    seen,
                                    armory.codex_entry == Some(id),
                                );
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
                        .with_children(|title_column| {
                            label(title_column, font, title, layout.body_size, GOLD);
                            label(
                                title_column,
                                font,
                                format!("{found}/{total}"),
                                layout.body_size - 2.0,
                                if found == total { GREEN } else { DIM },
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
                            for index in 0..names.len() {
                                let id = codex_jewel(kind, index);
                                entry_cell(
                                    icons,
                                    font,
                                    id,
                                    layout.body_size * 2.7,
                                    account.shared_jewel_seen(kind, index),
                                    armory.codex_entry == Some(id),
                                );
                            }
                        });
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
            details
                .spawn((DetailArt(armory.codex_entry), Node::default()))
                .with_children(|image| {
                    if let Some(id) = armory.codex_entry {
                        entry_icon(image, font, id, layout.title_size * 3.0, true);
                    }
                });
            details.spawn((
                DetailText(false),
                hud::text_bundle(font, detail_text(armory, account), layout.body_size, DIM),
            ));
            details.spawn((
                DetailText(true),
                hud::text_bundle(font, effect_text(armory), layout.body_size, GREEN),
            ));
        });
    });
}

const INTRO: &str = "Every ring, trinket and amulet offered to any hero is remembered, even if you do not take it. Dim icons have not been offered yet. Select an entry to read it.\n\nShared jewelry lasts one run. Legendary amulets are cataloged per spell; their bonuses apply only to their associated spell.";

#[derive(Component)]
pub(in crate::survivors) struct DetailText(bool);

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

fn effect_text(armory: &Armory) -> String {
    let Some(id) = armory.codex_entry else {
        return String::new();
    };
    let index = usize::from(id & 0x0fff);
    let kind = match id >> 12 {
        1 => {
            let Some(class) = CLASSES.get(index >> 6) else {
                return String::new();
            };
            let keys = class.ability_keys();
            let Some(key) = keys.get(index & 0x3f) else {
                return String::new();
            };
            campaign::JewelKind::Legend(key)
        }
        2 if index < campaign::RING_NAMES.len() => campaign::JewelKind::Ring(index as u8),
        3 if index < campaign::TRINKET_NAMES.len() => campaign::JewelKind::Trinket(index as u8),
        4 if index < campaign::AMULET_NAMES.len() => campaign::JewelKind::Amulet(index as u8),
        _ => return String::new(),
    };
    let mut worn = campaign::Worn::default();
    worn.equip(kind);
    let (heading, bonus) = match hud::card_growth::legend_effect(worn) {
        Some((key, bonus)) => (format!("{} only", super::spell_title(key)), bonus),
        None => ("Bonuses when worn alone".into(), worn.bonus()),
    };
    let lines = hud::card_growth::jewelry_changes(campaign::Bonus::default(), bonus);
    format!(
        "{heading}\n{}",
        lines
            .into_iter()
            .map(|(text, _)| text)
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Selection changes text only: keep the catalog entities and their scroll offset alive.
pub(in crate::survivors) fn refresh_details(
    armory: Res<Armory>,
    account: Res<Account>,
    mut details: Query<(&DetailText, &mut Text)>,
    mut cells: Query<(&CatalogCell, &mut BorderColor)>,
    mut art: Query<(Entity, &mut DetailArt)>,
    mut commands: Commands,
    font: Res<super::super::hud::HudFont>,
    layout: Res<super::super::UiLayout>,
) {
    if details.is_empty() || (!armory.is_changed() && !account.is_changed()) {
        return;
    }
    for (cell, mut border) in &mut cells {
        *border = BorderColor::all(if armory.codex_entry == Some(cell.id) {
            GOLD
        } else if cell.seen {
            Color::srgb(0.6, 0.6, 0.6)
        } else {
            Color::srgb(0.17, 0.15, 0.10)
        });
    }
    for (entity, mut previous) in &mut art {
        if previous.0 != armory.codex_entry {
            previous.0 = armory.codex_entry;
            commands.entity(entity).despawn_related::<Children>();
            if let Some(id) = armory.codex_entry {
                commands.entity(entity).with_children(|image| {
                    entry_icon(image, &font.0, id, layout.value.title_size * 3.0, true)
                });
            }
        }
    }
    let value = detail_text(&armory, &account);
    let effect = effect_text(&armory);
    for (kind, mut text) in &mut details {
        let value = if kind.0 { &effect } else { &value };
        if text.0 != *value {
            text.0.clone_from(value);
        }
    }
}

fn discovery_total(account: &Account) -> (usize, usize) {
    (0..CLASSES.len())
        .map(|class| account.legendary_progress(class))
        .chain((2..=4).map(|kind| account.shared_jewel_progress(kind)))
        .fold((0, 0), |(found, total), (f, t)| (found + f, total + t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_updates_art_and_highlight_without_replacing_the_catalog_scroll() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .insert_resource(super::super::super::account::blank())
            .insert_resource(hud::HudFont(Handle::default()))
            .init_resource::<super::super::super::UiLayout>()
            .add_systems(Update, refresh_details);
        let scroll = app
            .world_mut()
            .spawn(ScrollPosition(Vec2::new(0.0, 250.0)))
            .id();
        app.world_mut().spawn((DetailText(false), Text::default()));
        let art = app
            .world_mut()
            .spawn((DetailArt(None), Node::default()))
            .id();
        let id = codex_legend(0, 0);
        let cell = app
            .world_mut()
            .spawn((CatalogCell { id, seen: false }, BorderColor::all(DIM)))
            .id();
        app.world_mut().resource_mut::<Armory>().codex_entry = Some(id);
        app.update();
        assert_eq!(
            *app.world().get::<BorderColor>(cell).unwrap(),
            BorderColor::all(GOLD)
        );
        let first_image = app.world().get::<Children>(art).unwrap()[0];
        let icons = app
            .world_mut()
            .query::<&hud::SpellIcon>()
            .iter(app.world())
            .count();
        assert_eq!(icons, 1);
        app.world_mut().resource_mut::<Armory>().codex_entry = Some(codex_jewel(2, 0));
        app.update();
        assert_ne!(app.world().get::<Children>(art).unwrap()[0], first_image);
        assert_ne!(
            *app.world().get::<BorderColor>(cell).unwrap(),
            BorderColor::all(GOLD)
        );
        assert_eq!(
            app.world().get::<ScrollPosition>(scroll).unwrap().0.y,
            250.0
        );
        assert_eq!(
            app.world_mut()
                .query::<&hud::SpellIcon>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn catalog_effects_use_worn_bonuses_and_reject_invalid_entries() {
        let mut armory = Armory::default();
        assert!(effect_text(&armory).is_empty());
        armory.codex_entry = Some(codex_jewel(2, 0));
        let ring = effect_text(&armory);
        assert!(ring.contains("Damage bonus 0 » 2%"), "{ring}");
        assert!(ring.contains("Critical chance bonus 0 » 1%"), "{ring}");
        armory.codex_entry = Some(codex_jewel(4, 0));
        assert!(effect_text(&armory).contains("Damage reduction bonus 0 » 3%"));
        armory.codex_entry = Some(codex_jewel(4, campaign::AMULET_NAMES.len()));
        assert!(effect_text(&armory).is_empty());
        armory.codex_entry = Some(codex_legend(0, 0));
        assert!(!effect_text(&armory).contains("Direct-attack targets"));
        armory.codex_entry = Some(codex_legend(1, 0));
        let legend = effect_text(&armory);
        assert!(
            legend.contains("Direct-attack targets bonus 0 » 1"),
            "{legend}"
        );
    }
}
