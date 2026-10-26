//! Controller focus uses the same guarded actions as pointer selection.
use super::*;

#[derive(Resource, Default)]
pub(in crate::survivors) struct ControllerFocus {
    focused: Option<Entity>,
    pub activated: Option<Entity>,
    pub keyboard_selected: bool,
}

#[derive(Clone, Copy)]
struct Item {
    entity: Entity,
    center: Vec2,
    size: Vec2,
}

fn step(items: &[Item], current: Option<Entity>, direction: Vec2) -> Option<Entity> {
    let Some(from) = items.iter().find(|item| Some(item.entity) == current) else {
        return items
            .iter()
            .min_by(|a, b| {
                a.center
                    .y
                    .total_cmp(&b.center.y)
                    .then(a.center.x.total_cmp(&b.center.x))
                    .then(a.entity.cmp(&b.entity))
            })
            .map(|item| item.entity);
    };
    items
        .iter()
        .filter_map(|item| {
            let offset = item.center - from.center;
            let along = offset.dot(direction);
            (along > 1.).then_some((item.entity, along + offset.perp_dot(direction).abs() * 3.))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|item| item.0)
        .or(current)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(in crate::survivors) fn navigate(
    mut commands: Commands,
    pads: Query<&Gamepad>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<State<Phase>>,
    settings: Res<hud::PausePage>,
    preview: Res<super::super::cards::PreviewHand>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    typing: Res<Typing>,
    mut armory: ResMut<Armory>,
    mut focus: ResMut<ControllerFocus>,
    buttons: Query<
        (
            Entity,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
        ),
        (With<Act>, With<Button>),
    >,
    parents: Query<&ChildOf>,
    mut scrolls: Query<(
        &Node,
        &ComputedNode,
        &UiGlobalTransform,
        &mut ScrollPosition,
    )>,
) {
    focus.activated = None;
    focus.keyboard_selected = false;
    let old = focus.focused;
    let eligible = *phase.get() == Phase::Menu
        && !settings.open
        && !preview.0
        && !loading.covering()
        && !typing.0;
    if !eligible || mouse.get_just_pressed().next().is_some() {
        focus.focused = None;
    } else {
        let items: Vec<_> = buttons
            .iter()
            .filter(|(_, node, _, visible)| visible.get() && node.size().min_element() > 0.)
            .map(|(entity, node, transform, _)| Item {
                entity,
                center: transform.translation,
                size: node.size(),
            })
            .collect();
        if !items.iter().any(|item| Some(item.entity) == focus.focused) {
            focus.focused = None;
        }
        let pad = pads.iter().find(|pad| {
            [
                GamepadButton::DPadUp,
                GamepadButton::DPadDown,
                GamepadButton::DPadLeft,
                GamepadButton::DPadRight,
                GamepadButton::South,
                GamepadButton::East,
                GamepadButton::Start,
            ]
            .into_iter()
            .any(|key| pad.just_pressed(key))
        });
        let pressed = |button| pad.is_some_and(|pad| pad.just_pressed(button));
        let key_select = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
        if pad.is_some() || keys.get_just_pressed().next().is_some() {
            if pressed(GamepadButton::Start) {
                // The shared Options handler owns this press later in the frame.
                focus.focused = None;
            } else if pressed(GamepadButton::East) {
                if armory.page == Page::Heroes && armory.manage_heroes {
                    armory.manage_heroes = false;
                } else if armory.page != Page::Door {
                    armory.open_page(Page::Door);
                }
                focus.focused = None;
            } else {
                let direction = [
                    (GamepadButton::DPadUp, KeyCode::ArrowUp, Vec2::NEG_Y),
                    (GamepadButton::DPadDown, KeyCode::ArrowDown, Vec2::Y),
                    (GamepadButton::DPadLeft, KeyCode::ArrowLeft, Vec2::NEG_X),
                    (GamepadButton::DPadRight, KeyCode::ArrowRight, Vec2::X),
                ]
                .into_iter()
                .find(|(button, key, _)| pressed(*button) || keys.just_pressed(*key));
                if let Some((_, _, direction)) = direction {
                    focus.focused = step(&items, focus.focused, direction);
                } else if pressed(GamepadButton::South) || (key_select && focus.focused.is_some()) {
                    // A first select establishes focus instead of firing an unseen action.
                    if focus.focused.is_none() {
                        focus.focused = step(&items, None, Vec2::Y);
                    } else {
                        focus.activated = focus.focused;
                        focus.keyboard_selected = key_select;
                    }
                }
            }
        }
        if old != focus.focused {
            if let Some(item) = items.iter().find(|item| Some(item.entity) == focus.focused) {
                let mut ancestor = item.entity;
                while let Ok(parent) = parents.get(ancestor) {
                    ancestor = parent.parent();
                    if let Ok((style, node, transform, mut scroll)) = scrolls.get_mut(ancestor) {
                        if style.overflow.y != OverflowAxis::Scroll {
                            continue;
                        }
                        let margin = 6. / node.inverse_scale_factor().max(0.001);
                        let top = transform.translation.y - node.size().y / 2. + margin;
                        let bottom = transform.translation.y + node.size().y / 2. - margin;
                        let item_top = item.center.y - item.size.y / 2.;
                        let item_bottom = item.center.y + item.size.y / 2.;
                        let delta = if item_top < top {
                            item_top - top
                        } else if item_bottom > bottom {
                            item_bottom - bottom
                        } else {
                            0.
                        };
                        scroll.0.y = (scroll.0.y + delta * node.inverse_scale_factor()).max(0.);
                        break;
                    }
                }
            }
        }
    }
    if old != focus.focused {
        if let Some(old) = old {
            if let Ok(mut entity) = commands.get_entity(old) {
                entity.remove::<Outline>();
            }
        }
        if let Some(entity) = focus.focused {
            commands.entity(entity).insert(Outline {
                width: Val::Px(2.),
                offset: Val::Px(3.),
                color: Color::WHITE,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directional_focus_follows_rows_columns_and_keeps_edges() {
        let items: Vec<_> = [(0., 0.), (100., 0.), (0., 100.), (100., 100.)]
            .into_iter()
            .enumerate()
            .map(|(index, (x, y))| Item {
                entity: Entity::from_bits(index as u64 + 1),
                center: Vec2::new(x, y),
                size: Vec2::splat(40.),
            })
            .collect();
        let first = Some(items[0].entity);
        assert_eq!(step(&items, None, Vec2::Y), first);
        assert_eq!(step(&items, first, Vec2::X), Some(items[1].entity));
        assert_eq!(step(&items, first, Vec2::Y), Some(items[2].entity));
        assert_eq!(step(&items, first, Vec2::NEG_Y), first);
        assert_eq!(step(&[], first, Vec2::Y), None);
    }
    #[test]
    fn controller_focus_reveals_content_and_activation_is_one_frame_and_modal_safe() {
        let mut world = World::new();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(State::new(Phase::Menu));
        world.init_resource::<hud::PausePage>();
        world.init_resource::<super::super::super::cards::PreviewHand>();
        world.init_resource::<crate::loading_screen::LoadingScreen>();
        world.init_resource::<Typing>();
        world.init_resource::<Armory>();
        world.init_resource::<ControllerFocus>();
        let pad = world.spawn(Gamepad::default()).id();
        let scroller = world
            .spawn((
                Node {
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ComputedNode {
                    size: Vec2::splat(200.),
                    inverse_scale_factor: 0.5,
                    ..default()
                },
                UiGlobalTransform::from_xy(100., 100.),
                ScrollPosition::default(),
            ))
            .id();
        let wrapper = world.spawn((Node::default(), ChildOf(scroller))).id();
        let button = world
            .spawn((
                Button,
                Act::Page(Page::Talents),
                ComputedNode {
                    size: Vec2::splat(40.),
                    ..default()
                },
                UiGlobalTransform::from_xy(100., 300.),
                InheritedVisibility::VISIBLE,
                ChildOf(wrapper),
            ))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(navigate);
        let press = |world: &mut World, schedule: &mut Schedule, key| {
            let mut pad = world.get_mut::<Gamepad>(pad).unwrap();
            pad.digital_mut().reset_all();
            pad.digital_mut().press(key);
            schedule.run(world);
        };
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().focused, Some(button));
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert!(world.get::<Outline>(button).is_some());
        assert_eq!(world.get::<ScrollPosition>(scroller).unwrap().0.y, 66.);
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().activated, Some(button));
        world.get_mut::<Gamepad>(pad).unwrap().digital_mut().clear();
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().activated, Some(button));
        assert!(world.resource::<ControllerFocus>().keyboard_selected);
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.resource_mut::<hud::PausePage>().open = true;
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        assert!(world.get::<Outline>(button).is_none());
        world.resource_mut::<hud::PausePage>().open = false;
        world.resource_mut::<Typing>().0 = true;
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
    }
}
