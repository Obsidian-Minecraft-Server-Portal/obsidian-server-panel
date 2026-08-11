use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use anyhow::Result;
use log::{debug, error, warn};
use obsidian_upnp::{UpnpManager, PortMappingProtocol};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_interactive::{AsynchronousInteractiveProcess, ProcessHandle};

#[cfg(target_os = "linux")]
use crate::server::screen_session::{self, ScreenSession};

#[derive(Clone)]
pub(crate) enum ServerHandle {
    Direct(u32),
    #[cfg(target_os = "linux")]
    Screen(Arc<ScreenSession>),
}

pub(crate) static ACTIVE_SERVERS: OnceLock<Arc<Mutex<HashMap<u64, ServerHandle>>>> = OnceLock::new();

fn active_servers() -> &'static Arc<Mutex<HashMap<u64, ServerHandle>>> {
    ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

async fn handle_for(id: u64) -> Result<ServerHandle> {
    active_servers().lock().await.get(&id).cloned().ok_or_else(|| anyhow::anyhow!("Server not running"))
}

async fn process_for(pid: u32) -> Result<ProcessHandle> {
    AsynchronousInteractiveProcess::get_process_by_pid(pid).await.ok_or_else(|| anyhow::anyhow!("Server process not found"))
}

enum ConsoleReader {
    Direct(ProcessHandle),
    #[cfg(target_os = "linux")]
    Screen(tokio::sync::broadcast::Receiver<String>),
}

impl ConsoleReader {
    async fn from_handle(handle: &ServerHandle) -> Result<Self> {
        match handle {
            ServerHandle::Direct(pid) => Ok(Self::Direct(process_for(*pid).await?)),
            #[cfg(target_os = "linux")]
            ServerHandle::Screen(session) => Ok(Self::Screen(session.subscribe())),
        }
    }

    async fn next_line(&mut self) -> Result<Option<String>> {
        match self {
            Self::Direct(process) => process.receive_output().await,
            #[cfg(target_os = "linux")]
            Self::Screen(rx) => {
                use tokio::sync::broadcast::error::RecvError;
                match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                    Ok(Ok(line)) => Ok(Some(line)),
                    Ok(Err(RecvError::Closed)) => Err(anyhow::anyhow!("Console stream closed")),
                    Ok(Err(RecvError::Lagged(_))) => Ok(None),
                    Err(_) => Ok(None),
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) async fn reattach_screen_sessions(pool: &crate::database::Pool) -> Result<()> {
    if !screen_session::screen_available().await {
        return Ok(());
    }
    for entry in screen_session::list_sessions().await {
        let Some(id) = screen_session::server_id_from_session_name(&entry.name) else { continue };
        if !screen_session::is_pid_alive(entry.pid) {
            continue;
        }
        let Ok(Some(mut server)) = ServerData::get_with_pool(id, pool).await else { continue };
        if server.has_server_process().await {
            continue;
        }
        let logfile = server.get_directory_path().join(screen_session::CONSOLE_LOG_FILE);
        let session = ScreenSession::reattach(entry.pid, entry.name.clone(), logfile, server.exit_handler());
        active_servers().lock().await.insert(id, ServerHandle::Screen(session));
        server.status = ServerStatus::Running;
        server.save_with_pool(pool).await?;
        broadcast::broadcast(BroadcastMessage::ServerUpdate { server: server.clone() });
        log::info!("Reattached to screen session {} for server {}", entry.name, server.name);
    }
    Ok(())
}

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

        let mut args: Vec<String> = vec![format!("-Xmx{}G", self.max_memory), format!("-Xms{}G", self.min_memory)];
        args.extend(self.java_args.split_whitespace().map(String::from));
        if !self.server_jar.is_empty() {
            args.push("-jar".into());
            args.push(self.server_jar.clone());
        }
        args.extend(self.minecraft_args.split_whitespace().map(String::from));

        #[cfg(target_os = "linux")]
        let handle = if screen_session::screen_available().await {
            let session = ScreenSession::spawn(self.id, &self.java_executable, &args, &directory_path, self.exit_handler()).await?;
            debug!("Server {} started in screen session {} (pid {})", self.id, session.name, session.pid);
            ServerHandle::Screen(session)
        } else {
            warn!("GNU screen not found; starting server {} as a direct child process", self.id);
            self.spawn_direct(args, &directory_path).await?
        };
        #[cfg(not(target_os = "linux"))]
        let handle = self.spawn_direct(args, &directory_path).await?;

        let mut reader = ConsoleReader::from_handle(&handle).await?;
        active_servers().lock().await.insert(self.id, handle);

        self.last_started = Some(chrono::Utc::now().timestamp() as u64);
        self.save().await?;

        loop {
            let line = reader.next_line().await?;
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

    fn exit_handler(&self) -> impl Fn(i32) + Send + Sync + 'static {
        let server = self.clone();
        move |exit_code| {
            let mut server = server.clone();
            tokio::spawn(async move {
                debug!("Server exited with code {}", exit_code);
                let result = if exit_code != 0 { server.remove_server_crashed().await } else { server.remove_server().await };
                if let Err(e) = result {
                    error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
                }
            });
        }
    }

    async fn spawn_direct(&self, args: Vec<String>, directory: &Path) -> Result<ServerHandle> {
        let mut builder = AsynchronousInteractiveProcess::new(&self.java_executable)
            .with_arguments(args)
            .with_working_directory(directory)
            .process_exit_callback(self.exit_handler());
        let pid = builder.start().await?;
        debug!("Server started with pid {}", pid);
        Ok(ServerHandle::Direct(pid))
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
        match handle_for(self.id).await? {
            ServerHandle::Direct(pid) => process_for(pid).await?.kill().await?,
            #[cfg(target_os = "linux")]
            ServerHandle::Screen(session) => session.quit().await?,
        }
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
        active_servers().lock().await.remove(&self.id);
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
        active_servers().lock().await.remove(&self.id);
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
        match handle_for(self.id).await? {
            ServerHandle::Direct(pid) => process_for(pid).await?.send_input(command).await?,
            #[cfg(target_os = "linux")]
            ServerHandle::Screen(session) => session.send_input(command).await?,
        }
        Ok(())
    }

    pub async fn attach_to_stdout(&self, sender: tokio::sync::mpsc::Sender<actix_web_lab::sse::Event>) -> Result<()> {
        let handle = handle_for(self.id).await?;
        let mut reader = ConsoleReader::from_handle(&handle).await?;

        loop {
            // Add timeout to detect stale connections
            let output_future = reader.next_line();
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
        active_servers().lock().await.contains_key(&self.id)
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
