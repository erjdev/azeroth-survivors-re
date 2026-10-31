//! Opt-in development specimen; remains in Menu and never spawns combat components.
use super::{data, hero::Hero, units, Phase};
use benilla_world::view::WorldCamera;
use bevy::prelude::*;

#[derive(Resource)]
struct Specimen(&'static data::EnemyDef);
#[derive(Component)]
struct PreviewActor;
#[derive(Component)]
struct PreviewCaption;
#[derive(Resource)]
struct DisarmControl {
    path: std::path::PathBuf,
    poll: Timer,
}

pub(super) fn plugin(app: &mut App) {
    if !crate::run_mode::dev_affordances() {
        return;
    }
    let Some(entry) = std::env::var("WOW_SURVIVORS_EQUIPMENT_PREVIEW")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return;
    };
    let enemy = (0..data::MAPS.len()).find_map(|map| {
        let roster = data::roster(map);
        roster
            .enemies
            .iter()
            .chain(&roster.bosses)
            .find(|enemy| enemy.template_entry == entry)
    });
    let Some(enemy) = enemy else {
        warn!("Unknown equipment preview template {entry}");
        return;
    };
    if let Some(path) = std::env::var_os("WOW_SURVIVORS_EQUIPMENT_STATE") {
        app.insert_resource(DisarmControl {
            path: path.into(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
        });
    }
    app.insert_resource(Specimen(enemy)).add_systems(
        PostUpdate,
        show.before(bevy::transform::TransformSystems::Propagate)
            .run_if(in_state(Phase::Menu)),
    );
}

#[allow(clippy::type_complexity)]
fn show(
    mut commands: Commands,
    specimen: Res<Specimen>,
    mut run: ResMut<super::Run>,
    font: Res<super::hud::HudFont>,
    floor: super::Floor,
    mut hero: Query<(&Transform, &mut Visibility), (With<Hero>, Without<WorldCamera>)>,
    mut cameras: Query<&mut Transform, (With<WorldCamera>, Without<Hero>)>,
    mut roots: Query<&mut Node, Or<(With<super::hud::MenuRoot>, With<super::armory::ArmoryRoot>)>>,
    mut actors: Query<&mut crate::net::ObjectStore, With<PreviewActor>>,
    mut captions: Query<&mut Text, With<PreviewCaption>>,
    control: Option<ResMut<DisarmControl>>,
    time: Res<Time>,
) {
    let Ok((hero, mut visibility)) = hero.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    for mut node in &mut roots {
        node.display = Display::None;
    }
    let def = specimen.0;
    // Stock attachment parts have their own visibility; keep the specimen away
    // from the menu hero instead of depending on hiding its root alone.
    let mut pos = hero.translation + Vec3::X * 20.;
    let Some(height) = floor.height(pos) else {
        return;
    };
    pos.y = height;
    let target = pos + Vec3::Y * 1.3 * def.scale;
    for mut camera in &mut cameras {
        *camera = Transform::from_translation(target + Vec3::new(4., 2., 6.) * def.scale)
            .looking_at(target, Vec3::Y);
    }
    if !actors.is_empty() {
        if let Some(mut control) = control {
            if control.poll.tick(time.delta()).just_finished() {
                if let Ok(text) = std::fs::read_to_string(&control.path) {
                    let active = match text.trim() {
                        "disarm" => Some(true),
                        "restore" => Some(false),
                        _ => None,
                    };
                    if let Some(active) = active {
                        let active = active
                            && survivors_core::disarm::eligibility(def.template_entry)
                                == survivors_core::disarm::Eligibility::Weapon;
                        for store in &mut actors {
                            super::disarm::flag(store, active);
                        }
                        for mut caption in &mut captions {
                            caption.0 = format!(
                                "{} · template {}\nEquipment preview · no combat · {}",
                                def.name,
                                def.template_entry,
                                if active { "disarmed" } else { "armed" }
                            );
                        }
                    }
                }
            }
        }
        return;
    }
    let entity = units::spawn_unit(
        &mut commands,
        &units::UnitSpec {
            serial: run.guid(),
            display: def.display,
            scale: def.scale,
            level: def.level,
            faction: units::FACTION_MONSTER,
            bytes0: 1 << 8,
            weapons: survivors_core::enemy_equipment::for_template(def.template_entry)
                .unwrap_or_default(),
            sheath: 1,
        },
        pos,
        units::yaw_toward(Vec3::new(4., 0., 6.)),
    );
    commands.entity(entity).insert(PreviewActor);
    commands.spawn((
        PreviewCaption,
        Text::new(format!(
            "{} · template {}\nEquipment preview · no combat",
            def.name, def.template_entry
        )),
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
