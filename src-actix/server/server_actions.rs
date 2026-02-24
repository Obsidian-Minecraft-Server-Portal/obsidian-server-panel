use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use anyhow::Result;
use log::{debug, error, warn};
use obsidian_upnp::{UpnpManager, PortMappingProtocol};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_interactive::AsynchronousInteractiveProcess;

pub(crate) static ACTIVE_SERVERS: OnceLock<Arc<Mutex<HashMap<u64, u32>>>> = OnceLock::new();

impl ServerData {
    pub async fn start_server(&mut self) -> Result<()> {
        if self.has_server_process().await {
            return Err(anyhow::anyhow!("Server is already running"));
        }

        // Check if this is a forge installer that needs to be run first
        if self.is_forge_installer() {
            debug!("Detected forge installer for server {}", self.id);
            self.install_forge_server().await?;
            return Ok(());
        }

        self.status = ServerStatus::Starting;
        self.save().await?;

        // Broadcast server status change
        broadcast::broadcast(BroadcastMessage::ServerUpdate {
            server: self.clone(),
        });

        if self.upnp {
            let properties = self.get_server_properties();
            if let Err(e) = properties {
                self.status = ServerStatus::Crashed;
                self.save().await?;

                // Broadcast server status change
                broadcast::broadcast(BroadcastMessage::ServerUpdate {
                    server: self.clone(),
                });

                return Err(anyhow::anyhow!("Failed to get server properties: {}", e));
            } else if let Ok(properties) = properties {
                let port = properties.server_port.unwrap_or(25565) as u16;
                debug!("Opening port {} for server {}", port, self.id);

                if let Err(e) = UpnpManager::global()
                    .add_port(
                        port,
                        format!("Minecraft Server {}", self.id),
                        PortMappingProtocol::TCP,
                    )
                    .await
                {
                    error!("Failed to open UPnP port {} for server {}: {}", port, self.id, e);
                } else {
                    debug!("Successfully opened UPnP port {} for server {}", port, self.id);
                }
            }
        }
        debug!("Starting server {}", self.id);

        let directory_path = self.get_directory_path().canonicalize()?;
        let self_clone = self.clone();

        // Create the process builder
        let mut process_builder = AsynchronousInteractiveProcess::new(&self.java_executable);

        // Add java arguments
        process_builder = process_builder.with_argument(format!("-Xmx{}G", &self.max_memory)).with_argument(format!("-Xms{}G", &self.min_memory));

        if !self.java_args.trim().is_empty() {
            for arg in self.java_args.split_whitespace() {
                process_builder = process_builder.with_argument(arg);
            }
        }

        if !self.server_jar.is_empty() {
            process_builder = process_builder.with_argument("-jar").with_argument(&self.server_jar);
        }

        // Add minecraft arguments
        if !self.minecraft_args.trim().is_empty() {
            for arg in self.minecraft_args.split_whitespace() {
                process_builder = process_builder.with_argument(arg);
            }
        }

        let pid = process_builder
			.with_working_directory(&directory_path)
			.process_exit_callback(move |exit_code| {
				let mut self_clone = self_clone.clone();
				tokio::spawn(async move {
					debug!("Server exited with code {}", exit_code);
					if exit_code != 0 {
						if let Err(e) = self_clone.remove_server_crashed().await {
							error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
						}

						return;
					}
					if let Err(e) = self_clone.remove_server().await {
						error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
					}
				});
			})
			.start()
			.await?;

        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        servers.lock().await.insert(self.id, pid);
        debug!("Server started with pid {}", pid);
        self.last_started = Some(chrono::Utc::now().timestamp() as u64);
        self.save().await?;

        let hang_duration = Duration::from_secs(120); // 2 minutes

        let id = self.id;
        let owner_id = self.owner_id;
        tokio::spawn(async move {
            tokio::time::sleep(hang_duration).await;
            let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let servers = servers.lock().await;
            if let Some(pid) = servers.get(&id)
                && let Some(process) = AsynchronousInteractiveProcess::get_process_by_pid(*pid).await
                && !process.is_process_running().await {
                    return;
                }
            if let Ok(Some(server)) = ServerData::get(id, owner_id).await
                && server.status == ServerStatus::Starting {}
        });

        let process = match AsynchronousInteractiveProcess::get_process_by_pid(pid).await {
            Some(process) => process,
            None => return Err(anyhow::anyhow!("Server process not found after starting")),
        };
        let mut process = process;

        loop {
            let line = process.receive_output().await?;
            if let Some(line) = line {
                if line.contains("Done (") && line.contains(r#")! For help, type "help""#) {
                    self.status = ServerStatus::Running;
                    self.save().await?;

                    // Broadcast server status change
                    broadcast::broadcast(BroadcastMessage::ServerUpdate {
                        server: self.clone(),
                    });

                    // Send notification that server has started
                    if let Err(e) = self.send_start_notification().await {
                        error!("Failed to send server start notification: {}", e);
                    }

                    break;
                }
                if line.contains("has been compiled by a more recent version of the Java Runtime") {
                    self.status = ServerStatus::Crashed;
                    self.save().await?;

                    // Broadcast server status change
                    broadcast::broadcast(BroadcastMessage::ServerUpdate {
                        server: self.clone(),
                    });

                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<()> {
        self.status = ServerStatus::Stopping;
        self.save().await?;

        // Broadcast server status change
        broadcast::broadcast(BroadcastMessage::ServerUpdate {
            server: self.clone(),
        });

        self.send_command("stop").await?;
        Ok(())
    }

    pub async fn kill_server(&mut self) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let servers = servers.lock().await;
        let pid = servers.get(&self.id).ok_or_else(|| anyhow::anyhow!("Server not running"))?;
        let process = AsynchronousInteractiveProcess::get_process_by_pid(*pid).await.ok_or_else(|| anyhow::anyhow!("Server process not found"))?;
        process.kill().await?;
        self.remove_server().await?;
        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<()> {
        self.stop_server().await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        self.start_server().await?;
        Ok(())
    }

    pub(crate) async fn remove_server(&mut self) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let mut servers = servers.lock().await;
        servers.remove(&self.id);
        self.status = ServerStatus::Stopped;
        self.save().await?;

        // Clean up UPnP port mapping if it was enabled
        if self.upnp {
            if let Ok(properties) = self.get_server_properties() {
                let port = properties.server_port.unwrap_or(25565) as u16;
                if let Err(e) = UpnpManager::global().remove_port(port).await {
                    error!("Failed to remove UPnP port {} for server {}: {}", port, self.id, e);
                }
            }
        }

        // Broadcast server status change
        broadcast::broadcast(BroadcastMessage::ServerUpdate {
            server: self.clone(),
        });

        // Send notification that server has stopped
        if let Err(e) = self.send_stop_notification().await {
            error!("Failed to send server stop notification: {}", e);
        }

        Ok(())
    }

    pub(crate) async fn remove_server_crashed(&mut self) -> Result<()> {
        {
            let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let mut servers = servers.lock().await;
            servers.remove(&self.id);
        }
        self.status = ServerStatus::Crashed;
        self.save().await?;

        // Clean up UPnP port mapping if it was enabled
        if self.upnp {
            if let Ok(properties) = self.get_server_properties() {
                let port = properties.server_port.unwrap_or(25565) as u16;
                if let Err(e) = UpnpManager::global().remove_port(port).await {
                    error!("Failed to remove UPnP port {} for server {}: {}", port, self.id, e);
                }
            }
        }

        // Broadcast server status change
        broadcast::broadcast(BroadcastMessage::ServerUpdate {
            server: self.clone(),
        });

        // Send notification that server has crashed
        if let Err(e) = self.send_crash_notification().await {
            error!("Failed to send server crash notification: {}", e);
        }

        Ok(())
    }

    pub async fn send_command(&self, command: impl Into<String>) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let pid = {
            let servers = servers.lock().await;
            match servers.get(&self.id) {
                Some(pid) => *pid,
                None => return Err(anyhow::anyhow!("Server not running")),
            }
        };
        let process = match AsynchronousInteractiveProcess::get_process_by_pid(pid).await {
            Some(process) => process,
            None => return Err(anyhow::anyhow!("Server process not found")),
        };
        process.send_input(command).await?;

        Ok(())
    }

    pub async fn attach_to_stdout(&self, sender: tokio::sync::mpsc::Sender<actix_web_lab::sse::Event>) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let pid = {
            let servers = servers.lock().await;
            match servers.get(&self.id) {
                Some(pid) => *pid,
                None => return Err(anyhow::anyhow!("Server not running")),
            }
        };
        let mut process = match AsynchronousInteractiveProcess::get_process_by_pid(pid).await {
            Some(process) => process,
            None => return Err(anyhow::anyhow!("Server process not found")),
        };

        loop {
            // Add timeout to detect stale connections
            let output_future = process.receive_output();
            let timeout_future = tokio::time::sleep(Duration::from_secs(30));

            tokio::select! {
                line_result = output_future => {
                    let line = line_result?;
                    if let Some(line) = line {
                        debug!("Sending message to client: {}", line);
                        let message = actix_web_lab::sse::Data::new(line).event("console");

                        // Check if sender is closed first
                        if sender.is_closed() {
                            warn!("Client connection closed, stopping console output forwarding");
                            break;
                        }

                        // Try to send with timeout
                        match tokio::time::timeout(Duration::from_secs(5), sender.send(message.into())).await {
                            Ok(Ok(_)) => {}, // Successfully sent
                            Ok(Err(e)) => {
                                warn!("Failed to send message to client, client may have disconnected: {}", e);
                                break;
                            }
                            Err(_) => {
                                warn!("Timeout sending message to client, assuming disconnected");
                                break;
                            }
                        }
                    }
                }
                _ = timeout_future => {
                    // Periodic check for closed connection
                    if sender.is_closed() {
                        warn!("Client connection closed during timeout check, stopping console output forwarding");
                        break;
                    }
                    // Send a heartbeat to test connection
                    let heartbeat = actix_web_lab::sse::Data::new("").event("heartbeat");
                    if let Err(e) = sender.try_send(heartbeat.into()) {
                        warn!("Heartbeat failed, client likely disconnected: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn has_server_process(&self) -> bool {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let servers = servers.lock().await;
        servers.contains_key(&self.id)
    }

    // Notification helper functions
    async fn send_start_notification(&self) -> Result<()> {
        use crate::notifications::{NotificationActionType, NotificationData, NotificationType};

        let pool = crate::database::get_pool();
        let server_id_hash = serde_hash::hashids::encode_single(self.id);

        let notification = NotificationData::create(
            format!("{} Started", self.name),
            format!("Server \"{}\" has been successfully started.", self.name),
            NotificationType::System,
            NotificationActionType::StopServer.to_bits(),
            Some(server_id_hash.clone()),
            pool,
        )
        .await?;

        // Broadcast to all connected users
        let notification_item = crate::notifications::NotificationItem {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            is_read: false,
            timestamp: notification.timestamp,
            notification_type: notification.notification_type,
            action: notification.action,
            referenced_server: Some(server_id_hash),
        };

        crate::notifications::broadcast_notification(notification_item).await;

        Ok(())
    }

    async fn send_stop_notification(&self) -> Result<()> {
        use crate::notifications::{NotificationActionType, NotificationData, NotificationType};

        let pool = crate::database::get_pool();
        let server_id_hash = serde_hash::hashids::encode_single(self.id);

        let notification = NotificationData::create(
            format!("{} Stopped", self.name),
            format!("Server \"{}\" has been stopped.", self.name),
            NotificationType::System,
            NotificationActionType::StartServer.to_bits(),
            Some(server_id_hash.clone()),
            pool,
        )
        .await?;

        // Broadcast to all connected users
        let notification_item = crate::notifications::NotificationItem {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            is_read: false,
            timestamp: notification.timestamp,
            notification_type: notification.notification_type,
            action: notification.action,
            referenced_server: Some(server_id_hash),
        };

        crate::notifications::broadcast_notification(notification_item).await;

        Ok(())
    }

    async fn send_crash_notification(&self) -> Result<()> {
        use crate::notifications::{NotificationActionType, NotificationData, NotificationType};

        let pool = crate::database::get_pool();
        let server_id_hash = serde_hash::hashids::encode_single(self.id);

        let notification = NotificationData::create(
            format!("{} Crashed", self.name),
            format!("Server \"{}\" has crashed unexpectedly.", self.name),
            NotificationType::System,
            NotificationActionType::RestartServer.to_bits() | NotificationActionType::ViewDetails.to_bits(),
            Some(server_id_hash.clone()),
            pool,
        )
        .await?;

        // Broadcast to all connected users
        let notification_item = crate::notifications::NotificationItem {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            is_read: false,
            timestamp: notification.timestamp,
            notification_type: notification.notification_type,
            action: notification.action,
            referenced_server: Some(server_id_hash),
        };

        crate::notifications::broadcast_notification(notification_item).await;

        Ok(())
    }
}
