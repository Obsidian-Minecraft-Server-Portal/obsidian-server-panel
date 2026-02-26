use crate::models::ServerStatus;

/// Events emitted during server lifecycle operations.
#[derive(Debug, Clone)]
pub enum ServerEvent {
    /// Server status has changed.
    StatusChanged { status: ServerStatus },
    /// A line of console output was received from the server process.
    ConsoleOutput { line: String },
    /// Installation progress update.
    InstallProgress {
        file: String,
        completed: bool,
        total: usize,
        current: usize,
    },
    /// Server has successfully started (detected "Done" in console output).
    Started,
    /// Server has stopped normally.
    Stopped,
    /// Server process crashed with the given exit code.
    Crashed { exit_code: i32 },
    /// Java version mismatch detected in console output.
    JavaVersionError,
}

/// Trait for handling server events. Implement this to receive callbacks
/// during server lifecycle operations (start, stop, crash, console output, etc.).
///
/// The web app implements this to persist status to the database and broadcast
/// updates via WebSocket. The CLI implements this to print status to the terminal.
pub trait ServerEventHandler: Send + Sync + 'static {
    /// Called when a server event occurs. Implementations should handle events
    /// quickly to avoid blocking the server process management.
    fn on_event(
        &self,
        event: ServerEvent,
    ) -> impl std::future::Future<Output = ()> + Send;
}

/// A no-op event handler that discards all events.
pub struct NoOpHandler;

impl ServerEventHandler for NoOpHandler {
    async fn on_event(&self, _event: ServerEvent) {}
}
