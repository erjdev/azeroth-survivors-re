//! Preserve OS text and key ordering, including repeated presses within one frame.
use super::{Account, Armory, Selection};
use bevy::input::{keyboard::KeyboardInput, ButtonState};
use bevy::prelude::KeyCode;

/// Raw modifier snapshots preserve Command/Ctrl across a press and release in one frame.
#[derive(Default)]
pub(super) struct NativeText {
    windows: std::collections::HashMap<winit::window::WindowId, winit::keyboard::ModifiersState>,
}

impl NativeText {
    pub(super) fn read<'a>(
        &mut self,
        events: impl Iterator<Item = &'a bevy::winit::RawWinitWindowEvent>,
    ) -> Vec<(KeyboardInput, bool)> {
        use winit::event::WindowEvent;
        let mut inputs = Vec::new();
        for raw in events {
            match &raw.event {
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.windows.insert(raw.window_id, modifiers.state());
                }
                WindowEvent::Focused(false) | WindowEvent::Destroyed => {
                    self.windows.remove(&raw.window_id);
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic: false,
                    ..
                } => {
                    let modifiers = self
                        .windows
                        .get(&raw.window_id)
                        .copied()
                        .unwrap_or_default();
                    inputs.push((
                        bevy::winit::converters::convert_keyboard_input(
                            event,
                            bevy::prelude::Entity::PLACEHOLDER,
                        ),
                        modifiers.super_key() || modifiers.control_key(),
                    ));
                }
                _ => {}
            }
        }
        inputs
    }
}

pub(super) fn feed(
    event: &KeyboardInput,
    command: bool,
    armory: &mut Armory,
    account: &mut Account,
    selection: &Selection,
) {
    if event.state != ButtonState::Pressed || armory.focus == 0 {
        return;
    }
    if command {
        if event.key_code == KeyCode::KeyA {
            armory.name_selection = Some(armory.focus);
        }
        // Shortcut letters must never become part of the name.
        return;
    }
    match event.key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            armory.name_selection = None;
            if armory.rename.is_some() {
                super::save_name(armory, account, selection);
            } else {
                armory.focus = 0;
            }
            return;
        }
        KeyCode::Escape => {
            armory.name_selection = None;
            armory.rename = None;
            armory.focus = 0;
            armory.note.clear();
            return;
        }
        KeyCode::Tab => {
            armory.name_selection = None;
            armory.focus = match armory.focus {
                1 => 2,
                2 => 1,
                3 => 4,
                4 => 3,
                _ => 0,
            };
            return;
        }
        _ => {}
    }
    let selected = armory.name_selection == Some(armory.focus);
    let field = match armory.focus {
        1 => &mut armory.draft_first,
        2 => &mut armory.draft_last,
        3 | 4 => {
            let Some(draft) = armory.rename.as_mut() else {
                return;
            };
            if armory.focus == 3 {
                &mut draft.first
            } else {
                &mut draft.last
            }
        }
        _ => return,
    };
    let previous = field.clone();
    if event.key_code == KeyCode::Backspace || (selected && event.key_code == KeyCode::Delete) {
        if selected {
            field.clear();
            armory.name_selection = None;
        } else {
            field.pop();
        }
    } else if let Some(text) = &event.text {
        let accepted: String = text
            .chars()
            .filter(|ch| ch.is_ascii_alphabetic() || " -'".contains(*ch))
            .collect();
        if selected && !accepted.is_empty() {
            field.clear();
            armory.name_selection = None;
        }
        for ch in accepted.chars() {
            if field.chars().count() < 16 {
                field.push(ch);
            }
        }
    }
    if *field != previous {
        armory.note.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{input::keyboard::Key, prelude::Entity};

    fn event(key_code: KeyCode, text: Option<&str>) -> KeyboardInput {
        KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: text.map(Into::into),
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn native_modifiers_are_window_scoped_and_cleared_on_focus_loss() {
        use bevy::winit::RawWinitWindowEvent;
        use winit::{event::WindowEvent, keyboard::ModifiersState, window::WindowId};
        let window = WindowId::dummy();
        let mut native = NativeText::default();
        let pressed = RawWinitWindowEvent {
            window_id: window,
            event: WindowEvent::ModifiersChanged(ModifiersState::SUPER.into()),
        };
        assert!(native.read(std::iter::once(&pressed)).is_empty());
        assert!(native.windows[&window].super_key());
        let released = RawWinitWindowEvent {
            window_id: window,
            event: WindowEvent::ModifiersChanged(ModifiersState::empty().into()),
        };
        native.read(std::iter::once(&released));
        assert!(!native.windows[&window].super_key());
        let ctrl = RawWinitWindowEvent {
            window_id: window,
            event: WindowEvent::ModifiersChanged(ModifiersState::CONTROL.into()),
        };
        native.read(std::iter::once(&ctrl));
        assert!(native.windows[&window].control_key());
        let lost = RawWinitWindowEvent {
            window_id: window,
            event: WindowEvent::Focused(false),
        };
        native.read(std::iter::once(&lost));
        assert!(native.windows.is_empty());
    }

    #[test]
    fn ordered_text_repeats_tab_and_enter_share_one_batch() {
        let mut account = crate::survivors::account::empty();
        let id = account.create("Old", "Hero", 0).unwrap();
        let selection = Selection {
            class: Some(0),
            character: Some(id),
            ..Default::default()
        };
        let mut armory = Armory {
            rename: super::super::name_edit::Draft::begin(&account, &selection),
            focus: 3,
            ..Default::default()
        };
        for ev in [
            event(KeyCode::Backspace, None),
            event(KeyCode::Backspace, None),
            event(KeyCode::Backspace, None),
            event(KeyCode::KeyL, Some("L")),
            event(KeyCode::KeyE, Some("e")),
            event(KeyCode::KeyE, Some("e")),
            event(KeyCode::Tab, None),
            event(KeyCode::KeyS, Some("s")),
            event(KeyCode::Enter, None),
            event(KeyCode::KeyX, Some("x")),
        ] {
            feed(&ev, false, &mut armory, &mut account, &selection);
        }
        let hero = account.character(id).unwrap();
        assert_eq!((&*hero.first, &*hero.last), ("Lee", "Heros"));
        assert!(armory.rename.is_none());
        assert_eq!(armory.focus, 0);
    }

    #[test]
    fn select_all_replaces_without_inserting_shortcut_letters_and_clears_equal_length_errors() {
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut armory = Armory {
            focus: 1,
            draft_first: "Old".into(),
            ..Default::default()
        };
        feed(
            &event(KeyCode::KeyA, Some("a")),
            true,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Old");
        assert_eq!(armory.name_selection, Some(1));
        feed(
            &event(KeyCode::Digit1, Some("123")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Old");
        assert_eq!(armory.name_selection, Some(1));
        armory.note = "Old error".into();
        feed(
            &event(KeyCode::KeyN, Some("New")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "New");
        assert!(armory.note.is_empty());
        assert_eq!(armory.name_selection, None);
        feed(
            &event(KeyCode::KeyV, Some("v")),
            true,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "New");
        feed(
            &event(KeyCode::KeyA, Some("a")),
            true,
            &mut armory,
            &mut account,
            &selection,
        );
        feed(
            &event(KeyCode::Delete, None),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert!(armory.draft_first.is_empty());
        assert_eq!(armory.name_selection, None);
    }

    #[test]
    fn rename_selection_is_field_local_and_escape_does_not_save() {
        let mut account = crate::survivors::account::empty();
        let id = account.create("Old", "Hero", 0).unwrap();
        let selection = Selection {
            class: Some(0),
            character: Some(id),
            ..Default::default()
        };
        let mut armory = Armory {
            rename: super::super::name_edit::Draft::begin(&account, &selection),
            focus: 3,
            ..Default::default()
        };
        for (key, text, command) in [
            (KeyCode::KeyA, Some("a"), true),
            (KeyCode::Tab, None, false),
            (KeyCode::KeyS, Some("s"), false),
            (KeyCode::KeyA, Some("a"), true),
            (KeyCode::Backspace, None, false),
        ] {
            feed(
                &event(key, text),
                command,
                &mut armory,
                &mut account,
                &selection,
            );
        }
        let draft = armory.rename.as_ref().unwrap();
        assert_eq!(draft.first, "Old");
        assert!(draft.last.is_empty());
        feed(
            &event(KeyCode::Escape, None),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert!(armory.rename.is_none());
        assert_eq!(armory.name_selection, None);
        assert_eq!(account.character(id).unwrap().last, "Hero");
    }

    #[test]
    fn creation_text_filters_and_caps_without_using_physical_key() {
        let mut armory = Armory {
            focus: 1,
            draft_first: String::new(),
            note: "A hero needs a first and last name".into(),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        feed(
            &event(KeyCode::KeyQ, Some("Abc-def' ghi123456789jklmnop")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
        assert!(armory.note.is_empty());
        let mut released = event(KeyCode::Backspace, None);
        released.state = ButtonState::Released;
        feed(&released, false, &mut armory, &mut account, &selection);
        assert_eq!(armory.draft_first.chars().count(), 16);
    }
}
