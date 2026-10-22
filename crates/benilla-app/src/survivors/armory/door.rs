//! The Armory front door: one glue frame, three columns, and the tab row. A tab that does not
//! have its own page yet draws the draft for that topic inside this frame.

use bevy::prelude::*;
use bevy::ui_render::ui_material::MaterialNode;

use super::super::account::Account;
use super::super::campaign::{self, ZONES};
use super::super::data::{self, ClassDef, CLASSES};
use super::super::hud::{self, SpellIcon};
use super::super::renown::Renown;
use super::super::stash::Stash;
use super::super::{Selection, BASE_HP};
use super::{Act, Armory, Page};
use crate::char_select::race_name;
use crate::glue::art::{class_tc, tc_rect, GlueArt, DIM, GOLD, NAME_EDGE};
use crate::glue::backdrop::{backdrop_border, tiled_bg_node};
use crate::glue::widgets::{
    abs, glue_button, outlined_text, outlined_text_centered, overlay, GlueBtnKind, GlueText,
    Hilight, LockHighlight,
};

const FILL: Color = Color::srgba(0.05, 0.04, 0.02, 0.92);
const FRAME: Color = Color::srgb(0.78, 0.62, 0.22);
const BONUS: Color = Color::srgb(0.45, 0.9, 0.4);

pub(super) fn build(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
    note: &str,
) {
    header(ui, art, font);
    tabs(ui, art, font, armory.page, account.unspent());
    if !note.is_empty() {
        line(ui, font, note, 13.0, GOLD);
    }
    if armory.page == Page::Door {
        columns(ui, art, font, account, selection);
        enter_bar(ui, art, font, account, selection);
    } else {
        panel(ui, art, |body| {
            super::fill_draft(body, font, armory, account, selection, renown, stash)
        });
    }
}

fn header(ui: &mut ChildSpawnerCommands, art: &GlueArt, font: &Handle<Font>) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(40.0),
        ..default()
    })
    .with_children(|row| {
        outlined_text_centered(
            row,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            (),
            (),
            GlueText {
                text: "Azeroth Survivors",
                size: 26.0,
                color: GOLD,
                wrap: false,
            },
            font,
            1.0,
        );
        row.spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|b| {
            glue_button(
                b,
                art,
                font,
                Act::Options,
                "Options",
                96.0,
                28.0,
                GlueBtnKind::Small,
                1.0,
            );
            glue_button(
                b,
                art,
                font,
                Act::Quit,
                "Quit",
                72.0,
                28.0,
                GlueBtnKind::Small,
                1.0,
            );
        });
    });
}

fn tabs(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    page: Page,
    unspent: u32,
) {
    let rewards = format!("Rewards ({unspent})");
    let tabs = [
        ("Character", Page::Heroes),
        ("Collection", Page::Collection),
        ("Drop Tables", Page::DropTables),
        ("Codex", Page::Codex),
        ("Talents", Page::Talents),
        ("Achievements", Page::Achievements),
        (rewards.as_str(), Page::Rewards),
    ];
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        column_gap: Val::Px(4.0),
        row_gap: Val::Px(4.0),
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    })
    .with_children(|row| {
        for (label, tab) in tabs {
            tab_button(row, art, font, label, Act::Page(tab), page == tab);
        }
        glue_button(
            row,
            art,
            font,
            Act::Page(Page::Door),
            "Done",
            72.0,
            28.0,
            GlueBtnKind::Small,
            1.0,
        );
    });
}

fn tab_button(
    row: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    label: &str,
    act: Act,
    on: bool,
) {
    let width = (label.len() as f32 * 7.2 + 28.0).clamp(78.0, 168.0);
    row.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|col| {
        glue_button(
            col,
            art,
            font,
            act,
            label,
            width,
            28.0,
            GlueBtnKind::Small,
            1.0,
        );
        col.spawn((
            Node {
                width: Val::Percent(70.0),
                height: Val::Px(2.0),
                margin: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(if on { GOLD } else { Color::NONE }),
        ));
    });
}

fn columns(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        min_height: Val::Px(0.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|row| {
        panel(row, art, |body| {
            class_column(body, art, font, account, selection)
        });
        panel(row, art, |body| {
            hero_column(body, art, font, account, selection)
        });
        panel(row, art, |body| zone_column(body, font, account, selection));
    });
}

fn panel(
    parent: &mut ChildSpawnerCommands,
    art: &GlueArt,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            height: Val::Percent(100.0),
            min_height: Val::Px(0.0),
            ..default()
        })
        .with_children(|frame| {
            if let (Some(bg), Some(border)) = (&art.tooltip_bg, &art.name_border) {
                frame.spawn((
                    tiled_bg_node(bg.clone(), NAME_EDGE, 1.0, FILL),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                ));
                backdrop_border(frame, border, NAME_EDGE, FRAME);
            } else {
                frame.spawn((
                    BackgroundColor(FILL),
                    BorderColor::all(FRAME),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                ));
            }
            frame
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    overflow: Overflow::scroll_y(),
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(body);
        });
}

fn class_column(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    heading(body, font, "Class");
    let class_i = selection
        .class
        .unwrap_or(0)
        .min(CLASSES.len().saturating_sub(1));
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(6.0),
        row_gap: Val::Px(2.0),
        ..default()
    })
    .with_children(|grid| {
        for (i, class) in CLASSES.iter().enumerate() {
            class_icon(grid, art, font, i, class, selection.class == Some(i));
        }
    });
    let class = &CLASSES[class_i];
    line(body, font, class.name, 16.0, hud::rgb(class.color));
    line(body, font, fantasy(class_i), 13.0, Color::WHITE);
    let (level, clears) = super::hero_gate(account, selection);
    let open = (0..ZONES.len())
        .filter(|i| campaign::zone_open(*i, level, &clears))
        .count();
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(4.0),
        margin: UiRect::vertical(Val::Px(4.0)),
        ..default()
    })
    .with_children(|row| {
        stat(row, font, "Health", &class_health(class.id).to_string());
        stat(row, font, "Starts with", &super::spell_title(class.start));
        stat(
            row,
            font,
            "Zones open",
            &format!("{open} of {}", ZONES.len()),
        );
    });
    line(body, font, "Spell pool", 12.0, GOLD);
    spell_row(body, class);
}

fn class_icon(
    grid: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    index: usize,
    class: &ClassDef,
    on: bool,
) {
    grid.spawn(Node {
        width: Val::Px(58.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|col| {
        col.spawn((
            Act::PickClass(index),
            Button,
            LockHighlight(on),
            Node {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                ..default()
            },
        ))
        .with_children(|b| {
            if let Some(shadow) = &art.icon_shadow {
                b.spawn((
                    ImageNode::new(shadow.clone()),
                    abs(1.0, -6.0, -6.0, 64.0, 64.0),
                ));
            }
            if let Some((sheet, size)) = &art.classes {
                if let Some(tc) = class_tc(class.id) {
                    b.spawn((
                        ImageNode {
                            image: sheet.clone(),
                            rect: Some(tc_rect(*size, tc)),
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                }
            }
            if let Some(hilight) = &art.hilight {
                b.spawn((
                    Hilight,
                    Visibility::Hidden,
                    MaterialNode(hilight.clone()),
                    overlay(),
                ));
            }
        });
        outlined_text_centered(
            col,
            Node {
                width: Val::Px(58.0),
                ..default()
            },
            (),
            (),
            GlueText {
                text: class.name,
                size: 11.0,
                color: if on { GOLD } else { DIM },
                wrap: false,
            },
            font,
            1.0,
        );
    });
}

fn hero_column(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    heading(body, font, "Hero");
    let class_i = selection.class.unwrap_or(0);
    let class = CLASSES.get(class_i);
    let hero = shown_hero(account, selection);
    let (name, name_color) = match (hero, class) {
        (Some(hero), Some(class)) => (
            format!("{} {}", hero.first, hero.last),
            hud::rgb(class.color),
        ),
        _ => ("No hero of this class".to_string(), DIM),
    };
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        ..default()
    })
    .with_children(|row| {
        line(row, font, &name, 18.0, name_color);
        glue_button(
            row,
            art,
            font,
            Act::RandomName,
            "Random",
            88.0,
            26.0,
            GlueBtnKind::Small,
            1.0,
        );
    });
    let look = selection
        .look
        .and_then(|i| selection.looks.get(i))
        .map(|&(_, race, sex)| (race, sex));
    let look_line = match look {
        Some((race, sex)) => format!("{} {} {}", faction(race), race_name(race), sex_name(sex)),
        None => "No look chosen".to_string(),
    };
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        line(row, font, &look_line, 13.0, Color::WHITE);
        glue_button(
            row,
            art,
            font,
            Act::Look,
            "Look",
            72.0,
            26.0,
            GlueBtnKind::Small,
            1.0,
        );
    });
    heading(body, font, "Equipment");
    let owned = gear_lines(body, font, account, class_i);
    let total = campaign::SLOTS.len();
    line(
        body,
        font,
        &format!("Collection  {owned} of {total}"),
        13.0,
        GOLD,
    );
    bar(body, owned as f32 / total as f32);
    let bonus = account.bonus_picked(class_i, selection.character);
    line(body, font, &bonus_line(bonus), 12.0, BONUS);
    let spent = hero
        .map(|c| c.talents.iter().map(|n| u32::from(*n)).sum::<u32>())
        .unwrap_or(0);
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(6.0),
        margin: UiRect::top(Val::Px(6.0)),
        ..default()
    })
    .with_children(|row| {
        door_button(
            row,
            art,
            font,
            Act::Page(Page::Heroes),
            "Armory",
            "Heroes, look and gear",
        );
        door_button(
            row,
            art,
            font,
            Act::Page(Page::Talents),
            "Talents",
            &format!("{spent} of {}", campaign::TALENT_CAP),
        );
    });
    if let Some((race, _)) = look {
        line(body, font, race_name(race), 14.0, GOLD);
        line(body, font, "No permanent bonus.", 12.0, DIM);
    }
}

fn door_button(
    row: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    act: Act,
    caption: &str,
    under: &str,
) {
    row.spawn(Node {
        flex_grow: 1.0,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(2.0),
        ..default()
    })
    .with_children(|col| {
        glue_button(
            col,
            art,
            font,
            act,
            caption,
            150.0,
            32.0,
            GlueBtnKind::Small,
            1.0,
        );
        line(col, font, under, 11.0, DIM);
    });
}

fn zone_column(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    heading(body, font, "Zone");
    let (level, clears) = super::hero_gate(account, selection);
    let map_name = selection.map_def().name;
    for (index, zone) in ZONES.iter().enumerate() {
        let on = zone.map == map_name;
        let reason = campaign::zone_lock_reason(index, level, &clears);
        body.spawn((
            Act::Zone(index),
            Button,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(if on {
                Color::srgba(0.85, 0.65, 0.2, 0.28)
            } else {
                Color::NONE
            }),
        ))
        .with_children(|row| {
            let color = if reason.is_some() { DIM } else { GOLD };
            line(row, font, zone.name, 14.0, color);
            if let Some(reason) = &reason {
                line(row, font, reason, 11.0, DIM);
            }
        });
    }
    if let Some((index, zone)) = ZONES.iter().enumerate().find(|(_, z)| z.map == map_name) {
        line(body, font, zone.name, 16.0, GOLD);
        if zone.name != zone.map {
            line(body, font, zone.map, 12.0, DIM);
        }
        if let Some(reason) = campaign::zone_lock_reason(index, level, &clears) {
            line(body, font, &reason, 13.0, Color::srgb(0.9, 0.45, 0.35));
        }
        let elites = zone.elites.join(", ");
        line(
            body,
            font,
            &format!("{}. {elites}.", zone.boss),
            13.0,
            Color::WHITE,
        );
        if let Some(map) = data::MAPS.iter().find(|m| m.name == zone.map) {
            if !map.victory.is_empty() {
                line(body, font, map.victory, 12.0, DIM);
            }
        }
    } else {
        let map = selection.map_def();
        line(body, font, map.name, 16.0, GOLD);
        line(
            body,
            font,
            "Not on the campaign path. It stays open.",
            12.0,
            DIM,
        );
        if !map.victory.is_empty() {
            line(body, font, map.victory, 12.0, Color::WHITE);
        }
    }
}

fn enter_bar(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let (title, under) = enter_copy(account, selection);
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(12.0),
        margin: UiRect::top(Val::Px(8.0)),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|col| {
            glue_button(
                col,
                art,
                font,
                Act::RunOptions,
                "Run Options",
                130.0,
                36.0,
                GlueBtnKind::Small,
                1.0,
            );
            line(col, font, "Normal difficulty", 11.0, DIM);
        });
        row.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|col| {
            glue_button(
                col,
                art,
                font,
                Act::Enter,
                &title,
                280.0,
                44.0,
                GlueBtnKind::Normal,
                1.0,
            );
            line(col, font, &under, 12.0, GOLD);
        });
    });
}

fn enter_copy(account: &Account, selection: &Selection) -> (String, String) {
    let (level, clears) = super::hero_gate(account, selection);
    let map = selection.map_def();
    if let Some((index, zone)) = ZONES.iter().enumerate().find(|(_, z)| z.map == map.name) {
        let under = campaign::zone_lock_reason(index, level, &clears)
            .unwrap_or_else(|| format!("{} is the last fight.", zone.boss));
        (format!("Enter {}", zone.name), under)
    } else {
        (format!("Enter {}", map.name), map.victory.to_string())
    }
}

fn gear_lines(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    class: usize,
) -> usize {
    let progress = account.classes.get(class);
    let gear = progress.map(|p| p.gear).unwrap_or([false; 10]);
    let weapon = progress.map(|p| p.legendary_weapon).unwrap_or(false);
    let names = campaign::GEAR_NAMES.get(class);
    let two_hand = campaign::two_handed(class);
    let mut owned = 0;
    for slot in 0..campaign::SLOTS.len() {
        let have = gear.get(slot).copied().unwrap_or(false);
        let item = names.map(|row| row[slot]).unwrap_or("Unknown");
        let main_filled = weapon || gear.get(8).copied().unwrap_or(false);
        let occupied = slot == 9 && two_hand && main_filled;
        let (text, color) = if occupied {
            (
                format!(
                    "{} — filled by the two-handed weapon",
                    campaign::SLOTS[slot]
                ),
                DIM,
            )
        } else if have {
            owned += 1;
            let quality = campaign::slot_quality(slot)
                .map(quality_color)
                .unwrap_or(Color::WHITE);
            (format!("{} — {item}", campaign::SLOTS[slot]), quality)
        } else {
            (format!("{} — {item}", campaign::SLOTS[slot]), DIM)
        };
        line(body, font, &text, 12.0, color);
    }
    if let Some(weapon_name) = names.map(|row| row[10]) {
        let (text, color) = if weapon {
            (format!("Legendary weapon — {weapon_name}"), GOLD)
        } else {
            (format!("Legendary weapon — {weapon_name}"), DIM)
        };
        line(body, font, &text, 12.0, color);
    }
    owned
}

fn spell_row(body: &mut ChildSpawnerCommands, class: &ClassDef) {
    let mut keys = Vec::new();
    for key in std::iter::once(class.start).chain(class.pool.iter().copied()) {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(4.0),
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|row| {
        for key in keys {
            row.spawn((
                Node {
                    width: Val::Px(28.0),
                    height: Val::Px(28.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(FRAME),
                ImageNode {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                    ..default()
                },
                SpellIcon(data::ability(key).spell),
            ));
        }
    });
}

fn stat(row: &mut ChildSpawnerCommands, font: &Handle<Font>, title: &str, value: &str) {
    row.spawn((
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(3.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(FRAME),
    ))
    .with_children(|cell| {
        line(cell, font, title, 10.0, DIM);
        line(cell, font, value, 13.0, GOLD);
    });
}

fn bar(body: &mut ChildSpawnerCommands, frac: f32) {
    let width = (frac.clamp(0.0, 1.0) * 100.0).max(0.0);
    body.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(8.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.16, 0.08, 1.0)),
    ))
    .with_children(|track| {
        track.spawn((
            Node {
                width: Val::Percent(width),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(GOLD),
        ));
    });
}

fn heading(body: &mut ChildSpawnerCommands, font: &Handle<Font>, text: &str) {
    outlined_text_centered(
        body,
        Node {
            width: Val::Percent(100.0),
            margin: UiRect::top(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        (),
        (),
        GlueText {
            text,
            size: 15.0,
            color: GOLD,
            wrap: false,
        },
        font,
        1.0,
    );
}

fn line(body: &mut ChildSpawnerCommands, font: &Handle<Font>, text: &str, size: f32, color: Color) {
    outlined_text(
        body,
        Node {
            max_width: Val::Percent(100.0),
            ..default()
        },
        (),
        (),
        GlueText {
            text,
            size,
            color,
            wrap: true,
        },
        font,
        1.0,
    );
}

fn shown_hero<'a>(
    account: &'a Account,
    selection: &Selection,
) -> Option<&'a super::super::account::Character> {
    let class = selection.class?;
    selection
        .character
        .and_then(|id| account.character(id))
        .filter(|c| c.class == class)
        .or_else(|| account.characters.iter().find(|c| c.class == class))
}

fn class_health(id: u8) -> u32 {
    let mut hp = BASE_HP;
    match id {
        1 | 2 => hp += 40.0,
        3 | 4 | 7 | 11 => hp += 15.0,
        _ => {}
    }
    hp.round() as u32
}

fn fantasy(index: usize) -> &'static str {
    match index {
        0 => "Opens with Cleave, then shouts, whirlwinds and executes.",
        1 => "Opens with Hammer of Wrath. Consecrates the ground and can shield himself.",
        2 => "Opens with Arcane Shot, then stings, traps and a volley.",
        3 => "Opens with Sinister Strike, in close, and finishes what is left.",
        4 => "Opens with Smite. Shields himself and screams the pack away.",
        5 => "Opens with Lightning Bolt, shocks, and drops totems.",
        6 => "Opens with Frostbolt, then fire, frost and the ground.",
        7 => "Opens with Shadow Bolt, then burns and curses the pack.",
        _ => "Opens with Wrath, then moonfire, thorns and the storm.",
    }
}

fn faction(race: u8) -> &'static str {
    match race {
        1 | 3 | 4 | 7 => "Alliance",
        2 | 5 | 6 | 8 => "Horde",
        _ => "",
    }
}

fn sex_name(sex: u8) -> &'static str {
    if sex == 0 {
        "Male"
    } else {
        "Female"
    }
}

fn quality_color(quality: campaign::Quality) -> Color {
    match quality {
        campaign::Quality::Poor => hud::rgb(0x9d9d9d),
        campaign::Quality::Common => Color::WHITE,
        campaign::Quality::Uncommon => hud::rgb(0x1eff00),
        campaign::Quality::Rare => hud::rgb(0x0070dd),
        campaign::Quality::Epic => hud::rgb(0xa335ee),
        campaign::Quality::Legendary => hud::rgb(0xff8000),
    }
}

fn bonus_line(bonus: campaign::Bonus) -> String {
    let mut parts = Vec::new();
    let pct = |v: f32| format!("{:+.0}%", v * 100.0);
    if bonus.armor.abs() > 0.01 {
        parts.push(format!("{:+.0} armor", bonus.armor));
    }
    if bonus.might.abs() > 0.0001 {
        parts.push(format!("{} damage", pct(bonus.might)));
    }
    if bonus.crit.abs() > 0.0001 {
        parts.push(format!("{} crit", pct(bonus.crit)));
    }
    if bonus.multistrike.abs() > 0.0001 {
        parts.push(format!("{} multistrike", pct(bonus.multistrike)));
    }
    if bonus.speed.abs() > 0.0001 {
        parts.push(format!("{} speed", pct(bonus.speed)));
    }
    if bonus.leech.abs() > 0.0001 {
        parts.push(format!("{} leech", pct(bonus.leech)));
    }
    if bonus.area.abs() > 0.0001 {
        parts.push(format!("{} area", pct(bonus.area)));
    }
    if bonus.crit_damage.abs() > 0.0001 {
        parts.push(format!("{} crit damage", pct(bonus.crit_damage)));
    }
    if bonus.reduction.abs() > 0.0001 {
        parts.push(format!("{} damage reduction", pct(bonus.reduction)));
    }
    if bonus.cooldown.abs() > 0.0001 {
        parts.push(format!("{} cooldown", pct(bonus.cooldown)));
    }
    if bonus.gold.abs() > 0.0001 {
        parts.push(format!("{} gold", pct(bonus.gold)));
    }
    if bonus.growth.abs() > 0.0001 {
        parts.push(format!("{} experience", pct(bonus.growth)));
    }
    if bonus.luck.abs() > 0.01 {
        parts.push(format!("{:+.0} luck", bonus.luck));
    }
    if bonus.targets > 0 {
        parts.push(format!("+{} targets", bonus.targets));
    }
    if parts.is_empty() {
        "No bonuses yet.".into()
    } else {
        parts.join("   ")
    }
}
