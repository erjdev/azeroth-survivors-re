//! The Armory: the front door before a run, and the draft pages a tab opens inside that frame.
//! Appearance stays on the glue race and class screen, reached from the hero column.

mod door;

use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use super::account::Account;
use super::campaign::{self, ZONES};
use super::data::{self, CLASSES};
use super::hud::{self, HudFont, MenuRoot};
use super::renown::{self, Renown};
use super::rewards::{self, Perk};
use super::stash::Stash;
use super::{Phase, Rng, Run, Selection, Typing};
use crate::glue::art::GlueArt;
use crate::glue::GlueClicks;

const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Page {
    Door,
    Heroes,
    Talents,
    Rewards,
    Collection,
    Codex,
    Achievements,
    Notes,
    DropTables,
    Look,
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
            page: Page::Door,
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
    Reward(u8),
    Restart,
    Update,
    PickClass(usize),
    RandomName,
    Look,
    Enter,
    RunOptions,
    Options,
    Quit,
    Renown(u8),
    Relic(usize),
    Upgrade,
    Seek,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    clicks: Res<GlueClicks>,
    acts: Query<(Entity, &Act)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut typing: ResMut<Typing>,
    mut armory: ResMut<Armory>,
    mut account: ResMut<Account>,
    mut selection: ResMut<Selection>,
    mut run: ResMut<Run>,
    mut renown: ResMut<Renown>,
    mut stash: ResMut<Stash>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<Phase>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    phase: Res<State<Phase>>,
    mut granted: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    if *phase.get() != Phase::Menu {
        typing.0 = false;
        return;
    }
    if !*granted {
        if account.grant_new() {
            account.save();
        }
        *granted = true;
    }
    if selection.character.is_none() {
        selection.character = account
            .characters
            .iter()
            .find(|c| selection.class == Some(c.class))
            .map(|c| c.id);
    }
    typing.0 = armory.focus != 0 && armory.page == Page::Heroes;
    if !typing.0 && keys.just_pressed(KeyCode::Escape) && armory.page != Page::Door {
        armory.page = Page::Door;
        armory.focus = 0;
    } else if typing.0 && keys.just_pressed(KeyCode::Enter) {
        armory.focus = 0;
    } else if typing.0 {
        type_name(&keys, &mut armory);
    }
    for (entity, act) in &acts {
        if !clicks.hit(entity) {
            continue;
        }
        match act.clone() {
            Act::Page(page) => {
                armory.page = page;
                armory.focus = 0;
            }
            Act::Focus(slot) => armory.focus = slot,
            Act::Class => {
                armory.draft_class = (armory.draft_class + 1) % CLASSES.len();
            }
            Act::Create => create_hero(&mut armory, &mut account, &mut selection),
            Act::Pick(id) => {
                let needs_look = pick_hero(&mut selection, &account, id);
                if needs_look {
                    armory.note = "Open Look to choose this hero's appearance.".into();
                }
            }
            Act::Zone(index) => {
                pick_zone(&mut armory, &account, &mut selection, index);
                super::menu::save_pick(&selection);
            }
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
            Act::Reward(code) => armory.note = buy_reward(&mut account, code).into(),
            Act::Restart => {
                super::notes::restart_same(&mut exit);
                armory.note = "Restarting".into();
            }
            Act::Update => armory.note = super::notes::restart_configured(&mut exit).into(),
            Act::PickClass(index) => pick_class(
                &mut armory,
                &account,
                &mut selection,
                creatures.as_deref(),
                index,
            ),
            Act::RandomName => random_name(&mut armory, &mut account, &selection, &mut rng),
            Act::Look => {
                armory.page = Page::Look;
                armory.focus = 0;
            }
            Act::Enter => try_enter(
                &mut armory,
                &account,
                &mut selection,
                &mut run,
                &renown,
                &stash,
                &mut next,
            ),
            Act::RunOptions => armory.note = "Difficulty is Normal.".into(),
            Act::Options => armory.page = Page::Notes,
            Act::Quit => {
                exit.write(AppExit::Success);
            }
            Act::Renown(index) => {
                if let Some(class) = selection.class {
                    let track = renown::tracks().get(usize::from(index)).copied();
                    if let Some(track) = track {
                        renown.spend(class, track);
                    }
                }
            }
            Act::Relic(slot) => stash.cycle_slot(slot),
            Act::Upgrade => {
                stash.upgrade();
            }
            Act::Seek => {
                stash.seek(rng.f32());
            }
        }
    }
}

fn buy_reward(account: &mut Account, code: u8) -> &'static str {
    let result = match code {
        0 => account.buy_perk(Perk::SecondThoughts),
        1 => account.buy_perk(Perk::Skip),
        2 => account.buy_perk(Perk::Veterans),
        3 => account.buy_perk(Perk::Banish),
        4 => account.buy_perk(Perk::OpeningHand),
        5 => account.buy_perk(Perk::KeenEye),
        6 => account.buy_leash(),
        7 => account.buy_paws(),
        8 => account.buy_instinct(1),
        9 => account.buy_instinct(2),
        10 => account.buy_instinct(3),
        11 => account.buy_second(),
        12 => account.reset_perks(),
        _ => Err("Unknown reward"),
    };
    match result {
        Ok(()) => "Bought",
        Err(reason) => reason,
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
                    ". Open Look for their appearance"
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
    armory.note = campaign::zone_lock_reason(index, level, &clears).unwrap_or_default();
}

/// Why Enter World must refuse the selected battleground. An open zone, and any map that is not
/// on the campaign path, returns nothing.
pub(super) fn enter_refused(account: &Account, selection: &Selection) -> Option<String> {
    let name = selection.map_def().name;
    let (level, clears) = hero_gate(account, selection);
    if campaign::map_allowed(name, level, &clears) {
        return None;
    }
    campaign::zone_index(name)
        .and_then(|index| campaign::zone_lock_reason(index, level, &clears))
        .or_else(|| Some(format!("{name} is locked")))
}

fn pick_class(
    armory: &mut Armory,
    account: &Account,
    selection: &mut Selection,
    creatures: Option<&crate::entities::Creatures>,
    index: usize,
) {
    let Some(creatures) = creatures else {
        armory.note = "The client data is still loading.".into();
        return;
    };
    super::menu::choose_class(selection, creatures, index);
    selection.character = account
        .characters
        .iter()
        .find(|c| c.class == index)
        .map(|c| c.id);
    selection.version += 1;
    armory.draft_class = index;
    armory.note.clear();
    super::menu::save_pick(selection);
}

fn random_name(armory: &mut Armory, account: &mut Account, selection: &Selection, rng: &mut Rng) {
    let (first, last) = name_pair(rng);
    if let Some(id) = selection.character {
        if account
            .character(id)
            .is_some_and(|c| selection.class == Some(c.class))
            && account.rename(id, &first, &last)
        {
            armory.note.clear();
            return;
        }
    }
    armory.draft_first = first;
    armory.draft_last = last;
    armory.draft_class = selection.class.unwrap_or(armory.draft_class);
    armory.note = "That name is ready on the Character tab.".into();
}

fn name_pair(rng: &mut Rng) -> (String, String) {
    const GIVEN: &[&str] = &[
        "Aryn", "Bram", "Cora", "Dain", "Elyn", "Fael", "Gorr", "Hale", "Ira", "Jorn", "Kael",
        "Lira", "Morn", "Nyla", "Orin", "Pyra", "Quinn", "Rook", "Sera", "Tarn", "Una", "Venn",
        "Wynn", "Yara",
    ];
    const FAMILY: &[&str] = &[
        "Ashfall",
        "Blackwood",
        "Cairn",
        "Dawnforge",
        "Evenstar",
        "Frost",
        "Goldleaf",
        "Hearth",
        "Ironbrow",
        "Keen",
        "Lightward",
        "Marsh",
        "North",
        "Oak",
        "Pine",
        "River",
        "Stone",
        "Thorn",
        "Underhill",
        "Vale",
    ];
    (
        GIVEN[rng.index(GIVEN.len())].to_string(),
        FAMILY[rng.index(FAMILY.len())].to_string(),
    )
}

fn try_enter(
    armory: &mut Armory,
    account: &Account,
    selection: &mut Selection,
    run: &mut Run,
    renown: &Renown,
    stash: &Stash,
    next: &mut NextState<Phase>,
) {
    if let Some(reason) = enter_refused(account, selection) {
        armory.note = reason;
        return;
    }
    let ranks = selection.class.map(|i| renown.ranks(i)).unwrap_or([0; 3]);
    let class = selection.class.unwrap_or(0);
    let hero = selection.character;
    if super::hud::start_run(
        run,
        selection,
        ranks,
        stash.gear(),
        account.bonus_picked(class, hero),
    ) {
        run.apply_perks(account.perks);
        next.set(Phase::Playing);
    } else {
        armory.note = "Pick a class and a look before entering.".into();
    }
}

pub(super) fn hero_gate(account: &Account, selection: &Selection) -> (u32, [bool; 7]) {
    let hero = selection.character.and_then(|id| account.character(id));
    let class = hero.map(|c| c.class).or(selection.class).unwrap_or(0);
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

#[allow(clippy::too_many_arguments)]
pub(super) fn refresh(
    mut commands: Commands,
    menus: Query<Entity, (With<MenuRoot>, With<super::menu::MenuBuilt>)>,
    mut roots: Query<&mut Node, With<ArmoryRoot>>,
    mut looks: Query<&mut Node, (With<super::menu::LookScreen>, Without<ArmoryRoot>)>,
    font: Res<HudFont>,
    art: Res<GlueArt>,
    armory: Res<Armory>,
    account: Res<Account>,
    selection: Res<Selection>,
    renown: Res<Renown>,
    stash: Res<Stash>,
    phase: Res<State<Phase>>,
    mut drawn: Local<Option<u64>>,
    mut root_id: Local<Option<Entity>>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    let show_door = armory.page != Page::Look;
    if let Ok(mut look) = looks.single_mut() {
        let display = if show_door {
            Display::None
        } else {
            Display::Flex
        };
        if look.display != display {
            look.display = display;
        }
    }
    if let Ok(mut node) = roots.single_mut() {
        let display = if show_door {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let sig = signature(&armory, &account, &selection, &renown, &stash);
    if *drawn == Some(sig) {
        return;
    }
    let Ok(menu) = menus.single() else {
        return;
    };
    *drawn = Some(sig);
    let root = if let Some(root) = *root_id {
        root
    } else {
        let root = commands
            .spawn((
                ArmoryRoot,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    display: if show_door {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.015, 0.01, 0.62)),
            ))
            .id();
        commands.entity(menu).add_child(root);
        *root_id = Some(root);
        root
    };
    if armory.page == Page::Look {
        return;
    }
    commands.entity(root).despawn_related::<Children>();
    let font = font.0.clone();
    let note = if armory.note.is_empty() && armory.page != Page::Door {
        "Pick a hero, then a zone. Appearance is on Look."
    } else {
        armory.note.as_str()
    };
    commands.entity(root).with_children(|panel| {
        door::build(
            panel, &art, &font, &armory, &account, &selection, &renown, &stash, note,
        );
    });
}

fn heroes(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            if account.characters.is_empty() {
                row.spawn(hud::text_bundle(font, "No heroes yet", 13.0, Color::WHITE));
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
            chip(row, font, "Appearance", Act::Look, false);
        });
    panel.spawn(hud::text_bundle(
        font,
        renown.status_line(selection.class),
        13.0,
        Color::WHITE,
    ));
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (index, track) in renown::tracks().into_iter().enumerate() {
                chip(row, font, track.label(), Act::Renown(index as u8), false);
            }
        });
    panel.spawn(hud::text_bundle(font, stash.line(), 13.0, GOLD));
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (slot, label) in [(0, "Relic I"), (1, "Relic II"), (2, "Relic III")] {
                chip(row, font, label, Act::Relic(slot), false);
            }
            chip(row, font, "Upgrade", Act::Upgrade, false);
            chip(row, font, "Seek relic", Act::Seek, false);
        });
}

fn fill_draft(
    panel: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
) {
    match armory.page {
        Page::Heroes => heroes(panel, font, armory, account, selection, renown, stash),
        Page::Talents => talents(panel, font, account, selection),
        Page::Rewards => rewards_page(panel, font, account),
        Page::Collection => collection(panel, font, account, selection),
        Page::Codex => codex(panel, font, account, selection),
        Page::Achievements => achievements(panel, font, account),
        Page::Notes => notes_page(panel, font),
        Page::DropTables => drop_tables(panel, font),
        Page::Door | Page::Look => {}
    }
}

fn drop_tables(panel: &mut impl ChipParent, font: &Handle<Font>) {
    scroll(
        panel,
        font,
        &[
            ("Drop tables are not listed yet.".into(), GOLD),
            (
                "A piece comes from a named creature. Clearing the zone does not, by itself, grant it.".into(),
                Color::WHITE,
            ),
        ],
    );
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

fn rewards_page(panel: &mut impl ChipParent, font: &Handle<Font>, account: &Account) {
    let earned = rewards::points_of(account.earned_ids());
    let have = rewards::earned_count(account.earned_ids());
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "{} to spend  ·  {earned} earned  ·  {have} of {}  ·  {} resets",
            account.unspent(),
            rewards::catalog_len(),
            account.resets_left()
        ),
        13.0,
        Color::WHITE,
    ));
    if let Some(next) = account.next_locked() {
        panel.spawn(hud::text_bundle(font, next, 13.0, GOLD));
    }
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (code, perk) in rewards::PERKS.into_iter().enumerate() {
                let rank = account.perks.rank(perk);
                let label = match rewards::perk_cost(perk, rank) {
                    Some(cost) => format!("{} {rank} · {cost}", rewards::perk_name(perk)),
                    None => format!("{} {rank} max", rewards::perk_name(perk)),
                };
                chip(row, font, &label, Act::Reward(code as u8), false);
            }
        });
    let leash = pet_label("Long Leash", account.pets.leash, 3);
    let paws = pet_label("Quick Paws", account.pets.paws, 3);
    let second = if account.pets.owned >= 2 {
        "Second companion max".to_string()
    } else {
        "Second companion · 40".to_string()
    };
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            chip(row, font, &leash, Act::Reward(6), false);
            chip(row, font, &paws, Act::Reward(7), false);
            chip(row, font, "Gold-sniffer · 30", Act::Reward(8), false);
            chip(row, font, "Wisp-herder · 30", Act::Reward(9), false);
            chip(row, font, "Rune-hound · 30", Act::Reward(10), false);
            chip(row, font, &second, Act::Reward(11), false);
            chip(
                row,
                font,
                &format!("Reset perks ({})", account.resets_left()),
                Act::Reward(12),
                false,
            );
        });
}

fn pet_label(name: &str, rank: u8, max: u8) -> String {
    if rank >= max {
        format!("{name} {rank} max")
    } else {
        let cost = [15, 25, 35][usize::from(rank)];
        format!("{name} {rank} · {cost}")
    }
}

const DIM: Color = Color::srgb(0.55, 0.5, 0.42);

fn picked_class(account: &Account, selection: &Selection) -> usize {
    selection
        .character
        .and_then(|id| account.character(id).map(|c| c.class))
        .or(selection.class)
        .unwrap_or(0)
        .min(CLASSES.len().saturating_sub(1))
}

fn scroll(panel: &mut impl ChipParent, font: &Handle<Font>, lines: &[(String, Color)]) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            height: Val::Px(390.0),
            overflow: Overflow::scroll_y(),
            ..default()
        })
        .with_children(|col| {
            for (line, color) in lines {
                col.spawn(hud::text_bundle(font, line.clone(), 13.0, *color));
            }
        });
}

fn collection(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let class = picked_class(account, selection);
    let names = campaign::GEAR_NAMES.get(class);
    let progress = account.classes.get(class);
    let class_name = CLASSES.get(class).map(|c| c.name).unwrap_or("Hero");
    let mut lines = Vec::new();
    let owned = progress
        .map(|p| p.gear.iter().filter(|g| **g).count())
        .unwrap_or(0);
    let have_weapon = progress.map(|p| p.legendary_weapon).unwrap_or(false);
    lines.push((
        format!("{class_name} collection — {owned} of 10 pieces"),
        GOLD,
    ));
    for slot in 0..campaign::SLOTS.len() {
        let have = progress
            .and_then(|p| p.gear.get(slot).copied())
            .unwrap_or(false);
        let slot_name = campaign::SLOTS[slot];
        let item = names.map(|row| row[slot]).unwrap_or("Unknown");
        let quality = campaign::slot_quality(slot)
            .map(campaign::quality_name)
            .unwrap_or("?");
        let main_filled = have_weapon
            || progress
                .and_then(|p| p.gear.get(8).copied())
                .unwrap_or(false);
        let occupied = slot == 9 && campaign::two_handed(class) && main_filled;
        let (text, color) = if occupied {
            (
                format!("{slot_name} — filled by the two-handed weapon"),
                Color::WHITE,
            )
        } else if have {
            (format!("{slot_name} — {item} ({quality})"), Color::WHITE)
        } else {
            (
                format!("{slot_name} — {item} ({quality}) — not collected"),
                DIM,
            )
        };
        lines.push((text, color));
    }
    if campaign::two_handed(class) {
        lines.push((
            "The weapon is two-handed, so the off-hand piece is not worn.".into(),
            DIM,
        ));
    }
    let weapon = names.map(|row| row[10]).unwrap_or("Legendary weapon");
    lines.push(if have_weapon {
        (format!("Legendary weapon — {weapon}"), GOLD)
    } else {
        (format!("Legendary weapon — {weapon} — the raid"), DIM)
    });
    if let Some(id) = selection.character {
        if account.character(id).is_some_and(|c| c.class == class) {
            let cloak = campaign::cloak_quality(account.feats(class, id));
            lines.push(match cloak {
                Some(q) => (format!("Cloak — {}", campaign::quality_name(q)), GOLD),
                None => ("Cloak — no feats yet".into(), DIM),
            });
        }
    }
    let gear = progress.map(|p| p.gear).unwrap_or([false; 10]);
    if campaign::set_complete(&gear, have_weapon, campaign::two_handed(class)) {
        lines.push(("Set bonus — the ten slots are filled.".into(), GOLD));
    }
    scroll(panel, font, &lines);
}

fn codex(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let class_i = picked_class(account, selection);
    let Some(class) = CLASSES.get(class_i) else {
        return;
    };
    let seen = account
        .classes
        .get(class_i)
        .map(|p| p.seen.as_slice())
        .unwrap_or(&[]);
    let mut lines = vec![(
        format!(
            "{} spells. A legendary amulet is marked when its card is dealt.",
            class.name
        ),
        GOLD,
    )];
    let mut keys = Vec::new();
    for key in std::iter::once(class.start).chain(class.pool.iter().copied()) {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    for key in keys {
        let def = data::ability(key);
        let amulet = campaign::legendary_amulet(key);
        let found = seen.iter().any(|name| name == &amulet);
        let color = if found { GOLD } else { Color::WHITE };
        let mark = if found {
            "amulet seen"
        } else {
            "amulet unseen"
        };
        lines.push((
            format!(
                "{} — {} — {} damage, {}s cooldown — {mark}",
                spell_title(key),
                kind_name(def.kind),
                def.damage,
                def.cooldown
            ),
            color,
        ));
    }
    lines.push((
        format!(
            "Rings, worn {}: {}",
            campaign::RINGS_WORN,
            campaign::RING_NAMES.join(", ")
        ),
        Color::WHITE,
    ));
    lines.push((
        format!(
            "Trinkets, worn {}: {}",
            campaign::TRINKETS_WORN,
            campaign::TRINKET_NAMES.join(", ")
        ),
        Color::WHITE,
    ));
    lines.push((
        format!(
            "Amulets, worn {}: {}",
            campaign::AMULETS_WORN,
            campaign::AMULET_NAMES.join(", ")
        ),
        Color::WHITE,
    ));
    scroll(panel, font, &lines);
}

fn achievements(panel: &mut impl ChipParent, font: &Handle<Font>, account: &Account) {
    let rows = account.achievement_list();
    let earned = rows.iter().filter(|row| row.earned).count();
    let mut lines = vec![(
        format!(
            "{earned} of {} achievements. Points are spent on the Rewards page.",
            rows.len()
        ),
        GOLD,
    )];
    for row in rows {
        let (mark, color) = if row.earned {
            ("Earned", GOLD)
        } else {
            ("Open", DIM)
        };
        lines.push((
            format!("{mark} — {} — {} points", row.name, row.points),
            color,
        ));
    }
    scroll(panel, font, &lines);
}

fn spell_title(key: &str) -> String {
    key.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn kind_name(kind: data::Kind) -> &'static str {
    match kind {
        data::Kind::Bolt => "Bolt",
        data::Kind::Strike => "Strike",
        data::Kind::Melee => "Melee",
        data::Kind::Nova => "Nova",
        data::Kind::Cone => "Cone",
        data::Kind::Ground => "Ground",
        data::Kind::GroundSelf => "Ground",
        data::Kind::Chain => "Chain",
        data::Kind::Dot => "Damage over time",
        data::Kind::Aura => "Aura",
        data::Kind::Orbit => "Orbit",
        data::Kind::Shield => "Shield",
        data::Kind::Immune => "Immune",
        data::Kind::Thorns => "Thorns",
        data::Kind::Heal => "Heal",
        data::Kind::Totem => "Totem",
    }
}

fn notes_page(panel: &mut impl ChipParent, font: &Handle<Font>) {
    let mut lines: Vec<(String, Color)> = super::notes::NOTES
        .iter()
        .map(|line| ((*line).to_string(), Color::WHITE))
        .collect();
    lines.insert(0, ("Patch notes".into(), GOLD));
    scroll(panel, font, &lines);
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            chip(row, font, "Restart", Act::Restart, false);
            chip(row, font, "Update and restart", Act::Update, false);
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

fn signature(
    armory: &Armory,
    account: &Account,
    selection: &Selection,
    renown: &Renown,
    stash: &Stash,
) -> u64 {
    let mut n = match armory.page {
        Page::Door => 0,
        Page::Heroes => 1,
        Page::Talents => 3,
        Page::Rewards => 4,
        Page::Collection => 5,
        Page::Codex => 6,
        Page::Achievements => 7,
        Page::Notes => 8,
        Page::DropTables => 9,
        Page::Look => 10,
    };
    n = fold(n, u64::from(account.unspent()));
    n = fold(n, account.earned_ids().len() as u64);
    n = fold(n, u64::from(account.resets_left()));
    let perks = account.perks;
    n = fold(n, u64::from(perks.second_thoughts));
    n = fold(n, u64::from(perks.skip));
    n = fold(n, u64::from(perks.veterans));
    n = fold(n, u64::from(perks.banish));
    n = fold(n, u64::from(perks.opening_hand));
    n = fold(n, u64::from(perks.keen_eye));
    n = fold(n, u64::from(account.pets.owned));
    n = fold(n, u64::from(account.pets.leash));
    n = fold(n, u64::from(account.pets.paws));
    n = fold(n, u64::from(account.pets.instinct[0]));
    n = fold(n, u64::from(account.pets.instinct[1]));
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
    for class in &account.classes {
        for piece in class.gear {
            n = fold(n, u64::from(piece));
        }
        n = fold(n, u64::from(class.legendary_weapon));
        n = fold(n, class.seen.len() as u64);
        for cleared in class.clears {
            n = fold(n, u64::from(cleared));
        }
    }
    n = fold(n, hash_str(&renown.status_line(selection.class)));
    n = fold(n, hash_str(&stash.line()));
    n = fold(n, selection.look.unwrap_or(0) as u64);
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

#[cfg(test)]
mod tests {
    use super::super::account;
    use super::super::data;
    use super::super::Selection;
    use super::enter_refused;

    #[test]
    fn a_locked_zone_does_not_start() {
        let account = account::blank();
        let mut selection = Selection::default();
        let elwynn = data::MAPS
            .iter()
            .position(|m| m.name == "Elwynn Forest")
            .unwrap();
        selection.map = elwynn;
        assert!(enter_refused(&account, &selection).is_none());
        let duskwood = data::MAPS
            .iter()
            .position(|m| m.name == "Duskwood")
            .unwrap();
        selection.map = duskwood;
        assert!(enter_refused(&account, &selection).is_some());
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
