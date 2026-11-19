//! Player accounts over HTTP (PLAN.md §5 "Contas", ADR 0007).
//!
//! | Route | What |
//! | --- | --- |
//! | `POST /api/accounts` | Sign up; answers the recovery key and logs in |
//! | `POST /api/accounts/recover` | New password with the recovery key; answers a new key and logs in |
//! | `GET /api/session` | Who is logged in |
//! | `POST /api/session` | Log in |
//! | `DELETE /api/session` | Log out |
//!
//! The session lives in an `httpOnly` cookie holding a random token; the
//! database keeps only its hash.

mod credentials;
mod rate_limit;
mod store;

use std::net::SocketAddr;
use std::time::Instant;

use axum::Json;
use axum::extract::{ConnectInfo, FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use protocol::{
    AccountError, AccountErrorResponse, Credentials, RecoverRequest, RecoveryKeyResponse,
    SessionResponse,
};

pub(crate) use self::rate_limit::Attempts;
pub(crate) use self::store::AccountId;
use self::store::InsertError;
use crate::http::AppState;

/// Name of the session cookie.
const SESSION_COOKIE: &str = "rpg_session";

/// The account routes, to nest under `/api`.
pub(crate) fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/accounts", post(sign_up))
        .route("/accounts/recover", post(recover))
        .route("/session", get(session).post(log_in).delete(log_out))
}

/// The logged-in account of a request, from its session cookie; `None`
/// without a valid session (or when the database is down).
pub(crate) async fn account_of(state: &AppState, jar: &CookieJar) -> Option<AccountId> {
    let cookie = jar.get(SESSION_COOKIE)?;
    match store::session_account(&state.db, &credentials::digest(cookie.value())).await {
        Ok(account) => account,
        Err(error) => {
            tracing::error!(%error, "session lookup failed");
            None
        }
    }
}

/// A failed account request: status code and error body.
#[derive(Debug)]
struct ApiError(StatusCode, AccountError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(AccountErrorResponse { error: self.1 })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "account query failed");
        unavailable()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        tracing::error!(%error, "account request failed");
        unavailable()
    }
}

fn unavailable() -> ApiError {
    ApiError(StatusCode::SERVICE_UNAVAILABLE, AccountError::Unavailable)
}

/// Address of the client: `X-Real-IP` when the proxy in front of the server
/// set it (the server port is reachable only from the proxy and loopback),
/// otherwise the peer address.
struct ClientIp(String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let header = parts
            .headers
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let ip = header.unwrap_or_else(|| {
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map_or_else(|| "unknown".to_owned(), |info| info.0.ip().to_string())
        });
        std::future::ready(Ok(Self(ip)))
    }
}

/// Runs a CPU-heavy password hash off the async threads.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| ApiError::from(anyhow::Error::from(error)))
}

/// Starts a session for `account` and adds its cookie to `jar`.
async fn start_session(
    state: &AppState,
    jar: CookieJar,
    account: AccountId,
) -> Result<CookieJar, ApiError> {
    let token = credentials::new_session_token()?;
    store::insert_session(&state.db, account, &credentials::digest(&token)).await?;
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(i64::from(store::SESSION_DAYS)));
    Ok(jar.add(cookie))
}

/// Checks both rules; the username first, as the form shows it first.
fn validate(username: &str, password: &str) -> Result<(), ApiError> {
    if !credentials::valid_username(username) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            AccountError::InvalidUsername,
        ));
    }
    if !credentials::valid_password(password) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            AccountError::InvalidPassword,
        ));
    }
    Ok(())
}

/// A fresh recovery key and the hash to store for it.
fn new_recovery_key() -> Result<(String, Vec<u8>), ApiError> {
    let key = credentials::new_recovery_key()?;
    let normalized = credentials::normalize_recovery_key(&key)
        .ok_or_else(|| anyhow::anyhow!("a new recovery key does not normalize"))?;
    let hash = credentials::digest(&normalized);
    Ok((key, hash))
}

/// `POST /api/accounts`.
async fn sign_up(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<Credentials>,
) -> Result<(StatusCode, CookieJar, Json<RecoveryKeyResponse>), ApiError> {
    validate(&body.username, &body.password)?;
    let password_hash = blocking(move || credentials::hash_password(&body.password)).await??;
    let (recovery_key, key_hash) = new_recovery_key()?;
    let account = store::insert_account(&state.db, &body.username, &password_hash, &key_hash)
        .await
        .map_err(|error| match error {
            InsertError::Taken => ApiError(StatusCode::CONFLICT, AccountError::UsernameTaken),
            InsertError::Database(error) => ApiError::from(error),
        })?;
    let jar = start_session(&state, jar, account).await?;
    Ok((
        StatusCode::CREATED,
        jar,
        Json(RecoveryKeyResponse { recovery_key }),
    ))
}

/// Refuses the attempt when the username or the address failed too often.
fn check_attempts(state: &AppState, keys: &[String]) -> Result<(), ApiError> {
    if state.attempts.blocked(keys, Instant::now()) {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            AccountError::TooManyAttempts,
        ));
    }
    Ok(())
}

/// `POST /api/session`.
async fn log_in(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    jar: CookieJar,
    Json(body): Json<Credentials>,
) -> Result<(StatusCode, CookieJar), ApiError> {
    let keys = [
        rate_limit::user_key(&body.username),
        rate_limit::ip_key(&ip),
    ];
    check_attempts(&state, &keys)?;
    let row = store::find_login(&state.db, &body.username).await?;
    let (account, hash) = row.map(|row| (row.id, row.password_hash)).unzip();
    let password = body.password;
    let matches =
        blocking(move || credentials::verify_password(&password, hash.as_deref())).await?;
    let Some(account) = account.filter(|_| matches) else {
        state.attempts.fail(&keys, Instant::now());
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            AccountError::WrongCredentials,
        ));
    };
    state.attempts.clear(&keys[0]);
    let jar = start_session(&state, jar, account).await?;
    Ok((StatusCode::NO_CONTENT, jar))
}

/// `POST /api/accounts/recover`.
async fn recover(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    jar: CookieJar,
    Json(body): Json<RecoverRequest>,
) -> Result<(CookieJar, Json<RecoveryKeyResponse>), ApiError> {
    let keys = [
        rate_limit::user_key(&body.username),
        rate_limit::ip_key(&ip),
    ];
    check_attempts(&state, &keys)?;
    if !credentials::valid_password(&body.new_password) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            AccountError::InvalidPassword,
        ));
    }
    let account = match credentials::normalize_recovery_key(&body.recovery_key) {
        Some(key) => {
            store::find_by_recovery_key(&state.db, &body.username, &credentials::digest(&key))
                .await?
        }
        None => None,
    };
    let Some(account) = account else {
        state.attempts.fail(&keys, Instant::now());
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            AccountError::WrongRecoveryKey,
        ));
    };
    let password = body.new_password;
    let password_hash = blocking(move || credentials::hash_password(&password)).await??;
    let (recovery_key, key_hash) = new_recovery_key()?;
    store::reset_credentials(&state.db, account, &password_hash, &key_hash).await?;
    state.attempts.clear(&keys[0]);
    let jar = start_session(&state, jar, account).await?;
    Ok((jar, Json(RecoveryKeyResponse { recovery_key })))
}

/// `GET /api/session`.
async fn session(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<SessionResponse>, ApiError> {
    let not_logged_in = ApiError(StatusCode::UNAUTHORIZED, AccountError::NotLoggedIn);
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Err(not_logged_in);
    };
    let username = store::session_username(&state.db, &credentials::digest(cookie.value()))
        .await?
        .ok_or(not_logged_in)?;
    Ok(Json(SessionResponse { username }))
}

/// `DELETE /api/session`.
async fn log_out(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), ApiError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        store::delete_session(&state.db, &credentials::digest(cookie.value())).await?;
    }
    let jar = jar.remove(Cookie::build(SESSION_COOKIE).path("/"));
    Ok((StatusCode::NO_CONTENT, jar))
}
