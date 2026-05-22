use auth::AuthenticationConfig;
use dashboard::DashboardConfig;
use proxy::ProxyConfig;
use query::QueryConfig;
use rcon::RCONConfig;
use serde::{Deserialize, Serialize};

use crate::{CompressionConfig, LANBroadcastConfig};

pub mod auth;
pub mod compression;
/// Web dashboard configuration.
pub mod dashboard;
pub mod lan_broadcast;
pub mod proxy;
pub mod query;
pub mod rcon;

/// Configuration for server networking features.
///
/// Covers authentication, query, RCON, proxying, packet compression,
/// LAN broadcast behaviour, and the web dashboard.
#[derive(Deserialize, Serialize, Default)]
pub struct NetworkingConfig {
    /// Authentication settings for client connections.
    pub authentication: AuthenticationConfig,
    /// Query protocol settings for server status requests.
    pub query: QueryConfig,
    /// RCON (remote console) configuration.
    pub rcon: RCONConfig,
    /// Proxy-related networking settings.
    pub proxy: ProxyConfig,
    /// Java Edition packet compression settings.
    pub java_compression: CompressionConfig,
    /// Bedrock Edition packet compression settings.
    pub bedrock_compression: CompressionConfig,
    /// LAN broadcast settings.
    pub lan_broadcast: LANBroadcastConfig,
    /// Web dashboard settings.
    pub dashboard: DashboardConfig,
}
