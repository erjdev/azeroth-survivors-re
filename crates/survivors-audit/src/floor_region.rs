//! Conservative source-face connectivity; never a gameplay readiness verdict.
use glam::Vec3;
use std::collections::{HashMap, HashSet};

type Vertex = [u32; 3];
type Edge = [Vertex; 2];

#[derive(Debug)]
pub(super) struct Region {
    pub faces: usize,
    pub area: f64,
    pub minimum: Vec3,
    pub maximum: Vec3,
    pub boundary_edges: usize,
    pub ambiguous_edges: usize,
}

fn vertex(point: Vec3) -> Vertex {
    point
        .to_array()
        .map(|v| if v == 0. { 0 } else { v.to_bits() })
}
fn edges(face: &[Vec3; 3]) -> [Edge; 3] {
    let [a, b, c] = face.map(vertex);
    [[a, b], [b, c], [c, a]].map(|mut edge| {
        edge.sort_unstable();
        edge
    })
}

pub(super) fn connected(triangles: &[[Vec3; 3]], anchor: Vec3) -> Option<Region> {
    if !anchor.is_finite() {
        return None;
    }
    let mut unique = HashSet::new();
    let faces: Vec<_> = triangles
        .iter()
        .filter(|face| {
            let normal = (face[1] - face[0]).cross(face[2] - face[0]);
            if !face.iter().all(|v| v.is_finite()) || normal.normalize_or_zero().z <= 0.5 {
                return false;
            }
            let mut key = face.map(vertex);
            key.sort_unstable();
            unique.insert(key)
        })
        .collect();
    let seed = faces
        .iter()
        .enumerate()
        .filter_map(|(index, face)| {
            let z = crate::floor::floor_z(face, anchor)?;
            (z >= anchor.z - 8. && z <= anchor.z + 2.).then_some((index, z))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))?
        .0;
    let mut owners: HashMap<Edge, Vec<usize>> = HashMap::new();
    for (index, face) in faces.iter().enumerate() {
        for edge in edges(face) {
            owners.entry(edge).or_default().push(index);
        }
    }
    let mut visited = HashSet::new();
    let mut pending = vec![seed];
    let mut boundary = HashSet::new();
    let mut ambiguous = HashSet::new();
    let mut result = Region {
        faces: 0,
        area: 0.,
        minimum: Vec3::splat(f32::INFINITY),
        maximum: Vec3::splat(f32::NEG_INFINITY),
        boundary_edges: 0,
        ambiguous_edges: 0,
    };
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        let face = faces[index];
        result.faces += 1;
        result.area += f64::from((face[1] - face[0]).cross(face[2] - face[0]).length()) * 0.5;
        for &v in face {
            result.minimum = result.minimum.min(v);
            result.maximum = result.maximum.max(v);
        }
        for edge in edges(face) {
            match owners[&edge].as_slice() {
                [a, b] => pending.push(if *a == index { *b } else { *a }),
                [_] => {
                    boundary.insert(edge);
                }
                _ => {
                    ambiguous.insert(edge);
                }
            }
        }
    }
    result.boundary_edges = boundary.len();
    result.ambiguous_edges = ambiguous.len();
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn square(z: f32) -> [[Vec3; 3]; 2] {
        let a = Vec3::new(0., 0., z);
        let b = Vec3::new(2., 0., z);
        let c = Vec3::new(2., 2., z);
        let d = Vec3::new(0., 2., z);
        [[a, b, c], [a, c, d]]
    }
    #[test]
    fn shared_edges_connect_but_duplicates_do_not_inflate_area() {
        let mut faces = square(0.).to_vec();
        faces.push(faces[0]);
        let r = connected(&faces, Vec3::new(0.5, 1., 1.)).unwrap();
        assert_eq!(r.faces, 2);
        assert_eq!(r.area, 4.);
        assert_eq!(r.boundary_edges, 4);
        assert_eq!(r.ambiguous_edges, 0);
    }
    #[test]
    fn overlapping_storeys_and_corner_contacts_stay_separate() {
        let mut faces = square(0.).to_vec();
        faces.extend(square(20.));
        faces.extend(square(0.).map(|face| face.map(|v| v + Vec3::new(2., 2., 0.))));
        let r = connected(&faces, Vec3::new(1., 1., 21.)).unwrap();
        assert_eq!(r.faces, 2);
        assert_eq!(r.minimum.z, 20.);
        assert_eq!(connected(&faces, Vec3::new(1., 1., 1.)).unwrap().faces, 2);
        assert!(connected(&faces, Vec3::new(1., 1., 11.)).is_none());
    }
    #[test]
    fn connected_slope_retains_height_and_does_not_cross_steep_faces() {
        let faces = square(0.).map(|face| face.map(|v| Vec3::new(v.x, v.y, v.x * 0.5)));
        let r = connected(&faces, Vec3::new(1., 1., 1.)).unwrap();
        assert_eq!(r.faces, 2);
        assert_eq!(r.maximum.z, 1.);
        let steep = square(0.).map(|face| face.map(|v| Vec3::new(v.x, v.y, v.x * 3.)));
        assert!(connected(&steep, Vec3::new(1., 1., 3.)).is_none());
        assert!(connected(&faces, Vec3::NAN).is_none());
    }
    #[test]
    fn non_manifold_edges_are_reported_without_traversal() {
        let mut faces = square(0.).to_vec();
        faces.push([Vec3::ZERO, Vec3::new(2., 2., 0.), Vec3::new(0., 3., 1.)]);
        let r = connected(&faces, Vec3::new(1.5, 0.25, 0.)).unwrap();
        assert_eq!(r.faces, 1);
        assert_eq!(r.ambiguous_edges, 1);
        assert_eq!(r.boundary_edges, 2);
    }
}
