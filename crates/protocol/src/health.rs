//! `GET /api/health`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Body of `GET /api/health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HealthResponse {
    /// Overall status: [`ServiceStatus::Ok`] only when every dependency is ok.
    pub status: ServiceStatus,
    /// Server version, from the `server` crate manifest.
    pub version: String,
    /// Whether the database answered a ping.
    pub database: ServiceStatus,
}

/// Availability of the server or one of its dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ServiceStatus {
    /// Working normally.
    Ok,
    /// Not reachable or not responding.
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_uses_camel_case_fields_and_snake_case_values() {
        let response = HealthResponse {
            status: ServiceStatus::Unavailable,
            version: "1.2.3".to_owned(),
            database: ServiceStatus::Ok,
        };

        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "status": "unavailable",
                "version": "1.2.3",
                "database": "ok",
            })
        );
    }
}
