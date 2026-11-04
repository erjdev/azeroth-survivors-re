//! WMO minimap layers for the Survivors CPU compositor.
use super::minimap::Rgba;
use benilla_assets::{coords::bevy_to_wow, minimap_grid::group_axis_grid, WmoModel, WorldAssets};
use benilla_formats::MinimapTranslate;
use benilla_world::{
    wmo_portal::{down_ray_seeds, terrain_z_local, WmoPortalInstance, INTERIOR_PROBE_HEIGHT},
    world_point::WorldPoint,
};
use bevy::{math::Affine3A, prelude::*};
use std::collections::HashMap;

// Compile the engine's exact portal selection helper here without carrying upstream edits.
#[allow(clippy::duplicate_mod)] // Share the implementation while its upstream module stays private.
#[path = "../minimap/interior.rs"]
mod selection;

#[derive(PartialEq, Clone)]
pub(super) struct Key {
    stem: String,
    transform: Affine3A,
    group: usize,
    height: f32,
}

pub(super) fn select<'a>(
    pos: Vec3,
    instances: &Query<&WmoPortalInstance>,
    models: &'a Assets<WmoModel>,
    world: &WorldPoint,
    server: &AssetServer,
) -> Option<(Key, &'a WmoModel)> {
    world.area_interior()?;
    let eye = pos + Vec3::Y * INTERIOR_PROBE_HEIGHT;
    let terrain = world.terrain_height_under(eye);
    instances.iter().find_map(|inst| {
        let model = models.get(&inst.handle)?;
        if model.wmo_id == 0 {
            return None;
        }
        let inverse = inst.world_from_local.inverse();
        let local = bevy_to_wow(inverse.transform_point3(eye));
        let group = down_ray_seeds(
            model,
            local,
            terrain.map(|z| terrain_z_local(&inverse, eye, z)),
        )
        .in_group?;
        let path = server.get_path(inst.handle.id())?;
        let stem = selection::wmo_minimap_stem(&path.path().to_string_lossy())?;
        Some((
            Key {
                stem,
                transform: inst.world_from_local,
                group,
                height: pos.y,
            },
            model,
        ))
    })
}

struct Layer {
    hash: String,
    min: Vec2,
    max: Vec2,
}

pub(super) struct Layers {
    inverse: Affine3A,
    layers: Vec<Layer>,
}

impl Layers {
    pub(super) fn prepare(
        key: &Key,
        model: &WmoModel,
        pos: Vec3,
        radius: f32,
        translate: &MinimapTranslate,
        textures: &mut HashMap<String, Option<Rgba>>,
        assets: &mut WorldAssets,
    ) -> Self {
        let inverse = key.transform.inverse();
        let drawable =
            selection::interior_group_selection(model, &key.transform, pos, radius, key.group);
        let player_z = bevy_to_wow(inverse.transform_point3(pos))[2];
        let sort_key = |i: usize| {
            let g = &model.group_nav[i];
            if i == key.group {
                f32::MAX
            } else {
                (g.bbox_min[2] + g.bbox_max[2]) * 0.5 - player_z
            }
        };
        let mut order: Vec<_> = (0..model.group_nav.len())
            .filter(|&i| drawable[i])
            .collect();
        order.sort_by(|&a, &b| sort_key(a).total_cmp(&sort_key(b)));
        let mut layers = Vec::new();
        for i in order {
            let g = &model.group_nav[i];
            let (nx, dx) = group_axis_grid(g.bbox_max[0] - g.bbox_min[0]);
            let (ny, dy) = group_axis_grid(g.bbox_max[1] - g.bbox_min[1]);
            for x in 0..nx {
                for y in 0..ny {
                    let Some(hash) =
                        translate.get(&format!("{}_{i:03}_{x:02}_{y:02}.blp", key.stem))
                    else {
                        continue;
                    };
                    textures.entry(hash.into()).or_insert_with(|| {
                        Rgba::decode(assets, &format!("textures\\Minimap\\{hash}"))
                    });
                    layers.push(Layer {
                        hash: hash.into(),
                        min: Vec2::new(
                            g.bbox_min[0] + x as f32 * dx - f32::from(x == 0),
                            g.bbox_min[1] + y as f32 * dy - f32::from(y == 0),
                        ),
                        max: Vec2::new(
                            g.bbox_min[0] + (x + 1) as f32 * dx + f32::from(x + 1 == nx),
                            g.bbox_min[1] + (y + 1) as f32 * dy + f32::from(y + 1 == ny),
                        ),
                    });
                }
            }
        }
        Self { inverse, layers }
    }

    #[inline]
    pub(super) fn sample(&self, world: Vec3, textures: &HashMap<String, Option<Rgba>>) -> [f32; 3] {
        let p = bevy_to_wow(self.inverse.transform_point3(world));
        let p = Vec2::new(p[0], p[1]);
        let mut color = [0.; 3];
        for layer in &self.layers {
            let Some(uv) = tile_uv(p, layer.min, layer.max) else {
                continue;
            };
            let Some(Some(texture)) = textures.get(&layer.hash) else {
                continue;
            };
            // Half-texel inset and the engine's >=224/255 alpha test, before masking.
            let rgba = texture.sample(
                0.5 + uv.x * (texture.w - 1) as f32,
                0.5 + uv.y * (texture.h - 1) as f32,
            );
            if rgba[3] >= 224. {
                color.copy_from_slice(&rgba[..3]);
            }
        }
        color
    }
}

#[inline]
fn tile_uv(point: Vec2, min: Vec2, max: Vec2) -> Option<Vec2> {
    let f = (point - min) / (max - min);
    (f.cmpge(Vec2::ZERO).all() && f.cmple(Vec2::ONE).all()).then_some(Vec2::new(1. - f.y, 1. - f.x))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placed_layers_preserve_floor_order_and_reject_transparent_overlays() {
        let texture = |rgba: [u8; 4]| {
            Some(Rgba {
                w: 2,
                h: 2,
                px: rgba.repeat(4),
            })
        };
        let mut textures = HashMap::from([
            ("lower".into(), texture([20, 40, 60, 255])),
            ("upper".into(), texture([100, 120, 140, 223])),
        ]);
        let transform = Affine3A::from_rotation_translation(
            Quat::from_rotation_y(1.2),
            Vec3::new(100., 25., -90.),
        );
        let layers = Layers {
            inverse: transform.inverse(),
            layers: ["lower", "upper"]
                .map(|hash| Layer {
                    hash: hash.into(),
                    min: Vec2::ZERO,
                    max: Vec2::splat(10.),
                })
                .into(),
        };
        let world = transform.transform_point3(benilla_assets::coords::wow_to_bevy([5., 5., 0.]));
        assert_eq!(layers.sample(world, &textures), [20., 40., 60.]);
        textures.insert("upper".into(), texture([100, 120, 140, 224]));
        assert_eq!(layers.sample(world, &textures), [100., 120., 140.]);
        let outside =
            transform.transform_point3(benilla_assets::coords::wow_to_bevy([20., 5., 0.]));
        assert_eq!(layers.sample(outside, &textures), [0.; 3]);
    }

    #[test]
    fn local_north_is_up_and_west_is_left() {
        let min = Vec2::new(10., 20.);
        let max = Vec2::new(30., 60.);
        assert_eq!(tile_uv(Vec2::new(30., 60.), min, max), Some(Vec2::ZERO));
        assert_eq!(tile_uv(Vec2::new(10., 20.), min, max), Some(Vec2::ONE));
        assert_eq!(
            tile_uv(Vec2::new(20., 40.), min, max),
            Some(Vec2::splat(0.5))
        );
        assert_eq!(tile_uv(Vec2::new(9., 40.), min, max), None);
    }
}
