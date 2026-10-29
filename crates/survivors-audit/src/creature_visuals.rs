//! Decode plain creature displays with their actual replacement skins.
use anyhow::{bail, Context, Result};
use benilla_formats::Chain;
use std::collections::BTreeSet;

pub fn report(chain: &mut Chain, ids: &str) -> Result<()> {
    let ids: BTreeSet<u32> = ids
        .split(',')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    let catalog = benilla_formats::load_creature_catalog(chain)?;
    let mut textures = BTreeSet::new();
    for id in ids {
        let model = catalog
            .model(id)
            .with_context(|| format!("missing display {id}"))?;
        if model.npc_appearance.is_some() {
            bail!("display {id}: character composition requires a separate audit");
        }
        let meshes =
            benilla_formats::load_m2_mesh_skinned(chain, &model.model_path, &model.textures)?;
        let triangles: usize = meshes.iter().map(|mesh| mesh.indices.len() / 3).sum();
        if triangles == 0 {
            bail!("display {id}: no render triangles");
        }
        for mesh in &meshes {
            if mesh.skin_slot.is_some() && mesh.texture.is_none() {
                bail!(
                    "display {id}: unresolved creature skin slot {:?}",
                    mesh.skin_slot
                );
            }
            if let Some(path) = &mesh.texture {
                textures.insert(path.to_ascii_lowercase());
            }
        }
        println!(
            "DISPLAY {id}: {} · scale {} · {} batches · {triangles} triangles · skins {:?}",
            model.model_path,
            model.scale,
            meshes.len(),
            model.textures
        );
    }
    for path in &textures {
        let image = benilla_blp::decode(&chain.read_file(path)?)
            .with_context(|| format!("decoding {path}"))?;
        if image.width == 0 || image.height == 0 {
            bail!("empty texture {path}");
        }
        println!("TEXTURE {path}: {}x{}", image.width, image.height);
    }
    println!("{} unique mesh textures decoded. Scope: geometry and mesh textures only; animations, particles, appearance and gameplay require native acceptance.", textures.len());
    Ok(())
}
