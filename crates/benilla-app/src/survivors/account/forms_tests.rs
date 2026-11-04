use super::*;
use survivors_core::talents::forms::DruidForm;

#[test]
fn permanent_forms_preserve_independent_specs_save_refunds_and_run_snapshots() {
    let mut account = empty();
    let id = account.ensure(8);
    account.characters[0].talent_xp = 5100;
    let cat = DruidForm::Cat.talent();
    let bear = DruidForm::Bear.talent();
    assert!(account.learn_talent(id, 1, cat, 1, 0, true));
    let snapshot = account.milestones_picked(8, Some(id));
    assert_eq!(snapshot.druid_form(), Some(DruidForm::Cat));
    assert!(!account.learn_talent(id, 1, bear, 1, 0, true));
    account.select_spec(id, 1);
    assert!(account.learn_talent(id, 1, bear, 1, 0, true));
    let mut loaded = parse(&encode(&account)).unwrap();
    assert_eq!(
        loaded.milestones_picked(8, Some(id)).druid_form(),
        Some(DruidForm::Bear)
    );
    assert!((loaded.milestones_picked(8, Some(id)).health_multiplier() - 1.25).abs() < 0.00001);
    loaded.select_spec(id, 0);
    assert_eq!(
        loaded.milestones_picked(8, Some(id)).druid_form(),
        Some(DruidForm::Cat)
    );
    assert!(loaded.unlearn_talent(id, cat, true));
    assert_eq!(loaded.milestones_picked(8, Some(id)).druid_form(), None);
    assert_eq!(
        snapshot.druid_form(),
        Some(DruidForm::Cat),
        "in-progress run must retain its snapshot"
    );
    assert!(loaded.learn_talent(id, 1, bear, 1, 0, true));
    loaded.respec(id);
    assert_eq!(loaded.milestones_picked(8, Some(id)).druid_form(), None);
}

#[test]
fn conflicting_imported_forms_grant_no_talents_and_remain_repairable() {
    for left in DruidForm::ALL {
        for right in DruidForm::ALL {
            if left == right {
                continue;
            }
            let mut account = empty();
            let id = account.ensure(8);
            account.characters[0].talent_xp = 5100;
            account.characters[0].dbc[0] = vec![
                (left.tree(), left.talent(), 1),
                (right.tree(), right.talent(), 1),
            ];
            let mut loaded = parse(&encode(&account)).unwrap();
            assert!(!loaded.talents_valid(&loaded.characters[0]));
            assert_eq!(loaded.milestones_picked(8, Some(id)).points, [0; 3]);
            assert_eq!(loaded.characters[0].dbc[0], account.characters[0].dbc[0]);
            assert!(loaded.unlearn_talent(id, right.talent(), true));
            assert_eq!(
                loaded.milestones_picked(8, Some(id)).druid_form(),
                Some(left)
            );
        }
    }
}

#[test]
fn installed_druid_catalog_form_cells_are_unique_and_purchase_valid() {
    let data = benilla_formats::wow_data_or_skip!();
    let mut chain = benilla_formats::Chain::open(&data).unwrap();
    let catalog = benilla_formats::load_talent_catalog(&mut chain).unwrap();
    let trees = super::super::armory::class_trees(&catalog, 11);
    for (_, nodes) in &trees {
        for (i, node) in nodes.iter().enumerate() {
            assert!(
                !nodes[..i]
                    .iter()
                    .any(|other| (other.row, other.col) == (node.row, node.col)),
                "overlapping talent cell {}",
                node.id
            );
        }
    }
    let mut app = App::new();
    let mut account = empty();
    let id = account.ensure(8);
    account.characters[0].talent_xp = 5100;
    app.insert_resource(account)
        .insert_resource(crate::ui_talent::Talents { catalog })
        .add_systems(PreUpdate, super::super::talent_validation::load_rules);
    app.update();
    let mut account = app.world_mut().resource_mut::<Account>();
    assert!(account.learn_talent(id, 1, DruidForm::Cat.talent(), 1, 0, true));
    assert!(account.talents_valid(&account.characters[0]));
    assert!(account.unlearn_talent(id, DruidForm::Cat.talent(), true));
    assert!(account.learn_talent(id, 1, DruidForm::Bear.talent(), 1, 0, true));
    assert!(account.talents_valid(&account.characters[0]));
}
