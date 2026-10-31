//! Preserve OS text and key ordering, including repeated presses within one frame.
use super::{Account, Armory, Selection};
use crate::textinput::{Chord, Mods};
use benilla_ui::{script::EditAction, widget::EditBoxState};
use bevy::input::{keyboard::KeyboardInput, ButtonState};
use bevy::prelude::KeyCode;

/// Byte offsets into a single name field; editing is delegated to the shared engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Cursor {
    pub slot: u8,
    pub at: usize,
    pub start: usize,
    pub end: usize,
}

impl Cursor {
    #[cfg(test)]
    fn selected(slot: u8, len: usize) -> Self {
        Self {
            slot,
            at: len,
            start: 0,
            end: len,
        }
    }
}

pub(super) fn editor(value: &str, slot: u8, cursor: Option<Cursor>) -> EditBoxState {
    let mut edit = EditBoxState {
        text: value.to_owned(),
        cursor: value.len(),
        sel_start: value.len(),
        sel_end: value.len(),
        max_letters: 16,
        ..Default::default()
    };
    if let Some(cursor) = cursor.filter(|cursor| cursor.slot == slot) {
        let boundary = |position: usize| value.floor_char_boundary(position.min(value.len()));
        edit.cursor = boundary(cursor.at);
        edit.sel_start = boundary(cursor.start);
        edit.sel_end = boundary(cursor.end);
    }
    edit
}

fn field(armory: &mut Armory) -> Option<&mut String> {
    match armory.focus {
        1 => Some(&mut armory.draft_first),
        2 => Some(&mut armory.draft_last),
        3 => armory.rename.as_mut().map(|draft| &mut draft.first),
        4 => armory.rename.as_mut().map(|draft| &mut draft.last),
        _ => None,
    }
}

fn insert(edit: &mut EditBoxState, text: &str) {
    let selected = edit.selected_text().map_or(0, |text| text.chars().count());
    let available = 16usize.saturating_sub(edit.text.chars().count() - selected);
    let accepted: String = text
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic() || " -'".contains(*ch))
        .take(available)
        .collect();
    if !accepted.is_empty() {
        edit.insert(&accepted);
    }
}

fn shortcut(event: &KeyboardInput, mods: Mods) -> Option<Chord> {
    // Survivors accepts both Command and Ctrl for its existing clipboard shortcuts.
    if (mods.sup || mods.ctrl) && !mods.alt {
        match event.key_code {
            KeyCode::KeyA => return Some(Chord::Edit(EditAction::SelectAll)),
            KeyCode::KeyC => return Some(Chord::Copy),
            KeyCode::KeyX => return Some(Chord::Cut),
            KeyCode::KeyV => return Some(Chord::Paste),
            _ => {}
        }
    }
    crate::textinput::chord(event.key_code, None, mods, cfg!(target_os = "macos"))
}

/// Raw modifier snapshots preserve Command/Ctrl across a press and release in one frame.
#[derive(Default)]
pub(super) struct NativeText {
    windows: std::collections::HashMap<winit::window::WindowId, winit::keyboard::ModifiersState>,
}

impl NativeText {
    pub(super) fn read<'a>(
        &mut self,
        events: impl Iterator<Item = &'a bevy::winit::RawWinitWindowEvent>,
    ) -> Vec<(KeyboardInput, Mods)> {
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
                        Mods {
                            shift: modifiers.shift_key(),
                            ctrl: modifiers.control_key(),
                            alt: modifiers.alt_key(),
                            sup: modifiers.super_key(),
                        },
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
        mods: Mods,
        armory: &mut Armory,
        account: &mut Account,
        selection: &Selection,
    ) {
        let display = crate::textinput::wayland_display(self.windows.single().ok());
        feed_clipboard(
            event,
            mods,
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
    mods: Mods,
    armory: &mut Armory,
    account: &mut Account,
    selection: &Selection,
    clipboard: &mut impl Pasteboard,
) {
    if event.state != ButtonState::Pressed || armory.focus == 0 {
        return;
    }
    match event.key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            armory.name_cursor = None;
            if armory.rename.is_some() {
                super::save_name(armory, account, selection);
            } else {
                armory.focus = 0;
            }
            return;
        }
        KeyCode::Escape => {
            armory.name_cursor = None;
            armory.rename = None;
            armory.focus = 0;
            armory.note.clear();
            return;
        }
        KeyCode::Tab => {
            armory.name_cursor = None;
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
    let slot = armory.focus;
    let cursor = armory.name_cursor;
    let Some(value) = field(armory) else { return };
    let mut edit = editor(value, slot, cursor);
    match shortcut(event, mods) {
        Some(Chord::Edit(action)) => {
            edit.apply(action);
        }
        Some(Chord::Copy | Chord::Cut) => {
            if let Some(text) = edit.selected_text() {
                clipboard.write(&text);
                if shortcut(event, mods) == Some(Chord::Cut) {
                    // An unavailable pasteboard must never turn Cut into data loss.
                    if clipboard.read().as_deref() != Some(text.as_str()) {
                        armory.note = "Clipboard unavailable; name kept.".into();
                        return;
                    }
                    edit.cut_selection();
                }
            }
        }
        Some(Chord::Paste) => {
            if let Some(text) = clipboard.read() {
                insert(&mut edit, &text);
            }
        }
        None if !mods.sup && (!mods.ctrl || mods.alt) => {
            if let Some(text) = &event.text {
                insert(&mut edit, text);
            }
        }
        None => {}
    }
    let changed = *value != edit.text;
    *value = edit.text;
    armory.name_cursor = Some(Cursor {
        slot,
        at: edit.cursor,
        start: edit.sel_start,
        end: edit.sel_end,
    });
    if changed {
        armory.note.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{input::keyboard::Key, prelude::Entity};

    fn feed_clipboard(
        event: &KeyboardInput,
        command: bool,
        armory: &mut Armory,
        account: &mut Account,
        selection: &Selection,
        clipboard: &mut impl Pasteboard,
    ) {
        super::feed_clipboard(
            event,
            Mods {
                ctrl: command,
                ..Default::default()
            },
            armory,
            account,
            selection,
            clipboard,
        );
    }

    fn feed(
        event: &KeyboardInput,
        command: bool,
        armory: &mut Armory,
        account: &mut Account,
        selection: &Selection,
    ) {
        feed_clipboard(
            event,
            command,
            armory,
            account,
            selection,
            &mut MemoryClipboard::default(),
        );
    }

    fn has_selection(armory: &Armory) -> bool {
        armory
            .name_cursor
            .is_some_and(|c| c.slot == armory.focus && c.start != c.end)
    }

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
    fn caret_inserts_and_deletes_inside_a_name_without_touching_the_suffix() {
        let mut armory = Armory {
            focus: 1,
            draft_first: "MiraDawn".into(),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut clipboard = MemoryClipboard::default();
        let mut send = |key, text, mods| {
            super::feed_clipboard(
                &event(key, text),
                mods,
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            )
        };
        send(KeyCode::Home, None, Mods::default());
        for _ in 0..4 {
            send(KeyCode::ArrowRight, None, Mods::default());
        }
        send(KeyCode::Space, Some(" "), Mods::default());
        send(KeyCode::Delete, None, Mods::default());
        send(KeyCode::KeyD, Some("D"), Mods::default());
        send(KeyCode::Backspace, None, Mods::default());
        send(KeyCode::KeyD, Some("D"), Mods::default());
        assert_eq!(armory.draft_first, "Mira Dawn");
        assert_eq!(armory.name_cursor.unwrap().at, 6);
    }

    #[test]
    fn partial_selection_copies_cuts_and_replaces_only_the_selected_letters() {
        let mut armory = Armory {
            focus: 1,
            draft_first: "Mira Dawn".into(),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut clipboard = MemoryClipboard::default();
        for _ in 0..4 {
            super::feed_clipboard(
                &event(KeyCode::ArrowLeft, None),
                Mods {
                    shift: true,
                    ..Default::default()
                },
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
        }
        feed_clipboard(
            &event(KeyCode::KeyC, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(clipboard.0.as_deref(), Some("Dawn"));
        feed_clipboard(
            &event(KeyCode::KeyX, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Mira ");
        feed_clipboard(
            &event(KeyCode::KeyV, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Mira Dawn");
        super::feed_clipboard(
            &event(KeyCode::Home, None),
            Mods::default(),
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        for _ in 0..4 {
            super::feed_clipboard(
                &event(KeyCode::ArrowRight, None),
                Mods {
                    shift: true,
                    ..Default::default()
                },
                &mut armory,
                &mut account,
                &selection,
                &mut clipboard,
            );
        }
        clipboard.0 = Some("Lena123".into());
        feed_clipboard(
            &event(KeyCode::KeyV, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "Lena Dawn");
        assert_eq!(armory.name_cursor.unwrap().at, 4);
    }

    #[test]
    fn full_names_preserve_unselected_suffix_when_pasting_in_the_middle() {
        let mut armory = Armory {
            focus: 1,
            draft_first: "Abcdefghijklmnop".into(),
            name_cursor: Some(Cursor {
                slot: 1,
                at: 4,
                start: 2,
                end: 4,
            }),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut clipboard = MemoryClipboard(Some("XYZ".into()));
        feed_clipboard(
            &event(KeyCode::KeyV, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(armory.draft_first, "AbXYefghijklmnop");
        feed(
            &event(KeyCode::KeyQ, Some("Q")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "AbXYefghijklmnop");
        assert_eq!(armory.name_cursor.unwrap().at, 4);
    }

    #[test]
    fn word_movement_extends_selection_and_tab_resets_the_next_field_caret() {
        let mut armory = Armory {
            focus: 1,
            draft_first: "Mira Dawn".into(),
            draft_last: "Vale".into(),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut clipboard = MemoryClipboard::default();
        let word = Mods {
            shift: true,
            ctrl: !cfg!(target_os = "macos"),
            alt: cfg!(target_os = "macos"),
            ..Default::default()
        };
        super::feed_clipboard(
            &event(KeyCode::ArrowLeft, None),
            word,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        feed_clipboard(
            &event(KeyCode::KeyC, None),
            true,
            &mut armory,
            &mut account,
            &selection,
            &mut clipboard,
        );
        assert_eq!(clipboard.0.as_deref(), Some("Dawn"));
        feed(
            &event(KeyCode::Tab, None),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        feed(
            &event(KeyCode::KeyS, Some("s")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Mira Dawn");
        assert_eq!(armory.draft_last, "Vales");
        assert_eq!(armory.name_cursor.unwrap().slot, 2);
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
            name_cursor: Some(Cursor::selected(1, 4)),
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
        assert!(has_selection(&armory));
        assert_eq!(armory.note, "Clipboard unavailable; name kept.");
    }

    #[test]
    fn paste_filters_limits_and_preserves_selection_when_no_valid_text_exists() {
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        let mut armory = Armory {
            focus: 1,
            draft_first: "Selected".into(),
            name_cursor: Some(Cursor::selected(1, 8)),
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
            assert!(has_selection(&armory));
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
        assert!(!has_selection(&armory));
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
        armory.name_cursor = Some(Cursor::selected(1, armory.draft_first.len()));
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
        assert!(has_selection(&armory));
        feed(
            &event(KeyCode::Digit1, Some("123")),
            false,
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Old");
        assert!(has_selection(&armory));
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
        assert!(!has_selection(&armory));
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
        assert!(!has_selection(&armory));
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
        assert!(!has_selection(&armory));
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
