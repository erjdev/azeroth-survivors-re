//! Opt-in live controls for the production menu, independent of gameplay and saves.
#[cfg(not(target_arch = "wasm32"))]
use super::Phase;
use super::{Armory, Page, Typing};
use crate::survivors::{
    campaign::{self, JewelKind},
    cards::PreviewHand,
    data,
    hud::{Choice, Offer},
    Owned, Run,
};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PrimaryWindow;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn plugin(app: &mut App) {
    // The engine owns the build seam. A player build never reads the preview path
    // or installs the polling system, even if these environment variables are set.
    if !crate::run_mode::dev_affordances() {
        return;
    }
    let Some(path) = std::env::var_os("WOW_SURVIVORS_UI_PREVIEW") else {
        return;
    };
    app.insert_resource(Preview {
        path: path.into(),
        seen: String::new(),
        poll: Timer::from_seconds(0.25, TimerMode::Repeating),
        saved: None,
        inspected: None,
    })
    .add_systems(
        Update,
        reload.before(super::input).run_if(in_state(Phase::Menu)),
    );
}

#[derive(Resource)]
pub(in crate::survivors) struct Preview {
    path: std::path::PathBuf,
    seen: String,
    poll: Timer,
    saved: Option<(Run, Offer)>,
    inspected: Option<(usize, u32)>,
}

impl Preview {
    pub(super) fn inspected(&self) -> Option<(usize, u32)> {
        self.inspected
    }
}

#[derive(Clone, Copy)]
enum Card {
    Ability(&'static data::AbilityDef, u8),
    Jewel(JewelKind),
    Passive(&'static data::PassiveDef, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Piece {
    Legacy(usize),
    Poor(usize),
    RaidEpic(usize),
}

struct Request {
    page: Page,
    width: u32,
    height: u32,
    selected: bool,
    category: Option<u8>,
    talent: Option<u32>,
    entry: Option<u16>,
    hero: Option<u32>,
    piece: Option<Piece>,
    zone: Option<usize>,
    cards: Option<Vec<Card>>,
}

impl Request {
    fn parse(text: &str) -> Option<Self> {
        let (mut page, mut width, mut height) = (None, None, None);
        let mut cards_page = false;
        let mut cards = None;
        let mut profile = None;
        let mut category = None;
        let mut talent = None;
        let mut entry = None;
        let mut hero = None;
        let mut piece = None;
        let mut zone = None;
        for line in text.lines() {
            let line = line.split('#').next()?.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            match key.trim() {
                "page" if page.is_none() => {
                    cards_page = value == "cards";
                    page = Some(if cards_page {
                        Page::Door
                    } else {
                        Page::preview(value)?
                    });
                }
                "abilities" if cards.is_none() => {
                    let parsed: Option<Vec<_>> = value
                        .split(',')
                        .map(|entry| {
                            let (key, rank) = entry.trim().split_once(':')?;
                            let def = data::ABILITIES.iter().find(|def| def.key == key.trim())?;
                            let rank = rank.trim().parse::<u8>().ok()?;
                            (rank < data::MAX_ABILITY_LEVEL).then_some(Card::Ability(def, rank))
                        })
                        .collect();
                    cards = Some(parsed?);
                }
                "passives" if cards.is_none() => {
                    let parsed: Option<Vec<_>> = value
                        .split(',')
                        .map(|entry| {
                            let (key, rank) = entry.trim().split_once(':')?;
                            let def = data::PASSIVES.iter().find(|def| def.key == key.trim())?;
                            let rank = rank.trim().parse::<u8>().ok()?;
                            (rank < data::MAX_PASSIVE_LEVEL).then_some(Card::Passive(def, rank))
                        })
                        .collect();
                    cards = Some(parsed?);
                }
                "jewelry" if cards.is_none() => {
                    let parsed: Option<Vec<_>> = value
                        .split(',')
                        .map(|entry| {
                            let (kind, value) = entry.trim().split_once(':')?;
                            let value = value.trim();
                            let jewel = if kind.trim() == "legend" {
                                let def = data::ABILITIES.iter().find(|def| def.key == value)?;
                                JewelKind::Legend(def.key)
                            } else {
                                let index = value.parse::<u8>().ok()?;
                                match kind.trim() {
                                    "ring" if (index as usize) < campaign::RINGS => {
                                        JewelKind::Ring(index)
                                    }
                                    "trinket" if (index as usize) < campaign::TRINKETS => {
                                        JewelKind::Trinket(index)
                                    }
                                    "amulet" if (index as usize) < campaign::AMULETS => {
                                        JewelKind::Amulet(index)
                                    }
                                    _ => return None,
                                }
                            };
                            Some(Card::Jewel(jewel))
                        })
                        .collect();
                    cards = Some(parsed?);
                }
                "profile" if profile.is_none() => {
                    profile = Some(match value {
                        "baseline" => false,
                        "selected" => true,
                        _ => return None,
                    });
                }
                "entry" if entry.is_none() => {
                    let (kind, key) = value.split_once(':')?;
                    entry =
                        Some(match kind.trim() {
                            "legend" => data::CLASSES.iter().enumerate().find_map(
                                |(class, definition)| {
                                    definition
                                        .ability_keys()
                                        .iter()
                                        .position(|ability| *ability == key.trim())
                                        .map(|slot| super::codex_legend(class, slot))
                                },
                            )?,
                            kind => {
                                let (kind, count) = match kind {
                                    "ring" => (2, campaign::RINGS),
                                    "trinket" => (3, campaign::TRINKETS),
                                    "amulet" => (4, campaign::AMULETS),
                                    _ => return None,
                                };
                                let index = key.trim().parse::<usize>().ok()?;
                                if index >= count {
                                    return None;
                                }
                                super::codex_jewel(kind, index)
                            }
                        });
                }
                "category" if category.is_none() => category = Some(value),
                "hero" if hero.is_none() => {
                    hero = Some(value.parse::<u32>().ok().filter(|id| *id > 0)?)
                }
                "talent" if talent.is_none() => talent = Some(value.parse::<u32>().ok()?),
                "zone" if zone.is_none() => {
                    zone = Some(
                        campaign::ZONES
                            .iter()
                            .position(|zone| zone.name.eq_ignore_ascii_case(value))?,
                    );
                }
                "piece" if piece.is_none() => {
                    let (band, slot) = value.split_once(':')?;
                    let slot = campaign::SLOTS[..campaign::GRID_SLOTS]
                        .iter()
                        .position(|name| name.eq_ignore_ascii_case(slot.trim()))?;
                    piece = Some(if band.trim().eq_ignore_ascii_case("poor") {
                        Piece::Poor(slot)
                    } else if band.trim().eq_ignore_ascii_case("raid epic") {
                        if slot == campaign::GRID_SLOTS - 1 {
                            return None;
                        }
                        Piece::RaidEpic(slot)
                    } else {
                        let band = campaign::BANDS_NAMED
                            .iter()
                            .position(|name| name.eq_ignore_ascii_case(band.trim()))?;
                        Piece::Legacy(band * campaign::GRID_SLOTS + slot)
                    });
                }
                "width" if width.is_none() => width = Some(value.parse::<u32>().ok()?),
                "height" if height.is_none() => height = Some(value.parse::<u32>().ok()?),
                _ => return None,
            }
        }
        if matches!(piece, Some(Piece::RaidEpic(_)))
            && !matches!(page, Some(Page::Collection | Page::DropTables))
        {
            return None;
        }
        if (!cards_page && profile.is_some())
            || cards_page != cards.is_some()
            || cards
                .as_ref()
                .is_some_and(|cards| cards.is_empty() || cards.len() > 4)
        {
            return None;
        }
        let page = page?;
        if (entry.is_some() && page != Page::Codex)
            || talent.is_some_and(|id| id == 0 || page != Page::Talents)
            || (piece.is_some() && !matches!(page, Page::Collection | Page::DropTables))
            || (zone.is_some() && page != Page::DropTables)
        {
            return None;
        }
        let category = match category {
            None => None,
            Some(name) => {
                let names: &[&str] = match page {
                    Page::Heroes => &["appearance", "manage"],
                    Page::Rewards => &["perks", "companions", "tabards"],
                    Page::Achievements => &[
                        "summary",
                        "zones",
                        "classes",
                        "collection",
                        "feats",
                        "lifetime",
                    ],
                    _ => return None,
                };
                Some(names.iter().position(|candidate| *candidate == name)? as u8)
            }
        };
        let request = Self {
            page,
            category,
            talent,
            entry,
            hero,
            piece,
            zone,
            width: width?,
            height: height?,
            cards,
            selected: profile.unwrap_or(false),
        };
        ((640..=7680).contains(&request.width) && (480..=4320).contains(&request.height))
            .then_some(request)
    }

    /// Stage selection changes until the complete request has passed validation.
    /// Unlike a normal menu pick, a preview does not persist the selection.
    fn selection(
        &self,
        current: &crate::survivors::Selection,
        account: &crate::survivors::account::Account,
    ) -> Option<crate::survivors::Selection> {
        let mut candidate = current.clone();
        if let Some(id) = self.hero {
            let hero = account.character(id)?;
            if candidate.class != Some(hero.class) {
                candidate.class = Some(hero.class);
                candidate.looks.clear();
                candidate.look = None;
            }
            if candidate.character != Some(id) || candidate.class != current.class {
                candidate.character = Some(id);
                candidate.version = candidate.version.wrapping_add(1);
            }
        }
        Some(candidate)
    }

    fn apply(&self, armory: &mut Armory, typing: &mut Typing, window: &mut Window, class: usize) {
        let supplemental = match self.piece {
            Some(Piece::Poor(slot) | Piece::RaidEpic(slot)) => {
                let tier = if matches!(self.piece, Some(Piece::RaidEpic(_))) {
                    survivors_core::gear_tier::GearTier::RaidEpic
                } else {
                    survivors_core::gear_tier::GearTier::Poor
                };
                let Some(piece) = survivors_core::supplemental_gear::piece(class, slot, tier)
                else {
                    return;
                };
                Some(piece.key)
            }
            _ => None,
        };
        armory.page = self.page;
        if let Some(entry) = self.entry {
            armory.codex_entry = Some(entry);
        }
        if let Some(zone) = self.zone {
            armory.drop_zone = zone;
        }
        if let Some(piece) = self.piece {
            armory.piece = match piece {
                Piece::Legacy(index) => index,
                Piece::Poor(slot) | Piece::RaidEpic(slot) => slot,
            };
            armory.supplemental_piece = supplemental;
        }
        armory.manage_heroes = false;
        if let Some(category) = self.category {
            match self.page {
                Page::Heroes => armory.manage_heroes = category == 1,
                Page::Rewards => armory.reward_page = category,
                Page::Achievements => armory.achieve = category,
                _ => unreachable!("category was validated against the page"),
            }
        }
        armory.focus = 0;
        typing.0 = false;
        window.resolution.set(self.width as f32, self.height as f32);
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(not(target_arch = "wasm32"))]
fn reload(
    time: Res<Time<Real>>,
    mut preview: ResMut<Preview>,
    mut armory: ResMut<Armory>,
    mut typing: ResMut<Typing>,
    mut run: ResMut<Run>,
    mut offer: ResMut<Offer>,
    mut hand: ResMut<PreviewHand>,
    account: Res<crate::survivors::account::Account>,
    mut selection: ResMut<crate::survivors::Selection>,
    renown: Res<crate::survivors::renown::Renown>,
    stash: Res<crate::survivors::stash::Stash>,
    talents: Option<Res<crate::ui_talent::Talents>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    rows: Query<Entity, With<crate::survivors::hud::OfferRow>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !preview.poll.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let Ok(text) = std::fs::read_to_string(&preview.path) else {
        return;
    };
    if text == preview.seen {
        return;
    }
    let request = Request::parse(&text);
    let Some(request) = request else {
        preview.seen = text;
        warn!("Invalid Survivors UI preview; keeping current page and size");
        return;
    };
    if request.cards.is_some() && rows.is_empty() {
        return;
    }
    let Some(mut candidate) = request.selection(&selection, &account) else {
        preview.seen = text;
        warn!("Unknown Survivors preview hero; keeping current view");
        return;
    };
    let inspected = if let Some(id) = request.talent {
        let Some(talents) = talents.as_deref() else {
            return;
        };
        let class = candidate
            .character
            .and_then(|id| account.character(id))
            .map(|hero| hero.class);
        let Some(class) = class.filter(|&class| {
            super::CLASSES.get(class).is_some_and(|definition| {
                super::class_trees(&talents.catalog, definition.id)
                    .iter()
                    .any(|(_, nodes)| nodes.iter().any(|node| node.id == id))
            })
        }) else {
            preview.seen = text;
            warn!("Invalid Survivors talent preview for the selected hero; keeping current view");
            return;
        };
        Some((class, id))
    } else {
        None
    };
    let snapshot = if request.selected {
        if talents.is_none() {
            return;
        }
        // Resolve the staged hero before building its snapshot. Waiting for the live
        // menu to fill these looks deadlocks when the request changes class.
        if candidate.looks.is_empty() {
            let (Some(class), Some(creatures)) = (candidate.class_def(), creatures.as_deref())
            else {
                return;
            };
            candidate.looks = crate::survivors::hud::looks_for(class, creatures);
        }
        if !select_preview_body(&mut candidate, &account) {
            return;
        }
        let Some(snapshot) = selected_run(&account, &candidate, &renown, &stash) else {
            return;
        };
        snapshot
    } else {
        Run::default()
    };
    if selection.character != candidate.character || selection.class != candidate.class {
        armory.note.clear();
    }
    *selection = candidate;
    preview.seen = text;
    preview.inspected = inspected;
    request.apply(
        &mut armory,
        &mut typing,
        &mut window,
        super::picked_class(&account, &selection),
    );
    preview.hand(&request, &mut run, &mut offer, &mut hand, snapshot);
    info!("Survivors UI preview applied: {}", preview.seen.trim());
}

/// Resolve race-dependent card stats without persisting or exposing a partial request.
fn select_preview_body(
    selection: &mut crate::survivors::Selection,
    account: &crate::survivors::account::Account,
) -> bool {
    let saved_body = selection
        .character
        .and_then(|id| account.character(id))
        .and_then(|hero| hero.body);
    selection.look = if let Some(body) = saved_body {
        selection
            .looks
            .iter()
            .position(|&(_, race, sex)| (race, sex) == body)
    } else {
        selection
            .look
            .filter(|&index| index < selection.looks.len())
            .or_else(|| (!selection.looks.is_empty()).then_some(0))
    };
    selection.look.is_some()
}

/// Use the production start snapshot on a private selection; never move the menu's world.
fn selected_run(
    account: &crate::survivors::account::Account,
    selection: &crate::survivors::Selection,
    renown: &crate::survivors::renown::Renown,
    stash: &crate::survivors::stash::Stash,
) -> Option<Run> {
    let class = selection.class?;
    let mut local = crate::survivors::Selection {
        class: selection.class,
        character: selection.character,
        looks: selection.looks.clone(),
        look: selection.look,
        map: selection.map,
        ..default()
    };
    let mut run = Run::default();
    if !crate::survivors::hud::start_run(
        &mut run,
        &mut local,
        renown.ranks(class),
        stash.gear(),
        account.bonus_picked(class, selection.character),
    ) {
        return None;
    }
    run.finish_start(
        account.perks,
        account.milestones_picked(class, selection.character),
    );
    Some(run)
}

impl Preview {
    fn hand(
        &mut self,
        request: &Request,
        run: &mut Run,
        offer: &mut Offer,
        hand: &mut PreviewHand,
        snapshot: Run,
    ) {
        if let Some(cards) = &request.cards {
            if self.saved.is_none() {
                self.saved = Some((std::mem::take(run), std::mem::take(offer)));
            }
            *run = snapshot;
            run.abilities.clear();
            run.passives.clear();
            offer.choices.clear();
            for &card in cards {
                match card {
                    Card::Jewel(kind) => offer.choices.push(Choice::Jewel(kind)),
                    Card::Passive(def, 0) => offer.choices.push(Choice::NewPassive(def)),
                    Card::Passive(def, rank) => {
                        offer.choices.push(Choice::UpPassive(run.passives.len()));
                        run.passives.push((def, rank));
                    }
                    Card::Ability(def, 0) => offer.choices.push(Choice::NewAbility(def)),
                    Card::Ability(def, rank) => {
                        offer.choices.push(Choice::Upgrade(run.abilities.len()));
                        run.abilities.push(Owned {
                            def,
                            level: rank,
                            cd: 0.0,
                        });
                    }
                }
            }
            hand.0 = true;
        } else if let Some((saved_run, saved_offer)) = self.saved.take() {
            *run = saved_run;
            *offer = saved_offer;
            hand.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn preview_watcher_respects_the_engine_development_boundary() {
        use crate::local_state::test_env::{EnvGuard, ENV_LOCK};
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _path = EnvGuard::set("WOW_SURVIVORS_UI_PREVIEW", "unused-preview-path");
        let mut app = App::new();
        plugin(&mut app);
        assert_eq!(
            app.world().contains_resource::<Preview>(),
            crate::run_mode::dev_affordances()
        );
        if !crate::run_mode::dev_affordances() {
            // No polling system was registered: this world deliberately has none
            // of the time, window, account or menu resources it would require.
            app.update();
        }
    }

    #[test]
    fn hero_preview_validates_before_changing_selection_and_preserves_world() {
        use crate::survivors::{account::Account, Selection};
        let mut account = Account::default();
        let priest = account.ensure(4);
        let shaman = account.ensure(5);
        let current = Selection {
            class: Some(4),
            character: Some(priest),
            looks: vec![(1, 1, 0)],
            look: Some(0),
            map: 3,
            stage: 2,
            version: 8,
        };
        let text = "page=talents\nwidth=1280\nheight=720";
        for hero in [None, Some(priest)] {
            let request = Request::parse(&format!(
                "{text}{}",
                hero.map_or(String::new(), |id| format!("\nhero={id}"))
            ))
            .unwrap();
            let selected = request.selection(&current, &account).unwrap();
            assert_eq!(
                (selected.class, selected.character, selected.version),
                (Some(4), Some(priest), 8)
            );
            assert_eq!(selected.looks, current.looks);
            assert_eq!(selected.look, current.look);
        }
        let request = Request::parse(&format!("{text}\nhero={shaman}")).unwrap();
        let selected = request.selection(&current, &account).unwrap();
        assert_eq!(
            (selected.class, selected.character, selected.version),
            (Some(5), Some(shaman), 9)
        );
        assert_eq!((selected.map, selected.stage), (3, 2));
        assert!(selected.looks.is_empty());
        assert_eq!(selected.look, None);
        assert_eq!(
            current.character,
            Some(priest),
            "staging never mutates live selection"
        );
        assert!(Request::parse(&format!("{text}\nhero=9999"))
            .unwrap()
            .selection(&current, &account)
            .is_none());
        for suffix in ["hero=0", "hero=-1", "hero=Priest", "hero=1\nhero=2"] {
            assert!(Request::parse(&format!("{text}\n{suffix}")).is_none());
        }
    }

    #[test]
    fn direct_cross_class_cards_resolve_saved_body_before_snapshot_without_mutating_menu() {
        use crate::survivors::{account::Account, renown::Renown, stash::Stash, Selection};
        let mut account = Account::default();
        let warrior = account.ensure(0);
        let hunter = account.ensure(2);
        account.set_body(hunter, (4, 1));
        let current = Selection {
            class: Some(0),
            character: Some(warrior),
            looks: vec![(49, 1, 0)],
            look: Some(0),
            map: 3,
            stage: 2,
            version: 8,
        };
        let request = Request::parse(&format!("page=cards\nwidth=640\nheight=720\nhero={hunter}\nprofile=selected\nabilities=hunters_mark:1")).unwrap();
        let mut candidate = request.selection(&current, &account).unwrap();
        assert!(!select_preview_body(&mut candidate, &account));
        candidate.looks = vec![(100, 3, 0), (101, 4, 0), (102, 4, 1)];
        assert!(select_preview_body(&mut candidate, &account));
        assert_eq!(candidate.look, Some(2));
        let snapshot =
            selected_run(&account, &candidate, &Renown::default(), &Stash::default()).unwrap();
        assert_eq!(snapshot.race, 4);
        assert_eq!(snapshot.class.name, data::CLASSES[2].name);
        assert_eq!(
            (
                current.class,
                current.character,
                current.look,
                current.stage
            ),
            (Some(0), Some(warrior), Some(0), 2)
        );
        assert_eq!(candidate.stage, 2, "snapshot must not move the menu world");
        // An unavailable saved body must not silently preview another race's bonuses.
        candidate.looks.pop();
        assert!(!select_preview_body(&mut candidate, &account));
        // Legacy characters without a saved body retain a valid selection, or the first look.
        let mut legacy = current.clone();
        legacy.look = Some(99);
        assert!(select_preview_body(&mut legacy, &account));
        assert_eq!(legacy.look, Some(0));
    }

    #[test]
    fn collection_piece_previews_validate_all_cells_and_reject_invalid_requests() {
        for (band, name) in campaign::BANDS_NAMED.iter().enumerate() {
            for (slot, name_slot) in campaign::SLOTS[..campaign::GRID_SLOTS].iter().enumerate() {
                let request = Request::parse(&format!(
                    "page=collection\nwidth=1280\nheight=720\npiece={name}:{name_slot}"
                ))
                .unwrap();
                assert_eq!(
                    request.piece,
                    Some(Piece::Legacy(band * campaign::GRID_SLOTS + slot))
                );
            }
        }
        assert_eq!(
            Request::parse("page=collection\nwidth=640\nheight=720\npiece= LEGENDARY : main hand ")
                .unwrap()
                .piece,
            Some(Piece::Legacy(campaign::PIECES - 1))
        );
        for extra in [
            "piece=epic:off hand",
            "piece=raid epic:main hand",
            "piece=epic:unknown",
            "piece=3:7",
            "piece=epic:hands:extra",
            "piece=epic:hands\npiece=rare:feet",
            "piece=",
            "piece=epic",
        ] {
            assert!(
                Request::parse(&format!("page=collection\nwidth=640\nheight=720\n{extra}"))
                    .is_none(),
                "{extra}"
            );
        }
        for page in [
            "character",
            "talents",
            "rewards",
            "cards\nabilities=smite:1",
        ] {
            assert!(
                Request::parse(&format!(
                    "page={page}\nwidth=640\nheight=720\npiece=epic:hands"
                ))
                .is_none(),
                "{page}"
            );
        }
    }

    #[test]
    fn character_preview_selects_management_without_editing_heroes() {
        let mut armory = Armory::default();
        let mut typing = Typing(true);
        let mut window = Window::default();
        let selection = crate::survivors::Selection::default();
        let account = crate::survivors::account::blank();
        for (category, managing) in [("manage", true), ("appearance", false)] {
            let request = Request::parse(&format!(
                "page=heroes\ncategory={category}\nwidth=640\nheight=720"
            ))
            .unwrap();
            let candidate = request.selection(&selection, &account).unwrap();
            assert_eq!(candidate.character, selection.character);
            assert_eq!(candidate.class, selection.class);
            request.apply(&mut armory, &mut typing, &mut window, 0);
            assert_eq!(armory.manage_heroes, managing);
            assert!(!typing.0);
            assert!(account.characters.is_empty());
        }
        armory.manage_heroes = true;
        Request::parse("page=heroes\nwidth=640\nheight=720")
            .unwrap()
            .apply(&mut armory, &mut typing, &mut window, 0);
        assert!(!armory.manage_heroes, "default remains the dressed preview");
        for request in [
            "page=heroes\ncategory=unknown",
            "page=talents\ncategory=manage",
            "page=heroes\ncategory=manage\ncategory=appearance",
        ] {
            assert!(Request::parse(&format!("{request}\nwidth=640\nheight=720")).is_none());
        }
    }

    #[test]
    fn drop_previews_select_zone_and_item_without_changing_run_selection() {
        let selection = crate::survivors::Selection::default();
        let account = crate::survivors::account::blank();
        for (index, zone) in campaign::ZONES.iter().enumerate() {
            let request = Request::parse(&format!(
                "page=drops\nzone={}\npiece=poor:hands\nwidth=640\nheight=720",
                zone.name.to_uppercase()
            ))
            .unwrap();
            let candidate = request.selection(&selection, &account).unwrap();
            assert_eq!(
                (
                    candidate.map,
                    candidate.stage,
                    candidate.character,
                    candidate.class
                ),
                (
                    selection.map,
                    selection.stage,
                    selection.character,
                    selection.class
                )
            );
            let mut armory = Armory::default();
            let mut typing = Typing(false);
            let mut window = Window::default();
            request.apply(&mut armory, &mut typing, &mut window, 2);
            assert!(armory.page == Page::DropTables);
            assert_eq!(armory.drop_zone, index);
            assert_eq!(
                armory.supplemental_piece.as_deref(),
                Some("poor:hunter:hands")
            );
            Request::parse("page=drops\npiece=rare:helm\nwidth=1280\nheight=720")
                .unwrap()
                .apply(&mut armory, &mut typing, &mut window, 2);
            assert_eq!(armory.drop_zone, index);
            assert!(armory.supplemental_piece.is_none());
            assert_eq!(armory.piece, 2 * campaign::GRID_SLOTS);
        }
        for text in [
            "page=drops\nzone=Unknown",
            "page=drops\nzone=",
            "page=drops\nzone=0",
            "page=drops\nzone=Elwynn Forest\nzone=Duskwood",
            "page=collection\nzone=Elwynn Forest",
            "page=talents\nzone=Duskwood",
        ] {
            assert!(Request::parse(&format!("{text}\nwidth=640\nheight=720")).is_none());
        }
    }

    #[test]
    fn codex_entry_previews_cover_catalog_and_preserve_other_selections() {
        let mut entries = Vec::new();
        for (class, definition) in data::CLASSES.iter().enumerate() {
            for (slot, key) in definition.ability_keys().iter().enumerate() {
                entries.push((
                    format!("legend:{key}"),
                    super::super::codex_legend(class, slot),
                ));
            }
        }
        for (name, kind, count) in [
            ("ring", 2, campaign::RINGS),
            ("trinket", 3, campaign::TRINKETS),
            ("amulet", 4, campaign::AMULETS),
        ] {
            for index in 0..count {
                entries.push((
                    format!("{name}:{index}"),
                    super::super::codex_jewel(kind, index),
                ));
            }
        }
        for (entry, id) in entries {
            let request =
                Request::parse(&format!("page=codex\nwidth=640\nheight=720\nentry={entry}"))
                    .unwrap();
            let mut armory = Armory {
                piece: 7,
                drop_zone: 2,
                achieve: 4,
                ..default()
            };
            let mut typing = Typing(true);
            let mut window = Window::default();
            request.apply(&mut armory, &mut typing, &mut window, 0);
            assert_eq!(armory.codex_entry, Some(id));
            assert_eq!((armory.piece, armory.drop_zone, armory.achieve), (7, 2, 4));
            assert_eq!(window.resolution.width(), 640.);
            assert!(!typing.0);
            Request::parse("page=codex\nwidth=1280\nheight=720")
                .unwrap()
                .apply(&mut armory, &mut typing, &mut window, 0);
            assert_eq!(armory.codex_entry, Some(id));
        }
    }

    #[test]
    fn codex_entry_rejects_wrong_pages_unknown_keys_and_duplicate_requests() {
        for fields in [
            "page=collection\nentry=ring:0",
            "page=cards\nabilities=wrath:1\nentry=ring:0",
            "page=codex\nentry=legend:missing",
            "page=codex\nentry=ring:-1",
            "page=codex\nentry=ring:999999",
            "page=codex\nentry=ring:0\nentry=amulet:0",
            "page=codex\nentry=spell:wrath",
            "page=codex\nentry=legend:strength",
        ] {
            assert!(
                Request::parse(&format!("{fields}\nwidth=640\nheight=720")).is_none(),
                "{fields}"
            );
        }
    }

    #[test]
    fn poor_piece_preview_uses_each_class_catalog_and_switches_back_to_legacy() {
        for class in 0..data::CLASSES.len() {
            for (slot, name) in campaign::SLOTS[..campaign::GRID_SLOTS].iter().enumerate() {
                let request = Request::parse(&format!(
                    "page=collection\nwidth=1280\nheight=720\npiece= PoOr : {name} "
                ))
                .unwrap();
                assert_eq!(request.piece, Some(Piece::Poor(slot)));
                let mut armory = Armory::default();
                let mut typing = Typing(true);
                let mut window = Window::default();
                request.apply(&mut armory, &mut typing, &mut window, class);
                let piece = armory
                    .supplemental_piece
                    .as_deref()
                    .and_then(survivors_core::supplemental_gear::by_key)
                    .unwrap();
                assert_eq!((piece.class, piece.slot), (class, slot));
                let key = armory.supplemental_piece.clone();
                Request::parse("page=collection\nwidth=640\nheight=720")
                    .unwrap()
                    .apply(&mut armory, &mut typing, &mut window, class);
                assert_eq!(armory.supplemental_piece, key);
                Request::parse("page=collection\nwidth=640\nheight=720\npiece=epic:hands")
                    .unwrap()
                    .apply(&mut armory, &mut typing, &mut window, class);
                assert!(armory.supplemental_piece.is_none());
                assert_eq!(armory.piece, 3 * campaign::GRID_SLOTS + 7);
            }
        }
        assert!(Request::parse("page=talents\nwidth=640\nheight=720\npiece=poor:hands").is_none());
        assert!(
            Request::parse("page=collection\nwidth=640\nheight=720\npiece=poor:off hand").is_none()
        );
    }

    #[test]
    fn raid_previews_resolve_each_class_and_its_drop_source() {
        for class in 0..data::CLASSES.len() {
            for (slot, name) in campaign::SLOTS[..8].iter().enumerate() {
                let request = Request::parse(&format!(
                    "page=collection\nwidth=640\nheight=720\npiece= RaId EpIc : {name} "
                ))
                .unwrap();
                assert_eq!(request.piece, Some(Piece::RaidEpic(slot)));
                assert_eq!(
                    Request::parse(&format!(
                        "page=drops\nwidth=640\nheight=720\npiece=raid epic:{name}"
                    ))
                    .unwrap()
                    .piece,
                    request.piece
                );
                let mut armory = Armory::default();
                let mut typing = Typing(true);
                let mut window = Window::default();
                request.apply(&mut armory, &mut typing, &mut window, class);
                let key = armory.supplemental_piece.clone().unwrap();
                let piece = survivors_core::supplemental_gear::by_key(&key).unwrap();
                assert_eq!(
                    (piece.class, piece.slot, piece.tier),
                    (class, slot, survivors_core::gear_tier::GearTier::RaidEpic)
                );
                Request::parse("page=collection\nwidth=1280\nheight=720")
                    .unwrap()
                    .apply(&mut armory, &mut typing, &mut window, class);
                assert_eq!(armory.supplemental_piece.as_deref(), Some(key.as_str()));
                Request::parse("page=collection\nwidth=1280\nheight=720\npiece=epic:hands")
                    .unwrap()
                    .apply(&mut armory, &mut typing, &mut window, class);
                assert!(armory.supplemental_piece.is_none());
                assert_eq!(armory.piece, 3 * campaign::GRID_SLOTS + 7);
            }
        }
        for request in [
            "page=talents\npiece=raid epic:hands",
            "page=collection\npiece=raid epic:main hand",
            "page=collection\npiece=raid epic:off hand",
            "page=collection\npiece=raid epic:hands\npiece=poor:hands",
        ] {
            assert!(Request::parse(&format!("{request}\nwidth=640\nheight=720")).is_none());
        }
    }

    #[test]
    fn collection_piece_preview_changes_inspection_and_preserves_other_selections() {
        let mut armory = Armory {
            piece: 1,
            drop_zone: 4,
            codex_entry: Some(3),
            collection_scroll: Vec2::new(0., 80.),
            ..default()
        };
        let mut typing = Typing(true);
        let mut window = Window::default();
        Request::parse("page=collection\nwidth=640\nheight=720\npiece=epic:hands")
            .unwrap()
            .apply(&mut armory, &mut typing, &mut window, 0);
        assert_eq!(armory.piece, 3 * campaign::GRID_SLOTS + 7);
        assert!(armory.page == Page::Collection);
        assert_eq!((armory.drop_zone, armory.codex_entry), (4, Some(3)));
        assert_eq!(armory.collection_scroll, Vec2::new(0., 80.));
        assert_eq!(window.width(), 640.);
        assert!(!typing.0);
        Request::parse("page=collection\nwidth=1280\nheight=720")
            .unwrap()
            .apply(&mut armory, &mut typing, &mut window, 0);
        assert_eq!(
            armory.piece,
            3 * campaign::GRID_SLOTS + 7,
            "omission preserves manual inspection"
        );
    }

    #[test]
    fn passive_requests_validate_catalog_ranks_page_and_single_source() {
        for body in [
            "passives=",
            "passives=missing:1",
            "passives=renew:5",
            "passives=renew:-1",
            "passives=renew:256",
            "passives=renew:1,",
            "passives=renew:0,renew:1,renew:2,renew:3,renew:4",
            "passives=renew:1\nabilities=smite:1",
            "jewelry=ring:0\npassives=renew:1",
            "passives=renew:1\npassives=renew:2",
        ] {
            assert!(
                Request::parse(&format!("page=cards\nwidth=1280\nheight=720\n{body}")).is_none(),
                "{body}"
            );
        }
        assert!(Request::parse("page=talents\nwidth=1280\nheight=720\npassives=renew:1").is_none());
        let request = Request::parse(
            "page=cards\nwidth=640\nheight=720\npassives=renew:0,renew:4\nprofile=selected",
        )
        .unwrap();
        assert!(request.selected);
        assert!(matches!(
            request.cards.as_deref(),
            Some([Card::Passive(_, 0), Card::Passive(_, 4)])
        ));
    }

    #[test]
    fn passive_hand_uses_training_reloads_and_restores_original_passives() {
        let renew = data::PASSIVES.iter().find(|p| p.key == "renew").unwrap();
        let mut preview = Preview {
            path: "unused".into(),
            seen: String::new(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
            saved: None,
            inspected: None,
        };
        let mut run = Run {
            passives: vec![(renew, 3)],
            hp: 37.0,
            ..default()
        };
        let mut offer = Offer {
            choices: vec![Choice::Heal],
        };
        let mut hand = PreviewHand::default();
        let request = Request::parse(
            "page=cards\nwidth=1280\nheight=720\npassives=renew:0,renew:2\nprofile=selected",
        )
        .unwrap();
        let trained = Run {
            milestones: survivors_core::talents::Milestones {
                class: 4,
                ..default()
            }
            .with_ranks(&[(1, 406, 1)]),
            ..default()
        };
        preview.hand(&request, &mut run, &mut offer, &mut hand, trained);
        assert!(hand.0);
        assert!(matches!(
            offer.choices(),
            [Choice::NewPassive(_), Choice::UpPassive(0)]
        ));
        assert_eq!(run.passives.len(), 1);
        assert_eq!(run.passives[0].1, 2);
        assert!((run.stats().regen - 1.05).abs() < 0.00001);
        assert!((run.passive_value(renew, 1) - 0.525).abs() < 0.00001);
        preview.hand(&request, &mut run, &mut offer, &mut hand, Run::default());
        assert_eq!(run.stats().regen, 1.0);
        let ability =
            Request::parse("page=cards\nwidth=1280\nheight=720\nabilities=smite:1").unwrap();
        preview.hand(&ability, &mut run, &mut offer, &mut hand, Run::default());
        assert!(run.passives.is_empty());
        let exit = Request::parse("page=talents\nwidth=1280\nheight=720").unwrap();
        preview.hand(&exit, &mut run, &mut offer, &mut hand, Run::default());
        assert!(!hand.0);
        assert_eq!(run.passives[0].1, 3);
        assert_eq!(run.hp, 37.0);
        assert!(matches!(offer.choices(), [Choice::Heal]));
    }

    #[test]
    fn partial_unknown_duplicate_and_unsafe_sizes_are_rejected() {
        for text in [
            "page=collection\nwidth=1280",
            "page=playing\nwidth=1280\nheight=720",
            "page=collection\nwidth=1280\nheight=0",
            "page=collection\nwidth=NaN\nheight=720",
            "page=collection\nwidth=99999\nheight=720",
            "page=collection\nwidth=1280\nheight=720\nwidth=1024",
            "page=collection\nwidth=1280\nheight=720\nunlock=true",
            "page=talents\nwidth=1280\nheight=720\nprofile=selected",
            "page=cards\nwidth=1280\nheight=720\nabilities=smite:1\nprofile=unknown",
            "page=cards\nwidth=1280\nheight=720\nabilities=smite:1\nprofile=selected\nprofile=baseline",
        ] {
            assert!(Request::parse(text).is_none(), "{text}");
        }
    }

    #[test]
    fn talent_inspection_requires_one_positive_id_on_the_talent_page() {
        let valid = Request::parse("talent=964\npage=talents\nwidth=640\nheight=720").unwrap();
        assert_eq!(valid.talent, Some(964));
        for input in [
            "page=talents\ntalent=0",
            "page=talents\ntalent=-1",
            "page=talents\ntalent=abc",
            "page=talents\ntalent=964\ntalent=1021",
            "page=collection\ntalent=964",
            "page=cards\nabilities=smite:1\ntalent=964",
        ] {
            assert!(Request::parse(&format!("{input}\nwidth=640\nheight=720")).is_none());
        }
        assert_eq!(
            Request::parse("page=talents\nwidth=640\nheight=720")
                .unwrap()
                .talent,
            None
        );
    }

    #[test]
    fn inspection_pins_the_requested_detail_without_changing_ranks_or_actions() {
        use super::super::{controller::ControllerFocus, TalentDetail, TalentInfo};
        let mut world = World::new();
        world.insert_resource(ControllerFocus::default());
        world.insert_resource(Preview {
            path: "unused".into(),
            seen: String::new(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
            saved: None,
            inspected: Some((7, 964)),
        });
        let node = world
            .spawn((
                Interaction::None,
                TalentInfo {
                    class: 7,
                    id: 964,
                    spell: 0,
                    rank: 1,
                    max: 2,
                    milestone: None,
                },
            ))
            .id();
        world.spawn((
            Interaction::Hovered,
            TalentInfo {
                class: 7,
                id: 0,
                spell: 0,
                rank: 0,
                max: 5,
                milestone: None,
            },
        ));
        let effect = world
            .spawn((TalentDetail::Effect, Text::new(""), TextColor(Color::WHITE)))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(super::super::talent_details);
        schedule.run(&mut world);
        assert!(world
            .get::<Text>(effect)
            .unwrap()
            .0
            .contains("Rain of Fire"));
        let spans = world.get::<Children>(effect).unwrap();
        assert!(world
            .get::<TextSpan>(spans[0])
            .unwrap()
            .0
            .contains("10 » 20%"));
        assert_eq!(world.get::<TalentInfo>(node).unwrap().rank, 1);
        assert_eq!(world.resource::<ControllerFocus>().activated, None);
        world.resource_mut::<Preview>().inspected = None;
        schedule.run(&mut world);
        assert!(world
            .get::<Text>(effect)
            .unwrap()
            .0
            .contains("Individual effect pending"));
        assert_eq!(world.get::<TextColor>(effect).unwrap().0, Color::WHITE);
        let spans = world.get::<Children>(effect).unwrap();
        assert_eq!(world.get::<TextSpan>(spans[0]).unwrap().0, "+0 » 1% damage");
        assert_eq!(
            world.get::<TextColor>(spans[0]).unwrap().0,
            crate::survivors::hud::card_growth::GAIN
        );
        assert_eq!(world.get::<TextSpan>(spans[1]).unwrap().0, ".");
        assert_eq!(world.get::<TalentInfo>(node).unwrap().rank, 1);
    }

    #[test]
    fn categories_are_page_specific_atomic_and_optional() {
        let mut armory = Armory {
            draft_first: "Keep my draft".into(),
            ..default()
        };
        let mut typing = Typing(true);
        let mut window = Window::default();
        for (page, categories) in [
            ("rewards", &["perks", "companions", "tabards"][..]),
            (
                "achievements",
                &[
                    "summary",
                    "zones",
                    "classes",
                    "collection",
                    "feats",
                    "lifetime",
                ][..],
            ),
        ] {
            for (index, category) in categories.iter().enumerate() {
                // Key order must not matter; category can precede the page.
                let request = Request::parse(&format!(
                    "category={category}\npage={page}\nwidth=640\nheight=720"
                ))
                .unwrap();
                request.apply(&mut armory, &mut typing, &mut window, 0);
                assert_eq!(
                    if page == "rewards" {
                        armory.reward_page
                    } else {
                        armory.achieve
                    },
                    index as u8
                );
                assert_eq!(armory.draft_first, "Keep my draft");
                assert_eq!(window.resolution.width(), 640.);
            }
        }
        Request::parse("page=rewards\nwidth=1280\nheight=720")
            .unwrap()
            .apply(&mut armory, &mut typing, &mut window, 0);
        assert_eq!(
            armory.reward_page, 2,
            "omission preserves manual category selection"
        );
        assert_eq!(armory.achieve, 5, "other pages retain their category");
        for text in [
            "page=rewards\ncategory=zones",
            "page=achievements\ncategory=tabards",
            "page=talents\ncategory=perks",
            "page=cards\nabilities=smite:1\ncategory=perks",
            "page=rewards\ncategory=",
            "page=rewards\ncategory=tabards\ncategory=perks",
            "page=achievements\ncategory=unknown",
        ] {
            assert!(
                Request::parse(&format!("{text}\nwidth=1280\nheight=720")).is_none(),
                "{text}"
            );
        }
    }

    #[test]
    fn cards_require_bounded_known_spells_and_upgradeable_ranks() {
        for abilities in [
            "missing:1",
            "smite:8",
            "smite:255",
            "smite:-1",
            "smite",
            "",
            "smite:1,smite:1,smite:1,smite:1,smite:1",
        ] {
            assert!(Request::parse(&format!(
                "page=cards\nwidth=1280\nheight=720\nabilities={abilities}"
            ))
            .is_none());
        }
        assert!(Request::parse("page=cards\nwidth=1280\nheight=720").is_none());
        assert!(
            Request::parse("page=talents\nwidth=1280\nheight=720\nabilities=smite:1").is_none()
        );
        assert!(
            Request::parse("page=cards\nwidth=640\nheight=480\nabilities=smite:0,holy_fire:7")
                .is_some()
        );
    }

    #[test]
    fn jewelry_requires_known_catalog_entries_and_one_hand_source() {
        for input in [
            "jewelry=",
            "jewelry=ring:9",
            "jewelry=trinket:13",
            "jewelry=amulet:17",
            "jewelry=ring:-1",
            "jewelry=ring:256",
            "jewelry=legend:missing",
            "jewelry=unknown:0",
            "jewelry=ring",
            "jewelry=ring:0,",
            "jewelry=ring:0,ring:1,ring:2,ring:3,ring:4",
            "jewelry=ring:0\nabilities=smite:1",
            "abilities=smite:1\njewelry=ring:0",
            "jewelry=ring:0\njewelry=ring:1",
        ] {
            assert!(
                Request::parse(&format!("page=cards\nwidth=1280\nheight=720\n{input}")).is_none(),
                "{input}"
            );
        }
        assert!(Request::parse("page=codex\nwidth=1280\nheight=720\njewelry=ring:0").is_none());
        let request = Request::parse("jewelry=legend:rejuvenation,ring:8,trinket:12,amulet:16\npage=cards\nwidth=640\nheight=720").unwrap();
        assert!(matches!(
            request.cards.unwrap().as_slice(),
            [
                Card::Jewel(JewelKind::Legend("rejuvenation")),
                Card::Jewel(JewelKind::Ring(8)),
                Card::Jewel(JewelKind::Trinket(12)),
                Card::Jewel(JewelKind::Amulet(16))
            ]
        ));
    }

    #[test]
    fn preview_reloads_hand_and_restores_original_run_and_offer() {
        let mut preview = Preview {
            path: "unused".into(),
            seen: String::new(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
            saved: None,
            inspected: None,
        };
        let mut run = Run {
            hp: 37.0,
            kills: 42,
            ..default()
        };
        let mut offer = Offer {
            choices: vec![Choice::Heal],
        };
        let mut hand = PreviewHand::default();
        for size in [1280, 640] {
            let request = Request::parse(&format!(
                "page=cards\nwidth={size}\nheight=720\nabilities=smite:0,holy_fire:7"
            ))
            .unwrap();
            preview.hand(&request, &mut run, &mut offer, &mut hand, Run::default());
            assert!(hand.0);
            assert_eq!(offer.len(), 2);
            assert!(matches!(offer.choices()[0], Choice::NewAbility(def) if def.key == "smite"));
            assert!(matches!(offer.choices()[1], Choice::Upgrade(0)));
            assert_eq!(run.abilities[0].level, 7);
            assert_eq!(run.kills, 0);
        }
        let jewelry =
            Request::parse("page=cards\nwidth=640\nheight=720\njewelry=legend:rejuvenation,ring:0")
                .unwrap();
        preview.hand(&jewelry, &mut run, &mut offer, &mut hand, Run::default());
        assert!(hand.0);
        assert!(run.abilities.is_empty());
        assert!(matches!(
            offer.choices(),
            [
                Choice::Jewel(JewelKind::Legend("rejuvenation")),
                Choice::Jewel(JewelKind::Ring(0))
            ]
        ));
        let request = Request::parse("page=talents\nwidth=1280\nheight=720").unwrap();
        preview.hand(&request, &mut run, &mut offer, &mut hand, Run::default());
        assert!(!hand.0);
        assert_eq!(run.hp, 37.0);
        assert_eq!(run.kills, 42);
        assert!(matches!(offer.choices(), [Choice::Heal]));
        assert!(preview.saved.is_none());
    }

    #[test]
    fn selected_profile_preserves_effects_and_baseline_reload_clears_them() {
        let mut preview = Preview {
            path: "unused".into(),
            seen: String::new(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
            saved: None,
            inspected: None,
        };
        let mut run = Run::default();
        let mut offer = Offer::default();
        let mut hand = PreviewHand::default();
        let text = "page=cards\nwidth=640\nheight=720\nabilities=holy_fire:1";
        let request = Request::parse(&format!("{text}\nprofile=selected")).unwrap();
        assert!(request.selected);
        let snapshot = Run {
            milestones: survivors_core::talents::Milestones {
                class: 4,
                points: [0, 30, 0],
                ..default()
            },
            ..default()
        };
        preview.hand(&request, &mut run, &mut offer, &mut hand, snapshot);
        assert_eq!(run.milestones.points, [0, 30, 0]);
        assert_eq!(run.abilities.len(), 1);
        let baseline = Request::parse(text).unwrap();
        assert!(!baseline.selected);
        preview.hand(&baseline, &mut run, &mut offer, &mut hand, Run::default());
        assert_eq!(run.milestones.points, [0; 3]);
    }

    #[test]
    fn live_navigation_resizes_and_releases_typing_without_resetting_drafts() {
        let request =
            Request::parse("# Logical pixels\npage = talents\nwidth = 1024\nheight = 768 # 4:3\n")
                .unwrap();
        let mut armory = Armory {
            page: Page::Heroes,
            focus: 1,
            draft_first: "Preview".into(),
            ..default()
        };
        let mut typing = Typing(true);
        let mut window = Window::default();
        request.apply(&mut armory, &mut typing, &mut window, 0);
        assert!(armory.page == Page::Talents);
        assert_eq!(armory.focus, 0);
        assert!(!typing.0);
        assert_eq!(armory.draft_first, "Preview");
        assert_eq!(window.resolution.width(), 1024.);
        assert_eq!(window.resolution.height(), 768.);
    }
}
