use crate::error::SchedulerError;
use std::future::Future;

pub trait Timer {
    /// Starts the timer, the timer will run continuously until stopped
    fn start(&self) -> impl Future<Output = Result<(), SchedulerError>> + Send;
    /// Aborts the timer early
    fn stop(&self) -> impl Future<Output = Result<(), SchedulerError>> + Send;
    /// Resets the timer by stopping and restarting it
    fn reset(&self) -> impl Future<Output = Result<(), SchedulerError>> + Send;
    /// Returns true if the timer is currently running
    fn is_running(&self) -> impl Future<Output = bool> + Send;
}
