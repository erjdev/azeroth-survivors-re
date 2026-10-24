//! Read-only WMO floor evidence in raw WoW coordinates. Not a navigation proof.
use anyhow::{Context, Result};
use benilla_formats::{Chain, MapTiles};
use glam::{Quat, Vec3};
use std::collections::HashSet;

fn placement(rotation: [f32; 3]) -> Quat {
    // Same MODF transform as benilla-assets::coords::placement_rotation, before
    // its Bevy basis conjugation. Keep this audit independent of the renderer.
    use std::f32::consts::{FRAC_PI_2, PI};
    Quat::from_rotation_x(FRAC_PI_2)
        * Quat::from_rotation_y(rotation[1].to_radians() - PI)
        * Quat::from_rotation_z(-rotation[0].to_radians())
        * Quat::from_rotation_x(rotation[2].to_radians() - FRAC_PI_2)
}

fn floor_z(triangle: &[Vec3; 3], point: Vec3) -> Option<f32> {
    let [a, b, c] = *triangle;
    let ab = b - a;
    let ac = c - a;
    let normal = ab.cross(ac);
    if normal.normalize_or_zero().z <= 0.5 {
        return None;
    }
    let p = point - a;
    let u = (p.x * ac.y - p.y * ac.x) / normal.z;
    let v = (ab.x * p.y - ab.y * p.x) / normal.z;
    (u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001).then_some(a.z + u * ab.z + v * ac.z)
}

fn nearby_floor(triangles: &[[Vec3; 3]], point: Vec3) -> Option<f32> {
    triangles
        .iter()
        .filter_map(|t| floor_z(t, point))
        .filter(|z| *z <= point.z + 2.0 && *z >= point.z - 8.0)
        .max_by(f32::total_cmp)
}

pub fn probe(chain: &mut Chain, map: u32, point: [f32; 3]) -> Result<()> {
    let maps = benilla_formats::load_map_catalog(chain)?;
    let directory = maps.directory(map).context("map has no directory")?;
    let tiles = MapTiles::load(chain, directory)?;
    let (tx, ty) = tiles.tile_at(point[0], point[1]);
    let tile = benilla_formats::load_tile_mesh(chain, directory, tx, ty)?;
    println!("FLOOR PROBE: map {map} ({directory}) tile {tx},{ty} at {point:?}");
    println!(
        "Terrain height: {:?}",
        benilla_formats::terrain_height_at(&tile.chunks, point)
    );
    let mut seen = HashSet::new();
    let mut triangles = Vec::new();
    for instance in tile.wmos {
        if !seen.insert(instance.unique_id) {
            continue;
        }
        // Unlike the convenience loader, fail on any missing group; partial geometry must
        // never look like complete evidence.
        let root = benilla_formats::parse_wmo_root(&chain.read_file(&instance.model)?)?;
        let stem = instance
            .model
            .strip_suffix(".wmo")
            .context("WMO extension")?;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for group in 0..root.group_count() {
            let path = format!("{stem}_{group:03}.wmo");
            let bytes = chain
                .read_file(&path)
                .with_context(|| format!("required group {path}"))?;
            let parsed = benilla_wmo::parse_wmo(&mut std::io::Cursor::new(&bytes))
                .map_err(|error| anyhow::anyhow!("invalid group {path}: {error}"))?;
            if !matches!(parsed, benilla_wmo::ParsedWmo::Group(_)) {
                anyhow::bail!("{path} is not a WMO group");
            }
            benilla_formats::accumulate_wmo_group_collision(&bytes, &mut positions, &mut indices);
        }
        println!(
            "WMO {}: position {:?}, rotation {:?}, {} groups, {} walking triangles",
            instance.model,
            instance.position,
            instance.rotation,
            root.group_count(),
            indices.len() / 3
        );
        let rotation = placement(instance.rotation);
        let translation = Vec3::from_array(instance.position);
        let vertices: Vec<_> = positions
            .into_iter()
            .map(|p| translation + rotation * Vec3::from_array(p))
            .collect();
        triangles.extend(indices.as_chunks::<3>().0.iter().map(|t| {
            [
                vertices[t[0] as usize],
                vertices[t[1] as usize],
                vertices[t[2] as usize],
            ]
        }));
    }
    let point = Vec3::from_array(point);
    let mut heights: Vec<_> = triangles.iter().filter_map(|t| floor_z(t, point)).collect();
    heights.sort_by(f32::total_cmp);
    heights.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    println!("WMO floor heights at XY: {heights:?}");
    println!(
        "Runtime-window WMO floor: {:?}",
        nearby_floor(&triangles, point)
    );
    for radius in [10., 20., 30., 40., 60., 80.] {
        let count = (0..64)
            .filter(|i| {
                let angle = *i as f32 * std::f32::consts::TAU / 64.;
                let p = point + Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.);
                nearby_floor(&triangles, p).is_some()
            })
            .count();
        println!("WMO floor ring r={radius}: {count}/64 samples within runtime height window");
    }
    println!("Scope: walking faces of complete WMO groups referenced by the start ADT. Excludes doodads, neighbouring-only placements, walls, connectivity, camera visibility and rendered identity. Ring samples are not proof of a traversable disk.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_probe_interpolates_and_rejects_backfaces_and_outside_points() {
        let t = [
            Vec3::new(0., 0., 10.),
            Vec3::new(10., 0., 12.),
            Vec3::new(0., 10., 10.),
        ];
        assert_eq!(floor_z(&t, Vec3::new(5., 2., 99.)), Some(11.));
        assert_eq!(floor_z(&t, Vec3::new(8., 8., 10.)), None);
        assert_eq!(floor_z(&[t[0], t[2], t[1]], Vec3::new(1., 1., 10.)), None);
        assert_eq!(nearby_floor(&[t], Vec3::new(1., 1., 30.)), None);
    }
    #[test]
    fn zero_modf_rotation_preserves_height_and_flips_horizontal_axes() {
        let p = placement([0.; 3]) * Vec3::new(3., 4., 5.);
        assert!((p - Vec3::new(-3., -4., 5.)).length() < 1e-5);
    }
}
