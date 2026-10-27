//! Stable, asset-free mountain silhouettes inspired by the starter intention reference.
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::ui_render::UiMaterialPlugin;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct Backdrop {
    #[uniform(0)]
    opacity: f32,
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
            commands
                .entity(menu)
                .insert(MaterialNode(materials.add(Backdrop { opacity })));
        }
    }
}
