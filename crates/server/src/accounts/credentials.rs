//! Rules and secrets of an account: what a valid username and password are,
//! password hashing, recovery keys and session tokens. No database here.

use std::ops::RangeInclusive;
use std::sync::LazyLock;

use anyhow::anyhow;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// Allowed username length, in characters.
const USERNAME_LEN: RangeInclusive<usize> = 8..=20;

/// Allowed password length, in characters.
const PASSWORD_LEN: RangeInclusive<usize> = 8..=64;

/// Crockford base32: no I, L, O or U, so a key copied by hand is hard to get
/// wrong.
const KEY_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Characters in a recovery key (5 bits each: 100 bits of randomness).
const KEY_CHARS: usize = 20;

/// Characters between dashes when a key is shown.
const KEY_GROUP: usize = 4;

/// Random bytes in a session token.
const TOKEN_BYTES: usize = 32;

/// A valid hash of an unknown password: logging in with an unknown username
/// still spends the time of a hash check, so the answer time does not tell
/// which usernames exist.
static DUMMY_HASH: LazyLock<Option<String>> =
    LazyLock::new(|| hash_password("not-a-real-password").ok());

/// 8 to 20 ASCII letters and digits.
pub(crate) fn valid_username(username: &str) -> bool {
    USERNAME_LEN.contains(&username.len()) && username.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// 8 to 64 characters, any characters.
pub(crate) fn valid_password(password: &str) -> bool {
    PASSWORD_LEN.contains(&password.chars().count())
}

/// Argon2id hash of `password` with a random salt, as a PHC string.
pub(crate) fn hash_password(password: &str) -> anyhow::Result<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|error| anyhow!("password hashing failed: {error}"))
}

/// Whether `password` matches `hash` (a PHC string). `None` checks against a
/// dummy hash and is always false (see [`DUMMY_HASH`]).
pub(crate) fn verify_password(password: &str, hash: Option<&str>) -> bool {
    let Some(hash) = hash.or(DUMMY_HASH.as_deref()) else {
        return false;
    };
    let matches = Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok();
    matches && hash != DUMMY_HASH.as_deref().unwrap_or_default()
}

/// A new recovery key, grouped for reading: `XXXX-XXXX-XXXX-XXXX-XXXX`.
pub(crate) fn new_recovery_key() -> anyhow::Result<String> {
    let mut bytes = [0u8; KEY_CHARS];
    getrandom::fill(&mut bytes).map_err(|error| anyhow!("no system randomness: {error}"))?;
    let chars: Vec<char> = bytes
        .iter()
        .map(|byte| char::from(KEY_ALPHABET[usize::from(byte % 32)]))
        .collect();
    Ok(chars
        .chunks(KEY_GROUP)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-"))
}

/// The key as stored: dashes and spaces removed, upper case, and the letters
/// people confuse with digits read as those digits (I, L → 1; O → 0).
/// `None` when it cannot be a key.
pub(crate) fn normalize_recovery_key(input: &str) -> Option<String> {
    let key: String = input
        .chars()
        .filter(|c| !matches!(c, '-' | ' '))
        .map(|c| match c.to_ascii_uppercase() {
            'I' | 'L' => '1',
            'O' => '0',
            other => other,
        })
        .collect();
    let valid = key.len() == KEY_CHARS && key.bytes().all(|b| KEY_ALPHABET.contains(&b));
    valid.then_some(key)
}

/// A new random session token, URL-safe.
pub(crate) fn new_session_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| anyhow!("no system randomness: {error}"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// SHA-256 of a high-entropy secret (session token, normalized recovery
/// key). Enough for random secrets; passwords use [`hash_password`].
pub(crate) fn digest(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_username_takes_8_to_20_letters_and_digits() {
        assert!(valid_username("taren123"));
        assert!(valid_username("ABCDEFGHIJ0123456789"));
        assert!(!valid_username("taren12"));
        assert!(!valid_username("ABCDEFGHIJ01234567890"));
        assert!(!valid_username("taren_123"));
        assert!(!valid_username("tarên1234"));
        assert!(!valid_username("taren 123"));
    }

    #[test]
    fn a_password_takes_8_to_64_characters_of_any_kind() {
        assert!(valid_password("açaí-123"));
        assert!(!valid_password("1234567"));
        assert!(valid_password(&"x".repeat(64)));
        assert!(!valid_password(&"x".repeat(65)));
    }

    #[test]
    fn a_hashed_password_verifies_only_with_itself() {
        let hash = hash_password("correct horse").unwrap();

        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("correct horse", Some(&hash)));
        assert!(!verify_password("wrong horse", Some(&hash)));
    }

    #[test]
    fn an_unknown_account_never_verifies() {
        assert!(!verify_password("not-a-real-password", None));
    }

    #[test]
    fn a_recovery_key_is_five_groups_that_normalize_back() {
        let key = new_recovery_key().unwrap();

        assert_eq!(key.len(), 24);
        assert_eq!(key.split('-').count(), 5);
        let normalized = normalize_recovery_key(&key).unwrap();
        assert_eq!(normalized, key.replace('-', ""));
    }

    #[test]
    fn a_recovery_key_typed_loosely_still_matches() {
        assert_eq!(
            normalize_recovery_key("abcd efgh-ijkl mnop-0o1l"),
            Some("ABCDEFGH1JK1MN0P0011".to_owned())
        );
        assert_eq!(normalize_recovery_key("ABCD-EFGH"), None);
        assert_eq!(normalize_recovery_key("UUUU-UUUU-UUUU-UUUU-UUUU"), None);
    }

    #[test]
    fn session_tokens_are_long_and_different() {
        let a = new_session_token().unwrap();
        let b = new_session_token().unwrap();

        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
    }
}
