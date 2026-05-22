use std::sync::Arc;
use std::time::Instant;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use pumpkin_config::DashboardConfig;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, UpdateKind};
use tracing::{error, info};

use crate::command::CommandSender;
use crate::server::Server;

#[derive(Embed)]
#[folder = "../dashboard/public/"]
struct DashboardAssets;

/// Shared application state for the dashboard.
struct AppState {
    start_time: Instant,
}

// ---------- Response/Request types ----------

#[derive(Serialize)]
struct StatusResponse {
    online: bool,
    uptime: u64,
    address: String,
    resources: ResourceInfo,
}

#[derive(Serialize)]
struct ResourceInfo {
    cpu: f32,
    memory: MemoryInfo,
}

#[derive(Serialize)]
struct MemoryInfo {
    used: u64,
    total: u64,
}

#[derive(Serialize)]
struct PlayersResponse {
    online_count: usize,
    max: u32,
    players: Vec<PlayerInfo>,
}

#[derive(Serialize)]
struct PlayerInfo {
    name: String,
    uuid: String,
}

#[derive(Deserialize)]
struct ConsoleRequest {
    command: String,
}

#[derive(Serialize)]
struct ConsoleResponse {
    command: String,
    response: String,
}

#[derive(Serialize)]
struct SettingsResponse {
    motd: String,
    max_players: u32,
    default_gamemode: String,
    default_difficulty: String,
}

#[derive(Deserialize)]
struct SettingsUpdateRequest {
    motd: Option<String>,
    max_players: Option<u32>,
    default_gamemode: Option<String>,
    default_difficulty: Option<String>,
}

#[derive(Serialize)]
struct PowerResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ---------- Handlers ----------

/// Returns the current server status including uptime and resource usage.
async fn get_status(
    server: web::Data<Arc<Server>>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let address = server.basic_config.java_edition_address.to_string();

    let (cpu, memory_used) = get_process_resources();

    let sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );
    let total_memory = sys.total_memory();

    HttpResponse::Ok().json(StatusResponse {
        online: true,
        uptime,
        address,
        resources: ResourceInfo {
            cpu,
            memory: MemoryInfo {
                used: memory_used,
                total: total_memory,
            },
        },
    })
}

/// Returns the list of currently connected players.
async fn get_players(server: web::Data<Arc<Server>>) -> HttpResponse {
    let all_players = server.get_all_players();
    let online_count = all_players.len();
    let max = server.basic_config.max_players;

    let players: Vec<PlayerInfo> = all_players
        .iter()
        .map(|p| PlayerInfo {
            name: p.gameprofile.name.clone(),
            uuid: p.gameprofile.id.to_string(),
        })
        .collect();

    HttpResponse::Ok().json(PlayersResponse {
        online_count,
        max,
        players,
    })
}

/// Executes a console command and returns the output.
async fn post_console(
    server: web::Data<Arc<Server>>,
    body: web::Json<ConsoleRequest>,
) -> HttpResponse {
    let command = body.command.clone();

    if command.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Command cannot be empty".to_string(),
        });
    }

    let output = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    let output_clone = output.clone();

    let server_ref: &Arc<Server> = &server;
    let command_source = CommandSender::Rcon(output_clone)
        .into_source(server_ref)
        .await;

    server_ref
        .command_dispatcher
        .read()
        .await
        .handle_command(&command_source, &command)
        .await;

    let result = output.lock().await.join("\n");

    HttpResponse::Ok().json(ConsoleResponse {
        command: body.command.clone(),
        response: result,
    })
}

/// Returns the current server settings from the configuration.
async fn get_settings(server: web::Data<Arc<Server>>) -> HttpResponse {
    HttpResponse::Ok().json(SettingsResponse {
        motd: server.basic_config.motd.clone(),
        max_players: server.basic_config.max_players,
        default_gamemode: server.basic_config.default_gamemode.to_str().to_string(),
        default_difficulty: difficulty_display_name(server.basic_config.default_difficulty),
    })
}

/// Updates server settings in pumpkin.toml (requires restart to take effect).
async fn put_settings(body: web::Json<SettingsUpdateRequest>) -> HttpResponse {
    let config_path = match std::env::current_dir() {
        Ok(dir) => dir.join("pumpkin.toml"),
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get current directory: {e}"),
            });
        }
    };

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to read pumpkin.toml: {e}"),
            });
        }
    };

    let mut config: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to parse pumpkin.toml: {e}"),
            });
        }
    };

    if let Some(table) = config.as_table_mut() {
        if let Some(motd) = &body.motd {
            table.insert("motd".to_string(), toml::Value::String(motd.clone()));
        }
        if let Some(max_players) = body.max_players {
            table.insert(
                "max_players".to_string(),
                toml::Value::Integer(i64::from(max_players)),
            );
        }
        if let Some(gamemode) = &body.default_gamemode {
            table.insert(
                "default_gamemode".to_string(),
                toml::Value::String(gamemode.clone()),
            );
        }
        if let Some(difficulty) = &body.default_difficulty {
            table.insert(
                "default_difficulty".to_string(),
                toml::Value::String(difficulty.clone()),
            );
        }
    }

    let updated_content = match toml::to_string(&config) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to serialize config: {e}"),
            });
        }
    };

    if let Err(e) = std::fs::write(&config_path, &updated_content) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to write pumpkin.toml: {e}"),
        });
    }

    // Return the values that were set (or defaults from file)
    let motd = body
        .motd
        .clone()
        .unwrap_or_else(|| extract_string(&config, "motd"));
    let max_players = body
        .max_players
        .unwrap_or_else(|| extract_u32(&config, "max_players"));
    let default_gamemode = body
        .default_gamemode
        .clone()
        .unwrap_or_else(|| extract_string(&config, "default_gamemode"));
    let default_difficulty = body
        .default_difficulty
        .clone()
        .unwrap_or_else(|| extract_string(&config, "default_difficulty"));

    HttpResponse::Ok().json(SettingsResponse {
        motd,
        max_players,
        default_gamemode,
        default_difficulty,
    })
}

/// Initiates a graceful server shutdown.
async fn post_power_stop() -> HttpResponse {
    crate::stop_server();
    HttpResponse::Ok().json(PowerResponse {
        success: true,
        message: "Stop command sent".to_string(),
    })
}

/// Health check endpoint.
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

// ---------- Helpers ----------

/// Returns a capitalized display name for a difficulty level.
fn difficulty_display_name(difficulty: pumpkin_util::difficulty::Difficulty) -> String {
    match difficulty {
        pumpkin_util::difficulty::Difficulty::Peaceful => "Peaceful".to_string(),
        pumpkin_util::difficulty::Difficulty::Easy => "Easy".to_string(),
        pumpkin_util::difficulty::Difficulty::Normal => "Normal".to_string(),
        pumpkin_util::difficulty::Difficulty::Hard => "Hard".to_string(),
    }
}

/// Gets the current process CPU and memory usage via sysinfo.
fn get_process_resources() -> (f32, u64) {
    let pid = sysinfo::get_current_pid().unwrap_or(sysinfo::Pid::from(0));
    let mut sys = sysinfo::System::new();
    sys.refresh_process_specifics(
        pid,
        ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory(UpdateKind::Always),
    );

    match sys.process(pid) {
        Some(process) => (process.cpu_usage(), process.memory()),
        None => (0.0, 0),
    }
}

/// Extracts a string value from a TOML table by key.
fn extract_string(value: &toml::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Extracts a u32 value from a TOML table by key.
fn extract_u32(value: &toml::Value, key: &str) -> u32 {
    value
        .get(key)
        .and_then(toml::Value::as_integer)
        .map_or(0, |v| v as u32)
}

/// Serves embedded static files for the dashboard frontend.
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
/// Binds to the configured address and serves embedded static files
/// along with REST API endpoints for server management.
pub async fn run_dashboard(config: DashboardConfig, server: Arc<Server>) {
    info!("Starting web dashboard on {}", config.address);

    let server_data = web::Data::new(server);
    let app_state = web::Data::new(AppState {
        start_time: Instant::now(),
    });

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .route("/status", web::get().to(get_status))
                    .route("/players", web::get().to(get_players))
                    .route("/console", web::post().to(post_console))
                    .route("/settings", web::get().to(get_settings))
                    .route("/settings", web::put().to(put_settings))
                    .route("/power/stop", web::post().to(post_power_stop)),
            )
            .default_service(web::to(serve_static))
    })
    .bind(config.address);

    let http_server = match http_server {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Failed to bind dashboard HTTP server to {}: {e}",
                config.address
            );
            return;
        }
    };

    if let Err(e) = http_server.run().await {
        error!("Dashboard HTTP server error: {e}");
    }
}
