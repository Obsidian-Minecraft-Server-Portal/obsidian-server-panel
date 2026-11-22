mod topics;

use std::collections::HashSet;
use std::time::{Duration, Instant};

use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt;
use obsidian_auth::AuthUser;
use obsidian_common::Event;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::state::AppState;

const HEARTBEAT: Duration = Duration::from_secs(20);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_TOPICS: usize = 128;
const TASKS_TOPIC: &str = "tasks";

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ClientOp {
	Subscribe { topic: String },
	Unsubscribe { topic: String },
}

#[get("/ws")]
pub async fn socket(
	req: HttpRequest,
	payload: web::Payload,
	state: web::Data<AppState>,
	user: crate::auth::AuthUser,
) -> Result<HttpResponse, actix_web::Error> {
	let (response, session, stream) = actix_ws::handle(&req, payload)?;
	let events = state.bus.subscribe();
	rt::spawn(serve(session, stream, events, state.into_inner(), user.0));
	Ok(response)
}

async fn serve(
	mut session: Session,
	mut stream: MessageStream,
	mut events: broadcast::Receiver<Event>,
	state: std::sync::Arc<AppState>,
	user: AuthUser,
) {
	let mut topics = HashSet::new();
	let mut heartbeat = tokio::time::interval(HEARTBEAT);
	let mut last_seen = Instant::now();
	loop {
		let delivered = tokio::select! {
			incoming = stream.next() => {
				let Some(Ok(message)) = incoming else { break };
				last_seen = Instant::now();
				match message {
					Message::Text(text) => match handle_op(&state, &user, &mut topics, &text).await {
						Some(error) => session.text(error).await,
						None => Ok(()),
					},
					Message::Ping(bytes) => session.pong(&bytes).await,
					Message::Close(_) => break,
					_ => Ok(()),
				}
			}
			event = events.recv() => match event {
				Ok(event) if visible(&event, &user, &topics) => {
					session.text(json!({ "topic": event.topic, "data": event.data }).to_string()).await
				}
				Ok(_) | Err(RecvError::Lagged(_)) => Ok(()),
				Err(RecvError::Closed) => break,
			},
			_ = heartbeat.tick() => {
				if last_seen.elapsed() > CLIENT_TIMEOUT {
					break;
				}
				session.ping(b"").await
			}
		};
		if delivered.is_err() {
			break;
		}
	}
	let _ = session.close(None).await;
}

fn visible(event: &Event, user: &AuthUser, topics: &HashSet<String>) -> bool {
	let addressed = event.user.is_none_or(|target| target == user.id || (user.is_admin && event.topic == TASKS_TOPIC));
	addressed && topics.contains(&event.topic)
}

async fn handle_op(state: &AppState, user: &AuthUser, topics: &mut HashSet<String>, text: &str) -> Option<String> {
	let op = match serde_json::from_str::<ClientOp>(text) {
		Ok(op) => op,
		Err(error) => return Some(error_frame(&error.to_string())),
	};
	match op {
		ClientOp::Subscribe { topic } if topics.len() >= MAX_TOPICS && !topics.contains(&topic) => {
			Some(error_frame("too many subscriptions"))
		}
		ClientOp::Subscribe { topic } => match topics::authorize(state, user, &topic).await {
			Ok(()) => {
				topics.insert(topic);
				None
			}
			Err(error) => Some(error_frame(&error.to_string())),
		},
		ClientOp::Unsubscribe { topic } => {
			topics.remove(&topic);
			None
		}
	}
}

fn error_frame(message: &str) -> String {
	json!({ "error": message }).to_string()
}
