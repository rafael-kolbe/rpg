//! HTTP routes: the JSON API under `/api`, the battle WebSocket on `/ws`, and
//! the built web client for everything else.
//!
//! The web client is a single-page app, so any non-API path that is not a file
//! (e.g. `/sandbox/battle-basic`) is answered with `index.html` and the client
//! router takes over. Unknown `/api/*` paths are a plain `404`, never the page.

use std::path::Path;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use protocol::{HealthResponse, ServiceStatus};
use sqlx::{Connection, PgPool};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

/// How long `/api/health` waits for the database before reporting it down.
const DATABASE_PING_TIMEOUT: Duration = Duration::from_secs(2);

/// Shared state handed to every request handler.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: PgPool,
    /// Failed login and recovery attempts.
    pub(crate) attempts: std::sync::Arc<crate::accounts::Attempts>,
    /// Each account's live game (one tab at a time).
    pub(crate) open_games: std::sync::Arc<crate::open_games::OpenGames>,
    /// The campaign world and the definitions battles need.
    pub(crate) world: std::sync::Arc<world::World>,
    #[cfg(feature = "sandbox")]
    pub(crate) sandbox: std::sync::Arc<crate::sandbox::Sandbox>,
}

/// Builds the full HTTP router.
pub(crate) fn router(state: AppState, web_dir: &Path) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/game", get(game_summary))
        .merge(crate::accounts::routes());
    #[cfg(feature = "sandbox")]
    let api = api.route("/sandbox", get(sandbox_list));
    let api = api.fallback(|| async { StatusCode::NOT_FOUND });

    let web = ServeDir::new(web_dir).fallback(ServeFile::new(web_dir.join("index.html")));

    Router::new()
        .route("/ws", get(crate::ws::upgrade))
        .nest("/api", api)
        .fallback_service(web)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// `GET /api/health`: `200` when the server and its database are ok, `503`
/// otherwise. The body always says which part is down.
async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let database = if ping_database(&state.db).await {
        ServiceStatus::Ok
    } else {
        ServiceStatus::Unavailable
    };

    let code = match database {
        ServiceStatus::Ok => StatusCode::OK,
        ServiceStatus::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };

    let body = HealthResponse {
        status: database,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        database,
    };

    (code, Json(body))
}

/// `GET /api/game`: the logged-in player's games, for the mode choice.
async fn game_summary(
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::CookieJar,
) -> Result<Json<protocol::GameSummary>, StatusCode> {
    let account = crate::accounts::account_of(&state, &jar)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let progress = crate::saves::progress(&state.db, account, &state.world).await;
    let campaign =
        crate::saves::save(&state.db, account, world::GameMode::Campaign, &state.world).await;
    match (progress, campaign) {
        (Ok(progress), Ok(campaign)) => Ok(Json(protocol::GameSummary {
            campaign_started: campaign.is_some(),
            multiplayer_unlocked: progress.multiplayer_unlocked(),
        })),
        (Err(db_error), _) | (_, Err(db_error)) => {
            tracing::error!(%db_error, "cannot read the game summary");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// `GET /api/sandbox`: names of the sandbox scenarios.
#[cfg(feature = "sandbox")]
async fn sandbox_list(State(state): State<AppState>) -> Json<protocol::SandboxList> {
    Json(protocol::SandboxList {
        scenarios: state.sandbox.scenarios.keys().cloned().collect(),
    })
}

async fn ping_database(db: &PgPool) -> bool {
    let ping = async {
        let mut connection = db.acquire().await?;
        connection.ping().await
    };

    match tokio::time::timeout(DATABASE_PING_TIMEOUT, ping).await {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            tracing::warn!(%error, "database ping failed");
            false
        }
        Err(_) => {
            tracing::warn!(timeout = ?DATABASE_PING_TIMEOUT, "database ping timed out");
            false
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    /// A pool pointing at a port where nothing listens: every ping fails fast.
    pub(crate) fn unreachable_database() -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(500))
            .connect_lazy("postgres://rpg@127.0.0.1:1/rpg")
            .unwrap()
    }

    async fn get(app: Router, path: &str) -> (StatusCode, String) {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    fn app_with_web_dir() -> (Router, tempfile::TempDir) {
        let web_dir = tempfile::tempdir().unwrap();
        std::fs::write(web_dir.path().join("index.html"), "<html>game</html>").unwrap();
        let app = router(test_state(), web_dir.path());
        (app, web_dir)
    }

    #[cfg(not(feature = "sandbox"))]
    fn test_state() -> AppState {
        AppState {
            db: unreachable_database(),
            attempts: std::sync::Arc::default(),
            open_games: std::sync::Arc::default(),
            world: test_world(),
        }
    }

    /// The repository's world.
    pub(crate) fn test_world() -> std::sync::Arc<world::World> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
        let content = content::Content::load(&dir).unwrap();
        std::sync::Arc::new(content.load_world(&dir).unwrap())
    }

    #[cfg(feature = "sandbox")]
    fn test_state() -> AppState {
        sandbox_state()
    }

    /// State with the repository's real content and sandbox scenarios.
    #[cfg(feature = "sandbox")]
    pub(crate) fn sandbox_state() -> AppState {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let sandbox =
            crate::sandbox::Sandbox::load(&root.join("content"), &root.join("sandbox")).unwrap();
        AppState {
            db: unreachable_database(),
            attempts: std::sync::Arc::default(),
            open_games: std::sync::Arc::default(),
            world: test_world(),
            sandbox: std::sync::Arc::new(sandbox),
        }
    }

    #[tokio::test]
    async fn health_reports_the_database_as_unavailable_with_503() {
        let (app, _web_dir) = app_with_web_dir();

        let (status, body) = get(app, "/api/health").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let body: HealthResponse = serde_json::from_str(&body).unwrap();
        assert_eq!(body.status, ServiceStatus::Unavailable);
        assert_eq!(body.database, ServiceStatus::Unavailable);
    }

    #[tokio::test]
    async fn unknown_api_paths_are_404_instead_of_the_web_client() {
        let (app, _web_dir) = app_with_web_dir();

        let (status, _) = get(app, "/api/does-not-exist").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn client_side_routes_are_answered_with_index_html() {
        let (app, _web_dir) = app_with_web_dir();

        let (status, body) = get(app, "/sandbox/battle-basic").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "<html>game</html>");
    }
}
