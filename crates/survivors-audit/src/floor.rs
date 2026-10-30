//! Read-only WMO floor evidence in raw WoW coordinates. Not a navigation proof.
use anyhow::{Context, Result};
use benilla_formats::{Chain, MapTiles};
use glam::{Quat, Vec3};
use std::collections::HashSet;

pub(super) fn placement(rotation: [f32; 3]) -> Quat {
    // Same MODF transform as benilla-assets::coords::placement_rotation, before
    // its Bevy basis conjugation. Keep this audit independent of the renderer.
    use std::f32::consts::{FRAC_PI_2, PI};
    Quat::from_rotation_x(FRAC_PI_2)
        * Quat::from_rotation_y(rotation[1].to_radians() - PI)
        * Quat::from_rotation_z(-rotation[0].to_radians())
        * Quat::from_rotation_x(rotation[2].to_radians() - FRAC_PI_2)
}

pub(super) fn floor_z(triangle: &[Vec3; 3], point: Vec3) -> Option<f32> {
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

#[derive(Debug, PartialEq)]
struct FloorSample {
    selected: Option<f32>,
    layers: usize,
}

fn sample(triangles: &[[Vec3; 3]], point: Vec3) -> FloorSample {
    let mut heights: Vec<_> = triangles.iter().filter_map(|t| floor_z(t, point)).collect();
    heights.sort_by(f32::total_cmp);
    // Shared triangle edges must not masquerade as stacked storeys.
    heights.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    FloorSample {
        selected: heights
            .iter()
            .rev()
            .copied()
            .find(|z| *z <= point.z + 2.0 && *z >= point.z - 8.0),
        layers: heights.len(),
    }
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
    match crate::floor_region::connected(&triangles, point) {
        Some(region) => println!("WMO floor-face component: {region:?}"),
        None => {
            println!("WMO floor-face component: no eligible anchor face in runtime height window")
        }
    }
    println!("Component scope: exact shared 3D edges of upward faces only; excludes corner contacts and non-manifold edges. Not a navigation mesh: body clearance, walls, water, T-junctions and step links remain unverified.");
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
    let (mut covered, mut samples) = (0, 0);
    let (mut absent, mut other_level, mut stacked) = (0, 0, 0);
    let mut nearest_miss: Option<(f32, Vec3, usize)> = None;
    for x in -30..=30 {
        for y in -30..=30 {
            if x * x + y * y > 900 {
                continue;
            }
            samples += 1;
            let p = point + Vec3::new(x as f32, y as f32, 0.);
            let evidence = sample(&triangles, p);
            covered += usize::from(evidence.selected.is_some());
            stacked += usize::from(evidence.layers > 1);
            if evidence.selected.is_none() {
                absent += usize::from(evidence.layers == 0);
                other_level += usize::from(evidence.layers > 0);
                let distance = (p - point).length();
                if nearest_miss.is_none_or(|(old, _, _)| distance < old) {
                    nearest_miss = Some((distance, p, evidence.layers));
                }
            }
        }
    }
    println!("WMO floor disk r=30 at one-yard spacing: {covered}/{samples} samples within runtime height window");
    println!("Disk exclusions: {absent} with no walking face at XY, {other_level} with faces only outside the height window; {stacked} samples have multiple distinct floor levels");
    if let Some((distance, at, layers)) = nearest_miss {
        println!("Nearest unsupported sample: {distance:.3} horizontal yards at {:?}, {layers} distinct floor levels", at.to_array());
    }
    println!("Scope: walking faces of complete WMO groups referenced by the start ADT. Excludes doodads, neighbouring-only placements, walls, body-clearance routes, camera visibility and rendered identity. Ring samples are not proof of a traversable disk.");
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
    fn floor_samples_distinguish_holes_other_storeys_and_shared_edges() {
        let base = [Vec3::ZERO, Vec3::X * 10., Vec3::Y * 10.];
        let upper = base.map(|p| p + Vec3::Z * 20.);
        let triangles = [base, base, upper];
        assert_eq!(
            sample(&triangles, Vec3::new(2., 2., 1.)),
            FloorSample {
                selected: Some(0.),
                layers: 2,
            }
        );
        assert_eq!(
            sample(&triangles, Vec3::new(2., 2., 11.)),
            FloorSample {
                selected: None,
                layers: 2,
            }
        );
        assert_eq!(
            sample(&triangles, Vec3::new(9., 9., 1.)),
            FloorSample {
                selected: None,
                layers: 0,
            }
        );
        assert_eq!(
            sample(&triangles, Vec3::new(2., 2., 21.)).selected,
            Some(20.)
        );
        for z in [-3., -2., 0., 8., 9., 18., 20., 28., 29.] {
            let point = Vec3::new(2., 2., z);
            assert_eq!(
                sample(&triangles, point).selected,
                nearby_floor(&triangles, point)
            );
        }
    }

    #[test]
    fn zero_modf_rotation_preserves_height_and_flips_horizontal_axes() {
        let p = placement([0.; 3]) * Vec3::new(3., 4., 5.);
        assert!((p - Vec3::new(-3., -4., 5.)).length() < 1e-5);
    }
}
