//! Current input from one connected controller; raw stick events avoid double filtering.
use super::{hud::PausePage, Phase, Run};
use bevy::input::gamepad::RawGamepadEvent;
use bevy::{prelude::*, window::PrimaryWindow};
use std::collections::HashMap;
use survivors_core::controller::{Frame, DEFAULT_DEAD_ZONE};

#[derive(Resource, Default)]
pub(super) struct ControllerInput {
    selected: Option<Entity>,
    axes: HashMap<Entity, [f32; 4]>,
    armed: bool,
    pub frame: Frame,
    pub pause: bool,
}

pub(super) fn sample(
    mut input: ResMut<ControllerInput>,
    mut events: MessageReader<RawGamepadEvent>,
    pads: Query<(Entity, &Gamepad)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    phase: Res<State<Phase>>,
    run: Res<Run>,
    page: Res<PausePage>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    input.axes.retain(|entity, _| pads.contains(*entity));
    for event in events.read() {
        let RawGamepadEvent::Axis(event) = event else {
            continue;
        };
        if !pads.contains(event.gamepad) {
            continue;
        }
        let slot = match event.axis {
            GamepadAxis::LeftStickX => 0,
            GamepadAxis::LeftStickY => 1,
            GamepadAxis::RightStickX => 2,
            GamepadAxis::RightStickY => 3,
            _ => continue,
        };
        input.axes.entry(event.gamepad).or_default()[slot] = event.value;
    }
    if !input.selected.is_some_and(|entity| pads.contains(entity)) {
        input.selected = pads.iter().map(|(entity, _)| entity).min();
        input.armed = false;
    }
    input.frame = Frame::default();
    input.pause = false;
    let Some((entity, pad)) = input.selected.and_then(|entity| pads.get(entity).ok()) else {
        return;
    };
    let active = *phase.get() == Phase::Playing
        && run.dead_t.is_none()
        && !page.open
        && !loading.covering()
        && windows.single().is_ok_and(|window| window.focused);
    input.pause = active && pad.just_pressed(GamepadButton::Start);
    let axes = input.axes.get(&entity).copied().unwrap_or_default();
    // Opposing bumpers/triggers cancel; the two buttons on a side never double speed.
    let inward = pad
        .get(GamepadButton::RightTrigger2)
        .unwrap_or(0.0)
        .max(u8::from(pad.pressed(GamepadButton::RightTrigger)) as f32);
    let outward = pad
        .get(GamepadButton::LeftTrigger2)
        .unwrap_or(0.0)
        .max(u8::from(pad.pressed(GamepadButton::LeftTrigger)) as f32);
    let frame = Frame {
        movement: [axes[0], axes[1]],
        look: [axes[2], axes[3]],
        zoom: inward - outward,
        jump: pad.just_pressed(GamepadButton::South),
    }
    .filtered(DEFAULT_DEAD_ZONE, true);
    if !active || input.pause || keys.just_pressed(KeyCode::Escape) {
        input.armed = false;
    } else if !input.armed
        && frame.movement == [0.0; 2]
        && frame.look == [0.0; 2]
        && frame.zoom == 0.0
        && !pad.pressed(GamepadButton::South)
        && !pad.pressed(GamepadButton::Start)
    {
        input.armed = true;
    }
    if input.armed {
        input.frame = frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::gamepad::{RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent};

    fn fixture() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .init_resource::<ControllerInput>()
            .init_resource::<Run>()
            .init_resource::<PausePage>()
            .init_resource::<crate::loading_screen::LoadingScreen>()
            .insert_resource(State::new(Phase::Playing))
            .add_systems(PreUpdate, sample.after(bevy::input::InputSystems));
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let pad = app.world_mut().spawn(Gamepad::default()).id();
        app.update();
        (app, pad, window)
    }
    fn axis(app: &mut App, pad: Entity, axis: GamepadAxis, value: f32) {
        app.world_mut()
            .write_message(RawGamepadEvent::Axis(RawGamepadAxisChangedEvent::new(
                pad, axis, value,
            )));
    }
    fn button(app: &mut App, pad: Entity, button: GamepadButton, value: f32) {
        app.world_mut()
            .write_message(RawGamepadEvent::Button(RawGamepadButtonChangedEvent::new(
                pad, button, value,
            )));
    }

    #[test]
    fn raw_sticks_filter_once_and_disconnected_devices_cannot_move_the_hero() {
        let (mut app, pad, _) = fixture();
        // Distinct processed-axis calibration proves the sampler reads raw events.
        app.world_mut()
            .get_mut::<GamepadSettings>(pad)
            .unwrap()
            .default_axis_settings =
            bevy::input::gamepad::AxisSettings::new(-1.0, -0.7, 0.7, 1.0, 0.01).unwrap();
        axis(&mut app, pad, GamepadAxis::LeftStickX, 0.625);
        axis(&mut app, pad, GamepadAxis::RightStickY, -0.625);
        app.update();
        let input = app.world().resource::<ControllerInput>();
        assert_eq!(input.frame.movement, [0.5, 0.0]);
        assert_eq!(input.frame.look, [0.0, -0.5]);
        // Bevy suppresses this value under that calibration; our radial filter does not.
        assert_ne!(
            app.world().get::<Gamepad>(pad).unwrap().left_stick().x,
            0.625
        );
        let other = app.world_mut().spawn(Gamepad::default()).id();
        axis(&mut app, other, GamepadAxis::LeftStickX, -1.0);
        app.update();
        assert_eq!(
            app.world().resource::<ControllerInput>().frame.movement,
            [0.5, 0.0]
        );
        app.world_mut().despawn(pad);
        app.update();
        assert_eq!(
            app.world().resource::<ControllerInput>().frame,
            Frame::default()
        );
        axis(&mut app, other, GamepadAxis::LeftStickX, 0.0);
        app.update();
        axis(&mut app, other, GamepadAxis::LeftStickX, -0.625);
        app.update();
        assert_eq!(
            app.world().resource::<ControllerInput>().frame.movement,
            [-0.5, 0.0]
        );
        app.world_mut().despawn(other);
        app.update();
        assert_eq!(
            app.world().resource::<ControllerInput>().frame,
            Frame::default()
        );
    }

    #[test]
    fn pause_modals_focus_and_death_discard_input_until_neutral() {
        let (mut app, pad, window) = fixture();
        for phase in [Phase::Menu, Phase::Paused, Phase::LevelUp, Phase::GameOver] {
            app.world_mut().insert_resource(State::new(phase));
            axis(&mut app, pad, GamepadAxis::LeftStickY, 1.0);
            button(&mut app, pad, GamepadButton::South, 1.0);
            button(&mut app, pad, GamepadButton::RightTrigger, 1.0);
            app.update();
            assert_eq!(
                app.world().resource::<ControllerInput>().frame,
                Frame::default()
            );
            app.world_mut().insert_resource(State::new(Phase::Playing));
            app.update();
            assert_eq!(
                app.world().resource::<ControllerInput>().frame,
                Frame::default()
            );
            axis(&mut app, pad, GamepadAxis::LeftStickY, 0.0);
            button(&mut app, pad, GamepadButton::South, 0.0);
            button(&mut app, pad, GamepadButton::RightTrigger, 0.0);
            app.update();
        }
        for blocked in 0..3 {
            match blocked {
                0 => app.world_mut().resource_mut::<PausePage>().open = true,
                1 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
                _ => app.world_mut().resource_mut::<Run>().dead_t = Some(0.0),
            }
            button(&mut app, pad, GamepadButton::South, 1.0);
            app.update();
            assert_eq!(
                app.world().resource::<ControllerInput>().frame,
                Frame::default()
            );
            app.world_mut().resource_mut::<PausePage>().open = false;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
            app.world_mut().resource_mut::<Run>().dead_t = None;
            app.update();
            assert!(!app.world().resource::<ControllerInput>().frame.jump);
            button(&mut app, pad, GamepadButton::South, 0.0);
            app.update();
        }
        button(&mut app, pad, GamepadButton::South, 1.0);
        app.update();
        assert!(app.world().resource::<ControllerInput>().frame.jump);
        app.update();
        assert!(!app.world().resource::<ControllerInput>().frame.jump);
        button(&mut app, pad, GamepadButton::Start, 1.0);
        app.update();
        assert!(app.world().resource::<ControllerInput>().pause);
        assert_eq!(
            app.world().resource::<ControllerInput>().frame,
            Frame::default()
        );
        button(&mut app, pad, GamepadButton::Start, 0.0);
        button(&mut app, pad, GamepadButton::South, 0.0);
        app.update();
        app.world_mut()
            .resource_mut::<crate::loading_screen::LoadingScreen>()
            .hold_for_capture(None);
        axis(&mut app, pad, GamepadAxis::RightStickX, 1.0);
        app.update();
        assert_eq!(
            app.world().resource::<ControllerInput>().frame,
            Frame::default()
        );
    }
}
