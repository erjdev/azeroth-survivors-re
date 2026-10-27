#[test]
fn legacy_rank_template_resolves_translated_argument_order() {
    let script = benilla_ui::script::UiScript::new().unwrap();
    script
        .lua()
        .load("TOOLTIP_TALENT_RANK = 'Maximum %2$s, rang %1$s'")
        .exec()
        .unwrap();
    let strings = super::super::localization::Strings::from_script(&script);
    assert_eq!(
        super::milestone_label(&strings, 0, 0, 0, 2, 5),
        "Maximum 5, rang 2"
    );
}
use super::super::account;
use super::super::data;
use super::super::Selection;
use super::enter_refused;

#[test]
fn item_source_navigation_preserves_item_and_next_run_zone() {
    let mut armory = super::Armory {
        page: super::Page::Collection,
        piece: 17,
        picked: Some(4),
        drop_scroll: bevy::prelude::Vec2::splat(300.),
        drop_columns: [bevy::prelude::Vec2::splat(200.); 4],
        ..Default::default()
    };
    armory.open_drop_source(1);
    assert!(armory.page == super::Page::DropTables);
    assert_eq!(armory.drop_zone, 1);
    assert_eq!(armory.piece, 17);
    assert_eq!(armory.entered_zone(), Some(4));
    assert_eq!(armory.drop_scroll, bevy::prelude::Vec2::ZERO);
    assert_eq!(armory.drop_columns, [bevy::prelude::Vec2::ZERO; 4]);
    armory.open_drop_source(usize::MAX);
    assert_eq!(armory.drop_zone, 1);
    assert_eq!(armory.piece, 17);
}

#[test]
fn talent_purchase_feedback_identifies_the_blocker_without_changing_saved_ranks() {
    let data = benilla_formats::wow_data_or_skip!();
    let mut chain = benilla_formats::Chain::open(&data).unwrap();
    let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
    for (xp, ranks, id, expected) in [
        (
            0,
            vec![],
            69,
            "No talent points to spend. Earn talent experience in runs or refund a point.",
        ),
        (
            200,
            vec![(2, 70, 2)],
            70,
            "Talent 70 is already at maximum rank 2",
        ),
        (
            5100,
            vec![],
            69,
            "Talent 69 requires 10 points in earlier rows of this tree; 0 spent.",
        ),
        (
            5100,
            vec![(2, 37, 5)],
            69,
            "Talent 69 requires 10 points in earlier rows of this tree; 5 spent.",
        ),
        (5100, vec![], 421, "Talent 421 requires Talent 86 rank 1"),
        (
            5100,
            vec![(2, 37, 6)],
            70,
            "This saved tree is invalid. Refund conflicting ranks or reset this spec.",
        ),
    ] {
        let mut account = account::empty();
        let hero = account.ensure(6);
        account.characters[0].talent_xp = xp;
        account.characters[0].dbc[0] = ranks;
        account.characters[0].dbc[1] = vec![(2, 37, 1)];
        let before = account.characters[0].dbc.clone();
        assert_eq!(
            super::spend_talent(&mut account, hero, Some(&catalog), None, id, false),
            expected
        );
        assert_eq!(account.characters[0].dbc, before);
    }
    let mut account = account::empty();
    let hero = account.ensure(6);
    account.characters[0].talent_xp = 100;
    account.characters[0].specs[0][0] = 1;
    assert_eq!(
        super::spend_talent(&mut account, hero, Some(&catalog), None, 70, false),
        "Talent 70 rank 1"
    );
    assert_eq!(account.characters[0].dbc_rank(70), 1);
    assert_eq!(
        account.characters[0].spent(),
        0,
        "first DBC purchase returns legacy points"
    );
    assert_eq!(
        super::spend_talent(&mut account, hero, Some(&catalog), None, 70, true),
        "Talent 70 rank 0"
    );
    assert_eq!(account.characters[0].dbc_rank(70), 0);
}

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
fn stale_or_cross_class_hero_cannot_start_or_reset_the_previous_run() {
    let mut account = account::empty();
    let paladin = account.create("Selected", "Paladin", 1).unwrap();
    let mut selection = Selection {
        class: Some(0),
        character: Some(paladin),
        look: Some(0),
        looks: vec![(20318, 1, 0)],
        map: data::MAPS
            .iter()
            .position(|map| map.name == "Elwynn Forest")
            .unwrap(),
        ..Default::default()
    };
    let mut run = super::Run {
        elapsed: 123.0,
        hp: 37.0,
        ..Default::default()
    };
    for id in [paladin, u32::MAX] {
        selection.character = Some(id);
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
        assert!(matches!(next, bevy::prelude::NextState::Unchanged));
        assert!(armory.note.contains("hero"));
        assert_eq!(run.elapsed, 123.0);
        assert_eq!(run.hp, 37.0);
    }
    // A valid re-selection clears the refusal without changing account progress.
    selection.character = Some(paladin);
    selection.class = Some(1);
    assert!(enter_refused(&account, &selection, None).is_none());
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
