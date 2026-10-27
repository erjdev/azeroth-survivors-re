//! Reference-shaped talent trees: three illustrated panels and a separate spec/detail column.
use super::{
    campaign, class_trees, dbc_points, hud, Account, Act, Selection, TalentDetail, TalentInfo,
    CLASSES, GOLD,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const DIM: Color = Color::srgb(0.64, 0.58, 0.4);
const FRAME: Color = Color::srgb(0.48, 0.37, 0.12);
const ACTIVE: Color = Color::srgb(0.32, 0.8, 0.95);
const ICON: f32 = 36.0;

/// Keep provisional effects honest while showing the same inline purchase comparison.
pub(super) fn provisional_description(rank: u8, max: u8) -> (String, Vec<std::ops::Range<usize>>) {
    let rank = rank.min(max);
    let next = rank.saturating_add(1).min(max);
    let mut text = "Individual effect pending; provisional bonus: ".to_owned();
    let start = text.len();
    if rank < next {
        text.push_str(&format!("+{rank} » {next}% damage"));
    } else {
        text.push_str(&format!("+{rank}% damage"));
    }
    let end = text.len();
    text.push('.');
    (text, std::iter::once(start..end).collect())
}

/// Numeric effect values and their units; prose and spell names retain normal text color.
pub(super) fn milestone_value_ranges(text: &str) -> Option<Vec<std::ops::Range<usize>>> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut start = i;
        if i > 0 && matches!(bytes[i - 1], b'+' | b'-') {
            start -= 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
        for unit in [
            " percentage points",
            " seconds",
            " second",
            " targets",
            " target",
            " sec",
            "%",
        ] {
            if text[i..].starts_with(unit) {
                let end = i + unit.len();
                if !bytes.get(end).is_some_and(u8::is_ascii_alphabetic) {
                    i = end;
                    break;
                }
            }
        }
        ranges.push(start..i);
    }
    (!ranges.is_empty()).then_some(ranges)
}

#[cfg(test)]
mod milestone_value_tests {
    use super::*;

    #[test]
    fn values_keep_units_signs_and_decimals_without_coloring_prose() {
        for (text, expected) in [
            ("Consecration: +25% radius and duration", vec!["+25%"]),
            (
                "Critical chance: +5 percentage points",
                vec!["+5 percentage points"],
            ),
            (
                "Shock: +25% damage and +1 direct target",
                vec!["+25%", "+1"],
            ),
            (
                "Stuns for 1 sec (bosses: 0.3 sec)",
                vec!["1 sec", "0.3 sec"],
            ),
            ("Wolf: from 2 to 1.6 sec", vec!["2", "1.6 sec"]),
            (
                "Éclair: +2 targets for 3 seconds",
                vec!["+2 targets", "3 seconds"],
            ),
        ] {
            let ranges = milestone_value_ranges(text).unwrap();
            assert_eq!(
                ranges.iter().map(|r| &text[r.clone()]).collect::<Vec<_>>(),
                expected
            );
            assert!(ranges.windows(2).all(|r| r[0].end <= r[1].start));
        }
        assert!(milestone_value_ranges("Always critically strikes").is_none());
        for class in 0..9 {
            for tree in 0..3 {
                for at in [10, 20, 30] {
                    let text = survivors_core::talents::description(class, tree, at).unwrap();
                    if let Some(ranges) = milestone_value_ranges(text) {
                        for range in ranges {
                            assert!(!text[range].is_empty());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;

    #[test]
    fn rebuilt_trees_keep_offsets_with_the_rendered_hero_and_spec() {
        let mut account = crate::survivors::account::empty();
        let id = account.create("Scroll", "Preview", 0).unwrap();
        let mut app = App::new();
        app.insert_resource(account)
            .insert_resource(super::super::Armory {
                page: super::super::Page::Talents,
                ..default()
            })
            .init_resource::<ScrollMemory>()
            .add_systems(Update, remember_scroll);
        let old = app
            .world_mut()
            .spawn((ScrollKey(id, 0), ScrollPosition(Vec2::new(0., 250.))))
            .id();
        // Selection may already point at the next spec while the old tree still exists.
        app.world_mut().resource_mut::<Account>().characters[0].spec = 1;
        app.update();
        app.world_mut().despawn(old);
        app.world_mut()
            .spawn((ScrollKey(id, 1), ScrollPosition(Vec2::new(0., 420.))));
        app.update();
        let selection = Selection {
            character: Some(id),
            ..default()
        };
        let memory = app.world().resource::<ScrollMemory>();
        let mut account = app.world().resource::<Account>().clone();
        assert_eq!(memory.offset(&account, &selection).y, 420.);
        account.characters[0].spec = 0;
        assert_eq!(memory.offset(&account, &selection).y, 250.);
        let another = account.create("Another", "Hero", 0).unwrap();
        assert_eq!(
            memory.offset(
                &account,
                &Selection {
                    character: Some(another),
                    ..default()
                }
            ),
            Vec2::ZERO
        );
    }
}

/// Each rendered scroll surface owns its key, even while selection changes this frame.
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::survivors) struct ScrollKey(u32, u8);

#[derive(Resource, Default)]
pub(in crate::survivors) struct ScrollMemory(std::collections::BTreeMap<ScrollKey, Vec2>);

impl ScrollMemory {
    pub(super) fn offset(&self, account: &Account, selection: &Selection) -> Vec2 {
        selection
            .character
            .and_then(|id| account.character(id))
            .and_then(|hero| self.0.get(&ScrollKey(hero.id, hero.spec)))
            .copied()
            .unwrap_or_default()
    }
}

pub(in crate::survivors) fn remember_scroll(
    armory: Res<super::Armory>,
    account: Res<Account>,
    mut memory: ResMut<ScrollMemory>,
    surfaces: Query<(&ScrollKey, &ScrollPosition)>,
) {
    if armory.page != super::Page::Talents {
        return;
    }
    memory.0.retain(|key, _| account.character(key.0).is_some());
    for (key, position) in &surfaces {
        if position.0.is_finite() {
            memory.0.insert(*key, position.0);
        }
    }
}

#[derive(Component)]
pub(in crate::survivors) struct Backdrop(pub(super) String);
#[derive(Component)]
pub(in crate::survivors) struct BackdropDone;

#[derive(Component)]
pub(in crate::survivors) struct MilestoneIcon {
    class: usize,
    tab: u32,
    name: &'static str,
}

/// Themed vanilla spell art for Survivors-only Paladin milestones.
/// These identities select artwork only, never spell mechanics.
fn custom_milestone_art(class: usize, name: &str) -> Option<(u32, &'static str)> {
    if class != 1 {
        return None;
    }
    Some(match name {
        "Daybreak" => (20473, "Holy Shock"),
        "Light's Grace" => (19750, "Flash of Light"),
        "Hallowed Ground" => (26573, "Consecration"),
        "Ardent Defender" => (20925, "Holy Shield"),
        "Hammer of the Righteous" => (853, "Hammer of Justice"),
        "The Art of War" => (20271, "Judgement"),
        "Divine Storm" => (21084, "Seal of Righteousness"),
        _ => return None,
    })
}

#[cfg(test)]
mod milestone_art_tests {
    use super::*;

    #[test]
    fn custom_milestones_resolve_verified_spell_art_only_for_paladins() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let spells = benilla_formats::load_spell_catalog(&mut chain).unwrap();
        let mut count = 0;
        for tree in 0..3 {
            for at in [10, 20, 30] {
                let name = campaign::milestone_name(1, tree, at);
                if let Some((id, expected)) = custom_milestone_art(1, name) {
                    let spell = spells.get(id).expect(name);
                    assert_eq!(spell.name, expected);
                    let icon = spell.icon.as_ref().expect(name);
                    assert!(!chain.read(&format!("{icon}.blp")).expect(name).is_empty());
                    count += 1;
                } else {
                    assert!(["Divine Favor", "Seal of Command"].contains(&name));
                }
                for class in [0, 2, 3, 4, 5, 6, 7, 8] {
                    assert!(custom_milestone_art(class, name).is_none());
                }
            }
        }
        assert_eq!(count, 7);
        assert!(custom_milestone_art(1, "Unknown").is_none());
    }
}

pub(in crate::survivors) fn fill_art(
    mut commands: Commands,
    mut slots: Query<(Entity, &Backdrop, &mut ImageNode), Without<BackdropDone>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    milestones: Query<(Entity, &MilestoneIcon)>,
    talents: Option<Res<crate::ui_talent::Talents>>,
    spells: Option<Res<crate::ui_action::Spells>>,
) {
    if let (Some(talents), Some(spells)) = (talents, spells) {
        for (entity, milestone) in &milestones {
            let spell = talents
                .catalog
                .talents_in_tab(milestone.tab)
                .iter()
                .find_map(|talent| {
                    let id = talent.ranks[0];
                    spells
                        .catalog
                        .get(id)
                        .filter(|spell| spell.name == milestone.name)
                        .map(|_| id)
                })
                .or_else(|| custom_milestone_art(milestone.class, milestone.name).map(|art| art.0));
            if let Some(spell) = spell {
                commands
                    .entity(entity)
                    .remove::<MilestoneIcon>()
                    .insert(hud::SpellIcon(spell));
            } else {
                // Unknown milestones retain their readable text without invented artwork.
                commands.entity(entity).despawn();
            }
        }
    }
    let Some(assets) = assets.as_deref_mut() else {
        return;
    };
    for (entity, path, mut image) in &mut slots {
        if let Some(handle) = assets.sprite_texture(&path.0, &mut images) {
            image.image = handle;
        }
        commands.entity(entity).insert(BackdropDone);
    }
}

fn text(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, value, size, color));
}

fn panel(
    ui: &mut ChildSpawnerCommands,
    weight: f32,
    layout: &survivors_core::layout::Layout,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            flex_grow: weight,
            flex_basis: Val::Px(240.0),
            min_width: Val::Px(200.0),
            // Let long descriptions grow the shared scroll content. A viewport-sized
            // panel clips its own overflow before the outer scroll area can reach it.
            min_height: Val::Px(440.0),
            overflow: Overflow::clip(),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.027, 0.014)),
        BorderColor::all(FRAME),
    ))
    .with_children(|frame| {
        frame
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(layout.padding)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(layout.gap * 0.75),
                ..default()
            })
            .with_children(body);
    });
}

fn button(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: String,
    act: Act,
    selected: bool,
    body_size: f32,
) {
    ui.spawn((
        Button,
        act,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            min_height: Val::Px(42.0),
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(if selected { GOLD } else { FRAME }),
        BackgroundColor(if selected {
            Color::srgb(0.29, 0.22, 0.08)
        } else {
            Color::srgb(0.08, 0.025, 0.015)
        }),
    ))
    .with_children(|button| {
        text(button, font, label, body_size, GOLD);
    });
}

fn center(node: &benilla_formats::Talent) -> (f32, f32) {
    (12.5 + node.col as f32 * 25.0, 7.0 + node.row as f32 * 14.0)
}

fn connector(ui: &mut ChildSpawnerCommands, from: (f32, f32), to: (f32, f32), color: Color) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(from.0),
            top: Val::Percent(from.1.min(to.1)),
            width: Val::Px(2.0),
            height: Val::Percent((to.1 - from.1).abs()),
            ..default()
        },
        BackgroundColor(color),
    ));
    if (from.0 - to.0).abs() > 0.01 {
        ui.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(from.0.min(to.0)),
                top: Val::Percent(to.1),
                width: Val::Percent((to.0 - from.0).abs()),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(color),
        ));
    }
}

pub(super) fn build(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
    catalog: Option<&benilla_formats::TalentCatalog>,
    scroll: Vec2,
    layout: &survivors_core::layout::Layout,
) {
    let Some(hero) = selection.character.and_then(|id| account.character(id)) else {
        ui.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        })
        .with_children(|empty| {
            super::typography::section(empty, font, "Your hero's talents", layout.title_size + 4.0, GOLD);
            text(empty, font, "Create a hero to explore their three talent trees.", 16.0, Color::WHITE);
            text(empty, font, "Runs earn talent experience. Spend points across three saved specs and reset them for free.", 13.0, DIM);
            empty.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(320.0),
                ..default()
            }).with_children(|action| {
                button(action, font, "Create a hero".into(), Act::Page(super::Page::Heroes), false, layout.body_size);
            });
        });
        return;
    };
    let Some(catalog) = catalog else {
        text(ui, font, "Loading talent trees…", 16.0, GOLD);
        return;
    };
    let trees = class_trees(catalog, CLASSES[hero.class].id);
    if trees.len() != 3 {
        text(
            ui,
            font,
            "This class's talent trees are unavailable.",
            16.0,
            GOLD,
        );
        return;
    }
    let active = usize::from(hero.spec).min(2);
    let points = dbc_points(hero, active);
    let spent: u32 = points.iter().sum();
    let earned = campaign::talent_level(hero.talent_xp);
    let valid = account.talents_valid(hero);
    let unspent = campaign::unspent_talents(hero.talent_xp, spent);
    text(
        ui,
        font,
        format!(
            "{} · {} {} · three saved specs",
            CLASSES[hero.class].name, hero.first, hero.last
        ),
        13.0,
        DIM,
    );
    // Wrapping keeps the four-column talent grids readable on narrower aspect ratios.
    // Only this outer surface scrolls, so a wheel event cannot move two nested panels.
    ui.spawn(Node { width: Val::Percent(100.0), flex_grow: 1.0,
        flex_basis: Val::Px(0.0), min_height: Val::Px(0.0), ..default() })
    .with_children(|frame| {
    let viewport = frame.spawn((super::collection::CollectionScroll, ScrollKey(hero.id, hero.spec), RelativeCursorPosition::default(), ScrollPosition(scroll),
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), padding: UiRect::right(Val::Px(16.0)),
            flex_wrap: FlexWrap::Wrap, align_content: AlignContent::Start,
            column_gap: Val::Px(layout.gap * 1.25), row_gap: Val::Px(layout.gap * 1.25), overflow: Overflow::scroll_y(), ..default() }))
    .with_children(|row| {
        for (tree, (name, nodes)) in trees.iter().enumerate() {
            panel(row, 1.0, layout, |col| {
                col.spawn(Node { width: Val::Percent(100.0), align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|heading| {
                    if let Some(node) = nodes.first() {
                        heading.spawn((super::talent_heading::Icon(node.tab), ImageNode::default(),
                            Node { width: Val::Px(24.0), height: Val::Px(24.0), flex_shrink: 0.0, ..default() }));
                    }
                    heading.spawn((super::typography::DisplayHeading, hud::text_bundle(font, name, layout.title_size - 1.0, GOLD),
                        Node { flex_grow: 1.0, ..default() }));
                    text(heading, font, format!("{} {}", points[tree], if points[tree] == 1 { "point" } else { "points" }), layout.body_size, DIM);
                });
                col.spawn((Node { width: Val::Percent(100.0), aspect_ratio: Some(300.0 / 331.0), flex_shrink: 0.0,
                    overflow: Overflow::clip(), border_radius: BorderRadius::all(Val::Px(3.0)), ..default() }, BackgroundColor(Color::BLACK)))
                .with_children(|grid| {
                    let tabs = catalog.tabs_for_class(1, CLASSES[hero.class].id);
                    if let Some(tab) = tabs.iter().find(|tab| tab.name == *name) {
                        grid.spawn((super::talent_background::TreeBackdrop(tab.background.clone()),
                            ImageNode { color: Color::srgba(1.0, 1.0, 1.0, 0.8), ..default() },
                            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
                    }
                    for node in nodes {
                        if let Some(prerequisite) = nodes.iter().find(|other| other.id == node.prereq_talent) {
                            let met = u32::from(hero.dbc_rank(prerequisite.id)) > node.prereq_rank;
                            connector(grid, center(prerequisite), center(node), if met { GOLD } else { DIM });
                        }
                    }
                    for node in nodes {
                        let rank = hero.dbc_rank(node.id);
                        let max = node.max_rank() as u8;
                        let proposed: Vec<_> = nodes.iter().map(|other| (other.id, hero.dbc_rank(other.id).saturating_add(u8::from(other.id == node.id)))).collect();
                        let available = unspent > 0 && super::talent_ranks_ok(nodes, &proposed);
                        let dim = if rank > 0 || available { 1.0 } else { 0.4 };
                        let (x,y) = center(node);
                        // Scale the 36px icon with the 300px painted tree so rank labels
                        // cannot crowd the next row when four panels fit a narrower window.
                        grid.spawn((Button, Act::Talent(node.id), TalentInfo { class: hero.class, id: node.id, spell: node.ranks[0], rank, max, milestone: None },
                            Node { position_type: PositionType::Absolute, left: Val::Percent(x), top: Val::Percent(y),
                                width: Val::Percent(12.0), max_width: Val::Px(ICON), aspect_ratio: Some(1.0),
                                border: UiRect::all(Val::Px(2.0)), ..default() },
                            UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
                            BorderColor::all(if rank == max { GOLD } else if available { Color::srgb(0.3, 0.8, 0.3) } else { DIM }),
                            ImageNode { color: Color::srgba(1.0, 1.0, 1.0, dim), ..default() }, hud::SpellIcon(node.ranks[0]), hud::IconDim(dim)))
                        .with_children(|icon| {
                            icon.spawn((Node { position_type: PositionType::Absolute, bottom: Val::Px(-5.0), right: Val::Px(-2.0),
                                padding: UiRect::horizontal(Val::Px(3.0)), ..default() }, BackgroundColor(Color::BLACK)))
                            .with_children(|label| { text(label, font, format!("{rank}/{max}"), 10.0, if rank > 0 { GOLD } else { DIM }); });
                        });
                    }
                });
                for at in [10, 20, 30] {
                    col.spawn((Button, TalentInfo { class: hero.class, id: 0, spell: 0, rank: 0, max: 0, milestone: Some((tree, at)) },
                        Node { width: Val::Percent(100.0), min_height: Val::Px(22.0), column_gap: Val::Px(6.0), align_items: AlignItems::Center, ..default() }))
                    .with_children(|line| {
                        let name = campaign::milestone_name(hero.class, tree, at);
                        if let Some(node) = nodes.first() {
                            let dim = if valid && points[tree] >= at { 1.0 } else { 0.4 };
                            line.spawn((MilestoneIcon { class: hero.class, tab: node.tab, name }, hud::IconDim(dim),
                                ImageNode { color: Color::srgba(1.0, 1.0, 1.0, dim), ..default() },
                                Node { width: Val::Px(20.0), height: Val::Px(20.0), flex_shrink: 0.0, ..default() }));
                        }
                        text(line, font, format!("{at} points · {name}"), layout.body_size,
                            if valid && points[tree] >= at { ACTIVE } else { DIM });
                    });
                }
            });
        }
        panel(row, 0.86, layout, |side| {
            side.spawn(Node { width: Val::Percent(100.0), column_gap: Val::Px(4.0), ..default() }).with_children(|specs| {
                for spec in 0..3u8 {
                    let p = dbc_points(hero, usize::from(spec));
                    let spread = if p == [0;3] { "Empty".into() } else { format!("{}/{}/{}", p[0],p[1],p[2]) };
                    button(specs, font, format!("Spec {}\n{spread}", spec+1), Act::Spec(spec), hero.spec == spec, layout.body_size);
                }
            });
            super::typography::section(side, font, "Talent Points", layout.title_size + 4.0, GOLD);
            text(side, font, format!("{unspent} to spend"), layout.title_size + 7.0, GOLD);
            text(side, font, format!("{spent} of {earned} earned spent · 51 at most"), layout.body_size, DIM);
            if spent > earned {
                text(side, font, "This saved spec exceeds your earned points. Its talent effects are inactive until you refund points, reset the spec, or earn enough points.", layout.body_size, Color::srgb(1.0, 0.45, 0.35));
            }
            if spent <= earned && !valid {
                text(side, font, "This saved spec does not match the talent tree rules. Its effects are inactive. Refund invalid ranks or reset this spec to repair it.", layout.body_size, Color::srgb(1.0, 0.45, 0.35));
            }
            text(side, font, "Runs award talent experience for time survived, kills, bosses and victory. Later zones pay more. Respecs are free.", layout.body_size, DIM);
            side.spawn(Node { width: Val::Percent(100.0), ..default() }).with_children(|reset| {
                button(reset, font, "Reset this spec".into(), Act::Respec, false, layout.body_size);
            });
            text(side, font, "Point or use arrows / D-pad to inspect talents and milestones. Select spends a point. Right-click, Backspace or controller west refunds one. Each row needs five points per tier in earlier rows.", layout.body_size, DIM);
            side.spawn((TalentDetail::Title, hud::text_bundle(font, "Choose a talent to read its Survivors effect.", layout.body_size + 2.0, Color::WHITE)));
            side.spawn((TalentDetail::Effect, hud::text_bundle(font, "", layout.body_size + 2.0, Color::WHITE)));
            if hero.spent() > 0 && spent == 0 {
                text(side, font, "Spending on these trees returns the legacy spread to unspent points.", layout.body_size, DIM);
            }
        });
    }).id();
    super::scrollbar::spawn(frame, viewport);
    });
}
