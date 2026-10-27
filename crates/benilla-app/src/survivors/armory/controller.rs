//! Keyboard/controller focus uses the same guarded actions as pointer selection.
use super::*;

#[derive(Resource, Default)]
pub(in crate::survivors) struct ControllerFocus {
    focused: Option<Entity>,
    talent: Option<(Option<u32>, TalentKey)>,
    action: Option<(ActionContext, Act, bool)>,
    pub activated: Option<Entity>,
    pub refund: Option<Entity>,
    pub keyboard_selected: bool,
}

impl ControllerFocus {
    pub(super) fn focused(&self) -> Option<Entity> {
        self.focused
    }
}

#[derive(Component)]
pub(in crate::survivors) struct CollectionGridItem;

/// Secondary selection never activates purchases, appearance controls or navigation.
pub(super) fn secondary_action(act: &Act) -> bool {
    matches!(
        act,
        Act::Talent(_) | Act::Learn(_) | Act::InspectPiece(_) | Act::InspectSupplemental(_)
    )
}

/// An action belongs to one page, hero and category, even when its entity is replaced.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ActionContext {
    page: Page,
    hero: Option<u32>,
    class: Option<usize>,
    section: u8,
}

impl ActionContext {
    fn new(armory: &Armory, selection: Option<&Selection>) -> Self {
        Self {
            page: armory.page,
            hero: selection.and_then(|s| s.character),
            class: selection.and_then(|s| s.class),
            section: match armory.page {
                Page::Rewards => armory.reward_page,
                Page::Achievements => armory.achieve,
                Page::Heroes => u8::from(armory.manage_heroes),
                _ => 0,
            },
        }
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
    grid_items: Query<(), With<CollectionGridItem>>,
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
    let context = ActionContext::new(&armory, selection.as_deref());
    let hero = context.hero;
    if armory.page != Page::Talents || focus.talent.is_some_and(|(owner, _)| owner != hero) {
        if focus.talent.is_some() {
            focus.focused = None;
        }
        focus.talent = None;
    }
    if focus
        .action
        .as_ref()
        .is_some_and(|(owner, _, _)| *owner != context)
    {
        focus.focused = None;
        focus.action = None;
    }
    let eligible = *phase.get() == Phase::Menu
        && !settings.open
        && !preview.0
        && !loading.covering()
        && !typing.0;
    if !eligible || mouse.get_just_pressed().next().is_some() {
        focus.focused = None;
        focus.talent = None;
        focus.action = None;
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
                    let (_, action, grid) = focus.action.as_ref()?;
                    let mut matches = items.iter().filter(|item| {
                        buttons
                            .get(item.entity)
                            .ok()
                            .and_then(|(_, _, _, _, _, act)| act)
                            .is_some_and(|act| act == action)
                            && grid_items.contains(item.entity) == *grid
                    });
                    let candidate = matches.next()?.entity;
                    matches.next().is_none().then_some(candidate)
                });
            // A missing/disabled action is not a promise to focus it if it returns later.
            // Keep the identity only while its replacement is waiting for layout.
            if focus.focused.is_none()
                && !buttons.is_empty()
                && focus.action.as_ref().is_some_and(|(_, action, grid)| {
                    buttons
                        .iter()
                        .filter(|(entity, _, _, visible, _, act)| {
                            visible.get()
                                && act == &Some(action)
                                && grid_items.contains(*entity) == *grid
                        })
                        .count()
                        != 1
                })
            {
                focus.action = None;
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
                focus.action = None;
            } else if pressed(GamepadButton::East) {
                if armory.page == Page::Heroes && armory.manage_heroes {
                    armory.manage_heroes = false;
                } else if armory.page != Page::Door {
                    armory.open_page(Page::Door);
                }
                focus.focused = None;
                focus.talent = None;
                focus.action = None;
            } else {
                let direction = [
                    (GamepadButton::DPadUp, KeyCode::ArrowUp, Vec2::NEG_Y),
                    (GamepadButton::DPadDown, KeyCode::ArrowDown, Vec2::Y),
                    (GamepadButton::DPadLeft, KeyCode::ArrowLeft, Vec2::NEG_X),
                    (GamepadButton::DPadRight, KeyCode::ArrowRight, Vec2::X),
                ]
                .into_iter()
                .find(|(button, key, _)| pressed(*button) || keys.just_pressed(*key));
                let had_focus = focus.focused.is_some();
                if let Some((_, _, direction)) = direction {
                    focus.focused = step(&items, focus.focused, direction);
                }
                if direction.is_none()
                    && (pressed(GamepadButton::West) || keys.just_pressed(KeyCode::Backspace))
                {
                    // The shared handler filters this to talent refunds and item equip toggles.
                    focus.refund = focus.focused;
                } else if pressed(GamepadButton::South) || (key_select && focus.focused.is_some()) {
                    // A first select establishes focus instead of firing an unseen action.
                    // Coalesced navigation and selection target the newly focused control.
                    // Consume Enter even when first establishing focus so it cannot start a run.
                    focus.keyboard_selected = key_select;
                    if !had_focus {
                        if focus.focused.is_none() {
                            focus.focused = step(&items, None, Vec2::Y);
                        }
                    } else {
                        focus.activated = focus.focused;
                    }
                }
            }
        }
        if let Some(entity) = focus.focused {
            focus.action = buttons
                .get(entity)
                .ok()
                .filter(|(_, _, _, _, info, _)| info.is_none())
                .and_then(|(_, _, _, _, _, act)| act)
                .map(|act| (context, act.clone(), grid_items.contains(entity)));
            // Inspection follows navigation, without dispatching an equipment action.
            if armory.page == Page::Collection {
                match focus.action.as_ref().map(|(_, action, _)| action) {
                    Some(Act::InspectPiece(index)) => {
                        let index = (*index).min(campaign::PIECES - 1);
                        if armory.piece != index || armory.supplemental_piece.is_some() {
                            armory.piece = index;
                            armory.supplemental_piece = None;
                        }
                    }
                    Some(Act::InspectSupplemental(key))
                        if armory.supplemental_piece.as_ref() != Some(key) =>
                    {
                        armory.supplemental_piece = Some(key.clone());
                    }
                    _ => {}
                }
            }
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
    fn page_actions_retain_focus_without_replaying_and_clear_for_hero_or_page_changes() {
        for (page, action) in [
            (Page::Heroes, Act::Feature(0, 1)),
            (Page::Collection, Act::InspectPiece(2)),
            (Page::Codex, Act::Codex(2)),
            (Page::DropTables, Act::BrowseDrops(2)),
            (Page::Achievements, Act::Achieve(2)),
            (Page::Door, Act::Zone(2)),
            (Page::Talents, Act::Spec(1)),
        ] {
            let mut world = World::new();
            world.init_resource::<ButtonInput<MouseButton>>();
            world.init_resource::<ButtonInput<KeyCode>>();
            world.insert_resource(State::new(Phase::Menu));
            world.init_resource::<hud::PausePage>();
            world.init_resource::<super::super::super::cards::PreviewHand>();
            world.init_resource::<crate::loading_screen::LoadingScreen>();
            world.init_resource::<Typing>();
            world.insert_resource(Armory { page, ..default() });
            world.insert_resource(Selection {
                character: Some(1),
                class: Some(0),
                ..default()
            });
            world.init_resource::<ControllerFocus>();
            let spawn = |world: &mut World, size| {
                world
                    .spawn((
                        Button,
                        action.clone(),
                        ComputedNode {
                            size: Vec2::splat(size),
                            ..default()
                        },
                        UiGlobalTransform::from_xy(100., 100.),
                        InheritedVisibility::VISIBLE,
                    ))
                    .id()
            };
            let original = spawn(&mut world, 40.);
            let mut schedule = Schedule::default();
            schedule.add_systems(navigate);
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::ArrowDown);
            schedule.run(&mut world);
            assert_eq!(world.resource::<ControllerFocus>().focused, Some(original));
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world.despawn(original);
            let replacement = spawn(&mut world, 0.);
            schedule.run(&mut world);
            assert_eq!(world.resource::<ControllerFocus>().focused, None);
            world.get_mut::<ComputedNode>(replacement).unwrap().size = Vec2::splat(40.);
            schedule.run(&mut world);
            assert_eq!(
                world.resource::<ControllerFocus>().focused,
                Some(replacement)
            );
            assert!(world.get::<Outline>(replacement).is_some());
            assert!(world.resource::<ControllerFocus>().activated.is_none());
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Space);
            schedule.run(&mut world);
            assert_eq!(
                world.resource::<ControllerFocus>().activated,
                Some(replacement)
            );
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            schedule.run(&mut world);
            assert!(world.resource::<ControllerFocus>().activated.is_none());
            world.resource_mut::<Selection>().character = Some(2);
            schedule.run(&mut world);
            assert!(world.resource::<ControllerFocus>().focused.is_none());
            world.resource_mut::<Selection>().character = Some(1);
            schedule.run(&mut world);
            assert!(world.resource::<ControllerFocus>().focused.is_none());
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::ArrowDown);
            schedule.run(&mut world);
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world.resource_mut::<Armory>().page = Page::Notes;
            schedule.run(&mut world);
            assert!(world.resource::<ControllerFocus>().focused.is_none());
            assert!(world.resource::<ControllerFocus>().action.is_none());
        }
    }

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
    fn collection_grid_focus_survives_equipped_sidebar_duplicate() {
        let mut world = World::new();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(State::new(Phase::Menu));
        world.init_resource::<hud::PausePage>();
        world.init_resource::<super::super::super::cards::PreviewHand>();
        world.init_resource::<crate::loading_screen::LoadingScreen>();
        world.init_resource::<Typing>();
        world.init_resource::<Armory>();
        world.resource_mut::<Armory>().page = Page::Collection;
        world.init_resource::<ControllerFocus>();
        let spawn = |world: &mut World, grid, size| {
            let mut entity = world.spawn((
                Button,
                Act::InspectSupplemental("poor:hunter:hands".into()),
                ComputedNode {
                    size: Vec2::splat(size),
                    ..default()
                },
                UiGlobalTransform::from_xy(100., 100.),
                InheritedVisibility::VISIBLE,
            ));
            if grid {
                entity.insert(CollectionGridItem);
            }
            entity.id()
        };
        let original = spawn(&mut world, true, 40.);
        let mut schedule = Schedule::default();
        schedule.add_systems(navigate);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, Some(original));
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.despawn(original);
        let sidebar = spawn(&mut world, false, 40.);
        let replacement = spawn(&mut world, true, 0.);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
        world.get_mut::<ComputedNode>(replacement).unwrap().size = Vec2::splat(40.);
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ControllerFocus>().focused,
            Some(replacement)
        );
        assert!(world.get::<Outline>(replacement).is_some());
        assert!(world.get::<Outline>(sidebar).is_none());
        assert_eq!(
            world.resource::<Armory>().supplemental_piece.as_deref(),
            Some("poor:hunter:hands")
        );
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
        world.resource_mut::<ButtonInput<KeyCode>>().clear();
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        let legacy = world
            .spawn((
                Button,
                CollectionGridItem,
                Act::InspectPiece(34),
                ComputedNode {
                    size: Vec2::splat(40.),
                    ..default()
                },
                UiGlobalTransform::from_xy(200., 100.),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        schedule.run(&mut world);
        assert_eq!(world.resource::<ControllerFocus>().focused, Some(legacy));
        assert_eq!(world.resource::<Armory>().piece, 34);
        assert!(world.resource::<Armory>().supplemental_piece.is_none());
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        assert_eq!(world.resource::<ControllerFocus>().refund, None);
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        world.resource_mut::<hud::PausePage>().open = true;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Armory>().piece, 34);
        assert!(world.resource::<Armory>().supplemental_piece.is_none());
        assert_eq!(world.resource::<ControllerFocus>().focused, None);
    }

    #[test]
    fn secondary_selection_is_limited_to_inspected_gear_and_talent_refunds() {
        for act in [
            Act::InspectPiece(0),
            Act::InspectSupplemental("poor:hunter:hands".into()),
            Act::Talent(150),
            Act::Learn(0),
        ] {
            assert!(secondary_action(&act));
        }
        for act in [
            Act::Reward(0),
            Act::Page(Page::Door),
            Act::WearPiece(0, Some(0)),
            Act::ShowPiece(0, super::super::super::equipment::Look::Hidden),
            Act::ShowSupplemental("poor:hunter:hands".into()),
        ] {
            assert!(!secondary_action(&act));
        }
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
    fn coalesced_navigation_and_selection_activate_the_destination_once() {
        for keyboard in [true, false] {
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
            let mut buttons = Vec::new();
            for (index, x) in [0., 100.].into_iter().enumerate() {
                buttons.push(
                    world
                        .spawn((
                            Button,
                            Act::Tabard(index as u8),
                            ComputedNode {
                                size: Vec2::splat(40.),
                                ..default()
                            },
                            UiGlobalTransform::from_xy(x, 100.),
                            InheritedVisibility::VISIBLE,
                        ))
                        .id(),
                );
            }
            let mut schedule = Schedule::default();
            schedule.add_systems(navigate);
            let press = |world: &mut World| {
                if keyboard {
                    let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
                    keys.reset_all();
                    keys.press(KeyCode::ArrowRight);
                    keys.press(KeyCode::Enter);
                } else {
                    let mut gamepad = world.get_mut::<Gamepad>(pad).unwrap();
                    gamepad.digital_mut().reset_all();
                    gamepad.digital_mut().press(GamepadButton::DPadRight);
                    gamepad.digital_mut().press(GamepadButton::South);
                }
            };
            press(&mut world);
            schedule.run(&mut world);
            let focus = world.resource::<ControllerFocus>();
            assert_eq!(focus.focused, Some(buttons[0]));
            assert_eq!(
                focus.activated, None,
                "first gesture only establishes focus"
            );
            assert_eq!(
                focus.keyboard_selected, keyboard,
                "consume the initial Enter"
            );
            press(&mut world);
            schedule.run(&mut world);
            let focus = world.resource::<ControllerFocus>();
            assert_eq!(focus.focused, Some(buttons[1]));
            assert_eq!(focus.activated, Some(buttons[1]));
            assert_eq!(focus.keyboard_selected, keyboard);
            world.resource_mut::<ButtonInput<KeyCode>>().clear();
            world.get_mut::<Gamepad>(pad).unwrap().digital_mut().clear();
            schedule.run(&mut world);
            assert_eq!(world.resource::<ControllerFocus>().activated, None);
            assert!(!world.resource::<ControllerFocus>().keyboard_selected);
            world.resource_mut::<hud::PausePage>().open = true;
            press(&mut world);
            schedule.run(&mut world);
            assert_eq!(world.resource::<ControllerFocus>().activated, None);
            assert_eq!(world.resource::<ControllerFocus>().focused, None);
        }
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
        assert!(world.resource::<ControllerFocus>().action.is_none());
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
        assert!(world.resource::<ControllerFocus>().action.is_none());
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
        assert_eq!(world.get::<Text>(effect).unwrap().0, "Retaliation: ");
        let spans = world.get::<Children>(effect).unwrap();
        assert_eq!(world.get::<TextSpan>(spans[0]).unwrap().0, "+15 » 30%");
        assert_eq!(
            world.get::<TextSpan>(spans[1]).unwrap().0,
            " duration (Survivors conversion)"
        );
        assert_eq!(world.get::<TextColor>(effect).unwrap().0, Color::WHITE);
        assert_eq!(
            world.get::<TextColor>(spans[0]).unwrap().0,
            crate::survivors::hud::card_growth::GAIN
        );
        assert_eq!(world.get::<TextColor>(spans[1]).unwrap().0, Color::WHITE);
        for child in spans.iter() {
            assert_eq!(
                world.get::<TextFont>(child).unwrap().font_size,
                world.get::<TextFont>(effect).unwrap().font_size
            );
            assert_eq!(
                world.get::<TextFont>(child).unwrap().font,
                world.get::<TextFont>(effect).unwrap().font
            );
        }
        let children = spans.to_vec();
        schedule.run(&mut world);
        assert_eq!(world.get::<Children>(effect).unwrap().to_vec(), children);
        assert_eq!(world.get::<TalentInfo>(focused).unwrap().rank, 1);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        world.resource_mut::<ControllerFocus>().focused = Some(hovered);
        world.get_mut::<TalentInfo>(hovered).unwrap().milestone = Some((0, 10));
        schedule.run(&mut world);
        assert_eq!(
            world.get::<Text>(effect).unwrap().0,
            survivors_core::talents::description(0, 0, 10).unwrap()
        );
        assert!(world
            .get::<Children>(effect)
            .is_none_or(|children| children.is_empty()));
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
