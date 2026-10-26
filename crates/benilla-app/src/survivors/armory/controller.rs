//! Keyboard/controller focus uses the same guarded actions as pointer selection.
use super::*;

#[derive(Resource, Default)]
pub(in crate::survivors) struct ControllerFocus {
    focused: Option<Entity>,
    talent: Option<(Option<u32>, TalentKey)>,
    reward: Option<(Option<u32>, u8, Act)>,
    pub activated: Option<Entity>,
    pub refund: Option<Entity>,
    pub keyboard_selected: bool,
}

impl ControllerFocus {
    pub(super) fn focused(&self) -> Option<Entity> {
        self.focused
    }
}

/// Rank-independent identity survives the tree's entity replacement.
#[derive(Clone, Copy, PartialEq, Eq)]
struct TalentKey {
    class: usize,
    id: u32,
    milestone: Option<(usize, u32)>,
}

impl From<&TalentInfo> for TalentKey {
    fn from(info: &TalentInfo) -> Self {
        Self {
            class: info.class,
            id: info.id,
            milestone: info.milestone,
        }
    }
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
    selection: Option<Res<Selection>>,
    mut armory: ResMut<Armory>,
    mut focus: ResMut<ControllerFocus>,
    buttons: Query<
        (
            Entity,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&TalentInfo>,
            Option<&Act>,
        ),
        (Or<(With<Act>, With<TalentInfo>)>, With<Button>),
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
    focus.refund = None;
    focus.keyboard_selected = false;
    let old = focus.focused;
    let hero = selection.as_ref().and_then(|selection| selection.character);
    if armory.page != Page::Talents || focus.talent.is_some_and(|(owner, _)| owner != hero) {
        if focus.talent.is_some() {
            focus.focused = None;
        }
        focus.talent = None;
    }
    if focus.reward.as_ref().is_some_and(|(owner, category, _)| {
        armory.page != Page::Rewards || *owner != hero || *category != armory.reward_page
    }) {
        focus.focused = None;
        focus.reward = None;
    }
    let eligible = *phase.get() == Phase::Menu
        && !settings.open
        && !preview.0
        && !loading.covering()
        && !typing.0;
    if !eligible || mouse.get_just_pressed().next().is_some() {
        focus.focused = None;
        focus.talent = None;
        focus.reward = None;
    } else {
        let items: Vec<_> = buttons
            .iter()
            .filter(|(_, node, _, visible, _, _)| visible.get() && node.size().min_element() > 0.)
            .map(|(entity, node, transform, _, _, _)| Item {
                entity,
                center: transform.translation,
                size: node.size(),
            })
            .collect();
        if !items.iter().any(|item| Some(item.entity) == focus.focused) {
            focus.focused = focus
                .talent
                .and_then(|(_, key)| {
                    items
                        .iter()
                        .find(|item| {
                            buttons
                                .get(item.entity)
                                .ok()
                                .and_then(|(_, _, _, _, info, _)| info)
                                .is_some_and(|info| TalentKey::from(info) == key)
                        })
                        .map(|item| item.entity)
                })
                .or_else(|| {
                    let (_, _, action) = focus.reward.as_ref()?;
                    let mut matches = items.iter().filter(|item| {
                        buttons
                            .get(item.entity)
                            .ok()
                            .and_then(|(_, _, _, _, _, act)| act)
                            .is_some_and(|act| act == action)
                    });
                    let candidate = matches.next()?.entity;
                    matches.next().is_none().then_some(candidate)
                });
            // A missing/disabled action is not a promise to focus it if it returns later.
            // Keep the identity only while its replacement is waiting for layout.
            if focus.focused.is_none()
                && !buttons.is_empty()
                && focus.reward.as_ref().is_some_and(|(_, _, action)| {
                    !buttons
                        .iter()
                        .any(|(_, _, _, visible, _, act)| visible.get() && act == Some(action))
                })
            {
                focus.reward = None;
            }
        }
        let pad = pads.iter().find(|pad| {
            [
                GamepadButton::DPadUp,
                GamepadButton::DPadDown,
                GamepadButton::DPadLeft,
                GamepadButton::DPadRight,
                GamepadButton::South,
                GamepadButton::West,
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
                focus.talent = None;
                focus.reward = None;
            } else if pressed(GamepadButton::East) {
                if armory.page == Page::Heroes && armory.manage_heroes {
                    armory.manage_heroes = false;
                } else if armory.page != Page::Door {
                    armory.open_page(Page::Door);
                }
                focus.focused = None;
                focus.talent = None;
                focus.reward = None;
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
                } else if pressed(GamepadButton::West) || keys.just_pressed(KeyCode::Backspace) {
                    // Refund requests are filtered to talent actions by the shared handler.
                    focus.refund = focus.focused;
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
        if let Some(entity) = focus.focused {
            focus.reward = (armory.page == Page::Rewards)
                .then(|| {
                    buttons
                        .get(entity)
                        .ok()
                        .and_then(|(_, _, _, _, _, act)| act)
                        .map(|act| (hero, armory.reward_page, act.clone()))
                })
                .flatten();
            focus.talent = (armory.page == Page::Talents)
                .then(|| {
                    buttons
                        .get(entity)
                        .ok()
                        .and_then(|(_, _, _, _, info, _)| info)
                        .map(|info| (hero, TalentKey::from(info)))
                })
                .flatten();
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
        press(&mut world, &mut schedule, GamepadButton::West);
        assert_eq!(world.resource::<ControllerFocus>().refund, Some(button));
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        world.get_mut::<Gamepad>(pad).unwrap().digital_mut().clear();
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Backspace);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().refund, Some(button));
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.resource_mut::<hud::PausePage>().open = true;
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        assert!(world.get::<Outline>(button).is_none());
        world.resource_mut::<hud::PausePage>().open = false;
        world.resource_mut::<Typing>().0 = true;
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
    }
    #[test]
    fn talent_focus_survives_rebuild_and_layout_without_replaying_actions() {
        let mut world = World::new();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(State::new(Phase::Menu));
        world.init_resource::<hud::PausePage>();
        world.init_resource::<super::super::super::cards::PreviewHand>();
        world.init_resource::<crate::loading_screen::LoadingScreen>();
        world.init_resource::<Typing>();
        world.init_resource::<Armory>();
        world.resource_mut::<Armory>().page = Page::Talents;
        world.init_resource::<ControllerFocus>();
        let spawn = |world: &mut World, rank, size| {
            world
                .spawn((
                    Button,
                    Act::Talent(150),
                    TalentInfo {
                        class: 0,
                        id: 150,
                        spell: 0,
                        rank,
                        max: 5,
                        milestone: None,
                    },
                    ComputedNode {
                        size: Vec2::splat(size),
                        ..default()
                    },
                    UiGlobalTransform::from_xy(100., 100.),
                    InheritedVisibility::VISIBLE,
                ))
                .id()
        };
        let original = spawn(&mut world, 1, 40.);
        let mut schedule = Schedule::default();
        schedule.add_systems(navigate);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, Some(original));
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.despawn(original);
        let replacement = spawn(&mut world, 2, 0.);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world.get_mut::<ComputedNode>(replacement).unwrap().size = Vec2::splat(40.);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ControllerFocus>().focused,
            Some(replacement)
        );
        assert!(world.get::<Outline>(replacement).is_some());
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Backspace);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ControllerFocus>().refund,
            Some(replacement)
        );
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.resource_mut::<Armory>().page = Page::Door;
        schedule.run(&mut world);
        world.resource_mut::<Armory>().page = Page::Talents;
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
    }

    #[test]
    fn reward_focus_survives_rebuild_but_not_category_or_disabled_controls() {
        let mut world = World::new();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(State::new(Phase::Menu));
        world.init_resource::<hud::PausePage>();
        world.init_resource::<super::super::super::cards::PreviewHand>();
        world.init_resource::<crate::loading_screen::LoadingScreen>();
        world.init_resource::<Typing>();
        world.init_resource::<Armory>();
        world.resource_mut::<Armory>().page = Page::Rewards;
        world.init_resource::<ControllerFocus>();
        let spawn = |world: &mut World, _rank, size| {
            world
                .spawn((
                    Button,
                    Act::Reward(0),
                    ComputedNode {
                        size: Vec2::splat(size),
                        ..default()
                    },
                    UiGlobalTransform::from_xy(100., 100.),
                    InheritedVisibility::VISIBLE,
                ))
                .id()
        };
        let original = spawn(&mut world, 1, 40.);
        let mut schedule = Schedule::default();
        schedule.add_systems(navigate);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, Some(original));
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.despawn(original);
        let replacement = spawn(&mut world, 2, 0.);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world.get_mut::<ComputedNode>(replacement).unwrap().size = Vec2::splat(40.);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ControllerFocus>().focused,
            Some(replacement)
        );
        assert!(world.get::<Outline>(replacement).is_some());
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Backspace);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ControllerFocus>().refund,
            Some(replacement)
        );
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.resource_mut::<Armory>().reward_page = 1;
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world.resource_mut::<Armory>().reward_page = 0;
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        schedule.run(&mut world);
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        let _navigation = world
            .spawn((
                Button,
                Act::Page(Page::Door),
                ComputedNode {
                    size: Vec2::splat(40.),
                    ..default()
                },
                UiGlobalTransform::from_xy(300., 300.),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        world.entity_mut(replacement).remove::<Act>();
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        assert!(world.resource::<ControllerFocus>().reward.is_none());
        world.entity_mut(replacement).insert(Act::Reward(0));
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        schedule.run(&mut world);
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        schedule.run(&mut world);
        assert!(world.resource::<ControllerFocus>().reward.is_none());
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
    }

    #[test]
    fn focused_talent_details_precede_pointer_hover_without_spending_points() {
        let mut world = World::new();
        let hovered = world
            .spawn((
                Interaction::Hovered,
                TalentInfo {
                    class: 0,
                    id: 0,
                    spell: 0,
                    rank: 0,
                    max: 5,
                    milestone: None,
                },
            ))
            .id();
        let focused = world
            .spawn((
                Interaction::None,
                TalentInfo {
                    class: 0,
                    id: 150,
                    spell: 0,
                    rank: 1,
                    max: 5,
                    milestone: None,
                },
            ))
            .id();
        world.insert_resource(ControllerFocus {
            focused: Some(focused),
            ..default()
        });
        let effect = world
            .spawn((TalentDetail::Effect, Text::new(""), TextColor(Color::WHITE)))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(super::super::talent_details);
        schedule.run(&mut world);
        let definition = survivors_core::talents::talent_definition(0, 150).unwrap();
        assert!(world
            .get::<Text>(effect)
            .unwrap()
            .0
            .contains(&definition.rank_preview(1)));
        assert_eq!(
            world.get::<TextColor>(effect).unwrap().0,
            Color::srgb(0.45, 1., 0.45)
        );
        assert_eq!(world.get::<TalentInfo>(focused).unwrap().rank, 1);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        world.resource_mut::<ControllerFocus>().focused = Some(hovered);
        world.get_mut::<TalentInfo>(hovered).unwrap().milestone = Some((0, 10));
        schedule.run(&mut world);
        assert_eq!(
            world.get::<Text>(effect).unwrap().0,
            survivors_core::talents::description(0, 0, 10).unwrap()
        );
        world.resource_mut::<ControllerFocus>().focused = None;
        world.get_mut::<TalentInfo>(hovered).unwrap().milestone = None;
        schedule.run(&mut world);
        assert!(world
            .get::<Text>(effect)
            .unwrap()
            .0
            .contains("Individual effect pending"));
        assert_eq!(world.get::<TextColor>(effect).unwrap().0, Color::WHITE);
    }
}
