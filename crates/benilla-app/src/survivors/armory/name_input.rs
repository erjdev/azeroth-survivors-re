//! Preserve OS text and key ordering, including repeated presses within one frame.
use super::{Account, Armory, Selection};
use bevy::input::{keyboard::KeyboardInput, ButtonState};
use bevy::prelude::KeyCode;

pub(super) fn feed(
    event: &KeyboardInput,
    armory: &mut Armory,
    account: &mut Account,
    selection: &Selection,
) {
    if event.state != ButtonState::Pressed || armory.focus == 0 {
        return;
    }
    match event.key_code {
        KeyCode::Enter | KeyCode::NumpadEnter => {
            if armory.rename.is_some() {
                super::save_name(armory, account, selection);
            } else {
                armory.focus = 0;
            }
            return;
        }
        KeyCode::Escape => {
            armory.rename = None;
            armory.focus = 0;
            armory.note.clear();
            return;
        }
        KeyCode::Tab => {
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
    if event.key_code == KeyCode::Backspace {
        field.pop();
    } else if let Some(text) = &event.text {
        for ch in text.chars() {
            if field.chars().count() < 16 && (ch.is_ascii_alphabetic() || " -'".contains(ch)) {
                field.push(ch);
            }
        }
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
            feed(&ev, &mut armory, &mut account, &selection);
        }
        let hero = account.character(id).unwrap();
        assert_eq!((&*hero.first, &*hero.last), ("Lee", "Heros"));
        assert!(armory.rename.is_none());
        assert_eq!(armory.focus, 0);
    }

    #[test]
    fn creation_text_filters_and_caps_without_using_physical_key() {
        let mut armory = Armory {
            focus: 1,
            draft_first: String::new(),
            ..Default::default()
        };
        let mut account = crate::survivors::account::empty();
        let selection = Selection::default();
        feed(
            &event(KeyCode::KeyQ, Some("Abc-def' ghi123456789jklmnop")),
            &mut armory,
            &mut account,
            &selection,
        );
        assert_eq!(armory.draft_first, "Abc-def' ghijklm");
        let mut released = event(KeyCode::Backspace, None);
        released.state = ButtonState::Released;
        feed(&released, &mut armory, &mut account, &selection);
        assert_eq!(armory.draft_first.chars().count(), 16);
    }
}
