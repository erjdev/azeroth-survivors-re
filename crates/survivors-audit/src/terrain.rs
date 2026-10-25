//! Terrain-only arena sampling. Does not include solid models or WMO geometry.
use anyhow::{ensure, Context, Result};
use benilla_formats::{Chain, MapTiles, TileMesh};
use std::collections::HashMap;

fn disk(radius: f32) -> Vec<[f32; 2]> {
    let steps = (radius / 2.0).ceil() as i32;
    (-steps..=steps)
        .flat_map(|x| {
            (-steps..=steps).filter_map(move |y| {
                let p = [x as f32 * 2.0, y as f32 * 2.0];
                (p[0] * p[0] + p[1] * p[1] <= radius * radius).then_some(p)
            })
        })
        .collect()
}

fn liquid_above(tile: &TileMesh, point: [f32; 3]) -> bool {
    tile.chunks.iter().flat_map(|c| &c.liquids).any(|liquid| {
        liquid.indices.as_chunks::<3>().0.iter().any(|t| {
            let tri = [
                liquid.positions[t[0] as usize],
                liquid.positions[t[1] as usize],
                liquid.positions[t[2] as usize],
            ];
            benilla_formats::triangle_z_at(&tri, point[0], point[1])
                .is_some_and(|z| z > point[2] + 0.1)
        })
    })
}

// Diagnostic only: compare the public height lookup with the triangles actually emitted by
// the terrain loader. Missing samples remain missing in the survey's acceptance counts.
fn mesh_height(tile: &TileMesh, point: [f32; 3]) -> Option<f32> {
    tile.chunks.iter().find_map(|chunk| {
        chunk.indices.as_chunks::<3>().0.iter().find_map(|t| {
            let triangle = [
                chunk.positions[t[0] as usize],
                chunk.positions[t[1] as usize],
                chunk.positions[t[2] as usize],
            ];
            benilla_formats::triangle_z_at(&triangle, point[0], point[1])
        })
    })
}

pub fn survey(chain: &mut Chain, map: u32, input: [f32; 3]) -> Result<()> {
    let [x, y, radius] = input;
    ensure!(
        radius > 0.0 && radius <= 200.0,
        "terrain radius must be greater than zero and at most 200 yards"
    );
    let maps = benilla_formats::load_map_catalog(chain)?;
    let dir = maps.directory(map).context("map has no directory")?;
    let tiles = MapTiles::load(chain, dir)?;
    let mut cache = HashMap::new();
    let points = disk(radius);
    // Load every required tile, including one-yard gradient neighbours. Missing files fail
    // the survey rather than disappearing from the sample count.
    for p in &points {
        for offset in [[0., 0.], [1., 0.], [-1., 0.], [0., 1.], [0., -1.]] {
            let key = tiles.tile_at(x + p[0] + offset[0], y + p[1] + offset[1]);
            if let std::collections::hash_map::Entry::Vacant(entry) = cache.entry(key) {
                entry.insert(benilla_formats::load_tile_mesh(chain, dir, key.0, key.1)?);
            }
        }
    }
    let height = |x, y| {
        let tile = &cache[&tiles.tile_at(x, y)];
        benilla_formats::terrain_height_at(&tile.chunks, [x, y, 0.])
    };
    let (mut supported, mut wet, mut impassable, mut steep, mut edge) = (0, 0, 0, 0, 0);
    let mut mesh_supported_misses = 0;
    let mut missing_examples = Vec::new();
    let (mut low, mut high) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in &points {
        let (px, py) = (x + p[0], y + p[1]);
        let Some(z) = height(px, py) else {
            let tile = &cache[&tiles.tile_at(px, py)];
            let mesh = mesh_height(tile, [px, py, 0.]);
            mesh_supported_misses += usize::from(mesh.is_some());
            if missing_examples.len() < 8 {
                missing_examples.push((px, py, mesh));
            }
            continue;
        };
        supported += 1;
        low = low.min(z);
        high = high.max(z);
        let tile = &cache[&tiles.tile_at(px, py)];
        wet += usize::from(liquid_above(tile, [px, py, z]));
        impassable += usize::from(
            tile.chunks
                .iter()
                .any(|c| c.impassable && c.height_at([px, py, z]).is_some()),
        );
        match [
            height(px + 1., py),
            height(px - 1., py),
            height(px, py + 1.),
            height(px, py - 1.),
        ] {
            [Some(e), Some(w), Some(n), Some(s)] => {
                let gradient = ((e - w).powi(2) + (n - s).powi(2)).sqrt() * 0.5;
                steep += usize::from(gradient > 1.0);
            }
            _ => edge += 1,
        }
    }
    println!("TERRAIN DISK: map {map} ({dir}), center [{x:.3}, {y:.3}], radius {radius}, spacing 2 yards, {} tiles",cache.len());
    println!(
        "Samples: {} total, {supported} height-query supported, {} unresolved height-query samples",
        points.len(),
        points.len() - supported
    );
    if supported > 0 {
        println!("Terrain height range: {low:.3} to {high:.3}");
    }
    println!("Supported samples: {wet} liquid deeper than 0.1 yard, {impassable} impassable, {steep} estimated slope above 45 degrees, {edge} incomplete gradient neighbourhoods");
    println!("Missing height-query samples with emitted-mesh support: {mesh_supported_misses}");
    for (px, py, mesh) in missing_examples {
        println!("Missing sample [{px:.3}, {py:.3}]: emitted-mesh height {mesh:?}");
    }
    println!("Counts may overlap. Scope: sampled ADT terrain and MCLQ only. Excludes WMO floors/liquids, doodad collision, walls, connectivity, camera and between-sample hazards. Not arena readiness or navigation proof.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_includes_center_and_boundary_but_excludes_corners() {
        let points = disk(2.0);
        assert_eq!(points.len(), 5);
        assert!(points.contains(&[0., 0.]));
        assert!(points.contains(&[2., 0.]));
        assert!(!points.contains(&[2., 2.]));
        assert_eq!(disk(1.0), vec![[0., 0.]]);
    }
}
