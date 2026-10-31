//! Validate held NPC displays using the same Weapon/Shield directories as the renderer.
use anyhow::{bail, Context, Result};
use benilla_formats::Chain;
use std::collections::BTreeSet;

fn manifest(text: &str) -> Result<BTreeSet<(u32, u32)>> {
    let mut result = BTreeSet::new();
    for (line, row) in text.lines().enumerate() {
        let fields: Vec<_> = row.split_whitespace().collect();
        if fields.len() != 2 {
            bail!("line {}: expected display ID and inventory type", line + 1);
        }
        let display = fields[0].parse::<u32>()?;
        let inventory = fields[1].parse::<u32>()?;
        if display == 0 || inventory > 28 {
            bail!("line {}: invalid display/inventory", line + 1);
        }
        result.insert((display, inventory));
    }
    if result.is_empty() {
        bail!("empty equipment manifest");
    }
    Ok(result)
}

pub fn report(chain: &mut Chain, text: &str) -> Result<()> {
    let manifest = manifest(text)?;
    let displays = benilla_formats::load_item_display_catalog(chain)?;
    let mut textures = BTreeSet::new();
    let mut models = BTreeSet::new();
    let mut gaps = Vec::new();
    for &(id, inventory) in &manifest {
        let checked = (|| -> Result<()> {
            let display = displays.get(id).context("missing ItemDisplayInfo row")?;
            let model = display.model[0].as_ref().context("missing held model")?;
            let folder = if inventory == 14 { "Shield" } else { "Weapon" };
            let dir = format!("Item\\ObjectComponents\\{folder}");
            let path = format!("{dir}\\{model}");
            if !models.contains(&path) {
                let meshes = benilla_formats::load_object_model(chain, &path)?;
                if !meshes.iter().any(|mesh| !mesh.indices.is_empty()) {
                    bail!("no render triangles: {path}");
                }
                textures.extend(meshes.into_iter().filter_map(|mesh| mesh.texture));
                models.insert(path);
            }
            if let Some(texture) = &display.model_texture[0] {
                textures.insert(format!("{dir}\\{texture}.blp"));
            }
            Ok(())
        })();
        if let Err(error) = checked {
            gaps.push(format!("display {id}, inventory {inventory}: {error:#}"));
        }
    }
    for texture in &textures {
        let checked = chain
            .read_file(texture)
            .and_then(|bytes| Ok(benilla_blp::decode(&bytes)?));
        match checked {
            Ok(image) if image.width > 0 && image.height > 0 => {}
            other => gaps.push(format!(
                "texture {texture}: {}",
                match other {
                    Ok(_) => "empty image".to_owned(),
                    Err(error) => format!("{error:#}"),
                }
            )),
        }
    }
    println!("NPC EQUIPMENT: {} display/inventory pairs, {} parsed models, {} texture references checked", manifest.len(), models.len(), textures.len());
    println!("Scope: asset decoding only; no proof of attachment alignment, animations, rendered appearance or disarm gameplay.");
    if !gaps.is_empty() {
        bail!("{} equipment asset gaps:\n{}", gaps.len(), gaps.join("\n"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_rejects_empty_malformed_and_out_of_range_entries() {
        for invalid in ["", "1", "1 13 extra", "0 13", "1 29", "-1 13", "1 x"] {
            assert!(manifest(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn manifest_deduplicates_without_losing_directory_identity() {
        assert_eq!(
            manifest("123 13\n123 14\n123 13\n").unwrap(),
            BTreeSet::from([(123, 13), (123, 14)])
        );
    }
}
