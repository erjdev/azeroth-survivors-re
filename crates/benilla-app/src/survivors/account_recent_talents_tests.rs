//! Save-boundary acceptance for acquired abilities' named training nodes.
use super::*;

#[test]
fn recent_training_purchases_survive_specs_reload_and_refund_without_granting_other_bonuses() {
    let data = benilla_formats::wow_data_or_skip!();
    let mut chain = benilla_formats::Chain::open(&data).unwrap();
    let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
    let mut app = App::new();
    app.insert_resource(empty())
        .insert_resource(crate::ui_talent::Talents { catalog })
        .add_systems(PreUpdate, super::super::talent_validation::load_rules);
    app.update();
    let rules = app
        .world()
        .resource::<Account>()
        .talent_rules
        .clone()
        .unwrap();
    for (class, node, row, max_rank, key, initial) in [
        (
            3,
            206,
            3,
            2,
            "kick",
            vec![
                (1, 203, 3),
                (1, 201, 2),
                (1, 181, 5),
                (1, 202, 3),
                (1, 204, 2),
            ],
        ),
        (
            4,
            541,
            4,
            1,
            "silence",
            vec![
                (2, 464, 5),
                (2, 465, 5),
                (2, 463, 5),
                (2, 542, 2),
                (2, 481, 3),
            ],
        ),
        (
            6,
            88,
            3,
            2,
            "counterspell",
            vec![(0, 76, 5), (0, 80, 5), (0, 1650, 5)],
        ),
        (
            1,
            1402,
            5,
            5,
            "Weapon Attack",
            vec![
                (2, 1401, 5),
                (2, 1407, 5),
                (2, 1403, 5),
                (2, 1411, 5),
                (2, 1633, 3),
                (2, 1405, 2),
            ],
        ),
        (
            1,
            1429,
            5,
            5,
            "Weapon Attack",
            vec![
                (1, 1422, 5),
                (1, 1421, 5),
                (1, 1423, 5),
                (1, 1629, 5),
                (1, 1426, 5),
            ],
        ),
        (
            1,
            1410,
            4,
            3,
            "Weapon Attack",
            vec![(2, 1401, 5), (2, 1407, 5), (2, 1403, 5), (2, 1411, 5)],
        ),
        (
            5,
            616,
            4,
            1,
            "lightning_bolt",
            vec![
                (1, 614, 5),
                (1, 613, 5),
                (1, 605, 2),
                (1, 615, 5),
                (1, 601, 3),
            ],
        ),
        (5, 586, 0, 5, "healing_wave", vec![]),
        (5, 593, 0, 5, "healing_wave", vec![]),
        (
            5,
            594,
            3,
            5,
            "healing_wave",
            vec![(2, 586, 5), (2, 593, 5), (2, 595, 5)],
        ),
        (
            5,
            592,
            5,
            5,
            "healing_wave",
            vec![
                (2, 586, 5),
                (2, 593, 5),
                (2, 595, 5),
                (2, 588, 5),
                (2, 594, 5),
            ],
        ),
        (
            1,
            1461,
            3,
            5,
            "holy_light",
            vec![(0, 1450, 5), (0, 1449, 5), (0, 1432, 5)],
        ),
        (
            5,
            1648,
            4,
            3,
            "healing_wave",
            vec![
                (2, 586, 5),
                (2, 593, 5),
                (2, 581, 3),
                (2, 595, 5),
                (2, 582, 1),
                (2, 583, 1),
            ],
        ),
        (
            1,
            1444,
            2,
            3,
            "holy_light",
            vec![(0, 1450, 5), (0, 1449, 5)],
        ),
        (
            1,
            1444,
            2,
            3,
            "flash_of_light",
            vec![(0, 1450, 5), (0, 1449, 5)],
        ),
        (
            1,
            1405,
            3,
            2,
            "retribution_aura",
            vec![(2, 1401, 5), (2, 1407, 5), (2, 1403, 5)],
        ),
        (
            4,
            408,
            3,
            3,
            "greater_heal",
            vec![(1, 401, 5), (1, 1181, 5), (1, 411, 5)],
        ),
        (
            3,
            303,
            2,
            1,
            "ghostly_strike",
            vec![(2, 261, 5), (2, 1700, 2), (2, 247, 2), (2, 244, 5)],
        ),
        (
            3,
            681,
            4,
            1,
            "hemorrhage",
            vec![
                (2, 261, 5),
                (2, 1700, 2),
                (2, 247, 2),
                (2, 244, 5),
                (2, 303, 1),
                (2, 246, 2),
                (2, 1123, 3),
            ],
        ),
        (
            8,
            825,
            5,
            5,
            "regrowth",
            vec![
                (2, 821, 5),
                (2, 824, 5),
                (2, 823, 5),
                (2, 841, 5),
                (2, 843, 5),
            ],
        ),
    ] {
        let mut account = empty();
        account.talent_rules = Some(rules.clone());
        let hero = account.ensure(class);
        let c = account
            .characters
            .iter_mut()
            .find(|c| c.id == hero)
            .unwrap();
        c.talent_xp = 5100;
        c.dbc[0] = initial.clone();
        assert!(
            account.talents_valid(account.character(hero).unwrap()),
            "node {node}"
        );
        let tree = initial.first().map_or(2, |entry| entry.0);
        if node == 616 {
            assert!(account.collect_poor_drop(class, 8).is_some());
        }
        if matches!(node, 1429 | 1410) {
            account.classes[class].pieces[8] = 1;
            account.classes[class].pieces[17] = 1;
            let band = u8::from(node == 1410);
            assert!(account.wear_piece(class, 8, Some(band)));
            assert!(account.show_piece(class, 8, super::super::equipment::Look::Piece(1 - band)));
        }
        let before = account.milestones_picked(class, Some(hero));
        for rank in 1..=max_rank {
            assert!(account.learn_talent(hero, tree, node, max_rank, row, true));
            let trained = account.milestones_picked(class, Some(hero));
            let mut expected = before.cast(key);
            if matches!(node, 1429 | 1410) {
                expected.damage *= 1. + 0.02 * f32::from(rank);
                assert_eq!(trained.cast("judgement"), before.cast("judgement"));
            } else if node == 206 {
                assert_eq!(trained.kick_silence(), 0.5 * f32::from(rank));
            } else if node == 541 {
                assert!(!before.ability_unlocked(key));
                assert!(trained.ability_unlocked(key));
            } else if node == 88 {
                assert_eq!(trained.counterspell_silence(), 0.5 * f32::from(rank));
            } else if node == 1402 {
                assert_eq!(trained.vengeance_bonus(true), 0.03 * f32::from(rank));
                assert!(trained.triggers_vengeance(key, 10., true));
            } else if node == 616 {
                assert_eq!(trained.melee_parry(), 0.05);
                assert_eq!(before.melee_parry(), 0.);
            } else if node == 586 || node == 593 {
                expected.cooldown *= 1. - if node == 586 { 0.04 } else { 0.02 } * f32::from(rank);
            } else if node == 592 {
                expected.damage *= 1. + 0.02 * f32::from(rank);
            } else if node == 594 {
                expected.crit += 0.01 * f32::from(rank);
            } else if node == 1461 {
                assert_eq!(
                    trained.illumination_chance(key, 1., true),
                    0.2 * f32::from(rank)
                );
            } else if node == 1648 {
                let expected_chance = [0., 0.33, 0.66, 1.][usize::from(rank)];
                assert_eq!(
                    survivors_core::talents::HealingWay::chance(trained, key),
                    expected_chance
                );
            } else if node == 1444 {
                expected.damage *= 1.0 + 0.04 * f32::from(rank);
            } else if node == 1405 {
                expected.damage *= 1.0 + 0.25 * f32::from(rank);
            } else if node == 408 {
                expected.cooldown *= 1. - 0.05 * f32::from(rank);
            } else if node == 825 {
                expected.crit += 0.1 * f32::from(rank);
            } else {
                expected.damage *= 1.2;
            }
            assert_eq!(trained.cast(key), expected, "node {node} rank {rank}");
            assert_eq!(trained.cast("fireball"), before.cast("fireball"));
        }
        assert!(!account.learn_talent(hero, tree, node, max_rank, row, true));
        let trained = account.milestones_picked(class, Some(hero));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules.clone());
        assert!(loaded.talents_valid(loaded.character(hero).unwrap()));
        assert_eq!(loaded.milestones_picked(class, Some(hero)), trained);
        if node == 616 {
            let snapshot = loaded.milestones_picked(class, Some(hero));
            let key = survivors_core::supplemental_gear::piece(
                class,
                8,
                survivors_core::gear_tier::GearTier::Poor,
            )
            .unwrap()
            .key;
            assert!(loaded.wear_piece(class, 8, None));
            assert!(loaded.select_supplemental(class, &key, true));
            assert_eq!(
                loaded.milestones_picked(class, Some(hero)).melee_parry(),
                0.,
                "cosmetic-only weapon cannot parry"
            );
            assert_eq!(
                snapshot.melee_parry(),
                0.05,
                "running snapshot remains immutable"
            );
            assert!(loaded.select_supplemental(class, &key, false));
            assert_eq!(
                loaded.milestones_picked(class, Some(hero)).melee_parry(),
                0.05
            );
        }
        loaded.select_spec(hero, 1);
        assert!(loaded.character(hero).unwrap().dbc[1].is_empty());
        assert_eq!(
            loaded.milestones_picked(class, Some(hero)).cast(key),
            survivors_core::talents::Milestones {
                class,
                ..Default::default()
            }
            .cast(key)
        );
        loaded.select_spec(hero, 0);
        assert_eq!(loaded.milestones_picked(class, Some(hero)), trained);
        assert!(!loaded.unlearn_talent(hero, node, false));
        for _ in 0..max_rank {
            assert!(loaded.unlearn_talent(hero, node, true));
        }
        assert!(!loaded.unlearn_talent(hero, node, true));
        assert_eq!(loaded.character(hero).unwrap().dbc[0], initial);
        assert_eq!(loaded.milestones_picked(class, Some(hero)), before);
        let refunded = parse(&encode(&loaded)).unwrap();
        assert_eq!(refunded.character(hero).unwrap().dbc[0], initial);
        assert!(refunded.character(hero).unwrap().dbc[1].is_empty());
    }
}
