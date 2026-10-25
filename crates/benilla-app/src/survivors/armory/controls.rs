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
    let (fill, border) = colors(selected, Interaction::None);
    parent
        .spawn((
            Button,
            act,
            Control { selected },
            Node {
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..node
            },
            BackgroundColor(fill),
            BorderColor::all(border),
        ))
        .with_children(|body| {
            body.spawn(hud::text_bundle(
                font,
                label,
                13.0,
                Color::srgb(0.91, 0.78, 0.40),
            ));
        });
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
