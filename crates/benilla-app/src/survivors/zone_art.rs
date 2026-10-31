//! Starter row art resolved from the installed map/loading-screen catalogs.
use benilla_assets::LockRecover;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub(super) struct ZoneArt(pub u32);
#[derive(Component)]
pub(super) struct Filled;

pub(super) fn fill(
    mut commands: Commands,
    mut nodes: Query<(Entity, &ZoneArt, &mut ImageNode), Without<Filled>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut paths: Local<Option<HashMap<u32, String>>>,
    mut cache: Local<HashMap<u32, Handle<Image>>>,
) {
    if nodes.is_empty() {
        return;
    }
    let Some(assets) = assets.as_deref_mut() else {
        return;
    };
    if paths.is_none() {
        let mut chain = assets.chain.lock_recover();
        let (Ok(maps), Ok(screens)) = (
            benilla_formats::load_map_catalog(&mut chain),
            benilla_formats::load_loading_screens(&mut chain),
        ) else {
            warn!("Starter zone artwork unavailable: map/loading-screen catalogs failed");
            *paths = Some(HashMap::new());
            return;
        };
        *paths = Some(
            super::data::MAPS
                .iter()
                .filter_map(|map| {
                    // Continent art cannot stand in for a distinct outdoor zone illustration.
                    if map.map <= 1 {
                        return None;
                    }
                    let path = screens.path(maps.loading_screen_id(map.map)?)?;
                    Some((map.map, path.to_owned()))
                })
                .collect(),
        );
    }
    for (entity, art, mut image) in &mut nodes {
        let handle = cache.get(&art.0).cloned().or_else(|| {
            let path = paths.as_ref()?.get(&art.0)?;
            let handle = assets.sprite_texture(path, &mut images)?;
            cache.insert(art.0, handle.clone());
            Some(handle)
        });
        if let Some(handle) = handle {
            image.image = handle;
            image.color = Color::srgba(0.6, 0.6, 0.6, 0.65);
        }
        commands.entity(entity).insert(Filled);
    }
}

/// Fill each banner without stretching the source artwork as its row reflows.
pub(super) fn crop(
    images: Res<Assets<Image>>,
    mut nodes: Query<(&ComputedNode, &mut ImageNode), With<ZoneArt>>,
) {
    for (node, mut image) in &mut nodes {
        let Some(source) = images.get(&image.image) else {
            continue;
        };
        if node.size.x <= 0.0 || node.size.y <= 0.0 {
            continue;
        }
        let size = source.size().as_vec2();
        let aspect = node.size.x / node.size.y;
        let crop = Vec2::new(size.x.min(size.y * aspect), size.y.min(size.x / aspect));
        let min = (size - crop) * 0.5;
        let rect = Some(Rect::from_corners(min, min + crop));
        if image.rect != rect {
            image.rect = rect;
        }
    }
}
