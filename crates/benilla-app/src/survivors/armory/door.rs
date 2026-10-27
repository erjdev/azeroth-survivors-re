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
use crate::glue::art::{class_tc, race_tc, tc_rect, GlueArt, DIM, GOLD, NAME_EDGE};
use crate::glue::backdrop::{backdrop_border, tiled_bg_node};
use crate::glue::widgets::{
    outlined_text, outlined_text_centered, overlay, GlueText, Hilight, LockHighlight,
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
    talents: Option<&benilla_formats::TalentCatalog>,
    layout: &survivors_core::layout::Layout,
    features: Option<&super::super::appearance::Options>,
    talent_scroll: Vec2,
    compact_collection: bool,
) {
    if armory.page == Page::Door {
        header(ui, font);
        tabs(ui, font, armory.page, account.unspent());
    } else {
        armory_header(ui, font, armory, account, selection);
    }
    if !note.is_empty() && (armory.page != Page::Codex || account.recovery_notice().is_some()) {
        line(ui, font, note, 13.0, GOLD);
    }
    if armory.page == Page::Door {
        columns(
            ui,
            art,
            font,
            armory,
            account,
            selection,
            compact_collection,
        );
        enter_bar(ui, font, armory, account, selection);
    } else if armory.page == Page::Talents {
        super::talent_page::build(ui, font, account, selection, talents, talent_scroll, layout);
    } else if armory.page == Page::Rewards {
        super::rewards_page(ui, font, armory, account);
    } else if armory.page == Page::Achievements {
        super::achievements(ui, font, armory, account);
    } else if armory.page == Page::Heroes {
        super::character::build(
            ui,
            art,
            font,
            armory,
            account,
            selection,
            renown,
            stash,
            layout,
            features,
            compact_collection,
        );
    } else if armory.page == Page::Collection {
        super::collection::build(
            ui,
            art,
            font,
            armory,
            account,
            selection,
            layout,
            compact_collection,
        );
    } else if armory.page == Page::Codex {
        super::codex_page::build(
            ui,
            font,
            armory,
            account,
            selection,
            layout,
            compact_collection,
        );
    } else if armory.page == Page::DropTables {
        super::drop_page::build(
            ui,
            art,
            font,
            armory,
            account,
            selection,
            layout,
            compact_collection,
        );
    } else {
        panel(ui, art, false, None, |body| {
            super::fill_draft(
                body, font, armory, account, selection, renown, stash, talents,
            )
        });
    }
}

fn armory_header(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(6.0),
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            hud::text_bundle(font, "Armory", 30.0, GOLD),
            super::typography::DisplayHeading,
            Node {
                flex_basis: Val::Px(160.0),
                flex_grow: 1.0,
                ..default()
            },
        ));
        row.spawn(Node {
            flex_basis: Val::Px(740.0),
            flex_grow: 4.0,
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|navigation| tabs(navigation, font, armory.page, account.unspent()));
    });
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(4.0),
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    })
    .with_children(|row| {
        let class_index = super::picked_class(account, selection);
        let class = &CLASSES[class_index];
        let count = account
            .classes
            .get(class_index)
            .map_or(0, |progress| campaign::owned_pieces(&progress.pieces));
        row.spawn((
            hud::text_bundle(font, "", 12.0, DIM),
            Node {
                flex_basis: Val::Px(360.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            },
        ))
        .with_children(|subtitle| {
            for (part, color) in [
                (
                    class.name.to_owned(),
                    Color::srgb_u8(
                        (class.color >> 16) as u8,
                        (class.color >> 8) as u8,
                        class.color as u8,
                    ),
                ),
                (
                    format!(
                        " · Common–Legendary: {count}/{} · gear and looks are kept per class",
                        campaign::PIECES
                    ),
                    DIM,
                ),
            ] {
                subtitle.spawn((
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
        row.spawn(Node {
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|controls| {
            tab_button(controls, font, "Options", Act::Options, false);
            tab_button(controls, font, "Quit", Act::Quit, false);
        });
    });
}

fn header(ui: &mut ChildSpawnerCommands, font: &Handle<Font>) {
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
            super::typography::DisplayHeading,
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
            tab_button(b, font, "Options", Act::Options, false);
            tab_button(b, font, "Quit", Act::Quit, false);
        });
    });
}

fn tabs(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, page: Page, unspent: u32) {
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
            tab_button(row, font, label, Act::Page(tab), page == tab);
        }
        if page != Page::Door {
            tab_button(row, font, "Done", Act::Page(Page::Door), false);
        }
    });
}

fn tab_button(
    row: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    act: Act,
    on: bool,
) {
    super::controls::button(
        row,
        font,
        label,
        act,
        on,
        Node {
            min_height: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            max_width: Val::Percent(100.0),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        },
    );
}

fn columns(
    ui: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    compact: bool,
) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        flex_basis: Val::Px(0.0),
        min_height: Val::Px(0.0),
        ..default()
    })
    .with_children(|frame| {
        let mut content = frame.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::right(Val::Px(if compact { 16.0 } else { 0.0 })),
            flex_direction: if compact {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            },
            overflow: if compact {
                Overflow::scroll_y()
            } else {
                Overflow::visible()
            },
            row_gap: Val::Px(8.0),
            column_gap: Val::Px(8.0),
            ..default()
        });
        if compact {
            content.insert((
                super::collection::CollectionScroll,
                super::collection::CompactScroll::Starter,
                bevy::ui::RelativeCursorPosition::default(),
                ScrollPosition(armory.starter_scroll),
            ));
        }
        let viewport = content.id();
        content.with_children(|row| {
            panel(
                row,
                art,
                compact,
                Some((0, armory.starter_panel_scroll[0])),
                |body| class_column(body, art, font, account, selection, compact),
            );
            panel(
                row,
                art,
                compact,
                Some((1, armory.starter_panel_scroll[1])),
                |body| hero_column(body, art, font, armory, account, selection),
            );
            panel(
                row,
                art,
                compact,
                Some((2, armory.starter_panel_scroll[2])),
                |body| zone_column(body, font, armory, account, selection),
            );
        });
        if compact {
            super::scrollbar::spawn(frame, viewport);
        }
    });
}

fn panel(
    parent: &mut ChildSpawnerCommands,
    art: &GlueArt,
    compact: bool,
    scroll: Option<(usize, Vec2)>,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            flex_grow: if compact { 0.0 } else { 1.0 },
            flex_shrink: 0.0,
            flex_basis: if compact { Val::Auto } else { Val::Px(0.0) },
            height: if compact {
                Val::Auto
            } else {
                Val::Percent(100.0)
            },
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
            let mut content = frame.spawn(Node {
                width: Val::Percent(100.0),
                height: if compact {
                    Val::Auto
                } else {
                    Val::Percent(100.0)
                },
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                overflow: if compact {
                    Overflow::visible()
                } else {
                    Overflow::scroll_y()
                },
                row_gap: Val::Px(4.0),
                ..default()
            });
            if !compact {
                content.insert((
                    super::collection::CollectionScroll,
                    bevy::ui::RelativeCursorPosition::default(),
                    ScrollPosition(scroll.map_or(Vec2::ZERO, |(_, position)| position)),
                ));
            }
            if let Some((index, _)) = scroll.filter(|_| !compact) {
                content.insert(super::collection::CompactScroll::StarterPanel(index));
            }
            let viewport = content.id();
            content.with_children(body);
            if !compact {
                super::scrollbar::spawn(frame, viewport);
            }
        });
}

fn class_column(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
    compact: bool,
) {
    heading(body, font, "Class");
    let class_i = selection
        .class
        .unwrap_or(0)
        .min(CLASSES.len().saturating_sub(1));
    body.spawn(Node {
        width: Val::Percent(if compact { 100.0 } else { 90.0 }),
        max_width: if compact { Val::Auto } else { Val::Px(320.0) },
        align_self: AlignSelf::Center,
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(6.0),
        row_gap: Val::Px(if compact { 2.0 } else { 10.0 }),
        ..default()
    })
    .with_children(|grid| {
        for (i, class) in CLASSES.iter().enumerate() {
            class_icon(
                grid,
                art,
                font,
                i,
                class,
                selection.class == Some(i),
                compact,
            );
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
    compact: bool,
) {
    grid.spawn(Node {
        width: if compact {
            Val::Px(58.0)
        } else {
            Val::Percent(30.0)
        },
        flex_shrink: 0.0,
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
                width: if compact {
                    Val::Px(48.0)
                } else {
                    Val::Percent(64.0)
                },
                min_width: Val::Px(40.0),
                max_width: Val::Px(64.0),
                aspect_ratio: Some(1.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|b| {
            if let Some(shadow) = &art.icon_shadow {
                b.spawn((
                    ImageNode::new(shadow.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(-12.5),
                        top: Val::Percent(-12.5),
                        width: Val::Percent(133.33),
                        height: Val::Percent(133.33),
                        ..default()
                    },
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
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            (),
            (),
            GlueText {
                text: class.name,
                size: 11.0,
                color: if on { hud::rgb(class.color) } else { DIM },
                wrap: false,
            },
            font,
            1.0,
        );
    });
}

/// Decorative overlays follow the selected body and class, leaving portrait drag input intact.
fn portrait_badges(
    frame: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    class: &ClassDef,
    race: u8,
) {
    let color = if matches!(race, 1 | 3 | 4 | 7) {
        Color::srgb(0.3, 0.6, 1.0)
    } else {
        Color::srgb(1.0, 0.3, 0.2)
    };
    frame
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(3.0),
                top: Val::Percent(3.0),
                max_width: Val::Percent(94.0),
                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.01, 0.02, 0.95)),
            BorderColor::all(color),
        ))
        .with_children(|badge| {
            badge.spawn(hud::text_bundle(
                font,
                faction(race).to_uppercase(),
                8.0,
                color,
            ));
        });
    if let (Some((sheet, size)), Some(tc)) = (&art.classes, class_tc(class.id)) {
        frame.spawn((
            ImageNode {
                image: sheet.clone(),
                rect: Some(tc_rect(*size, tc)),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(3.0),
                bottom: Val::Percent(3.0),
                width: Val::Percent(32.0),
                max_width: Val::Px(40.0),
                aspect_ratio: Some(1.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(hud::rgb(class.color)),
        ));
    }
}

fn hero_column(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
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
    let look = selection
        .look
        .and_then(|i| selection.looks.get(i))
        .map(|&(_, race, sex)| (race, sex));
    body.spawn(Node {
        width: Val::Percent(100.0),
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|row| {
        if hero.is_some() {
            row.spawn((
                Node {
                    width: Val::Percent(30.0),
                    aspect_ratio: Some(0.75),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.01, 0.03, 0.85)),
            ))
            .with_children(|frame| {
                frame.spawn((
                    crate::portrait::survivors_pane::ModelPane,
                    ImageNode::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
                if let (Some(class), Some((race, _))) = (class, look) {
                    portrait_badges(frame, art, font, class, race);
                }
            });
        }
        row.spawn(Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|details| {
            details
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(4.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    if let Some(draft) = armory
                        .rename
                        .as_ref()
                        .filter(|draft| Some(draft.hero) == selection.character)
                    {
                        super::chip(
                            row,
                            font,
                            &super::field_label(&draft.first, armory.focus == 3),
                            Act::Focus(3),
                            armory.focus == 3,
                        );
                        super::chip(
                            row,
                            font,
                            &super::field_label(&draft.last, armory.focus == 4),
                            Act::Focus(4),
                            armory.focus == 4,
                        );
                        super::chip(row, font, "Save", Act::SaveName, false);
                        super::chip(row, font, "Cancel", Act::CancelName, false);
                        return;
                    }
                    if hero.is_some() {
                        super::controls::content_button(
                            row,
                            Act::EditName,
                            false,
                            Node {
                                flex_basis: Val::Percent(65.0),
                                flex_grow: 1.0,
                                min_width: Val::Px(0.0),
                                min_height: Val::Px(36.0),
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                ..default()
                            },
                            |field| {
                                field.spawn((
                                    hud::text_bundle(font, &name, 18.0, name_color),
                                    Node {
                                        max_width: Val::Percent(100.0),
                                        ..default()
                                    },
                                ));
                            },
                        );
                    } else {
                        line(row, font, &name, 18.0, name_color);
                    }
                    tab_button(row, font, "Random", Act::RandomName, false);
                });
            let look_line = match look {
                Some((race, sex)) => {
                    format!(
                        "{} {} · {}",
                        race_name(race),
                        class.map_or("", |c| c.name),
                        sex_name(sex)
                    )
                }
                None => "No look chosen".to_string(),
            };
            details
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(4.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    line(row, font, &look_line, 13.0, Color::WHITE);
                    tab_button(row, font, "Look", Act::Page(Page::Heroes), false);
                });
        });
    });
    heading(body, font, "Equipment");
    let owned = gear_lines(body, font, account, class_i);
    let total = campaign::PIECES;
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
    let health = account
        .milestones_picked(class_i, selection.character)
        .health_multiplier();
    if health > 1.0 {
        line(
            body,
            font,
            &format!(
                "Talents: +{:.0}% total maximum health",
                (health - 1.0) * 100.0
            ),
            12.0,
            BONUS,
        );
    }
    let spent = hero.map(|c| c.active_points()).unwrap_or(0);
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
            font,
            Act::Page(Page::Heroes),
            "Armory",
            "Heroes, look and gear",
        );
        door_button(
            row,
            font,
            Act::Page(Page::Talents),
            "Talents",
            &format!("{spent} of {}", campaign::TALENT_CAP),
        );
    });
    if let Some((race, sex)) = look {
        race_summary(body, art, font, race, sex);
    }
}

/// The starter reference pairs a body-matched race portrait with a compact bonus list.
fn race_summary(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    race: u8,
    sex: u8,
) {
    body.spawn(Node {
        width: Val::Percent(100.0),
        column_gap: Val::Px(8.0),
        margin: UiRect::top(Val::Px(6.0)),
        align_items: AlignItems::Start,
        ..default()
    })
    .with_children(|row| {
        if let Some(((image, size), tc)) = art.races.as_ref().zip(race_tc(race, sex)) {
            row.spawn((
                ImageNode {
                    image: image.clone(),
                    rect: Some(tc_rect(*size, tc)),
                    ..default()
                },
                Node {
                    width: Val::Percent(14.0),
                    max_width: Val::Px(40.0),
                    aspect_ratio: Some(1.0),
                    flex_shrink: 0.0,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BorderColor::all(FRAME),
            ));
        }
        row.spawn(Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|copy| {
            line(copy, font, race_name(race), 18.0, GOLD);
            if let Some(racials) = survivors_core::racials::description(race) {
                for racial in racials {
                    line(copy, font, &format!("• {racial}"), 12.0, BONUS);
                }
            } else {
                line(copy, font, "Racial bonuses are not yet defined.", 12.0, DIM);
            }
        });
    });
}

fn door_button(
    row: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    act: Act,
    caption: &str,
    under: &str,
) {
    super::controls::action_card(
        row,
        font,
        caption,
        under,
        act,
        false,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            min_height: Val::Px(48.0),
            ..default()
        },
    );
}

fn shown_zone<'a>(armory: &Armory, selection: &Selection) -> Option<(usize, &'a campaign::Zone)> {
    let index = armory.picked.or_else(|| {
        let name = selection.map_def().name;
        ZONES.iter().position(|z| z.map == name)
    })?;
    ZONES.get(index).map(|zone| (index, zone))
}

fn zone_column(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
) {
    heading(body, font, "Zone");
    let (level, clears) = super::hero_gate(account, selection);
    let shown = shown_zone(armory, selection).map(|(index, _)| index);
    for (index, zone) in ZONES.iter().enumerate() {
        let on = shown == Some(index);
        let reason = campaign::zone_lock_reason(index, level, &clears);
        body.spawn((
            Act::Zone(index),
            Button,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(6.0), Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(if on {
                GOLD
            } else {
                Color::srgba(0.5, 0.4, 0.15, 0.35)
            }),
            BackgroundColor(if on {
                Color::srgba(0.85, 0.65, 0.2, 0.28)
            } else {
                Color::NONE
            }),
        ))
        .with_children(|row| {
            if let Some(map) = data::MAPS.iter().find(|m| m.name == zone.map && m.map > 1) {
                row.spawn((
                    super::super::zone_art::ZoneArt(map.map),
                    ImageNode {
                        color: Color::NONE,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                ));
            }
            let color = if reason.is_some() { DIM } else { GOLD };
            row.spawn(Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|header| {
                display_line(
                    header,
                    font,
                    zone.name,
                    if on { 18.0 } else { 16.0 },
                    if on { GOLD } else { color },
                );
                zone_stage(header, font, index);
            });
            if let Some(reason) = &reason {
                line(
                    row,
                    font,
                    reason,
                    11.0,
                    if on {
                        Color::srgb(0.9, 0.45, 0.35)
                    } else {
                        DIM
                    },
                );
            }
            if on {
                line(row, font, zone.title, 13.0, Color::srgb(0.95, 0.55, 0.25));
                line(row, font, zone.briefing, 12.0, Color::WHITE);
                if zone.name != zone.map {
                    line(row, font, zone.map, 12.0, DIM);
                }
                if !zone.boss.is_empty() {
                    let elites = zone.elites.join(", ");
                    let stakes = if elites.is_empty() {
                        format!("{}.", zone.boss)
                    } else {
                        format!("{}. {elites}.", zone.boss)
                    };
                    line(row, font, &stakes, 11.0, DIM);
                }
            }
        });
    }
    if shown.is_none() {
        let map = selection.map_def();
        line(body, font, map.name, 16.0, GOLD);
        line(
            body,
            font,
            "Not on the campaign path. It stays open.",
            12.0,
            DIM,
        );
        line(
            body,
            font,
            "Survive the enemy waves and defeat the scheduled bosses.",
            12.0,
            Color::WHITE,
        );
    }
}

fn enter_bar(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
) {
    let (title, under) = enter_copy(armory, account, selection);
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(6.0),
        margin: UiRect::top(Val::Px(8.0)),
        ..default()
    })
    .with_children(|row| {
        super::controls::action_card(
            row,
            font,
            "Run Options",
            "Normal difficulty",
            Act::RunOptions,
            false,
            Node {
                min_height: Val::Px(48.0),
                max_width: Val::Percent(100.0),
                ..default()
            },
        );
        super::controls::action_card(
            row,
            font,
            &title,
            &under,
            Act::Enter,
            true,
            Node {
                width: Val::Percent(45.0),
                min_width: Val::Px(260.0),
                max_width: Val::Percent(100.0),
                min_height: Val::Px(60.0),
                ..default()
            },
        );
    });
}

/// Campaign order, not clear progress or an independently selectable difficulty.
fn zone_stage(row: &mut ChildSpawnerCommands, font: &Handle<Font>, index: usize) {
    row.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            flex_shrink: 0.0,
            row_gap: Val::Px(2.0),
            ..default()
        },
        bevy::ui::FocusPolicy::Pass,
    ))
    .with_children(|stage| {
        stage
            .spawn(Node {
                width: Val::Px(96.0),
                height: Val::Px(7.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|dots| {
                for position in 0..ZONES.len() {
                    dots.spawn((
                        Node {
                            width: Val::Px(6.0),
                            height: Val::Px(6.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(if position <= index {
                            Color::srgb(0.95, 0.36, 0.20)
                        } else {
                            Color::srgb(0.10, 0.08, 0.04)
                        }),
                        BorderColor::all(Color::BLACK),
                        bevy::ui::FocusPolicy::Pass,
                    ));
                }
            });
        line(
            stage,
            font,
            &format!("Stage {} of {}", index + 1, ZONES.len()),
            9.0,
            DIM,
        );
    });
}

fn enter_copy(armory: &Armory, account: &Account, selection: &Selection) -> (String, String) {
    let (level, clears) = super::hero_gate(account, selection);
    if let Some((index, zone)) = shown_zone(armory, selection) {
        let under = campaign::zone_lock_reason(index, level, &clears).unwrap_or_else(|| {
            let Some(map) = data::MAPS.iter().position(|map| map.name == zone.map) else {
                return String::new();
            };
            let finale = super::super::encounters::finale(map);
            let Some((_, at)) = finale.first() else {
                return String::new();
            };
            let names = finale
                .iter()
                .map(|(enemy, _)| enemy.name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("Final fight: {names} · {}:{:02}", at / 60, at % 60)
        });
        (format!("Enter {}", zone.name), under)
    } else {
        let map = selection.map_def();
        (
            format!("Enter {}", map.name),
            "Survive the enemy waves".into(),
        )
    }
}

fn gear_lines(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    class: usize,
) -> usize {
    let pieces = account
        .classes
        .get(class)
        .map(|p| p.pieces)
        .unwrap_or([0; campaign::PIECES]);
    let owned = campaign::owned_pieces(&pieces);
    body.spawn(Node {
        width: Val::Percent(100.0),
        display: Display::Grid,
        grid_template_columns: vec![RepeatedGridTrack::minmax(
            GridTrackRepetition::AutoFit,
            MinTrackSizingFunction::Px(36.0),
            MaxTrackSizingFunction::Fraction(1.0),
        )],
        column_gap: Val::Px(3.0),
        row_gap: Val::Px(5.0),
        ..default()
    })
    .with_children(|row| {
        for slot in 0..campaign::GRID_SLOTS {
            let progress = &account.classes[class];
            let band = progress.equipment.worn[slot].map(usize::from);
            let piece = progress
                .equipment
                .worn(class, &pieces, &account.supplemental, slot);
            row.spawn((
                Button,
                Act::OpenEquipment(slot),
                Node {
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    row_gap: Val::Px(3.0),
                    ..default()
                },
                BorderColor::all(band.map(band_color).unwrap_or(DIM)),
                BackgroundColor(Color::srgb(0.025, 0.02, 0.01)),
            ))
            .with_children(|cell| {
                if let Some(piece) = piece {
                    super::super::gear_icons::spawn_visual(cell, piece.visual, 30.0);
                } else {
                    cell.spawn(Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|empty| line(empty, font, "—", 12.0, DIM));
                }
                line(cell, font, campaign::SLOTS[slot], 9.0, DIM);
            });
        }
    });
    line(
        body,
        font,
        "Select a slot to inspect its gear in Collection.",
        11.0,
        DIM,
    );
    if campaign::two_handed(class) {
        line(
            body,
            font,
            "A two-handed weapon fills the main hand only.",
            11.0,
            DIM,
        );
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
    super::typography::section(body, font, text, 18.0, GOLD);
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

fn display_line(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: &str,
    size: f32,
    color: Color,
) {
    outlined_text(
        body,
        Node {
            max_width: Val::Percent(100.0),
            ..default()
        },
        super::typography::DisplayHeading,
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

pub(super) fn band_color(band: usize) -> Color {
    match band {
        0 => Color::WHITE,
        1 => hud::rgb(0x1eff00),
        2 => hud::rgb(0x0070dd),
        3 => hud::rgb(0xa335ee),
        4 => hud::rgb(0xc9a227),
        _ => hud::rgb(0xff8000),
    }
}

pub(super) fn bonus_line(bonus: campaign::Bonus) -> String {
    let mut parts = Vec::new();
    let pct = |v: f32| format!("{:+.1}%", v * 100.0);
    if bonus.health.abs() > 0.01 {
        parts.push(format!("{:+.0} health", bonus.health));
    }
    if bonus.pickup.abs() > 0.0001 {
        parts.push(format!("{} pickup range", pct(bonus.pickup)));
    }
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
        parts.push(format!("{} cooldown reduction", pct(bonus.cooldown)));
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
