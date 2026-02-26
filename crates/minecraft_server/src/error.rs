use thiserror::Error;

#[derive(Error, Debug)]
pub enum McServerError {
    #[error("Server is already running")]
    AlreadyRunning,

    #[error("Server is not running")]
    NotRunning,

    #[error("Server process not found")]
    ProcessNotFound,

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("No server download available for version {0}")]
    NoServerDownload(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),

    #[error("EULA not accepted")]
    EulaNotAccepted,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, McServerError>;
