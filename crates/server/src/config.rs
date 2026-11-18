//! Runtime configuration, read once from environment variables at startup.
//!
//! | Variable       | Default    | Meaning                                   |
//! | -------------- | ---------- | ----------------------------------------- |
//! | `PORT`         | `3000`     | TCP port for HTTP and WebSocket           |
//! | `DATABASE_URL` | (required) | Postgres connection string                |
//! | `WEB_DIR`      | `web/dist` | Built web client served as static files   |
//! | `LOG_FORMAT`   | `text`     | `text` for terminals, `json` in containers |
//! | `RUST_LOG`     | `info`     | Log filter (`tracing` env-filter syntax)  |
//! | `CONTENT_DIR`  | `content`  | Game data (sandbox builds only)           |
//! | `SANDBOX_DIR`  | `sandbox`  | Sandbox scenarios (sandbox builds only)   |

use std::path::PathBuf;

use anyhow::{Context, bail};

/// Server settings. Built by [`Config::from_env`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) port: u16,
    pub(crate) database_url: String,
    pub(crate) web_dir: PathBuf,
    pub(crate) log_format: LogFormat,
    #[cfg(feature = "sandbox")]
    pub(crate) content_dir: PathBuf,
    #[cfg(feature = "sandbox")]
    pub(crate) sandbox_dir: PathBuf,
}

/// How log lines are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogFormat {
    /// Human-readable, for local development.
    Text,
    /// One JSON object per line, for containers and log collectors.
    Json,
}

impl Config {
    /// Reads the configuration from the process environment.
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the configuration through `lookup`, so tests do not touch the
    /// real process environment.
    fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> anyhow::Result<Self> {
        let port = match lookup("PORT") {
            Some(value) => value.parse().with_context(|| {
                format!("PORT must be a number between 1 and 65535, got `{value}`")
            })?,
            None => 3000,
        };

        let database_url = lookup("DATABASE_URL").context("DATABASE_URL is required")?;

        let web_dir = lookup("WEB_DIR").map_or_else(|| PathBuf::from("web/dist"), PathBuf::from);

        let log_format = match lookup("LOG_FORMAT").as_deref() {
            None | Some("text") => LogFormat::Text,
            Some("json") => LogFormat::Json,
            Some(other) => bail!("LOG_FORMAT must be `text` or `json`, got `{other}`"),
        };

        Ok(Self {
            port,
            database_url,
            web_dir,
            log_format,
            #[cfg(feature = "sandbox")]
            content_dir: lookup("CONTENT_DIR")
                .map_or_else(|| PathBuf::from("content"), PathBuf::from),
            #[cfg(feature = "sandbox")]
            sandbox_dir: lookup("SANDBOX_DIR")
                .map_or_else(|| PathBuf::from("sandbox"), PathBuf::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config_from(pairs: &[(&str, &str)]) -> anyhow::Result<Config> {
        let vars: HashMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        Config::from_lookup(|name| vars.get(name).cloned())
    }

    #[test]
    fn applies_defaults_when_only_the_database_is_set() {
        let config = config_from(&[("DATABASE_URL", "postgres://localhost/rpg")]).unwrap();

        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "postgres://localhost/rpg");
        assert_eq!(config.web_dir, PathBuf::from("web/dist"));
        assert_eq!(config.log_format, LogFormat::Text);
    }

    #[test]
    fn requires_database_url() {
        let error = config_from(&[]).unwrap_err();

        assert!(error.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn rejects_a_port_that_is_not_a_number() {
        let error = config_from(&[("DATABASE_URL", "x"), ("PORT", "eighty")]).unwrap_err();

        assert!(error.to_string().contains("PORT"));
    }

    #[test]
    fn rejects_an_unknown_log_format() {
        let error = config_from(&[("DATABASE_URL", "x"), ("LOG_FORMAT", "xml")]).unwrap_err();

        assert!(error.to_string().contains("LOG_FORMAT"));
    }
}
