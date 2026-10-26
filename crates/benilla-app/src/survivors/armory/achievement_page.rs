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
pub(in crate::survivors) struct ScrollCategory(usize);

pub(in crate::survivors) fn remember_scroll(
    mut armory: ResMut<Armory>,
    surfaces: Query<(&ScrollCategory, &ScrollPosition)>,
) {
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
    ui.spawn((
        super::talent_page::Backdrop(format!("Interface\\Icons\\{}.blp", ICONS[category])),
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

fn page(category: rewards::Category) -> usize {
    use rewards::Category::*;
    match category {
        Zones => 1,
        Classes => 2,
        Collection => 3,
        Feats => 4,
        Lifetime => 5,
    }
}

fn counts(
    rows: &[rewards::Listed],
    category: Option<rewards::Category>,
) -> (usize, usize, u32, u32) {
    rows.iter()
        .filter(|r| category.is_none_or(|c| r.category == c))
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
            super::collection::CollectionScroll,
            ScrollCategory(selected),
            RelativeCursorPosition::default(),
            ScrollPosition(armory.achievement_scroll[selected]),
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                overflow: Overflow::scroll_y(),
                align_content: AlignContent::Start,
                ..default()
            },
        ))
        .with_children(|row| {
            super::reward_page::frame(row, 190.0, 1.0, |side| {
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
                            .with_children(|content| {
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
                                            text(label, font, format!("{n}/{total}"), 12.0, DIM);
                                        }
                                    });
                                if i > 0 {
                                    meter(content, fraction(pts, all));
                                }
                            });
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
            super::reward_page::frame(row, 540.0, 4.0, |body| {
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
            });
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
            flex_grow: 1.0,
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
        icon(card, page(entry.category), 32.0);
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
