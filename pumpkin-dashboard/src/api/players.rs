//! Player management API endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::{AppState, ErrorResponse};

/// GET /api/v1/players
///
/// Returns a list of all currently connected players with their
/// UUID, name, ping, world, and gamemode.
pub async fn list_players(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let players = state.server.get_players().await;
    (StatusCode::OK, axum::Json(players))
}

/// Request body for kick/ban actions.
#[derive(serde::Deserialize)]
pub struct ActionRequest {
    /// Optional reason for the action.
    pub reason: Option<String>,
}

/// POST /api/v1/players/:uuid/kick
///
/// Kick a player from the server by their UUID.
pub async fn kick_player(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
    body: Option<axum::Json<ActionRequest>>,
) -> impl IntoResponse {
    let reason = body
        .and_then(|b| b.0.reason)
        .unwrap_or_else(|| "Kicked by dashboard".to_string());

    match state.server.kick_player(&uuid, &reason).await {
        Ok(()) => (StatusCode::OK, axum::Json(serde_json::json!({"success": true}))).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            axum::Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

/// POST /api/v1/players/:uuid/ban
///
/// Ban a player from the server by their UUID.
pub async fn ban_player(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
    body: Option<axum::Json<ActionRequest>>,
) -> impl IntoResponse {
    let reason = body
        .and_then(|b| b.0.reason)
        .unwrap_or_else(|| "Banned by dashboard".to_string());

    match state.server.ban_player(&uuid, &reason).await {
        Ok(()) => (StatusCode::OK, axum::Json(serde_json::json!({"success": true}))).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            axum::Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}
