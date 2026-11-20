//! WebSocket messages on `/ws`. Every frame is one JSON object with a `type`.

use engine::{BattleEvent, CombatantId, Command, CommandError, StatusKind};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use world::view::GameView;
use world::{GameCommand, GameError, GameMode};

use crate::state::{BattleState, Catalog};

/// Client → server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export)]
pub enum ClientMessage {
    /// Starts (or restarts) a sandbox battle. Refused by servers built without
    /// the `sandbox` feature.
    StartSandbox {
        /// Scenario name (file name in `sandbox/` without `.json`).
        scenario: String,
        /// Fixed seed for a reproducible battle; random when absent.
        #[serde(default)]
        seed: Option<u32>,
        /// The engine plays the party too (auto-battle).
        #[serde(default)]
        auto: bool,
        /// Level of the heroes (not guests) in a campaign scenario; the
        /// scenario's own levels when absent.
        #[serde(default)]
        level: Option<u8>,
    },
    /// Opens the logged-in player's game in a mode (a new one the first
    /// time). Needs the session cookie on the WebSocket.
    StartGame {
        /// Campaign or Multiplayer.
        mode: GameMode,
    },
    /// A command outside battle for the open game.
    Game {
        /// The command.
        command: GameCommand,
    },
    /// A command for a party member whose turn is ready.
    Command {
        /// The command.
        command: Command,
    },
    /// Wait mode: the player is inside a submenu (`true`) or left it
    /// (`false`). Time stops while held; ignored in Active mode.
    Hold {
        /// Whether a submenu is open.
        held: bool,
    },
    /// Switches between Active and Wait.
    SetTimeMode {
        /// The new mode.
        mode: TimeMode,
    },
    /// Battle speed in percent: 50, 100 or 200 (sandbox also 400 and 800).
    SetSpeed {
        /// The new speed.
        percent: u16,
    },
    /// Sandbox debug tools. Refused without the `sandbox` feature.
    Debug {
        /// The tool.
        action: DebugAction,
    },
}

/// Server → client. Only serialized: the server never reads these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum ServerMessage {
    /// A battle began; replaces any previous one.
    BattleStarted {
        /// Sandbox scenario name; `None` for a battle of the game.
        scenario: Option<String>,
        /// Seed in use (to replay the same battle).
        seed: u32,
        /// What the menus need about actions and items.
        catalog: Catalog,
        /// Initial state.
        state: BattleState,
    },
    /// Something changed: one tick passed or a command was accepted.
    BattleUpdate {
        /// What happened, in order (empty on quiet ticks).
        events: Vec<BattleEvent>,
        /// State after the events.
        state: BattleState,
    },
    /// The game as the player sees it, after every change.
    GameUpdate {
        /// The view (boxed: by far the largest message).
        view: Box<GameView>,
    },
    /// A game command was refused; nothing changed.
    GameRejected {
        /// Why.
        error: GameError,
    },
    /// A command was refused; nothing changed.
    CommandRejected {
        /// Why.
        reason: RejectReason,
    },
    /// A message could not be handled (unknown scenario, sandbox disabled…).
    Error {
        /// Error code for the client to translate.
        code: ErrorCode,
    },
}

/// How time flows around the player's choices (see `PLAN.md` §3 "Modos de
/// tempo"). Wait and Turn are single-player only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TimeMode {
    /// Time never stops.
    #[default]
    Active,
    /// Time stops while the player is in a submenu.
    Wait,
    /// Time stops whenever a hero waits for the player's command.
    Turn,
}

/// Sandbox debug tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export)]
pub enum DebugAction {
    /// Stops time.
    Pause,
    /// Restarts time.
    Resume,
    /// Advances one tick while paused.
    Step,
    /// Knocks a combatant out.
    KnockOut {
        /// Who.
        target: CombatantId,
    },
    /// Applies a status for 30 s (no accuracy roll).
    ApplyStatus {
        /// Who.
        target: CombatantId,
        /// The status.
        status: StatusKind,
    },
    /// Fills a combatant's ATB gauge.
    FillGauge {
        /// Who.
        target: CombatantId,
    },
}

/// Why a command was refused, for the client to translate
/// (`ui-reject-<reason>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum RejectReason {
    /// No battle is running, or it ended.
    NoBattle,
    /// The combatant is not the player's.
    NotPlayerControlled,
    /// The combatant's turn is not ready.
    NotAwaitingCommand,
    /// The combatant does not know the action.
    UnknownAction,
    /// Silenced.
    Silenced,
    /// Not enough MP.
    NotEnoughMp,
    /// None of that item left.
    OutOfItem,
    /// A target is missing, not expected, or not valid.
    InvalidTarget,
}

impl From<&CommandError> for RejectReason {
    fn from(error: &CommandError) -> Self {
        match error {
            CommandError::BattleOver => Self::NoBattle,
            CommandError::UnknownCombatant(_) | CommandError::NotPlayerControlled(_) => {
                Self::NotPlayerControlled
            }
            CommandError::NotAwaitingCommand(_) => Self::NotAwaitingCommand,
            CommandError::UnknownAction { .. } => Self::UnknownAction,
            CommandError::Silenced(_) => Self::Silenced,
            CommandError::NotEnoughMp { .. } => Self::NotEnoughMp,
            CommandError::OutOfItem(_) => Self::OutOfItem,
            CommandError::MissingTarget
            | CommandError::UnexpectedTarget
            | CommandError::InvalidTarget(_) => Self::InvalidTarget,
        }
    }
}

/// Errors that are not command refusals, for the client to translate
/// (`ui-error-<code>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ErrorCode {
    /// The message was not valid JSON for the protocol.
    BadMessage,
    /// The server was built without the sandbox.
    SandboxDisabled,
    /// No scenario with that name.
    UnknownScenario,
    /// Speed not allowed.
    InvalidSpeed,
    /// The game needs a logged-in player.
    NotLoggedIn,
    /// The story has not unlocked this mode yet.
    ModeLocked,
    /// No game is open on this connection.
    NoGame,
    /// The saves could not be read or written; try again later.
    Unavailable,
    /// The same game was opened in another tab or device; this one closed.
    OpenedElsewhere,
    /// Messages came faster than any player sends them; some were dropped.
    TooManyMessages,
}

#[cfg(test)]
mod tests {
    use engine::{CombatantId, CommandKind};

    use super::*;

    #[test]
    fn client_messages_use_a_type_tag_and_camel_case_fields() {
        let json =
            r#"{"type":"command","command":{"actor":0,"kind":{"type":"attack"},"target":3}}"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert_eq!(
            message,
            ClientMessage::Command {
                command: Command {
                    actor: CombatantId(0),
                    kind: CommandKind::Attack,
                    target: Some(CombatantId(3)),
                }
            }
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let json = r#"{"type":"hold","held":true,"extra":1}"#;

        assert!(serde_json::from_str::<ClientMessage>(json).is_err());
    }
}
