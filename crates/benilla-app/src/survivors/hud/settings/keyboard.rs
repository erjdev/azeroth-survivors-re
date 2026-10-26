//! Shared keyboard/controller Options focus excludes hidden categories and starter-only actions.
use super::*;
use bevy::input::{mouse::MouseButtonInput, ButtonState};

#[derive(Clone, Copy, PartialEq)]
enum Target {
    Action(UiAction),
    Slider(Knob),
}

#[derive(Default)]
pub(in crate::survivors) struct Focus {
    target: Option<Target>,
    entity: Option<Entity>,
    native_shift: bool,
}

fn order(category: Category, phase: Phase) -> Vec<Target> {
    let mut targets = [
        Category::Controls,
        Category::Audio,
        Category::Graphics,
        Category::About,
    ]
    .map(|category| Target::Action(UiAction::SettingsCategory(category)))
    .to_vec();
    let knobs: &[Knob] = match category {
        Category::Audio => &[Knob::Master, Knob::Effects, Knob::Music, Knob::Ambience],
        Category::Graphics => &[Knob::View, Knob::Grass, Knob::Spells, Knob::Scale],
        Category::Controls => &[Knob::CameraSpeed, Knob::ZoomSpeed],
        Category::About => &[],
    };
    if category == Category::Controls {
        targets.push(Target::Action(UiAction::Toggle(Knob::InvertX)));
        targets.push(Target::Action(UiAction::Toggle(Knob::InvertY)));
    }
    targets.extend(knobs.iter().copied().map(Target::Slider));
    match category {
        Category::Controls => {}
        Category::Audio => {
            targets.push(Target::Action(UiAction::Toggle(Knob::Sound)));
            targets.push(Target::Action(UiAction::Toggle(Knob::MusicOn)));
        }
        Category::Graphics => {
            targets.push(Target::Action(UiAction::Toggle(Knob::AutoPush)));
            if cfg!(feature = "survivors-visibility") {
                targets.push(Target::Action(UiAction::Toggle(Knob::Cutaway)));
            }
        }
        Category::About if phase == Phase::Menu => {
            targets.push(Target::Action(UiAction::BuildNotes))
        }
        Category::About => {}
    }
    if category != Category::About {
        targets.push(Target::Action(UiAction::RestoreSettings(category)));
    }
    targets.push(Target::Action(UiAction::CloseSettings));
    targets
}

type FocusNodes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static UiAction>,
        Option<&'static sliders::Slider>,
    ),
    With<Button>,
>;

#[allow(clippy::too_many_arguments)]
pub(in crate::survivors) fn navigate(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse: MessageReader<MouseButtonInput>,
    mut raw: MessageReader<bevy::winit::RawWinitWindowEvent>,
    phase: Res<State<Phase>>,
    preview: Res<super::super::super::cards::PreviewHand>,
    mut page: ResMut<PausePage>,
    cvars: Res<crate::cvars::Cvars>,
    camera: Res<super::super::super::camera::CameraSettings>,
    mut next: ResMut<NextState<Phase>>,
    mut exit: MessageWriter<AppExit>,
    mut edits: ResMut<SettingEdits>,
    mut armory: ResMut<super::super::super::armory::Armory>,
    nodes: FocusNodes,
    pads: Query<&Gamepad>,
    mut focus: Local<Focus>,
) {
    // macOS can send modifier flags without physical Shift key events.
    // Capture Shift at Tab-down, before a release later in this frame.
    let mut shifted_tab = false;
    for event in raw.read() {
        use winit::event::{ElementState, WindowEvent};
        match &event.event {
            WindowEvent::ModifiersChanged(modifiers) => {
                focus.native_shift = modifiers.state().shift_key();
            }
            WindowEvent::Focused(false) => focus.native_shift = false,
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == winit::keyboard::KeyCode::Tab =>
            {
                shifted_tab |= focus.native_shift;
            }
            _ => {}
        }
    }
    // Take one controller's new presses per frame, so two pads cannot combine
    // a navigation press on one device with an unintended selection on another.
    let pad = pads.iter().find(|pad| {
        [
            GamepadButton::Start,
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ]
        .into_iter()
        .any(|button| pad.just_pressed(button))
    });
    let pressed = |button| pad.is_some_and(|pad| pad.just_pressed(button));
    let eligible = !preview.0 && matches!(phase.get(), Phase::Menu | Phase::Paused);
    if eligible {
        if page.open && pressed(GamepadButton::East) {
            page.open = false;
        } else if !page.open && pressed(GamepadButton::Start) {
            page.open = true;
            focus.target = Some(Target::Action(UiAction::SettingsCategory(
                Category::Controls,
            )));
        }
    }
    let pointer = mouse
        .read()
        .any(|event| event.state == ButtonState::Pressed);
    if !page.open || preview.0 || !matches!(phase.get(), Phase::Menu | Phase::Paused) || pointer {
        focus.target = None;
    } else {
        let order = order(page.category, *phase.get());
        let current = focus
            .target
            .and_then(|target| order.iter().position(|item| *item == target));
        let backwards =
            shifted_tab || keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let movement = if keys.just_pressed(KeyCode::Tab) {
            Some(if backwards { -1 } else { 1 })
        } else if keys.just_pressed(KeyCode::ArrowUp) || pressed(GamepadButton::DPadUp) {
            Some(-1)
        } else if keys.just_pressed(KeyCode::ArrowDown) || pressed(GamepadButton::DPadDown) {
            Some(1)
        } else {
            None
        };
        if let Some(direction) = movement {
            let index = current.map_or_else(
                || if direction < 0 { order.len() - 1 } else { 0 },
                |index| (index as isize + direction).rem_euclid(order.len() as isize) as usize,
            );
            focus.target = Some(order[index]);
        }
        if let Some(target) = focus.target {
            match target {
                Target::Slider(knob) => {
                    let position = if keys.just_pressed(KeyCode::Home) {
                        Some(0.)
                    } else if keys.just_pressed(KeyCode::End) {
                        Some(1.)
                    } else if keys.just_pressed(KeyCode::ArrowLeft)
                        || pressed(GamepadButton::DPadLeft)
                    {
                        Some(sliders::nudge(&cvars, &camera, knob, -1.))
                    } else if keys.just_pressed(KeyCode::ArrowRight)
                        || pressed(GamepadButton::DPadRight)
                    {
                        Some(sliders::nudge(&cvars, &camera, knob, 1.))
                    } else {
                        None
                    };
                    if let Some(position) = position {
                        edits.set.push((knob, position));
                    }
                }
                Target::Action(action)
                    if keys.just_pressed(KeyCode::Enter)
                        || keys.just_pressed(KeyCode::Space)
                        || pressed(GamepadButton::South) =>
                {
                    activate_action(
                        action,
                        *phase.get(),
                        &mut next,
                        &mut exit,
                        &mut edits,
                        &mut page,
                        &mut armory,
                    );
                }
                _ => {}
            }
        }
    }
    if !page.open {
        focus.target = None;
    }
    let entity = focus.target.and_then(|target| {
        nodes
            .iter()
            .find(|(_, action, slider)| match target {
                Target::Action(want) => action.is_some_and(|action| *action == want),
                Target::Slider(want) => slider.is_some_and(|slider| slider.0 == want),
            })
            .map(|(entity, _, _)| entity)
    });
    if focus.entity != entity {
        if let Some(old) = focus.entity {
            commands.entity(old).remove::<Outline>();
        }
        if let Some(entity) = entity {
            commands.entity(entity).insert(Outline {
                width: Val::Px(2.),
                offset: Val::Px(3.),
                color: Color::WHITE,
            });
        }
        focus.entity = entity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_focus_reaches_done_in_both_phases_without_changing_run_state() {
        for phase in [Phase::Menu, Phase::Paused] {
            let mut world = World::new();
            world.init_resource::<ButtonInput<KeyCode>>();
            world.init_resource::<Messages<MouseButtonInput>>();
            world.init_resource::<Messages<bevy::winit::RawWinitWindowEvent>>();
            world.init_resource::<Messages<AppExit>>();
            world.init_resource::<NextState<Phase>>();
            world.init_resource::<crate::cvars::Cvars>();
            world.init_resource::<super::super::super::super::camera::CameraSettings>();
            world.init_resource::<SettingEdits>();
            world.init_resource::<super::super::super::super::armory::Armory>();
            world.insert_resource(State::new(phase));
            world.insert_resource(PausePage {
                open: true,
                ..default()
            });
            world.init_resource::<super::super::super::super::cards::PreviewHand>();
            world.init_resource::<super::super::super::super::renown::Renown>();
            world.init_resource::<super::super::super::super::stash::Stash>();
            world.insert_resource(super::super::super::super::account::empty());
            world.init_resource::<super::super::super::super::Typing>();
            world.init_resource::<Run>();
            world.insert_resource(Selection {
                class: Some(0),
                looks: vec![(49, 1, 0)],
                look: Some(0),
                map: data::MAPS
                    .iter()
                    .position(|map| map.name == "Elwynn Forest")
                    .unwrap(),
                ..default()
            });
            let mut schedule = Schedule::default();
            schedule.add_systems((super::super::super::keys, navigate).chain());
            // Shift-Tab from no focus selects the last visible control: Done.
            {
                let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::ShiftLeft);
                keys.press(KeyCode::Tab);
            }
            schedule.run(&mut world);
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            schedule.run(&mut world);
            assert!(!world.resource::<PausePage>().open);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
            // Held Enter does not reactivate anything after the modal is gone.
            world.resource_mut::<ButtonInput<KeyCode>>().clear();
            schedule.run(&mut world);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
        }
    }

    fn controller_world(phase: Phase) -> (World, Entity, Schedule) {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<Messages<MouseButtonInput>>();
        world.init_resource::<Messages<bevy::winit::RawWinitWindowEvent>>();
        world.init_resource::<Messages<AppExit>>();
        world.init_resource::<NextState<Phase>>();
        world.init_resource::<crate::cvars::Cvars>();
        let mut camera = super::super::super::super::camera::CameraSettings::default();
        camera.camera_speed = 1.;
        camera.zoom_speed = 1.;
        world.insert_resource(camera);
        world.init_resource::<SettingEdits>();
        world.init_resource::<super::super::super::super::armory::Armory>();
        world.init_resource::<super::super::super::super::cards::PreviewHand>();
        world.init_resource::<PausePage>();
        world.insert_resource(State::new(phase));
        let pad = world.spawn(Gamepad::default()).id();
        let mut schedule = Schedule::default();
        schedule.add_systems(navigate);
        (world, pad, schedule)
    }

    fn press_pad(world: &mut World, pad: Entity, schedule: &mut Schedule, button: GamepadButton) {
        let mut gamepad = world.get_mut::<Gamepad>(pad).unwrap();
        gamepad.digital_mut().reset_all();
        gamepad.digital_mut().press(button);
        schedule.run(world);
    }

    #[test]
    fn controller_opens_navigates_adjusts_and_closes_options_without_starting_or_resuming() {
        for phase in [Phase::Menu, Phase::Paused] {
            let (mut world, pad, mut schedule) = controller_world(phase);
            press_pad(&mut world, pad, &mut schedule, GamepadButton::Start);
            assert!(world.resource::<PausePage>().open);
            press_pad(&mut world, pad, &mut schedule, GamepadButton::South);
            assert!(world.resource::<PausePage>().category == Category::Controls);
            for _ in 0..4 {
                press_pad(&mut world, pad, &mut schedule, GamepadButton::DPadDown);
            }
            press_pad(&mut world, pad, &mut schedule, GamepadButton::South);
            assert!(world.resource::<SettingEdits>().toggle == vec![Knob::InvertX]);
            for _ in 0..2 {
                press_pad(&mut world, pad, &mut schedule, GamepadButton::DPadDown);
            }
            press_pad(&mut world, pad, &mut schedule, GamepadButton::DPadRight);
            let edits = world.resource::<SettingEdits>();
            assert!(edits.set.len() == 1 && edits.set[0].0 == Knob::CameraSpeed);
            assert_eq!(
                sliders::speed_value(edits.set[0].0, edits.set[0].1),
                Some(1.25)
            );
            press_pad(&mut world, pad, &mut schedule, GamepadButton::East);
            assert!(!world.resource::<PausePage>().open);
            world.get_mut::<Gamepad>(pad).unwrap().digital_mut().clear();
            schedule.run(&mut world);
            assert!(!world.resource::<PausePage>().open);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
        }
    }

    #[test]
    fn controller_options_entry_ignores_gameplay_and_card_previews() {
        for phase in [Phase::Menu, Phase::Playing] {
            let (mut world, pad, mut schedule) = controller_world(phase);
            if phase == Phase::Menu {
                world
                    .resource_mut::<super::super::super::super::cards::PreviewHand>()
                    .0 = true;
            }
            press_pad(&mut world, pad, &mut schedule, GamepadButton::Start);
            assert!(!world.resource::<PausePage>().open);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
        }
    }

    #[test]
    fn paused_about_omits_restart_and_hidden_categories_do_not_receive_focus() {
        let targets = order(Category::About, Phase::Paused);
        assert!(!targets.contains(&Target::Action(UiAction::BuildNotes)));
        assert!(!targets
            .iter()
            .any(|target| matches!(target, Target::Slider(_))));
        assert!(order(Category::About, Phase::Menu).contains(&Target::Action(UiAction::BuildNotes)));
        let mut cvars = crate::cvars::Cvars::default();
        let camera = super::super::super::super::camera::CameraSettings::default();
        cvars.set("MasterVolume", "0.37");
        assert_eq!(sliders::nudge(&cvars, &camera, Knob::Master, 1.), 0.38);
        assert_eq!(sliders::nudge(&cvars, &camera, Knob::Master, -1.), 0.36);
        let position = sliders::nudge(&cvars, &camera, Knob::View, 1.);
        sliders::apply(&mut cvars, Knob::View, position);
        assert_eq!(cvars.num("farclip"), Some(500.));
    }
}
