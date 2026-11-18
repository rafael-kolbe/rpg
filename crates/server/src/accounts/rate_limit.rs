//! Failed login and recovery attempts, counted per username and per IP, to
//! slow down password guessing. In memory: one server process, and a restart
//! forgetting the counts is harmless.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Failures allowed per key inside [`WINDOW`].
const MAX_FAILURES: usize = 10;

/// How long a failure counts.
const WINDOW: Duration = Duration::from_mins(15);

/// Recent failures by key (`user:<lowercase name>`, `ip:<address>`).
#[derive(Debug, Default)]
pub(crate) struct Attempts {
    failures: Mutex<HashMap<String, Vec<Instant>>>,
}

impl Attempts {
    /// Whether any of `keys` used up its failures at `now`.
    pub(crate) fn blocked(&self, keys: &[String], now: Instant) -> bool {
        let failures = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
        keys.iter().any(|key| {
            failures
                .get(key)
                .is_some_and(|times| recent(times, now).count() >= MAX_FAILURES)
        })
    }

    /// Counts a failure for each of `keys` at `now`, forgetting expired ones.
    pub(crate) fn fail(&self, keys: &[String], now: Instant) {
        let mut failures = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
        failures.retain(|_, times| {
            times.retain(|&time| now.duration_since(time) < WINDOW);
            !times.is_empty()
        });
        for key in keys {
            failures.entry(key.clone()).or_default().push(now);
        }
    }

    /// Forgets the failures of `key` (a successful login for that username).
    pub(crate) fn clear(&self, key: &str) {
        self.failures
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(key);
    }
}

fn recent(times: &[Instant], now: Instant) -> impl Iterator<Item = &Instant> {
    times
        .iter()
        .filter(move |&&time| now.duration_since(time) < WINDOW)
}

/// Rate-limit key of a username (case does not matter).
pub(crate) fn user_key(username: &str) -> String {
    format!("user:{}", username.to_ascii_lowercase())
}

/// Rate-limit key of a client address.
pub(crate) fn ip_key(ip: &str) -> String {
    format!("ip:{ip}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_a_key_after_ten_failures_until_they_expire() {
        let attempts = Attempts::default();
        let keys = [user_key("Taren123")];
        let start = Instant::now();

        for _ in 0..9 {
            attempts.fail(&keys, start);
        }
        assert!(!attempts.blocked(&keys, start));
        attempts.fail(&keys, start);
        assert!(attempts.blocked(&[user_key("taren123")], start));
        assert!(!attempts.blocked(&keys, start + WINDOW));
    }

    #[test]
    fn any_blocked_key_blocks_the_attempt() {
        let attempts = Attempts::default();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES {
            attempts.fail(&[ip_key("10.0.0.1")], now);
        }

        assert!(attempts.blocked(&[user_key("someone1"), ip_key("10.0.0.1")], now));
        assert!(!attempts.blocked(&[user_key("someone1"), ip_key("10.0.0.2")], now));
    }

    #[test]
    fn clearing_a_key_forgets_its_failures() {
        let attempts = Attempts::default();
        let keys = [user_key("taren123")];
        let now = Instant::now();
        for _ in 0..MAX_FAILURES {
            attempts.fail(&keys, now);
        }

        attempts.clear(&keys[0]);

        assert!(!attempts.blocked(&keys, now));
    }
}
