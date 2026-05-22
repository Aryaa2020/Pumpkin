//! Dashboard server provider implementation.
//!
//! Bridges the `pumpkin_dashboard::ServerProvider` trait with the
//! actual `Server` struct to provide live server data to the dashboard.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use pumpkin_dashboard::provider::{PlayerInfo, ServerProvider, ServerStatus};

use crate::server::Server;

/// Adapter that implements `ServerProvider` for the Pumpkin server.
pub struct DashboardServerProvider {
    /// Reference to the main server instance.
    server: Arc<Server>,
    /// The time the server was started, used to compute uptime.
    start_time: Instant,
}

impl DashboardServerProvider {
    /// Create a new dashboard provider wrapping the given server.
    #[must_use]
    pub fn new(server: Arc<Server>) -> Self {
        Self {
            server,
            start_time: Instant::now(),
        }
    }
}

impl ServerProvider for DashboardServerProvider {
    fn get_status(&self) -> Pin<Box<dyn std::future::Future<Output = ServerStatus> + Send + '_>> {
        Box::pin(async {
            let player_count = self.server.get_player_count() as u32;
            let max_players = self.server.basic_config.max_players;

            // Compute average tick time from stored tick times
            let avg_tick_ms = {
                let tick_times = self.server.tick_times_nanos.lock().await;
                let sum: i64 = tick_times.iter().sum();
                let count = tick_times.len() as f64;
                if count > 0.0 {
                    (sum as f64 / count) / 1_000_000.0
                } else {
                    0.0
                }
            };

            // Compute actual TPS from tick timing; cap at configured max
            let tps = if avg_tick_ms > 0.0 {
                (1000.0 / avg_tick_ms).min(f64::from(self.server.basic_config.tps))
            } else {
                f64::from(self.server.basic_config.tps)
            };

            // Memory usage from process-specific RSS
            let memory_usage_mb = {
                use sysinfo::Pid;
                let pid = Pid::from_u32(std::process::id());
                let mut sys = sysinfo::System::new();
                sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                sys.process(pid)
                    .map(|p| p.memory() / (1024 * 1024))
                    .unwrap_or(0)
            };

            let uptime_secs = self.start_time.elapsed().as_secs();
            let version = format!(
                "Pumpkin {}",
                env!("CARGO_PKG_VERSION")
            );
            let motd = self.server.basic_config.motd.clone();

            ServerStatus {
                online: true,
                player_count,
                max_players,
                tps,
                avg_tick_ms,
                memory_usage_mb,
                uptime_secs,
                version,
                motd,
            }
        })
    }

    fn get_players(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Vec<PlayerInfo>> + Send + '_>> {
        Box::pin(async {
            let players = self.server.get_all_players();
            players
                .iter()
                .map(|p| {
                    let world_name = p
                        .living_entity
                        .entity
                        .world
                        .load()
                        .dimension
                        .minecraft_name
                        .to_string();

                    PlayerInfo {
                        uuid: p.gameprofile.id.to_string(),
                        name: p.gameprofile.name.clone(),
                        ping: i64::from(p.ping.load(Ordering::Relaxed)),
                        world: world_name,
                        gamemode: format!("{:?}", p.gamemode.load()),
                    }
                })
                .collect()
        })
    }

    fn kick_player(
        &self,
        uuid: &str,
        reason: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let uuid = uuid.to_string();
        let reason = reason.to_string();
        Box::pin(async move {
            let player_uuid = uuid
                .parse::<uuid::Uuid>()
                .map_err(|e| format!("Invalid UUID: {e}"))?;
            let player = self
                .server
                .get_player_by_uuid(player_uuid)
                .ok_or_else(|| "Player not found".to_string())?;
            player
                .kick(
                    crate::net::DisconnectReason::Kicked,
                    pumpkin_util::text::TextComponent::text(&reason),
                )
                .await;
            Ok(())
        })
    }

    fn ban_player(
        &self,
        uuid: &str,
        reason: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let uuid = uuid.to_string();
        let reason = reason.to_string();
        Box::pin(async move {
            let player_uuid = uuid
                .parse::<uuid::Uuid>()
                .map_err(|e| format!("Invalid UUID: {e}"))?;
            let player = self
                .server
                .get_player_by_uuid(player_uuid)
                .ok_or_else(|| "Player not found".to_string())?;
            // Kick the player with the ban reason
            player
                .kick(
                    crate::net::DisconnectReason::Kicked,
                    pumpkin_util::text::TextComponent::text(&format!("Banned: {reason}")),
                )
                .await;
            tracing::warn!(
                "Dashboard ban: player {} kicked but ban persistence is not yet implemented",
                uuid
            );
            Ok(())
        })
    }

    fn execute_command(
        &self,
        command: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + '_>> {
        let command = command.to_string();
        Box::pin(async move {
            let output = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
            let output_clone = output.clone();

            let source = crate::command::CommandSender::Rcon(output_clone)
                .into_source(&self.server)
                .await;

            self.server
                .command_dispatcher
                .read()
                .await
                .handle_command(&source, &command)
                .await;

            let result = output.lock().await;
            Ok(result.join("\n"))
        })
    }

    fn get_config(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = serde_json::Value> + Send + '_>> {
        Box::pin(async {
            // Return a safe subset of the configuration
            serde_json::json!({
                "max_players": self.server.basic_config.max_players,
                "view_distance": self.server.basic_config.view_distance,
                "simulation_distance": self.server.basic_config.simulation_distance,
                "default_difficulty": format!("{:?}", self.server.basic_config.default_difficulty),
                "default_gamemode": format!("{:?}", self.server.basic_config.default_gamemode),
                "hardcore": self.server.basic_config.hardcore,
                "online_mode": self.server.basic_config.online_mode,
                "motd": self.server.basic_config.motd,
                "tps": self.server.basic_config.tps,
                "java_edition": self.server.basic_config.java_edition,
                "bedrock_edition": self.server.basic_config.bedrock_edition,
            })
        })
    }
}
