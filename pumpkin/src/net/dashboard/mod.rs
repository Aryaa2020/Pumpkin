use std::sync::Arc;
use std::time::Instant;

use actix_web::dev::ServiceRequest;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use pumpkin_config::DashboardConfig;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::command::CommandSender;
use crate::server::Server;

#[derive(Embed)]
#[folder = "../dashboard/public/"]
struct DashboardAssets;

/// Shared application state for the dashboard.
struct AppState {
    start_time: Instant,
    /// Persistent sysinfo System instance for accurate CPU measurements.
    /// CPU usage requires two samples with a time gap; persisting the System
    /// instance ensures subsequent calls return meaningful values.
    sys: Mutex<sysinfo::System>,
    /// The configured password for API authentication.
    /// Empty means no authentication is required.
    password: String,
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

// ---------- Authentication ----------

/// Valid gamemode values accepted by the settings endpoint.
const VALID_GAMEMODES: &[&str] = &["Survival", "Creative", "Adventure", "Spectator"];

/// Valid difficulty values accepted by the settings endpoint.
const VALID_DIFFICULTIES: &[&str] = &["Peaceful", "Easy", "Normal", "Hard"];

/// Checks the Bearer token from the Authorization header against the configured password.
/// Returns true if authentication passes (either no password configured or token matches).
fn check_auth(req: &ServiceRequest, password: &str) -> bool {
    if password.is_empty() {
        return true;
    }

    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map_or(false, |header| {
            header
                .strip_prefix("Bearer ")
                .map_or(false, |token| token == password)
        })
}

// ---------- Handlers ----------

/// Returns the current server status including uptime and resource usage.
async fn get_status(
    server: web::Data<Arc<Server>>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let address = server.basic_config.java_edition_address.to_string();

    let (cpu, memory_used, total_memory) = {
        let mut sys = state.sys.lock().await;
        let pid = sysinfo::get_current_pid().unwrap_or(sysinfo::Pid::from(0));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            false,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory(),
        );
        sys.refresh_memory();
        let (cpu_val, mem_used) = match sys.process(pid) {
            Some(process) => (process.cpu_usage(), process.memory()),
            None => (0.0, 0),
        };
        (cpu_val, mem_used, sys.total_memory())
    };

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
    // Validate gamemode if provided
    if let Some(ref gamemode) = body.default_gamemode {
        if !VALID_GAMEMODES.contains(&gamemode.as_str()) {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!(
                    "Invalid gamemode '{gamemode}'. Must be one of: {}",
                    VALID_GAMEMODES.join(", ")
                ),
            });
        }
    }

    // Validate difficulty if provided
    if let Some(ref difficulty) = body.default_difficulty {
        if !VALID_DIFFICULTIES.contains(&difficulty.as_str()) {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!(
                    "Invalid difficulty '{difficulty}'. Must be one of: {}",
                    VALID_DIFFICULTIES.join(", ")
                ),
            });
        }
    }

    let config_path = match std::env::current_dir() {
        Ok(dir) => dir.join("pumpkin.toml"),
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get current directory: {e}"),
            });
        }
    };

    let content = match tokio::fs::read_to_string(&config_path).await {
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

    if let Err(e) = tokio::fs::write(&config_path, &updated_content).await {
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
///
/// Note on runtime compatibility: actix-web 4.x uses actix-rt 2.x under the
/// hood, which is built on top of tokio. This means it can run inside an
/// existing tokio runtime without conflict (no separate System needed).
pub async fn run_dashboard(config: DashboardConfig, server: Arc<Server>) {
    info!("Starting web dashboard on {}", config.address);

    if config.password.is_empty() {
        warn!(
            "Dashboard authentication is disabled (no password configured). \
             Set a password in the [networking.dashboard] config section."
        );
    }

    let server_data = web::Data::new(server);
    let app_state = web::Data::new(AppState {
        start_time: Instant::now(),
        sys: Mutex::new(sysinfo::System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(sysinfo::MemoryRefreshKind::everything()),
        )),
        password: config.password.clone(),
    });

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .wrap(actix_web::middleware::from_fn(auth_middleware))
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

/// Middleware that enforces Bearer token authentication on API routes.
/// When no password is configured (empty string), all requests are allowed through.
async fn auth_middleware(
    req: ServiceRequest,
    next: actix_web::middleware::Next<impl actix_web::body::MessageBody + 'static>,
) -> Result<actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, actix_web::Error> {
    let password = req
        .app_data::<web::Data<AppState>>()
        .map(|state| state.password.clone())
        .unwrap_or_default();

    if !check_auth(&req, &password) {
        let response = HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Unauthorized: invalid or missing Bearer token".to_string(),
        });
        return Ok(req.into_response(response).map_into_right_body());
    }

    next.call(req).await.map(|res| res.map_into_left_body())
}
