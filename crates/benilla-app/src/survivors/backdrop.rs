//! Original embedded landscape inspired by the starter intention reference.
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::ui_render::UiMaterialPlugin;
use bevy::{asset::RenderAssetUsages, image::ImageSampler};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct Backdrop {
    #[uniform(0)]
    opacity: f32,
    #[texture(1)]
    #[sampler(2)]
    landscape: Handle<Image>,
}

impl UiMaterial for Backdrop {
    fn fragment_shader() -> ShaderRef {
        "embedded://benilla_app/survivors/backdrop/mountains.wgsl".into()
    }
}

pub(super) fn plugin(app: &mut App) {
    bevy::asset::embedded_asset!(app, "backdrop/mountains.wgsl");
    app.add_plugins(UiMaterialPlugin::<Backdrop>::default())
        .add_systems(Update, attach);
}

fn attach(
    mut commands: Commands,
    mut materials: ResMut<Assets<Backdrop>>,
    mut images: ResMut<Assets<Image>>,
    mut landscape: Local<Option<Handle<Image>>>,
    looks: Query<&Node, With<super::menu::LookScreen>>,
    menus: Query<(Entity, Option<&MaterialNode<Backdrop>>), With<super::hud::MenuRoot>>,
) {
    let opacity = if looks.iter().any(|node| node.display != Display::None) {
        0.0
    } else {
        1.0
    };
    for (menu, material) in &menus {
        if let Some(material) = material {
            if materials
                .get(&material.0)
                .is_some_and(|backdrop| backdrop.opacity != opacity)
            {
                // Do not mark the asset changed each frame while the menu is idle.
                if let Some(backdrop) = materials.get_mut(&material.0) {
                    backdrop.opacity = opacity;
                }
            }
        } else {
            let landscape = landscape
                .get_or_insert_with(|| {
                    let pixels =
                        image::load_from_memory(include_bytes!("backdrop/sunset-valley-v1.png"))
                            .expect("valid embedded starter landscape");
                    let mut image =
                        Image::from_dynamic(pixels, true, RenderAssetUsages::RENDER_WORLD);
                    image.sampler = ImageSampler::linear();
                    images.add(image)
                })
                .clone();
            commands
                .entity(menu)
                .insert(MaterialNode(materials.add(Backdrop { opacity, landscape })));
        }
    }
}
