//! Reference Confirm/Undo flow: replay purchases without touching the live account or disk.
use super::{buy_reward, Account};
use crate::survivors::account::PurchaseError;

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

    pub fn preview(&self, account: &Account) -> Result<Account, PurchaseError> {
        let mut preview = account.clone();
        for &code in &self.0 {
            buy_reward(&mut preview, code)?;
        }
        Ok(preview)
    }

    pub fn stage(&mut self, account: &Account, code: u8) -> Result<(), PurchaseError> {
        let mut preview = self.preview(account)?;
        buy_reward(&mut preview, code)?;
        self.0.push(code);
        Ok(())
    }

    /// Remove one pending purchase without changing the destination of later instincts.
    /// Replay both histories so dependency failures leave the entire draft untouched.
    pub fn remove(&mut self, account: &Account, code: u8) -> Result<(), PurchaseError> {
        let index = self
            .0
            .iter()
            .rposition(|&pending| pending == code)
            .ok_or("No pending upgrade on this card")?;
        let mut original = account.clone();
        let mut revised = account.clone();
        for (i, &pending) in self.0.iter().enumerate() {
            if i != index {
                if (8..=10).contains(&pending)
                    && original.instinct_target() != revised.instinct_target()
                {
                    return Err("Undo later instinct purchases first".into());
                }
                buy_reward(&mut revised, pending).map_err(|_| "Undo dependent purchases first")?;
            }
            buy_reward(&mut original, pending)?;
        }
        self.0.remove(index);
        Ok(())
    }

    /// Revalidate against the current account; a failed batch never partially applies.
    pub fn confirm(&mut self, account: &mut Account) -> Result<(), PurchaseError> {
        if self.is_empty() {
            return Err("No pending rewards".into());
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

    #[test]
    fn unavailable_rune_instinct_cannot_enter_a_funded_draft() {
        let account = funded();
        let mut draft = Draft::default();
        assert!(draft.stage(&account, 10).is_err());
        assert!(draft.is_empty());
    }

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
    fn duplicate_instinct_rejection_preserves_draft_and_balance() {
        let account = funded();
        let points = account.unspent();
        let mut draft = Draft::default();
        draft.stage(&account, 6).unwrap();
        draft.stage(&account, 8).unwrap();
        assert_eq!(draft.stage(&account, 8), Err(PurchaseError::AlreadyLearned));
        assert_eq!(draft.codes(), &[6, 8]);
        assert_eq!(account.unspent(), points);
        assert_eq!(draft.preview(&account).unwrap().unspent(), points - 45);
    }

    #[test]
    fn confirm_revalidates_atomically_and_preserves_intervening_changes() {
        let mut account = funded();
        let mut draft = Draft::default();
        draft.stage(&account, 6).unwrap();
        draft.stage(&account, 11).unwrap();
        account.buy_second().unwrap();
        let points = account.unspent();
        assert_eq!(draft.confirm(&mut account), Err("Max rank".into()));
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
    fn removing_one_rank_reprices_the_draft_without_spending() {
        let mut account = funded();
        let points = account.unspent();
        let mut draft = Draft::default();
        for code in [0, 6, 0] {
            draft.stage(&account, code).unwrap();
        }
        draft.remove(&account, 0).unwrap();
        assert_eq!(draft.codes(), &[0, 6]);
        assert_eq!(draft.preview(&account).unwrap().unspent(), points - 30);
        assert_eq!(account.unspent(), points);
        draft.confirm(&mut account).unwrap();
        assert_eq!(account.perks.second_thoughts, 1);
        assert_eq!(account.pets.leash, 1);
        assert!(draft.remove(&account, 0).is_err());
    }

    #[test]
    fn removing_a_companion_or_instinct_cannot_retarget_later_purchases() {
        let account = funded();
        let mut draft = Draft::default();
        for code in [11, 8, 9] {
            draft.stage(&account, code).unwrap();
        }
        assert!(draft.remove(&account, 11).is_err());
        assert!(draft.remove(&account, 8).is_err());
        assert_eq!(draft.codes(), &[11, 8, 9]);
        draft.remove(&account, 9).unwrap();
        draft.remove(&account, 11).unwrap();
        assert_eq!(draft.codes(), &[8]);
        assert_eq!(draft.preview(&account).unwrap().pets.instinct, [1, 0]);
    }

    #[test]
    fn removing_a_reset_cannot_invalidate_later_ranks() {
        let mut account = funded();
        for _ in 0..3 {
            account
                .buy_perk(super::super::Perk::SecondThoughts)
                .unwrap();
        }
        let mut draft = Draft::default();
        draft.stage(&account, 12).unwrap();
        draft.stage(&account, 0).unwrap();
        assert!(draft.remove(&account, 12).is_err());
        assert_eq!(draft.codes(), &[12, 0]);
        draft.remove(&account, 0).unwrap();
        draft.remove(&account, 12).unwrap();
        assert!(draft.is_empty());
        assert_eq!(account.resets_left(), 2);
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
