//! Account API: `/api/accounts` and `/api/session` (PLAN.md §5 "Contas").
//!
//! Errors come back as [`AccountErrorResponse`] with an [`AccountError`]
//! code; the client translates the code.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Body of `POST /api/accounts` and `POST /api/session`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Credentials {
    /// 8 to 20 ASCII letters and digits.
    pub username: String,
    /// 8 to 64 characters.
    pub password: String,
}

/// Body of `POST /api/accounts/recover`: a forgotten password changed with
/// the recovery key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecoverRequest {
    /// The account.
    pub username: String,
    /// The key shown at sign-up (dashes, spaces and case do not matter).
    pub recovery_key: String,
    /// The new password, same rules as at sign-up.
    pub new_password: String,
}

/// A new recovery key, shown once: answer of sign-up and of recovery (using
/// a key replaces it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecoveryKeyResponse {
    /// Grouped for reading, e.g. `7KQ2-M9XD-…`.
    pub recovery_key: String,
}

/// Body of `GET /api/session`: who is logged in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionResponse {
    /// The username as typed at sign-up.
    pub username: String,
}

/// Body of every failed account request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountErrorResponse {
    /// What went wrong.
    pub error: AccountError,
}

/// Why an account request failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AccountError {
    /// Not 8–20 ASCII letters and digits.
    InvalidUsername,
    /// Not 8–64 characters.
    InvalidPassword,
    /// Another account has this username (case ignored).
    UsernameTaken,
    /// Unknown username or wrong password (never says which).
    WrongCredentials,
    /// Unknown username or wrong recovery key (never says which).
    WrongRecoveryKey,
    /// Too many failed attempts; wait and try again.
    TooManyAttempts,
    /// No valid session.
    NotLoggedIn,
    /// The database is down.
    Unavailable,
}
