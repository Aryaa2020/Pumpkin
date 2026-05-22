use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

/// Configuration for the embedded web dashboard.
///
/// Controls whether the dashboard HTTP server is enabled and where it listens.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DashboardConfig {
    /// Whether the web dashboard is enabled.
    pub enabled: bool,
    /// The network address and port where the dashboard HTTP server will listen.
    pub address: SocketAddr,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8080),
        }
    }
}
