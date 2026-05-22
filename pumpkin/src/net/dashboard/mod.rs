use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use pumpkin_config::DashboardConfig;
use rust_embed::Embed;
use tracing::{error, info};

use crate::server::Server;

#[derive(Embed)]
#[folder = "../dashboard/public/"]
struct DashboardAssets;

async fn serve_static(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');

    let path = if path.is_empty() { "index.html" } else { path };

    match DashboardAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(content.data.into_owned())
        }
        None => match DashboardAssets::get("index.html") {
            Some(content) => HttpResponse::Ok()
                .content_type("text/html")
                .body(content.data.into_owned()),
            None => HttpResponse::NotFound().body("Not Found"),
        },
    }
}

/// Starts the embedded web dashboard HTTP server.
///
/// Binds to the configured address and serves embedded static files.
/// If binding fails, an error is logged and the function returns without crashing.
pub async fn run_dashboard(config: DashboardConfig, _server: Arc<Server>) {
    info!("Starting web dashboard on {}", config.address);

    let server = HttpServer::new(|| {
        App::new()
            .service(web::scope("/api"))
            .default_service(web::to(serve_static))
    })
    .bind(config.address);

    let server = match server {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to bind dashboard HTTP server to {}: {e}", config.address);
            return;
        }
    };

    if let Err(e) = server.run().await {
        error!("Dashboard HTTP server error: {e}");
    }
}
