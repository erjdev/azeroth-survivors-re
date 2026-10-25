//! An uncommitted rename belongs to one hero, independently of the creation draft.
use super::{Account, Selection};

#[derive(Clone)]
pub(super) struct Draft {
    pub hero: u32,
    pub first: String,
    pub last: String,
}

impl Draft {
    pub fn begin(account: &Account, selection: &Selection) -> Option<Self> {
        let hero = account.character(selection.character?)?;
        (selection.class == Some(hero.class)).then(|| Self {
            hero: hero.id,
            first: hero.first.clone(),
            last: hero.last.clone(),
        })
    }

    pub fn matches(&self, account: &Account, selection: &Selection) -> bool {
        selection.character == Some(self.hero)
            && account
                .character(self.hero)
                .is_some_and(|hero| selection.class == Some(hero.class))
    }

    pub fn confirm(
        &self,
        account: &mut Account,
        selection: &Selection,
    ) -> Result<(), &'static str> {
        if !self.matches(account, selection) {
            return Err("Select the hero whose name you are editing.");
        }
        if account.rename(self.hero, &self.first, &self.last) {
            Ok(())
        } else {
            Err("Enter both a first and last name.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_is_explicit_and_cannot_follow_a_different_hero() {
        let mut account = crate::survivors::account::empty();
        let first = account.create("First", "Hero", 0).unwrap();
        let second = account.create("Second", "Hero", 0).unwrap();
        let mut selection = Selection {
            character: Some(first),
            class: Some(0),
            ..Default::default()
        };
        let mut draft = Draft::begin(&account, &selection).unwrap();
        draft.first = "Changed".into();
        assert_eq!(account.character(first).unwrap().first, "First");
        selection.character = Some(second);
        assert!(draft.confirm(&mut account, &selection).is_err());
        assert_eq!(account.character(second).unwrap().first, "Second");
        selection.character = Some(first);
        draft.last.clear();
        assert!(draft.confirm(&mut account, &selection).is_err());
        assert_eq!(account.character(first).unwrap().first, "First");
        draft.last = "Surname".into();
        draft.confirm(&mut account, &selection).unwrap();
        assert_eq!(account.character(first).unwrap().first, "Changed");
        assert_eq!(account.character(first).unwrap().last, "Surname");
    }
}
