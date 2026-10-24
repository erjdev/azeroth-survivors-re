//! Character loadout uses the same equipment and appearance actions as Collection.
use super::{collection, hud, Account, Armory, Renown, Selection, Stash, GOLD};
use bevy::prelude::*;
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
                    body.spawn(hud::text_bundle(font, "Heroes", layout.title_size, GOLD));
                    super::heroes(body, font, armory, account, selection, renown, stash);
                },
            );
            collection::column(row, layout.right_width, layout, |body| {
                collection::detail(body, font, armory, account, selection, layout);
            });
        });
}
