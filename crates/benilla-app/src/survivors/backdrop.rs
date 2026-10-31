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

#[allow(clippy::type_complexity)]
fn attach(
    mut commands: Commands,
    mut materials: ResMut<Assets<Backdrop>>,
    mut images: ResMut<Assets<Image>>,
    mut landscape: Local<Option<Handle<Image>>>,
    phase: Res<State<super::Phase>>,
    looks: Query<&Node, With<super::menu::LookScreen>>,
    menus: Query<
        (
            Entity,
            Option<&MaterialNode<Backdrop>>,
            Has<super::hud::PauseRoot>,
        ),
        Or<(With<super::hud::MenuRoot>, With<super::hud::PauseRoot>)>,
    >,
) {
    let opacity = if looks.iter().any(|node| node.display != Display::None) {
        0.0
    } else {
        1.0
    };
    for (menu, material, options) in &menus {
        // The shared pause root shows scenery only when entered from the starter.
        if options && *phase.get() != super::Phase::Menu {
            if material.is_some() {
                commands.entity(menu).remove::<MaterialNode<Backdrop>>();
            }
            continue;
        }
        let opacity = if options { 1.0 } else { opacity };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_options_keep_landscape_but_game_pause_keeps_world() {
        let mut app = App::new();
        app.init_resource::<Assets<Backdrop>>()
            .init_resource::<Assets<Image>>()
            .insert_resource(State::new(super::super::Phase::Menu))
            .add_systems(Update, attach);
        let menu = app
            .world_mut()
            .spawn((super::super::hud::MenuRoot, Node::default()))
            .id();
        let tint = BackgroundColor(Color::srgba(0., 0., 0., 0.55));
        let pause = app
            .world_mut()
            .spawn((super::super::hud::PauseRoot, Node::default(), tint))
            .id();
        let look = app
            .world_mut()
            .spawn((super::super::menu::LookScreen, Node::default()))
            .id();
        app.update();
        let menu_handle = app
            .world()
            .get::<MaterialNode<Backdrop>>(menu)
            .unwrap()
            .0
            .clone();
        let pause_handle = app
            .world()
            .get::<MaterialNode<Backdrop>>(pause)
            .unwrap()
            .0
            .clone();
        let materials = app.world().resource::<Assets<Backdrop>>();
        assert_eq!(materials.get(&menu_handle).unwrap().opacity, 0.);
        assert_eq!(materials.get(&pause_handle).unwrap().opacity, 1.);
        assert_eq!(
            materials.get(&menu_handle).unwrap().landscape,
            materials.get(&pause_handle).unwrap().landscape
        );
        app.world_mut().get_mut::<Node>(look).unwrap().display = Display::None;
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Backdrop>>()
                .get(&menu_handle)
                .unwrap()
                .opacity,
            1.
        );
        for phase in [super::super::Phase::Playing, super::super::Phase::Paused] {
            app.insert_resource(State::new(phase));
            app.update();
            assert!(app.world().get::<MaterialNode<Backdrop>>(pause).is_none());
            assert_eq!(*app.world().get::<BackgroundColor>(pause).unwrap(), tint);
        }
        app.insert_resource(State::new(super::super::Phase::Menu));
        app.update();
        assert!(app.world().get::<MaterialNode<Backdrop>>(pause).is_some());
    }
}
