use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

/// Configuration for the embedded web dashboard.
///
/// Controls whether the dashboard HTTP server is enabled and where it listens.
/// When `password` is non-empty, all `/api/` endpoints require a Bearer token
/// matching the configured password.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DashboardConfig {
    /// Whether the web dashboard is enabled.
    pub enabled: bool,
    /// The network address and port where the dashboard HTTP server will listen.
    pub address: SocketAddr,
    /// The password required to access API endpoints.
    /// When empty, no authentication is required.
    pub password: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080),
            password: String::new(),
        }
    }
}
