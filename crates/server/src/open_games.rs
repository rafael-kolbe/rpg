//! Which connection holds each account's game. Two tabs on the same game
//! would each keep their own copy and overwrite the other's save, so a tab
//! opened before a lost fight could store the world as it was before it.
//! Only the newest game of an account is live; an older one is closed at
//! its next message or tick, without storing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use crate::accounts::AccountId;

/// The live game of each account, by ticket.
#[derive(Debug, Default)]
pub(crate) struct OpenGames {
    live: Mutex<HashMap<AccountId, u64>>,
    last_ticket: AtomicU64,
}

impl OpenGames {
    /// Makes a new game the account's live one; returns its ticket.
    pub(crate) fn claim(&self, account: AccountId) -> u64 {
        let ticket = self.last_ticket.fetch_add(1, Ordering::Relaxed) + 1;
        self.lock().insert(account, ticket);
        ticket
    }

    /// Whether `ticket` is still the account's live game.
    pub(crate) fn holds(&self, account: AccountId, ticket: u64) -> bool {
        self.lock().get(&account) == Some(&ticket)
    }

    /// Forgets a closed game, unless a newer one took its place.
    pub(crate) fn release(&self, account: AccountId, ticket: u64) {
        let mut live = self.lock();
        if live.get(&account) == Some(&ticket) {
            live.remove(&account);
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<AccountId, u64>> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_game_of_an_account_is_the_live_one() {
        let games = OpenGames::default();
        let account = AccountId::for_tests(1);
        let first = games.claim(account);
        let second = games.claim(account);

        assert!(!games.holds(account, first));
        assert!(games.holds(account, second));

        games.release(account, first);
        assert!(
            games.holds(account, second),
            "an old tab closing changes nothing"
        );
        games.release(account, second);
        assert!(!games.holds(account, second));
    }

    #[test]
    fn accounts_do_not_share_a_game() {
        let games = OpenGames::default();
        let ticket = games.claim(AccountId::for_tests(1));
        games.claim(AccountId::for_tests(2));

        assert!(games.holds(AccountId::for_tests(1), ticket));
    }
}
