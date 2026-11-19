//! Game saves in Postgres (`migrations/*_game_saves.sql`): the account's
//! shared story [`Progress`] and one [`Save`] per [`GameMode`], stored as
//! JSON documents of the `world` crate.

use sqlx::PgPool;
use sqlx::types::Json;
use world::{Game, GameMode, Progress, Save};

use crate::accounts::AccountId;

/// The database name of a mode.
fn mode_key(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Campaign => "campaign",
        GameMode::Multiplayer => "multiplayer",
    }
}

/// The account's story progress; empty for a new player.
pub(crate) async fn progress(db: &PgPool, account: AccountId) -> sqlx::Result<Progress> {
    let row = sqlx::query_scalar!(
        r#"SELECT data AS "data: Json<Progress>" FROM progress WHERE account_id = $1"#,
        account.get(),
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map_or_else(Progress::default, |Json(progress)| progress))
}

/// The account's save in `mode`, if it ever played it.
pub(crate) async fn save(
    db: &PgPool,
    account: AccountId,
    mode: GameMode,
) -> sqlx::Result<Option<Save>> {
    let row = sqlx::query_scalar!(
        r#"SELECT data AS "data: Json<Save>" FROM saves WHERE account_id = $1 AND mode = $2"#,
        account.get(),
        mode_key(mode),
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|Json(save)| save))
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
