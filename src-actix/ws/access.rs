use std::collections::HashSet;
use std::future::Future;

use obsidian_auth::AuthUser;
use obsidian_common::{Event, EventBus, UserId};
use serde_json::Value;

use super::topics;
use crate::auth::Credential;
use crate::state::AppState;

const ACCESS_TOPIC: &str = "access.changed";

// Internal only: `topics::authorize` rejects this topic, so clients can never subscribe to it.
pub fn access_changed(bus: &EventBus, user: UserId) {
	bus.publish_to(user, ACCESS_TOPIC, Value::Null);
}

pub fn concerns(event: &Event, user: &AuthUser) -> bool {
	event.topic == ACCESS_TOPIC && event.user.is_none_or(|target| target == user.id)
}

pub enum Revalidation {
	Close,
	Keep { revoked: Vec<String> },
}

pub async fn revalidate(
	state: &AppState,
	credential: &Credential,
	user: &mut AuthUser,
	subscribed: &mut HashSet<String>,
) -> Revalidation {
	match credential.resolve(state).await {
		Ok(Some(current)) if current.id == user.id => *user = current,
		_ => return Revalidation::Close,
	}
	let revoked = revoke(subscribed, |topic| {
		let user = user.clone();
		async move { topics::authorize(state, &user, &topic).await.map_err(|error| error.to_string()) }
	})
	.await;
	Revalidation::Keep { revoked }
}

async fn revoke<F, Fut>(subscribed: &mut HashSet<String>, mut check: F) -> Vec<String>
where
	F: FnMut(String) -> Fut,
	Fut: Future<Output = Result<(), String>>,
{
	let mut reasons = Vec::new();
	let mut denied = Vec::new();
	for topic in subscribed.iter() {
		if let Err(reason) = check(topic.clone()).await {
			denied.push(topic.clone());
			reasons.push(reason);
		}
	}
	for topic in &denied {
		subscribed.remove(topic);
	}
	reasons
}

#[cfg(test)]
mod tests {
	use super::*;

	fn user(id: UserId) -> AuthUser {
		AuthUser { id, username: format!("user{id}"), is_admin: false }
	}

	fn event(topic: &str, target: Option<UserId>) -> Event {
		Event { topic: topic.into(), data: Value::Null, user: target }
	}

	#[tokio::test]
	async fn revoke_drops_only_denied_topics() {
		let mut subscribed: HashSet<String> =
			["instance.1.console", "instance.2.console", "tasks"].into_iter().map(String::from).collect();
		let reasons = revoke(&mut subscribed, |topic| async move {
			if topic == "instance.2.console" { Err(format!("no access to {topic}")) } else { Ok(()) }
		})
		.await;
		assert_eq!(reasons, vec!["no access to instance.2.console".to_owned()]);
		assert!(subscribed.contains("instance.1.console"));
		assert!(subscribed.contains("tasks"));
		assert!(!subscribed.contains("instance.2.console"));
	}

	#[tokio::test]
	async fn revoke_keeps_everything_when_allowed() {
		let mut subscribed: HashSet<String> = ["notifications"].into_iter().map(String::from).collect();
		let reasons = revoke(&mut subscribed, |_| async { Ok(()) }).await;
		assert!(reasons.is_empty());
		assert_eq!(subscribed.len(), 1);
	}

	#[test]
	fn access_events_target_their_user() {
		assert!(concerns(&event(ACCESS_TOPIC, Some(7)), &user(7)));
		assert!(concerns(&event(ACCESS_TOPIC, None), &user(7)));
		assert!(!concerns(&event(ACCESS_TOPIC, Some(8)), &user(7)));
		assert!(!concerns(&event("instance.1.console", Some(7)), &user(7)));
	}
}
