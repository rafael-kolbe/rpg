//! `rpg-server`: the authoritative game server.
//!
//! Every rule of the game runs here, in both Story Mode and Online Mode; the
//! browser only sends intentions and renders what the server answers.
//!
//! Usage:
//! - `rpg-server` runs the server.
//! - `rpg-server healthcheck` exits with 0 when the local server answers
//!   `GET /api/health` with `200 OK`. The Docker healthcheck uses it because the
//!   production image is distroless: no shell, no curl.

mod accounts;
mod config;
mod healthcheck;
mod http;
#[cfg(feature = "sandbox")]
mod sandbox;
mod ws;

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::{Context, bail};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::{Config, LogFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    match std::env::args().nth(1).as_deref() {
        None => serve(config).await,
        Some("healthcheck") => healthcheck::run(config.port).await,
        Some(other) => bail!("unknown command `{other}` (expected no command or `healthcheck`)"),
    }
}

async fn serve(config: Config) -> anyhow::Result<()> {
    init_tracing(config.log_format);

    // Lazy: the server starts even when the database is down, and
    // `/api/health` reports it instead of the process crash-looping.
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&config.database_url)
        .context("invalid DATABASE_URL")?;

    #[cfg(feature = "sandbox")]
    let sandbox = {
        let sandbox = sandbox::Sandbox::load(&config.content_dir, &config.sandbox_dir)?;
        tracing::warn!(
            scenarios = sandbox.scenarios.len(),
            "SANDBOX ENABLED: never publish this build"
        );
        std::sync::Arc::new(sandbox)
    };
    tokio::spawn(migrate(db.clone()));

    let state = http::AppState {
        db,
        attempts: std::sync::Arc::default(),
        #[cfg(feature = "sandbox")]
        sandbox,
    };
    let app = http::router(state, &config.web_dir);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind {address}"))?;

    tracing::info!(%address, web_dir = %config.web_dir.display(), "listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error")
}

/// Applies `migrations/` once the database answers, retrying while it is
/// down: the server keeps serving meanwhile (health reports the database).
async fn migrate(db: sqlx::PgPool) {
    loop {
        match sqlx::migrate!("../../migrations").run(&db).await {
            Ok(()) => {
                tracing::info!("database migrations applied");
                return;
            }
            Err(error) => {
                tracing::warn!(%error, "database migrations not applied; retrying in 5 s");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

fn init_tracing(format: LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Text => builder.init(),
    }
}

/// Resolves on Ctrl+C or SIGTERM (what `docker stop` sends), so in-flight
/// requests finish before the process exits.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "could not listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };

    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "could not listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
