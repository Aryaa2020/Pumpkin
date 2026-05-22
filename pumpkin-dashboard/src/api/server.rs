//! Server status and configuration API endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::AppState;

/// GET /api/v1/server/status
///
/// Returns the current server status including player count, TPS,
/// memory usage, uptime, and version information.
pub async fn get_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let status = state.server.get_status().await;
    (StatusCode::OK, axum::Json(status))
}

/// GET /api/v1/config
///
/// Returns a read-only view of the current server configuration.
pub async fn get_config(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = state.server.get_config().await;
    (StatusCode::OK, axum::Json(config))
}
