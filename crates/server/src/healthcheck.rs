//! The `rpg-server healthcheck` command.
//!
//! The production image is distroless (no shell, no curl), so the Docker
//! healthcheck calls the server binary itself. It sends one plain HTTP/1.1
//! request over a TCP socket: pulling in a full HTTP client for a single local
//! request is not worth the extra dependencies.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use anyhow::{Context, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const TIMEOUT: Duration = Duration::from_secs(3);

/// Succeeds when `GET /api/health` on the local `port` answers `200 OK`.
pub(crate) async fn run(port: u16) -> anyhow::Result<()> {
    let status_line = tokio::time::timeout(TIMEOUT, fetch_status_line(port))
        .await
        .context("health check timed out")??;

    if !is_ok_status_line(&status_line) {
        bail!("unhealthy: {status_line}");
    }
    Ok(())
}

async fn fetch_status_line(port: u16) -> anyhow::Result<String> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect(address)
        .await
        .with_context(|| format!("could not connect to {address}"))?;

    stream
        .write_all(b"GET /api/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;

    let response = String::from_utf8_lossy(&response);
    Ok(response.lines().next().unwrap_or_default().to_owned())
}

fn is_ok_status_line(line: &str) -> bool {
    line.split_whitespace().nth(1) == Some("200")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_status_200() {
        assert!(is_ok_status_line("HTTP/1.1 200 OK"));
        assert!(!is_ok_status_line("HTTP/1.1 503 Service Unavailable"));
        assert!(!is_ok_status_line(""));
    }
}
