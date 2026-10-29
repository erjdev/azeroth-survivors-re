//! Reference achievement rail, progress summary and wrapping cards over the live catalog.
use super::{hud, rewards, Account, Act, Armory, ChipParent, DIM, GOLD};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const FRAME: Color = Color::srgb(0.48, 0.37, 0.12);
const NAMES: [&str; 6] = [
    "Summary",
    "Zones",
    "Classes",
    "Collection",
    "Feats",
    "Lifetime",
];
const ICONS: [&str; 6] = [
    "INV_Misc_Note_01",
    "INV_Misc_Map_01",
    "INV_Shirt_GuildTabard_01",
    "INV_Misc_Bag_10",
    "INV_Misc_Cape_11",
    "INV_Misc_PocketWatch_01",
];

#[derive(Component)]
pub(in crate::survivors) struct ScrollCategory(pub(super) usize);

#[derive(Component)]
pub(in crate::survivors) struct RailScroll;
#[derive(Component)]
pub(in crate::survivors) struct PageLayout;
#[derive(Component)]
pub(in crate::survivors) enum RailMode {
    Wide,
    Compact,
}

#[derive(Component, Clone, Copy)]
pub(in crate::survivors) enum Pane {
    Rail,
    Cards,
}

type RailModeFilter = (Without<Pane>, Without<PageLayout>);

/// Decide from available logical width, so interface scale and DPI participate.
pub(in crate::survivors) fn layout(
    roots: Query<&ComputedNode, With<PageLayout>>,
    mut panes: Query<(&Pane, &mut Node)>,
    mut root_styles: Query<&mut Node, (With<PageLayout>, Without<Pane>)>,
    mut rail_modes: Query<(&RailMode, &mut Node), RailModeFilter>,
) {
    let Ok(root) = roots.single() else { return };
    if root.size().x <= 0.0 {
        return;
    }
    let stacked = root.size().x * root.inverse_scale_factor() < 560.0;
    for mut style in &mut root_styles {
        let direction = if stacked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        };
        if style.flex_direction != direction {
            style.flex_direction = direction;
        }
    }
    for (mode, mut style) in &mut rail_modes {
        let visible = matches!(mode, RailMode::Compact) == stacked;
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if style.display != display {
            style.display = display;
        }
    }
    for (pane, mut style) in &mut panes {
        let (width, height, basis) = match (stacked, pane) {
            (true, Pane::Rail) => (Val::Percent(100.0), Val::Percent(35.0), Val::Auto),
            (true, Pane::Cards) => (Val::Percent(100.0), Val::Auto, Val::Px(0.0)),
            (false, Pane::Rail) => (Val::Percent(24.0), Val::Percent(100.0), Val::Auto),
            (false, Pane::Cards) => (Val::Auto, Val::Percent(100.0), Val::Px(0.0)),
        };
        if (style.width, style.height, style.flex_basis) != (width, height, basis) {
            style.width = width;
            style.height = height;
            style.flex_basis = basis;
        }
    }
}

fn pane(
    parent: &mut ChildSpawnerCommands,
    role: Pane,
    marker: impl Bundle,
    scroll: Vec2,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            role,
            Node {
                width: if matches!(role, Pane::Rail) {
                    Val::Percent(24.0)
                } else {
                    Val::Auto
                },
                height: Val::Percent(100.0),
                flex_grow: if matches!(role, Pane::Cards) {
                    1.0
                } else {
                    0.0
                },
                flex_basis: if matches!(role, Pane::Cards) {
                    Val::Px(0.0)
                } else {
                    Val::Auto
                },
                flex_shrink: 0.0,
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.027, 0.014)),
            BorderColor::all(FRAME),
        ))
        .with_children(|frame| {
            let viewport = frame
                .spawn((
                    marker,
                    super::collection::CollectionScroll,
                    RelativeCursorPosition::default(),
                    ScrollPosition(scroll),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        min_height: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::right(Val::Px(12.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(content);
                })
                .id();
            super::scrollbar::spawn(frame, viewport);
        });
}

pub(in crate::survivors) fn remember_scroll(
    mut armory: ResMut<Armory>,
    surfaces: Query<(&ScrollCategory, &ScrollPosition)>,
    rail: Query<&ScrollPosition, With<RailScroll>>,
) {
    if let Ok(position) = rail.single() {
        if position.0.is_finite() && armory.achievement_rail_scroll != position.0 {
            armory.achievement_rail_scroll = position.0;
        }
    }
    for (category, position) in &surfaces {
        if position.0.is_finite() {
            if let Some(old) = armory.achievement_scroll.get_mut(category.0) {
                if *old != position.0 {
                    *old = position.0;
                }
            }
        }
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

fn icon(ui: &mut ChildSpawnerCommands, category: usize, size: f32) {
    artwork(ui, ICONS[category], size);
}

fn artwork(ui: &mut ChildSpawnerCommands, name: &str, size: f32) {
    ui.spawn((
        super::talent_page::Backdrop(format!("Interface\\Icons\\{name}.blp")),
        ImageNode::default(),
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn category(page: usize) -> Option<rewards::Category> {
    use rewards::Category::*;
    [Zones, Classes, Collection, Feats, Lifetime]
        .get(page.checked_sub(1)?)
        .copied()
}

fn counts(
    rows: &[rewards::Listed],
    category: Option<rewards::Category>,
) -> (usize, usize, u32, u32) {
    rows.iter()
        .filter(|r| r.available && category.is_none_or(|c| r.category == c))
        .fold((0, 0, 0, 0), |(have, total, pts, all), row| {
            (
                have + usize::from(row.earned),
                total + 1,
                pts + if row.earned { row.points } else { 0 },
                all + row.points,
            )
        })
}

fn fraction(have: u32, total: u32) -> f32 {
    have as f32 / total.max(1) as f32
}

fn meter(ui: &mut ChildSpawnerCommands, fraction: f32) {
    ui.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(7.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(FRAME),
        BackgroundColor(Color::BLACK),
    ))
    .with_children(|bar| {
        bar.spawn((
            Node {
                width: Val::Percent(fraction.clamp(0.0, 1.0) * 100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(GOLD),
        ));
    });
}

fn grid(ui: &mut ChildSpawnerCommands, body: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(8.0),
        row_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(body);
}

pub(super) fn build(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let rows = account.achievement_list();
    let selected = usize::from(armory.achieve.min(5));
    let (have, total, pts, all) = counts(&rows, None);
    panel
        .spawn((
            PageLayout,
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|row| {
            pane(
                row,
                Pane::Rail,
                RailScroll,
                armory.achievement_rail_scroll,
                |side| {
                    side.spawn((
                        RailMode::Wide,
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                    ))
                    .with_children(|side| {
                        super::typography::section(side, font, "Achievements", 22.0, GOLD);
                        side.spawn(Node {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|center| {
                            center
                                .spawn((
                                    Node {
                                        width: Val::Px(116.0),
                                        height: Val::Px(116.0),
                                        border_radius: BorderRadius::MAX,
                                        border: UiRect::all(Val::Px(2.0)),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BorderColor::all(GOLD),
                                    BackgroundColor(Color::srgb(0.12, 0.075, 0.018)),
                                ))
                                .with_children(|badge| {
                                    text(badge, font, pts.to_string(), 32.0, GOLD);
                                    text(badge, font, format!("of {all} points"), 12.0, DIM);
                                });
                        });
                        text(
                            side,
                            font,
                            format!("Account progress · {:.0}%", fraction(pts, all) * 100.0),
                            13.0,
                            GOLD,
                        );
                        meter(side, fraction(pts, all));
                        for (i, name) in NAMES.iter().enumerate() {
                            super::controls::content_button(
                                side,
                                Act::Achieve(i as u8),
                                selected == i,
                                Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(42.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                |nav| {
                                    icon(nav, i, 24.0);
                                    nav.spawn(Node {
                                        flex_grow: 1.0,
                                        flex_basis: Val::Px(0.0),
                                        min_width: Val::Px(0.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(4.0),
                                        ..default()
                                    })
                                    .with_children(
                                        |content| {
                                            let (n, total, pts, all) = counts(&rows, category(i));
                                            content
                                                .spawn(Node {
                                                    width: Val::Percent(100.0),
                                                    flex_wrap: FlexWrap::Wrap,
                                                    justify_content: JustifyContent::SpaceBetween,
                                                    column_gap: Val::Px(6.0),
                                                    ..default()
                                                })
                                                .with_children(|label| {
                                                    text(label, font, *name, 13.0, GOLD);
                                                    if i > 0 {
                                                        text(
                                                            label,
                                                            font,
                                                            format!("{n}/{total}"),
                                                            12.0,
                                                            DIM,
                                                        );
                                                    }
                                                });
                                            if i > 0 {
                                                meter(content, fraction(pts, all));
                                            }
                                        },
                                    );
                                },
                            );
                        }
                        text(side, font, format!("{have} of {total} earned"), 13.0, GOLD);
                        text(
                    side,
                    font,
                    "Every hero contributes to this account. Spend achievement points on Rewards.",
                    12.0,
                    DIM,
                );
                    });
                    side.spawn((
                        RailMode::Compact,
                        Node {
                            display: Display::None,
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                    ))
                    .with_children(|compact| {
                        text(
                            compact,
                            font,
                            format!("{pts} / {all} points · {have} / {total} earned"),
                            13.0,
                            GOLD,
                        );
                        compact
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|nav| {
                                for (i, name) in NAMES.iter().enumerate() {
                                    super::controls::content_button(
                                        nav,
                                        Act::Achieve(i as u8),
                                        selected == i,
                                        Node {
                                            flex_basis: Val::Percent(30.0),
                                            flex_grow: 1.0,
                                            min_width: Val::Px(100.0),
                                            min_height: Val::Px(32.0),
                                            padding: UiRect::all(Val::Px(5.0)),
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        |button| text(button, font, *name, 13.0, GOLD),
                                    );
                                }
                            });
                    });
                },
            );
            pane(
                row,
                Pane::Cards,
                ScrollCategory(selected),
                armory.achievement_scroll[selected],
                |body| {
                    super::typography::section(body, font, NAMES[selected], 22.0, GOLD);
                    if selected == 0 {
                        summary(body, font, account, &rows);
                    } else {
                        let (have, total, pts, all) = counts(&rows, category(selected));
                        text(
                            body,
                            font,
                            format!("{have} of {total} earned · {pts} of {all} points"),
                            13.0,
                            DIM,
                        );
                        meter(body, fraction(pts, all));
                        let mut entries: Vec<_> = rows
                            .iter()
                            .filter(|r| Some(r.category) == category(selected))
                            .collect();
                        if selected == 1 {
                            entries.sort_by_key(|r| r.progress.is_some());
                        }
                        grid(body, |grid| {
                            for entry in entries {
                                card(grid, font, account, entry);
                            }
                        });
                    }
                },
            );
        });
}

fn card(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    entry: &rewards::Listed,
) {
    ui.spawn((
        Node {
            flex_basis: Val::Percent(48.0),
            flex_grow: 0.0,
            min_width: Val::Px(200.0),
            padding: UiRect::all(Val::Px(9.0)),
            column_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            ..default()
        },
        BorderColor::all(if entry.earned { GOLD } else { FRAME }),
        BackgroundColor(if entry.earned {
            Color::srgb(0.14, 0.09, 0.02)
        } else {
            Color::srgb(0.025, 0.02, 0.012)
        }),
    ))
    .with_children(|card| {
        artwork(card, entry.icon, 32.0);
        card.spawn(Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|content| {
            text(
                content,
                font,
                &entry.name,
                16.0,
                if entry.earned {
                    GOLD
                } else {
                    Color::srgb(0.8, 0.74, 0.55)
                },
            );
            text(content, font, &entry.description, 12.0, DIM);
            if let Some((have, need)) = entry.progress.filter(|_| !entry.earned) {
                meter(content, fraction(have, need));
                text(content, font, format!("{have} / {need}"), 12.0, DIM);
            }
            if entry.id == 65 {
                text(
                    content,
                    font,
                    "Companion: Westfall Chicken",
                    12.0,
                    hud::card_growth::GAIN,
                );
            }
            if entry.id == rewards::DEATHLESS_HOUR {
                text(content, font, "Companion: Panda Cub", 12.0, DIM);
            }
            if entry.id == rewards::NO_U {
                text(content, font, "Companion: Sapphiron", 12.0, DIM);
            }
            let status = if !entry.available {
                "Criteria pending".to_string()
            } else if entry.earned {
                account
                    .earned_date(entry.id)
                    .map_or_else(|| "Earned".to_string(), |date| format!("Earned {date}"))
            } else {
                "Not earned".to_string()
            };
            text(
                content,
                font,
                status,
                11.0,
                if entry.earned { GOLD } else { DIM },
            );
        });
        if entry.available {
            card.spawn((
                Node {
                    width: Val::Px(35.0),
                    height: Val::Px(35.0),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::MAX,
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor::all(FRAME),
            ))
            .with_children(|badge| {
                text(
                    badge,
                    font,
                    entry.points.to_string(),
                    16.0,
                    if entry.earned { GOLD } else { DIM },
                )
            });
        }
    });
}

fn almost_there(rows: &[rewards::Listed]) -> Vec<&rewards::Listed> {
    let mut close: Vec<_> = rows
        .iter()
        .filter(|r| {
            r.available
                && !r.earned
                && r.progress
                    .is_some_and(|(have, need)| have > 0 && have < need)
        })
        .collect();
    close.sort_by(|a, b| {
        let (ah, an) = a.progress.unwrap();
        let (bh, bn) = b.progress.unwrap();
        (u64::from(bh) * u64::from(an)).cmp(&(u64::from(ah) * u64::from(bn)))
    });
    close.truncate(4);
    close
}

fn summary(
    body: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    rows: &[rewards::Listed],
) {
    text(body, font, "Recently earned", 14.0, GOLD);
    let recent: Vec<_> = account
        .earned_ids()
        .iter()
        .rev()
        .filter_map(|id| rows.iter().find(|r| r.id == *id && r.earned))
        .take(4)
        .collect();
    if recent.is_empty() {
        text(
            body,
            font,
            "None yet. Complete achievements to earn account rewards.",
            12.0,
            DIM,
        );
    }
    grid(body, |grid| {
        for entry in recent {
            card(grid, font, account, entry);
        }
    });
    text(body, font, "Progress", 14.0, GOLD);
    grid(body, |grid| {
        for (i, name) in NAMES.iter().enumerate().skip(1) {
            grid.spawn(Node {
                flex_basis: Val::Percent(48.0),
                flex_grow: 0.0,
                min_width: Val::Px(200.0),
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                icon(row, i, 24.0);
                row.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|content| {
                    let (have, total, pts, all) = counts(rows, category(i));
                    text(
                        content,
                        font,
                        format!("{name} · {have}/{total} · {pts}/{all} points"),
                        12.0,
                        GOLD,
                    );
                    meter(content, fraction(pts, all));
                });
            });
        }
    });
    text(body, font, "Almost there", 14.0, GOLD);
    let close = almost_there(rows);
    if close.is_empty() {
        text(body, font, "No achievements in progress yet.", 12.0, DIM);
    }
    grid(body, |grid| {
        for entry in close {
            card(grid, font, account, entry);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panes_reflow_in_logical_units_and_keep_independent_reading_positions() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .add_systems(Update, (layout, remember_scroll));
        let root = app
            .world_mut()
            .spawn((
                PageLayout,
                Node::default(),
                ComputedNode {
                    size: Vec2::new(1200., 800.),
                    inverse_scale_factor: 0.5,
                    ..default()
                },
            ))
            .id();
        let rail = app
            .world_mut()
            .spawn((
                Pane::Rail,
                Node::default(),
                RailScroll,
                ScrollPosition(Vec2::new(0., 100.)),
            ))
            .id();
        let cards = app
            .world_mut()
            .spawn((
                Pane::Cards,
                Node::default(),
                ScrollCategory(1),
                ScrollPosition(Vec2::new(0., 900.)),
            ))
            .id();
        let wide = app
            .world_mut()
            .spawn((RailMode::Wide, Node::default()))
            .id();
        let compact = app
            .world_mut()
            .spawn((RailMode::Compact, Node::default()))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(wide).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(compact).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(root).unwrap().flex_direction,
            FlexDirection::Row
        );
        assert_eq!(
            app.world().get::<Node>(rail).unwrap().width,
            Val::Percent(24.)
        );
        app.world_mut()
            .get_mut::<ComputedNode>(root)
            .unwrap()
            .size
            .x = 1000.;
        app.update();
        assert_eq!(
            app.world().get::<Node>(root).unwrap().flex_direction,
            FlexDirection::Column
        );
        assert_eq!(
            app.world().get::<Node>(rail).unwrap().height,
            Val::Percent(35.)
        );
        assert_eq!(
            app.world().get::<Node>(cards).unwrap().width,
            Val::Percent(100.)
        );
        assert_eq!(
            app.world().get::<Node>(wide).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(compact).unwrap().display,
            Display::Flex
        );
        let armory = app.world().resource::<Armory>();
        assert_eq!(armory.achievement_rail_scroll.y, 100.);
        assert_eq!(armory.achievement_scroll[1].y, 900.);
        assert_eq!(armory.achievement_scroll[0], Vec2::ZERO);
        app.world_mut()
            .get_mut::<ComputedNode>(root)
            .unwrap()
            .size
            .x = 1200.;
        app.update();
        assert_eq!(
            app.world().get::<Node>(root).unwrap().flex_direction,
            FlexDirection::Row
        );
        assert_eq!(app.world().get::<ScrollPosition>(cards).unwrap().0.y, 900.);
    }

    #[test]
    fn available_completion_does_not_require_placeholder_achievements() {
        let snapshot = rewards::Snapshot::default();
        let all_ids: Vec<_> = rewards::listed(&snapshot, &[])
            .iter()
            .map(|r| r.id)
            .collect();
        let rows = rewards::listed(&snapshot, &all_ids);
        let (have, total, points, all_points) = counts(&rows, None);
        assert_eq!(
            (have, total),
            (rewards::available_count(), rewards::available_count())
        );
        assert_eq!(points, all_points);
        let feats = counts(&rows, Some(rewards::Category::Feats));
        assert_eq!(feats, (3, 3, 75, 75));
        assert_eq!(
            rows.iter()
                .filter(|r| r.category == rewards::Category::Feats)
                .count(),
            5
        );
        assert_eq!(rows.iter().filter(|r| !r.available && !r.earned).count(), 2);
    }

    #[test]
    fn almost_there_excludes_zero_progress_earned_and_placeholders_and_sorts_ratios() {
        let snap = rewards::Snapshot {
            runs: 78,
            wins: 9,
            pieces: 20,
            ..default()
        };
        let rows = rewards::listed(&snap, &[]);
        let close = almost_there(&rows);
        assert_eq!(
            close.iter().map(|r| r.id).collect::<Vec<_>>(),
            [62, 65, 42, 63]
        );
        assert!(close.iter().all(|r| r.available && !r.earned));
        assert!(almost_there(&rewards::listed(&rewards::Snapshot::default(), &[])).is_empty());
    }
}
