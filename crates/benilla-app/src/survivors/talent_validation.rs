//! Validate saved specs against the same catalog and tree rules as talent purchases.

use bevy::prelude::*;

use super::account::{Account, Character};

#[derive(Debug, Default)]
pub(super) struct Rules {
    classes: Vec<Vec<Vec<benilla_formats::Talent>>>,
}

impl Rules {
    fn from_catalog(catalog: &benilla_formats::TalentCatalog) -> Self {
        Self {
            classes: super::data::CLASSES
                .iter()
                .map(|class| {
                    super::armory::class_trees(catalog, class.id)
                        .into_iter()
                        .map(|(_, nodes)| nodes)
                        .collect()
                })
                .collect(),
        }
    }

    pub(super) fn valid(&self, hero: &Character) -> bool {
        let ranks = &hero.dbc[usize::from(hero.spec).min(2)];
        if ranks.is_empty() {
            return true;
        }
        let Some(trees) = self
            .classes
            .get(hero.class)
            .filter(|trees| trees.len() == 3)
        else {
            return false;
        };
        // Membership checks prevent unknown or foreign IDs being ignored by valid_tree.
        for (index, &(tree, id, rank)) in ranks.iter().enumerate() {
            if rank == 0
                || ranks[..index].iter().any(|entry| entry.1 == id)
                || !trees
                    .get(usize::from(tree))
                    .is_some_and(|nodes| nodes.iter().any(|node| node.id == id))
            {
                return false;
            }
        }
        trees.iter().enumerate().all(|(tree, nodes)| {
            let selected: Vec<_> = ranks
                .iter()
                .filter(|entry| usize::from(entry.0) == tree)
                .map(|entry| (entry.1, entry.2))
                .collect();
            super::armory::talent_ranks_ok(nodes, &selected)
        })
    }
}

/// PreUpdate installs a fail-closed pending catalog before any menu/run consumers.
pub(super) fn load_rules(
    mut account: ResMut<Account>,
    talents: Option<Res<crate::ui_talent::Talents>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    if let Some(talents) = talents {
        account.talent_rules = Some(std::sync::Arc::new(Rules::from_catalog(&talents.catalog)));
        *loaded = true;
    } else if account.talent_rules.is_none() {
        account.talent_rules = Some(std::sync::Arc::new(Rules::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Rules {
        let node = |id, row, max: usize, prerequisite| benilla_formats::Talent {
            id,
            tab: 1,
            row,
            col: 0,
            ranks: std::array::from_fn(|i| if i < max { 1 } else { 0 }),
            prereq_talent: prerequisite,
            prereq_rank: 4,
            required_spell: 0,
            exceptional: false,
        };
        let mut classes = vec![vec![vec![], vec![], vec![]]; 4];
        classes[3][0] = vec![node(270, 0, 5, 0), node(276, 1, 3, 270), node(272, 2, 2, 0)];
        classes[3][1] = vec![node(187, 0, 5, 0)];
        Rules { classes }
    }

    #[test]
    fn catalog_rejects_foreign_nodes_wrong_trees_caps_rows_and_prerequisites() {
        let rules = rules();
        let mut account = super::super::account::empty();
        account.ensure(3);
        let hero = &mut account.characters[0];
        hero.talent_xp = 5100;
        for ranks in [
            vec![(0, 999999, 1)],                        // foreign/unknown ID
            vec![(1, 270, 5)],                           // wrong tree
            vec![(3, 270, 5)],                           // impossible tree
            vec![(0, 270, 6)],                           // rank cap
            vec![(0, 270, 0)],                           // empty saved node
            vec![(0, 270, 5), (0, 270, 1)],              // duplicate bypassing parser
            vec![(0, 272, 1)],                           // insufficient earlier rows
            vec![(0, 270, 4), (0, 276, 1)],              // prerequisite not full
            vec![(0, 270, 5), (0, 276, 3), (0, 272, 1)], // same row cannot fund itself
        ] {
            hero.dbc[0] = ranks;
            assert!(!rules.valid(hero), "{:?}", hero.dbc[0]);
        }
        hero.dbc[0] = vec![(0, 270, 5), (0, 276, 3)];
        assert!(rules.valid(hero));
        hero.class = 2;
        assert!(!rules.valid(hero));
    }

    #[test]
    fn invalid_spec_is_preserved_but_inactive_and_recovers_after_refund() {
        let mut account = super::super::account::empty();
        let id = account.ensure(3);
        account.characters[0].talent_xp = 5100;
        let baseline = account.bonus(3, id);
        account.characters[0].dbc[0] = vec![(0, 270, 5), (0, 276, 3), (0, 272, 1)];
        account.characters[0].dbc[1] = vec![(1, 187, 5)];
        let saved = account.characters[0].dbc.clone();
        account.talent_rules = Some(std::sync::Arc::new(rules()));
        assert_eq!(account.bonus(3, id).might, baseline.might);
        let snapshot = account.milestones_picked(3, Some(id));
        assert_eq!(snapshot.points, [0; 3]);
        assert_eq!(snapshot.cast("eviscerate"), Default::default());
        assert_eq!(account.characters[0].dbc, saved);
        account.select_spec(id, 1);
        assert_eq!(account.bonus(3, id).armor, baseline.armor + 15.0);
        account.select_spec(id, 0);
        assert!(account.unlearn_talent(id, 272, true));
        assert!(account.talents_valid(&account.characters[0]));
        assert!(
            account
                .milestones_picked(3, Some(id))
                .cast("eviscerate")
                .damage
                > 1.0
        );
        assert_eq!(snapshot.points, [0; 3]);
        account.respec(id);
        assert!(account.talents_valid(&account.characters[0]));
    }

    #[test]
    fn runtime_suspends_saved_dbc_effects_until_catalog_is_available() {
        let mut account = super::super::account::empty();
        let id = account.ensure(3);
        account.characters[0].talent_xp = 500;
        account.characters[0].dbc[0] = vec![(0, 270, 5)];
        let mut app = App::new();
        app.insert_resource(account)
            .add_systems(PreUpdate, load_rules);
        app.update();
        let mut account = app.world_mut().resource_mut::<Account>();
        assert_eq!(account.milestones_picked(3, Some(id)).points, [0; 3]);
        account.talent_rules = Some(std::sync::Arc::new(rules()));
        assert_eq!(account.milestones_picked(3, Some(id)).points, [5, 0, 0]);
    }
}
