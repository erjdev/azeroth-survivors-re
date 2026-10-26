//! Reference-style switches with a full labeled hit area and a geometric checkmark.
use super::*;

#[derive(Component)]
pub(in crate::survivors) struct Check(Knob);

pub(super) fn row(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, knob: Knob) {
    let label = match knob {
        Knob::Sound => "Enable sound",
        Knob::MusicOn => "Enable music",
        Knob::AutoPush => "Auto-push camera",
        Knob::Cutaway => "Scenery cutaway",
        _ => unreachable!("checkbox requires a boolean preference"),
    };
    ui.spawn((
        button(UiAction::Toggle(knob), Color::NONE),
        Node {
            width: Val::Percent(100.),
            min_height: Val::Px(28.),
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.),
            border_radius: BorderRadius::all(Val::Px(3.)),
            ..default()
        },
    ))
    .with_children(|row| {
        row.spawn((
            text(font, label, 14., GOLD),
            Node {
                width: Val::Percent(34.),
                min_width: Val::Px(0.),
                ..default()
            },
        ));
        row.spawn((
            Node {
                width: Val::Px(18.),
                height: Val::Px(18.),
                flex_shrink: 0.,
                border: UiRect::all(Val::Px(1.)),
                border_radius: BorderRadius::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            BorderColor::all(GOLD),
        ))
        .with_children(|square| {
            square
                .spawn((
                    Check(knob),
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|mark| {
                    // Two strokes avoid a font-dependent check glyph or external texture.
                    for (x, y, width, angle) in [(2., 8., 6., 45.), (5., 6., 10., -45.)] {
                        mark.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(x),
                                top: Val::Px(y),
                                width: Val::Px(width),
                                height: Val::Px(2.),
                                ..default()
                            },
                            UiTransform {
                                rotation: Rot2::degrees(angle),
                                ..default()
                            },
                            BackgroundColor(GOLD),
                        ));
                    }
                });
        });
    });
}

pub(in crate::survivors) fn refresh(
    cvars: Res<crate::cvars::Cvars>,
    camera: Res<super::super::super::camera::CameraSettings>,
    mut checks: Query<(&Check, &mut Visibility)>,
) {
    for (check, mut visibility) in &mut checks {
        let enabled = match check.0 {
            Knob::Sound => cvars.num("MasterSoundEffects").unwrap_or(1.) != 0.,
            Knob::MusicOn => cvars.num("EnableMusic").unwrap_or(1.) != 0.,
            Knob::AutoPush => camera.auto_push,
            Knob::Cutaway => camera.cutaway,
            _ => unreachable!("checkbox requires a boolean preference"),
        };
        let want = if enabled {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
}
