//! Trait defining the data interface between the dashboard and the server.
//!
//! This avoids circular dependencies by allowing the main server crate
//! to implement this trait and pass it to the dashboard.

use serde::Serialize;
use std::future::Future;
use std::pin::Pin;

/// Information about a connected player.
#[derive(Clone, Serialize)]
pub struct PlayerInfo {
    /// The player's unique identifier.
    pub uuid: String,
    /// The player's display name.
    pub name: String,
    /// The player's ping in milliseconds.
    pub ping: i64,
    /// The world the player is currently in.
    pub world: String,
    /// The player's current gamemode.
    pub gamemode: String,
}

/// Server status information for the dashboard.
#[derive(Clone, Serialize)]
pub struct ServerStatus {
    /// Whether the server is currently running.
    pub online: bool,
    /// Current number of connected players.
    pub player_count: u32,
    /// Maximum allowed players.
    pub max_players: u32,
    /// Current ticks per second.
    pub tps: f64,
    /// Average tick time in milliseconds.
    pub avg_tick_ms: f64,
    /// Server memory usage in megabytes.
    pub memory_usage_mb: u64,
    /// Server uptime in seconds.
    pub uptime_secs: u64,
    /// The server version string.
    pub version: String,
    /// The server's message of the day.
    pub motd: String,
}

/// Trait for providing server data to the dashboard without circular dependencies.
///
/// The main server crate implements this trait to expose the data the dashboard needs.
pub trait ServerProvider: Send + Sync + 'static {
    /// Get the current server status.
    fn get_status(&self) -> Pin<Box<dyn Future<Output = ServerStatus> + Send + '_>>;

    /// Get a list of all online players.
    fn get_players(&self) -> Pin<Box<dyn Future<Output = Vec<PlayerInfo>> + Send + '_>>;

    /// Kick a player by UUID with an optional reason.
    fn kick_player(
        &self,
        uuid: &str,
        reason: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// Ban a player by UUID with an optional reason.
    fn ban_player(
        &self,
        uuid: &str,
        reason: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// Execute a server command.
    fn execute_command(
        &self,
        command: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;

    /// Get a JSON representation of the current server configuration.
    fn get_config(&self) -> Pin<Box<dyn Future<Output = serde_json::Value> + Send + '_>>;
}
