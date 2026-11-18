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

mod config;
mod healthcheck;
mod http;

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

    let app = http::router(http::AppState { db }, &config.web_dir);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind {address}"))?;

    tracing::info!(%address, web_dir = %config.web_dir.display(), "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
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
