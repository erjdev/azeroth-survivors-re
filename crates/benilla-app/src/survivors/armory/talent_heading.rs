//! Heading artwork from the installed specialization catalog.
use benilla_assets::LockRecover;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub(in crate::survivors) struct Icon(pub u32);

pub(in crate::survivors) fn fill(
    mut commands: Commands,
    mut nodes: Query<(Entity, &Icon, &mut ImageNode)>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut paths: Local<Option<HashMap<u32, String>>>,
) {
    if nodes.is_empty() {
        return;
    }
    let Some(assets) = assets.as_deref_mut() else {
        return;
    };
    if paths.is_none() {
        let loaded = benilla_formats::load_survivors_talent_icons(&mut assets.chain.lock_recover());
        *paths = Some(loaded.unwrap_or_else(|error| {
            warn!("Talent heading artwork unavailable: {error:#}");
            HashMap::new()
        }));
    }
    for (entity, icon, mut image) in &mut nodes {
        let handle = paths
            .as_ref()
            .and_then(|paths| paths.get(&icon.0))
            .and_then(|path| assets.sprite_texture(path, &mut images));
        if let Some(handle) = handle {
            image.image = handle;
            commands.entity(entity).remove::<Icon>();
        } else {
            commands.entity(entity).despawn();
        }
    }
}
