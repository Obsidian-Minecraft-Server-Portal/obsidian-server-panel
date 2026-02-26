use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use log::{debug, error};
use minecraft_server::{ServerEvent, ServerEventHandler};
use obsidian_upnp::UpnpManager;

/// Web application event handler that bridges `minecraft_server` crate events
/// back to the web app's database, broadcast system, UPnP, and notifications.
pub struct WebEventHandler {
    server_id: u64,
    owner_id: u64,
    upnp_enabled: bool,
    server_port: u16,
    server_name: String,
}

impl WebEventHandler {
    pub fn new(server: &ServerData) -> Self {
        let server_port = server
            .get_server_properties()
            .ok()
            .and_then(|p| p.server_port)
            .unwrap_or(25565) as u16;

        Self {
            server_id: server.id,
            owner_id: server.owner_id,
            upnp_enabled: server.upnp,
            server_port,
            server_name: server.name.clone(),
        }
    }

    async fn update_status(&self, status: ServerStatus) {
        if let Ok(Some(mut server)) = ServerData::get(self.server_id, self.owner_id).await {
            server.status = status;
            if let Err(e) = server.save().await {
                error!("Failed to save server status: {}", e);
            }
            broadcast::broadcast(BroadcastMessage::ServerUpdate {
                server: server.clone(),
            });
        }
    }

    async fn cleanup_upnp(&self) {
        if self.upnp_enabled
            && let Err(e) = UpnpManager::global()
                .remove_port(self.server_port)
                .await
        {
            error!(
                "Failed to remove UPnP port {} for server {}: {}",
                self.server_port, self.server_id, e
            );
        }
    }

    async fn send_notification(
        &self,
        title: String,
        message: String,
        action: u16,
    ) {
        use crate::notifications::{NotificationData, NotificationType};

        let pool = crate::database::get_pool();
        let server_id_hash = serde_hash::hashids::encode_single(self.server_id);

        match NotificationData::create(
            title,
            message,
            NotificationType::System,
            action,
            Some(server_id_hash.clone()),
            pool,
        )
        .await
        {
            Ok(notification) => {
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
            }
            Err(e) => {
                error!("Failed to create notification: {}", e);
            }
        }
    }
}

impl ServerEventHandler for WebEventHandler {
    async fn on_event(&self, event: ServerEvent) {
        match event {
            ServerEvent::StatusChanged { ref status } => {
                let web_status = match status {
                    minecraft_server::ServerStatus::Idle => ServerStatus::Idle,
                    minecraft_server::ServerStatus::Running => ServerStatus::Running,
                    minecraft_server::ServerStatus::Stopped => ServerStatus::Stopped,
                    minecraft_server::ServerStatus::Error => ServerStatus::Error,
                    minecraft_server::ServerStatus::Starting => ServerStatus::Starting,
                    minecraft_server::ServerStatus::Stopping => ServerStatus::Stopping,
                    minecraft_server::ServerStatus::Crashed => ServerStatus::Crashed,
                    minecraft_server::ServerStatus::Hanging => ServerStatus::Hanging,
                };
                self.update_status(web_status).await;
            }
            ServerEvent::Started => {
                debug!("Server {} started", self.server_id);
                use crate::notifications::NotificationActionType;
                self.send_notification(
                    format!("{} Started", self.server_name),
                    format!("Server \"{}\" has been successfully started.", self.server_name),
                    NotificationActionType::StopServer.to_bits(),
                )
                .await;
            }
            ServerEvent::Stopped => {
                debug!("Server {} stopped", self.server_id);
                self.cleanup_upnp().await;
                use crate::notifications::NotificationActionType;
                self.send_notification(
                    format!("{} Stopped", self.server_name),
                    format!("Server \"{}\" has been stopped.", self.server_name),
                    NotificationActionType::StartServer.to_bits(),
                )
                .await;
            }
            ServerEvent::Crashed { exit_code } => {
                error!(
                    "Server {} crashed with exit code {}",
                    self.server_id, exit_code
                );
                self.cleanup_upnp().await;
                use crate::notifications::NotificationActionType;
                self.send_notification(
                    format!("{} Crashed", self.server_name),
                    format!("Server \"{}\" has crashed unexpectedly.", self.server_name),
                    NotificationActionType::RestartServer.to_bits()
                        | NotificationActionType::ViewDetails.to_bits(),
                )
                .await;
            }
            ServerEvent::ConsoleOutput { .. } => {
                // Console output is handled separately via subscribe_output()
            }
            ServerEvent::InstallProgress { .. } => {
                // Installation progress can be forwarded via broadcast if needed
            }
            ServerEvent::JavaVersionError => {
                error!(
                    "Java version mismatch detected for server {}",
                    self.server_id
                );
            }
        }
    }
}
