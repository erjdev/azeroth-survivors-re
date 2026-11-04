//! Candidate locations derived from local map metadata, not authored spawn coordinates.
use anyhow::Result;
use benilla_formats::{Chain, MapTiles};
use survivors_core::campaign;

fn normalized(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn report(chain: &mut Chain) -> Result<()> {
    let maps = benilla_formats::load_map_catalog(chain)?;
    let areas = benilla_formats::load_world_map_area_catalog(chain)?;
    let names = benilla_formats::load_area_table_catalog(chain)?;
    println!("# Campaign location candidates\n\nDerived from the explicitly supplied local Map.dbc, WorldMapArea.dbc and terrain data.\nMap-art midpoints are investigation candidates, not safe spawn positions or traversal proof.\n");
    for zone in campaign::ZONES {
        println!("## {}\n", zone.name);
        let expected = campaign::required_map_id(zone.name);
        if let Some(id) = expected {
            println!(
                "Required instance: {id}, {:?}, directory {:?}.\n",
                maps.name(id),
                maps.directory(id)
            );
        }
        let rows: Vec<_> = areas
            .iter()
            .filter(|(_, area)| {
                expected.map_or_else(
                    || {
                        normalized(&area.name) == normalized(zone.name)
                            || names
                                .name(area.area_id)
                                .is_some_and(|name| normalized(name) == normalized(zone.name))
                    },
                    |id| area.map_id == id,
                )
            })
            .collect();
        if rows.is_empty() {
            println!("No matching WorldMapArea row. Resolve the location from other local geometry/area metadata before authoring a spawn.\n");
        }
        for (id, area) in rows {
            println!(
                "WorldMapArea row {id}: `{}`, map {}, area {}.\n",
                area.name, area.map_id, area.area_id
            );
            let center = [
                (area.loc_top + area.loc_bottom) * 0.5,
                (area.loc_left + area.loc_right) * 0.5,
                0.0,
            ];
            if area.loc_top == area.loc_bottom
                || area.loc_left == area.loc_right
                || !center.iter().all(|v| v.is_finite())
            {
                println!("No usable map-art rectangle.\n");
                continue;
            }
            println!(
                "Map-art midpoint: x {:.3}, y {:.3}.\n",
                center[0], center[1]
            );
            let terrain = (|| -> Result<Option<f32>> {
                let dir = maps
                    .directory(area.map_id)
                    .ok_or_else(|| anyhow::anyhow!("missing Map.dbc directory"))?;
                let tiles = MapTiles::load(chain, dir)?;
                let (x, y) = tiles.tile_at(center[0], center[1]);
                let tile = benilla_formats::load_tile_mesh(chain, dir, x, y)?;
                Ok(benilla_formats::terrain_height_at(&tile.chunks, center))
            })();
            match terrain {
                Ok(Some(z)) => println!("Terrain at midpoint: z {z:.3}. Check water, slope, WMO floors, obstacles and arena footprint before use.\n"),
                result => println!("No verified terrain support at midpoint: {result:?}. WMO/interior inspection may be required.\n"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zone_names_match_internal_art_names_without_losing_words() {
        assert_eq!(normalized("Un'Goro Crater"), normalized("UngoroCrater"));
        assert_eq!(normalized("Warsong Gulch"), normalized("WarsongGulch"));
        assert_ne!(normalized("Gates of Ahn'Qiraj"), normalized("AhnQiraj"));
    }
}
