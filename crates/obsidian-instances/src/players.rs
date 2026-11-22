use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_runner::RunState;
use serde_json::{Value, json};
use tokio::sync::broadcast::error::RecvError;

use crate::live::console_topic;
use crate::manager::InstanceManager;
use crate::model::{Player, PlayerAction};

const OPS: &str = "ops.json";
const WHITELIST: &str = "whitelist.json";
const BANNED: &str = "banned-players.json";
const USERCACHE: &str = "usercache.json";
const ONLINE_TIMEOUT: Duration = Duration::from_secs(3);
const ONLINE_MARKER: &str = "players online:";
const MAX_NAME_LEN: usize = 16;

fn validate_name(name: &str) -> Result<()> {
	let valid = !name.is_empty()
		&& name.len() <= MAX_NAME_LEN
		&& name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
	if valid {
		Ok(())
	} else {
		Err(CommonError::Invalid(format!("'{name}' is not a valid player name")))
	}
}

fn parse_online(line: &str) -> Option<Vec<String>> {
	let (_, names) = line.split_once(ONLINE_MARKER)?;
	Some(names.split(',').map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect())
}

fn matches_entry(entry: &Value, uuid: Option<&str>, name: &str) -> bool {
	let same_uuid = uuid.is_some_and(|uuid| entry.get("uuid").and_then(Value::as_str) == Some(uuid));
	let same_name = entry.get("name").and_then(Value::as_str).is_some_and(|known| known.eq_ignore_ascii_case(name));
	same_uuid || same_name
}

async fn read_entries(dir: &Path, file: &str) -> Result<Vec<Value>> {
	match tokio::fs::read(dir.join(file)).await {
		Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
		Err(error) => Err(error.into()),
	}
}

async fn edit_entries(dir: &Path, file: &str, change: impl FnOnce(&mut Vec<Value>)) -> Result<()> {
	let mut entries = read_entries(dir, file).await?;
	change(&mut entries);
	tokio::fs::write(dir.join(file), serde_json::to_vec_pretty(&entries)?).await?;
	Ok(())
}

async fn uuid_for(dir: &Path, name: &str) -> Result<String> {
	read_entries(dir, USERCACHE)
		.await?
		.iter()
		.find(|entry| matches_entry(entry, None, name))
		.and_then(|entry| entry.get("uuid")?.as_str().map(str::to_owned))
		.ok_or_else(|| CommonError::Invalid(format!("{name} has never joined this server")))
}

async fn upsert(dir: &Path, file: &str, uuid: &str, name: &str, extra: Value) -> Result<()> {
	edit_entries(dir, file, |entries| {
		entries.retain(|entry| !matches_entry(entry, Some(uuid), name));
		let mut entry = json!({ "uuid": uuid, "name": name });
		if let (Value::Object(target), Value::Object(extra)) = (&mut entry, extra) {
			target.extend(extra);
		}
		entries.push(entry);
	})
	.await
}

async fn remove(dir: &Path, file: &str, name: &str) -> Result<()> {
	edit_entries(dir, file, |entries| entries.retain(|entry| !matches_entry(entry, None, name))).await
}

fn merge<'a>(players: &'a mut BTreeMap<String, Player>, entry: &Value) -> Option<&'a mut Player> {
	let name = entry.get("name")?.as_str()?;
	let uuid = entry.get("uuid").and_then(Value::as_str).map(str::to_owned);
	Some(players.entry(name.to_ascii_lowercase()).or_insert_with(|| Player {
		name: name.to_owned(),
		uuid,
		online: false,
		op: false,
		whitelisted: false,
		banned: false,
	}))
}

impl InstanceManager {
	pub async fn players(&self, id: InstanceId) -> Result<Vec<Player>> {
		let instance = self.get(id).await?;
		let online = self.online_players(id).await?;
		let dir = &instance.directory;
		let mut players = BTreeMap::new();
		for entry in read_entries(dir, OPS).await? {
			if let Some(player) = merge(&mut players, &entry) {
				player.op = true;
			}
		}
		for entry in read_entries(dir, WHITELIST).await? {
			if let Some(player) = merge(&mut players, &entry) {
				player.whitelisted = true;
			}
		}
		for entry in read_entries(dir, BANNED).await? {
			if let Some(player) = merge(&mut players, &entry) {
				player.banned = true;
			}
		}
		for name in online {
			players
				.entry(name.to_ascii_lowercase())
				.or_insert_with(|| Player { name, uuid: None, online: false, op: false, whitelisted: false, banned: false })
				.online = true;
		}
		Ok(players.into_values().collect())
	}

	pub async fn player_action(&self, id: InstanceId, name: &str, action: PlayerAction) -> Result<()> {
		validate_name(name)?;
		if self.is_active_instance(id) {
			let command = match action {
				PlayerAction::Op => format!("op {name}"),
				PlayerAction::Deop => format!("deop {name}"),
				PlayerAction::Whitelist => format!("whitelist add {name}"),
				PlayerAction::Unwhitelist => format!("whitelist remove {name}"),
				PlayerAction::Kick => format!("kick {name}"),
				PlayerAction::Ban => format!("ban {name}"),
				PlayerAction::Pardon => format!("pardon {name}"),
			};
			return self.send_command(id, &command).await;
		}
		let dir = self.get(id).await?.directory;
		let offline_ban = json!({
			"created": Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string(),
			"source": "Obsidian",
			"expires": "forever",
			"reason": "Banned by an operator.",
		});
		match action {
			PlayerAction::Kick => Err(CommonError::Conflict(format!("{name} is not online"))),
			PlayerAction::Op => {
				let uuid = uuid_for(&dir, name).await?;
				upsert(&dir, OPS, &uuid, name, json!({ "level": 4, "bypassesPlayerLimit": false })).await
			}
			PlayerAction::Deop => remove(&dir, OPS, name).await,
			PlayerAction::Whitelist => {
				let uuid = uuid_for(&dir, name).await?;
				upsert(&dir, WHITELIST, &uuid, name, json!({})).await
			}
			PlayerAction::Unwhitelist => remove(&dir, WHITELIST, name).await,
			PlayerAction::Ban => {
				let uuid = uuid_for(&dir, name).await?;
				upsert(&dir, BANNED, &uuid, name, offline_ban).await
			}
			PlayerAction::Pardon => remove(&dir, BANNED, name).await,
		}
	}

	async fn online_players(&self, id: InstanceId) -> Result<Vec<String>> {
		if self.current_status(id).state != RunState::Running {
			return Ok(Vec::new());
		}
		let mut events = self.bus.subscribe();
		self.send_command(id, "list").await?;
		let topic = console_topic(id);
		let response = async {
			loop {
				match events.recv().await {
					Ok(event) if event.topic == topic => {
						if let Some(names) = event.data["line"].as_str().and_then(parse_online) {
							return names;
						}
					}
					Ok(_) | Err(RecvError::Lagged(_)) => {}
					Err(RecvError::Closed) => return Vec::new(),
				}
			}
		};
		Ok(tokio::time::timeout(ONLINE_TIMEOUT, response).await.unwrap_or_default())
	}
}

#[cfg(test)]
mod tests {
	use super::{parse_online, validate_name};

	#[test]
	fn parses_list_output() {
		let line = "[Server thread/INFO]: There are 2 of a max of 20 players online: Alex, Steve";
		assert_eq!(parse_online(line), Some(vec!["Alex".to_owned(), "Steve".to_owned()]));
		assert_eq!(parse_online("There are 0 of a max of 20 players online: "), Some(Vec::new()));
	}

	#[test]
	fn rejects_unsafe_names() {
		assert!(validate_name("Steve_01").is_ok());
		assert!(validate_name("bad name").is_err());
		assert!(validate_name("x;stop").is_err());
	}
}
