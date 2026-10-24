//! Read-only evidence for catalog completion. An explicit data path prevents install discovery.
use anyhow::{bail, Context, Result};
use benilla_formats::{load_creature_catalog, load_map_catalog, open_chain, MapTiles};
use survivors_core::{campaign, data, encounters};

mod floor;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .context("usage: survivors-audit <explicit WoW Data directory>")?;
    let rest: Vec<_> = args.collect();
    let mut chain = open_chain(std::path::Path::new(&path))?;
    if !rest.is_empty() {
        if rest.len() != 5 || rest[0] != "--floor" {
            bail!("usage: survivors-audit <Data> [--floor <map> <x> <y> <z>]");
        }
        let map = rest[1].to_str().context("map must be UTF-8")?.parse()?;
        let mut point = [0.0_f32; 3];
        for (value, arg) in point.iter_mut().zip(&rest[2..]) {
            *value = arg.to_str().context("coordinate must be UTF-8")?.parse()?;
            if !value.is_finite() {
                bail!("coordinates must be finite");
            }
        }
        return floor::probe(&mut chain, map, point);
    }
    let maps = load_map_catalog(&mut chain)?;
    let creatures = load_creature_catalog(&mut chain)?;
    let mut gaps = 0;
    for zone in campaign::ZONES {
        let Some(index) = data::MAPS.iter().position(|map| map.name == zone.map) else {
            println!("MISSING MAP: {}", zone.name);
            gaps += 1;
            continue;
        };
        let map = &data::MAPS[index];
        // Map.dbc in the 1.12.1 patch chain identifies these instanced destinations.
        let expected_map = campaign::required_map_id(zone.name);
        if let Some(expected) = expected_map.filter(|expected| *expected != map.map) {
            println!(
                "WRONG DESTINATION: {} · configured map {} · expected {} ({:?})",
                zone.name,
                map.map,
                expected,
                maps.name(expected)
            );
            gaps += 1;
        }
        if !encounters::ready(index) {
            println!("MISSING ENCOUNTERS: {}", zone.name);
            gaps += 1;
        }
        let Some(directory) = maps.directory(map.map) else {
            println!("MISSING MAP DBC ROW: {} ({})", zone.name, map.map);
            gaps += 1;
            continue;
        };
        let surface = (|| -> Result<Option<f32>> {
            let tiles = MapTiles::load(&mut chain, directory)?;
            let (x, y) = tiles.tile_at(map.center[0], map.center[1]);
            let mesh = benilla_formats::load_tile_mesh(&mut chain, directory, x, y)?;
            Ok(benilla_formats::terrain_height_at(&mesh.chunks, map.center))
        })();
        match surface {
            Ok(Some(height)) => println!(
                "TERRAIN: {} · map {} · start z {:.2} · terrain z {:.2}",
                zone.name, map.map, map.center[2], height
            ),
            other => {
                // Indoor WMO floors require a separate geometry check, not an invented ADT height.
                println!("NEEDS FLOOR VALIDATION: {} · {other:?}", zone.name);
                gaps += 1;
            }
        }
        let roster = data::roster(index);
        for enemy in roster.enemies.iter().chain(&roster.bosses) {
            let model = creatures.model(enemy.display);
            let exists = model.as_ref().is_some_and(|model| {
                let path = model
                    .model_path
                    .replace(".mdx", ".m2")
                    .replace(".MDX", ".M2");
                chain.contains(&path)
            });
            if !exists {
                println!(
                    "MISSING CREATURE MODEL: {} · {} · display {}",
                    zone.name, enemy.name, enemy.display
                );
                gaps += 1;
            }
        }
    }
    for (class, def) in data::CLASSES.iter().enumerate() {
        let mut unnamed = 0;
        let mut unsourced = 0;
        for band in 0..campaign::BANDS {
            for slot in 0..campaign::GRID_SLOTS {
                unnamed += usize::from(campaign::piece_name(class, slot, band).is_none());
                unsourced += usize::from(
                    !campaign::DROPS
                        .iter()
                        .any(|row| row.band == band && row.slot == slot),
                );
            }
        }
        println!(
            "COLLECTION: {} · {unnamed} unnamed cells · {unsourced} cells without drops",
            def.name
        );
        gaps += unnamed + unsourced;
    }
    println!("Scope: map rows, start terrain, roster model files, item names and drop coverage.\nNot proof of traversal, name/model identity, item icons/appearances, talents, UI or raid-set completion.");
    if gaps > 0 {
        bail!("{gaps} content audit gaps remain");
    }
    Ok(())
}
