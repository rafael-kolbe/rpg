//! The `/ws` endpoint: one player's game and battles per connection.
//!
//! The connection task owns its game and battle (no shared state, no
//! locks): it reads [`ClientMessage`]s, steps the engine on a timer (one
//! tick per 100 ms at normal speed), and sends a [`ServerMessage`] after
//! every change. A game ([`ClientMessage::StartGame`]) needs the session
//! cookie; the game's battles run here, and their end goes back to the game,
//! which is then stored. A game battle's record is stored too — at each
//! command, at most every [`BATTLE_STORE_INTERVAL`] while time flows, and
//! when the connection closes — so a reload resumes it where it was.
//! Sandbox battles need no account.
//!
//! Every message is a request to check, never a fact: the client can be
//! the browser console or a script (`docs/decisions/0009-the-client-is-untrusted.md`).
//! The handshake must come from this site's own pages ([`same_origin`]),
//! messages are small and few ([`MessageBudget`]), and only the newest tab
//! of an account plays ([`crate::open_games`]).

use std::time::Duration;

use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;
use engine::{Battle, PartyControl, ReplayError, Side, TICK_MS};
use protocol::{
    BattleState, Catalog, ClientMessage, ErrorCode, RejectReason, ServerMessage, TimeMode,
};
use tokio::time::{Instant, Interval, MissedTickBehavior, interval};
use world::{BattleOrder, Game, GameCommand, GameMode, Save};

use crate::accounts::{self, AccountId};
use crate::http::AppState;
use crate::open_games::OpenGames;
use crate::saves;

/// Largest message a client may send; real ones are under 1 KiB.
const MAX_MESSAGE_BYTES: usize = 16 * 1024;
/// Messages a client may send in a burst.
const BURST_MESSAGES: u64 = 60;
/// Messages per second a client may keep sending (a held key repeats
/// about 30 times a second).
const MESSAGES_PER_SECOND: u64 = 30;

/// How often a game battle's record is stored while only time flows.
const BATTLE_STORE_INTERVAL: Duration = Duration::from_secs(1);

/// Speeds any battle may use, in percent.
const SPEEDS: [u16; 3] = [50, 100, 200];
/// Extra speeds for sandbox testing.
#[cfg(feature = "sandbox")]
const SANDBOX_SPEEDS: [u16; 2] = [400, 800];

/// `GET /ws`: upgrades to a WebSocket and runs the session, logged in when
/// the request carries a valid session cookie.
pub(crate) async fn upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Response {
    let header = |name| headers.get(name).and_then(|value| value.to_str().ok());
    if !same_origin(header(header::ORIGIN), header(header::HOST)) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let account = accounts::account_of(&state, &jar).await;
    ws.max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| run(socket, state, account))
}

/// Whether a handshake comes from a page of this site. The session cookie
/// is `SameSite=Lax`, which still lets a page on a sibling subdomain open a
/// socket with it; the `Origin` a browser sends tells them apart. A client
/// without `Origin` is not a browser, so it holds no one else's cookie.
fn same_origin(origin: Option<&str>, host: Option<&str>) -> bool {
    let Some(origin) = origin else {
        return true;
    };
    let authority = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"));
    match (authority, host) {
        (Some(authority), Some(host)) => authority.eq_ignore_ascii_case(host),
        _ => false,
    }
}

/// Rate limit of one connection's messages: a bucket of
/// [`BURST_MESSAGES`] refilled at [`MESSAGES_PER_SECOND`], counted in
/// thousandths so the refill needs no floats.
#[derive(Debug)]
struct MessageBudget {
    thousandths: u64,
    refilled_at: Instant,
}

impl MessageBudget {
    fn new() -> Self {
        Self {
            thousandths: BURST_MESSAGES * 1000,
            refilled_at: Instant::now(),
        }
    }

    /// Spends one message; `false` when the budget is empty.
    fn spend(&mut self) -> bool {
        let now = Instant::now();
        let elapsed =
            u64::try_from(now.duration_since(self.refilled_at).as_millis()).unwrap_or(u64::MAX);
        self.refilled_at = now;
        self.thousandths = self
            .thousandths
            .saturating_add(elapsed.saturating_mul(MESSAGES_PER_SECOND))
            .min(BURST_MESSAGES * 1000);
        if self.thousandths < 1000 {
            return false;
        }
        self.thousandths -= 1000;
        true
    }
}

/// This connection's claim on its account's game ([`OpenGames`]).
#[derive(Debug)]
struct Lease {
    games: Arc<OpenGames>,
    account: AccountId,
    ticket: u64,
}

impl Lease {
    fn live(&self) -> bool {
        self.games.holds(self.account, self.ticket)
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.games.release(self.account, self.ticket);
    }
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
    /// Ordered by the game: its end goes back to it.
    of_game: bool,
}

impl Session {
    fn new(battle: Battle, settings: &Connection, of_game: bool) -> Self {
        Self {
            battle,
            time_mode: settings.time_mode,
            speed_percent: settings.speed_percent,
            held: false,
            paused: false,
            of_game,
        }
    }

    /// Whether time flows on this tick.
    pub(crate) fn running(&self) -> bool {
        let waiting_for_player = match self.time_mode {
            TimeMode::Active => false,
            TimeMode::Wait => self.held,
            TimeMode::Turn => self.battle.awaiting_command().next().is_some(),
        };
        self.battle.outcome().is_none() && !self.paused && !waiting_for_player
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

/// A connection: the player, the open game, the battle, and the timer
/// driving it.
pub(crate) struct Connection {
    pub(crate) session: Option<Session>,
    ticker: Interval,
    /// The logged-in player, from the session cookie.
    account: Option<AccountId>,
    game: Option<Game>,
    /// The claim on `game`; a newer tab of the account voids it.
    lease: Option<Lease>,
    budget: MessageBudget,
    /// Time settings, kept across battles.
    time_mode: TimeMode,
    speed_percent: u16,
    /// The game changed since it was last stored.
    unsaved: bool,
    /// The game's battle moved on since it was last stored.
    battle_moved: bool,
    stored_at: Instant,
}

impl Connection {
    pub(crate) fn new(account: Option<AccountId>) -> Self {
        Self {
            session: None,
            ticker: ticker(100),
            account,
            game: None,
            lease: None,
            budget: MessageBudget::new(),
            time_mode: TimeMode::Active,
            speed_percent: 100,
            unsaved: false,
            battle_moved: false,
            stored_at: Instant::now(),
        }
    }

    /// Reacts to one client message; returns what to send back.
    pub(crate) async fn handle(
        &mut self,
        state: &AppState,
        message: ClientMessage,
    ) -> Vec<ServerMessage> {
        if !matches!(message, ClientMessage::StartGame { .. })
            && let Some(closed) = self.close_if_opened_elsewhere()
        {
            return vec![closed];
        }
        match message {
            ClientMessage::StartGame { mode } => self.start_game(state, mode).await,
            ClientMessage::Game { command } => self.game_command(command),
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
                    Ok(events) => {
                        self.unsaved |= session.of_game;
                        vec![session.update(events)]
                    }
                    Err(error) => vec![reject(RejectReason::from(&error))],
                }
            }
            ClientMessage::Hold { held } => self.with_session(|session| {
                session.held = held;
                Vec::new()
            }),
            ClientMessage::SetTimeMode { mode } => {
                self.time_mode = mode;
                self.session.as_mut().map_or_else(Vec::new, |session| {
                    session.time_mode = mode;
                    vec![session.update(Vec::new())]
                })
            }
            ClientMessage::SetSpeed { percent } => {
                if !allowed_speed(percent) {
                    return vec![error(ErrorCode::InvalidSpeed)];
                }
                self.ticker = ticker(percent);
                self.speed_percent = percent;
                self.session.as_mut().map_or_else(Vec::new, |session| {
                    session.speed_percent = percent;
                    vec![session.update(Vec::new())]
                })
            }
            ClientMessage::Debug { action } => self.debug(action),
        }
    }

    /// One timer tick: steps the battle if time is flowing. A game battle
    /// that ends goes back to the game.
    pub(crate) fn tick(&mut self) -> Vec<ServerMessage> {
        if let Some(closed) = self.close_if_opened_elsewhere() {
            return vec![closed];
        }
        let Some(session) = self.session.as_mut().filter(|session| session.running()) else {
            return Vec::new();
        };
        let events = session.battle.step();
        self.battle_moved |= session.of_game;
        let mut replies = vec![session.update(events)];
        replies.extend(self.settle());
        replies
    }

    /// Hands a finished game battle back to the game.
    fn settle(&mut self) -> Option<ServerMessage> {
        let session = self.session.as_ref()?;
        let outcome = session.battle.outcome()?;
        let game = self.game.as_mut().filter(|_| session.of_game)?;
        let party: Vec<_> = session
            .battle
            .combatants()
            .iter()
            .filter(|c| c.side() == Side::Party)
            .map(|c| (c.hp(), c.mp()))
            .collect();
        game.finish_battle(outcome, &party, session.battle.inventory());
        let view = game.view();
        self.session = None;
        self.unsaved = true;
        Some(ServerMessage::GameUpdate { view })
    }

    /// Drops the game (unsaved) when a newer tab of the account opened it.
    fn close_if_opened_elsewhere(&mut self) -> Option<ServerMessage> {
        if self.lease.as_ref()?.live() {
            return None;
        }
        self.lease = None;
        self.game = None;
        if self.session.as_ref().is_some_and(|session| session.of_game) {
            self.session = None;
        }
        self.unsaved = false;
        self.battle_moved = false;
        Some(error(ErrorCode::OpenedElsewhere))
    }

    /// The game to store now, if any, with its battle's record; clears the
    /// marks. Never a game a newer tab took over. A battle that only moved in time waits for
    /// [`BATTLE_STORE_INTERVAL`] unless `closing`.
    fn take_unsaved(&mut self, closing: bool) -> Option<(AccountId, &Game)> {
        let battle_due =
            self.battle_moved && (closing || self.stored_at.elapsed() >= BATTLE_STORE_INTERVAL);
        if (!self.unsaved && !battle_due) || !self.lease.as_ref().is_some_and(Lease::live) {
            return None;
        }
        self.unsaved = false;
        self.battle_moved = false;
        self.stored_at = Instant::now();
        let game = self.game.as_mut()?;
        if let Some(session) = self.session.as_ref().filter(|session| session.of_game) {
            game.record_battle(session.battle.record());
        }
        Some((self.account?, self.game.as_ref()?))
    }

    /// Opens the player's game in `mode`, new or saved.
    async fn start_game(&mut self, state: &AppState, mode: GameMode) -> Vec<ServerMessage> {
        let Some(account) = self.account else {
            return vec![error(ErrorCode::NotLoggedIn)];
        };
        // Claimed before loading, so an older tab stops storing at once.
        self.lease = Some(Lease {
            games: Arc::clone(&state.open_games),
            account,
            ticket: state.open_games.claim(account),
        });
        let loaded = async {
            let progress = saves::progress(&state.db, account).await?;
            let save = saves::save(&state.db, account, mode).await?;
            Ok::<_, sqlx::Error>((progress, save))
        };
        let (progress, save) = match loaded.await {
            Ok(loaded) => loaded,
            Err(db_error) => {
                tracing::error!(%db_error, "cannot load the game");
                return vec![error(ErrorCode::Unavailable)];
            }
        };
        if mode == GameMode::Multiplayer && !progress.multiplayer_unlocked() {
            return vec![error(ErrorCode::ModeLocked)];
        }
        let save = save.unwrap_or_else(|| Save::new(&state.world, getrandom::u64().unwrap_or(0)));
        let game = Game::new(Arc::clone(&state.world), mode, progress, save);
        let view = game.view();
        // A battle the page left unfinished starts again where it was.
        let owed = game.battle_order();
        self.game = Some(game);
        self.session = None;
        self.unsaved = true;
        let mut replies = Vec::new();
        if let Some(order) = owed {
            replies.push(self.start_game_battle(order));
        }
        replies.push(ServerMessage::GameUpdate { view });
        // A record that reached the battle's last blow.
        replies.extend(self.settle());
        replies
    }

    fn game_command(&mut self, command: GameCommand) -> Vec<ServerMessage> {
        let Some(game) = self.game.as_mut() else {
            return vec![error(ErrorCode::NoGame)];
        };
        match game.act(command) {
            Err(game_error) => vec![ServerMessage::GameRejected { error: game_error }],
            Ok(order) => {
                self.unsaved = true;
                let view = game.view();
                let mut replies = Vec::new();
                if let Some(order) = order {
                    replies.push(self.start_game_battle(order));
                }
                replies.push(ServerMessage::GameUpdate { view });
                replies
            }
        }
    }

    fn start_game_battle(&mut self, order: BattleOrder) -> ServerMessage {
        let replayed = Battle::replay(
            order.setup.clone(),
            order.seed,
            PartyControl::Player,
            &order.record,
        );
        let battle = match replayed {
            Ok(battle) => battle,
            Err(ReplayError::Setup(setup_error)) => {
                // Content is validated at startup; this is a bug.
                tracing::error!(%setup_error, "the game ordered an invalid battle");
                return error(ErrorCode::Unavailable);
            }
            Err(replay_error) => {
                // The content changed under a saved battle: it starts over.
                tracing::warn!(%replay_error, "cannot resume a saved battle");
                match Battle::new(order.setup, order.seed, PartyControl::Player) {
                    Ok(battle) => battle,
                    Err(setup_error) => {
                        tracing::error!(%setup_error, "the game ordered an invalid battle");
                        return error(ErrorCode::Unavailable);
                    }
                }
            }
        };
        let session = Session::new(battle, self, true);
        let message = ServerMessage::BattleStarted {
            scenario: None,
            seed: u32::try_from(order.seed & u64::from(u32::MAX)).unwrap_or(0),
            catalog: Catalog::of(&session.battle),
            state: session.state(),
        };
        self.session = Some(session);
        message
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

        let session = Session::new(battle, self, false);
        tracing::debug!(%scenario, seed, auto, "sandbox battle started");
        let message = ServerMessage::BattleStarted {
            scenario: Some(scenario),
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

        // A game battle replays from its record, which these do not enter.
        if self.session.as_ref().is_some_and(|session| session.of_game) {
            return vec![error(ErrorCode::SandboxDisabled)];
        }
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

async fn run(mut socket: WebSocket, state: AppState, account: Option<AccountId>) {
    let mut connection = Connection::new(account);
    'connection: loop {
        let replies = tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(_))) if !connection.budget.spend() => {
                    vec![error(ErrorCode::TooManyMessages)]
                }
                Some(Ok(Message::Text(text))) => match serde_json::from_str(&text) {
                    Ok(message) => connection.handle(&state, message).await,
                    Err(parse_error) => {
                        tracing::debug!(%parse_error, "bad client message");
                        vec![error(ErrorCode::BadMessage)]
                    }
                },
                Some(Ok(Message::Close(_)) | Err(_)) | None => break 'connection,
                Some(Ok(_)) => Vec::new(), // binary, ping, pong
            },
            _ = connection.ticker.tick() => connection.tick(),
        };

        store(&state, &mut connection, false).await;

        for reply in replies {
            let Ok(text) = serde_json::to_string(&reply) else {
                continue;
            };
            if socket.send(Message::Text(text.into())).await.is_err() {
                break 'connection;
            }
        }
    }
    // A reload or a dropped connection: the battle resumes from here.
    store(&state, &mut connection, true).await;
}

async fn store(state: &AppState, connection: &mut Connection, closing: bool) {
    if let Some((account, game)) = connection.take_unsaved(closing)
        && let Err(db_error) = saves::store(&state.db, account, game).await
    {
        tracing::error!(%db_error, "cannot store the game");
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

    async fn start(connection: &mut Connection, state: &AppState) -> Vec<ServerMessage> {
        connection
            .handle(
                state,
                ClientMessage::StartSandbox {
                    scenario: "battle-basic".to_owned(),
                    seed: Some(42),
                    auto: false,
                },
            )
            .await
    }

    #[tokio::test]
    async fn starting_a_scenario_sends_the_initial_state() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);

        let replies = start(&mut connection, &state).await;

        assert!(matches!(
            &replies[..],
            [ServerMessage::BattleStarted { seed: 42, .. }]
        ));
    }

    #[tokio::test]
    async fn an_unknown_scenario_is_an_error() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);

        let replies = connection
            .handle(
                &state,
                ClientMessage::StartSandbox {
                    scenario: "nope".to_owned(),
                    seed: None,
                    auto: false,
                },
            )
            .await;

        assert_eq!(replies, vec![error(ErrorCode::UnknownScenario)]);
    }

    #[tokio::test]
    async fn commands_before_the_turn_is_ready_are_rejected() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;

        let replies = connection
            .handle(
                &state,
                ClientMessage::Command {
                    command: Command {
                        actor: CombatantId(0),
                        kind: CommandKind::Attack,
                        target: Some(CombatantId(3)),
                    },
                },
            )
            .await;

        assert_eq!(replies, vec![reject(RejectReason::NotAwaitingCommand)]);
    }

    #[tokio::test]
    async fn a_full_gauge_lets_the_player_attack() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;
        connection
            .handle(
                &state,
                ClientMessage::Debug {
                    action: DebugAction::FillGauge {
                        target: CombatantId(0),
                    },
                },
            )
            .await;
        connection.tick();

        let replies = connection
            .handle(
                &state,
                ClientMessage::Command {
                    command: Command {
                        actor: CombatantId(0),
                        kind: CommandKind::Attack,
                        target: Some(CombatantId(3)),
                    },
                },
            )
            .await;

        let Some(ServerMessage::BattleUpdate { state: battle, .. }) = replies.first() else {
            panic!("expected an update, got {replies:?}");
        };
        assert_eq!(battle.combatants[0].turn, TurnState::Queued);
    }

    #[tokio::test]
    async fn wait_mode_stops_time_while_a_submenu_is_open() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;

        connection
            .handle(
                &state,
                ClientMessage::SetTimeMode {
                    mode: TimeMode::Wait,
                },
            )
            .await;
        connection
            .handle(&state, ClientMessage::Hold { held: true })
            .await;
        assert_eq!(connection.tick(), Vec::new());

        connection
            .handle(&state, ClientMessage::Hold { held: false })
            .await;
        assert_ne!(connection.tick(), Vec::new());
    }

    #[tokio::test]
    async fn turn_mode_stops_time_while_a_hero_waits_for_a_command() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;
        connection
            .handle(
                &state,
                ClientMessage::SetTimeMode {
                    mode: TimeMode::Turn,
                },
            )
            .await;

        let session = connection.session.as_mut().unwrap();
        while session.battle.awaiting_command().next().is_none() {
            session.battle.step();
        }
        assert_eq!(connection.tick(), Vec::new());
    }

    #[tokio::test]
    async fn pause_stops_time_and_step_advances_one_tick() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;

        connection
            .handle(
                &state,
                ClientMessage::Debug {
                    action: DebugAction::Pause,
                },
            )
            .await;
        assert_eq!(connection.tick(), Vec::new());
        connection
            .handle(
                &state,
                ClientMessage::Debug {
                    action: DebugAction::Step,
                },
            )
            .await;

        let tick = connection.session.as_ref().unwrap().battle.tick();
        assert_eq!(tick, 1);
    }

    #[tokio::test]
    async fn only_listed_speeds_are_accepted() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);
        start(&mut connection, &state).await;

        let replies = connection
            .handle(&state, ClientMessage::SetSpeed { percent: 150 })
            .await;

        assert_eq!(replies, vec![error(ErrorCode::InvalidSpeed)]);
    }

    #[tokio::test]
    async fn a_game_needs_a_logged_in_player() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);

        let replies = connection
            .handle(
                &state,
                ClientMessage::StartGame {
                    mode: GameMode::Campaign,
                },
            )
            .await;

        assert_eq!(replies, vec![error(ErrorCode::NotLoggedIn)]);
    }

    #[tokio::test]
    async fn game_commands_need_an_open_game() {
        let state = sandbox_state();
        let mut connection = Connection::new(None);

        let replies = connection
            .handle(
                &state,
                ClientMessage::Game {
                    command: GameCommand::Next,
                },
            )
            .await;

        assert_eq!(replies, vec![error(ErrorCode::NoGame)]);
    }

    #[test]
    fn only_pages_of_this_site_may_open_a_socket() {
        let host = Some("rpg.example.com");

        assert!(same_origin(Some("https://rpg.example.com"), host));
        assert!(same_origin(Some("http://RPG.example.com"), host));
        assert!(
            same_origin(None, host),
            "not a browser: no one else's cookie"
        );
        assert!(!same_origin(Some("https://other.example.com"), host));
        assert!(!same_origin(Some("https://rpg.example.com.evil.net"), host));
        assert!(!same_origin(Some("null"), host));
        assert!(!same_origin(Some("https://rpg.example.com"), None));
    }

    #[test]
    fn a_burst_of_messages_runs_out() {
        let mut budget = MessageBudget::new();
        let sent = (0..BURST_MESSAGES * 2).filter(|_| budget.spend()).count();

        assert!(sent >= usize::try_from(BURST_MESSAGES).unwrap());
        assert!(sent < usize::try_from(BURST_MESSAGES * 2).unwrap());
    }

    /// A connection playing a new Campaign of `account`, without the database.
    fn playing(state: &AppState, account: AccountId) -> Connection {
        let mut connection = Connection::new(Some(account));
        let save = Save::new(&state.world, 1);
        let progress = world::Progress::default();
        connection.game = Some(Game::new(
            Arc::clone(&state.world),
            GameMode::Campaign,
            progress,
            save,
        ));
        connection.lease = Some(Lease {
            games: Arc::clone(&state.open_games),
            account,
            ticket: state.open_games.claim(account),
        });
        connection
    }

    #[tokio::test]
    async fn a_game_opened_in_another_tab_closes_the_older_one_unsaved() {
        let state = sandbox_state();
        let account = AccountId::for_tests(1);
        let mut old = playing(&state, account);
        let mut new = playing(&state, account);
        old.unsaved = true;

        assert_eq!(old.tick(), vec![error(ErrorCode::OpenedElsewhere)]);
        assert!(old.game.is_none());
        assert!(
            old.take_unsaved(true).is_none(),
            "it never overwrites the newer tab"
        );
        let replies = old
            .handle(
                &state,
                ClientMessage::Game {
                    command: GameCommand::Next,
                },
            )
            .await;
        assert_eq!(replies, vec![error(ErrorCode::NoGame)]);

        new.unsaved = true;
        assert!(new.take_unsaved(true).is_some());
    }

    #[tokio::test]
    async fn debug_tools_do_not_touch_a_game_battle() {
        let state = sandbox_state();
        let mut connection = playing(&state, AccountId::for_tests(1));
        let mut replies = Vec::new();
        for _ in 0..200 {
            if connection.session.is_some() {
                break;
            }
            // Back and forth until an encounter.
            for node in ["fields", "square"] {
                replies = connection
                    .handle(
                        &state,
                        ClientMessage::Game {
                            command: GameCommand::Move {
                                node: world::NodeId::new(node),
                            },
                        },
                    )
                    .await;
                if connection.session.is_some() {
                    break;
                }
                // Close the opening scene, if any.
                connection
                    .handle(
                        &state,
                        ClientMessage::Game {
                            command: GameCommand::Next,
                        },
                    )
                    .await;
            }
        }
        assert!(connection.session.is_some(), "no encounter: {replies:?}");

        let replies = connection
            .handle(
                &state,
                ClientMessage::Debug {
                    action: DebugAction::KnockOut {
                        target: CombatantId(1),
                    },
                },
            )
            .await;

        assert_eq!(replies, vec![error(ErrorCode::SandboxDisabled)]);
    }
}
