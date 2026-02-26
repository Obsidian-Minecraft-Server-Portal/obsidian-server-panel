use crate::error::McServerError;
use crate::events::ServerEventHandler;
use crate::models::{ServerConfig, ServerInfo, ServerStatus};
use crate::process::ServerProcess;
use crate::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

#[cfg(feature = "logging")]
use log::info;

/// Main orchestrator for managing a single Minecraft server instance.
///
/// Generic over `H: ServerEventHandler` so the web app can provide a handler
/// that persists to the database and broadcasts WebSocket updates, while
/// the CLI can provide a handler that prints to the terminal.
pub struct ServerManager<H: ServerEventHandler> {
    config: ServerConfig,
    handler: Arc<H>,
    process: Option<ServerProcess>,
    status: ServerStatus,
}

impl<H: ServerEventHandler> ServerManager<H> {
    /// Create a new server manager with the given configuration and event handler.
    pub fn new(config: ServerConfig, handler: H) -> Self {
        Self {
            config,
            handler: Arc::new(handler),
            process: None,
            status: ServerStatus::Idle,
        }
    }

    /// Get the current server configuration.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get a mutable reference to the server configuration.
    pub fn config_mut(&mut self) -> &mut ServerConfig {
        &mut self.config
    }

    /// Get the current server status.
    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    /// Get information about the server.
    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            config: self.config.clone(),
            status: self.status.clone(),
            pid: self.process.as_ref().map(|p| p.pid()),
        }
    }

    /// Install the server (download JAR and accept EULA).
    /// Updates `config.server_jar` and `config.java_args` with installation results.
    pub async fn install(&mut self) -> Result<()> {
        #[cfg(feature = "logging")]
        info!(
            "Installing {} server for MC {}",
            self.config.server_type, self.config.minecraft_version
        );

        crate::installer::install_server(&mut self.config, self.handler.as_ref()).await?;

        #[cfg(feature = "logging")]
        info!("Installation complete. Server JAR: {}", self.config.server_jar);

        Ok(())
    }

    /// Start the server process.
    pub async fn start(&mut self) -> Result<()> {
        if self.process.is_some() {
            return Err(McServerError::AlreadyRunning);
        }

        #[cfg(feature = "logging")]
        info!("Starting server '{}'", self.config.name);

        self.status = ServerStatus::Starting;
        let process = ServerProcess::start(&self.config, self.handler.clone()).await?;
        self.status = ServerStatus::Running;
        self.process = Some(process);

        Ok(())
    }

    /// Stop the server gracefully by sending the "stop" command.
    pub async fn stop(&mut self) -> Result<()> {
        let process = self.process.as_ref().ok_or(McServerError::NotRunning)?;

        #[cfg(feature = "logging")]
        info!("Stopping server '{}'", self.config.name);

        self.status = ServerStatus::Stopping;
        process.stop().await?;
        Ok(())
    }

    /// Force kill the server process.
    pub async fn kill(&mut self) -> Result<()> {
        let process = self.process.take().ok_or(McServerError::NotRunning)?;

        #[cfg(feature = "logging")]
        info!("Killing server '{}'", self.config.name);

        process.kill().await?;
        self.status = ServerStatus::Stopped;
        Ok(())
    }

    /// Restart the server (stop + start).
    pub async fn restart(&mut self) -> Result<()> {
        #[cfg(feature = "logging")]
        info!("Restarting server '{}'", self.config.name);

        self.stop().await?;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        self.process = None;
        self.start().await?;
        Ok(())
    }

    /// Send a command to the running server.
    pub async fn send_command(&self, command: &str) -> Result<()> {
        let process = self.process.as_ref().ok_or(McServerError::NotRunning)?;
        process.send_command(command).await
    }

    /// Subscribe to console output from the running server.
    pub async fn subscribe_output(&self) -> Result<mpsc::Receiver<String>> {
        let process = self.process.as_ref().ok_or(McServerError::NotRunning)?;
        process.subscribe_output().await
    }

    /// Check if the server process is still running.
    pub async fn is_running(&self) -> bool {
        match &self.process {
            Some(process) => process.is_running().await,
            None => false,
        }
    }

    /// Clean up after the server process has exited.
    /// Call this when notified of a stop/crash event to update internal state.
    pub fn mark_stopped(&mut self) {
        self.process = None;
        self.status = ServerStatus::Stopped;
    }

    /// Mark the server as crashed and clean up.
    pub fn mark_crashed(&mut self) {
        self.process = None;
        self.status = ServerStatus::Crashed;
    }
}
