//! Opt-in menu-only render fixture. Actors have no Enemy/AI/combat-pet components.
use super::{FixtureSide, Mask};
use crate::survivors::{hero::Hero, units, Phase};
use benilla_world::view::WorldCamera;
use bevy::{asset::RenderAssetUsages, image::ImageSampler, prelude::*, render::render_resource::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Case {
    #[default]
    Clear,
    Front,
    Partial,
    Behind,
    Foliage,
    FrontOff,
    Concealed,
    Enemies,
    Friendlies,
    Neither,
}
impl Case {
    fn parse(text: &str) -> Option<Self> {
        Some(match text.trim() {
            "clear" => Self::Clear,
            "front" => Self::Front,
            "partial" => Self::Partial,
            "behind" => Self::Behind,
            "foliage" => Self::Foliage,
            "front-off" => Self::FrontOff,
            "concealed" => Self::Concealed,
            "enemies" => Self::Enemies,
            "friendlies" => Self::Friendlies,
            "neither" => Self::Neither,
            _ => return None,
        })
    }
}
#[derive(Resource)]
struct Fixture {
    path: std::path::PathBuf,
    poll: Timer,
    case: Case,
    center: Option<Vec3>,
    solid: Handle<StandardMaterial>,
    cutout: Handle<StandardMaterial>,
}
#[derive(Component)]
struct Barrier;
#[derive(Component)]
struct Caption;

pub(super) fn plugin(app: &mut App) -> bool {
    if !crate::run_mode::dev_affordances() {
        return false;
    }
    let Some(path) = std::env::var_os("WOW_SURVIVORS_SILHOUETTE_FIXTURE") else {
        return false;
    };
    app.insert_resource(Fixture {
        path: path.into(),
        poll: Timer::from_seconds(0.25, TimerMode::Repeating),
        case: Case::Clear,
        center: None,
        solid: default(),
        cutout: default(),
    })
    .add_systems(
        PostUpdate,
        show.before(bevy::transform::TransformSystems::Propagate)
            .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate)
            .run_if(in_state(Phase::Menu)),
    );
    app.add_systems(
        PostUpdate,
        (
            conceal
                .after(benilla_world::exterior_cull::ExteriorCullSet)
                .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
            audit.after(super::sync),
        ),
    );
    true
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut mask: ResMut<Mask>,
    mut settings: ResMut<crate::survivors::camera::CameraSettings>,
    mut run: ResMut<crate::survivors::Run>,
    time: Res<Time>,
    font: Res<crate::survivors::hud::HudFont>,
    mut hero: Query<
        (&mut Transform, &mut Visibility),
        (With<Hero>, Without<WorldCamera>, Without<Barrier>),
    >,
    mut cameras: Query<&mut Transform, (With<WorldCamera>, Without<Hero>, Without<Barrier>)>,
    mut roots: Query<
        &mut Node,
        Or<(
            With<crate::survivors::hud::MenuRoot>,
            With<crate::survivors::armory::ArmoryRoot>,
        )>,
    >,
    mut barrier: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<Barrier>,
            Without<Hero>,
            Without<WorldCamera>,
            Without<FixtureSide>,
        ),
    >,
    mut caption: Query<&mut Text, With<Caption>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((mut hero, mut hero_visibility)) = hero.single_mut() else {
        return;
    };
    if fixture.poll.tick(time.delta()).just_finished() {
        if let Ok(text) = std::fs::read_to_string(&fixture.path) {
            if let Some(case) = Case::parse(&text) {
                if fixture.case != case {
                    info!(?case, "silhouette fixture changed");
                }
                fixture.case = case;
            }
        }
    }
    let case = fixture.case;
    mask.enabled = case != Case::FrontOff;
    settings.silhouette_enemies =
        !matches!(case, Case::Friendlies | Case::Neither | Case::FrontOff);
    settings.silhouette_friendlies =
        !matches!(case, Case::Enemies | Case::Neither | Case::FrontOff);
    for mut root in &mut roots {
        root.display = Display::None;
    }
    let center = *fixture.center.get_or_insert(Vec3::new(
        hero.translation.x + 20.,
        10_000.,
        hero.translation.z,
    ));
    hero.translation = center;
    hero.rotation = Quat::from_rotation_y(units::yaw_toward(Vec3::Z));
    *hero_visibility = Visibility::Visible;
    let target = center + Vec3::Y * 1.5;
    for mut camera in &mut cameras {
        *camera = Transform::from_translation(target + Vec3::new(0., 0.5, 13.))
            .looking_at(target, Vec3::Y);
    }
    let title = format!("Silhouette render fixture · {case:?} · no combat\nLeft: enemy specimen · center: saved hero · right: neutral specimen");
    for mut text in &mut caption {
        text.0.clone_from(&title);
    }
    if let Ok((mut transform, mut visibility, mut material)) = barrier.single_mut() {
        *visibility = if case == Case::Clear {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        transform.translation = target
            + Vec3::new(
                if case == Case::Partial { -4.5 } else { 0. },
                0.,
                if case == Case::Behind { -3. } else { 3. },
            );
        transform.scale = Vec3::new(if case == Case::Partial { 9. } else { 18. }, 7., 1.);
        material.0 = if case == Case::Foliage {
            fixture.cutout.clone()
        } else {
            fixture.solid.clone()
        };
        return;
    }
    // Authored alpha-key obstruction isolates depth through transparent texels from collision.
    let mut pixels = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32 {
        for x in 0..32 {
            pixels.extend([40, 110, 45, if (x / 4 + y / 4) % 2 == 0 { 255 } else { 0 }]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    fixture.solid = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.25, 0.30),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    fixture.cutout = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(image)),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Barrier,
        Mesh3d(meshes.add(Rectangle::default())),
        MeshMaterial3d(fixture.solid.clone()),
        Transform::from_translation(target + Vec3::Z * 3.),
        Visibility::Hidden,
    ));
    for (side, display, x) in [(1, 1132, -3.5), (0, 4162, 3.5)] {
        let actor = units::spawn_unit(
            &mut commands,
            &units::UnitSpec {
                serial: run.guid(),
                display,
                scale: 1.,
                level: 1,
                faction: units::FACTION_FRIENDLY,
                bytes0: 1 << 8,
                weapons: &[],
                sheath: 0,
            },
            center + Vec3::X * x,
            units::yaw_toward(Vec3::Z),
        );
        commands.entity(actor).insert(FixtureSide(side));
    }
    commands.spawn((
        Caption,
        Text::new(title),
        TextFont {
            font: font.0.clone(),
            font_size: 22.,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12.),
            left: px(12.),
            padding: UiRect::all(px(10.)),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(1000),
    ));
    info!(?center, "silhouette noncombat fixture initialized");
}

fn conceal(fixture: Res<Fixture>, mut actors: Query<(&FixtureSide, &mut Visibility)>) {
    if fixture.case == Case::Concealed {
        for (side, mut visible) in &mut actors {
            if side.0 == 1 {
                *visible = Visibility::Hidden;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn audit(
    fixture: Res<Fixture>,
    actors: Query<(&FixtureSide, &Visibility, &InheritedVisibility)>,
    mut previous: Local<Option<(Case, Vec<(u8, Visibility, bool)>)>>,
) {
    let mut states: Vec<_> = actors
        .iter()
        .map(|(side, visible, inherited)| (side.0, *visible, inherited.get()))
        .collect();
    states.sort_by_key(|state| state.0);
    let current = (fixture.case, states);
    if previous.as_ref() != Some(&current) {
        info!(?current, "silhouette fixture visibility after mask sync");
        *previous = Some(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_or_incomplete_controls_never_change_the_render_case() {
        for text in ["", "front off", "front\nclear", "playing", "start"] {
            assert!(Case::parse(text).is_none());
        }
        assert_eq!(Case::parse(" front-off\n"), Some(Case::FrontOff));
        assert_eq!(Case::parse("concealed"), Some(Case::Concealed));
    }
}
