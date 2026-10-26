//! Resolution-independent Options sliders; graphics retain their supported stops.
use super::*;
use bevy::input::ButtonState;
use bevy::window::WindowEvent;

#[derive(Component)]
pub(in crate::survivors) struct Slider(Knob);
#[derive(Component)]
pub(in crate::survivors) struct Fill(Knob);
#[derive(Component)]
pub(in crate::survivors) struct Value(Knob);

struct Spec {
    label: &'static str,
    name: &'static str,
    steps: &'static [f32],
}
fn spec(knob: Knob) -> Spec {
    let (label, name, steps): (_, _, &'static [f32]) = match knob {
        Knob::Master => ("Master", "MasterVolume", &[]),
        Knob::Effects => ("Effects", "SoundVolume", &[]),
        Knob::Music => ("Music", "MusicVolume", &[]),
        Knob::Ambience => ("Ambience", "AmbienceVolume", &[]),
        Knob::View => (
            "View distance",
            "farclip",
            &[177., 250., 350., 500., 650., 777.],
        ),
        Knob::Grass => ("Grass", "WorldDetail", &[0., 1., 2.]),
        Knob::Spells => ("Spell detail", "spellEffectLevel", &[0., 1., 2.]),
        Knob::Scale => (
            "Render scale",
            "renderScale",
            &[0.5, 0.6, 0.75, 1., 1.25, 1.5, 2.],
        ),
        _ => unreachable!("toggles do not use sliders"),
    };
    Spec { label, name, steps }
}
fn category(knob: Knob) -> Category {
    match knob {
        Knob::Master | Knob::Effects | Knob::Music | Knob::Ambience => Category::Audio,
        _ => Category::Graphics,
    }
}
impl Spec {
    fn value(&self, position: f32) -> f32 {
        let position = position.clamp(0., 1.);
        if self.steps.is_empty() {
            (position * 100.).round() / 100.
        } else {
            self.steps[(position * (self.steps.len() - 1) as f32).round() as usize]
        }
    }
    fn position(&self, value: f32) -> f32 {
        if self.steps.is_empty() {
            value.clamp(0., 1.)
        } else {
            let index = self
                .steps
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
                .map_or(0, |(index, _)| index);
            index as f32 / (self.steps.len() - 1) as f32
        }
    }
}

pub(in crate::survivors::hud) fn apply(cvars: &mut crate::cvars::Cvars, knob: Knob, position: f32) {
    if position.is_finite() {
        let spec = spec(knob);
        cvars.set(spec.name, &spec.value(position).to_string());
    }
}

pub(super) fn row(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, knob: Knob) {
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                text(font, spec(knob).label, 14., GOLD),
                Node {
                    width: Val::Percent(34.),
                    min_width: Val::Px(0.),
                    ..default()
                },
            ));
            row.spawn((
                Slider(knob),
                Button,
                Node {
                    flex_grow: 1.,
                    flex_basis: Val::Px(0.),
                    min_width: Val::Px(64.),
                    height: Val::Px(28.),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|hit| {
                hit.spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(8.),
                        border: UiRect::all(Val::Px(1.)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    BorderColor::all(EDGE),
                ))
                .with_children(|track| {
                    track.spawn((
                        Fill(knob),
                        Node {
                            width: Val::Percent(0.),
                            height: Val::Percent(100.),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.68, 0.52, 0.18)),
                    ));
                });
            });
            row.spawn((
                Value(knob),
                text(font, "", 14., Color::WHITE),
                Node {
                    width: Val::Px(58.),
                    flex_shrink: 0.,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
        });
}

#[derive(Default)]
pub(in crate::survivors) struct Drag {
    point: Option<Vec2>,
    active: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::survivors) fn input(
    page: Res<PausePage>,
    phase: Res<State<Phase>>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    sliders: Query<(Entity, &Slider, &ComputedNode, &UiGlobalTransform)>,
    mut events: MessageReader<WindowEvent>,
    mut drag: Local<Drag>,
    mut edits: ResMut<SettingEdits>,
) {
    if !page.open || !matches!(phase.get(), Phase::Menu | Phase::Paused) || loading.covering() {
        events.clear();
        *drag = Drag::default();
        return;
    }
    let Ok((window_entity, window)) = windows.single() else {
        events.clear();
        *drag = Drag::default();
        return;
    };
    if drag.active.is_some_and(|entity| {
        sliders.get(entity).map_or(true, |(_, slider, _, _)| {
            category(slider.0) != page.category
        })
    }) {
        drag.active = None;
    }
    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(event) if event.window == window_entity => {
                drag.point = Some(event.position * window.scale_factor());
            }
            WindowEvent::MouseButtonInput(event)
                if event.window == window_entity && event.button == MouseButton::Left =>
            {
                if event.state == ButtonState::Pressed {
                    // A press can arrive without a cursor move in this frame.
                    let point = drag.point.or_else(|| window.physical_cursor_position());
                    drag.point = point;
                    drag.active = point.and_then(|point| {
                        sliders
                            .iter()
                            .find(|(_, slider, node, transform)| {
                                category(slider.0) == page.category
                                    && node.contains_point(**transform, point)
                            })
                            .map(|(entity, _, _, _)| entity)
                    });
                } else {
                    drag.active = None;
                }
            }
            WindowEvent::WindowFocused(event)
                if event.window == window_entity && !event.focused =>
            {
                *drag = Drag::default();
            }
            WindowEvent::CursorLeft(event) if event.window == window_entity => {
                *drag = Drag::default();
            }
            _ => continue,
        }
        if let (Some(entity), Some(point)) = (drag.active, drag.point) {
            if let Ok((_, slider, node, transform)) = sliders.get(entity) {
                if let Some(normalized) = node.normalize_point(*transform, point) {
                    edits.set.push((slider.0, normalized.x + 0.5));
                }
            }
        }
    }
}

pub(in crate::survivors) fn refresh(
    cvars: Res<crate::cvars::Cvars>,
    mut fills: Query<(&Fill, &mut Node)>,
    mut values: Query<(&Value, &mut Text)>,
) {
    for (fill, mut node) in &mut fills {
        let spec = spec(fill.0);
        let value = cvars.num(spec.name).unwrap_or(0.);
        node.width = Val::Percent(spec.position(value) * 100.);
    }
    for (value, mut text) in &mut values {
        let spec = spec(value.0);
        let number = cvars.num(spec.name).unwrap_or(0.);
        let label = match value.0 {
            Knob::Grass | Knob::Spells => match number.round() as i32 {
                0 => "Low",
                1 => "Medium",
                _ => "High",
            }
            .to_owned(),
            Knob::View => format!("{number:.0}"),
            _ => format!("{:.0}%", number * 100.),
        };
        if **text != label {
            **text = label;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_drag_preserves_order_at_both_display_scales_and_stops_on_release() {
        use bevy::ecs::system::RunSystemOnce;
        use bevy::input::mouse::MouseButtonInput;
        use bevy::math::Affine2;
        use bevy::window::{CursorMoved, WindowResolution};
        for scale in [1., 2.] {
            let mut world = World::new();
            world.insert_resource(PausePage {
                open: true,
                ..default()
            });
            world.insert_resource(State::new(Phase::Menu));
            world.init_resource::<crate::loading_screen::LoadingScreen>();
            world.init_resource::<SettingEdits>();
            world.init_resource::<Messages<WindowEvent>>();
            let window = world
                .spawn((
                    Window {
                        resolution: WindowResolution::new(640, 720)
                            .with_scale_factor_override(scale),
                        ..default()
                    },
                    PrimaryWindow,
                ))
                .id();
            world.spawn((
                Slider(Knob::Master),
                ComputedNode {
                    size: Vec2::new(200., 28.),
                    ..default()
                },
                UiGlobalTransform::from(Affine2::from_translation(Vec2::new(200., 100.))),
            ));
            let moved = |x| {
                WindowEvent::CursorMoved(CursorMoved {
                    window,
                    position: Vec2::new(x, 100.) / scale,
                    delta: None,
                })
            };
            let button = |state| {
                WindowEvent::MouseButtonInput(MouseButtonInput {
                    window,
                    button: MouseButton::Left,
                    state,
                })
            };
            for event in [
                moved(150.),
                button(ButtonState::Pressed),
                moved(280.),
                button(ButtonState::Released),
                moved(400.),
            ] {
                world.resource_mut::<Messages<WindowEvent>>().write(event);
            }
            world.run_system_once(input).unwrap();
            let edits = world.resource::<SettingEdits>();
            assert_eq!(edits.set.len(), 2);
            assert!((edits.set[0].1 - 0.25).abs() < 0.001);
            assert!((edits.set[1].1 - 0.9).abs() < 0.001);
        }
    }

    #[test]
    fn sliders_clamp_and_snap_without_losing_supported_graphics_levels() {
        let mut cvars = crate::cvars::Cvars::default();
        apply(&mut cvars, Knob::Master, -0.5);
        assert_eq!(cvars.num("MasterVolume"), Some(0.));
        apply(&mut cvars, Knob::Master, 1.5);
        assert_eq!(cvars.num("MasterVolume"), Some(1.));
        apply(&mut cvars, Knob::Master, 0.373);
        assert_eq!(cvars.num("MasterVolume"), Some(0.37));
        apply(&mut cvars, Knob::Master, f32::NAN);
        assert_eq!(cvars.num("MasterVolume"), Some(0.37));
        for knob in [Knob::View, Knob::Grass, Knob::Spells, Knob::Scale] {
            let spec = spec(knob);
            for value in spec.steps {
                apply(&mut cvars, knob, spec.position(*value));
                assert_eq!(cvars.num(spec.name), Some(*value));
            }
        }
    }
}
