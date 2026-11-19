//! Game saves in Postgres (`migrations/*_game_saves.sql`): the account's
//! shared story [`Progress`] and one [`Save`] per [`GameMode`], stored as
//! JSON documents of the `world` crate.

use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::types::Json;
use world::{Game, GameMode, Progress, Save, World};

use crate::accounts::AccountId;

/// The database name of a mode.
fn mode_key(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Campaign => "campaign",
        GameMode::Multiplayer => "multiplayer",
    }
}

/// The account's story progress; empty for a new player, and for one
/// whose progress belongs to another content version.
pub(crate) async fn progress(
    db: &PgPool,
    account: AccountId,
    world: &World,
) -> sqlx::Result<Progress> {
    let row = sqlx::query_scalar!(
        r#"SELECT data AS "data: Json<Value>" FROM progress WHERE account_id = $1"#,
        account.get(),
    )
    .fetch_optional(db)
    .await?;
    Ok(row
        .and_then(|Json(data)| current::<Progress>(data, |p| p.is_current(world)))
        .unwrap_or_else(|| Progress::new(world)))
}

/// The account's save in `mode`, if it played it with the current content.
pub(crate) async fn save(
    db: &PgPool,
    account: AccountId,
    mode: GameMode,
    world: &World,
) -> sqlx::Result<Option<Save>> {
    let row = sqlx::query_scalar!(
        r#"SELECT data AS "data: Json<Value>" FROM saves WHERE account_id = $1 AND mode = $2"#,
        account.get(),
        mode_key(mode),
    )
    .fetch_optional(db)
    .await?;
    Ok(row.and_then(|Json(data)| current::<Save>(data, |s| s.is_current(world))))
}

/// A stored document, unless it belongs to another content version or no
/// longer reads: the player then starts over instead of being locked out.
fn current<T: DeserializeOwned>(data: Value, is_current: impl Fn(&T) -> bool) -> Option<T> {
    match serde_json::from_value::<T>(data) {
        Ok(document) if is_current(&document) => Some(document),
        Ok(_) => {
            tracing::info!("a save of another content version starts over");
            None
        }
        Err(error) => {
            tracing::warn!(%error, "an unreadable save starts over");
            None
        }
    }
}

/// Writes the game's save, and the progress when the mode may change it
/// (only the Campaign), in one transaction.
pub(crate) async fn store(db: &PgPool, account: AccountId, game: &Game) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    if game.mode() == GameMode::Campaign {
        sqlx::query!(
            "INSERT INTO progress (account_id, data) VALUES ($1, $2)
             ON CONFLICT (account_id) DO UPDATE SET data = $2, updated_at = now()",
            account.get(),
            Json(game.progress()) as _,
        )
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query!(
        "INSERT INTO saves (account_id, mode, data) VALUES ($1, $2, $3)
         ON CONFLICT (account_id, mode) DO UPDATE SET data = $3, updated_at = now()",
        account.get(),
        mode_key(game.mode()),
        Json(game.save()) as _,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_of_another_version_or_unreadable_starts_over() {
        let of = |epoch: u32| serde_json::json!({ "epoch": epoch, "flags": [], "explored": [] });
        let is_current = |progress: &Progress| progress.epoch == 2;

        assert!(current::<Progress>(of(2), is_current).is_some());
        assert!(current::<Progress>(of(1), is_current).is_none());
        assert!(current::<Progress>(serde_json::json!({ "crowns": 5 }), is_current).is_none());
    }
}
