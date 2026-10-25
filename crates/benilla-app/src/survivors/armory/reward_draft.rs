//! Reference Confirm/Undo flow: replay purchases without touching the live account or disk.
use super::{buy_reward, Account};

#[derive(Default)]
pub(super) struct Draft(Vec<u8>);

impl Draft {
    pub fn codes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn preview(&self, account: &Account) -> Result<Account, &'static str> {
        let mut preview = account.clone();
        for &code in &self.0 {
            buy_reward(&mut preview, code)?;
        }
        Ok(preview)
    }

    pub fn stage(&mut self, account: &Account, code: u8) -> Result<(), &'static str> {
        let mut preview = self.preview(account)?;
        buy_reward(&mut preview, code)?;
        self.0.push(code);
        Ok(())
    }

    /// Revalidate against the current account; a failed batch never partially applies.
    pub fn confirm(&mut self, account: &mut Account) -> Result<(), &'static str> {
        if self.is_empty() {
            return Err("No pending rewards");
        }
        *account = self.preview(account)?;
        self.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivors::{account, campaign};

    fn funded() -> Account {
        let mut account = account::blank();
        for class in 0..9 {
            for zone in 0..campaign::ZONES.len() {
                account.grant_zone(class, zone);
            }
        }
        account.grant_new();
        account
    }

    #[test]
    fn preview_and_undo_do_not_spend_or_consume_resets() {
        let account = funded();
        let points = account.unspent();
        let mut draft = Draft::default();
        draft.stage(&account, 0).unwrap();
        draft.stage(&account, 6).unwrap();
        draft.stage(&account, 12).unwrap();
        let preview = draft.preview(&account).unwrap();
        assert_eq!(preview.pets.leash, 1);
        assert_eq!(preview.perks.second_thoughts, 0);
        assert_eq!(preview.resets_left(), 1);
        assert_eq!(preview.unspent(), points - 15);
        assert_eq!(account.pets.leash, 0);
        assert_eq!(account.resets_left(), 2);
        assert_eq!(account.unspent(), points);
        draft.clear();
        assert_eq!(draft.preview(&account).unwrap().unspent(), points);
    }

    #[test]
    fn confirm_revalidates_atomically_and_preserves_intervening_changes() {
        let mut account = funded();
        let mut draft = Draft::default();
        draft.stage(&account, 6).unwrap();
        draft.stage(&account, 11).unwrap();
        account.buy_second().unwrap();
        let points = account.unspent();
        assert_eq!(draft.confirm(&mut account), Err("Max rank"));
        assert_eq!(account.unspent(), points);
        assert_eq!(account.pets.leash, 0);
        assert_eq!(draft.codes(), &[6, 11]);
        draft.clear();
        draft.stage(&account, 6).unwrap();
        account.tabard = 2;
        account.pets.look[0] = 3;
        draft.confirm(&mut account).unwrap();
        assert_eq!(account.unspent(), points - 15);
        assert_eq!(account.pets.leash, 1);
        assert_eq!(account.pets.look[0], 3);
        assert_eq!(account.tabard, 2);
        assert!(draft.is_empty());
        assert!(draft.confirm(&mut account).is_err());
        assert_eq!(account.unspent(), points - 15);
    }

    #[test]
    fn invalid_or_unaffordable_actions_never_enter_the_draft() {
        let account = account::blank();
        let mut draft = Draft::default();
        assert!(draft.stage(&account, 6).is_err());
        assert!(draft.stage(&account, 255).is_err());
        assert!(draft.is_empty());
    }
}
