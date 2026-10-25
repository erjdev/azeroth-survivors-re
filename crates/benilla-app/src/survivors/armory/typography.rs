//! Display headings use the installed Morpheus face; body copy keeps the HUD font.
use bevy::prelude::*;

pub(super) fn heading(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn((
        super::super::hud::text_bundle(font, value, size, color),
        DisplayHeading,
    ));
}

/// A centered panel label with flexible rules fading toward the panel edges.
pub(super) fn section(
    ui: &mut impl super::ChipParent,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn((
        Node {
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            flex_shrink: 0.0,
            ..default()
        },
        DisplayHeading,
    ))
    .with_children(|row| {
        rule(row, true);
        row.spawn((
            super::super::hud::text_bundle(font, value, size, color),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                max_width: Val::Percent(80.0),
                min_width: Val::Px(0.0),
                ..default()
            },
        ));
        rule(row, false);
    });
}

fn rule(ui: &mut ChildSpawnerCommands, left: bool) {
    let gold = Color::srgba(0.68, 0.52, 0.16, 0.72);
    let ends = if left {
        [gold.with_alpha(0.0), gold]
    } else {
        [gold, gold.with_alpha(0.0)]
    };
    ui.spawn((
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_right(vec![
            ColorStop::percent(ends[0], 0.0),
            ColorStop::percent(ends[1], 100.0),
        ])),
    ));
}

#[derive(Component)]
pub(in crate::survivors) struct DisplayHeading;

pub(in crate::survivors) fn apply(
    mut commands: Commands,
    headings: Query<Entity, With<DisplayHeading>>,
    children: Query<&Children>,
    mut text: Query<&mut TextFont>,
    assets: Res<AssetServer>,
    fonts: Res<Assets<Font>>,
    mut display: Local<Option<Handle<Font>>>,
) {
    if headings.is_empty() {
        return;
    }
    let font = display.get_or_insert_with(|| assets.load("mpq://Fonts/MORPHEUS.ttf"));
    // Keep the readable body face until the display font is available.
    if !fonts.contains(font.id()) {
        return;
    }
    for root in &headings {
        // Outlined glue text has separate glyph copies; style the entire heading subtree.
        for entity in std::iter::once(root).chain(children.iter_descendants(root)) {
            if let Ok(mut text) = text.get_mut(entity) {
                text.font = font.clone();
            }
        }
        commands.entity(root).remove::<DisplayHeading>();
    }
}
