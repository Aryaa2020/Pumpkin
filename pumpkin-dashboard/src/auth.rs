//! Authentication utilities for the dashboard API.
//!
//! Implements simple token-based authentication using random session tokens.
//! Protected endpoints validate the Bearer token against the session store.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for the shared session store.
pub type SessionStore = Arc<RwLock<HashSet<String>>>;

/// Generate a new random session token.
///
/// Produces a 64-character hex string derived from 32 random bytes.
pub fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Validate a Bearer token against the session store.
///
/// Returns `true` if the token is present in the active sessions.
pub async fn validate_token(sessions: &SessionStore, auth_header: Option<&str>) -> bool {
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return false,
    };

    let store = sessions.read().await;
    store.contains(token)
}
