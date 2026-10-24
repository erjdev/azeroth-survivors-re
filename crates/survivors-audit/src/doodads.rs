//! Source-hull evidence for nearby outdoor placements; does not prove runtime collision.
use anyhow::{Context, Result};
use benilla_formats::{Chain, MapTiles};
use std::collections::{BTreeMap, HashSet};

pub fn probe(chain: &mut Chain, map: u32, point: [f32; 3]) -> Result<()> {
    let maps = benilla_formats::load_map_catalog(chain)?;
    let directory = maps.directory(map).context("map has no directory")?;
    let tiles = MapTiles::load(chain, directory)?;
    let mut seen = HashSet::new();
    let mut models = BTreeMap::new();
    println!(
        "DOODAD HULL PROBE: map {map} at {point:?}; placement origins within 200 horizontal yards"
    );
    for (x, y) in tiles.existing_in_radius(point[0], point[1], 1) {
        let tile = benilla_formats::load_tile_mesh(chain, directory, x, y)?;
        for d in tile.doodads {
            let distance = (d.position[0] - point[0]).hypot(d.position[1] - point[1]);
            if distance > 200.0 || !seen.insert(d.unique_id) {
                continue;
            }
            let model = d.model.to_ascii_lowercase();
            if !models.contains_key(&model) {
                let hull = benilla_formats::load_m2_collision_hull(chain, &model)?;
                models.insert(model.clone(), (hull.triangle_count(), 0usize));
            }
            let (triangles, count) = models.get_mut(&model).unwrap();
            *count += 1;
            println!("PLACEMENT {}: {model}, position {:?}, rotation {:?}, scale {}, distance {distance:.1}, hull triangles {triangles}", d.unique_id, d.position, d.rotation, d.scale);
        }
    }
    for (model, (triangles, count)) in models {
        println!("MODEL {model}: {count} placements, {triangles} hull triangles");
    }
    println!("Scope: authored hulls for ADT doodad origins within 200 yards in the surrounding 3×3 existing tiles. Excludes WMO props and distant origins whose geometry overlaps the area. A hull does not prove correct runtime transforms, registration, sweep behavior or navigation. Zero triangles means no usable hull from the parser, not necessarily intentional pass-through.");
    Ok(())
}
