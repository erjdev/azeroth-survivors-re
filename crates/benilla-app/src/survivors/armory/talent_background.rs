//! Join the stock tiles before scaling, avoiding independent UI rounding at their seams.
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;

#[derive(Component)]
pub(in crate::survivors) struct TreeBackdrop(pub String);

const WIDTH: usize = 300;
const HEIGHT: usize = 331;
const TILES: [(&str, usize, usize, usize, usize); 4] = [
    ("TopLeft", 0, 0, 256, 256),
    ("TopRight", 256, 0, 44, 256),
    ("BottomLeft", 0, 256, 256, 75),
    ("BottomRight", 256, 256, 44, 75),
];

type Decoded = (u32, u32, Vec<u8>);

fn compose(mut decode: impl FnMut(&str) -> Option<Decoded>) -> Option<Vec<u8>> {
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    for (name, x, y, width, height) in TILES {
        let (sw, sh, source) = decode(name)?;
        let (sw, sh) = (sw as usize, sh as usize);
        if sw < width || sh < height || sw.checked_mul(sh)?.checked_mul(4)? != source.len() {
            return None;
        }
        for row in 0..height {
            let from = row * sw * 4;
            let to = ((y + row) * WIDTH + x) * 4;
            pixels[to..to + width * 4].copy_from_slice(&source[from..from + width * 4]);
        }
    }
    Some(pixels)
}

pub(in crate::survivors) fn fill(
    mut commands: Commands,
    mut slots: Query<(Entity, &TreeBackdrop, &mut ImageNode)>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<Handle<Image>>>>,
) {
    let Some(assets) = assets.as_deref_mut() else {
        return;
    };
    for (entity, backdrop, mut node) in &mut slots {
        let handle = cache.entry(backdrop.0.clone()).or_insert_with(|| {
            let pixels = compose(|tile| {
                assets.decode_rgba(&format!(
                    "Interface\\TalentFrame\\{}-{tile}.blp",
                    backdrop.0
                ))
            })?;
            let mut image = Image::new(
                Extent3d {
                    width: WIDTH as u32,
                    height: HEIGHT as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            image.sampler = ImageSampler::linear();
            Some(images.add(image))
        });
        if let Some(handle) = handle {
            node.image = handle.clone();
        }
        commands.entity(entity).remove::<TreeBackdrop>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_every_painted_pixel_and_discards_stock_padding() {
        let pixels = compose(|name| {
            let index = TILES.iter().position(|tile| tile.0 == name).unwrap();
            let (_, _, _, w, h) = TILES[index];
            let (sw, sh) = (
                if index % 2 == 0 { 256 } else { 64 },
                if index < 2 { 256 } else { 128 },
            );
            let mut source = vec![255; sw * sh * 4];
            for y in 0..h {
                for x in 0..w {
                    source[(y * sw + x) * 4..(y * sw + x + 1) * 4].copy_from_slice(&[
                        index as u8,
                        x as u8,
                        y as u8,
                        255,
                    ]);
                }
            }
            Some((sw as u32, sh as u32, source))
        })
        .unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let index = usize::from(x >= 256) + 2 * usize::from(y >= 256);
                assert_eq!(
                    &pixels[(y * WIDTH + x) * 4..(y * WIDTH + x + 1) * 4],
                    &[index as u8, (x % 256) as u8, (y % 256) as u8, 255]
                );
            }
        }
        assert!(compose(|_| None).is_none());
        assert!(compose(|_| Some((1, 1, vec![0; 4]))).is_none());
        assert!(compose(|_| Some((256, 256, vec![0; 4]))).is_none());
    }
}
