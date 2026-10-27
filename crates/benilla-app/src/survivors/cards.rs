//! The level-up picker, as a survivors game deals it: three tall cards that flip up out of the
//! deck one after another, tilt toward the cursor, lift under it and burst on the pick. Each card
//! wears its rarity in WoW's item-quality colours, so what is better reads at a glance: a fresh
//! spell is Rare, a rank up Uncommon, a rank that adds a target or reaches the top Epic.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::data::{self, Kind};
use super::hud::{self, Choice, HudFont, Offer, SpellIcon};
use super::{Phase, Rng, Run};

/// A developer-only presentation overlay; never a playable offer.
#[derive(Resource, Default)]
pub(super) struct PreviewHand(pub bool);

pub(super) fn preview_active(preview: Res<PreviewHand>) -> bool {
    preview.0
}

/// WoW's item-quality colours (`ITEM_QUALITY_COLORS`).
pub(super) const COMMON: Color = Color::srgb(1.0, 1.0, 1.0);
pub(super) const UNCOMMON: Color = Color::srgb(0.118, 1.0, 0.0);
pub(super) const RARE: Color = Color::srgb(0.0, 0.439, 0.867);
pub(super) const EPIC: Color = Color::srgb(0.639, 0.208, 0.933);
pub(super) const LEGENDARY: Color = Color::srgb(1.0, 0.502, 0.0);

const CARD_H: f32 = 330.0;
/// Seconds between one card landing and the next.
const DEAL_STAGGER: f32 = 0.09;
const DEAL_SECS: f32 = 0.38;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub(super) fn color(self) -> Color {
        match self {
            Self::Common => COMMON,
            Self::Uncommon => UNCOMMON,
            Self::Rare => RARE,
            Self::Epic => EPIC,
            Self::Legendary => LEGENDARY,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Epic => "Epic",
            Self::Legendary => "Legendary",
        }
    }
}

/// A spell's rough damage per second at rank 1 against a crowd: per-cast damage times the
/// enemies it can reach, over its cooldown. Only for ranking a class's own spells against each
/// other; the defensive kinds have none and sit in the middle.
fn power(def: &data::AbilityDef) -> Option<f32> {
    let area = |r: f32| 1.0 + r * 0.6;
    let reach = match def.kind {
        Kind::Bolt | Kind::Strike => def.count as f32 + def.radius * 0.4,
        Kind::Melee | Kind::Orbit => def.count as f32,
        Kind::Nova | Kind::Cone | Kind::Ground | Kind::GroundSelf | Kind::Aura => area(def.radius),
        Kind::Chain => def.count as f32 + 1.0,
        Kind::Dot => def.count as f32,
        _ => return None,
    };
    let per = if def.damage > 0.0 {
        def.damage
    } else {
        def.dot * def.duration.max(1.0)
    };
    let ticks = if matches!(def.kind, Kind::Ground | Kind::GroundSelf) && def.duration > 0.0 {
        def.duration
    } else {
        1.0
    };
    Some(per * ticks * reach / def.cooldown.max(0.2))
}

/// Where a new spell stands among its class's: the strongest third Epic, the weakest third
/// Uncommon, the rest (and the defensive spells) Rare.
fn spell_tier(def: &data::AbilityDef, run: &Run) -> Rarity {
    let Some(mine) = power(def) else {
        return Rarity::Rare;
    };
    let mut all: Vec<f32> = run
        .class
        .pool
        .iter()
        .filter_map(|k| power(data::ability(k)))
        .collect();
    all.sort_by(f32::total_cmp);
    let rank = all.iter().filter(|&&p| p < mine).count() as f32 / all.len().max(1) as f32;
    if rank >= 0.66 {
        Rarity::Epic
    } else if rank < 0.33 {
        Rarity::Uncommon
    } else {
        Rarity::Rare
    }
}

/// What a choice is worth, by what it brings: a top rank is Legendary, a rank that adds a target
/// Epic, a new spell by its power among the class's ([`spell_tier`]), a plain rank Uncommon.
pub(super) fn rarity(choice: Choice, run: &Run) -> Rarity {
    match choice {
        Choice::NewAbility(def) => spell_tier(def, run),
        Choice::Upgrade(k) => {
            let o = &run.abilities[k];
            let next = o.level + 1;
            let targets = [3, 5, 7].contains(&next)
                && matches!(
                    o.def.kind,
                    Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Dot | Kind::Chain | Kind::Orbit
                );
            if next == data::MAX_ABILITY_LEVEL {
                Rarity::Legendary
            } else if targets {
                Rarity::Epic
            } else {
                Rarity::Uncommon
            }
        }
        Choice::NewPassive(_) => Rarity::Uncommon,
        Choice::UpPassive(k) => match run.passives[k].1 + 1 {
            n if n == data::MAX_PASSIVE_LEVEL => Rarity::Epic,
            n if n + 1 == data::MAX_PASSIVE_LEVEL => Rarity::Rare,
            2 => Rarity::Common,
            _ => Rarity::Uncommon,
        },
        Choice::Heal => Rarity::Common,
        Choice::Jewel(super::campaign::JewelKind::Ring(_)) => Rarity::Uncommon,
        Choice::Jewel(super::campaign::JewelKind::Trinket(_)) => Rarity::Rare,
        Choice::Jewel(super::campaign::JewelKind::Amulet(_)) => Rarity::Epic,
        Choice::Jewel(super::campaign::JewelKind::Legend(_)) => Rarity::Legendary,
    }
}

/// One card's motion: dealt in, tilted to the cursor, lifted under it, burst on the pick.
#[derive(Component)]
pub(super) struct Card {
    index: usize,
    /// Seconds since this card's deal began; negative while it waits its turn.
    t: f32,
    hover: f32,
    tilt: Vec2,
    picked: Option<f32>,
    rarity: Rarity,
}

/// The rarity lighting inside the card face.
#[derive(Component)]
pub(super) struct CardGlow;

/// The shine that sweeps across a card as it lands.
#[derive(Component)]
pub(super) struct CardShine(usize);

/// The pending pick: the card bursts, then the choice applies.
#[derive(Resource, Default)]
pub(super) struct PendingPick(Option<(usize, f32)>);

const PICK_SECS: f32 = 0.28;

/// The cards, rebuilt for each offer.
pub(super) fn deal_cards(
    mut commands: Commands,
    offer: Res<Offer>,
    strings: Option<Res<super::localization::Strings>>,
    preview: Res<PreviewHand>,
    run: Res<Run>,
    mut account: ResMut<super::account::Account>,
    spells: Option<Res<crate::ui_action::Spells>>,
    font: Res<HudFont>,
    mut rows: Query<(Entity, &mut ScrollPosition), With<hud::OfferRow>>,
    mut titles: Query<&mut Text, With<hud::LevelTitle>>,
    mut subs: Query<&mut Text, (With<hud::LevelUpSub>, Without<hud::LevelTitle>)>,
) {
    if !offer.is_changed() {
        return;
    }
    if let Ok(mut title) = titles.single_mut() {
        **title = if run.trading { "A CHEST" } else { "LEVEL UP" }.into();
    }
    if let Ok(mut sub) = subs.single_mut() {
        let mut line = if run.trading {
            "Choose your chest reward".to_string()
        } else {
            "Choose your power".to_string()
        };
        let mut hints = Vec::new();
        if !run.trading && run.rerolls > 0 {
            hints.push(format!("R reroll {}", run.rerolls));
        }
        if run.can_skip {
            hints.push("T skip".into());
        }
        if run.banishes > 0 {
            hints.push(format!("B banish {}", run.banishes));
        }
        if !hints.is_empty() {
            line.push_str("  ·  ");
            line.push_str(&hints.join("  "));
        }
        **sub = if preview.0 {
            "UI preview · edit the preview file to change cards or return to a page".into()
        } else {
            line
        };
    }
    let Ok((row, mut scroll)) = rows.single_mut() else {
        return;
    };
    // Record the actual hand being built, never discarded random candidates (e.g. banish).
    if let Some(class) = (!preview.0)
        .then(|| {
            super::data::CLASSES
                .iter()
                .position(|class| class.id == run.class.id)
        })
        .flatten()
    {
        account.remember_jewelry(
            class,
            offer.choices().iter().filter_map(|choice| match choice {
                Choice::Jewel(item) => Some(*item),
                _ => None,
            }),
        );
        account.grant_new();
        account.save();
    }
    scroll.0 = Vec2::ZERO;
    commands.entity(row).despawn_related::<Children>();
    let fallback = super::localization::Strings::default();
    let strings = strings.as_deref().unwrap_or(&fallback);
    let f = &font.0;
    commands.entity(row).with_children(|row| {
        for (i, &choice) in offer.choices().iter().enumerate() {
            let card = hud::describe_card(choice, &run, spells.as_deref());
            let rarity = rarity(choice, &run);
            let rc = rarity.color();
            row.spawn((
                Card {
                    index: i,
                    t: -(i as f32) * DEAL_STAGGER,
                    hover: 0.0,
                    tilt: Vec2::ZERO,
                    picked: None,
                    rarity,
                },
                Button,
                Interaction::default(),
                RelativeCursorPosition::default(),
                Node {
                    flex_basis: Val::Percent(22.0),
                    flex_grow: 1.0,
                    flex_shrink: 0.0,
                    min_width: Val::Px(200.0),
                    max_width: Val::Px(280.0),
                    min_height: Val::Px(CARD_H),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::new(
                        Val::Px(16.0),
                        Val::Px(16.0),
                        Val::Px(18.0),
                        Val::Px(14.0),
                    ),
                    row_gap: Val::Px(6.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                // A dark card face lit from the top in the rarity's colour.
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::percent(rc.mix(&Color::BLACK, 0.55).with_alpha(0.98), 0.0),
                    ColorStop::percent(Color::srgba(0.05, 0.04, 0.035, 0.97), 42.0),
                    ColorStop::percent(Color::srgba(0.02, 0.015, 0.01, 0.98), 100.0),
                ])),
                BorderColor::all(rc),
                UiTransform {
                    translation: Val2::px(0.0, 260.0),
                    scale: Vec2::splat(0.6),
                    rotation: Rot2::degrees(-14.0),
                },
                Visibility::Hidden,
            ))
            .with_children(|c| {
                c.spawn((
                    CardGlow,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        height: Val::Px(150.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundGradient::from(RadialGradient::new(
                        UiPosition::new(Vec2::new(0.0, -0.1), Val::Auto, Val::Auto),
                        RadialGradientShape::FarthestSide,
                        vec![
                            ColorStop::percent(rc.with_alpha(0.45), 0.0),
                            ColorStop::percent(rc.with_alpha(0.0), 100.0),
                        ],
                    )),
                    bevy::ui::FocusPolicy::Pass,
                ));
                // The key and the rarity on one line along the top.
                c.spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|top| {
                    top.spawn(hud::text_bundle(
                        f,
                        format!("{}", i + 1),
                        15.0,
                        hud::DIM_GREY,
                    ));
                    top.spawn(hud::text_bundle(f, rarity.label(), 13.0, rc));
                });
                // The icon in a gold-rimmed socket.
                c.spawn((
                    Node {
                        width: Val::Px(74.0),
                        height: Val::Px(74.0),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BorderColor::all(rc.mix(&hud::GOLD_TONE, 0.4)),
                    BoxShadow::new(
                        rc.with_alpha(0.55),
                        Val::Px(0.0),
                        Val::Px(0.0),
                        Val::Px(2.0),
                        Val::Px(14.0),
                    ),
                ))
                .with_children(|s| {
                    let mut node = s.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
                    ));
                    match card.icon {
                        hud::CardIcon::Spell(id) => {
                            node.insert(SpellIcon(id));
                        }
                        hud::CardIcon::Item(display) => {
                            node.insert(super::gear_icons::GearIcon(display, Color::WHITE));
                        }
                        hud::CardIcon::Missing => {
                            node.remove::<ImageNode>().with_children(|fallback| {
                                fallback.spawn(hud::text_bundle(
                                    f,
                                    "?",
                                    32.,
                                    Color::srgb(0.6, 0.6, 0.6),
                                ));
                            });
                        }
                    }
                });
                c.spawn(hud::centered_bundle(
                    f,
                    card.name,
                    21.0,
                    rc.mix(&Color::WHITE, 0.25),
                ));
                c.spawn(hud::centered_bundle(f, card.tag, 14.0, card.tag_color));
                // The pips: rank now, and the one this pick fills.
                if let Some((have, max)) = card.ranks {
                    c.spawn(Node {
                        column_gap: Val::Px(4.0),
                        margin: UiRect::vertical(Val::Px(2.0)),
                        ..default()
                    })
                    .with_children(|pips| {
                        for r in 0..max {
                            let color = if r < have {
                                hud::GOLD_TONE
                            } else if r == have {
                                rc
                            } else {
                                Color::srgba(1.0, 1.0, 1.0, 0.12)
                            };
                            pips.spawn((
                                Node {
                                    width: Val::Px(14.0),
                                    height: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(color),
                            ));
                        }
                    });
                }
                c.spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(1.0),
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundGradient::from(LinearGradient::to_right(vec![
                        ColorStop::percent(rc.with_alpha(0.0), 0.0),
                        ColorStop::percent(rc.with_alpha(0.8), 50.0),
                        ColorStop::percent(rc.with_alpha(0.0), 100.0),
                    ])),
                ));
                c.spawn(hud::centered_bundle(f, "", 14.0, hud::inline::PROSE))
                    .with_children(|paragraph| {
                        if !card.intro.is_empty() {
                            paragraph.spawn((
                                TextSpan::new(format!("{} ", card.intro)),
                                TextColor(hud::inline::PROSE),
                                TextFont {
                                    font: f.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                            ));
                        }
                        for (text, color) in hud::inline::describe(choice, &run, strings) {
                            paragraph.spawn((
                                TextSpan::new(text),
                                TextColor(color),
                                TextFont {
                                    font: f.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                            ));
                        }
                    });
                // The landing shine, a bright band that crosses once.
                c.spawn((
                    CardShine(i),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    landing_sheen(0.0),
                    bevy::ui::FocusPolicy::Pass,
                ));
            });
        }
    });
}

// Animate stops inside a rounded card-sized surface, not an oversized rotated quad.
// Percent stops preserve the sweep when the card is resized or transformed.
fn landing_sheen(progress: f32) -> BackgroundGradient {
    let center = -25.0 + progress * 150.0;
    BackgroundGradient::from(LinearGradient::new(
        108.0_f32.to_radians(),
        vec![
            ColorStop::percent(Color::srgba(1.0, 0.97, 0.85, 0.0), center - 20.0),
            ColorStop::percent(Color::srgba(1.0, 0.97, 0.85, 0.28), center),
            ColorStop::percent(Color::srgba(1.0, 0.97, 0.85, 0.0), center + 20.0),
        ],
    ))
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Deal, tilt, lift and burst: real time, since the game clock is paused under the picker.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn animate_cards(
    time: Res<Time<Real>>,
    phase: Res<State<Phase>>,
    preview: Res<PreviewHand>,
    mut cards: Query<(
        &mut Card,
        &Interaction,
        &RelativeCursorPosition,
        &mut UiTransform,
        &mut BorderColor,
        &mut Visibility,
    )>,
    mut shines: Query<(&CardShine, &mut BackgroundGradient)>,
    mut pending: ResMut<PendingPick>,
    mut run: ResMut<Run>,
    mut offer: ResMut<Offer>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<Phase>>,
) {
    if *phase.get() != Phase::LevelUp && !preview.0 {
        pending.0 = None;
        return;
    }
    if preview.0 {
        pending.0 = None;
    }
    let dt = time.delta_secs().min(0.05);
    let now = time.elapsed_secs();
    let chosen = pending.0.map(|(i, _)| i);
    let mut shine_at = [None::<f32>; 4];
    for (mut card, interaction, cursor, mut tf, mut border, mut vis) in &mut cards {
        let card = &mut *card;
        card.t += dt;
        if card.t < 0.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let deal = (card.t / DEAL_SECS).clamp(0.0, 1.0);
        let e = ease_out_back(deal);
        let hot = *interaction != Interaction::None && chosen.is_none() && deal >= 1.0;
        let k = 1.0 - (-dt * 14.0).exp();
        let hv = card.hover;
        card.hover = hv + ((if hot { 1.0 } else { 0.0 }) - hv) * k;
        let want_tilt = if hot {
            cursor
                .normalized
                .unwrap_or_default()
                .clamp(Vec2::splat(-0.5), Vec2::splat(0.5))
        } else {
            Vec2::ZERO
        };
        let tl = card.tilt;
        card.tilt = tl + (want_tilt - tl) * k;
        let h = card.hover;
        // The pick: the chosen card swells and rises, the others drop away.
        let (pick_scale, pick_y, pick_fade) = match (chosen, card.picked) {
            (Some(c), _) if c == card.index => {
                let p = card.picked.get_or_insert(0.0);
                *p += dt;
                let u = (*p / PICK_SECS).clamp(0.0, 1.0);
                (1.0 + 0.18 * u, -30.0 * u, 0.0)
            }
            (Some(_), _) => {
                let p = card.picked.get_or_insert(0.0);
                *p += dt;
                let u = (*p / PICK_SECS).clamp(0.0, 1.0);
                (1.0 - 0.15 * u, 140.0 * u * u, u)
            }
            _ => (1.0, 0.0, 0.0),
        };
        // A perspective tilt, faked: the card turns about the cursor's side and leans with it.
        let lean = card.tilt.x * 7.0;
        let squash = 1.0 - card.tilt.x.abs() * 0.08;
        let bob = (now * 2.2 + card.index as f32 * 1.3).sin() * 1.5 * (1.0 - h);
        tf.translation = Val2::px(
            0.0,
            (1.0 - e) * 260.0 - 18.0 * h + card.tilt.y * 8.0 + bob + pick_y,
        );
        let s = (0.6 + 0.4 * e) * (1.0 + 0.06 * h) * pick_scale;
        tf.scale = Vec2::new(s * squash, s * (1.0 + card.tilt.y.abs() * 0.02));
        tf.rotation = Rot2::degrees((1.0 - e) * -14.0 + lean);
        let rc = card.rarity.color();
        // Hover emphasis stays on the rounded card instead of lighting a rectangular
        // shadow surface across the backdrop and neighboring cards.
        *border = BorderColor::all(rc.mix(&Color::WHITE, 0.35 * h));
        if pick_fade >= 1.0 {
            *vis = Visibility::Hidden;
        }
        if card.index < shine_at.len() {
            shine_at[card.index] = Some(card.t);
        }
    }
    for (shine, mut gradient) in &mut shines {
        let Some(t) = shine_at.get(shine.0).copied().flatten() else {
            continue;
        };
        let u = ((t - DEAL_SECS * 0.6) / 0.5).clamp(0.0, 1.0);
        *gradient = landing_sheen(u);
    }
    if let Some((i, t)) = pending.0.as_mut() {
        *t += dt;
        if *t >= PICK_SECS {
            let i = *i;
            pending.0 = None;
            hud::pick(i, &mut run, &mut offer, &mut rng, &mut next);
        }
    }
}

/// A click on a card, or 1/2/3: start its burst; the choice applies when it finishes.
pub(super) fn choose_card(
    phase: Res<State<Phase>>,
    keys: Res<ButtonInput<KeyCode>>,
    cards: Query<(&Card, &Interaction), Changed<Interaction>>,
    mut offer: ResMut<Offer>,
    mut pending: ResMut<PendingPick>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    mut stash: ResMut<super::stash::Stash>,
    mut next: ResMut<NextState<Phase>>,
) {
    if *phase.get() != Phase::LevelUp || pending.0.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) && hud::reroll(&mut run, &mut offer, &mut rng) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyT)
        && hud::skip_hand(&mut run, &mut offer, &mut rng, &mut stash, &mut next)
    {
        return;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        run.banish_armed = run.banishes > 0;
        if run.banish_armed {
            run.banner = Some(("Banish a card: press its number".into(), 2.5));
        }
        return;
    }
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];
    if run.banish_armed {
        for (i, key) in digits.into_iter().enumerate() {
            if keys.just_pressed(key) {
                hud::banish_card(&mut run, &mut offer, &mut rng, i);
                // An unavailable replacement must not turn this same key into a pick.
                return;
            }
        }
    }
    let mut pick = cards
        .iter()
        .find(|(c, i)| **i == Interaction::Pressed && c.t >= DEAL_SECS)
        .map(|(c, _)| c.index);
    for (i, key) in digits.into_iter().enumerate() {
        if keys.just_pressed(key) {
            pick = Some(i);
        }
    }
    if let Some(i) = pick.filter(|&i| i < offer.len()) {
        pending.0 = Some((i, 0.0));
    }
}

/// Keep wrapped hands reachable without reducing text or hit targets below readable sizes.
pub(super) fn scroll_hand(
    phase: Res<State<Phase>>,
    preview: Res<PreviewHand>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut rows: Query<(&ComputedNode, &mut ScrollPosition), With<hud::OfferRow>>,
) {
    let delta: f32 = wheel
        .read()
        .map(|event| match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 32.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        })
        .sum();
    if *phase.get() != Phase::LevelUp && !preview.0 {
        return;
    }
    for (node, mut scroll) in &mut rows {
        let maximum = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.0);
        scroll.0.y = (scroll.0.y - delta).clamp(0.0, maximum);
    }
}
