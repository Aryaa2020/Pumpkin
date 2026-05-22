//! Console API endpoints and WebSocket handler.

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::AppState;

/// GET /api/v1/console/history
///
/// Returns recent console log lines. Currently returns an empty array
/// as history is only available through the WebSocket stream.
pub async fn get_history(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Console history is streamed via WebSocket; return empty for now
    (StatusCode::OK, axum::Json(serde_json::json!({"lines": []})))
}

/// Request body for executing a command.
#[derive(serde::Deserialize)]
pub struct CommandRequest {
    /// The command string to execute.
    pub command: String,
}

/// POST /api/v1/console/command
///
/// Execute a server command and return the result.
pub async fn execute_command(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<CommandRequest>,
) -> impl IntoResponse {
    match state.server.execute_command(&body.command).await {
        Ok(output) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"success": true, "output": output})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({"success": false, "error": e})),
        )
            .into_response(),
    }
}

/// GET /api/v1/ws
///
/// WebSocket endpoint for live console streaming.
/// Incoming messages are treated as commands to execute.
/// Outgoing messages are live log lines from the server.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

/// Handle an individual WebSocket connection.
async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.console.subscribe();

    loop {
        tokio::select! {
            // Forward log lines to the client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // Handle incoming messages from the client (commands)
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(text) => {
                        let _ = state.server.execute_command(&text).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}
