//! The `/ws` endpoint: one battle session per connection.
//!
//! The connection task owns its battle (no shared state, no locks): it reads
//! [`ClientMessage`]s, steps the engine on a timer (one tick per 100 ms at
//! normal speed), and sends a [`ServerMessage::BattleUpdate`] after every
//! change. Only sandbox battles exist so far; Story Mode rooms come later.

use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use engine::{Battle, TICK_MS};
use protocol::{BattleState, ClientMessage, ErrorCode, RejectReason, ServerMessage, TimeMode};
use tokio::time::{Interval, MissedTickBehavior, interval};

use crate::http::AppState;

/// Speeds any battle may use, in percent.
const SPEEDS: [u16; 3] = [50, 100, 200];
/// Extra speeds for sandbox testing.
#[cfg(feature = "sandbox")]
const SANDBOX_SPEEDS: [u16; 2] = [400, 800];

/// `GET /ws`: upgrades to a WebSocket and runs the session.
pub(crate) async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| run(socket, state))
}

/// One battle and the player's time settings.
#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) battle: Battle,
    pub(crate) time_mode: TimeMode,
    pub(crate) speed_percent: u16,
    /// Wait mode: the player is in a submenu.
    pub(crate) held: bool,
    /// Sandbox pause.
    pub(crate) paused: bool,
}

impl Session {
    #[cfg(feature = "sandbox")]
    fn new(battle: Battle) -> Self {
        Self {
            battle,
            time_mode: TimeMode::Active,
            speed_percent: 100,
            held: false,
            paused: false,
        }
    }

    /// Whether time flows on this tick.
    pub(crate) fn running(&self) -> bool {
        self.battle.outcome().is_none()
            && !self.paused
            && !(self.time_mode == TimeMode::Wait && self.held)
    }

    pub(crate) fn state(&self) -> BattleState {
        BattleState::of(
            &self.battle,
            self.time_mode,
            self.speed_percent,
            self.paused,
        )
    }

    fn update(&self, events: Vec<engine::BattleEvent>) -> ServerMessage {
        ServerMessage::BattleUpdate {
            events,
            state: self.state(),
        }
    }
}

/// A connection: its battle, if any, and the timer driving it.
pub(crate) struct Connection {
    pub(crate) session: Option<Session>,
    ticker: Interval,
}

impl Connection {
    pub(crate) fn new() -> Self {
        Self {
            session: None,
            ticker: ticker(100),
        }
    }

    /// Reacts to one client message; returns what to send back.
    pub(crate) fn handle(
        &mut self,
        state: &AppState,
        message: ClientMessage,
    ) -> Vec<ServerMessage> {
        match message {
            ClientMessage::StartSandbox {
                scenario,
                seed,
                auto,
            } => self.start_sandbox(state, scenario, seed, auto),
            ClientMessage::Command { command } => {
                let Some(session) = self.session.as_mut() else {
                    return vec![reject(RejectReason::NoBattle)];
                };
                match session.battle.submit(&command) {
                    Ok(events) => vec![session.update(events)],
                    Err(error) => vec![reject(RejectReason::from(&error))],
                }
            }
            ClientMessage::Hold { held } => self.with_session(|session| {
                session.held = held;
                Vec::new()
            }),
            ClientMessage::SetTimeMode { mode } => self.with_session(|session| {
                session.time_mode = mode;
                vec![session.update(Vec::new())]
            }),
            ClientMessage::SetSpeed { percent } => {
                if !allowed_speed(percent) {
                    return vec![error(ErrorCode::InvalidSpeed)];
                }
                self.ticker = ticker(percent);
                self.with_session(|session| {
                    session.speed_percent = percent;
                    vec![session.update(Vec::new())]
                })
            }
            ClientMessage::Debug { action } => self.debug(action),
        }
    }

    /// One timer tick: steps the battle if time is flowing.
    pub(crate) fn tick(&mut self) -> Option<ServerMessage> {
        let session = self.session.as_mut().filter(|session| session.running())?;
        let events = session.battle.step();
        Some(session.update(events))
    }

    fn with_session(
        &mut self,
        f: impl FnOnce(&mut Session) -> Vec<ServerMessage>,
    ) -> Vec<ServerMessage> {
        match self.session.as_mut() {
            Some(session) => f(session),
            None => vec![reject(RejectReason::NoBattle)],
        }
    }

    #[cfg(feature = "sandbox")]
    fn start_sandbox(
        &mut self,
        state: &AppState,
        scenario: String,
        seed: Option<u32>,
        auto: bool,
    ) -> Vec<ServerMessage> {
        use engine::PartyControl;
        use protocol::Catalog;

        let Some(encounter) = state.sandbox.scenarios.get(&scenario) else {
            return vec![error(ErrorCode::UnknownScenario)];
        };
        let seed = seed.unwrap_or_else(random_seed);
        let control = if auto {
            PartyControl::Ai
        } else {
            PartyControl::Player
        };
        let setup = state.sandbox.content.battle_setup(encounter.clone());
        let battle = match Battle::new(setup, u64::from(seed), control) {
            Ok(battle) => battle,
            Err(setup_error) => {
                // Auto-battle on a scenario whose party has no AI.
                tracing::warn!(%setup_error, %scenario, "cannot start sandbox battle");
                return vec![error(ErrorCode::UnknownScenario)];
            }
        };

        // Keep the player's time settings across restarts.
        let mut session = Session::new(battle);
        if let Some(previous) = &self.session {
            session.time_mode = previous.time_mode;
            session.speed_percent = previous.speed_percent;
        }
        tracing::debug!(%scenario, seed, auto, "sandbox battle started");
        let message = ServerMessage::BattleStarted {
            scenario,
            seed,
            catalog: Catalog::of(&session.battle),
            state: session.state(),
        };
        self.session = Some(session);
        vec![message]
    }

    #[cfg(not(feature = "sandbox"))]
    #[allow(clippy::unused_self)] // same signature as the sandbox build
    fn start_sandbox(
        &mut self,
        _state: &AppState,
        _scenario: String,
        _seed: Option<u32>,
        _auto: bool,
    ) -> Vec<ServerMessage> {
        vec![error(ErrorCode::SandboxDisabled)]
    }

    #[cfg(feature = "sandbox")]
    fn debug(&mut self, action: protocol::DebugAction) -> Vec<ServerMessage> {
        use protocol::DebugAction;

        /// Duration of statuses applied from the debug panel: 30 s.
        const DEBUG_STATUS_TICKS: u16 = 300;

        self.with_session(|session| {
            let events =
                match action {
                    DebugAction::Pause => {
                        session.paused = true;
                        Vec::new()
                    }
                    DebugAction::Resume => {
                        session.paused = false;
                        Vec::new()
                    }
                    DebugAction::Step if session.paused => session.battle.step(),
                    DebugAction::Step => Vec::new(),
                    DebugAction::KnockOut { target } => session.battle.debug_knock_out(target),
                    DebugAction::ApplyStatus { target, status } => session
                        .battle
                        .debug_apply_status(target, status, DEBUG_STATUS_TICKS),
                    DebugAction::FillGauge { target } => {
                        session.battle.debug_fill_gauge(target);
                        Vec::new()
                    }
                };
            vec![session.update(events)]
        })
    }

    #[cfg(not(feature = "sandbox"))]
    #[allow(clippy::unused_self)] // same signature as the sandbox build
    fn debug(&mut self, _action: protocol::DebugAction) -> Vec<ServerMessage> {
        vec![error(ErrorCode::SandboxDisabled)]
    }
}

async fn run(mut socket: WebSocket, state: AppState) {
    let mut connection = Connection::new();
    loop {
        let replies = tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => match serde_json::from_str(&text) {
                    Ok(message) => connection.handle(&state, message),
                    Err(parse_error) => {
                        tracing::debug!(%parse_error, "bad client message");
                        vec![error(ErrorCode::BadMessage)]
                    }
                },
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => Vec::new(), // binary, ping, pong
            },
            _ = connection.ticker.tick() => connection.tick().into_iter().collect(),
        };

        for reply in replies {
            let Ok(text) = serde_json::to_string(&reply) else {
                continue;
            };
            if socket.send(Message::Text(text.into())).await.is_err() {
                return;
            }
        }
    }
}

/// A timer firing once per tick at the given speed.
fn ticker(speed_percent: u16) -> Interval {
    let period = Duration::from_millis(u64::from(TICK_MS) * 100 / u64::from(speed_percent.max(1)));
    let mut ticker = interval(period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker
}

fn allowed_speed(percent: u16) -> bool {
    #[cfg(feature = "sandbox")]
    if SANDBOX_SPEEDS.contains(&percent) {
        return true;
    }
    SPEEDS.contains(&percent)
}

#[cfg(feature = "sandbox")]
fn random_seed() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos())
}

fn reject(reason: RejectReason) -> ServerMessage {
    ServerMessage::CommandRejected { reason }
}

fn error(code: ErrorCode) -> ServerMessage {
    ServerMessage::Error { code }
}

#[cfg(all(test, feature = "sandbox"))]
mod tests {
    use engine::{CombatantId, Command, CommandKind, TurnState};
    use protocol::DebugAction;

    use super::*;
    use crate::http::tests::sandbox_state;

    fn start(connection: &mut Connection, state: &AppState) -> Vec<ServerMessage> {
        connection.handle(
            state,
            ClientMessage::StartSandbox {
                scenario: "battle-basic".to_owned(),
                seed: Some(42),
                auto: false,
            },
        )
    }

    #[tokio::test]
    async fn starting_a_scenario_sends_the_initial_state() {
        let state = sandbox_state();
        let mut connection = Connection::new();

        let replies = start(&mut connection, &state);

        assert!(matches!(
            &replies[..],
            [ServerMessage::BattleStarted { seed: 42, .. }]
        ));
    }

    #[tokio::test]
    async fn an_unknown_scenario_is_an_error() {
        let state = sandbox_state();
        let mut connection = Connection::new();

        let replies = connection.handle(
            &state,
            ClientMessage::StartSandbox {
                scenario: "nope".to_owned(),
                seed: None,
                auto: false,
            },
        );

        assert_eq!(replies, vec![error(ErrorCode::UnknownScenario)]);
    }

    #[tokio::test]
    async fn commands_before_the_turn_is_ready_are_rejected() {
        let state = sandbox_state();
        let mut connection = Connection::new();
        start(&mut connection, &state);

        let replies = connection.handle(
            &state,
            ClientMessage::Command {
                command: Command {
                    actor: CombatantId(0),
                    kind: CommandKind::Attack,
                    target: Some(CombatantId(3)),
                },
            },
        );

        assert_eq!(replies, vec![reject(RejectReason::NotAwaitingCommand)]);
    }

    #[tokio::test]
    async fn a_full_gauge_lets_the_player_attack() {
        let state = sandbox_state();
        let mut connection = Connection::new();
        start(&mut connection, &state);
        connection.handle(
            &state,
            ClientMessage::Debug {
                action: DebugAction::FillGauge {
                    target: CombatantId(0),
                },
            },
        );
        connection.tick();

        let replies = connection.handle(
            &state,
            ClientMessage::Command {
                command: Command {
                    actor: CombatantId(0),
                    kind: CommandKind::Attack,
                    target: Some(CombatantId(3)),
                },
            },
        );

        let Some(ServerMessage::BattleUpdate { state: battle, .. }) = replies.first() else {
            panic!("expected an update, got {replies:?}");
        };
        assert_eq!(battle.combatants[0].turn, TurnState::Queued);
    }

    #[tokio::test]
    async fn wait_mode_stops_time_while_a_submenu_is_open() {
        let state = sandbox_state();
        let mut connection = Connection::new();
        start(&mut connection, &state);

        connection.handle(
            &state,
            ClientMessage::SetTimeMode {
                mode: TimeMode::Wait,
            },
        );
        connection.handle(&state, ClientMessage::Hold { held: true });
        assert!(connection.tick().is_none());

        connection.handle(&state, ClientMessage::Hold { held: false });
        assert!(connection.tick().is_some());
    }

    #[tokio::test]
    async fn pause_stops_time_and_step_advances_one_tick() {
        let state = sandbox_state();
        let mut connection = Connection::new();
        start(&mut connection, &state);

        connection.handle(
            &state,
            ClientMessage::Debug {
                action: DebugAction::Pause,
            },
        );
        assert!(connection.tick().is_none());
        connection.handle(
            &state,
            ClientMessage::Debug {
                action: DebugAction::Step,
            },
        );

        let tick = connection.session.as_ref().unwrap().battle.tick();
        assert_eq!(tick, 1);
    }

    #[tokio::test]
    async fn only_listed_speeds_are_accepted() {
        let state = sandbox_state();
        let mut connection = Connection::new();
        start(&mut connection, &state);

        let replies = connection.handle(&state, ClientMessage::SetSpeed { percent: 150 });

        assert_eq!(replies, vec![error(ErrorCode::InvalidSpeed)]);
    }
}
