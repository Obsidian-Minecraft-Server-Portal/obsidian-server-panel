use crate::error::McServerError;
use crate::events::{ServerEvent, ServerEventHandler};
use crate::models::{ServerConfig, ServerStatus};
use crate::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_interactive::AsynchronousInteractiveProcess;

#[cfg(feature = "logging")]
use log::debug;

/// A running Minecraft server process.
pub struct ServerProcess {
    pid: u32,
}

impl ServerProcess {
    /// Start a new Minecraft server process.
    ///
    /// This spawns the Java process, monitors stdout for the "Done" message
    /// to detect successful startup, and emits events via the handler.
    pub async fn start<H: ServerEventHandler>(
        config: &ServerConfig,
        handler: Arc<H>,
    ) -> Result<Self> {
        let directory_path = config.directory.canonicalize()?;

        // Build the process
        let mut process_builder = AsynchronousInteractiveProcess::new(&config.java_executable);

        // Add memory arguments
        process_builder = process_builder
            .with_argument(format!("-Xmx{}G", config.max_memory_gb))
            .with_argument(format!("-Xms{}G", config.min_memory_gb));

        // Add extra Java arguments
        if !config.java_args.trim().is_empty() {
            for arg in config.java_args.split_whitespace() {
                process_builder = process_builder.with_argument(arg);
            }
        }

        // Add -jar and server JAR
        if !config.server_jar.is_empty() {
            process_builder = process_builder
                .with_argument("-jar")
                .with_argument(&config.server_jar);
        }

        // Add extra Minecraft arguments
        if !config.minecraft_args.trim().is_empty() {
            for arg in config.minecraft_args.split_whitespace() {
                process_builder = process_builder.with_argument(arg);
            }
        }

        // Emit starting status
        handler
            .on_event(ServerEvent::StatusChanged {
                status: ServerStatus::Starting,
            })
            .await;

        // Set up exit callback
        let exit_handler = handler.clone();
        let pid = process_builder
            .with_working_directory(&directory_path)
            .process_exit_callback(move |exit_code| {
                let handler = exit_handler.clone();
                tokio::spawn(async move {
                    #[cfg(feature = "logging")]
                    debug!("Server exited with code {}", exit_code);

                    if exit_code != 0 {
                        handler
                            .on_event(ServerEvent::Crashed { exit_code })
                            .await;
                        handler
                            .on_event(ServerEvent::StatusChanged {
                                status: ServerStatus::Crashed,
                            })
                            .await;
                    } else {
                        handler.on_event(ServerEvent::Stopped).await;
                        handler
                            .on_event(ServerEvent::StatusChanged {
                                status: ServerStatus::Stopped,
                            })
                            .await;
                    }
                });
            })
            .start()
            .await
            .map_err(McServerError::Other)?;

        #[cfg(feature = "logging")]
        debug!("Server process started with PID {}", pid);

        // Monitor stdout for startup completion
        let startup_handler = handler.clone();
        let mut process = AsynchronousInteractiveProcess::get_process_by_pid(pid)
            .await
            .ok_or(McServerError::ProcessNotFound)?;

        loop {
            let line = process
                .receive_output()
                .await
                .map_err(McServerError::Other)?;
            if let Some(line) = line {
                // Emit console output
                startup_handler
                    .on_event(ServerEvent::ConsoleOutput { line: line.clone() })
                    .await;

                // Detect successful startup
                if line.contains("Done (") && line.contains(r#")! For help, type "help""#) {
                    startup_handler
                        .on_event(ServerEvent::StatusChanged {
                            status: ServerStatus::Running,
                        })
                        .await;
                    startup_handler.on_event(ServerEvent::Started).await;
                    break;
                }

                // Detect Java version mismatch
                if line.contains("has been compiled by a more recent version of the Java Runtime")
                {
                    startup_handler
                        .on_event(ServerEvent::JavaVersionError)
                        .await;
                    startup_handler
                        .on_event(ServerEvent::StatusChanged {
                            status: ServerStatus::Crashed,
                        })
                        .await;
                    break;
                }
            }
        }

        Ok(Self { pid })
    }

    /// Get the PID of the running server process.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Send a command to the server's stdin.
    pub async fn send_command(&self, command: &str) -> Result<()> {
        let process = AsynchronousInteractiveProcess::get_process_by_pid(self.pid)
            .await
            .ok_or(McServerError::ProcessNotFound)?;
        process
            .send_input(command)
            .await
            .map_err(McServerError::Other)?;
        Ok(())
    }

    /// Send the "stop" command to gracefully shut down the server.
    pub async fn stop(&self) -> Result<()> {
        self.send_command("stop").await
    }

    /// Force kill the server process.
    pub async fn kill(&self) -> Result<()> {
        let process = AsynchronousInteractiveProcess::get_process_by_pid(self.pid)
            .await
            .ok_or(McServerError::ProcessNotFound)?;
        process
            .kill()
            .await
            .map_err(McServerError::Other)?;
        Ok(())
    }

    /// Check if the server process is still running.
    pub async fn is_running(&self) -> bool {
        if let Some(process) = AsynchronousInteractiveProcess::get_process_by_pid(self.pid).await {
            process.is_process_running().await
        } else {
            false
        }
    }

    /// Subscribe to console output. Returns a receiver that yields output lines.
    pub async fn subscribe_output(&self) -> Result<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(256);
        let pid = self.pid;

        tokio::spawn(async move {
            let mut process = match AsynchronousInteractiveProcess::get_process_by_pid(pid).await {
                Some(p) => p,
                None => return,
            };

            loop {
                let timeout = tokio::time::sleep(Duration::from_secs(30));

                tokio::select! {
                    line_result = process.receive_output() => {
                        match line_result {
                            Ok(Some(line)) => {
                                if tx.send(line).await.is_err() {
                                    // Receiver dropped
                                    break;
                                }
                            }
                            Ok(None) => {
                                // Process ended
                                break;
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    _ = timeout => {
                        if tx.is_closed() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
