use thiserror::Error;

/// Errors that can occur during UPnP operations.
#[derive(Debug, Error)]
pub enum UpnpError {
    /// Attempted to remove a port that is not currently mapped.
    #[error("port {0} is not currently mapped")]
    PortNotFound(u16),

    /// Attempted to add a port that is already mapped.
    #[error("port {0} is already mapped")]
    PortAlreadyMapped(u16),

    /// A UPnP network operation failed (add or delete ports).
    #[error("UPnP operation failed: {0}")]
    UpnpOperationFailed(String),

    /// The renewal timer could not be started or stopped.
    #[error("renewal timer error: {0}")]
    RenewalError(String),
}
