//! Account rewards, arranged around the intention's currency rail and illustrated cards.
use super::{chip, hud, rewards, Account, Act, Armory, ChipParent, DIM, GOLD};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

/// The rendered category owns the offset even when an action changes the selected category.
#[derive(Component)]
pub(in crate::survivors) struct ScrollCategory(usize);

pub(in crate::survivors) fn remember_scroll(
    mut armory: ResMut<Armory>,
    surfaces: Query<(&ScrollCategory, &ScrollPosition)>,
) {
    for (category, position) in &surfaces {
        if position.0.is_finite()
            && armory
                .reward_scroll
                .get(category.0)
                .is_some_and(|old| *old != position.0)
        {
            armory.reward_scroll[category.0] = position.0;
        }
    }
}

const FRAME: Color = Color::srgb(0.48, 0.37, 0.12);

fn text(
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    ui.spawn(hud::text_bundle(font, value, size, color));
}

fn frame(
    ui: &mut ChildSpawnerCommands,
    basis: f32,
    grow: f32,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            flex_basis: Val::Px(basis),
            flex_grow: grow,
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.027, 0.014)),
        BorderColor::all(FRAME),
    ))
    .with_children(body);
}

pub(super) fn build(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let category = usize::from(armory.reward_page.min(2));
    panel.spawn((super::collection::CollectionScroll, ScrollCategory(category), RelativeCursorPosition::default(), ScrollPosition(armory.reward_scroll[category]), Node {
        width: Val::Percent(100.0), flex_grow: 1.0, flex_basis: Val::Px(0.0), min_height: Val::Px(0.0),
        flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), row_gap: Val::Px(12.0),
        overflow: Overflow::scroll_y(), align_content: AlignContent::Start, ..default()
    })).with_children(|row| {
        frame(row, 190.0, 1.0, |side| {
            text(side, font, "Rewards", 22.0, GOLD);
            text(side, font, format!("{} points to spend", account.unspent()), 22.0, GOLD);
            let earned = rewards::points_of(account.earned_ids());
            text(side, font, format!("{earned} earned · {} spent", earned.saturating_sub(account.unspent())), 12.0, DIM);
            text(side, font, format!("{} of {} achievements", rewards::earned_count(account.earned_ids()), rewards::catalog_len()), 12.0, DIM);
            for (i, name) in ["Perks", "Companions", "Tabards"].iter().enumerate() {
                side.spawn(Node { width: Val::Percent(100.0), ..default() }).with_children(|slot| {
                    chip(slot, font, name, Act::RewardPage(i as u8), armory.reward_page == i as u8);
                });
            }
            text(side, font, "Reset", 18.0, GOLD);
            text(side, font, format!("Refund every point spent on perks. Companion purchases and tabards are kept. Resets remaining: {}.", account.resets_left()), 12.0, DIM);
            side.spawn(Node::default()).with_children(|slot| chip(slot, font, "Reset perks", Act::Reward(12), false));
            text(side, font, "Purchases apply immediately. Rewards belong to the account and benefit every hero.", 12.0, DIM);
            if let Some(next) = account.next_locked() { text(side, font, next, 12.0, GOLD); }
        });
        frame(row, 540.0, 4.0, |body| {
            text(body, font, match armory.reward_page { 1 => "Companions", 2 => "Tabards", _ => "Perks" }, 22.0, GOLD);
            match armory.reward_page {
                1 => super::companion_page(body, font, armory, account),
                2 => super::tabard_page(body, font, account),
                _ => perks(body, font, account),
            }
        });
    });
}

fn perks(ui: &mut ChildSpawnerCommands, font: &Handle<Font>, account: &Account) {
    let icons = [
        "INV_Misc_PocketWatch_01",
        "INV_Misc_Coin_01",
        "INV_Misc_Book_09",
        "Spell_Shadow_Cripple",
        "INV_Misc_Bag_10",
        "INV_Misc_Spyglass_03",
    ];
    ui.spawn(Node {
        width: Val::Percent(100.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(8.0),
        row_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|grid| {
        // Row-major pairing matches the reference's two columns.
        for code in [0, 3, 1, 4, 2, 5] {
            let perk = rewards::PERKS[code];
            let rank = account.perks.rank(perk);
            let max = rewards::perk_max_rank(perk);
            let cost = rewards::perk_cost(perk, rank);
            grid.spawn((
                Node {
                    flex_basis: Val::Percent(48.0),
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    column_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(if rank > 0 {
                    Color::srgb(0.14, 0.10, 0.035)
                } else {
                    Color::srgb(0.025, 0.02, 0.012)
                }),
                BorderColor::all(FRAME),
            ))
            .with_children(|card| {
                card.spawn((
                    super::talent_page::Backdrop(format!("Interface\\Icons\\{}.blp", icons[code])),
                    ImageNode::default(),
                    Node {
                        width: Val::Px(34.0),
                        height: Val::Px(34.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));
                card.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|content| {
                    text(content, font, rewards::perk_name(perk), 16.0, GOLD);
                    content
                        .spawn(Node {
                            column_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|pips| {
                            for n in 0..max {
                                pips.spawn((
                                    Node {
                                        width: Val::Px(7.0),
                                        height: Val::Px(7.0),
                                        border_radius: BorderRadius::MAX,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor::all(FRAME),
                                    BackgroundColor(if n < rank { GOLD } else { Color::BLACK }),
                                ));
                            }
                        });
                    let (before, value, after) = rewards::perk_preview(perk, rank);
                    content
                        .spawn(hud::text_bundle(font, "", 12.0, DIM))
                        .with_children(|sentence| {
                            for (part, color) in [
                                (before.to_string(), DIM),
                                (value, hud::card_growth::GAIN),
                                (after.to_string(), DIM),
                            ] {
                                sentence.spawn((
                                    TextSpan::new(part),
                                    TextColor(color),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                ));
                            }
                        });
                    if let Some(cost) = cost {
                        content.spawn(Node::default()).with_children(|buy| {
                            chip(
                                buy,
                                font,
                                &format!("Rank {} · {cost} points", rank + 1),
                                Act::Reward(code as u8),
                                false,
                            )
                        });
                        if account.unspent() < cost {
                            text(content, font, "Not enough points", 12.0, DIM);
                        }
                    } else {
                        text(content, font, "Max rank", 12.0, GOLD);
                    }
                });
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_offsets_follow_the_rendered_page_and_reject_invalid_positions() {
        let mut app = App::new();
        app.init_resource::<Armory>()
            .add_systems(Update, remember_scroll);
        let old = app
            .world_mut()
            .spawn((ScrollCategory(0), ScrollPosition(Vec2::new(0., 280.))))
            .id();
        // A page action can change selection before the old rendered surface is replaced.
        app.world_mut().resource_mut::<Armory>().reward_page = 1;
        app.update();
        assert_eq!(
            app.world().resource::<Armory>().reward_scroll,
            [Vec2::new(0., 280.), Vec2::ZERO, Vec2::ZERO]
        );
        app.world_mut().despawn(old);
        let current = app
            .world_mut()
            .spawn((ScrollCategory(1), ScrollPosition(Vec2::new(0., 90.))))
            .id();
        app.update();
        app.world_mut()
            .entity_mut(current)
            .insert(ScrollPosition(Vec2::splat(f32::NAN)));
        app.update();
        let memory = app.world().resource::<Armory>().reward_scroll;
        assert_eq!(
            memory,
            [Vec2::new(0., 280.), Vec2::new(0., 90.), Vec2::ZERO]
        );
    }
}
