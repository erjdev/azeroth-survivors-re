//! The campaign panel on the glue menu: named heroes, the gated zone path, and spending talent
//! points. Appearance stays on the race and class screen beside it.

use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use super::account::Account;
use super::campaign::{self, ZONES};
use super::data::{self, CLASSES};
use super::hud::{self, HudFont, MenuRoot};
use super::{Phase, Selection, Typing};

const PANEL: Color = Color::srgba(0.05, 0.04, 0.03, 0.92);
const EDGE: Color = Color::srgb(0.45, 0.36, 0.16);
const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Page {
    Heroes,
    Zones,
    Talents,
}

#[derive(Resource)]
pub(super) struct Armory {
    page: Page,
    /// 0 none, 1 first name, 2 last name.
    focus: u8,
    draft_first: String,
    draft_last: String,
    draft_class: usize,
    pub note: String,
}

impl Default for Armory {
    fn default() -> Self {
        Self {
            page: Page::Heroes,
            focus: 0,
            draft_first: "Adventurer".into(),
            draft_last: "Stormwind".into(),
            draft_class: 0,
            note: String::new(),
        }
    }
}

#[derive(Component)]
pub(super) struct ArmoryRoot;

#[derive(Component, Clone)]
pub(super) enum Act {
    Page(Page),
    Pick(u32),
    Focus(u8),
    Class,
    Create,
    Zone(usize),
    Learn(usize),
    Respec,
}

pub(super) fn input(
    mut clicks: Query<(&Interaction, &Act), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut typing: ResMut<Typing>,
    mut armory: ResMut<Armory>,
    mut account: ResMut<Account>,
    mut selection: ResMut<Selection>,
    phase: Res<State<Phase>>,
) {
    if *phase.get() != Phase::Menu {
        typing.0 = false;
        return;
    }
    if selection.character.is_none() {
        let id = account
            .characters
            .iter()
            .find(|c| selection.class == Some(c.class))
            .or(account.characters.first())
            .map(|c| c.id);
        selection.character = id;
    }
    typing.0 = armory.focus != 0;
    if typing.0 && keys.just_pressed(KeyCode::Enter) {
        armory.focus = 0;
    } else if typing.0 {
        type_name(&keys, &mut armory);
    }
    for (interaction, act) in &mut clicks {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match act.clone() {
            Act::Page(page) => armory.page = page,
            Act::Focus(slot) => armory.focus = slot,
            Act::Class => {
                armory.draft_class = (armory.draft_class + 1) % CLASSES.len();
            }
            Act::Create => create_hero(&mut armory, &mut account, &mut selection),
            Act::Pick(id) => {
                let needs_look = pick_hero(&mut selection, &account, id);
                if needs_look {
                    armory.note = "Pick this hero's appearance on the class screen".into();
                }
            }
            Act::Zone(index) => pick_zone(&mut armory, &account, &mut selection, index),
            Act::Learn(node) => {
                if let Some(id) = selection.character {
                    if !account.learn(id, node) {
                        armory.note = "That talent is shut or the points are spent".into();
                    } else {
                        armory.note.clear();
                    }
                }
            }
            Act::Respec => {
                if let Some(id) = selection.character {
                    account.respec(id);
                    armory.note = "Talents refunded".into();
                }
            }
        }
    }
}

fn type_name(keys: &ButtonInput<KeyCode>, armory: &mut Armory) {
    let field = if armory.focus == 1 {
        &mut armory.draft_first
    } else {
        &mut armory.draft_last
    };
    if keys.just_pressed(KeyCode::Backspace) {
        field.pop();
        return;
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for key in keys.get_just_pressed() {
        let Some(ch) = letter(*key, shift) else {
            continue;
        };
        if field.chars().count() < 16 {
            field.push(ch);
        }
    }
}

fn letter(key: KeyCode, shift: bool) -> Option<char> {
    let lower = match key {
        KeyCode::KeyA => 'a',
        KeyCode::KeyB => 'b',
        KeyCode::KeyC => 'c',
        KeyCode::KeyD => 'd',
        KeyCode::KeyE => 'e',
        KeyCode::KeyF => 'f',
        KeyCode::KeyG => 'g',
        KeyCode::KeyH => 'h',
        KeyCode::KeyI => 'i',
        KeyCode::KeyJ => 'j',
        KeyCode::KeyK => 'k',
        KeyCode::KeyL => 'l',
        KeyCode::KeyM => 'm',
        KeyCode::KeyN => 'n',
        KeyCode::KeyO => 'o',
        KeyCode::KeyP => 'p',
        KeyCode::KeyQ => 'q',
        KeyCode::KeyR => 'r',
        KeyCode::KeyS => 's',
        KeyCode::KeyT => 't',
        KeyCode::KeyU => 'u',
        KeyCode::KeyV => 'v',
        KeyCode::KeyW => 'w',
        KeyCode::KeyX => 'x',
        KeyCode::KeyY => 'y',
        KeyCode::KeyZ => 'z',
        KeyCode::Space => ' ',
        KeyCode::Minus => '-',
        KeyCode::Quote => '\'',
        _ => return None,
    };
    Some(if shift && lower.is_ascii_alphabetic() {
        lower.to_ascii_uppercase()
    } else {
        lower
    })
}

fn create_hero(armory: &mut Armory, account: &mut Account, selection: &mut Selection) {
    match account.create(&armory.draft_first, &armory.draft_last, armory.draft_class) {
        Some(id) => {
            let needs_look = pick_hero(selection, account, id);
            armory.note = format!(
                "Created {} {}{}",
                armory.draft_first.trim(),
                armory.draft_last.trim(),
                if needs_look {
                    ". Pick their appearance on the class screen"
                } else {
                    ""
                }
            );
            armory.focus = 0;
        }
        None => armory.note = "A hero needs a first and last name".into(),
    }
}

/// Selects the hero. Returns whether their class changed, so the appearance has to be picked again.
fn pick_hero(selection: &mut Selection, account: &Account, id: u32) -> bool {
    let Some(class) = account.character(id).map(|c| c.class) else {
        return false;
    };
    let needs_look = selection.class != Some(class);
    if needs_look {
        selection.class = Some(class);
        selection.looks.clear();
        selection.look = None;
    }
    selection.character = Some(id);
    selection.version += 1;
    needs_look
}

fn pick_zone(armory: &mut Armory, account: &Account, selection: &mut Selection, index: usize) {
    let (level, clears) = hero_gate(account, selection);
    if let Some(reason) = campaign::zone_lock_reason(index, level, &clears) {
        armory.note = reason;
        return;
    }
    let Some(map_name) = ZONES.get(index).map(|z| z.map) else {
        return;
    };
    let Some(map) = data::MAPS.iter().position(|m| m.name == map_name) else {
        armory.note = format!("{map_name} is not on a battleground");
        return;
    };
    if selection.map != map {
        selection.map = map;
        selection.version += 1;
    }
    armory.note.clear();
}

pub(super) fn hero_gate(account: &Account, selection: &Selection) -> (u32, [bool; 7]) {
    let hero = selection
        .character
        .and_then(|id| account.character(id));
    let class = hero
        .map(|c| c.class)
        .or(selection.class)
        .unwrap_or(0);
    let level = hero
        .map(|c| campaign::talent_level(c.talent_xp))
        .unwrap_or(0);
    let clears = account
        .classes
        .get(class)
        .map(|p| p.clears)
        .unwrap_or([false; 7]);
    (level, clears)
}

pub(super) fn refresh(
    mut commands: Commands,
    menus: Query<Entity, (With<MenuRoot>, With<super::menu::MenuBuilt>)>,
    roots: Query<Entity, With<ArmoryRoot>>,
    font: Res<HudFont>,
    armory: Res<Armory>,
    account: Res<Account>,
    selection: Res<Selection>,
    phase: Res<State<Phase>>,
    mut drawn: Local<u64>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    let Ok(menu) = menus.single() else {
        return;
    };
    let root = if let Ok(root) = roots.single() {
        root
    } else {
        let root = commands
            .spawn((
                ArmoryRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    left: Val::Px(280.0),
                    width: Val::Px(700.0),
                    height: Val::Px(196.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(6.0)),
                    overflow: Overflow::clip(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
                GlobalZIndex(450),
            ))
            .id();
        commands.entity(menu).add_child(root);
        root
    };
    let sig = signature(&armory, &account, &selection);
    if *drawn == sig {
        return;
    }
    *drawn = sig;
    commands.entity(root).despawn_related::<Children>();
    let font = font.0.clone();
    commands.entity(root).with_children(|panel| {
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|row| {
                chip(row, &font, "Heroes", Act::Page(Page::Heroes), armory.page == Page::Heroes);
                chip(row, &font, "Zones", Act::Page(Page::Zones), armory.page == Page::Zones);
                chip(
                    row,
                    &font,
                    "Talents",
                    Act::Page(Page::Talents),
                    armory.page == Page::Talents,
                );
            });
        let note = if armory.note.is_empty() {
            "Pick a hero, then a zone. Appearance stays on the class screen."
        } else {
            armory.note.as_str()
        };
        panel.spawn(hud::text_bundle(&font, note, 13.0, GOLD));
        match armory.page {
            Page::Heroes => heroes(panel, &font, &armory, &account, &selection),
            Page::Zones => zones(panel, &font, &account, &selection),
            Page::Talents => talents(panel, &font, &account, &selection),
        }
    });
}

fn heroes(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            if account.characters.is_empty() {
                row.spawn(hud::text_bundle(
                    font,
                    "No heroes yet",
                    13.0,
                    Color::WHITE,
                ));
            }
            for hero in account.characters.iter().take(8) {
                let on = selection.character == Some(hero.id);
                let label = format!(
                    "{} {} — {}",
                    hero.first,
                    hero.last,
                    CLASSES.get(hero.class).map(|c| c.name).unwrap_or("Hero")
                );
                chip(row, font, &label, Act::Pick(hero.id), on);
            }
        });
    let class = CLASSES
        .get(armory.draft_class)
        .map(|c| c.name)
        .unwrap_or("Class");
    let first = field_label(&armory.draft_first, armory.focus == 1);
    let last = field_label(&armory.draft_last, armory.focus == 2);
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            chip(row, font, &first, Act::Focus(1), armory.focus == 1);
            chip(row, font, &last, Act::Focus(2), armory.focus == 2);
            chip(row, font, class, Act::Class, false);
            chip(row, font, "Create", Act::Create, false);
        });
}

fn zones(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let (level, clears) = hero_gate(account, selection);
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (index, zone) in ZONES.iter().enumerate() {
                let open = campaign::zone_open(index, level, &clears);
                let label = if open {
                    zone.name.to_string()
                } else {
                    format!("{} (locked)", zone.name)
                };
                let on = data::MAPS
                    .get(selection.map)
                    .is_some_and(|m| m.name == zone.map);
                chip(row, font, &label, Act::Zone(index), on);
            }
        });
}

fn talents(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let Some(id) = selection.character else {
        panel.spawn(hud::text_bundle(
            font,
            "Create a hero before spending talents",
            13.0,
            Color::WHITE,
        ));
        return;
    };
    let Some(hero) = account.character(id) else {
        return;
    };
    let spent: u32 = hero.talents.iter().map(|n| u32::from(*n)).sum();
    let unspent = campaign::unspent_talents(hero.talent_xp, spent);
    let level = campaign::talent_level(hero.talent_xp);
    panel.spawn(hud::text_bundle(
        font,
        format!("{unspent} unspent  ·  talent level {level}"),
        13.0,
        Color::WHITE,
    ));
    let trees = campaign::TREES
        .get(hero.class)
        .copied()
        .unwrap_or(["Tree", "Tree", "Tree"]);
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            for (tree, name) in trees.iter().enumerate() {
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(hud::text_bundle(font, *name, 13.0, GOLD));
                    for node in 0..campaign::NODES_PER_TREE {
                        let id_node = tree * campaign::NODES_PER_TREE + node;
                        let rank = hero.talents.get(id_node).copied().unwrap_or(0);
                        let max = campaign::NODE_RANKS[node];
                        chip(
                            col,
                            font,
                            &format!("Rank {rank}/{max}"),
                            Act::Learn(id_node),
                            false,
                        );
                    }
                });
            }
        });
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            chip(row, font, "Respec", Act::Respec, false);
        });
}

fn field_label(text: &str, focused: bool) -> String {
    let shown = if text.is_empty() { "name" } else { text };
    if focused {
        format!("{shown}_")
    } else {
        shown.to_string()
    }
}

fn signature(armory: &Armory, account: &Account, selection: &Selection) -> u64 {
    let mut n = match armory.page {
        Page::Heroes => 1,
        Page::Zones => 2,
        Page::Talents => 3,
    };
    n = fold(n, u64::from(armory.focus));
    n = fold(n, armory.draft_class as u64);
    n = fold(n, hash_str(&armory.draft_first));
    n = fold(n, hash_str(&armory.draft_last));
    n = fold(n, hash_str(&armory.note));
    n = fold(n, selection.character.unwrap_or(0) as u64);
    n = fold(n, u64::from(selection.version));
    n = fold(n, selection.map as u64);
    for hero in &account.characters {
        n = fold(n, u64::from(hero.id));
        n = fold(n, hero.class as u64);
        n = fold(n, u64::from(hero.talent_xp));
        n = fold(n, hash_str(&hero.first));
        n = fold(n, hash_str(&hero.last));
        for rank in hero.talents {
            n = fold(n, u64::from(rank));
        }
    }
    n
}

fn fold(n: u64, v: u64) -> u64 {
    n.wrapping_mul(131).wrapping_add(v)
}

fn hash_str(s: &str) -> u64 {
    s.bytes().fold(0u64, |n, b| fold(n, u64::from(b)))
}

/// What [`chip`] needs from a UI parent. Both the panel and a row satisfy it.
trait ChipParent {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands<'_>;
}

impl ChipParent for RelatedSpawnerCommands<'_, ChildOf> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands<'_> {
        RelatedSpawnerCommands::spawn(self, bundle)
    }
}

fn chip(parent: &mut impl ChipParent, font: &Handle<Font>, label: &str, act: Act, on: bool) {
    let color = if on {
        hud::rgb(0xc9a227)
    } else {
        Color::srgba(0.16, 0.13, 0.09, 0.95)
    };
    let ink = if on { Color::BLACK } else { Color::WHITE };
    parent
        .spawn((
            Button,
            act,
            Node {
                height: Val::Px(22.0),
                margin: UiRect::new(Val::Px(0.0), Val::Px(4.0), Val::Px(2.0), Val::Px(0.0)),
                padding: UiRect::horizontal(Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|b| {
            b.spawn(hud::text_bundle(font, label, 13.0, ink));
        });
}
