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

/// Shares the engine's process-wide clipboard and its actual Wayland display.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Clipboard<'w, 's> {
    host: bevy::prelude::NonSendMut<'w, crate::textinput::HostClipboard>,
    windows: bevy::prelude::Query<
        'w,
        's,
        &'static bevy::window::RawHandleWrapper,
        bevy::prelude::With<bevy::window::PrimaryWindow>,
    >,
}

trait Pasteboard {
    fn read(&mut self) -> Option<String>;
    fn write(&mut self, text: &str);
}

struct Host<'a> {
    clipboard: &'a mut crate::textinput::HostClipboard,
    display: Option<*mut std::ffi::c_void>,
}

impl Pasteboard for Host<'_> {
    fn read(&mut self) -> Option<String> {
        self.clipboard.read(self.display)
    }

    fn write(&mut self, text: &str) {
        self.clipboard.write(self.display, text);
    }
}

impl Clipboard<'_, '_> {
    pub(super) fn feed(
        &mut self,
        event: &KeyboardInput,
        command: bool,
        armory: &mut Armory,
        account: &mut Account,
        selection: &Selection,
    ) {
        let display = crate::textinput::wayland_display(self.windows.single().ok());
        feed_clipboard(
            event,
            command,
            armory,
            account,
            selection,
            &mut Host {
                clipboard: &mut self.host,
                display,
            },
        );
    }
}

fn feed_clipboard(
    event: &KeyboardInput,
    command: bool,
    armory: &mut Armory,
    account: &mut Account,
    selection: &Selection,
    clipboard: &mut impl Pasteboard,
) {
    if command && event.state == ButtonState::Pressed && armory.focus != 0 {
        match event.key_code {
            KeyCode::KeyV => {
                if let Some(text) = clipboard.read() {
                    // Pasted names follow exactly the same filtering, limit and selection rules.
                    let mut pasted = event.clone();
                    pasted.text = Some(text.into());
                    feed(&pasted, false, armory, account, selection);
                }
                return;
            }
            KeyCode::KeyC | KeyCode::KeyX => {
                if armory.name_selection == Some(armory.focus) {
                    let value = match armory.focus {
                        1 => Some(&armory.draft_first),
                        2 => Some(&armory.draft_last),
                        3 => armory.rename.as_ref().map(|draft| &draft.first),
                        4 => armory.rename.as_ref().map(|draft| &draft.last),
                        _ => None,
                    };
                    if let Some(value) = value.filter(|value| !value.is_empty()) {
                        clipboard.write(value);
                        if event.key_code == KeyCode::KeyX {
                            // An unavailable pasteboard must never turn Cut into data loss.
                            if clipboard.read().as_deref() != Some(value.as_str()) {
                                armory.note = "Clipboard unavailable; name kept.".into();
                                return;
                            }
                            let mut delete = event.clone();
                            delete.key_code = KeyCode::Backspace;
                            delete.text = None;
                            feed(&delete, false, armory, account, selection);
                        }
                    }
                }
                return;
            }
            _ => {}
        }
    }
    feed(event, command, armory, account, selection);
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

    #[derive(Default)]
    struct MemoryClipboard(Option<String>);

    impl Pasteboard for MemoryClipboard {
        fn read(&mut self) -> Option<String> {
            self.0.clone()
        }

        fn write(&mut self, text: &str) {
            self.0 = Some(text.to_owned());
        }
    }

    #[test]
    fn clipboard_copy_cut_paste_respects_selection_and_rename_commit() {
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
        let mut clipboard = MemoryClipboard(Some("Preserve".into()));
        // Without a selection, neither copy nor cut changes the clipboard or draft.
        for key in [KeyCode::KeyC, KeyCode::KeyX] {
            feed_clipboard(
                &event(key, None),
                true,
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
        }
        assert_eq!(clipboard.0.as_deref(), Some("Preserve"));
        assert_eq!(armory.rename.as_ref().unwrap().first, "Old");
        for key in [KeyCode::KeyA, KeyCode::KeyC] {
            feed_clipboard(
                &event(key, None),
                true,
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
        }
        assert_eq!(clipboard.0.as_deref(), Some("Old"));
        assert_eq!(armory.rename.as_ref().unwrap().first, "Old");
        feed_clipboard(
            &event(KeyCode::KeyX, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert!(armory.rename.as_ref().unwrap().first.is_empty());
        assert_eq!(account.character(id).unwrap().first, "Old");
        feed_clipboard(
            &event(KeyCode::KeyV, Some("v")),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.rename.as_ref().unwrap().first, "Old");
        for (key, command) in [
            (KeyCode::Tab, false),
            (KeyCode::KeyA, true),
            (KeyCode::KeyV, true),
            (KeyCode::Enter, false),
        ] {
            feed_clipboard(
                &event(key, None),
                command,
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
        }
        assert_eq!(account.character(id).unwrap().last, "Old");
        assert!(armory.rename.is_none());
    }

    #[test]
    fn unavailable_clipboard_cannot_erase_a_cut_name() {
        struct Unavailable;
        impl Pasteboard for Unavailable {
            fn read(&mut self) -> Option<String> {
                None
            }
            fn write(&mut self, _: &str) {}
        }
        let mut account = crate::survivors::account::empty();
        let mut armory = Armory {
            focus: 1,
            draft_first: "Keep".into(),
            name_selection: Some(1),
            ..Default::default()
        };
        feed_clipboard(
            &event(KeyCode::KeyX, Some("x")),
            true,
            &mut armory,
            &mut account,
            &Selection::default(),
            &mut Unavailable,
        );
        assert_eq!(armory.draft_first, "Keep");
        assert_eq!(armory.name_selection, Some(1));
        assert_eq!(armory.note, "Clipboard unavailable; name kept.");
    }

    #[test]
    fn paste_filters_limits_and_preserves_selection_when_no_valid_text_exists() {
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut armory = Armory {
            focus: 1,
            draft_first: "Selected".into(),
            name_selection: Some(1),
            ..Default::default()
        };
        let mut clipboard = MemoryClipboard::default();
        for text in [None, Some(""), Some("123\n\t😀")] {
            clipboard.0 = text.map(str::to_owned);
            feed_clipboard(
                &event(KeyCode::KeyV, Some("v")),
                true,
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
            assert_eq!(armory.draft_first, "Selected");
            assert_eq!(armory.name_selection, Some(1));
        }
        clipboard.0 = Some("Abc-def' ghi123456789jklmnop".into());
        feed_clipboard(
            &event(KeyCode::KeyV, Some("v")),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
        assert!(armory.name_selection.is_none());
        clipboard.0 = Some("More".into());
        feed_clipboard(
            &event(KeyCode::KeyV, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
        let mut released = event(KeyCode::KeyX, None);
        released.state = ButtonState::Released;
        armory.name_selection = Some(1);
        feed_clipboard(
            &released,
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
        armory.focus = 0;
        feed_clipboard(
            &event(KeyCode::KeyV, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
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
