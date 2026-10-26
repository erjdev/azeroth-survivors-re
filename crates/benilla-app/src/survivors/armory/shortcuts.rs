//! Preserve a right-click modifier through the gesture's release frame.
use bevy::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

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
