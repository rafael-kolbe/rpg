//! Messages exchanged between the game client and the server.
//!
//! This crate is the single definition of the wire format. Every type here
//! derives [`ts_rs::TS`], and `cargo test` writes the matching TypeScript
//! declarations to `web/src/protocol/generated/`, so the client never declares
//! these shapes by hand.
//!
//! Serialization conventions:
//! - fields are `camelCase` on the wire (TypeScript convention);
//! - enum variants used as values are `snake_case`.
//!
//! | Module | Contents |
//! | --- | --- |
//! | [`account`] | `/api/accounts`, `/api/session` |
//! | [`health`] | `GET /api/health` |
//! | [`messages`] | WebSocket messages (`/ws`) |
//! | [`state`] | The battle as the client sees it |

pub mod account;
pub mod health;
pub mod messages;
pub mod state;

pub use account::{
    AccountError, AccountErrorResponse, Credentials, RecoverRequest, RecoveryKeyResponse,
    SessionResponse,
};
pub use health::{HealthResponse, ServiceStatus};
pub use messages::{ClientMessage, DebugAction, ErrorCode, RejectReason, ServerMessage, TimeMode};
pub use state::{
    ActionInfo, BattleState, Catalog, CombatantState, InventoryEntry, ItemInfo, SandboxList,
};
