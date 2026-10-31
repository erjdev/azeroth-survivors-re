//! Source-hull evidence for nearby outdoor placements; does not prove runtime collision.
use anyhow::{Context, Result};
use benilla_formats::{Chain, MapTiles};
use glam::Vec3;
use std::collections::{BTreeMap, HashSet};

pub fn probe(chain: &mut Chain, map: u32, point: [f32; 3]) -> Result<()> {
    let maps = benilla_formats::load_map_catalog(chain)?;
    let directory = maps.directory(map).context("map has no directory")?;
    let tiles = MapTiles::load(chain, directory)?;
    let mut seen = HashSet::new();
    let mut models = BTreeMap::new();
    let mut nearest: [Option<(f32, u32)>; 4] = [None; 4];
    let axes = [Vec3::NEG_Y, Vec3::Y, Vec3::NEG_X, Vec3::X];
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
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for vertex in &hull.positions {
                    for axis in 0..3 {
                        min[axis] = min[axis].min(vertex[axis]);
                        max[axis] = max[axis].max(vertex[axis]);
                    }
                }
                if !hull.positions.is_empty() {
                    println!("LOCAL HULL {model}: min {min:?}, max {max:?}; model-local coordinates before placement rotation and scale");
                }
                models.insert(model.clone(), (hull, 0usize));
            }
            let (hull, count) = models.get_mut(&model).unwrap();
            let triangles = hull.triangle_count();
            *count += 1;
            let rotation = super::floor::placement(d.rotation);
            let center = Vec3::from_array(d.position);
            let vertices: Vec<_> = hull
                .positions
                .iter()
                .map(|p| center + rotation * (Vec3::from_array(*p) * d.scale))
                .collect();
            if !vertices.is_empty() {
                // Generalized winding distinguishes starting inside a closed authored hull
                // from entering it. One-sided movement deliberately allows leaving a hull.
                let p = Vec3::from_array(point);
                let winding = winding_number(&vertices, hull.indices.as_chunks::<3>().0, p);
                if distance < 50.0 && model.contains("\\trees\\") {
                    println!("SOURCE WINDING {}: {winding:.6} at {point:?}; magnitude near 1 means inside a closed consistently wound hull, near 0 outside; open/inconsistent hulls are inconclusive", d.unique_id);
                }
                let min = vertices
                    .iter()
                    .fold(Vec3::splat(f32::INFINITY), |a, b| a.min(*b));
                let max = vertices
                    .iter()
                    .fold(Vec3::splat(f32::NEG_INFINITY), |a, b| a.max(*b));
                let p = Vec3::from_array(point);
                let horizontal = horizontal_bounds_distance(p, min, max);
                println!("WORLD HULL {}: min {:?}, max {:?}, horizontal AABB distance {horizontal:.3}; bounds are conservative, not exact occupancy",d.unique_id,min.to_array(),max.to_array());
                for (index, axis) in axes.iter().enumerate() {
                    for ids in hull.indices.as_chunks::<3>().0 {
                        if let Some(distance) =
                            ray_triangle(p, *axis, ids.map(|i| vertices[i as usize]))
                                .filter(|distance| *distance <= 120.0)
                        {
                            if nearest[index].is_none_or(|(before, _)| distance < before) {
                                nearest[index] = Some((distance, d.unique_id));
                            }
                        }
                    }
                }
            }

            if distance < 50.0 && model.contains("\\trees\\") {
                let rotation = super::floor::placement(d.rotation);
                let center = Vec3::from_array(d.position);
                let vertices: Vec<_> = hull
                    .positions
                    .iter()
                    .map(|p| center + rotation * (Vec3::from_array(*p) * d.scale))
                    .collect();
                // Mirror the live probe's four Bevy directions in raw WoW coordinates.
                for axis in [Vec3::NEG_Y, Vec3::Y, Vec3::NEG_X, Vec3::X] {
                    let mut start = center - axis * 6.0;
                    let Some(z) =
                        benilla_formats::terrain_height_at(&tile.chunks, start.to_array())
                    else {
                        continue;
                    };
                    start.z = z + 1.0;
                    let nearest = hull
                        .indices
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .filter_map(|ids| {
                            ray_triangle(start, axis, ids.map(|i| vertices[i as usize]))
                        })
                        .filter(|d| *d <= 12.0)
                        .min_by(f32::total_cmp);
                    println!("SOURCE RAY {}: wow start {:?}, direction {axis:?}, hit {nearest:?}; terrain-grounded, this placement only", d.unique_id, start.to_array());
                }
                // Horizontal slice exposes offset trunks without mistaking the origin for one.
                let z = center.z + 1.0;
                let mut slice = Vec::new();
                for ids in hull.indices.as_chunks::<3>().0 {
                    let t = ids.map(|i| vertices[i as usize]);
                    for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                        if (a.z < z && b.z >= z) || (b.z < z && a.z >= z) {
                            slice.push(a.lerp(b, (z - a.z) / (b.z - a.z)));
                        }
                    }
                }
                if !slice.is_empty() {
                    let min = slice
                        .iter()
                        .fold(Vec3::splat(f32::INFINITY), |a, b| a.min(*b));
                    let max = slice
                        .iter()
                        .fold(Vec3::splat(f32::NEG_INFINITY), |a, b| a.max(*b));
                    println!(
                        "SOURCE SLICE {}: wow min {:?}, max {:?}, surface sample {:?}; one yard above placement origin",
                        d.unique_id,
                        min.to_array(),
                        max.to_array(),
                        slice[0].to_array()
                    );
                }
            }
            println!("PLACEMENT {}: {model}, position {:?}, rotation {:?}, scale {}, distance {distance:.1}, hull triangles {triangles}", d.unique_id, d.position, d.rotation, d.scale);
        }
    }
    for (axis, hit) in axes.iter().zip(nearest) {
        println!("CANDIDATE RAY: start {point:?}, direction {axis:?}, nearest {hit:?}; (distance, placement id), limit 120 yards, one-sided hulls only");
    }
    for (model, (hull, count)) in models {
        let triangles = hull.triangle_count();
        println!("MODEL {model}: {count} placements, {triangles} hull triangles");
    }
    println!("Scope: authored hulls for ADT doodad origins within 200 yards in the surrounding 3×3 existing tiles. Excludes WMO props and distant origins whose geometry overlaps the area. A hull does not prove correct runtime transforms, registration, sweep behavior or navigation. Zero triangles means no usable hull from the parser, not necessarily intentional pass-through.");
    Ok(())
}

fn winding_number(vertices: &[Vec3], triangles: &[[u32; 3]], point: Vec3) -> f32 {
    triangles
        .iter()
        .map(|ids| {
            let [a, b, c] = ids.map(|i| vertices[i as usize] - point);
            let denominator = a.length() * b.length() * c.length()
                + a.dot(b) * c.length()
                + b.dot(c) * a.length()
                + c.dot(a) * b.length();
            2.0 * a.dot(b.cross(c)).atan2(denominator)
        })
        .sum::<f32>()
        / (4.0 * std::f32::consts::PI)
}

fn horizontal_bounds_distance(point: Vec3, min: Vec3, max: Vec3) -> f32 {
    let nearest = point.clamp(min, max);
    (point.x - nearest.x).hypot(point.y - nearest.y)
}

/// One-sided triangle hit distance, matching the live body's front-face policy.
fn ray_triangle(origin: Vec3, direction: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let ab = b - a;
    let ac = c - a;
    if ab.cross(ac).normalize_or_zero().dot(direction) >= -1e-5 {
        return None;
    }
    let p = direction.cross(ac);
    let det = ab.dot(p);
    if det.abs() < 1e-8 {
        return None;
    }
    let t = origin - a;
    let u = t.dot(p) / det;
    let q = t.cross(ab);
    let v = direction.dot(q) / det;
    let distance = ac.dot(q) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && distance >= 0.0).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn winding_distinguishes_closed_hull_interior_exterior_and_reversal() {
        let vertices = [
            Vec3::new(1., 1., 1.),
            Vec3::new(-1., -1., 1.),
            Vec3::new(-1., 1., -1.),
            Vec3::new(1., -1., -1.),
        ];
        let faces = [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];
        let inside = winding_number(&vertices, &faces, Vec3::ZERO);
        assert!((inside.abs() - 1.).abs() < 1e-5);
        assert!(winding_number(&vertices, &faces, Vec3::splat(5.)).abs() < 1e-5);
        let reversed = faces.map(|[a, b, c]| [a, c, b]);
        assert!((winding_number(&vertices, &reversed, Vec3::ZERO) + inside).abs() < 1e-5);
        assert!((winding_number(&vertices, &faces[..3], Vec3::ZERO).abs() - 0.75).abs() < 1e-5);
    }

    #[test]
    fn bounds_distance_uses_horizontal_extent_not_origin_or_altitude() {
        let min = Vec3::new(-10., -2., 5.);
        let max = Vec3::new(10., 2., 6.);
        assert_eq!(
            horizontal_bounds_distance(Vec3::new(0., 0., 100.), min, max),
            0.
        );
        assert_eq!(
            horizontal_bounds_distance(Vec3::new(13., 6., 0.), min, max),
            5.
        );
    }

    #[test]
    fn source_ray_respects_winding_and_triangle_extent() {
        let triangle = [Vec3::ZERO, Vec3::X, Vec3::Y];
        assert_eq!(
            ray_triangle(Vec3::new(0.2, 0.2, 2.0), Vec3::NEG_Z, triangle),
            Some(2.0)
        );
        assert_eq!(
            ray_triangle(Vec3::new(0.2, 0.2, -2.0), Vec3::Z, triangle),
            None
        );
        assert_eq!(
            ray_triangle(Vec3::new(0.8, 0.8, 2.0), Vec3::NEG_Z, triangle),
            None
        );
    }
}
