//! Survivors controls follow the intention art's gold outline and burgundy selection.
use bevy::prelude::*;

use super::{hud, Act, ChipParent};

#[derive(Component)]
pub(in crate::survivors) struct Control {
    selected: bool,
}

fn colors(selected: bool, interaction: Interaction) -> (Color, Color) {
    let fill = match interaction {
        Interaction::Pressed => Color::srgb(0.30, 0.12, 0.07),
        Interaction::Hovered => Color::srgb(0.19, 0.13, 0.05),
        Interaction::None if selected => Color::srgb(0.16, 0.035, 0.025),
        Interaction::None => Color::srgba(0.025, 0.02, 0.01, 0.95),
    };
    let border = if selected || interaction != Interaction::None {
        Color::srgb(0.91, 0.74, 0.28)
    } else {
        Color::srgb(0.40, 0.31, 0.10)
    };
    (fill, border)
}

pub(super) fn button(
    parent: &mut impl ChipParent,
    font: &Handle<Font>,
    label: &str,
    act: Act,
    selected: bool,
    node: Node,
) {
    content_button(parent, act, selected, node, |body| {
        body.spawn(hud::text_bundle(
            font,
            label,
            13.0,
            Color::srgb(0.91, 0.78, 0.40),
        ));
    });
}

/// Custom contents share one action surface and the standard hover/focus styling.
pub(super) fn content_button(
    parent: &mut impl ChipParent,
    act: Act,
    selected: bool,
    node: Node,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let (fill, border) = colors(selected, Interaction::None);
    parent
        .spawn((
            Button,
            act,
            Control { selected },
            Node {
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                ..node
            },
            BackgroundColor(fill),
            BorderColor::all(border),
        ))
        .with_children(content);
}

/// A single action surface contains its title and supporting copy, including wrapped zone names.
pub(super) fn action_card(
    parent: &mut impl ChipParent,
    font: &Handle<Font>,
    title: &str,
    subtitle: &str,
    act: Act,
    primary: bool,
    node: Node,
) {
    let (fill, border) = colors(true, Interaction::None);
    parent
        .spawn((
            Button,
            act,
            Control { selected: true },
            Node {
                border: UiRect::all(Val::Px(if primary { 2.0 } else { 1.0 })),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                row_gap: Val::Px(2.0),
                ..node
            },
            BackgroundColor(fill),
            BorderColor::all(border),
        ))
        .with_children(|body| {
            body.spawn((
                hud::text_bundle(font, title, if primary { 24.0 } else { 18.0 }, super::GOLD),
                super::typography::DisplayHeading,
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Percent(100.0),
                    ..default()
                },
            ));
            body.spawn((
                hud::text_bundle(font, subtitle, 11.0, super::DIM),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
}

/// Unavailable actions keep their place without receiving input.
pub(super) fn disabled(parent: &mut impl ChipParent, font: &Handle<Font>, label: &str, node: Node) {
    disabled_content(parent, node, |body| {
        body.spawn(hud::text_bundle(
            font,
            label,
            13.0,
            Color::srgb(0.31, 0.28, 0.19),
        ));
    });
}

/// Illustrated locked choices remain readable without becoming focus or click targets.
pub(super) fn disabled_content(
    parent: &mut impl ChipParent,
    node: Node,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                ..node
            },
            BackgroundColor(Color::srgb(0.025, 0.02, 0.01)),
            BorderColor::all(Color::srgb(0.16, 0.13, 0.06)),
        ))
        .with_children(content);
}

pub(in crate::survivors) fn style(
    mut controls: Query<
        (
            &Control,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (control, interaction, mut fill, mut border) in &mut controls {
        let (background, outline) = colors(control.selected, *interaction);
        fill.0 = background;
        *border = BorderColor::all(outline);
    }
}
