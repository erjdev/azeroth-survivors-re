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
                Page::Collection => u8::from(armory.collection_inspecting),
                Page::DropTables => u8::from(armory.drop_inspecting),
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
    visible_center: bool,
}

fn step(items: &[Item], current: Option<Entity>, direction: Vec2) -> Option<Entity> {
    let Some(from) = items.iter().find(|item| Some(item.entity) == current) else {
        return items
            .iter()
            .filter(|item| item.visible_center)
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
            // Wide controls overlap nearby arrow columns even when their centers
            // are far apart; measure the perpendicular gap between their edges.
            let perpendicular = Vec2::new(direction.y.abs(), direction.x.abs());
            let reach = (from.size + item.size).dot(perpendicular) * 0.5;
            let gap = (offset.perp_dot(direction).abs() - reach).max(0.);
            (along > 1.).then_some((item.entity, gap > 0., along + gap * 3.))
        })
        .min_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)).then(a.0.cmp(&b.0)))
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
    (grid_items, equipment_details, achievement_pages, reward_pages, codex_pages, clips, roots): (
        Query<(), With<CollectionGridItem>>,
        Query<(&ComputedNode, &UiGlobalTransform), With<super::starter_equipment::Detail>>,
        Query<Entity, With<super::achievement_page::ScrollCategory>>,
        Query<Entity, With<super::reward_page::ScrollCategory>>,
        Query<(Entity, &collection::CompactScroll)>,
        Query<&CalculatedClip>,
        Query<&ComputedNode, With<ArmoryRoot>>,
    ),
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
                visible_center: clips
                    .get(entity)
                    .map_or(true, |clip| clip.clip.contains(transform.translation)),
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
            if matches!(
                armory.page,
                Page::Achievements | Page::Codex | Page::Rewards
            ) && [
                KeyCode::PageUp,
                KeyCode::PageDown,
                KeyCode::Home,
                KeyCode::End,
            ]
            .into_iter()
            .any(|key| keys.just_pressed(key))
            {
                let reading_pane = match armory.page {
                    Page::Achievements => achievement_pages.single().ok(),
                    Page::Rewards => reward_pages.single().ok(),
                    Page::Codex => codex_pages.iter().find_map(|(entity, pane)| {
                        matches!(pane, collection::CompactScroll::CodexColumn(1)).then_some(entity)
                    }),
                    _ => None,
                };
                if let Some(page) = reading_pane {
                    if let Ok((_, node, _, mut scroll)) = scrolls.get_mut(page) {
                        if node.size().y > 0.0 {
                            let viewport = node.size().y * node.inverse_scale_factor();
                            let maximum = ((node.content_size.y - node.size().y)
                                * node.inverse_scale_factor())
                            .max(0.0);
                            let target = if keys.just_pressed(KeyCode::Home) {
                                0.0
                            } else if keys.just_pressed(KeyCode::End) {
                                maximum
                            } else if keys.just_pressed(KeyCode::PageUp) {
                                scroll.0.y.clamp(0.0, maximum) - viewport * 0.8
                            } else {
                                scroll.0.y.clamp(0.0, maximum) + viewport * 0.8
                            };
                            scroll.0.y = target.clamp(0.0, maximum);
                        }
                    }
                }
            } else if pressed(GamepadButton::Start) {
                // The shared Options handler owns this press later in the frame.
                focus.focused = None;
                focus.talent = None;
                focus.action = None;
            } else if pressed(GamepadButton::East) {
                if armory.page != Page::Door {
                    armory.back(roots.single().is_ok_and(collection::compact_width));
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
                        let mut item_top = item.center.y - item.size.y / 2.;
                        let mut item_bottom = item.center.y + item.size.y / 2.;
                        // The starter's shared description is part of inspecting a slot.
                        if matches!(
                            focus.action.as_ref().map(|(_, action, _)| action),
                            Some(Act::OpenEquipment(_))
                        ) {
                            if let Ok((detail, transform)) = equipment_details.single() {
                                item_top =
                                    item_top.min(transform.translation.y - detail.size().y / 2.);
                                item_bottom =
                                    item_bottom.max(transform.translation.y + detail.size().y / 2.);
                            }
                        }
                        let delta = if item_top < top {
                            item_top - top
                        } else if item_bottom > bottom {
                            item_bottom - bottom
                        } else {
                            0.
                        };
                        let maximum = ((node.content_size.y - node.size().y)
                            * node.inverse_scale_factor())
                        .max(0.0);
                        scroll.0.y = (scroll.0.y.clamp(0.0, maximum)
                            + delta * node.inverse_scale_factor())
                        .clamp(0.0, maximum);
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
    #[test]
    fn reading_page_keys_reach_noninteractive_content_and_respect_overlays() {
        for page_kind in [Page::Achievements, Page::Codex, Page::Rewards] {
            let mut world = World::new();
            world.init_resource::<ButtonInput<MouseButton>>();
            world.init_resource::<ButtonInput<KeyCode>>();
            world.insert_resource(State::new(Phase::Menu));
            world.init_resource::<hud::PausePage>();
            world.init_resource::<super::super::super::cards::PreviewHand>();
            world.init_resource::<crate::loading_screen::LoadingScreen>();
            world.init_resource::<Typing>();
            world.insert_resource(Armory {
                page: page_kind,
                ..default()
            });
            world.init_resource::<ControllerFocus>();
            let page = world
                .spawn((
                    super::super::achievement_page::ScrollCategory(0),
                    super::super::reward_page::ScrollCategory(0),
                    collection::CompactScroll::CodexColumn(1),
                    Node {
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ComputedNode {
                        size: Vec2::splat(400.),
                        content_size: Vec2::new(400., 1400.),
                        inverse_scale_factor: 0.5,
                        ..default()
                    },
                    UiGlobalTransform::default(),
                    ScrollPosition::default(),
                ))
                .id();
            let catalog = world
                .spawn((
                    collection::CompactScroll::CodexColumn(0),
                    ScrollPosition(Vec2::new(0., 75.)),
                ))
                .id();
            let mut schedule = Schedule::default();
            schedule.add_systems(navigate);
            for (key, expected) in [
                (KeyCode::PageDown, 160.),
                (KeyCode::End, 500.),
                (KeyCode::PageDown, 500.),
                (KeyCode::PageUp, 340.),
                (KeyCode::Home, 0.),
                (KeyCode::PageUp, 0.),
            ] {
                world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
                world.resource_mut::<ButtonInput<KeyCode>>().press(key);
                schedule.run(&mut world);
                assert_eq!(world.get::<ScrollPosition>(page).unwrap().0.y, expected);
                assert!(world.resource::<ControllerFocus>().activated.is_none());
                assert_eq!(world.get::<ScrollPosition>(catalog).unwrap().0.y, 75.);
            }
            for guard in 0..4 {
                world.resource_mut::<hud::PausePage>().open = guard == 0;
                world.resource_mut::<Typing>().0 = guard == 1;
                world
                    .resource_mut::<super::super::super::cards::PreviewHand>()
                    .0 = guard == 2;
                world.resource_mut::<Armory>().page =
                    if guard == 3 { Page::Door } else { page_kind };
                world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::End);
                schedule.run(&mut world);
                assert_eq!(world.get::<ScrollPosition>(page).unwrap().0.y, 0.);
            }
            world.resource_mut::<Armory>().page = page_kind;
            world.get_mut::<ScrollPosition>(page).unwrap().0.y = 900.;
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::PageUp);
            schedule.run(&mut world);
            assert_eq!(world.get::<ScrollPosition>(page).unwrap().0.y, 340.);
        }
    }

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
                visible_center: true,
            })
            .collect();
        let first = Some(items[0].entity);
        assert_eq!(step(&items, None, Vec2::Y), first);
        assert_eq!(step(&items, first, Vec2::X), Some(items[1].entity));
        assert_eq!(step(&items, first, Vec2::Y), Some(items[2].entity));
        assert_eq!(step(&items, first, Vec2::NEG_Y), first);
        assert_eq!(step(&[], first, Vec2::Y), None);
        // Initial focus skips a category above the scrolled viewport, but explicit
        // directional navigation still reaches it and can reveal it.
        let mut clipped = items.clone();
        clipped[0].visible_center = false;
        assert_eq!(step(&clipped, None, Vec2::Y), Some(items[1].entity));
        assert_eq!(step(&clipped, Some(items[2].entity), Vec2::NEG_Y), first);
    }
    #[test]
    fn wide_feature_button_reaches_adjacent_arrows_before_distant_header() {
        let items: Vec<_> = [
            (Vec2::new(310., 550.), Vec2::new(550., 25.)),
            (Vec2::new(55., 500.), Vec2::splat(25.)),
            (Vec2::new(565., 500.), Vec2::splat(25.)),
            (Vec2::new(350., 150.), Vec2::new(130., 30.)),
            (Vec2::new(310., 650.), Vec2::new(550., 45.)),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (center, size))| Item {
            entity: Entity::from_bits(index as u64 + 1),
            center,
            size,
            visible_center: true,
        })
        .collect();
        let randomize = Some(items[0].entity);
        let left = Some(items[1].entity);
        let right = Some(items[2].entity);
        assert_eq!(step(&items, randomize, Vec2::NEG_Y), left);
        assert_eq!(step(&items, left, Vec2::X), right);
        assert_eq!(step(&items, right, Vec2::Y), randomize);
        assert_eq!(step(&items, Some(items[4].entity), Vec2::NEG_Y), randomize);
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
                    content_size: Vec2::new(200., 600.),
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
        // A starter equipment description below the icon must also clear the footer.
        world.entity_mut(button).insert(Act::OpenEquipment(2));
        world.spawn((
            super::super::starter_equipment::Detail,
            ComputedNode {
                size: Vec2::new(200., 60.),
                ..default()
            },
            UiGlobalTransform::from_xy(100., 360.),
            ChildOf(wrapper),
        ));
        *world.resource_mut::<ControllerFocus>() = ControllerFocus::default();
        world.get_mut::<ScrollPosition>(scroller).unwrap().0.y = 0.;
        press(&mut world, &mut schedule, GamepadButton::South);
        assert_eq!(world.get::<ScrollPosition>(scroller).unwrap().0.y, 101.);

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
    fn focused_starter_equipment_precedes_hover_and_survives_no_stale_entity() {
        use super::super::starter_equipment::{refresh, Detail, SlotInfo};
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let hovered = world
            .spawn((Interaction::Hovered, SlotInfo("Helm · Empty".into())))
            .id();
        let focused = world
            .spawn((
                Interaction::None,
                SlotInfo("Shoulders · Equipped item".into()),
            ))
            .id();
        let detail = world.spawn((Detail, Text::new(""))).id();
        world.insert_resource(ControllerFocus {
            focused: Some(focused),
            ..default()
        });
        world.run_system_once(refresh).unwrap();
        assert_eq!(
            world.get::<Text>(detail).unwrap().0,
            "Shoulders · Equipped item"
        );
        world.despawn(focused);
        world.run_system_once(refresh).unwrap();
        assert_eq!(world.get::<Text>(detail).unwrap().0, "Helm · Empty");
        world.despawn(hovered);
        world.run_system_once(refresh).unwrap();
        assert!(world
            .get::<Text>(detail)
            .unwrap()
            .0
            .starts_with("Point or focus"));
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
        assert_eq!(
            world.get::<TextSpan>(spans[0]).unwrap().0,
            "+15 » 30% duration"
        );
        assert_eq!(
            world.get::<TextSpan>(spans[1]).unwrap().0,
            " (Survivors conversion)"
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
        {
            let mut node = world.get_mut::<TalentInfo>(focused).unwrap();
            node.class = 8;
            node.id = 763;
        }
        schedule.run(&mut world);
        assert_eq!(world.get::<Text>(effect).unwrap().0, "Moonfire: ");
        let spans = world.get::<Children>(effect).unwrap();
        let expected = [
            "+2 » 4% direct and periodic damage",
            " and ",
            "+2 » 4 percentage points critical chance",
            "",
        ];
        assert_eq!(spans.len(), expected.len());
        for (index, child) in spans.iter().enumerate() {
            assert_eq!(world.get::<TextSpan>(child).unwrap().0, expected[index]);
            let color = if index % 2 == 0 {
                crate::survivors::hud::card_growth::GAIN
            } else {
                Color::WHITE
            };
            assert_eq!(world.get::<TextColor>(child).unwrap().0, color);
        }
        world.resource_mut::<ControllerFocus>().focused = Some(hovered);
        world.get_mut::<TalentInfo>(hovered).unwrap().milestone = Some((0, 10));
        schedule.run(&mut world);
        assert_eq!(
            world.get::<Text>(effect).unwrap().0,
            "Direct attack criticals bleed for "
        );
        let spans = world.get::<Children>(effect).unwrap();
        let expected = [
            "12%",
            " damage over ",
            "3 seconds",
            "; refreshes, does not stack",
        ];
        assert_eq!(spans.len(), expected.len());
        let mut reconstructed = world.get::<Text>(effect).unwrap().0.clone();
        for (index, child) in spans.iter().enumerate() {
            let span = &world.get::<TextSpan>(child).unwrap().0;
            assert_eq!(span, expected[index]);
            reconstructed.push_str(span);
            assert_eq!(
                world.get::<TextColor>(child).unwrap().0,
                if index % 2 == 0 {
                    crate::survivors::hud::card_growth::GAIN
                } else {
                    Color::WHITE
                }
            );
            assert_eq!(
                world.get::<TextFont>(child).unwrap().font,
                world.get::<TextFont>(effect).unwrap().font
            );
        }
        assert_eq!(
            reconstructed,
            survivors_core::talents::description(0, 0, 10).unwrap()
        );
        {
            let mut node = world.get_mut::<TalentInfo>(hovered).unwrap();
            node.class = 5;
            node.milestone = Some((0, 30));
        }
        schedule.run(&mut world);
        assert_eq!(
            world.get::<Text>(effect).unwrap().0,
            survivors_core::talents::description(5, 0, 30).unwrap()
        );
        assert_eq!(world.get::<TextColor>(effect).unwrap().0, Color::WHITE);
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
