//! Character loadout uses the same equipment and appearance actions as Collection.
use super::{collection, hud, Account, Armory, Renown, Selection, Stash, GOLD};
use crate::portrait::survivors_pane::{ModelPane, ModelSubject, ModelTurn};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use survivors_core::layout::Layout;

#[allow(clippy::too_many_arguments)]
pub(super) fn build(
    panel: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
    layout: &Layout,
) {
    let class = super::picked_class(account, selection);
    let hero = selection.character.and_then(|id| account.character(id));
    let name = hero
        .map(|h| format!("{} {}", h.first, h.last))
        .unwrap_or_else(|| "Choose a hero".into());
    panel.spawn(hud::text_bundle(
        font,
        format!("{name} · {}", super::CLASSES[class].name),
        layout.title_size,
        GOLD,
    ));
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
                body_controls(body, font, account, selection, layout);
                collection::equipment(body, font, account, selection, layout);
            });
            collection::column(
                row,
                100.0 - layout.left_width - layout.right_width,
                layout,
                |body| {
                    body.spawn((
                        ModelPane,
                        RelativeCursorPosition::default(),
                        ImageNode::default(),
                        Node {
                            width: Val::Auto,
                            height: Val::Percent(60.0),
                            max_width: Val::Percent(100.0),
                            aspect_ratio: Some(1.0),
                            align_self: AlignSelf::Center,
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                    body.spawn(hud::text_bundle(
                        font,
                        "Drag the hero to turn",
                        layout.body_size,
                        GOLD,
                    ));
                    body.spawn(Node {
                        column_gap: Val::Px(layout.gap),
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
                    body.spawn(hud::text_bundle(font, "Heroes", layout.title_size, GOLD));
                    super::heroes(body, font, armory, account, selection, renown, stash);
                },
            );
            collection::column(row, layout.right_width, layout, |body| {
                collection::detail(body, font, armory, account, selection, layout);
            });
        });
}

pub(in crate::survivors) fn subject(
    phase: Res<State<super::super::Phase>>,
    armory: Res<Armory>,
    heroes: Query<Entity, With<super::super::hero::Hero>>,
    mut subject: ResMut<ModelSubject>,
) {
    subject.0 = if *phase.get() == super::super::Phase::Menu && armory.page == super::Page::Heroes {
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

fn body_controls(
    body: &mut ChildSpawnerCommands,
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
    body.spawn(hud::text_bundle(
        font,
        "Appearance",
        layout.title_size,
        GOLD,
    ));
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(layout.gap),
        row_gap: Val::Px(layout.gap),
        ..default()
    })
    .with_children(|row| {
        let mut races = Vec::new();
        for &(_, r, s) in &selection.looks {
            if s == sex && !races.contains(&r) {
                races.push(r);
                super::chip(
                    row,
                    font,
                    crate::char_select::race_name(r),
                    super::Act::Body(r, sex),
                    r == race,
                );
            }
        }
    });
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
        let mut selection = Selection::default();
        selection.class = Some(0);
        selection.character = Some(a);
        selection.looks = vec![(100, 1, 0), (101, 1, 1)];
        selection.look = Some(0);
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
