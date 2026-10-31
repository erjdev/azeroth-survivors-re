//! Collection attachment asset coverage; mirrors equipment::ensure_item_model's file selection.
//! This checks source geometry/materials, not native attachment placement or animation.
use anyhow::{bail, Context, Result};
use benilla_formats::{Chain, CharSkinSlot, ItemDisplay, RenderSubmesh};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
struct Attachment {
    model: String,
    texture: Option<String>,
}

fn attachments(display: &ItemDisplay, inventory: u8) -> Vec<Attachment> {
    let (folder, columns): (&str, &[usize]) = match inventory {
        1 => ("Head", &[0]),
        3 => ("Shoulder", &[0, 1]),
        14 => ("Shield", &[0]),
        13 | 15 | 17 | 21..=23 | 25 | 26 => ("Weapon", &[0]),
        _ => return Vec::new(),
    };
    let dir = format!("Item\\ObjectComponents\\{folder}");
    let mut result = Vec::new();
    for &column in columns {
        // Model-less equipment (and single-sided shoulders) is valid in the renderer.
        let Some(model) = &display.model[column] else {
            continue;
        };
        let texture = display.model_texture[column]
            .as_ref()
            .map(|texture| format!("{dir}\\{texture}.blp"));
        if inventory == 1 {
            for race in ["Hu", "Or", "Dw", "Ni", "Sc", "Ta", "Gn", "Tr"] {
                for sex in ['M', 'F'] {
                    let stem = model.strip_suffix(".m2").unwrap_or(model);
                    result.push(Attachment {
                        model: format!("{dir}\\{stem}_{race}{sex}.m2"),
                        texture: texture.clone(),
                    });
                }
            }
        } else {
            result.push(Attachment {
                model: format!("{dir}\\{model}"),
                texture,
            });
        }
    }
    result
}

struct ModelArt {
    textures: Vec<String>,
    object: bool,
}

impl ModelArt {
    fn from_meshes(meshes: Vec<RenderSubmesh>) -> Result<Self> {
        if !meshes.iter().any(|mesh| !mesh.indices.is_empty()) {
            bail!("no render triangles");
        }
        Ok(Self {
            object: meshes
                .iter()
                .any(|mesh| mesh.char_slot == Some(CharSkinSlot::Object)),
            textures: meshes.into_iter().filter_map(|mesh| mesh.texture).collect(),
        })
    }

    fn selected_textures<'a>(&'a self, attachment: &'a Attachment) -> Result<Vec<&'a str>> {
        if self.object && attachment.texture.is_none() {
            bail!("unbound object skin: {}", attachment.model);
        }
        Ok(self
            .textures
            .iter()
            .chain(attachment.texture.iter().filter(|_| self.object))
            .map(String::as_str)
            .collect())
    }
}

#[derive(Default)]
pub(super) struct AttachmentArt {
    checked: HashSet<(u32, u8)>,
    // Each model can have different display-selected object skins.
    models: HashMap<String, ModelArt>,
    textures: HashSet<String>,
    variants: usize,
}

impl AttachmentArt {
    pub fn check(
        &mut self,
        chain: &mut Chain,
        id: u32,
        inventory: u8,
        display: &ItemDisplay,
    ) -> Result<()> {
        if self.checked.contains(&(id, inventory)) {
            return Ok(());
        }
        let plan = attachments(display, inventory);
        for attachment in &plan {
            if !self.models.contains_key(&attachment.model) {
                let meshes = benilla_formats::load_object_model(chain, &attachment.model)
                    .with_context(|| attachment.model.clone())?;
                let art =
                    ModelArt::from_meshes(meshes).with_context(|| attachment.model.clone())?;
                self.models.insert(attachment.model.clone(), art);
            }
            for path in self.models[&attachment.model].selected_textures(attachment)? {
                if !self.textures.contains(path) {
                    let image = benilla_blp::decode(&chain.read_file(path)?)
                        .with_context(|| format!("{} texture {path}", attachment.model))?;
                    if image.width == 0 || image.height == 0 {
                        bail!("empty attachment texture: {path}");
                    }
                    self.textures.insert(path.to_owned());
                }
            }
        }
        self.variants += plan.len();
        self.checked.insert((id, inventory));
        Ok(())
    }

    pub fn report(&self) {
        println!("ATTACHMENT ART: {} display/inventory model variants; {} unique models with render triangles; {} selected textures decoded. Helmets cover all eight races and both bodies. Model-less rows need no attachment. Excludes placement, animation, particles and native appearance.", self.variants, self.models.len(), self.textures.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_and_required_object_skins_cannot_silently_pass() {
        assert!(ModelArt::from_meshes(vec![]).is_err());
        assert!(ModelArt::from_meshes(vec![RenderSubmesh::default()]).is_err());
        let mesh = RenderSubmesh {
            indices: vec![0, 1, 2],
            texture: Some("Fixed.blp".into()),
            char_slot: Some(CharSkinSlot::Object),
            ..Default::default()
        };
        let art = ModelArt::from_meshes(vec![mesh]).unwrap();
        let mut attachment = Attachment {
            model: "Shared.m2".into(),
            texture: None,
        };
        assert!(art.selected_textures(&attachment).is_err());
        for skin in ["Red.blp", "Blue.blp"] {
            attachment.texture = Some(skin.into());
            assert_eq!(
                art.selected_textures(&attachment).unwrap(),
                ["Fixed.blp", skin]
            );
        }
        let art = ModelArt {
            object: false,
            ..art
        };
        assert_eq!(art.selected_textures(&attachment).unwrap(), ["Fixed.blp"]);
    }

    #[test]
    fn helmets_cover_every_body_without_deriving_skin_from_mesh_name() {
        let display = ItemDisplay {
            model: [Some("Helm.m2".into()), Some("unused.m2".into())],
            model_texture: [Some("Gold".into()), None],
            ..Default::default()
        };
        let plan = attachments(&display, 1);
        assert_eq!(plan.len(), 16);
        assert!(plan[0].model.ends_with("Helm_HuM.m2"));
        assert!(plan[15].model.ends_with("Helm_TrF.m2"));
        assert!(plan
            .iter()
            .all(|a| a.texture.as_deref() == Some("Item\\ObjectComponents\\Head\\Gold.blp")));
    }

    #[test]
    fn shoulder_sides_and_inventory_directories_remain_independent() {
        let display = ItemDisplay {
            model: [None, Some("Right.m2".into())],
            model_texture: [None, Some("Red".into())],
            ..Default::default()
        };
        assert_eq!(
            attachments(&display, 3),
            vec![Attachment {
                model: "Item\\ObjectComponents\\Shoulder\\Right.m2".into(),
                texture: Some("Item\\ObjectComponents\\Shoulder\\Red.blp".into()),
            }]
        );
        assert!(attachments(&display, 1).is_empty());
        let display = ItemDisplay {
            model: [Some("Shared.m2".into()), None],
            ..Default::default()
        };
        assert!(attachments(&display, 14)[0].model.contains("\\Shield\\"));
        assert!(attachments(&display, 17)[0].model.contains("\\Weapon\\"));
        for inventory in [4, 5, 6, 7, 8, 9, 10, 19, 20] {
            assert!(attachments(&display, inventory).is_empty());
        }
    }
}
