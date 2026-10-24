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
