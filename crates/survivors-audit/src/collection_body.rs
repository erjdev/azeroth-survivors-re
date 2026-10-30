//! Check the body overlays actually selected by the shared equipment compositor.
use anyhow::{bail, Context, Result};
use benilla_formats::{equip_blits, Chain, ItemDisplay};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct BodyArt {
    checked: HashSet<(u32, u8)>,
    textures: HashSet<String>,
    variants: HashSet<(u32, u8, u8)>,
    fallbacks: HashSet<String>,
}

fn body_slot(inventory: u8) -> Option<usize> {
    match inventory {
        4 => Some(0),
        5 | 20 => Some(1),
        6..=10 => Some(usize::from(inventory - 4)),
        19 => Some(7),
        _ => None,
    }
}

fn first_decodable(
    candidates: &[String],
    mut decode: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let mut failures = Vec::new();
    for path in candidates {
        match decode(path) {
            Ok(()) => return Ok(()),
            Err(error) => failures.push(format!("{path}: {error:#}")),
        }
    }
    bail!("no decodable body overlay: {}", failures.join("; "))
}

impl BodyArt {
    pub fn check(
        &mut self,
        chain: &mut Chain,
        id: u32,
        inventory: u8,
        display: &ItemDisplay,
    ) -> Result<()> {
        let Some(slot) = body_slot(inventory) else {
            return Ok(());
        };
        if self.checked.contains(&(id, inventory)) {
            return Ok(());
        }
        let mut equipment = [None; 8];
        equipment[slot] = Some(display);
        let plan = equip_blits(&equipment, None, false);
        // Isolate each piece so another equipped layer cannot mask a missing asset.
        let mut failures = Vec::new();
        for sex in 0..=1 {
            let start = failures.len();
            for step in &plan {
                let result = first_decodable(&step.candidates(sex), |path| {
                    let resolved = survivors_core::appearance_assets::read(true, path, |candidate| {
                        if !self.textures.contains(candidate) {
                            let image = benilla_blp::decode(&chain.read_file(candidate)?)?;
                            if image.width == 0 || image.height == 0 {
                                bail!("empty body overlay");
                            }
                            self.textures.insert(candidate.to_owned());
                        }
                        Ok::<_, anyhow::Error>(candidate.to_owned())
                    })?;
                    if resolved != path && self.fallbacks.insert(path.to_owned()) {
                        println!("SURVIVORS BODY FALLBACK: display {id}, inventory {inventory}, sex {sex}, layer {}: {path} -> {resolved}", step.layer);
                    }
                    Ok(())
                })
                .with_context(|| {
                    format!(
                        "display {id}, inventory {inventory}, sex {sex}, layer {}",
                        step.layer
                    )
                });
                if let Err(error) = result {
                    failures.push(format!("{error:#}"));
                }
            }
            if start == failures.len() {
                self.variants.insert((id, inventory, sex));
            }
        }
        if !failures.is_empty() {
            bail!("{}", failures.join("\n"));
        }
        self.checked.insert((id, inventory));
        Ok(())
    }

    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_resolution_uses_first_decodable_candidate_and_reports_total_failure() {
        let paths = vec!["unisex".to_owned(), "female".to_owned()];
        let mut visited = Vec::new();
        first_decodable(&paths, |path| {
            visited.push(path.to_owned());
            Ok(())
        })
        .unwrap();
        assert_eq!(visited, ["unisex"]);
        visited.clear();
        first_decodable(&paths, |path| {
            visited.push(path.to_owned());
            if path == "unisex" {
                bail!("corrupt")
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(visited, ["unisex", "female"]);
        let error = first_decodable(&paths, |_| bail!("missing"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("unisex: missing"));
        assert!(error.contains("female: missing"));
    }

    #[test]
    fn inventory_types_map_to_compositor_slots_without_treating_held_items_as_clothing() {
        for (inventory, slot) in [
            (4, 0),
            (5, 1),
            (20, 1),
            (6, 2),
            (7, 3),
            (8, 4),
            (9, 5),
            (10, 6),
            (19, 7),
        ] {
            assert_eq!(body_slot(inventory), Some(slot));
        }
        for inventory in [1, 3, 12, 13, 14, 15, 17, 18, 26] {
            assert_eq!(body_slot(inventory), None);
        }
    }
}
