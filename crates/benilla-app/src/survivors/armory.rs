//! The Armory: the front door before a run, and the draft pages a tab opens inside that frame.
//! Appearance stays on the glue race and class screen, reached from the hero column.

mod achievement_page;
pub(super) use achievement_page::remember_scroll as remember_achievement_scroll;
mod character;
pub(super) use character::subject as character_subject;
mod codex_page;
pub(super) use codex_page::refresh_details as codex_details;
mod collection;
mod controls;
mod shortcuts;
pub(super) use controls::style as style_controls;
mod door;
mod drop_page;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
pub(super) mod preview;
mod reward_draft;
mod reward_page;
pub(super) use reward_page::remember_scroll as remember_reward_scroll;
mod name_edit;
mod name_input;
mod typography;
pub(super) use typography::apply as style_headings;
mod talent_heading;
pub(super) use talent_heading::fill as fill_talent_headings;
mod talent_page;
pub(super) use talent_page::fill_art as fill_talent_art;
pub(super) use talent_page::{
    remember_scroll as remember_talent_scroll, ScrollMemory as TalentScrollMemory,
};

pub(super) use collection::{
    remember_scroll as remember_collection_scroll, scroll as scroll_collection,
};

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
    bodies: Option<Res<'w, crate::entities::CharCreate>>,
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

impl Page {
    #[cfg(any(test, all(feature = "dev", not(target_arch = "wasm32"))))]
    fn preview(name: &str) -> Option<Self> {
        Some(match name {
            "starter" => Self::Door,
            "heroes" => Self::Heroes,
            "talents" => Self::Talents,
            "rewards" => Self::Rewards,
            "collection" => Self::Collection,
            "codex" => Self::Codex,
            "achievements" => Self::Achievements,
            "notes" => Self::Notes,
            "drops" => Self::DropTables,
            "look" => Self::Look,
            _ => return None,
        })
    }

    fn initial() -> Self {
        #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
        if let Ok(name) = std::env::var("WOW_SURVIVORS_UI_PAGE") {
            if let Some(page) = Self::preview(&name) {
                return page;
            }
            warn!("Unknown WOW_SURVIVORS_UI_PAGE {name:?}; opening starter page");
        }
        Self::Door
    }
}

#[cfg(test)]
mod preview_tests {
    use super::Page;
    #[test]
    fn direct_ui_pages_are_explicit_and_unknown_names_do_not_start_gameplay() {
        for name in [
            "starter",
            "heroes",
            "talents",
            "rewards",
            "collection",
            "codex",
            "achievements",
            "notes",
            "drops",
            "look",
        ] {
            assert!(Page::preview(name).is_some());
        }
        assert!(matches!(
            Page::preview("collection"),
            Some(Page::Collection)
        ));
        assert!(Page::preview("playing").is_none());
        assert!(Page::preview("").is_none());
    }
}

#[derive(Resource)]
pub(super) struct Armory {
    page: Page,
    manage_heroes: bool,
    /// 0 none; 1/2 creation names; 3/4 selected-hero rename.
    focus: u8,
    rename: Option<name_edit::Draft>,
    draft_first: String,
    draft_last: String,
    draft_class: usize,
    /// The campaign row the door is showing. None follows the selected battleground.
    pub(super) picked: Option<usize>,
    pub note: String,
    /// 0 summary, then Zones, Classes, Collection, Feats, Lifetime.
    achieve: u8,
    achievement_scroll: [Vec2; 6],
    /// 0 perks, 1 companions, 2 tabards.
    reward_page: u8,
    /// Session-only positions, separate for Perks, Companions and Tabards.
    reward_scroll: [Vec2; 3],
    reward_draft: reward_draft::Draft,
    /// Which companion the Looks row dresses.
    pet_slot: u8,
    piece: usize,
    collection_scroll: Vec2,
    drop_zone: usize,
    drop_scroll: Vec2,
    character_scroll: Vec2,
    starter_scroll: Vec2,
    codex_entry: Option<u16>,
}

impl Default for Armory {
    fn default() -> Self {
        Self {
            page: Page::initial(),
            manage_heroes: false,
            focus: 0,
            rename: None,
            draft_first: "Adventurer".into(),
            draft_last: "Stormwind".into(),
            draft_class: 0,
            picked: None,
            note: String::new(),
            achieve: 0,
            achievement_scroll: [Vec2::ZERO; 6],
            reward_page: 0,
            reward_scroll: [Vec2::ZERO; 3],
            reward_draft: reward_draft::Draft::default(),
            pet_slot: 0,
            piece: 0,
            collection_scroll: Vec2::ZERO,
            drop_zone: 0,
            drop_scroll: Vec2::ZERO,
            character_scroll: Vec2::ZERO,
            starter_scroll: Vec2::ZERO,
            codex_entry: None,
        }
    }
}

impl Armory {
    fn open_page(&mut self, page: Page) {
        self.rename = None;
        self.page = page;
        self.focus = 0;
        self.manage_heroes = false;
        self.note.clear();
    }

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
    ManageHeroes(bool),
    Pick(u32),
    Body(u8, u8),
    Feature(usize, i8),
    RandomFeatures,
    Focus(u8),
    Class,
    Create,
    Zone(usize),
    BrowseDrops(usize),
    Learn(usize),
    Spec(u8),
    Respec,
    Reward(u8),
    RemoveReward(u8),
    ConfirmRewards,
    UndoRewards,
    Restart,
    Update,
    PickClass(usize),
    RandomName,
    EditName,
    SaveName,
    CancelName,
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
    OpenEquipment(usize),
    WearPiece(usize, Option<u8>),
    ShowPiece(usize, super::equipment::Look),
}

#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    pointer: Pointer,
    acts: Query<(Entity, &Act, &Interaction)>,
    keyboard: shortcuts::Keyboard,
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
    let shortcuts::Keyboard {
        keys,
        mut keyboard,
        mut shortcut,
        mut raw,
        mut native,
    } = keyboard;
    let native_cosmetic_click = native.read(raw.read());
    if *phase.get() != Phase::Menu {
        typing.0 = false;
        keyboard.clear();
        *shortcut = default();
        return;
    }
    let cosmetic_click = shortcut.update(&pointer.mouse, &keys) || native_cosmetic_click;
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
    if armory
        .rename
        .as_ref()
        .is_some_and(|draft| armory.page != Page::Door || !draft.matches(&account, &selection))
    {
        armory.rename = None;
        armory.focus = 0;
    }
    typing.0 = armory.focus != 0 && (armory.page == Page::Heroes || armory.rename.is_some());
    for event in keyboard.read() {
        if typing.0 {
            name_input::feed(event, &mut armory, &mut account, &selection);
        }
    }
    // Keep Typing set for the whole frame, including the Enter that finishes an edit.
    if !typing.0 && keys.just_pressed(KeyCode::Escape) && armory.page != Page::Door {
        if armory.page == Page::Heroes && armory.manage_heroes {
            armory.manage_heroes = false;
        } else {
            armory.open_page(Page::Door);
        }
        armory.focus = 0;
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
                    if cosmetic_click {
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
                armory.open_page(page);
            }
            Act::ManageHeroes(open) => {
                armory.manage_heroes = open;
                armory.focus = 0;
            }
            Act::InspectPiece(index) => armory.piece = index.min(campaign::PIECES - 1),
            Act::OpenEquipment(slot) => {
                let class = picked_class(&account, &selection);
                let slot = slot.min(campaign::GRID_SLOTS - 1);
                let band = account.classes[class].equipment.worn[slot].unwrap_or(0);
                armory.piece = usize::from(band) * campaign::GRID_SLOTS + slot;
                armory.open_page(Page::Collection);
            }
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
            Act::Feature(_, _) | Act::RandomFeatures => {
                if let (Some(creatures), Some(bodies)) =
                    (books.creatures.as_deref(), books.bodies.as_deref())
                {
                    if let Some(options) =
                        super::appearance::options(&selection, &account, creatures, &bodies.0)
                    {
                        let values = match act {
                            Act::Feature(dial, direction) => {
                                super::appearance::step(&options, *dial, *direction)
                            }
                            _ => Some(std::array::from_fn(|i| {
                                rng.index(usize::from(options.counts[i]).max(1)) as u8
                            })),
                        };
                        if let (Some(id), Some(values)) = (selection.character, values) {
                            if account.set_features(id, values) {
                                account.save();
                            }
                        }
                    }
                }
            }
            Act::Body(race, sex) => {
                if character::choose_body(&mut selection, &mut account, race, sex) {
                    account.save();
                    super::menu::save_pick(&selection);
                    armory.note.clear();
                }
            }
            Act::Focus(slot) => armory.focus = slot,
            Act::Class => {
                armory.draft_class = (armory.draft_class + 1) % CLASSES.len();
            }
            Act::Create => create_hero(&mut armory, &mut account, &mut selection),
            Act::Pick(id) => {
                let needs_look = pick_hero(&mut selection, &account, id);
                armory.manage_heroes = false;
                armory.focus = 0;
                armory.note = if needs_look
                    && account
                        .character(id)
                        .is_some_and(|hero| hero.body.is_none())
                {
                    "Choose this hero's appearance below.".into()
                } else {
                    String::new()
                };
            }
            Act::BrowseDrops(index) => armory.drop_zone = index.min(ZONES.len() - 1),
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
            Act::Reward(code) => {
                armory.note = match armory.reward_draft.stage(&account, code) {
                    Ok(()) => "Pending — Confirm to apply, or Undo to discard.".into(),
                    Err(reason) => reason.into(),
                };
            }
            Act::RemoveReward(code) => {
                armory.note = match armory.reward_draft.remove(&account, code) {
                    Ok(()) => "Pending upgrade removed. Confirm to apply remaining changes.".into(),
                    Err(reason) => reason.into(),
                };
            }
            Act::ConfirmRewards => {
                armory.note = match armory.reward_draft.confirm(&mut account) {
                    Ok(()) => "Rewards confirmed.".into(),
                    Err(reason) => reason.into(),
                };
            }
            Act::UndoRewards => {
                armory.reward_draft.clear();
                armory.note = "Pending rewards discarded.".into();
            }
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
            Act::EditName => {
                armory.rename = name_edit::Draft::begin(&account, &selection);
                armory.focus = if armory.rename.is_some() { 3 } else { 0 };
                armory.note.clear();
            }
            Act::SaveName => save_name(&mut armory, &mut account, &selection),
            Act::CancelName => {
                armory.rename = None;
                armory.focus = 0;
                armory.note.clear();
            }
            Act::RandomName => random_name(&mut armory, &mut account, &selection, &mut rng),
            Act::Look => {
                armory.open_page(Page::Look);
            }
            Act::Enter if armory.rename.is_some() => {
                armory.note = "Save or Cancel the name edit before entering a run.".into();
            }
            Act::Enter if !armory.reward_draft.is_empty() => {
                armory.open_page(Page::Rewards);
                armory.note = "Confirm or Undo pending rewards before entering a run.".into();
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
            Act::Options => armory.open_page(Page::Notes),
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
            Act::Codex(id) => armory.codex_entry = Some(id),
            Act::Achieve(page) => {
                armory.achieve = page.min(5);
                armory.open_page(Page::Achievements);
            }
            Act::RewardPage(page) => {
                armory.reward_page = page.min(2);
                armory.open_page(Page::Rewards);
            }
            Act::PetSlot(slot) => armory.pet_slot = slot.min(1),
            Act::WearLook(look) => {
                if armory.pet_slot >= account.pets.owned.min(2) {
                    armory.note = "Confirm the companion purchase before choosing its look.".into();
                } else {
                    account.wear_look(armory.pet_slot, look);
                }
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
    // Persist edits even when the OS closes the window directly from an Armory page.
    // Clean accounts perform no disk I/O; failed writes remain dirty for retry.
    account.save();
}

fn buy_reward(account: &mut Account, code: u8) -> Result<(), &'static str> {
    match code {
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
    }
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
            armory.manage_heroes = false;
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

fn save_name(armory: &mut Armory, account: &mut Account, selection: &Selection) {
    let Some(draft) = armory.rename.as_ref() else {
        return;
    };
    match draft.confirm(account, selection) {
        Ok(()) => {
            armory.rename = None;
            armory.focus = 0;
            armory.note.clear();
        }
        Err(reason) => armory.note = reason.into(),
    }
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
        run.milestones = account.milestones_picked(class, hero);
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
    layout: Res<'w, super::UiLayout>,
    talent_scroll: Res<'w, TalentScrollMemory>,
    creatures: Option<Res<'w, crate::entities::Creatures>>,
    bodies: Option<Res<'w, crate::entities::CharCreate>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn refresh(
    mut commands: Commands,
    menus: Query<Entity, (With<MenuRoot>, With<super::menu::MenuBuilt>)>,
    mut roots: Query<(&mut Node, &ComputedNode), With<ArmoryRoot>>,
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
    mut feature_values: Query<(&character::FeatureValue, &mut Text)>,
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
    let mut compact_collection = false;
    if let Ok((mut node, computed)) = roots.single_mut() {
        let width = computed.size.x * computed.inverse_scale_factor;
        compact_collection = width > 0.0 && width < 1100.0;
        let display = if show_door {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let features = assets
        .creatures
        .as_deref()
        .zip(assets.bodies.as_deref())
        .and_then(|(creatures, bodies)| {
            super::appearance::options(&selection, &account, creatures, &bodies.0)
        });
    // Features change the compositor and these counters, not the layout. Keep the
    // tree (and its focus, scrolling and preview) alive while the player edits.
    if let Some(features) = &features {
        for (dial, mut text) in &mut feature_values {
            let value = format!(
                "{} / {}",
                features.values[dial.0] + 1,
                features.counts[dial.0]
            );
            if text.0 != value {
                text.0 = value;
            }
        }
    }
    // Body reconciliation can finish after creation has already drawn the page.
    // Rebuild when controls become available or their shape changes, not on dial edits.
    let feature_layout = features.as_ref().map_or(0, |features| {
        features
            .counts
            .iter()
            .zip(features.labels)
            .fold(1, |n, (count, label)| {
                fold(fold(n, u64::from(*count)), hash_str(label))
            })
    });
    let sig = (
        fold(
            fold(
                signature(&armory, &account, &selection, &renown, &stash),
                feature_layout,
            ),
            u64::from(assets.creatures.is_some() && assets.bodies.is_some()),
        ),
        fold(
            assets.layout.version,
            u64::from(
                compact_collection
                    && matches!(
                        armory.page,
                        Page::Collection | Page::DropTables | Page::Heroes | Page::Door
                    ),
            ),
        ),
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
    let note = armory.note.as_str();
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
            features.as_ref(),
            assets.talent_scroll.offset(&account, &selection),
            compact_collection,
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
            for hero in &account.characters {
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
            flex_wrap: FlexWrap::Wrap,
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
        Page::Codex => {}
        Page::Achievements => achievements(panel, font, armory, account),
        Page::Notes => notes_page(panel, font),
        Page::DropTables => {}
        Page::Door | Page::Look => {}
    }
}

pub(super) fn class_trees(
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
pub(super) enum TalentDetail {
    Title,
    Effect,
}

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
    mut detail: Query<(&TalentDetail, &mut Text, &mut TextColor)>,
    spells: Option<Res<crate::ui_action::Spells>>,
) {
    let Some((_, node)) = nodes.iter().find(|(state, _)| **state != Interaction::None) else {
        return;
    };
    let (title, effect, supported) = if let Some((tree, at)) = node.milestone {
        let effect = survivors_core::talents::description(node.class, tree, at);
        (
            format!(
                "{} · {at} points",
                campaign::milestone_name(node.class, tree, at)
            ),
            effect
                .unwrap_or(
                    "Individual milestone effect pending; provisional tree stat bonuses apply.",
                )
                .to_owned(),
            effect.is_some(),
        )
    } else {
        let name = spells
            .as_deref()
            .and_then(|spells| spells.catalog.get(node.spell))
            .map(|spell| spell.name.as_str())
            .unwrap_or("Talent");
        let definition = survivors_core::talents::talent_definition(node.class, node.id);
        (
            format!("{name}  {}/{}", node.rank, node.max),
            definition
                .map(|def| format!("{}\n{}", def.description, def.rank_preview(node.rank)))
                .unwrap_or_else(|| {
                    "Individual effect pending; currently +1% damage per rank.".to_owned()
                }),
            definition.is_some(),
        )
    };
    for (part, mut text, mut color) in &mut detail {
        let (value, tint) = match part {
            TalentDetail::Title => (&title, Color::WHITE),
            TalentDetail::Effect => (
                &effect,
                if supported {
                    Color::srgb(0.45, 1.0, 0.45)
                } else {
                    Color::WHITE
                },
            ),
        };
        if **text != *value {
            **text = value.clone();
        }
        color.0 = tint;
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
        TalentDetail::Title,
        hud::text_bundle(
            font,
            "Hover a talent to inspect its rank and current Survivors effect.",
            13.0,
            Color::WHITE,
        ),
    ));
    panel.spawn((
        TalentDetail::Effect,
        hud::text_bundle(font, "", 13.0, Color::WHITE),
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

pub(super) fn talent_ranks_ok(list: &[benilla_formats::Talent], ranks: &[(u32, u8)]) -> bool {
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
    let talent_name = |entry: &benilla_formats::Talent| {
        spells
            .and_then(|spells| spells.get(entry.ranks[0]))
            .map(|spell| spell.name.clone())
            .unwrap_or_else(|| format!("Talent {}", entry.id))
    };
    let requirement = |entry: &benilla_formats::Talent| {
        let prerequisite = list
            .iter()
            .find(|other| other.id == entry.prereq_talent)
            .map(&talent_name)
            .unwrap_or_else(|| format!("Talent {}", entry.prereq_talent));
        format!(
            "{} requires {prerequisite} rank {}",
            talent_name(entry),
            entry.prereq_rank + 1
        )
    };
    let name = talent_name(talent);
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
            if let Some(dependent) = list.iter().find(|entry| {
                entry.prereq_talent == talent_id
                    && u32::from(current - 1) <= entry.prereq_rank
                    && ranks.iter().any(|&(id, rank)| id == entry.id && rank > 0)
            }) {
                return requirement(dependent);
            }
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
            requirement(talent)
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
        requirement(talent)
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
    reward_page::build(panel, font, armory, account);
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
    reward_page::companions(panel, font, armory, account);
    let first = super::companions::LOOKS
        .get(usize::from(account.pets.look[0]))
        .map(|look| look.name)
        .unwrap_or("Snowshoe Rabbit");
    let second_look = super::companions::LOOKS
        .get(usize::from(account.pets.look[1]))
        .map(|look| look.name)
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
                &format!(
                    "First companion: {first} · {}",
                    instinct_name(account.pets.instinct[0])
                ),
                Act::PetSlot(0),
                armory.pet_slot == 0,
            );
            chip(
                row,
                font,
                &if account.pets.owned >= 2 {
                    format!(
                        "Second companion: {second_look} · {}",
                        instinct_name(account.pets.instinct[1])
                    )
                } else {
                    "Second companion · Locked".into()
                },
                Act::PetSlot(1),
                armory.pet_slot == 1,
            );
        });
    if armory.pet_slot >= account.pets.owned.min(2) {
        panel.spawn(hud::text_bundle(
            font,
            "Unlock the second companion above to choose its appearance.",
            12.0,
            DIM,
        ));
        return;
    }
    panel.spawn(hud::text_bundle(
        font,
        "Pick a companion, then a look. Snowshoe Rabbit is the default.",
        12.0,
        DIM,
    ));
    reward_page::looks(panel, font, armory, account);
}

fn instinct_name(kind: u8) -> &'static str {
    match kind {
        1 => "Gold-sniffer",
        2 => "Wisp-herder",
        3 => "Rune-hound",
        _ => "No instinct",
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

fn achievements(
    panel: &mut impl ChipParent,
    font: &Handle<Font>,
    armory: &Armory,
    account: &Account,
) {
    achievement_page::build(panel, font, armory, account);
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
            let keys = class.ability_keys();
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
                "Amulet of {}\n{} · {} damage, {}s cooldown\n{mark}",
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

pub(super) fn spell_title(key: &str) -> String {
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
        n = fold(n, u64::from(account.talents_valid(hero)));
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
    for code in armory.reward_draft.codes() {
        n = fold(n, u64::from(*code) + 1);
    }
    n = fold(n, u64::from(armory.pet_slot));
    n = fold(n, armory.piece as u64);
    n = fold(n, armory.drop_zone as u64);
    // Codex selection updates its detail text in place, preserving catalog scrolling.
    n = fold(n, armory.picked.unwrap_or(99) as u64);
    n = fold(n, u64::from(armory.focus));
    n = fold(n, u64::from(armory.manage_heroes));
    n = fold(n, armory.draft_class as u64);
    if let Some(draft) = &armory.rename {
        n = fold(n, u64::from(draft.hero));
        n = fold(n, hash_str(&draft.first));
        n = fold(n, hash_str(&draft.last));
    }
    n = fold(n, u64::from(armory.rename.is_some()));
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

fn chip(parent: &mut impl ChipParent, font: &Handle<Font>, label: &str, act: Act, on: bool) {
    controls::button(
        parent,
        font,
        label,
        act,
        on,
        Node {
            min_height: Val::Px(22.0),
            justify_content: JustifyContent::Center,
            margin: UiRect::new(Val::Px(0.0), Val::Px(4.0), Val::Px(2.0), Val::Px(0.0)),
            padding: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        },
    );
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
        let selection = Selection {
            map: data::MAPS
                .iter()
                .position(|m| m.name == "Elwynn Forest")
                .unwrap(),
            ..Default::default()
        };
        let mut armory = super::Armory {
            picked: Some(2),
            ..Default::default()
        };
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
    fn front_door_snapshots_the_selected_heroes_active_talents() {
        let mut account = account::empty();
        let first = account.ensure(0);
        let id = account.create("Selected", "Warrior", 0).unwrap();
        let hero = account
            .characters
            .iter_mut()
            .find(|hero| hero.id == id)
            .unwrap();
        hero.talent_xp = 2000;
        hero.spec = 1;
        hero.dbc[1] = vec![(1, 158, 5), (1, 157, 5), (1, 161, 5), (1, 155, 5)];
        let mut selection = Selection {
            class: Some(0),
            character: Some(id),
            look: Some(0),
            looks: vec![(20318, 1, 0)],
            map: data::MAPS
                .iter()
                .position(|m| m.name == "Elwynn Forest")
                .unwrap(),
            ..Default::default()
        };
        let mut run = super::Run::default();
        let mut armory = super::Armory::default();
        let mut next = bevy::prelude::NextState::Unchanged;
        super::try_enter(
            &mut armory,
            &account,
            &mut selection,
            &mut run,
            &super::Renown::default(),
            &super::Stash::default(),
            &mut next,
        );
        assert!(
            matches!(
                next,
                bevy::prelude::NextState::Pending(super::Phase::Playing)
            ),
            "{}",
            armory.note
        );
        assert_eq!(run.milestones, account.milestones_picked(0, Some(id)));
        assert_eq!(run.milestones.cast("demoralizing_shout").duration, 1.5);
        assert!((run.milestones.enrage_damage(true) - 1.2).abs() < 0.00001);
        selection.character = Some(first);
        super::try_enter(
            &mut armory,
            &account,
            &mut selection,
            &mut run,
            &super::Renown::default(),
            &super::Stash::default(),
            &mut next,
        );
        assert_eq!(run.milestones.points, [0; 3]);
        assert_eq!(run.milestones.enrage_damage(true), 1.);
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
            Some("Warsong Gulch world content is not ready")
        );
    }
}
