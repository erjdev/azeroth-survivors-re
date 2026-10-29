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
        let before = account.milestones_picked(class, Some(hero));
        for rank in 1..=max_rank {
            assert!(account.learn_talent(hero, 2, node, max_rank, row, true));
            let trained = account.milestones_picked(class, Some(hero));
            let mut expected = before.cast(key);
            if node == 825 {
                expected.crit += 0.1 * f32::from(rank);
            } else {
                expected.damage *= 1.2;
            }
            assert_eq!(trained.cast(key), expected, "node {node} rank {rank}");
            assert_eq!(trained.cast("fireball"), before.cast("fireball"));
        }
        assert!(!account.learn_talent(hero, 2, node, max_rank, row, true));
        let trained = account.milestones_picked(class, Some(hero));
        let mut loaded = parse(&encode(&account)).unwrap();
        loaded.talent_rules = Some(rules.clone());
        assert!(loaded.talents_valid(loaded.character(hero).unwrap()));
        assert_eq!(loaded.milestones_picked(class, Some(hero)), trained);
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
