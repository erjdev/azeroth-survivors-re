//! A named hero owns its body choice; the menu pick remains a legacy migration source.
use super::{all_looks, find, save_pick, set, Phase, Selection};
use crate::survivors::account::Account;
use bevy::prelude::*;

pub(in crate::survivors) fn sync(
    phase: Res<State<Phase>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    mut selection: ResMut<Selection>,
    mut account: ResMut<Account>,
    mut previous: Local<Option<u32>>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    if !selection.is_changed() && *previous == selection.character {
        return;
    }
    let Some(creatures) = creatures.as_deref() else {
        return;
    };
    if reconcile(
        &mut selection,
        &mut account,
        &all_looks(creatures),
        &mut previous,
    ) {
        account.save();
        save_pick(&selection);
    }
}

fn reconcile(
    selection: &mut Selection,
    account: &mut Account,
    all: &[Vec<(u32, u8, u8)>],
    previous: &mut Option<u32>,
) -> bool {
    let Some(hero) = selection.character.and_then(|id| account.character(id)) else {
        *previous = None;
        return false;
    };
    let (id, class, saved) = (hero.id, hero.class, hero.body);
    if selection.class != Some(class) {
        *previous = None;
        return false;
    }
    let Some(looks) = all.get(class) else {
        return false;
    };
    let changed_hero = *previous != Some(id);
    if changed_hero {
        if let Some((race, sex)) = saved {
            let Some(index) = find(looks, Some(race), Some(sex)) else {
                // Keep the saved identity if its data is temporarily unavailable.
                selection.look = None;
                return false;
            };
            set(selection, all, class, index);
            *previous = Some(id);
            return true;
        }
    }
    let body = selection.look.and_then(|i| selection.looks.get(i)).copied();
    let Some((_, race, sex)) = body else {
        return false;
    };
    if find(looks, Some(race), Some(sex)).is_none() {
        return false;
    }
    *previous = Some(id);
    account.set_body(id, (race, sex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_saved_body_is_not_replaced_by_another_hero_or_class() {
        let mut account = crate::survivors::account::empty();
        let id = account.create("Saved", "Hero", 0).unwrap();
        account.set_body(id, (3, 1));
        let all = vec![vec![(100, 1, 0)]];
        let mut selection = Selection::default();
        set(&mut selection, &all, 0, 0);
        selection.character = Some(id);
        let mut previous = None;
        assert!(!reconcile(
            &mut selection,
            &mut account,
            &all,
            &mut previous
        ));
        assert_eq!(selection.look, None);
        assert_eq!(account.character(id).unwrap().body, Some((3, 1)));
        selection.class = Some(1);
        assert!(!reconcile(
            &mut selection,
            &mut account,
            &all,
            &mut previous
        ));
        assert_eq!(account.character(id).unwrap().body, Some((3, 1)));
    }

    #[test]
    fn heroes_keep_independent_bodies_and_legacy_choice_migrates() {
        let mut account = crate::survivors::account::empty();
        let a = account.create("First", "Hero", 0).unwrap();
        let b = account.create("Second", "Hero", 0).unwrap();
        let all = vec![vec![(100, 1, 0), (101, 1, 1)]];
        let mut selection = Selection::default();
        set(&mut selection, &all, 0, 0);
        selection.character = Some(a);
        let mut previous = None;
        assert!(reconcile(&mut selection, &mut account, &all, &mut previous));
        assert_eq!(account.character(a).unwrap().body, Some((1, 0)));
        selection.character = Some(b);
        set(&mut selection, &all, 0, 1);
        assert!(reconcile(&mut selection, &mut account, &all, &mut previous));
        selection.character = Some(a);
        assert!(reconcile(&mut selection, &mut account, &all, &mut previous));
        assert_eq!(selection.look, Some(0));
        selection.character = Some(b);
        assert!(reconcile(&mut selection, &mut account, &all, &mut previous));
        assert_eq!(selection.look, Some(1));
        assert!(!reconcile(
            &mut selection,
            &mut account,
            &all,
            &mut previous
        ));
        set(&mut selection, &all, 0, 0);
        assert!(reconcile(&mut selection, &mut account, &all, &mut previous));
        assert_eq!(account.character(b).unwrap().body, Some((1, 0)));
    }
}
