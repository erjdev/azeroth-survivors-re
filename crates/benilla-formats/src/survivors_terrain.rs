//! Survivors terrain lookup: recover fast-index misses against the emitted surface.
use crate::{ChunkMesh, CHUNK_SIZE};

/// Preserve the indexed height result; on a miss, use actual nearby mesh triangles.
/// Authored holes have no indices, so this does not bridge them or extrapolate terrain.
pub fn terrain_height_at(chunks: &[ChunkMesh], wow: [f32; 3]) -> Option<f32> {
    crate::terrain::terrain_height_at(chunks, wow).or_else(|| {
        chunks.iter().find_map(|chunk| {
            let nw = *chunk.positions.first()?;
            // A generous broad phase only; triangle containment remains exact. MCNK vertices
            // are snapped to a shared lattice, unlike the arithmetic fast-index footprint.
            if !(nw[0] - CHUNK_SIZE - 0.01..=nw[0] + 0.01).contains(&wow[0])
                || !(nw[1] - CHUNK_SIZE - 0.01..=nw[1] + 0.01).contains(&wow[1])
            {
                return None;
            }
            chunk.indices.as_chunks::<3>().0.iter().find_map(|t| {
                let triangle = [
                    chunk.positions[t[0] as usize],
                    chunk.positions[t[1] as usize],
                    chunk.positions[t[2] as usize],
                ];
                crate::triangle_z_at(&triangle, wow[0], wow[1])
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ungoro_boundary_recovers_mesh_support_without_filling_holes() {
        let data = crate::wow_data_or_skip!();
        let mut chain = crate::open_chain(&data).expect("open explicit test data");
        let maps = crate::MapTiles::load(&mut chain, "Kalimdor").unwrap();
        let pin = [-7100., -1625., 0.];
        let (x, y) = maps.tile_at(pin[0], pin[1]);
        let tile = crate::load_tile_mesh(&mut chain, "Kalimdor", x, y).unwrap();
        assert_eq!(crate::terrain::terrain_height_at(&tile.chunks, pin), None);
        assert!((terrain_height_at(&tile.chunks, pin).unwrap() + 271.66095).abs() < 0.001);
        for offset in [-0.01, 0., 0.01] {
            let point = [pin[0] + offset, pin[1], 0.];
            let actual = terrain_height_at(&tile.chunks, point).unwrap();
            if let Some(fast) = crate::terrain::terrain_height_at(&tile.chunks, point) {
                assert_eq!(actual, fast);
            }
            assert!((actual + 271.66095).abs() < 0.01);
        }
        let mut holes = tile.chunks.clone();
        for chunk in &mut holes {
            chunk.holes = u16::MAX;
            chunk.indices.clear();
        }
        assert_eq!(terrain_height_at(&holes, pin), None);
        assert_eq!(terrain_height_at(&[], pin), None);
        assert_eq!(terrain_height_at(&tile.chunks, [0.; 3]), None);
    }
}
