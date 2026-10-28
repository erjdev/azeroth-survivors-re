//! The Armory: the front door before a run, and the draft pages a tab opens inside that frame.
//! Appearance stays on the glue race and class screen, reached from the hero column.

mod achievement_page;
pub(super) use achievement_page::remember_scroll as remember_achievement_scroll;
mod character;
mod cloak;
pub(super) use character::subject as character_subject;
mod codex_page;
pub(super) use codex_page::refresh_details as codex_details;
mod collection;
mod controls;
mod scrollbar;
mod shortcuts;
mod weapon_alternatives;
pub(super) use controls::style as style_controls;
pub(super) use scrollbar::{input as scrollbar_input, update as update_scrollbars};
mod door;
mod drop_page;
#[cfg(any(test, not(target_arch = "wasm32")))]
pub(super) mod preview;
mod reward_draft;
mod reward_page;
pub(super) use reward_page::remember_scroll as remember_reward_scroll;
mod name_edit;
mod name_input;
mod typography;
pub(super) use typography::{apply as style_headings, DisplayHeading};
mod talent_background;
pub(super) use talent_background::fill as fill_talent_background;
mod talent_heading;
pub(super) use talent_heading::fill as fill_talent_headings;
mod talent_page;
pub(super) use talent_page::fill_art as fill_talent_art;
pub(super) use talent_page::keep_inspection_visible as keep_talent_inspection_visible;
pub(super) use talent_page::{
    remember_scroll as remember_talent_scroll, ScrollMemory as TalentScrollMemory,
};

pub(super) use collection::{
    remember_scroll as remember_collection_scroll, scroll as scroll_collection,
};

use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::system::SystemParam;
pub(super) mod controller;

use bevy::prelude::*;

use super::account::{Account, PurchaseError};
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
    controller: Res<'w, controller::ControllerFocus>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
}

/// Creature looks and the talent catalog, one system parameter so the menu stays within Bevy's limit.
#[derive(SystemParam)]
pub(super) struct MenuBooks<'w> {
    globals: Option<Res<'w, super::localization::Strings>>,
    strings: Option<Res<'w, crate::glue_strings::GlueStrings>>,
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
    RunOptions,
}

impl Page {
    #[cfg(any(test, not(target_arch = "wasm32")))]
    fn preview(name: &str) -> Option<Self> {
        Some(match name {
            "starter" => Self::Door,
            "run-options" => Self::RunOptions,
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
        #[cfg(not(target_arch = "wasm32"))]
        if crate::run_mode::dev_affordances() {
            if let Ok(name) = std::env::var("WOW_SURVIVORS_UI_PAGE") {
                if let Some(page) = Self::preview(&name) {
                    return page;
                }
                warn!("Unknown WOW_SURVIVORS_UI_PAGE {name:?}; opening starter page");
            }
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
            "run-options",
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
    supplemental_piece: Option<String>,
    collection_scroll: Vec2,
    drop_zone: usize,
    drop_scroll: Vec2,
    character_scroll: Vec2,
    starter_scroll: Vec2,
    starter_panel_scroll: [Vec2; 3],
    collection_columns: [Vec2; 3],
    drop_columns: [Vec2; 4],
    character_columns: [Vec2; 3],
    codex_columns: [Vec2; 2],
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
            supplemental_piece: None,
            collection_scroll: Vec2::ZERO,
            drop_zone: 0,
            drop_scroll: Vec2::ZERO,
            character_scroll: Vec2::ZERO,
            starter_scroll: Vec2::ZERO,
            starter_panel_scroll: [Vec2::ZERO; 3],
            collection_columns: [Vec2::ZERO; 3],
            drop_columns: [Vec2::ZERO; 4],
            character_columns: [Vec2::ZERO; 3],
            codex_columns: [Vec2::ZERO; 2],
            codex_entry: None,
        }
    }
}

impl Armory {
    pub(super) fn is_run_options(&self) -> bool {
        self.page == Page::RunOptions
    }

    fn open_drop_source(&mut self, zone: usize) {
        if zone < campaign::ZONES.len() {
            self.drop_zone = zone;
            self.drop_scroll = Vec2::ZERO;
            self.drop_columns = [Vec2::ZERO; 4];
            self.open_page(Page::DropTables);
        }
    }
    pub(super) fn open_page(&mut self, page: Page) {
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

#[derive(Component, Clone, PartialEq, Eq)]
pub(super) enum Act {
    Page(Page),
    ManageHeroes(bool),
    Pick(u32),
    Body(u8, u8),
    Feature(usize, i8),
    RandomFeatures,
    Focus(u8),
    DraftClass(usize),
    Create,
    Zone(usize),
    BrowseDrops(usize),
    OpenDropSource(usize),
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
    InspectSupplemental(String),
    WearSupplemental(String),
    ShowSupplemental(String),
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
        mut settings,
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
    if settings.open {
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
    if pointer.mouse.just_released(MouseButton::Right) || pointer.controller.refund.is_some() {
        for (entity, act, interaction) in &acts {
            let pointer_refund = pointer.mouse.just_released(MouseButton::Right)
                && *interaction != Interaction::None;
            let focused_refund =
                pointer.controller.refund == Some(entity) && controller::secondary_action(act);
            if !pointer_refund && !focused_refund {
                continue;
            }
            match act {
                Act::InspectPiece(index) => {
                    let class = picked_class(&account, &selection);
                    let slot = index % campaign::GRID_SLOTS;
                    let band = (index / campaign::GRID_SLOTS) as u8;
                    if pointer_refund && cosmetic_click {
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
                Act::InspectSupplemental(key) => {
                    let class = picked_class(&account, &selection);
                    if let Some(piece) =
                        survivors_core::supplemental_gear::by_key(key).filter(|piece| {
                            piece.class == class && account.supplemental.owns(class, key)
                        })
                    {
                        if pointer_refund && cosmetic_click {
                            account.select_supplemental(class, key, true);
                        } else if account.classes[class].equipment.supplemental_worn()[piece.slot]
                            .as_deref()
                            == Some(key)
                        {
                            account.wear_piece(class, piece.slot, None);
                        } else {
                            account.select_supplemental(class, key, false);
                        }
                        account.save();
                    }
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
        if !pointer.clicks.hit(entity) && pointer.controller.activated != Some(entity) {
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
            Act::InspectPiece(index) => {
                armory.piece = index.min(campaign::PIECES - 1);
                armory.supplemental_piece = None;
            }
            Act::InspectSupplemental(key) => armory.supplemental_piece = Some(key),
            Act::WearSupplemental(key) | Act::ShowSupplemental(key) => {
                let class = picked_class(&account, &selection);
                if account.select_supplemental(class, &key, matches!(act, Act::ShowSupplemental(_)))
                {
                    account.save();
                }
            }
            Act::OpenEquipment(slot) => {
                let class = picked_class(&account, &selection);
                let slot = slot.min(campaign::GRID_SLOTS - 1);
                armory.supplemental_piece =
                    account.classes[class].equipment.supplemental_worn()[slot].clone();
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
                    if let Some(options) = super::appearance::options(
                        &selection,
                        &account,
                        creatures,
                        &bodies.0,
                        books.strings.as_deref(),
                    ) {
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
            Act::DraftClass(class) => {
                if class < CLASSES.len() {
                    armory.draft_class = class;
                }
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
            Act::OpenDropSource(index) => armory.open_drop_source(index),
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
                    Err(reason) => purchase_error(reason, books.globals.as_deref()),
                };
            }
            Act::RemoveReward(code) => {
                armory.note = match armory.reward_draft.remove(&account, code) {
                    Ok(()) => "Pending upgrade removed. Confirm to apply remaining changes.".into(),
                    Err(reason) => purchase_error(reason, books.globals.as_deref()),
                };
            }
            Act::ConfirmRewards => {
                armory.note = match armory.reward_draft.confirm(&mut account) {
                    Ok(()) => "Rewards confirmed.".into(),
                    Err(reason) => purchase_error(reason, books.globals.as_deref()),
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
            Act::RunOptions => armory.open_page(Page::RunOptions),
            Act::Options => settings.open = true,
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

fn purchase_error(error: PurchaseError, strings: Option<&super::localization::Strings>) -> String {
    let fallback = super::localization::Strings::default();
    let strings = strings.unwrap_or(&fallback);
    match error {
        PurchaseError::AlreadyLearned => strings
            .text("ALREADY_LEARNED", "Already learned")
            .to_owned(),
        PurchaseError::Message(message) => message.to_owned(),
    }
}

fn buy_reward(account: &mut Account, code: u8) -> Result<(), PurchaseError> {
    if (8..=10).contains(&code) {
        return account.buy_instinct(code - 7);
    }
    match code {
        0 => account.buy_perk(Perk::SecondThoughts),
        1 => account.buy_perk(Perk::Skip),
        2 => account.buy_perk(Perk::Veterans),
        3 => account.buy_perk(Perk::Banish),
        4 => account.buy_perk(Perk::OpeningHand),
        5 => account.buy_perk(Perk::KeenEye),
        6 => account.buy_leash(),
        7 => account.buy_paws(),
        11 => account.buy_second(),
        12 => account.reset_perks(),
        _ => Err("Unknown reward"),
    }
    .map_err(PurchaseError::Message)
}

fn create_hero(armory: &mut Armory, account: &mut Account, selection: &mut Selection) {
    match account.create(&armory.draft_first, &armory.draft_last, armory.draft_class) {
        Some(id) => {
            pick_hero(selection, account, id);
            account.save();
            armory.note = format!(
                "Created {} {}",
                armory.draft_first.trim(),
                armory.draft_last.trim()
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

/// Shared menu entry gate: account recovery, explicit hero identity and campaign access.
/// An available map returns nothing only after the selected hero has been validated.
pub(super) fn enter_refused(
    account: &Account,
    selection: &Selection,
    picked: Option<usize>,
) -> Option<String> {
    if let Some(notice) = account.recovery_notice() {
        return Some(notice.into());
    }
    if let Some(id) = selection.character {
        let Some(hero) = account.character(id) else {
            return Some("This hero is no longer available. Choose a hero before entering.".into());
        };
        if selection.class != Some(hero.class) {
            return Some("Choose this hero again to load their class before entering.".into());
        }
    }

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
        run.finish_start(account.perks, account.milestones_picked(class, hero));
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
    globals: Option<Res<'w, super::localization::Strings>>,
    strings: Option<Res<'w, crate::glue_strings::GlueStrings>>,
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
    mut feature_dots: Query<(&character::FeatureDot, &mut BackgroundColor)>,
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
            super::appearance::options(
                &selection,
                &account,
                creatures,
                &bodies.0,
                assets.strings.as_deref(),
            )
        });
    // Features change the compositor and these counters, not the layout. Keep the
    // tree (and its focus, scrolling and preview) alive while the player edits.
    if let Some(features) = &features {
        for (dot, mut color) in &mut feature_dots {
            let next = character::feature_dot_color(features.values[dot.0] == dot.1);
            if color.0 != next {
                color.0 = next;
            }
        }
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
            .zip(&features.labels)
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
                        Page::Collection
                            | Page::DropTables
                            | Page::Heroes
                            | Page::Door
                            | Page::Codex
                    ),
            ),
        ),
    );
    if *drawn == Some(sig)
        && !assets
            .globals
            .as_ref()
            .is_some_and(|strings| strings.is_changed())
    {
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
    let fallback = super::localization::Strings::default();
    let strings = assets.globals.as_deref().unwrap_or(&fallback);
    let note = account.recovery_notice().unwrap_or(armory.note.as_str());
    commands.entity(root).with_children(|panel| {
        door::build(
            strings,
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
    typography::section(panel, font, "Create a hero", 20.0, GOLD);
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(6.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            ..default()
        })
        .with_children(|row| {
            for (slot, title, value) in [
                (1, "First name", &armory.draft_first),
                (2, "Last name", &armory.draft_last),
            ] {
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_basis: Val::Percent(45.0),
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    ..default()
                })
                .with_children(|field| {
                    field.spawn(hud::text_bundle(font, title, 13.0, GOLD));
                    chip(
                        field,
                        font,
                        &field_label(value, armory.focus == slot),
                        Act::Focus(slot),
                        armory.focus == slot,
                    );
                });
            }
        });
    panel.spawn(hud::text_bundle(font, "Class", 13.0, GOLD));
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(4.0),
            margin: UiRect::bottom(Val::Px(6.0)),
            ..default()
        })
        .with_children(|row| {
            for (index, class) in CLASSES.iter().enumerate() {
                chip(
                    row,
                    font,
                    class.name,
                    Act::DraftClass(index),
                    armory.draft_class == index,
                );
            }
        });
    chip(panel, font, "Create hero", Act::Create, false);
    panel.spawn(hud::text_bundle(
        font,
        "Choose race and appearance after creating your hero.",
        12.0,
        Color::WHITE,
    ));
    typography::section(panel, font, "Account upgrades", 20.0, GOLD);
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
    strings: &crate::survivors::localization::Strings,
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
        Page::Talents => talents(strings, panel, font, account, selection, catalog),
        Page::Rewards => rewards_page(panel, font, armory, account),
        Page::Collection => {}
        Page::Codex => {}
        Page::Achievements => achievements(panel, font, armory, account),
        Page::Notes => notes_page(panel, font),
        Page::DropTables => {}
        Page::Door | Page::Look | Page::RunOptions => {}
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

#[derive(Component, PartialEq)]
pub(super) struct TalentDetailContent(String, Option<Vec<std::ops::Range<usize>>>);

type TalentDetailQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static TalentDetail,
        &'static mut Text,
        &'static mut TextColor,
        Option<&'static TalentDetailContent>,
        &'static TextFont,
    ),
>;

/// Inspect focused talents before purchase, or the hovered talent in pointer mode.
pub(super) fn talent_details(
    mut commands: Commands,
    nodes: Query<(Entity, &Interaction, &TalentInfo)>,
    focus: Res<controller::ControllerFocus>,
    #[cfg(any(test, not(target_arch = "wasm32")))] preview: Option<Res<preview::Preview>>,
    mut detail: TalentDetailQuery,
    spells: Option<Res<crate::ui_action::Spells>>,
) {
    let selected = focus.focused().and_then(|entity| nodes.get(entity).ok());
    #[cfg(any(test, not(target_arch = "wasm32")))]
    let selected = preview
        .as_ref()
        .and_then(|preview| preview.inspected())
        .and_then(|(class, id)| {
            nodes.iter().find(|(_, _, info)| {
                info.class == class && info.id == id && info.milestone.is_none()
            })
        })
        .or(selected);
    let Some((_, _, node)) = selected.or_else(|| {
        nodes
            .iter()
            .find(|(_, state, _)| **state != Interaction::None)
    }) else {
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
                .map(|def| {
                    def.inline_description(node.rank).unwrap_or_else(|| {
                        format!("{}\n{}", def.description, def.rank_preview(node.rank))
                    })
                })
                .unwrap_or_else(|| talent_page::provisional_description(node.rank, node.max).0),
            definition.is_some(),
        )
    };
    let highlight = if node.milestone.is_some() {
        talent_page::milestone_value_ranges(&effect)
    } else {
        match survivors_core::talents::talent_definition(node.class, node.id) {
            Some(def) => def
                .inline_description_with_values(node.rank)
                .map(|(_, range)| range),
            None => Some(talent_page::provisional_description(node.rank, node.max).1),
        }
    };
    for (entity, part, mut text, mut color, previous, font) in &mut detail {
        let (value, range) = match part {
            TalentDetail::Title => (&title, None),
            TalentDetail::Effect => (&effect, highlight.clone()),
        };
        let content = TalentDetailContent(value.clone(), range.clone());
        if previous == Some(&content) {
            continue;
        }
        commands.entity(entity).despawn_children().insert(content);
        color.0 = Color::WHITE;
        if let Some(ranges) = range {
            **text = value[..ranges[0].start].to_owned();
            commands.entity(entity).with_children(|parent| {
                for (index, range) in ranges.iter().enumerate() {
                    parent.spawn((
                        TextSpan::new(&value[range.clone()]),
                        TextColor(hud::card_growth::GAIN),
                        font.clone(),
                    ));
                    let end = ranges.get(index + 1).map_or(value.len(), |next| next.start);
                    parent.spawn((
                        TextSpan::new(&value[range.end..end]),
                        TextColor(Color::WHITE),
                        font.clone(),
                    ));
                }
            });
        } else {
            **text = value.clone();
            if matches!(part, TalentDetail::Effect) && supported && node.milestone.is_none() {
                color.0 = Color::srgb(0.45, 1.0, 0.45);
            }
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
    if u32::from(current) >= talent.max_rank() {
        return format!("{name} is already at maximum rank {}", talent.max_rank());
    }
    let character = account.character(hero).expect("selected hero exists");
    // Match learn_talent: legacy skeleton ranks return to the pool on the first DBC spend.
    let spent = character.dbc[usize::from(character.spec).min(2)]
        .iter()
        .map(|entry| u32::from(entry.2))
        .sum();
    if campaign::unspent_talents(character.talent_xp, spent) == 0 {
        return "No talent points to spend. Earn talent experience in runs or refund a point."
            .into();
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
    if !prereq_met {
        return requirement(talent);
    }
    let earlier: u32 = list
        .iter()
        .filter(|entry| entry.row < talent.row)
        .map(|entry| u32::from(character.dbc_rank(entry.id)))
        .sum();
    let required = talent.row.saturating_mul(5);
    if earlier < required {
        return format!(
            "{name} requires {required} points in earlier rows of this tree; {earlier} spent."
        );
    }
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
        return "This saved tree is invalid. Refund conflicting ranks or reset this spec.".into();
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
    strings: &crate::survivors::localization::Strings,
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
                            let label = milestone_label(strings, hero.class, tree, node, rank, max);
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

fn milestone_label(
    strings: &crate::survivors::localization::Strings,
    class: usize,
    tree: usize,
    node: usize,
    rank: u8,
    max: u8,
) -> String {
    for at in [10, 20, 30] {
        if campaign::milestone_node(at) == Some(node) {
            return format!("{} {rank}/{max}", campaign::milestone_name(class, tree, at));
        }
    }
    strings.by_key(
        "TOOLTIP_TALENT_RANK",
        "Rank %s/%s",
        &[
            rank.to_string().as_str().into(),
            max.to_string().as_str().into(),
        ],
    )
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
            let amulet = campaign::legendary_amulet(key);
            let seen = account
                .classes
                .get(class_i)
                .is_some_and(|progress| progress.seen.iter().any(|name| name == &amulet));
            let mark = if seen {
                "Previously offered"
            } else {
                "Not offered yet"
            };
            let title = spell_title(key);
            let training = survivors_core::talents::definitions(class_i)
                .iter()
                .find(|node| node.stat == survivors_core::talents::TalentStat::AbilityUnlock(key))
                .map_or_else(String::new, |node| {
                    format!("Train {} in Talents first. ", node.name)
                });
            format!(
                "Amulet of {title}\n{mark}\n\nUsing this amulet\n{training}Acquire {title} during the run. The amulet does not grant the ability.\n\nSpell at rank 1 · before equipment and talents\n{}",
                hud::base_ability_body(data::ability(key))
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
        Page::RunOptions => 11,
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
    n = fold(
        n,
        hash_str(armory.supplemental_piece.as_deref().unwrap_or("")),
    );
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
    for key in account.supplemental.keys() {
        n = fold(n, hash_str(key));
    }
    for class in &account.classes {
        for key in class
            .equipment
            .supplemental_worn()
            .iter()
            .chain(class.equipment.supplemental_looks())
        {
            n = fold(n, hash_str(key.as_deref().unwrap_or("")));
        }
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
mod tests;
