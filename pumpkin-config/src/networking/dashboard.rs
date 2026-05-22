use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

/// Configuration for the web dashboard.
///
/// Controls whether the dashboard is enabled, connection settings,
/// authentication, and session limits.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DashboardConfig {
    /// Whether the web dashboard is enabled.
    pub enabled: bool,
    /// The network address and port where the dashboard web server will listen.
    pub address: SocketAddr,
    /// The password required to access the dashboard.
    pub password: String,
    /// The maximum number of concurrent dashboard sessions.
    pub max_sessions: u32,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 6969),
            password: String::new(),
            max_sessions: 10,
        }
    }
}
