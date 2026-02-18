use crate::authentication::auth_data::UserRequestExt;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::broadcast;
use crate::notifications::notification_data::{NotificationCommand, NotificationData, NotificationMessage};
use actix::{Actor, ActorContext, AsyncContext, StreamHandler, WrapFuture};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use anyhow::Result;
use log::{debug, error, warn};
use tokio::sync::broadcast::error::RecvError;

/// WebSocket actor that forwards broadcast messages to the client
pub struct UpdatesWebSocket {
    user_id: u64,
}

impl UpdatesWebSocket {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}

impl Actor for UpdatesWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Updates WebSocket started for user {}", self.user_id);

        // Subscribe to the broadcast channel
        let mut receiver = broadcast::subscribe();
        let user_id = self.user_id;

        // Send initial notification list (for backwards compatibility with notification system)
        let addr = ctx.address();
        ctx.spawn(
            async move {
                let pool = crate::database::get_pool();
                match NotificationData::get_for_user(user_id, pool).await {
                    Ok(notifications) => {
                        let msg = NotificationMessage::InitialList { notifications };
                        let broadcast_msg = BroadcastMessage::Notification { message: msg };
                        if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                            addr.do_send(SendText(json));
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch notifications for user {}: {}", user_id, e);
                    }
                }
            }
            .into_actor(self),
        );

        // Spawn a task to forward broadcast messages to this WebSocket
        let addr = ctx.address();
        ctx.spawn(
            async move {
                loop {
                    match receiver.recv().await {
                        Ok(message) => {
                            // Serialize and send the message
                            match serde_json::to_string(&message) {
                                Ok(json) => {
                                    addr.do_send(SendText(json));
                                }
                                Err(e) => {
                                    error!("Failed to serialize broadcast message: {}", e);
                                }
                            }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            warn!("WebSocket for user {} lagged behind and skipped {} messages", user_id, skipped);
                            // Continue receiving
                        }
                        Err(RecvError::Closed) => {
                            debug!("Broadcast channel closed, stopping WebSocket for user {}", user_id);
                            addr.do_send(StopWebSocket);
                            break;
                        }
                    }
                }
            }
            .into_actor(self),
        );
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("Updates WebSocket stopped for user {}", self.user_id);
    }
}

/// Internal message for sending text to the WebSocket
#[derive(actix::Message)]
#[rtype(result = "()")]
struct SendText(String);

impl actix::Handler<SendText> for UpdatesWebSocket {
    type Result = ();

    fn handle(&mut self, msg: SendText, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// Internal message for stopping the WebSocket
#[derive(actix::Message)]
#[rtype(result = "()")]
struct StopWebSocket;

impl actix::Handler<StopWebSocket> for UpdatesWebSocket {
    type Result = ();

    fn handle(&mut self, _msg: StopWebSocket, ctx: &mut Self::Context) {
        ctx.stop();
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UpdatesWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let text = text.trim();
                // Handle notification commands for backwards compatibility
                match serde_json::from_str::<NotificationCommand>(text) {
                    Ok(command) => {
                        let user_id = self.user_id;
                        ctx.spawn(
                            async move {
                                if let Err(e) = handle_notification_command(command, user_id).await {
                                    error!("Failed to handle notification command: {}", e);
                                }
                            }
                            .into_actor(self),
                        );
                    }
                    Err(e) => {
                        warn!("Failed to parse notification command: {}", e);
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
) -> Result<()> {
    let pool = crate::database::get_pool();

    match command {
        NotificationCommand::MarkAsRead { ref id } => {
            NotificationData::mark_as_read(id, user_id, pool).await?;
            let msg = NotificationMessage::MarkAsRead { id: id.clone() };
            broadcast::broadcast(BroadcastMessage::Notification { message: msg });
        }
        NotificationCommand::MarkAllAsRead => {
            NotificationData::mark_all_as_read(user_id, pool).await?;
            let msg = NotificationMessage::MarkAllAsRead;
            broadcast::broadcast(BroadcastMessage::Notification { message: msg });
        }
        NotificationCommand::DeleteNotification { ref id } => {
            NotificationData::hide_for_user(id, user_id, pool).await?;
            let msg = NotificationMessage::DeleteNotification { id: id.clone() };
            broadcast::broadcast(BroadcastMessage::Notification { message: msg });
        }
        NotificationCommand::DeleteAllNotifications => {
            NotificationData::hide_all_for_user(user_id, pool).await?;
            let msg = NotificationMessage::DeleteAllNotifications;
            broadcast::broadcast(BroadcastMessage::Notification { message: msg });
        }
    }

    Ok(())
}

/// WebSocket endpoint for real-time updates
pub async fn updates_ws(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let user = match req.get_user() {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to authenticate user for updates WebSocket: {}", e);
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

    let ws = UpdatesWebSocket::new(user_id);
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

/// Configure updates routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/updates/ws").route(web::get().to(updates_ws)));
}
