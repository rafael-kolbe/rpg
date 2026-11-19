//! `GET /api/game`: what the mode choice after login needs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The logged-in player's games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GameSummary {
    /// A Campaign save exists (Continue instead of New game).
    pub campaign_started: bool,
    /// The story has opened the Multiplayer.
    pub multiplayer_unlocked: bool,
}
