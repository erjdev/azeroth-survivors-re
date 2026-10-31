//! Preserve a right-click modifier through the gesture's release frame.
use bevy::prelude::*;

#[derive(bevy::ecs::system::SystemParam)]
pub(in crate::survivors) struct Keyboard<'w, 's> {
    pub(super) settings: ResMut<'w, crate::survivors::hud::PausePage>,
    pub(super) keys: Res<'w, ButtonInput<KeyCode>>,
    pub(super) keyboard: MessageReader<'w, 's, bevy::input::keyboard::KeyboardInput>,
    pub(super) shortcut: Local<'s, RightClickModifiers>,
    pub(super) raw: MessageReader<'w, 's, bevy::winit::RawWinitWindowEvent>,
    pub(super) native: Local<'s, NativeModifiers>,
    pub(super) names: Local<'s, super::name_input::NativeText>,
}

#[derive(Default)]
pub(in crate::survivors) struct RightClickModifiers {
    shifted: bool,
}

impl RightClickModifiers {
    pub(super) fn update(
        &mut self,
        mouse: &ButtonInput<MouseButton>,
        keys: &ButtonInput<KeyCode>,
    ) -> bool {
        // Platforms may deliver Shift-up and mouse-up in the same input frame.
        let shifted = [KeyCode::ShiftLeft, KeyCode::ShiftRight]
            .into_iter()
            .any(|key| keys.pressed(key) || keys.just_released(key));
        if mouse.just_pressed(MouseButton::Right) {
            self.shifted = shifted;
        }
        if mouse.just_released(MouseButton::Right) {
            let result = self.shifted || shifted;
            self.shifted = false;
            result
        } else {
            false
        }
    }
}

// macOS can deliver aggregate modifier flags without a physical Shift key event.
// Keep event order: Shift-up may follow mouse-up within the same Bevy frame.
#[derive(Default)]
pub(in crate::survivors) struct NativeModifiers {
    windows: std::collections::HashMap<winit::window::WindowId, NativeGesture>,
}

impl NativeModifiers {
    pub(super) fn read<'a>(
        &mut self,
        events: impl Iterator<Item = &'a bevy::winit::RawWinitWindowEvent>,
    ) -> bool {
        use winit::event::WindowEvent;
        let mut cosmetic_click = false;
        for event in events {
            match &event.event {
                WindowEvent::Focused(false) | WindowEvent::Destroyed => {
                    self.windows.remove(&event.window_id);
                }
                WindowEvent::ModifiersChanged(_) | WindowEvent::MouseInput { .. } => {
                    cosmetic_click |= self
                        .windows
                        .entry(event.window_id)
                        .or_default()
                        .event(&event.event);
                }
                _ => {}
            }
        }
        cosmetic_click
    }
}

#[derive(Default)]
struct NativeGesture {
    shift: bool,
    held_shift: bool,
}

impl NativeGesture {
    fn event(&mut self, event: &winit::event::WindowEvent) -> bool {
        use winit::event::{ElementState, MouseButton, WindowEvent};
        match event {
            WindowEvent::ModifiersChanged(modifiers) => self.shift = modifiers.state().shift_key(),
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => match state {
                ElementState::Pressed => self.held_shift = self.shift,
                ElementState::Released => {
                    return std::mem::take(&mut self.held_shift) || self.shift
                }
            },
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::winit::RawWinitWindowEvent;
    use winit::event::{DeviceId, ElementState, WindowEvent};
    use winit::keyboard::ModifiersState;

    fn raw(window: u64, event: WindowEvent) -> RawWinitWindowEvent {
        RawWinitWindowEvent {
            window_id: window.into(),
            event,
        }
    }

    fn shift(held: bool) -> WindowEvent {
        WindowEvent::ModifiersChanged(
            if held {
                ModifiersState::SHIFT
            } else {
                ModifiersState::empty()
            }
            .into(),
        )
    }

    fn right(state: ElementState) -> WindowEvent {
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state,
            button: winit::event::MouseButton::Right,
        }
    }

    #[test]
    fn aggregate_flags_work_without_physical_key_events_and_do_not_leak() {
        for release_shift_first in [false, true] {
            let mut native = NativeModifiers::default();
            assert!(
                !native.read([raw(1, shift(true)), raw(1, right(ElementState::Pressed))].iter())
            );
            let release = raw(1, right(ElementState::Released));
            let clear = raw(1, shift(false));
            let events = if release_shift_first {
                [clear, release]
            } else {
                [release, clear]
            };
            assert!(native.read(events.iter()));
            assert!(!native.read(
                [
                    raw(1, right(ElementState::Pressed)),
                    raw(1, right(ElementState::Released))
                ]
                .iter()
            ));
        }
    }

    #[test]
    fn native_focus_loss_and_window_destruction_cancel_held_modifiers() {
        for reset in [WindowEvent::Focused(false), WindowEvent::Destroyed] {
            let mut native = NativeModifiers::default();
            native.read(
                [
                    raw(1, shift(true)),
                    raw(1, right(ElementState::Pressed)),
                    raw(1, reset),
                ]
                .iter(),
            );
            assert!(native.windows.is_empty());
            assert!(!native.read([raw(1, right(ElementState::Released))].iter()));
        }
    }

    #[test]
    fn aggregate_modifiers_are_kept_per_window() {
        let mut native = NativeModifiers::default();
        assert!(!native.read(
            [
                raw(1, shift(true)),
                raw(2, right(ElementState::Pressed)),
                raw(2, right(ElementState::Released))
            ]
            .iter()
        ));
        assert!(native.read(
            [
                raw(1, right(ElementState::Pressed)),
                raw(1, right(ElementState::Released)),
                raw(1, shift(false))
            ]
            .iter()
        ));
    }

    #[test]
    fn shift_release_does_not_turn_a_cosmetic_gesture_into_equipment() {
        for shift in [KeyCode::ShiftLeft, KeyCode::ShiftRight] {
            let mut state = RightClickModifiers::default();
            let mut mouse = ButtonInput::default();
            let mut keys = ButtonInput::default();
            keys.press(shift);
            mouse.press(MouseButton::Right);
            assert!(!state.update(&mouse, &keys));
            mouse.clear();
            keys.clear();
            keys.release(shift);
            assert!(!state.update(&mouse, &keys));
            keys.clear();
            mouse.release(MouseButton::Right);
            assert!(state.update(&mouse, &keys));
            mouse.clear();
            mouse.press(MouseButton::Right);
            state.update(&mouse, &keys);
            mouse.clear();
            mouse.release(MouseButton::Right);
            assert!(
                !state.update(&mouse, &keys),
                "modifier must not leak to the next click"
            );
        }
    }

    #[test]
    fn coalesced_modifier_and_mouse_events_keep_the_cosmetic_intent() {
        let mut state = RightClickModifiers::default();
        let mut mouse = ButtonInput::default();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::ShiftLeft);
        keys.release(KeyCode::ShiftLeft);
        mouse.press(MouseButton::Right);
        mouse.release(MouseButton::Right);
        assert!(state.update(&mouse, &keys));
    }
}
