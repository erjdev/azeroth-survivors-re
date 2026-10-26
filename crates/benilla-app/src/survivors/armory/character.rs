//! Character loadout uses the same equipment and appearance actions as Collection.
use super::{collection, hud, Account, Armory, Renown, Selection, Stash, GOLD};
use crate::glue::art::{race_tc, tc_rect, GlueArt};
use crate::portrait::survivors_pane::{ModelPane, ModelSubject, ModelTurn};
use bevy::prelude::*;
use survivors_core::layout::Layout;

#[allow(clippy::too_many_arguments)]
pub(super) fn build(
    panel: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
    layout: &Layout,
    features: Option<&super::super::appearance::Options>,
    compact: bool,
) {
    if compact {
        panel
            .spawn((
                collection::CollectionScroll,
                collection::CompactScroll::Character,
                bevy::ui::RelativeCursorPosition::default(),
                ScrollPosition(armory.character_scroll),
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
                    hero_panel(
                        body, font, armory, account, selection, renown, stash, layout, compact,
                    )
                });
                collection::section(body, layout, |body| {
                    body_controls(body, art, font, account, selection, layout);
                    if let Some(features) = features {
                        feature_controls(body, font, features, layout);
                    }
                });
                collection::section(body, layout, |body| {
                    collection::equipment(body, font, armory, account, selection, layout)
                });
                collection::section(body, layout, |body| {
                    look_panel(body, font, armory, account, selection, layout)
                });
            });
        return;
    }
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            column_gap: Val::Px(layout.gap),
            ..default()
        })
        .with_children(|row| {
            collection::column(row, layout.left_width, layout, |body| {
                body_controls(body, art, font, account, selection, layout);
                collection::equipment(body, font, armory, account, selection, layout);
            });
            collection::column(
                row,
                100.0 - layout.left_width - layout.right_width,
                layout,
                |body| {
                    hero_panel(
                        body, font, armory, account, selection, renown, stash, layout, compact,
                    );
                },
            );
            collection::column(row, layout.right_width, layout, |body| {
                if let Some(features) = features {
                    feature_controls(body, font, features, layout);
                }
                body.spawn(Node {
                    flex_grow: 1.0,
                    min_height: Val::Px(layout.gap),
                    ..default()
                });
                look_panel(body, font, armory, account, selection, layout);
            });
        });
}

fn look_panel(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    collection::look_controls(body, font, armory, account, selection, layout);
    super::chip(
        body,
        font,
        "Inspect in Collection",
        super::Act::Page(super::Page::Collection),
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn hero_panel(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
    layout: &Layout,
    compact: bool,
) {
    let class = super::picked_class(account, selection);
    let hero = selection.character.and_then(|id| account.character(id));
    let name = hero
        .map(|h| format!("{} {}", h.first, h.last))
        .unwrap_or_else(|| "Choose a hero".into());
    body.spawn(hud::centered_bundle(font, name, layout.title_size, GOLD));
    // Describe the body actually selected for the model, not an unrelated cached look
    // while another hero's class catalog is still loading.
    let identity = hero
        .filter(|hero| selection.class == Some(hero.class))
        .and_then(|_| selection.look.and_then(|i| selection.looks.get(i)))
        .map(|&(_, race, sex)| {
            format!(
                "{} {} ",
                if sex == 0 { "Male" } else { "Female" },
                crate::char_select::race_name(race)
            )
        })
        .unwrap_or_default();
    body.spawn(hud::centered_bundle(
        font,
        identity,
        layout.body_size,
        Color::WHITE,
    ))
    .with_children(|line| {
        line.spawn((
            TextSpan::new(super::CLASSES[class].name),
            TextFont {
                font: font.clone(),
                font_size: layout.body_size,
                ..default()
            },
            TextColor(hud::rgb(super::CLASSES[class].color)),
        ));
    });
    let managing = armory.manage_heroes || hero.is_none();
    if hero.is_some() {
        body.spawn(Node {
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|row| {
            super::chip(
                row,
                font,
                if managing {
                    "Back to character"
                } else {
                    "Manage heroes"
                },
                super::Act::ManageHeroes(!managing),
                managing,
            );
        });
    }
    if managing {
        super::typography::section(body, font, "Heroes", layout.title_size, GOLD);
        super::heroes(body, font, armory, account, selection, renown, stash);
        return;
    }
    if compact {
        // Let the layout own the aspect ratio; the render texture's intrinsic size
        // must not inflate the scroll section when its image becomes available.
        body.spawn(Node {
            // Keep the full figure and rotation controls reachable on short windows.
            width: Val::Percent(60.0),
            max_width: Val::Vh(52.0),
            min_height: Val::Px(0.0),
            aspect_ratio: Some(1.0),
            align_self: AlignSelf::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|frame| {
            frame.spawn((
                ModelPane,
                ImageNode::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
    } else {
        body.spawn((
            ModelPane,
            ImageNode::default(),
            Node {
                width: Val::Auto,
                height: Val::Percent(78.0),
                max_width: Val::Percent(100.0),
                aspect_ratio: Some(1.0),
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                ..default()
            },
        ));
    }
    body.spawn(hud::centered_bundle(
        font,
        "Drag the hero to turn",
        layout.body_size,
        GOLD,
    ));
    body.spawn(Node {
        column_gap: Val::Px(layout.gap),
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|row| {
        for (label, angle) in [("Turn left", -0.4), ("Turn right", 0.4)] {
            row.spawn((
                Button,
                ModelTurn(angle),
                Node {
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.12, 0.06)),
            ))
            .with_children(|button| {
                button.spawn(hud::text_bundle(font, label, layout.body_size, GOLD));
            });
        }
    });
}

pub(in crate::survivors) fn subject(
    phase: Res<State<super::super::Phase>>,
    armory: Res<Armory>,
    heroes: Query<Entity, With<super::super::hero::Hero>>,
    mut subject: ResMut<ModelSubject>,
) {
    subject.0 = if *phase.get() == super::super::Phase::Menu
        && (armory.page == super::Page::Door
            || (armory.page == super::Page::Heroes && !armory.manage_heroes))
    {
        heroes.single().ok()
    } else {
        None
    };
}

/// Only the selected hero's class catalog may supply a replacement body.
pub(super) fn choose_body(
    selection: &mut Selection,
    account: &mut Account,
    race: u8,
    sex: u8,
) -> bool {
    let Some(hero) = selection.character.and_then(|id| account.character(id)) else {
        return false;
    };
    if selection.class != Some(hero.class) {
        return false;
    }
    let id = hero.id;
    let Some(index) = selection
        .looks
        .iter()
        .position(|&(_, r, s)| (r, s) == (race, sex))
    else {
        return false;
    };
    if !account.set_body(id, (race, sex)) && selection.look == Some(index) {
        return false;
    }
    selection.look = Some(index);
    selection.version += 1;
    true
}

pub(super) fn body_controls(
    body: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    if !selection
        .character
        .and_then(|id| account.character(id))
        .is_some_and(|hero| selection.class == Some(hero.class))
    {
        return;
    }
    let current = selection.look.and_then(|i| selection.looks.get(i)).copied();
    let Some((_, race, sex)) = current else {
        return;
    };
    super::typography::section(body, font, "Appearance", layout.title_size, GOLD);
    body.spawn(Node {
        width: Val::Percent(100.0),
        column_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|row| {
        for (label, races, color) in [
            ("Alliance", [1, 3, 4, 7], Color::srgb(0.35, 0.60, 0.95)),
            ("Horde", [2, 5, 6, 8], Color::srgb(0.95, 0.30, 0.22)),
        ] {
            row.spawn(Node {
                flex_basis: Val::Px(0.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(layout.gap * 0.5),
                ..default()
            })
            .with_children(|faction| {
                faction.spawn(hud::text_bundle(font, label, layout.body_size, color));
                faction
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(layout.gap * 0.25),
                        ..default()
                    })
                    .with_children(|icons| {
                        for r in races {
                            let available = selection
                                .looks
                                .iter()
                                .any(|&(_, candidate, s)| candidate == r && s == sex);
                            race_portrait(icons, art, font, r, sex, r == race, available, layout);
                        }
                    });
            });
        }
    });
    body.spawn(hud::text_bundle(
        font,
        crate::char_select::race_name(race),
        layout.body_size,
        GOLD,
    ));
    body.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|row| {
        for (s, name) in [(0, "Male"), (1, "Female")] {
            if selection
                .looks
                .iter()
                .any(|&(_, r, candidate)| r == race && candidate == s)
            {
                super::chip(row, font, name, super::Act::Body(race, s), s == sex);
            }
        }
    });
}

/// Runtime atlas art stays in the player's data; unavailable class races remain visible but inert.
#[allow(clippy::too_many_arguments)]
fn race_portrait(
    row: &mut ChildSpawnerCommands,
    art: &GlueArt,
    font: &Handle<Font>,
    race: u8,
    sex: u8,
    selected: bool,
    available: bool,
    layout: &Layout,
) {
    let mut button = row.spawn((
        Node {
            flex_basis: Val::Px(0.0),
            flex_grow: 1.0,
            min_width: Val::Px(0.0),
            aspect_ratio: Some(1.0),
            border: UiRect::all(Val::Px(if selected { 2.0 } else { 1.0 })),
            ..default()
        },
        BorderColor::all(if selected {
            GOLD
        } else {
            Color::srgb(0.22, 0.19, 0.12)
        }),
        BackgroundColor(Color::srgb(0.06, 0.05, 0.03)),
        Name::new(crate::char_select::race_name(race)),
    ));
    if available {
        button.insert((Button, super::Act::Body(race, sex)));
    }
    button.with_children(|button| {
        if let Some(((image, size), tc)) = art.races.as_ref().zip(race_tc(race, sex)) {
            button.spawn((
                ImageNode {
                    image: image.clone(),
                    rect: Some(tc_rect(*size, tc)),
                    color: if available {
                        Color::WHITE
                    } else {
                        Color::srgba(0.35, 0.35, 0.35, 0.55)
                    },
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        } else {
            // Keep the selection usable when the optional atlas is missing.
            button.spawn(hud::text_bundle(
                font,
                crate::char_select::race_name(race),
                layout.body_size * 0.7,
                if available {
                    GOLD
                } else {
                    Color::srgb(0.3, 0.3, 0.3)
                },
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_choices_preserve_class_and_other_heroes_and_reject_missing_bodies() {
        let mut account = crate::survivors::account::empty();
        let a = account.create("First", "Hero", 0).unwrap();
        let b = account.create("Second", "Hero", 0).unwrap();
        account.set_body(a, (1, 0));
        account.set_body(b, (3, 0));
        let mut selection = Selection {
            class: Some(0),
            character: Some(a),
            looks: vec![(100, 1, 0), (101, 1, 1)],
            look: Some(0),
            ..default()
        };
        assert!(!choose_body(&mut selection, &mut account, 3, 1));
        assert!(choose_body(&mut selection, &mut account, 1, 1));
        assert_eq!(selection.look, Some(1));
        assert_eq!(selection.class, Some(0));
        assert_eq!(account.character(a).unwrap().body, Some((1, 1)));
        assert_eq!(account.character(b).unwrap().body, Some((3, 0)));
        assert!(!choose_body(&mut selection, &mut account, 1, 1));
        selection.class = Some(1);
        assert!(!choose_body(&mut selection, &mut account, 1, 0));
    }
}

#[derive(Component)]
pub(in crate::survivors) struct FeatureValue(pub usize);

#[derive(Component)]
pub(in crate::survivors) struct FeatureDot(pub usize, pub u8);

pub(super) fn feature_dot_color(selected: bool) -> Color {
    if selected {
        GOLD
    } else {
        Color::srgb(0.24, 0.20, 0.12)
    }
}

fn feature_controls(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    features: &super::super::appearance::Options,
    layout: &Layout,
) {
    super::typography::section(body, font, "Features", layout.title_size, GOLD);
    for i in 0..5 {
        if features.counts[i] < 2 {
            continue;
        }
        body.spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(layout.gap)),
                border: UiRect::all(Val::Px(1.0)),
                column_gap: Val::Px(layout.gap),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor::all(Color::srgb(0.24, 0.20, 0.12)),
            BackgroundColor(Color::srgb(0.035, 0.03, 0.02)),
        ))
        .with_children(|row| {
            super::chip(row, font, "<", super::Act::Feature(i, -1), false);
            row.spawn(Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(layout.gap * 0.4),
                ..default()
            })
            .with_children(|value| {
                value.spawn(hud::centered_bundle(
                    font,
                    features.labels[i],
                    layout.body_size * 0.85,
                    Color::srgb(0.65, 0.61, 0.45),
                ));
                value.spawn((
                    FeatureValue(i),
                    hud::centered_bundle(
                        font,
                        format!("{} / {}", features.values[i] + 1, features.counts[i]),
                        layout.body_size,
                        GOLD,
                    ),
                ));
                value
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(layout.body_size * 0.12),
                        ..default()
                    })
                    .with_children(|dots| {
                        for position in 0..features.counts[i] {
                            dots.spawn((
                                FeatureDot(i, position),
                                Node {
                                    flex_basis: Val::Px(0.0),
                                    flex_grow: 1.0,
                                    min_width: Val::Px(0.0),
                                    max_width: Val::Px(layout.body_size * 0.32),
                                    height: Val::Px(layout.body_size * 0.32),
                                    aspect_ratio: Some(1.0),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(feature_dot_color(position == features.values[i])),
                            ));
                        }
                    });
            });
            super::chip(row, font, ">", super::Act::Feature(i, 1), false);
        });
    }
    super::chip(
        body,
        font,
        "Randomize features",
        super::Act::RandomFeatures,
        false,
    );
}
