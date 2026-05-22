//! Embedded web dashboard for the Pumpkin Minecraft server.
//!
//! Provides a browser-based interface for server management including
//! player management, live console streaming, performance monitoring,
//! and configuration viewing.

use std::sync::Arc;

use axum::Router;
use pumpkin_config::DashboardConfig;
use rust_embed::Embed;
use tracing::{error, info};

/// API route handlers for the dashboard REST endpoints.
pub mod api;
/// Authentication middleware and utilities.
pub mod auth;
/// Console broadcast and tracing layer for live log streaming.
pub mod console;
/// Server data provider trait definition.
pub mod provider;

pub use console::ConsoleBroadcast;
pub use console::DashboardTracingLayer;
pub use provider::ServerProvider;

/// Embedded frontend assets compiled into the binary.
#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

/// The main dashboard server that serves the web UI and API endpoints.
pub struct DashboardServer;

impl DashboardServer {
    /// Start the dashboard HTTP server.
    ///
    /// Binds to the configured address and serves both the embedded frontend
    /// assets and the REST/WebSocket API endpoints.
    pub async fn run(
        config: DashboardConfig,
        server: Arc<dyn ServerProvider>,
        console: Arc<ConsoleBroadcast>,
    ) {
        let state = api::AppState {
            server,
            config: config.clone(),
            sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
            console,
        };

        let app = Router::new()
            .nest("/api/v1", api::router(state.clone()))
            .fallback(serve_static);

        let listener = match tokio::net::TcpListener::bind(config.address).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind dashboard server to {}: {e}", config.address);
                return;
            }
        };

        info!("Dashboard server listening on {}", config.address);

        if let Err(e) = axum::serve(listener, app).await {
            error!("Dashboard server error: {e}");
        }
    }
}

/// Serve embedded static files with SPA fallback to index.html.
async fn serve_static(
    uri: axum::http::Uri,
) -> impl axum::response::IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return axum::response::Response::builder()
            .header("content-type", mime.as_ref())
            .body(axum::body::Body::from(content.data.to_vec()))
            .unwrap();
    }

    // SPA fallback: serve index.html for any unknown path
    if let Some(content) = Assets::get("index.html") {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        return axum::response::Response::builder()
            .header("content-type", mime.as_ref())
            .body(axum::body::Body::from(content.data.to_vec()))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(404)
        .body(axum::body::Body::from("Not Found"))
        .unwrap()
}
