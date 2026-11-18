//! Account and session rows (`migrations/*_accounts.sql`).

use sqlx::PgPool;

/// Primary key of an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AccountId(i64);

/// What a login needs from an account.
pub(crate) struct LoginRow {
    pub(crate) id: AccountId,
    pub(crate) password_hash: String,
}

/// How long a session lasts without logging in again; also the cookie
/// lifetime.
pub(crate) const SESSION_DAYS: i32 = 30;

/// Postgres error code of a unique constraint violation.
const UNIQUE_VIOLATION: &str = "23505";

/// Why [`insert_account`] failed.
pub(crate) enum InsertError {
    /// Another account has the username (case ignored).
    Taken,
    /// The database failed.
    Database(sqlx::Error),
}

/// Creates an account.
pub(crate) async fn insert_account(
    db: &PgPool,
    username: &str,
    password_hash: &str,
    recovery_key_hash: &[u8],
) -> Result<AccountId, InsertError> {
    let result = sqlx::query_scalar!(
        "INSERT INTO accounts (username, password_hash, recovery_key_hash)
         VALUES ($1, $2, $3) RETURNING id",
        username,
        password_hash,
        recovery_key_hash,
    )
    .fetch_one(db)
    .await;
    match result {
        Ok(id) => Ok(AccountId(id)),
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some(UNIQUE_VIOLATION) => {
            Err(InsertError::Taken)
        }
        Err(error) => Err(InsertError::Database(error)),
    }
}

/// The account with `username` (case ignored).
pub(crate) async fn find_login(db: &PgPool, username: &str) -> sqlx::Result<Option<LoginRow>> {
    let row = sqlx::query!(
        "SELECT id, password_hash FROM accounts WHERE lower(username) = lower($1)",
        username,
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|row| LoginRow {
        id: AccountId(row.id),
        password_hash: row.password_hash,
    }))
}

/// The account with `username` (case ignored) whose recovery key hashes to
/// `recovery_key_hash`.
pub(crate) async fn find_by_recovery_key(
    db: &PgPool,
    username: &str,
    recovery_key_hash: &[u8],
) -> sqlx::Result<Option<AccountId>> {
    let id = sqlx::query_scalar!(
        "SELECT id FROM accounts WHERE lower(username) = lower($1) AND recovery_key_hash = $2",
        username,
        recovery_key_hash,
    )
    .fetch_optional(db)
    .await?;
    Ok(id.map(AccountId))
}

/// Replaces the password and the recovery key and ends every session of the
/// account, in one transaction.
pub(crate) async fn reset_credentials(
    db: &PgPool,
    account: AccountId,
    password_hash: &str,
    recovery_key_hash: &[u8],
) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "UPDATE accounts SET password_hash = $2, recovery_key_hash = $3 WHERE id = $1",
        account.0,
        password_hash,
        recovery_key_hash,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM sessions WHERE account_id = $1", account.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

/// Stores a new session for `account`. Expired sessions of the account are
/// dropped on the way, so they do not pile up.
pub(crate) async fn insert_session(
    db: &PgPool,
    account: AccountId,
    token_hash: &[u8],
) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "DELETE FROM sessions WHERE account_id = $1 AND expires_at <= now()",
        account.0,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO sessions (token_hash, account_id, expires_at)
         VALUES ($1, $2, now() + make_interval(days => $3))",
        token_hash,
        account.0,
        SESSION_DAYS,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Username of the account of an unexpired session.
pub(crate) async fn session_username(
    db: &PgPool,
    token_hash: &[u8],
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar!(
        "SELECT a.username FROM sessions s JOIN accounts a ON a.id = s.account_id
         WHERE s.token_hash = $1 AND s.expires_at > now()",
        token_hash,
    )
    .fetch_optional(db)
    .await
}

/// Ends a session.
pub(crate) async fn delete_session(db: &PgPool, token_hash: &[u8]) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
        .execute(db)
        .await?;
    Ok(())
}
