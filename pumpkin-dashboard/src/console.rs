//! Console broadcast for live log streaming via WebSocket.
//!
//! Provides a tracing layer that captures log events and broadcasts
//! them to connected WebSocket clients.

use std::sync::Arc;
use tokio::sync::broadcast;

/// Broadcast channel for console log lines.
///
/// Connected WebSocket clients subscribe to this channel to receive
/// live log output from the server.
pub struct ConsoleBroadcast {
    /// The broadcast sender for distributing log lines.
    sender: broadcast::Sender<String>,
}

impl ConsoleBroadcast {
    /// Create a new console broadcast channel with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to the console broadcast to receive log lines.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    /// Send a log line to all subscribed clients.
    pub fn send(&self, message: String) {
        // Ignore send errors (no subscribers connected)
        let _ = self.sender.send(message);
    }
}

/// A tracing layer that forwards formatted log events to the console broadcast.
pub struct DashboardTracingLayer {
    /// Reference to the console broadcast channel.
    broadcast: Arc<ConsoleBroadcast>,
}

impl DashboardTracingLayer {
    /// Create a new tracing layer that sends log events to the broadcast.
    #[must_use]
    pub fn new(broadcast: Arc<ConsoleBroadcast>) -> Self {
        Self { broadcast }
    }
}

impl<S> tracing_subscriber::Layer<S> for DashboardTracingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use std::fmt::Write;
        let metadata = event.metadata();
        let mut message = String::new();
        let _ = write!(message, "[{}] {}: ", metadata.level(), metadata.target());

        // Extract the message field from the event
        let mut visitor = MessageVisitor {
            message: &mut message,
        };
        event.record(&mut visitor);

        self.broadcast.send(message);
    }
}

/// Visitor for extracting field values from tracing events.
struct MessageVisitor<'a> {
    /// The string buffer to write field values into.
    message: &'a mut String,
}

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            let _ = write!(self.message, " {}={value:?}", field.name());
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        use std::fmt::Write;
        if field.name() == "message" {
            let _ = write!(self.message, "{value}");
        } else {
            let _ = write!(self.message, " {}={value}", field.name());
        }
    }
}
