//! The Armory: the front door before a run, and the draft pages a tab opens inside that frame.
//! Appearance stays on the glue race and class screen, reached from the hero column.

mod collection;
mod door;
mod talent_page;
pub(super) use talent_page::fill_art as fill_talent_art;

pub(super) use collection::scroll as scroll_collection;

use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::system::SystemParam;
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

/// Clicks and the mouse button, one system parameter so the menu system stays within Bevy's limit.
#[derive(SystemParam)]
pub(super) struct Pointer<'w> {
    clicks: Res<'w, GlueClicks>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
}

/// Creature looks and the talent catalog, one system parameter so the menu stays within Bevy's limit.
#[derive(SystemParam)]
pub(super) struct MenuBooks<'w> {
    creatures: Option<Res<'w, crate::entities::Creatures>>,
    talents: Option<Res<'w, crate::ui_talent::Talents>>,
    spells: Option<Res<'w, crate::ui_action::Spells>>,
}

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
    /// The campaign row the door is showing. None follows the selected battleground.
    pub(super) picked: Option<usize>,
    pub note: String,
    /// 0 summary, then Zones, Classes, Collection, Feats, Lifetime.
    achieve: u8,
    /// 0 perks, 1 companions, 2 tabards.
    reward_page: u8,
    /// Which companion the Looks row dresses.
    pet_slot: u8,
    piece: usize,
}

impl Default for Armory {
    fn default() -> Self {
        Self {
            page: Page::Door,
            focus: 0,
            draft_first: "Adventurer".into(),
            draft_last: "Stormwind".into(),
            draft_class: 0,
            picked: None,
            note: String::new(),
            achieve: 0,
            reward_page: 0,
            pet_slot: 0,
            piece: 0,
        }
    }
}

impl Armory {
    /// Keep the selected campaign row authoritative when opening another Armory tab.
    pub(super) fn entered_zone(&self) -> Option<usize> {
        if self.page == Page::Look {
            None
        } else {
            self.picked
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
    Spec(u8),
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
    Codex(u16),
    Achieve(u8),
    RewardPage(u8),
    WearLook(u8),
    PetSlot(u8),
    Tabard(u8),
    Talent(u32),
    InspectPiece(usize),
    WearPiece(usize, Option<u8>),
    ShowPiece(usize, super::equipment::Look),
}

#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    pointer: Pointer,
    acts: Query<(Entity, &Act, &Interaction)>,
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
    books: MenuBooks,
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
    if pointer.mouse.just_released(MouseButton::Right) {
        for (_, act, interaction) in &acts {
            if *interaction == Interaction::None {
                continue;
            }
            match act {
                Act::InspectPiece(index) => {
                    let class = picked_class(&account, &selection);
                    let slot = index % campaign::GRID_SLOTS;
                    let band = (index / campaign::GRID_SLOTS) as u8;
                    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                        account.show_piece(class, slot, super::equipment::Look::Piece(band));
                    } else {
                        let worn = account.classes[class].equipment.worn[slot];
                        account.wear_piece(
                            class,
                            slot,
                            if worn == Some(band) { None } else { Some(band) },
                        );
                    }
                    account.save();
                }
                Act::Learn(node) => {
                    if let Some(id) = selection.character {
                        if account.unlearn(id, *node) {
                            armory.note.clear();
                        } else {
                            armory.note = "That point is holding a deeper row open".into();
                        }
                    }
                }
                Act::Talent(talent) => {
                    if let Some(id) = selection.character {
                        armory.note = spend_talent(
                            &mut account,
                            id,
                            books.talents.as_deref().map(|t| &t.catalog),
                            books.spells.as_deref().map(|s| &s.catalog),
                            *talent,
                            true,
                        );
                    }
                }
                _ => {}
            }
        }
    }
    for (entity, act, _) in &acts {
        if !pointer.clicks.hit(entity) {
            continue;
        }
        match act.clone() {
            Act::Page(page) => {
                armory.page = page;
                armory.focus = 0;
            }
            Act::InspectPiece(index) => armory.piece = index.min(campaign::PIECES - 1),
            Act::WearPiece(slot, band) => {
                let class = picked_class(&account, &selection);
                if account.wear_piece(class, slot, band) {
                    account.save();
                }
            }
            Act::ShowPiece(slot, look) => {
                let class = picked_class(&account, &selection);
                if account.show_piece(class, slot, look) {
                    account.save();
                }
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
            Act::Learn(node) if pointer.mouse.just_released(MouseButton::Right) => {
                let _ = node;
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
            Act::Spec(spec) => {
                if let Some(id) = selection.character {
                    account.select_spec(id, spec);
                    armory.note.clear();
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
                armory.note = super::notes::restart_same(&mut exit).into();
            }
            Act::Update => armory.note = super::notes::restart_configured(&mut exit).into(),
            Act::PickClass(index) => pick_class(
                &mut armory,
                &account,
                &mut selection,
                books.creatures.as_deref(),
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
            Act::Codex(id) => armory.note = codex_text(&account, id),
            Act::Achieve(page) => {
                armory.achieve = page.min(5);
                armory.page = Page::Achievements;
            }
            Act::RewardPage(page) => {
                armory.reward_page = page.min(2);
                armory.page = Page::Rewards;
            }
            Act::PetSlot(slot) => armory.pet_slot = slot.min(1),
            Act::WearLook(look) => {
                account.wear_look(armory.pet_slot, look);
            }
            Act::Talent(talent) if pointer.mouse.just_released(MouseButton::Right) => {
                let _ = talent;
            }
            Act::Talent(talent) => {
                if let Some(id) = selection.character {
                    armory.note = spend_talent(
                        &mut account,
                        id,
                        books.talents.as_deref().map(|t| &t.catalog),
                        books.spells.as_deref().map(|s| &s.catalog),
                        talent,
                        false,
                    );
                }
            }
            Act::Tabard(index) => {
                account.wear_tabard(index);
                armory.note = if account.tabard == 0 {
                    "No tabard. A tabard grants no stat.".into()
                } else {
                    "Tabard worn. A tabard grants no stat.".into()
                };
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
            account.save();
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
    super::menu::save_pick(selection);
    needs_look
}

fn pick_zone(armory: &mut Armory, account: &Account, selection: &mut Selection, index: usize) {
    let (level, clears) = hero_gate(account, selection);
    let Some(zone) = ZONES.get(index) else {
        return;
    };
    armory.picked = Some(index);
    if let Some(map) = data::MAPS.iter().position(|m| m.name == zone.map) {
        if selection.map != map {
            selection.map = map;
            selection.version += 1;
        }
    }
    armory.note = campaign::zone_lock_reason(index, level, &clears).unwrap_or_default();
}

/// Why Enter World must refuse the selected battleground. An open zone, and any map that is not
/// on the campaign path, returns nothing.
pub(super) fn enter_refused(
    account: &Account,
    selection: &Selection,
    picked: Option<usize>,
) -> Option<String> {
    let (level, clears) = hero_gate(account, selection);
    if let Some(index) = picked {
        if let Some(reason) = campaign::zone_lock_reason(index, level, &clears) {
            return Some(reason);
        }
        if let Some(zone) = ZONES.get(index) {
            if selection.map_def().name != zone.map {
                return Some(format!("{} is not on a battleground", zone.name));
            }
        }
    }
    let name = selection.map_def().name;
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
    if let Some(reason) = enter_refused(account, selection, armory.entered_zone()) {
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

pub(super) fn hero_gate(
    account: &Account,
    selection: &Selection,
) -> (u32, [bool; campaign::CLEAR_COUNT]) {
    let hero = selection.character.and_then(|id| account.character(id));
    let class = hero.map(|c| c.class).or(selection.class).unwrap_or(0);
    let level = hero
        .map(|c| campaign::talent_level(c.talent_xp))
        .unwrap_or(0);
    let clears = account
        .classes
        .get(class)
        .map(|p| p.clears)
        .unwrap_or([false; campaign::CLEAR_COUNT]);
    (level, clears)
}

#[derive(SystemParam)]
pub(super) struct ArmoryArt<'w> {
    font: Res<'w, HudFont>,
    art: Res<'w, GlueArt>,
    layout: Res<'w, super::tuning::UiLayout>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn refresh(
    mut commands: Commands,
    menus: Query<Entity, (With<MenuRoot>, With<super::menu::MenuBuilt>)>,
    mut roots: Query<&mut Node, With<ArmoryRoot>>,
    mut looks: Query<&mut Node, (With<super::menu::LookScreen>, Without<ArmoryRoot>)>,
    assets: ArmoryArt,
    armory: Res<Armory>,
    account: Res<Account>,
    selection: Res<Selection>,
    renown: Res<Renown>,
    stash: Res<Stash>,
    talents: Option<Res<crate::ui_talent::Talents>>,
    phase: Res<State<Phase>>,
    mut drawn: Local<Option<(u64, u64)>>,
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
    let sig = (
        signature(&armory, &account, &selection, &renown, &stash),
        assets.layout.version,
    );
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
    commands
        .entity(root)
        .insert(BackgroundColor(if armory.page == Page::Talents {
            Color::srgb(0.012, 0.01, 0.006)
        } else {
            Color::srgba(0.02, 0.015, 0.01, 0.62)
        }));
    commands.entity(root).despawn_related::<Children>();
    let font = assets.font.0.clone();
    let note =
        if armory.note.is_empty() && armory.page != Page::Door && armory.page != Page::Talents {
            "Pick a hero, then a zone. Appearance is on Look."
        } else {
            armory.note.as_str()
        };
    commands.entity(root).with_children(|panel| {
        door::build(
            panel,
            &assets.art,
            &font,
            &armory,
            &account,
            &selection,
            &renown,
            &stash,
            note,
            talents.as_deref().map(|loaded| &loaded.catalog),
            &assets.layout.value,
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
    catalog: Option<&benilla_formats::TalentCatalog>,
) {
    match armory.page {
        Page::Heroes => heroes(panel, font, armory, account, selection, renown, stash),
        Page::Talents => talents(panel, font, account, selection, catalog),
        Page::Rewards => rewards_page(panel, font, armory, account),
        Page::Collection => {}
        Page::Codex => codex(panel, font, account),
        Page::Achievements => achievements(panel, font, armory, account),
        Page::Notes => notes_page(panel, font),
        Page::DropTables => drop_tables(panel, font, account, selection),
        Page::Door | Page::Look => {}
    }
}

fn drop_tables(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
) {
    let class = picked_class(account, selection);
    let mut lines = vec![(
        "A piece comes from a named creature. Clearing the zone does not, by itself, grant it."
            .into(),
        Color::WHITE,
    )];
    for (index, zone) in campaign::ZONES.iter().enumerate() {
        lines.push((zone.name.to_string(), GOLD));
        let rows: Vec<_> = campaign::DROPS
            .iter()
            .filter(|row| row.zone == index)
            .collect();
        if rows.is_empty() {
            lines.push((
                "No drop table yet. The creature has to exist on this map.".into(),
                DIM,
            ));
            continue;
        }
        for row in rows {
            let item = campaign::piece_name(class, row.slot, row.band).unwrap_or("unnamed");
            let slot = campaign::SLOTS.get(row.slot).copied().unwrap_or("Piece");
            let band = campaign::BANDS_NAMED.get(row.band).copied().unwrap_or("");
            let minute = row.at_secs / 60;
            lines.push((
                format!(
                    "{slot} — {item} ({band}) — {} at {minute}:00 — {}%",
                    row.creature, row.chance
                ),
                Color::WHITE,
            ));
        }
    }
    scroll(panel, font, &lines);
}

fn class_trees(
    catalog: &benilla_formats::TalentCatalog,
    class_id: u8,
) -> Vec<(String, Vec<benilla_formats::Talent>)> {
    let mut ids = Vec::new();
    for race in 1u8..12 {
        let tabs = catalog.tabs_for_class(race, class_id);
        if tabs.len() >= 3 {
            ids = tabs
                .into_iter()
                .take(3)
                .map(|tab| (tab.id, tab.name.clone()))
                .collect();
            break;
        }
    }
    ids.into_iter()
        .map(|(id, name)| (name, catalog.talents_in_tab(id).to_vec()))
        .collect()
}

fn dbc_points(hero: &super::account::Character, spec: usize) -> [u32; 3] {
    let mut points = [0u32; 3];
    for &(tree, _, rank) in &hero.dbc[spec.min(2)] {
        if tree < 3 {
            points[usize::from(tree)] += u32::from(rank);
        }
    }
    points
}

#[derive(Component)]
pub(super) struct TalentDetail;

#[derive(Component)]
pub(super) struct TalentInfo {
    class: usize,
    id: u32,
    spell: u32,
    rank: u8,
    max: u8,
    milestone: Option<(usize, u32)>,
}

/// Update the detail line without rebuilding the tree under the pointer.
pub(super) fn talent_details(
    nodes: Query<(&Interaction, &TalentInfo)>,
    mut detail: Query<&mut Text, With<TalentDetail>>,
    spells: Option<Res<crate::ui_action::Spells>>,
) {
    let Some((_, node)) = nodes.iter().find(|(state, _)| **state != Interaction::None) else {
        return;
    };
    if let Some((tree, at)) = node.milestone {
        let effect = survivors_core::talents::description(node.class, tree, at)
            .unwrap_or("Individual milestone effect pending; provisional tree stat bonuses apply.");
        let value = format!(
            "{} · {at} points\n{effect}",
            campaign::milestone_name(node.class, tree, at)
        );
        for mut text in &mut detail {
            if **text != value {
                **text = value.clone();
            }
        }
        return;
    }
    let name = spells
        .as_deref()
        .and_then(|spells| spells.catalog.get(node.spell))
        .map(|spell| spell.name.as_str())
        .unwrap_or("Talent");
    let effect = survivors_core::talents::stat_talent(node.class, node.id)
        .map(|node| node.description)
        .unwrap_or("Individual effect pending; currently +1% damage per rank.");
    let value = format!("{name}  {}/{} — Survivors: {effect}", node.rank, node.max);
    for mut text in &mut detail {
        if **text != value {
            **text = value.clone();
        }
    }
}

fn draw_real_talents(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    hero: &super::account::Character,
    catalog: &benilla_formats::TalentCatalog,
) -> bool {
    let class_id = CLASSES.get(hero.class).map(|c| c.id).unwrap_or(1);
    let trees = class_trees(catalog, class_id);
    if trees.len() < 3 {
        return false;
    }
    panel.spawn((
        TalentDetail,
        hud::text_bundle(
            font,
            "Hover a talent to inspect its rank and current Survivors effect.",
            13.0,
            Color::WHITE,
        ),
    ));
    let active = usize::from(hero.spec).min(2);
    let points = dbc_points(hero, active);
    let spent: u32 = points.iter().sum();
    let unspent = campaign::unspent_talents(hero.talent_xp, spent);
    let level = campaign::talent_level(hero.talent_xp);
    panel.spawn(hud::text_bundle(
        font,
        format!("{unspent} to spend  ·  {spent} of {level} earned spent"),
        13.0,
        Color::WHITE,
    ));
    if hero.spent() > 0 && spent == 0 {
        panel.spawn(hud::text_bundle(
            font,
            "Spending on this tree returns the old spread to unspent.",
            12.0,
            DIM,
        ));
    }
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            for spec in 0..3u8 {
                let pts = dbc_points(hero, usize::from(spec));
                let label = if pts.iter().all(|n| *n == 0) {
                    format!("Spec {}  Empty", spec + 1)
                } else {
                    format!("Spec {}  {}/{}/{}", spec + 1, pts[0], pts[1], pts[2])
                };
                chip(row, font, &label, Act::Spec(spec), hero.spec == spec);
            }
            chip(row, font, "Reset", Act::Respec, false);
        });
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            for (tree, (name, talents)) in trees.iter().enumerate() {
                let tree_points = points[tree];
                row.spawn(Node {
                    width: Val::Px(160.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(hud::text_bundle(
                        font,
                        format!("{name}  {tree_points}"),
                        13.0,
                        GOLD,
                    ));
                    for tier in 0..7u32 {
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(2.0),
                            ..default()
                        })
                        .with_children(|line| {
                            for column in 0..4u32 {
                                let talent =
                                    talents.iter().find(|t| t.row == tier && t.col == column);
                                if let Some(talent) = talent {
                                    let rank = hero.dbc[active]
                                        .iter()
                                        .find(|entry| entry.1 == talent.id)
                                        .map(|entry| entry.2)
                                        .unwrap_or(0);
                                    let dim = if rank == 0 { 0.4 } else { 1.0 };
                                    line.spawn((
                                        Button,
                                        Act::Talent(talent.id),
                                        TalentInfo {
                                            class: hero.class,
                                            id: talent.id,
                                            spell: talent.ranks[0],
                                            rank,
                                            max: talent.max_rank() as u8,
                                            milestone: None,
                                        },
                                        Node {
                                            width: Val::Px(28.0),
                                            height: Val::Px(28.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor::all(if rank > 0 { GOLD } else { DIM }),
                                        ImageNode {
                                            color: Color::srgba(1.0, 1.0, 1.0, dim),
                                            ..default()
                                        },
                                        hud::SpellIcon(talent.ranks[0]),
                                        hud::IconDim(dim),
                                    ));
                                } else {
                                    line.spawn(Node {
                                        width: Val::Px(28.0),
                                        height: Val::Px(28.0),
                                        ..default()
                                    });
                                }
                            }
                        });
                    }
                    for at in [10, 20, 30] {
                        col.spawn(hud::text_bundle(
                            font,
                            format!(
                                "{at}  {}{}",
                                campaign::milestone_name(hero.class, tree, at),
                                survivors_core::talents::description(hero.class, tree, at)
                                    .map(|effect| format!("\n{effect}"))
                                    .unwrap_or_default()
                            ),
                            12.0,
                            if tree_points >= at { GOLD } else { DIM },
                        ));
                    }
                });
            }
        });
    panel.spawn(hud::text_bundle(
        font,
        "Click spends a point. Right-click takes one back. Reset clears this spec.",
        12.0,
        DIM,
    ));
    true
}

fn talent_ranks_ok(list: &[benilla_formats::Talent], ranks: &[(u32, u8)]) -> bool {
    let nodes: Vec<_> = list
        .iter()
        .map(|talent| survivors_core::talents::RankedTalent {
            id: talent.id,
            row: talent.row,
            max_rank: talent.max_rank() as u8,
            rank: ranks
                .iter()
                .find(|(id, _)| *id == talent.id)
                .map_or(0, |(_, rank)| *rank),
            prerequisite: (talent.prereq_talent != 0)
                .then_some((talent.prereq_talent, talent.prereq_rank.saturating_add(1))),
        })
        .collect();
    survivors_core::talents::valid_tree(&nodes)
}

fn spend_talent(
    account: &mut Account,
    hero: u32,
    catalog: Option<&benilla_formats::TalentCatalog>,
    spells: Option<&benilla_formats::SpellCatalog>,
    talent_id: u32,
    refund: bool,
) -> String {
    let Some(catalog) = catalog else {
        return "Talents are still loading".into();
    };
    let Some(class) = account.character(hero).map(|c| c.class) else {
        return "Create a hero before spending talents".into();
    };
    let class_id = CLASSES.get(class).map(|c| c.id).unwrap_or(1);
    let trees = class_trees(catalog, class_id);
    let Some((tree, list)) = trees.iter().enumerate().find_map(|(tree, (_, list))| {
        list.iter()
            .any(|t| t.id == talent_id)
            .then_some((tree, list))
    }) else {
        return "That talent is not on this class".into();
    };
    let Some(talent) = list.iter().find(|t| t.id == talent_id) else {
        return "That talent is not on this class".into();
    };
    let name = spells
        .and_then(|spells| spells.get(talent.ranks[0]))
        .map(|spell| spell.name.clone())
        .unwrap_or_else(|| "Talent".into());
    let current = account
        .character(hero)
        .map(|c| c.dbc_rank(talent_id))
        .unwrap_or(0);
    if refund {
        if current == 0 {
            return format!("{name} has no point to take back");
        }
        let ranks: Vec<(u32, u8)> = list
            .iter()
            .map(|t| {
                let rank = account
                    .character(hero)
                    .map(|c| c.dbc_rank(t.id))
                    .unwrap_or(0);
                let rank = if t.id == talent_id {
                    rank.saturating_sub(1)
                } else {
                    rank
                };
                (t.id, rank)
            })
            .collect();
        if !account.unlearn_talent(hero, talent_id, talent_ranks_ok(list, &ranks)) {
            return "That point is holding a deeper row open".into();
        }
        return format!("{name} rank {}", current - 1);
    }
    let prereq_met = if talent.prereq_talent == 0 {
        true
    } else {
        let need = talent.prereq_rank as u8 + 1;
        account
            .character(hero)
            .map(|c| c.dbc_rank(talent.prereq_talent) >= need)
            .unwrap_or(false)
    };
    let proposed: Vec<_> = list
        .iter()
        .map(|entry| {
            let rank = account
                .character(hero)
                .map_or(0, |hero| hero.dbc_rank(entry.id));
            (
                entry.id,
                if entry.id == talent_id {
                    rank.saturating_add(1)
                } else {
                    rank
                },
            )
        })
        .collect();
    if !talent_ranks_ok(list, &proposed) {
        return if !prereq_met {
            "That talent's prerequisite is missing".into()
        } else {
            "That talent needs more points in earlier rows or is already full".into()
        };
    }
    if account.learn_talent(
        hero,
        tree as u8,
        talent_id,
        talent.max_rank() as u8,
        talent.row as u8,
        prereq_met,
    ) {
        format!("{name} rank {}", current + 1)
    } else if !prereq_met {
        "That talent's prerequisite is missing".into()
    } else {
        "That talent is shut or the points are spent".into()
    }
}

fn talents(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
    catalog: Option<&benilla_formats::TalentCatalog>,
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
    if let Some(catalog) = catalog {
        if draw_real_talents(panel, font, hero, catalog) {
            return;
        }
    }
    let spent = hero.spent();
    let unspent = campaign::unspent_talents(hero.talent_xp, spent);
    let level = campaign::talent_level(hero.talent_xp);
    panel.spawn(hud::text_bundle(
        font,
        format!("{unspent} to spend  ·  {spent} of {level} earned spent"),
        13.0,
        Color::WHITE,
    ));
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            for spec in 0..campaign::SPEC_COUNT as u8 {
                let ranks = &hero.specs[usize::from(spec)];
                let points: [u32; 3] = [
                    campaign::points_in_tree(ranks, 0),
                    campaign::points_in_tree(ranks, 1),
                    campaign::points_in_tree(ranks, 2),
                ];
                let label = if points.iter().all(|n| *n == 0) {
                    format!("Spec {}  Empty", spec + 1)
                } else {
                    format!(
                        "Spec {}  {}/{}/{}",
                        spec + 1,
                        points[0],
                        points[1],
                        points[2]
                    )
                };
                chip(row, font, &label, Act::Spec(spec), hero.spec == spec);
            }
            chip(row, font, "Reset", Act::Respec, false);
        });
    let trees = campaign::TREES
        .get(hero.class)
        .copied()
        .unwrap_or(["Tree", "Tree", "Tree"]);
    let active = hero.ranks();
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            for (tree, name) in trees.iter().enumerate() {
                let points = campaign::points_in_tree(active, tree);
                row.spawn(Node {
                    width: Val::Px(240.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(hud::text_bundle(
                        font,
                        format!("{name}  {points}"),
                        13.0,
                        GOLD,
                    ));
                    col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|grid| {
                        for node in 0..campaign::NODES_PER_TREE {
                            let id_node = tree * campaign::NODES_PER_TREE + node;
                            let rank = active.get(id_node).copied().unwrap_or(0);
                            let max = campaign::NODE_RANKS[node];
                            let label = milestone_label(hero.class, tree, node, rank, max);
                            chip(grid, font, &label, Act::Learn(id_node), rank > 0);
                        }
                    });
                    for at in [10, 20, 30] {
                        let reached = points >= at;
                        col.spawn(hud::text_bundle(
                            font,
                            format!(
                                "{at}  {}{}",
                                campaign::milestone_name(hero.class, tree, at),
                                survivors_core::talents::description(hero.class, tree, at)
                                    .map(|effect| format!("\n{effect}"))
                                    .unwrap_or_default()
                            ),
                            12.0,
                            if reached { GOLD } else { DIM },
                        ));
                    }
                });
            }
        });
    panel.spawn(hud::text_bundle(
        font,
        "Click spends a point. Right-click takes one back. Reset clears this spec.",
        12.0,
        DIM,
    ));
}

fn milestone_label(class: usize, tree: usize, node: usize, rank: u8, max: u8) -> String {
    for at in [10, 20, 30] {
        if campaign::milestone_node(at) == Some(node) {
            return format!("{} {rank}/{max}", campaign::milestone_name(class, tree, at));
        }
    }
    format!("Rank {rank}/{max}")
}

fn rewards_page(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let earned = rewards::points_of(account.earned_ids());
    let have = rewards::earned_count(account.earned_ids());
    panel.spawn(hud::text_bundle(
        font,
        format!(
            "{} to spend  ·  {earned} earned  ·  {have} of {}",
            account.unspent(),
            rewards::catalog_len()
        ),
        13.0,
        Color::WHITE,
    ));
    if let Some(next) = account.next_locked() {
        panel.spawn(hud::text_bundle(font, next, 12.0, GOLD));
    }
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (i, name) in ["Perks", "Companions", "Tabards"].iter().enumerate() {
                chip(
                    row,
                    font,
                    name,
                    Act::RewardPage(i as u8),
                    armory.reward_page == i as u8,
                );
            }
        });
    match armory.reward_page {
        1 => companion_page(panel, font, armory, account),
        2 => tabard_page(panel, font, account),
        _ => perk_page(panel, font, account),
    }
}

fn perk_page(panel: &mut impl ChipParent, font: &Handle<Font>, account: &Account) {
    for (code, perk) in rewards::PERKS.into_iter().enumerate() {
        let rank = account.perks.rank(perk);
        let label = match rewards::perk_cost(perk, rank) {
            Some(cost) => format!("{} {rank} · {cost}", rewards::perk_name(perk)),
            None => format!("{} {rank} max", rewards::perk_name(perk)),
        };
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                chip(row, font, &label, Act::Reward(code as u8), false);
                row.spawn(hud::text_bundle(
                    font,
                    rewards::perk_effect(perk),
                    12.0,
                    DIM,
                ));
            });
    }
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            chip(
                row,
                font,
                &format!("Reset perks ({})", account.resets_left()),
                Act::Reward(12),
                false,
            );
        });
}

fn companion_page(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    panel.spawn(hud::text_bundle(
        font,
        "Companions fetch coins and experience, then rest. They are not a magnet.",
        12.0,
        DIM,
    ));
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
        });
    let first = super::companions::LOOKS
        .get(usize::from(account.pets.look[0]))
        .map(|(name, _)| *name)
        .unwrap_or("Snowshoe Rabbit");
    let second_look = super::companions::LOOKS
        .get(usize::from(account.pets.look[1]))
        .map(|(name, _)| *name)
        .unwrap_or("Snowshoe Rabbit");
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            chip(
                row,
                font,
                &format!("First companion: {first}"),
                Act::PetSlot(0),
                armory.pet_slot == 0,
            );
            chip(
                row,
                font,
                &format!("Second companion: {second_look}"),
                Act::PetSlot(1),
                armory.pet_slot == 1,
            );
        });
    panel.spawn(hud::text_bundle(
        font,
        "Pick a companion, then a look. Snowshoe Rabbit is the default.",
        12.0,
        DIM,
    ));
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            let worn = account.pets.look[usize::from(armory.pet_slot)];
            for (index, (name, _)) in super::companions::LOOKS.iter().enumerate() {
                chip(
                    row,
                    font,
                    name,
                    Act::WearLook(index as u8),
                    worn == index as u8,
                );
            }
        });
}

fn tabard_page(panel: &mut impl ChipParent, font: &Handle<Font>, account: &Account) {
    panel.spawn(hud::text_bundle(
        font,
        "Tabards are account cosmetics. No tabard grants a stat.",
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
            chip(row, font, "None", Act::Tabard(0), account.tabard == 0);
            for (index, tabard) in campaign::TABARDS.iter().enumerate() {
                let worn = account.tabard == index as u8 + 1;
                chip(row, font, tabard.name, Act::Tabard(index as u8 + 1), worn);
            }
        });
    let checked = campaign::TABARDS
        .iter()
        .map(|tabard| format!("{} ({})", tabard.name, tabard.display))
        .collect::<Vec<_>>()
        .join(", ");
    panel.spawn(hud::text_bundle(
        font,
        format!("Checked against the client: {checked}."),
        12.0,
        DIM,
    ));
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

fn codex(panel: &mut impl ChipParent, font: &Handle<Font>, account: &Account) {
    panel.spawn(hud::text_bundle(
        font,
        "Legendary amulets. Each class has one for every spell. Click an icon to read it.",
        13.0,
        GOLD,
    ));
    for (class_i, class) in CLASSES.iter().enumerate() {
        let keys = class_keys(class);
        let seen = account
            .classes
            .get(class_i)
            .map(|p| p.seen.as_slice())
            .unwrap_or(&[]);
        let found = keys
            .iter()
            .filter(|key| {
                seen.iter()
                    .any(|name| name == &campaign::legendary_amulet(key))
            })
            .count();
        panel.spawn(hud::text_bundle(
            font,
            format!("{}  {found}/{}", class.name, keys.len()),
            13.0,
            Color::WHITE,
        ));
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(3.0),
                row_gap: Val::Px(3.0),
                ..default()
            })
            .with_children(|row| {
                for (slot, key) in keys.iter().enumerate() {
                    let seen_one = seen
                        .iter()
                        .any(|name| name == &campaign::legendary_amulet(key));
                    let dim = if seen_one { 1.0 } else { 0.35 };
                    row.spawn((
                        Button,
                        Act::Codex(codex_legend(class_i, slot)),
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(if seen_one { GOLD } else { DIM }),
                        ImageNode {
                            color: Color::srgba(1.0, 1.0, 1.0, dim),
                            ..default()
                        },
                        hud::SpellIcon(data::ability(key).spell),
                        hud::IconDim(dim),
                    ));
                }
            });
    }
    panel.spawn(hud::text_bundle(
        font,
        "Rings, trinkets, and amulets are offered to every class and last one run.",
        13.0,
        GOLD,
    ));
    jewel_row(panel, font, "Rings", &campaign::RING_NAMES, 2);
    jewel_row(panel, font, "Trinkets", &campaign::TRINKET_NAMES, 3);
    jewel_row(panel, font, "Amulets", &campaign::AMULET_NAMES, 4);
}

fn jewel_row(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    title: &str,
    names: &[&str],
    kind: u8,
) {
    panel.spawn(hud::text_bundle(
        font,
        format!("{title}  {}/{}", names.len(), names.len()),
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
            for (index, name) in names.iter().enumerate() {
                chip(row, font, name, Act::Codex(codex_jewel(kind, index)), false);
            }
        });
}

fn achievements(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    let rows = account.achievement_list();
    let earned_n = rows.iter().filter(|row| row.earned).count();
    let earned_pts: u32 = rows
        .iter()
        .filter(|row| row.earned)
        .map(|row| row.points)
        .sum();
    let total_pts: u32 = rows.iter().map(|row| row.points).sum();
    let names = [
        "Summary",
        "Zones",
        "Classes",
        "Collection",
        "Feats",
        "Lifetime",
    ];
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Px(168.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|nav| {
                nav.spawn(hud::text_bundle(
                    font,
                    format!("{earned_pts} of {total_pts}"),
                    16.0,
                    GOLD,
                ));
                nav.spawn(hud::text_bundle(
                    font,
                    format!("{earned_n} of {} earned", rows.len()),
                    12.0,
                    DIM,
                ));
                for (index, name) in names.iter().enumerate() {
                    let label = if index == 0 {
                        (*name).to_string()
                    } else {
                        let (have, total, _, _) = category_counts(&rows, index as u8);
                        format!("{name}  {have}/{total}")
                    };
                    chip(
                        nav,
                        font,
                        &label,
                        Act::Achieve(index as u8),
                        armory.achieve == index as u8,
                    );
                    if index > 0 {
                        let (_, _, pts, all) = category_counts(&rows, index as u8);
                        meter(
                            nav,
                            if all == 0 {
                                0.0
                            } else {
                                pts as f32 / all as f32
                            },
                        );
                    }
                }
            });
            row.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|body| {
                if armory.achieve == 0 {
                    achievement_summary(body, font, account, &rows);
                } else if let Some(category) = category_of(armory.achieve) {
                    for entry in rows.iter().filter(|row| row.category == category) {
                        let progress = entry
                            .progress
                            .map(|(have, need)| format!("  {have}/{need}"))
                            .unwrap_or_default();
                        let (mark, color) = if entry.earned {
                            ("Earned", GOLD)
                        } else {
                            ("Open", Color::WHITE)
                        };
                        body.spawn(hud::text_bundle(
                            font,
                            format!(
                                "{mark} — {} — {} points{progress}",
                                entry.name, entry.points
                            ),
                            13.0,
                            color,
                        ));
                    }
                }
            });
        });
}

fn achievement_summary(
    body: &mut impl ChipParent,
    font: &Handle<Font>,
    account: &Account,
    rows: &[rewards::Listed],
) {
    body.spawn(hud::text_bundle(font, "Recently earned", 13.0, GOLD));
    let mut recent = 0;
    for id in account.earned_ids().iter().rev() {
        let Some(row) = rows.iter().find(|row| row.id == *id) else {
            continue;
        };
        body.spawn(hud::text_bundle(
            font,
            format!("{} — {} points", row.name, row.points),
            13.0,
            Color::WHITE,
        ));
        recent += 1;
        if recent == 4 {
            break;
        }
    }
    if recent == 0 {
        body.spawn(hud::text_bundle(font, "None yet.", 13.0, DIM));
    }
    body.spawn(hud::text_bundle(font, "Almost there", 13.0, GOLD));
    let mut close: Vec<_> = rows
        .iter()
        .filter(|row| !row.earned && row.progress.is_some())
        .collect();
    close.sort_by(|a, b| {
        let (ah, an) = a.progress.unwrap_or((0, 1));
        let (bh, bn) = b.progress.unwrap_or((0, 1));
        let af = ah as f32 / an.max(1) as f32;
        let bf = bh as f32 / bn.max(1) as f32;
        bf.partial_cmp(&af).unwrap_or(std::cmp::Ordering::Equal)
    });
    if close.is_empty() {
        body.spawn(hud::text_bundle(font, "Nothing in progress.", 13.0, DIM));
    }
    for row in close.into_iter().take(4) {
        let (have, need) = row.progress.unwrap_or((0, 1));
        body.spawn(hud::text_bundle(
            font,
            format!("{} — {} points  {have}/{need}", row.name, row.points),
            13.0,
            Color::WHITE,
        ));
    }
}

fn category_of(page: u8) -> Option<rewards::Category> {
    match page {
        1 => Some(rewards::Category::Zones),
        2 => Some(rewards::Category::Classes),
        3 => Some(rewards::Category::Collection),
        4 => Some(rewards::Category::Feats),
        5 => Some(rewards::Category::Lifetime),
        _ => None,
    }
}

fn category_counts(rows: &[rewards::Listed], page: u8) -> (usize, usize, u32, u32) {
    let Some(category) = category_of(page) else {
        return (0, 0, 0, 0);
    };
    let mine: Vec<_> = rows.iter().filter(|row| row.category == category).collect();
    let have = mine.iter().filter(|row| row.earned).count();
    let pts = mine
        .iter()
        .filter(|row| row.earned)
        .map(|row| row.points)
        .sum();
    let all = mine.iter().map(|row| row.points).sum();
    (have, mine.len(), pts, all)
}

fn meter(parent: &mut impl ChipParent, fraction: f32) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(4.0),
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.16, 0.08, 0.9)),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent((fraction.clamp(0.0, 1.0) * 100.0).max(0.0)),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(GOLD),
            ));
        });
}

fn class_keys(class: &data::ClassDef) -> Vec<&'static str> {
    let mut keys = Vec::new();
    for key in std::iter::once(class.start).chain(class.pool.iter().copied()) {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

fn codex_legend(class: usize, slot: usize) -> u16 {
    (1u16 << 12) | ((class as u16) << 6) | ((slot as u16) & 0x3f)
}

fn codex_jewel(kind: u8, index: usize) -> u16 {
    (u16::from(kind) << 12) | ((index as u16) & 0x0fff)
}

fn codex_text(account: &Account, id: u16) -> String {
    let kind = id >> 12;
    let index = usize::from(id & 0x0fff);
    match kind {
        1 => {
            let class_i = index >> 6;
            let slot = index & 0x3f;
            let Some(class) = CLASSES.get(class_i) else {
                return "Unknown spell".into();
            };
            let keys = class_keys(class);
            let Some(key) = keys.get(slot).copied() else {
                return "Unknown spell".into();
            };
            let def = data::ability(key);
            let amulet = campaign::legendary_amulet(key);
            let seen = account
                .classes
                .get(class_i)
                .is_some_and(|progress| progress.seen.iter().any(|name| name == &amulet));
            let mark = if seen { "seen" } else { "not seen yet" };
            format!(
                "{amulet} — {} — {} — {} damage, {}s cooldown — {mark}",
                spell_title(key),
                kind_name(def.kind),
                def.damage,
                def.cooldown
            )
        }
        2 => jewel_text(
            "Ring",
            campaign::RING_NAMES.get(index).copied(),
            "Two rings can be worn this run.",
        ),
        3 => jewel_text(
            "Trinket",
            campaign::TRINKET_NAMES.get(index).copied(),
            "Two trinkets can be worn this run.",
        ),
        4 => jewel_text(
            "Amulet",
            campaign::AMULET_NAMES.get(index).copied(),
            "One amulet can be worn this run. It is not written into the collection.",
        ),
        _ => "Unknown entry".into(),
    }
}

fn jewel_text(kind: &str, name: Option<&str>, rule: &str) -> String {
    format!("{kind} — {} — {rule}", name.unwrap_or("Unknown"))
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
            chip(row, font, "Restart local build", Act::Update, false);
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
    n = fold(n, u64::from(account.pets.look[0]));
    n = fold(n, u64::from(account.pets.look[1]));
    n = fold(n, u64::from(account.tabard));
    for hero in &account.characters {
        for spec in &hero.dbc {
            for &(tree, id, rank) in spec {
                n = fold(n, u64::from(tree));
                n = fold(n, u64::from(id));
                n = fold(n, u64::from(rank));
            }
        }
    }
    n = fold(n, u64::from(armory.achieve));
    n = fold(n, u64::from(armory.reward_page));
    n = fold(n, u64::from(armory.pet_slot));
    n = fold(n, armory.piece as u64);
    n = fold(n, armory.picked.unwrap_or(99) as u64);
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
        n = fold(n, u64::from(hero.spec));
        n = fold(n, hash_str(&hero.first));
        n = fold(n, hash_str(&hero.last));
        for spec in hero.specs {
            for rank in spec {
                n = fold(n, u64::from(rank));
            }
        }
    }
    for class in &account.classes {
        let (worn, looks) = class.equipment.encode();
        n = fold(n, hash_str(&worn));
        n = fold(n, hash_str(&looks));
        for piece in class.pieces {
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
    fn dbc_refunds_require_earlier_rows_and_the_full_prerequisite_rank() {
        let node = |id, row| benilla_formats::Talent {
            id,
            tab: 1,
            row,
            col: 0,
            ranks: [1, 2, 3, 4, 5],
            prereq_talent: 0,
            prereq_rank: 0,
            required_spell: 0,
            exceptional: false,
        };
        let mut list = vec![node(1, 0), node(2, 0), node(3, 1), node(4, 2)];
        assert!(super::talent_ranks_ok(&list, &[(1, 5), (3, 5), (4, 1)]));
        assert!(!super::talent_ranks_ok(&list, &[(1, 4), (3, 5), (4, 1)]));
        assert!(super::talent_ranks_ok(&list, &[(1, 5), (3, 4)]));
        list[2].prereq_talent = 1;
        list[2].prereq_rank = 4; // DBC rank 4 means all five ranks are required.
        assert!(super::talent_ranks_ok(&list, &[(1, 5), (2, 5), (3, 1)]));
        assert!(!super::talent_ranks_ok(&list, &[(1, 4), (2, 5), (3, 1)]));
    }

    #[test]
    fn locked_selection_survives_opening_an_armory_tab() {
        let account = account::blank();
        let mut selection = Selection::default();
        selection.map = data::MAPS
            .iter()
            .position(|m| m.name == "Elwynn Forest")
            .unwrap();
        let mut armory = super::Armory::default();
        armory.picked = Some(2);
        for page in [
            super::Page::Door,
            super::Page::Heroes,
            super::Page::Collection,
            super::Page::DropTables,
            super::Page::Codex,
            super::Page::Talents,
            super::Page::Achievements,
            super::Page::Rewards,
            super::Page::Notes,
        ] {
            armory.page = page;
            assert!(enter_refused(&account, &selection, armory.entered_zone()).is_some());
        }
    }

    #[test]
    fn a_locked_zone_does_not_start() {
        let account = account::blank();
        let mut selection = Selection::default();
        let elwynn = data::MAPS
            .iter()
            .position(|m| m.name == "Elwynn Forest")
            .unwrap();
        selection.map = elwynn;
        assert!(enter_refused(&account, &selection, None).is_none());
        let duskwood = data::MAPS
            .iter()
            .position(|m| m.name == "Duskwood")
            .unwrap();
        selection.map = duskwood;
        assert!(enter_refused(&account, &selection, None).is_some());
        let warsong = super::super::campaign::ZONES
            .iter()
            .position(|z| z.name == "Warsong Gulch")
            .unwrap();
        selection.map = elwynn;
        let reason = enter_refused(&account, &selection, Some(warsong));
        assert_eq!(
            reason.as_deref(),
            Some("Warsong Gulch has no map and no boss yet")
        );
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
            let mut label = b.spawn(hud::text_bundle(font, label, 13.0, ink));
            if on {
                label.remove::<TextShadow>();
            }
        });
}
