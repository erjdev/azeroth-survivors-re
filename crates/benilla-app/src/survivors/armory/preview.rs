//! Opt-in live controls for the production menu, independent of gameplay and saves.
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use super::Phase;
use super::{Armory, Page, Typing};
use crate::survivors::{
    cards::PreviewHand,
    data,
    hud::{Choice, Offer},
    Owned, Run,
};
use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy::window::PrimaryWindow;

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
pub(crate) fn plugin(app: &mut App) {
    let Some(path) = std::env::var_os("WOW_SURVIVORS_UI_PREVIEW") else {
        return;
    };
    app.insert_resource(Preview {
        path: path.into(),
        seen: String::new(),
        poll: Timer::from_seconds(0.25, TimerMode::Repeating),
        saved: None,
    })
    .add_systems(
        Update,
        reload.before(super::input).run_if(in_state(Phase::Menu)),
    );
}

#[derive(Resource)]
#[cfg_attr(not(feature = "dev"), allow(dead_code))]
struct Preview {
    path: std::path::PathBuf,
    seen: String,
    poll: Timer,
    saved: Option<(Run, Offer)>,
}

struct Request {
    page: Page,
    width: u32,
    height: u32,
    selected: bool,
    cards: Option<Vec<(&'static data::AbilityDef, u8)>>,
}

impl Request {
    fn parse(text: &str) -> Option<Self> {
        let (mut page, mut width, mut height) = (None, None, None);
        let mut cards_page = false;
        let mut cards = None;
        let mut profile = None;
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
                            (rank < data::MAX_ABILITY_LEVEL).then_some((def, rank))
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
                "width" if width.is_none() => width = Some(value.parse::<u32>().ok()?),
                "height" if height.is_none() => height = Some(value.parse::<u32>().ok()?),
                _ => return None,
            }
        }
        if (!cards_page && profile.is_some())
            || cards_page != cards.is_some()
            || cards
                .as_ref()
                .is_some_and(|cards| cards.is_empty() || cards.len() > 4)
        {
            return None;
        }
        let request = Self {
            page: page?,
            width: width?,
            height: height?,
            cards,
            selected: profile.unwrap_or(false),
        };
        ((640..=7680).contains(&request.width) && (480..=4320).contains(&request.height))
            .then_some(request)
    }

    fn apply(&self, armory: &mut Armory, typing: &mut Typing, window: &mut Window) {
        armory.page = self.page;
        armory.manage_heroes = false;
        armory.focus = 0;
        typing.0 = false;
        window.resolution.set(self.width as f32, self.height as f32);
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
fn reload(
    time: Res<Time<Real>>,
    mut preview: ResMut<Preview>,
    mut armory: ResMut<Armory>,
    mut typing: ResMut<Typing>,
    mut run: ResMut<Run>,
    mut offer: ResMut<Offer>,
    mut hand: ResMut<PreviewHand>,
    account: Res<crate::survivors::account::Account>,
    selection: Res<crate::survivors::Selection>,
    renown: Res<crate::survivors::renown::Renown>,
    stash: Res<crate::survivors::stash::Stash>,
    talents: Option<Res<crate::ui_talent::Talents>>,
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
    let snapshot = if request.selected {
        if talents.is_none() {
            return;
        }
        let Some(snapshot) = selected_run(&account, &selection, &renown, &stash) else {
            return;
        };
        snapshot
    } else {
        Run::default()
    };
    preview.seen = text;
    request.apply(&mut armory, &mut typing, &mut window);
    preview.hand(&request, &mut run, &mut offer, &mut hand, snapshot);
    info!("Survivors UI preview applied: {}", preview.seen.trim());
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
    run.apply_perks(account.perks);
    run.milestones = account.milestones_picked(class, selection.character);
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
            offer.choices.clear();
            for &(def, rank) in cards {
                if rank == 0 {
                    offer.choices.push(Choice::NewAbility(def));
                } else {
                    offer.choices.push(Choice::Upgrade(run.abilities.len()));
                    run.abilities.push(Owned {
                        def,
                        level: rank,
                        cd: 0.0,
                    });
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
    fn preview_reloads_hand_and_restores_original_run_and_offer() {
        let mut preview = Preview {
            path: "unused".into(),
            seen: String::new(),
            poll: Timer::from_seconds(0.25, TimerMode::Repeating),
            saved: None,
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
        request.apply(&mut armory, &mut typing, &mut window);
        assert!(armory.page == Page::Talents);
        assert_eq!(armory.focus, 0);
        assert!(!typing.0);
        assert_eq!(armory.draft_first, "Preview");
        assert_eq!(window.resolution.width(), 1024.);
        assert_eq!(window.resolution.height(), 768.);
    }
}
