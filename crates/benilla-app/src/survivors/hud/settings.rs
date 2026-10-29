//! One focused settings category at a time, shared by the starter and pause menu.
pub(super) mod checkboxes;
pub(super) mod keyboard;
pub(super) mod sliders;
use super::*;
use bevy::ecs::system::SystemParam;

#[derive(Clone, Copy, Default, PartialEq)]
pub(in crate::survivors) enum Category {
    #[default]
    Audio,
    Controls,
    Graphics,
    About,
}

#[derive(Component)]
struct Content(Category);
#[derive(Component)]
struct MenuOnly;

pub(super) fn build(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    col.spawn((
        centered(font, "Options", 36.0, GOLD),
        super::super::armory::DisplayHeading,
    ));
    col.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        flex_shrink: 0.0,
        column_gap: Val::Px(16.0),
        row_gap: Val::Px(12.0),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            width: Val::Percent(24.0),
            min_width: Val::Px(116.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|nav| {
            for (category, title) in [
                (Category::Controls, "Controls"),
                (Category::Audio, "Audio"),
                (Category::Graphics, "Graphics"),
                (Category::About, "About"),
            ] {
                action(nav, font, title, UiAction::SettingsCategory(category));
            }
        });
        row.spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(280.0),
                max_width: Val::Percent(100.0),
                min_height: Val::Px(260.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.015, 0.01)),
            BorderColor::all(EDGE.with_alpha(0.45)),
        ))
        .with_children(|panel| {
            for (category, title) in [
                (Category::Controls, "Controls"),
                (Category::Audio, "Audio"),
                (Category::Graphics, "Graphics"),
                (Category::About, "About"),
            ] {
                panel.spawn((Content(category), Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                })).with_children(|body| {
                    body.spawn(text(font, title, 20.0, GOLD));
                    match category {
                        Category::Controls => {
                            checkboxes::row(body, font, Knob::InvertX);
                            checkboxes::row(body, font, Knob::InvertY);
                            sliders::row(body, font, Knob::CameraSpeed);
                            sliders::row(body, font, Knob::ZoomSpeed);
                            sliders::row(body, font, Knob::StickDeadZone);
                            body.spawn(text(font, "Gamepad: left stick moves, right stick turns, A / South jumps, bumpers / triggers zoom, Start pauses. Release controls after resuming.", 14., DIM));
                            body.spawn(text(font, "C: fixed / orbit camera. In orbit mode, drag either mouse button to rotate. Scroll to zoom.", 14., DIM));
                            body.spawn(text(font, "WASD / arrows: move relative to the orbit camera. Space: jump. Escape: pause.", 14., DIM));
                        }
                        Category::Audio => {
                            for knob in [Knob::Master, Knob::Effects, Knob::Music, Knob::Ambience] {
                                sliders::row(body, font, knob);
                            }
                            for knob in [Knob::Sound, Knob::MusicOn] {
                                checkboxes::row(body, font, knob);
                            }
                        }
                        Category::Graphics => {
                            for knob in [Knob::View, Knob::Grass, Knob::Spells, Knob::Scale, Knob::InterfaceScale] {
                                sliders::row(body, font, knob);
                            }
                            checkboxes::row(body, font, Knob::AutoPush);
                            if cfg!(feature = "survivors-visibility") {
                                checkboxes::row(body, font, Knob::Cutaway);
                            }
                        }
                        Category::About => {
                            body.spawn(text(font, "Azeroth Survivors", 18.0, Color::WHITE));
                            body.spawn(text(font, "Development build. Campaign progress and preferences are saved locally.", 14.0, DIM));
                            body.spawn(text(font, "Build notes and restart controls are available from the starter. Downloaded updates are not available yet.", 14.0, DIM));
                            body.spawn((MenuOnly, Node::default())).with_children(|actions| {
                                action(actions, font, "Build notes / restart", UiAction::BuildNotes);
                            });
                        }
                    }
                });
            }
        });
    });
    col.spawn(text(
        font,
        "Tab / D-pad: focus  •  Left/right: adjust  •  Enter / south: select  •  Escape / east: return",
        12.0,
        DIM,
    ));
    col.spawn(Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::End,
        column_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|footer| {
        for (category, label) in [
            (Category::Controls, "Controls defaults"),
            (Category::Audio, "Audio defaults"),
            (Category::Graphics, "Graphics defaults"),
        ] {
            footer
                .spawn((Content(category), Node::default()))
                .with_children(|slot| {
                    action(slot, font, label, UiAction::RestoreSettings(category));
                });
        }
        action(footer, font, "Done", UiAction::CloseSettings);
    });
}

/// Use the registry's authoritative defaults and normal callback/persistence path.
/// Reset only the selected category, preserving unrelated client preferences.
pub(super) fn restore_cvars(cvars: &mut crate::cvars::Cvars, category: Category) {
    let names: &[&str] = match category {
        Category::Audio => &[
            "MasterVolume",
            "SoundVolume",
            "MusicVolume",
            "AmbienceVolume",
            "MasterSoundEffects",
            "EnableMusic",
        ],
        Category::Graphics => &[
            "farclip",
            "WorldDetail",
            "spellEffectLevel",
            "renderScale",
            "uiScale",
            "useUiScale",
        ],
        Category::Controls | Category::About => &[],
    };
    for name in names {
        if let Some(value) = cvars.default_of(name).map(str::to_owned) {
            cvars.set(name, &value);
        }
    }
}

fn action(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str, action: UiAction) {
    parent
        .spawn((
            button(action, BUTTON),
            Node {
                min_width: Val::Px(96.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn(text(font, label, 16.0, GOLD));
        });
}

type ContentFilter = (
    Without<PauseMain>,
    Without<SettingsPanel>,
    Without<MenuOnly>,
);
type MenuOnlyFilter = (With<MenuOnly>, Without<PauseMain>, Without<SettingsPanel>);

#[derive(SystemParam)]
pub(in crate::survivors) struct Categories<'w, 's> {
    phase: Res<'w, State<Phase>>,
    content: Query<'w, 's, (&'static Content, &'static mut Node), ContentFilter>,
    menu_only: Query<'w, 's, &'static mut Node, MenuOnlyFilter>,
    tabs: Query<
        'w,
        's,
        (
            &'static UiAction,
            &'static Interaction,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
        ),
    >,
}

impl Categories<'_, '_> {
    pub(super) fn sync(mut self, page: &PausePage) {
        for (category, mut node) in &mut self.content {
            node.display = if category.0 == page.category {
                Display::Flex
            } else {
                Display::None
            };
        }
        for mut node in &mut self.menu_only {
            node.display = if *self.phase.get() == Phase::Menu {
                Display::Flex
            } else {
                Display::None
            };
        }
        for (action, interaction, mut bg, mut border) in &mut self.tabs {
            if let UiAction::SettingsCategory(category) = action {
                let selected = *category == page.category;
                bg.0 = if *interaction != Interaction::None {
                    BUTTON_HOT
                } else if selected {
                    BUTTON
                } else {
                    Color::srgb(0.025, 0.018, 0.012)
                };
                *border = BorderColor::all(if selected { GOLD } else { EDGE });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn category_defaults_preserve_other_preferences_and_emit_normal_changes() {
        let mut cvars = crate::cvars::Cvars::default();
        cvars.set("MasterVolume", "0.2");
        cvars.set("EnableMusic", "0");
        cvars.set("farclip", "777");
        cvars.set("renderScale", "0.5");
        cvars.take_events();
        restore_cvars(&mut cvars, Category::Audio);
        assert_eq!(cvars.get("MasterVolume"), cvars.default_of("MasterVolume"));
        assert_eq!(cvars.get("EnableMusic"), cvars.default_of("EnableMusic"));
        assert_eq!(cvars.get("farclip"), Some("777"));
        assert_eq!(cvars.get("renderScale"), Some("0.5"));
        let events = cvars.take_events();
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|event| event.is("MasterVolume")));
        assert!(events.iter().any(|event| event.is("EnableMusic")));
        cvars.set("MasterVolume", "0.3");
        restore_cvars(&mut cvars, Category::Graphics);
        assert_eq!(cvars.get("MasterVolume"), Some("0.3"));
        assert_eq!(cvars.get("farclip"), cvars.default_of("farclip"));
        assert_eq!(cvars.get("renderScale"), cvars.default_of("renderScale"));
        restore_cvars(&mut cvars, Category::About);
        assert_eq!(cvars.get("MasterVolume"), Some("0.3"));
    }

    #[test]
    fn build_notes_are_reachable_from_starter_but_cannot_abandon_a_paused_run() {
        for phase in [Phase::Menu, Phase::Paused] {
            let mut world = World::new();
            world.insert_resource(State::new(phase));
            world.init_resource::<NextState<Phase>>();
            world.init_resource::<Messages<AppExit>>();
            world.init_resource::<SettingEdits>();
            world.init_resource::<super::super::super::armory::Armory>();
            world.insert_resource(PausePage {
                open: true,
                ..default()
            });
            world.spawn((
                Interaction::Pressed,
                UiAction::BuildNotes,
                BaseColor(BUTTON),
                BackgroundColor(BUTTON),
            ));
            world.run_system_once(buttons).unwrap();
            assert_eq!(world.resource::<PausePage>().open, phase == Phase::Paused);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
        }
    }
}
