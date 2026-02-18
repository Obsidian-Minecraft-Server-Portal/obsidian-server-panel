use crate::authentication::auth_data::UserRequestExt;
use crate::notifications::notification_data::{NotificationCommand, NotificationData, NotificationMessage};
use actix::{Actor, ActorContext, AsyncContext, Handler, Message as ActixMessage, StreamHandler, WrapFuture};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use anyhow::Result;
use log::{debug, error, warn};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

/// Global registry of active WebSocket connections per user
pub static NOTIFICATION_CONNECTIONS: LazyLock<Arc<RwLock<HashMap<u64, Vec<actix::Addr<NotificationWebSocket>>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// WebSocket actor for notification streaming
pub struct NotificationWebSocket {
    user_id: u64,
}

impl NotificationWebSocket {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}

impl Actor for NotificationWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Notification WebSocket started for user {}", self.user_id);
        let user_id = self.user_id;
        let addr = ctx.address();

        // Register this connection
        tokio::spawn(async move {
            let mut connections = NOTIFICATION_CONNECTIONS.write().await;
            connections.entry(user_id).or_insert_with(Vec::new).push(addr.clone());
        });

        // Send initial notification list
        let user_id = self.user_id;
        let addr = ctx.address();
        ctx.spawn(
            async move {
                let pool = crate::database::get_pool();
                match NotificationData::get_for_user(user_id, pool).await {
                    Ok(notifications) => {
                        let msg = NotificationMessage::InitialList { notifications };
                        addr.do_send(SendMessage(msg));
                    }
                    Err(e) => {
                        error!("Failed to fetch notifications for user {}: {}", user_id, e);
                        let msg = NotificationMessage::Error {
                            message: "Failed to load notifications".to_string(),
                        };
                        addr.do_send(SendMessage(msg));
                    }
                }
            }
            .into_actor(self),
        );
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("Notification WebSocket stopped for user {}", self.user_id);
        let user_id = self.user_id;

        // Unregister this connection (we can't easily remove the specific address, so we'll clean up later)
        tokio::spawn(async move {
            let mut connections = NOTIFICATION_CONNECTIONS.write().await;
            if let Some(addrs) = connections.get_mut(&user_id) {
                // Remove disconnected addresses by checking if they're still connected
                addrs.retain(|addr| addr.connected());
                if addrs.is_empty() {
                    connections.remove(&user_id);
                }
            }
        });
    }
}

/// Internal message for sending notifications through the WebSocket
#[derive(ActixMessage)]
#[rtype(result = "()")]
struct SendMessage(NotificationMessage);

impl Handler<SendMessage> for NotificationWebSocket {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg.0) {
            Ok(json) => ctx.text(json),
            Err(e) => {
                error!("Failed to serialize notification message: {}", e);
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let text = text.trim();
                match serde_json::from_str::<NotificationCommand>(text) {
                    Ok(command) => {
                        let user_id = self.user_id;
                        let addr = ctx.address();
                        ctx.spawn(
                            async move {
                                if let Err(e) = handle_notification_command(command, user_id, addr).await {
                                    error!("Failed to handle notification command: {}", e);
                                }
                            }
                            .into_actor(self),
                        );
                    }
                    Err(e) => {
                        warn!("Failed to parse notification command: {}", e);
                        let msg = NotificationMessage::Error {
                            message: format!("Invalid command format: {}", e),
                        };
                        ctx.address().do_send(SendMessage(msg));
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                debug!("WebSocket close requested for user {}: {:?}", self.user_id, reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// Handle a notification command from the client
async fn handle_notification_command(
    command: NotificationCommand,
    user_id: u64,
    addr: actix::Addr<NotificationWebSocket>,
) -> Result<()> {
    let pool = crate::database::get_pool();

    match command {
        NotificationCommand::MarkAsRead { id } => {
            NotificationData::mark_as_read(&id, user_id, pool).await?;
            addr.do_send(SendMessage(NotificationMessage::MarkAsRead { id }));
        }
        NotificationCommand::MarkAllAsRead => {
            NotificationData::mark_all_as_read(user_id, pool).await?;
            addr.do_send(SendMessage(NotificationMessage::MarkAllAsRead));
        }
        NotificationCommand::DeleteNotification { id } => {
            NotificationData::hide_for_user(&id, user_id, pool).await?;
            addr.do_send(SendMessage(NotificationMessage::DeleteNotification { id }));
        }
        NotificationCommand::DeleteAllNotifications => {
            NotificationData::hide_all_for_user(user_id, pool).await?;
            addr.do_send(SendMessage(NotificationMessage::DeleteAllNotifications));
        }
    }

    Ok(())
}

/// WebSocket endpoint for notifications
pub async fn notifications_ws(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let user = match req.get_user() {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to authenticate user for notification WebSocket: {}", e);
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication required"
            }));
        }
    };

    let user_id = match user.id {
        Some(id) => id,
        None => {
            error!("User has no ID");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid user"
            }));
        }
    };

    let ws = NotificationWebSocket::new(user_id);
    match ws::start(ws, &req, stream) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to start WebSocket: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to start WebSocket"
            }))
        }
    }
}

/// Broadcast a notification to all connected users
pub async fn broadcast_notification(notification: crate::notifications::notification_data::NotificationItem) {
    let connections = NOTIFICATION_CONNECTIONS.read().await;

    for (user_id, addrs) in connections.iter() {
        for addr in addrs {
            if addr.connected() {
                addr.do_send(SendMessage(NotificationMessage::NewNotification {
                    notification: notification.clone(),
                }));
            }
        }
    }
}

/// Broadcast to a specific user
pub async fn send_to_user(user_id: u64, message: NotificationMessage) {
    let connections = NOTIFICATION_CONNECTIONS.read().await;

    if let Some(addrs) = connections.get(&user_id) {
        for addr in addrs {
            if addr.connected() {
                addr.do_send(SendMessage(message.clone()));
            }
        }
    }
}

/// Configure notification routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/notifications/ws").route(web::get().to(notifications_ws)));
}
