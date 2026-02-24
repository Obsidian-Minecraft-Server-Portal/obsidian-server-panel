#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
	#[error("Timer with name '{0}' already exists")]
	TimerAlreadyExists(String),
	#[error("Timer with name '{0}' not found")]
	TimerNotFound(String),
	#[error("Timer with name '{0}' is not running")]
	TimerNotRunning(String),
	#[error("Failed to start timer: {0}")]
	TimerStartError(String),
	#[error("Failed to stop timer: {0}")]
	TimerStopError(String),
	#[error("Broadcast channel error: {0}")]
	BroadcastError(#[from] tokio::sync::broadcast::error::RecvError),
	#[error("Join handle error: {0}")]
	JoinHandleError(String),
}