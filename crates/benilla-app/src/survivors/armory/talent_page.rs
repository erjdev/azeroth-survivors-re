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

pub(in crate::survivors) fn fill_art(
    mut commands: Commands,
    mut slots: Query<(Entity, &Backdrop, &mut ImageNode), Without<BackdropDone>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
) {
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

fn panel(ui: &mut ChildSpawnerCommands, weight: f32, body: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn((
        Node {
            flex_grow: weight,
            flex_basis: Val::Px(240.0),
            min_width: Val::Px(200.0),
            height: Val::Percent(100.0),
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
                min_height: Val::Px(0.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
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
        text(button, font, label, 12.0, GOLD);
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
) {
    let Some(hero) = selection.character.and_then(|id| account.character(id)) else {
        text(
            ui,
            font,
            "Create a hero before spending talents.",
            16.0,
            GOLD,
        );
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
    ui.spawn((super::collection::CollectionScroll, ScrollKey(hero.id, hero.spec), RelativeCursorPosition::default(), ScrollPosition(scroll),
        Node { width: Val::Percent(100.0), flex_grow: 1.0, flex_basis: Val::Px(0.0), min_height: Val::Px(0.0),
            flex_wrap: FlexWrap::Wrap, align_content: AlignContent::Start,
            column_gap: Val::Px(10.0), row_gap: Val::Px(10.0), overflow: Overflow::scroll_y(), ..default() }))
    .with_children(|row| {
        for (tree, (name, nodes)) in trees.iter().enumerate() {
            panel(row, 1.0, |col| {
                text(col, font, format!("{name}   {} {}", points[tree], if points[tree] == 1 { "point" } else { "points" }), 17.0, GOLD);
                col.spawn((Node { width: Val::Percent(100.0), flex_grow: 1.0, flex_basis: Val::Px(0.0), min_height: Val::Px(300.0),
                    overflow: Overflow::clip(), border_radius: BorderRadius::all(Val::Px(3.0)), ..default() }, BackgroundColor(Color::BLACK)))
                .with_children(|grid| {
                    let tabs = catalog.tabs_for_class(1, CLASSES[hero.class].id);
                    if let Some(tab) = tabs.iter().find(|tab| tab.name == *name) {
                        // The 1.12 backgrounds form a 320x384 image from 256/64 by 256/128 tiles.
                        for (tile, left, top, width, height) in [
                            ("TopLeft", 0.0, 0.0, 80.0, 200.0/3.0), ("TopRight", 80.0, 0.0, 20.0, 200.0/3.0),
                            ("BottomLeft", 0.0, 200.0/3.0, 80.0, 100.0/3.0), ("BottomRight", 80.0, 200.0/3.0, 20.0, 100.0/3.0),
                        ] {
                            grid.spawn((Backdrop(format!("Interface\\TalentFrame\\{}-{tile}.blp", tab.background)),
                                ImageNode { color: Color::srgba(1.0, 1.0, 1.0, 0.8), ..default() },
                                Node { position_type: PositionType::Absolute, left: Val::Percent(left), top: Val::Percent(top),
                                    width: Val::Percent(width), height: Val::Percent(height), ..default() }));
                        }
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
                        grid.spawn((Button, Act::Talent(node.id), TalentInfo { class: hero.class, id: node.id, spell: node.ranks[0], rank, max, milestone: None },
                            Node { position_type: PositionType::Absolute, left: Val::Percent(x), top: Val::Percent(y),
                                margin: UiRect { left: Val::Px(-ICON/2.0), top: Val::Px(-ICON/2.0), ..default() },
                                width: Val::Px(ICON), height: Val::Px(ICON), border: UiRect::all(Val::Px(2.0)), ..default() },
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
                        Node { width: Val::Percent(100.0), min_height: Val::Px(22.0), align_items: AlignItems::Center, ..default() }))
                    .with_children(|line| { text(line, font, format!("{at} points · {}", campaign::milestone_name(hero.class, tree, at)), 12.0,
                        if points[tree] >= at { ACTIVE } else { DIM }); });
                }
            });
        }
        panel(row, 0.86, |side| {
            side.spawn(Node { width: Val::Percent(100.0), column_gap: Val::Px(4.0), ..default() }).with_children(|specs| {
                for spec in 0..3u8 {
                    let p = dbc_points(hero, usize::from(spec));
                    let spread = if p == [0;3] { "Empty".into() } else { format!("{}/{}/{}", p[0],p[1],p[2]) };
                    button(specs, font, format!("Spec {}\n{spread}", spec+1), Act::Spec(spec), hero.spec == spec);
                }
            });
            text(side, font, "Talent Points", 22.0, GOLD);
            text(side, font, format!("{unspent} to spend"), 25.0, GOLD);
            text(side, font, format!("{spent} of {earned} earned spent · 51 at most"), 12.0, DIM);
            text(side, font, "Runs award talent experience for time survived, kills, bosses and victory. Later zones pay more. Respecs are free.", 12.0, DIM);
            side.spawn(Node { width: Val::Percent(100.0), ..default() }).with_children(|reset| {
                button(reset, font, "Reset this spec".into(), Act::Respec, false);
            });
            text(side, font, "Point at a talent or milestone to inspect it. Click spends a point; right-click refunds one. Each row needs five points per tier in earlier rows of its tree.", 12.0, DIM);
            side.spawn((TalentDetail::Title, hud::text_bundle(font, "Choose a talent to read its Survivors effect.", 14.0, Color::WHITE)));
            side.spawn((TalentDetail::Effect, hud::text_bundle(font, "", 14.0, Color::WHITE)));
            if hero.spent() > 0 && spent == 0 {
                text(side, font, "Spending on these trees returns the legacy spread to unspent points.", 12.0, DIM);
            }
        });
    });
}
