//! Menu-only model specimens. No summons, enemy AI, or combat-pet components.
use super::{hero::Hero, units, Phase};
use benilla_world::view::WorldCamera;
use bevy::prelude::*;

#[derive(Component)]
struct Specimen;

#[derive(Resource)]
struct Specimens {
    displays: Vec<u32>,
    label: String,
}

pub(super) fn plugin(app: &mut App) {
    if !crate::run_mode::dev_affordances() {
        return;
    }
    let specimens = if let Ok(ids) = std::env::var("WOW_SURVIVORS_MODEL_PREVIEW") {
        let displays: Result<Vec<u32>, _> = ids.split(',').map(|id| id.trim().parse()).collect();
        let Ok(displays) = displays else {
            warn!("model preview: expected comma-separated creature display IDs");
            return;
        };
        if displays.is_empty() || displays.len() > 8 || displays.contains(&0) {
            warn!("model preview: expected one to eight nonzero creature display IDs");
            return;
        }
        Some(Specimens {
            label: format!("Model specimens · no combat\nCreature display IDs: {ids}"),
            displays,
        })
    } else if std::env::var("WOW_SURVIVORS_DEMON_PREVIEW").as_deref() == Ok("1") {
        Some(Specimens {
            displays: vec![4449, 1132, 4162, 850],
            label: "Combat demon specimens · no combat\nLeft to right: Imp · Voidwalker · Succubus · Felhunter".into(),
        })
    } else {
        None
    };
    if let Some(specimens) = specimens {
        app.insert_resource(specimens);
        app.add_systems(
            PostUpdate,
            (show, report)
                .before(bevy::transform::TransformSystems::Propagate)
                .run_if(in_state(Phase::Menu)),
        );
    }
}

#[allow(clippy::type_complexity)]
fn show(
    mut commands: Commands,
    mut run: ResMut<super::Run>,
    font: Res<super::hud::HudFont>,
    floor: super::Floor,
    mut hero: Query<(&Transform, &mut Visibility), (With<Hero>, Without<WorldCamera>)>,
    mut cameras: Query<&mut Transform, (With<WorldCamera>, Without<Hero>)>,
    mut roots: Query<&mut Node, Or<(With<super::hud::MenuRoot>, With<super::armory::ArmoryRoot>)>>,
    actors: Query<(), With<Specimen>>,
    specimens: Res<Specimens>,
) {
    let Ok((hero, mut visibility)) = hero.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    for mut node in &mut roots {
        node.display = Display::None;
    }
    // Stay clear of independently visible attachments on the menu hero.
    let mut center = hero.translation + Vec3::X * 20.;
    let Some(height) = floor.height(center) else {
        return;
    };
    center.y = height;
    let target = center + Vec3::Y * 1.5;
    for mut camera in &mut cameras {
        let distance = (specimens.displays.len() as f32 * 3.25).max(5.0);
        *camera = Transform::from_translation(target + Vec3::new(0., 3., distance))
            .looking_at(target, Vec3::Y);
    }
    if !actors.is_empty() {
        return;
    }
    // Default demon identities: docs/audits/demon-identities.csv.
    let displays = &specimens.displays;
    let mut positions = Vec::new();
    for index in 0..displays.len() {
        let mut pos = center + Vec3::X * (index as f32 - (displays.len() - 1) as f32 * 0.5) * 3.;
        let Some(height) = floor.height(pos) else {
            return;
        };
        pos.y = height;
        positions.push(pos);
    }
    for (&display, pos) in displays.iter().zip(positions) {
        let entity = units::spawn_unit(
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
            pos,
            units::yaw_toward(Vec3::Z),
        );
        commands.entity(entity).insert(Specimen);
    }
    commands.spawn((
        Text::new(specimens.label.clone()),
        TextFont {
            font: font.0.clone(),
            font_size: 24.,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(24.),
            left: Val::Px(24.),
            padding: UiRect::all(Val::Px(12.)),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(1000),
    ));
}

/// Log the settled native state once, so a screenshot can be checked against the
/// actual animation slot and material visibility rather than guessed from pixels.
fn report(
    time: Res<Time>,
    mut began: Local<Option<f32>>,
    mut reported: Local<bool>,
    specimens: Query<
        (
            Entity,
            &Transform,
            &AnimationPlayer,
            &benilla_assets::ModelAnimations,
        ),
        With<Specimen>,
    >,
    children: Query<&Children>,
    parts: Query<(
        &benilla_world::doodad_anim::MatAnim,
        &Visibility,
        &InheritedVisibility,
    )>,
) {
    if *reported || specimens.is_empty() {
        return;
    }
    let start = began.get_or_insert(time.elapsed_secs());
    if time.elapsed_secs() - *start < 5.0 {
        return;
    }
    for (entity, transform, player, animations) in &specimens {
        let playing: Vec<_> = player
            .playing_animations()
            .filter_map(|(node, active)| {
                animations
                    .clips
                    .iter()
                    .find(|clip| clip.node == *node)
                    .map(|clip| {
                        (
                            clip.anim_id,
                            clip.seq_index,
                            active.seek_time(),
                            active.weight(),
                        )
                    })
            })
            .collect();
        info!(?entity, position = ?transform.translation, ?playing, "model preview: specimen");
        for child in children.iter_descendants(entity) {
            if let Ok((alpha, visibility, inherited)) = parts.get(child) {
                info!(
                    ?child,
                    alpha = alpha.current,
                    ?visibility,
                    visible = inherited.get(),
                    "model preview: material"
                );
            }
        }
    }
    *reported = true;
}
