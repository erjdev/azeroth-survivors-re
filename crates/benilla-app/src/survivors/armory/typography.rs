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
