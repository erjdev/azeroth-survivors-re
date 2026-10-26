//! Collection icons use the same verified display identity as equipment rendering.
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub(super) struct GearIcon(pub u32, pub Color);
#[derive(Component)]
pub(super) struct Filled;

pub(super) fn spawn(
    ui: &mut ChildSpawnerCommands,
    class: usize,
    slot: usize,
    band: usize,
    size: f32,
) {
    let Some(visual) = survivors_core::gear_visuals::piece(class, slot, band) else {
        return;
    };
    spawn_visual(ui, visual, size);
}

pub(super) fn spawn_visual(
    ui: &mut ChildSpawnerCommands,
    visual: survivors_core::gear_visuals::GearVisual,
    size: f32,
) {
    ui.spawn((
        GearIcon(visual.display_id, Color::WHITE),
        ImageNode {
            color: Color::NONE,
            ..default()
        },
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

/// Collection cells retain dim artwork for unowned pieces; absent content has no invented icon.
pub(super) fn spawn_cell(
    ui: &mut ChildSpawnerCommands,
    class: usize,
    slot: usize,
    band: usize,
    size: f32,
    owned: bool,
) -> bool {
    let Some(visual) = survivors_core::gear_visuals::piece(class, slot, band) else {
        return false;
    };
    spawn_visual_cell(ui, visual, size, owned);
    true
}

pub(super) fn spawn_visual_cell(
    ui: &mut ChildSpawnerCommands,
    visual: survivors_core::gear_visuals::GearVisual,
    size: f32,
    owned: bool,
) {
    ui.spawn((
        GearIcon(
            visual.display_id,
            if owned {
                Color::WHITE
            } else {
                Color::srgba(0.3, 0.3, 0.3, 0.65)
            },
        ),
        ImageNode {
            color: Color::NONE,
            ..default()
        },
        Node {
            width: Val::Px(size),
            max_width: Val::Percent(90.0),
            aspect_ratio: Some(1.0),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

pub(super) fn fill(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GearIcon, &mut ImageNode), Without<Filled>>,
    displays: Option<Res<crate::entities::ItemDisplays>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<u32, Handle<Image>>>,
) {
    let (Some(displays), Some(assets)) = (displays.as_deref(), assets.as_deref_mut()) else {
        return;
    };
    for (entity, icon, mut image) in &mut nodes {
        let handle = cache.get(&icon.0).cloned().or_else(|| {
            let path = displays.catalog.get(icon.0)?.icon.as_ref()?;
            let handle = assets.sprite_texture(&format!("{path}.blp"), &mut images)?;
            cache.insert(icon.0, handle.clone());
            Some(handle)
        });
        if let Some(handle) = handle {
            image.image = handle;
            image.color = icon.1;
        }
        commands.entity(entity).insert(Filled);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mapped_gear_has_display_rows_and_readable_icons_in_vanilla_data() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_item_display_catalog(&mut chain).unwrap();
        let mut seen = std::collections::HashSet::new();
        let auxiliary = survivors_core::campaign::GEAR_NAMES
            .iter()
            .flatten()
            .filter_map(|name| survivors_core::gear_visuals::named(name));
        let pieces = (0..9).flat_map(|class| {
            (0..6).flat_map(move |band| {
                (0..9)
                    .filter_map(move |slot| survivors_core::gear_visuals::piece(class, slot, band))
            })
        });
        let supplemental = (0..9).flat_map(|class| {
            (0..9).map(move |slot| {
                survivors_core::supplemental_gear::piece(
                    class,
                    slot,
                    survivors_core::gear_tier::GearTier::Poor,
                )
                .expect("complete Poor catalog")
                .visual
            })
        });
        for visual in auxiliary.chain(pieces).chain(supplemental) {
            if !seen.insert(visual.display_id) {
                continue;
            }
            let display = catalog.get(visual.display_id).expect("mapped gear display");
            let icon = display.icon.as_ref().expect("mapped gear icon");
            assert!(!chain
                .read(&format!("{icon}.blp"))
                .expect("mapped gear texture")
                .is_empty());
        }
        for display in [35631, 35632, 35633, 35634] {
            assert!(
                seen.contains(&display),
                "class-specific Atiesh must be checked"
            );
        }
        assert!(seen.len() >= 84, "catalog coverage unexpectedly shrank");
    }
}
