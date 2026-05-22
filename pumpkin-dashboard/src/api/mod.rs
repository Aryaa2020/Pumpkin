//! API route definitions and shared state for the dashboard.
//!
//! All REST and WebSocket endpoints are defined under `/api/v1/`.
//! Protected endpoints require a valid Bearer token obtained via login.

use std::collections::HashSet;
use std::sync::Arc;

use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use pumpkin_config::DashboardConfig;
use tokio::sync::RwLock;

use crate::console::ConsoleBroadcast;
use crate::provider::ServerProvider;

/// Console-related API endpoints (history, commands, WebSocket).
pub mod console;
/// Player management API endpoints.
pub mod players;
/// Server status and configuration API endpoints.
pub mod server;

/// Shared application state passed to all API handlers.
#[derive(Clone)]
pub struct AppState {
    /// The server data provider.
    pub server: Arc<dyn ServerProvider>,
    /// Dashboard configuration.
    pub config: DashboardConfig,
    /// Active session tokens.
    ///
    /// TODO: Sessions currently never expire. A future iteration should add
    /// time-based expiry (e.g., storing creation timestamps and periodically
    /// pruning stale sessions) to limit the window of a leaked token.
    pub sessions: Arc<RwLock<HashSet<String>>>,
    /// Console broadcast channel for WebSocket streaming.
    pub console: Arc<ConsoleBroadcast>,
}

/// Build the API router with all endpoints.
///
/// The login endpoint is public; all other endpoints require authentication.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        // Server
        .route("/server/status", get(server::get_status))
        .route("/config", get(server::get_config))
        // Players
        .route("/players", get(players::list_players))
        .route("/players/{uuid}/kick", post(players::kick_player))
        .route("/players/{uuid}/ban", post(players::ban_player))
        // Console
        .route("/console/history", get(console::get_history))
        .route("/console/command", post(console::execute_command))
        // WebSocket
        .route("/ws", get(console::ws_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    Router::new()
        .route("/auth/login", post(login))
        .merge(protected)
        .with_state(state)
}

/// Middleware that validates Bearer token authentication.
///
/// Checks the Authorization header for a valid session token.
/// Also supports `?token=` query parameter for WebSocket connections.
/// Returns 401 Unauthorized if the token is missing or invalid.
async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // Also check query parameter for WebSocket connections
    let query_token = request
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find_map(|pair| pair.strip_prefix("token="))
                .map(String::from)
        });

    let is_valid = if crate::auth::validate_token(&state.sessions, auth_header.as_deref()).await {
        true
    } else if let Some(ref token) = query_token {
        let bearer = format!("Bearer {token}");
        crate::auth::validate_token(&state.sessions, Some(&bearer)).await
    } else {
        false
    };

    if is_valid {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response()
    }
}

/// Request body for the login endpoint.
#[derive(serde::Deserialize)]
struct LoginRequest {
    /// The password to authenticate with.
    password: String,
}

/// Response body for a successful login.
#[derive(serde::Serialize)]
struct LoginResponse {
    /// The session token for subsequent requests.
    token: String,
}

/// Error response body.
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    /// Description of the error.
    pub error: String,
}

/// Handle login requests by validating the password and returning a session token.
async fn login(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::Json(body): axum::Json<LoginRequest>,
) -> Response {
    if body.password != state.config.password {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(ErrorResponse {
                error: "Invalid password".to_string(),
            }),
        )
            .into_response();
    }

    // Check max sessions
    let mut sessions = state.sessions.write().await;
    if sessions.len() >= state.config.max_sessions as usize {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(ErrorResponse {
                error: "Maximum sessions reached".to_string(),
            }),
        )
            .into_response();
    }

    let token = crate::auth::generate_token();
    sessions.insert(token.clone());

    (StatusCode::OK, axum::Json(LoginResponse { token })).into_response()
}
