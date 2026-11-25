//! Live player positions for the map. While a client watches an instance's players topic and the server runs, the
//! poller reads the online players and where each one stands every few seconds: over RCON when that is opted into, and
//! through the console otherwise.

mod console;
mod heads;
mod rcon;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use obsidian_common::{CommonError, EventBus, InstanceId, Paths, Result, TopicDemand};
use obsidian_db::Db;
use obsidian_instances::{InstanceManager, read_server_properties};
use obsidian_runner::RunState;
use obsidian_worldmap::level::discover;
use obsidian_worldmap::players::{EntityPosition, PlayerList, parse_list, parse_positions};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::time::MissedTickBehavior;
use utoipa::ToSchema;

use self::console::Console;
use self::heads::Heads;
use self::rcon::RconClient;
use crate::jars;
use crate::settings;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const LIST_WAIT: Duration = Duration::from_secs(1);
const POSITION_WAIT: Duration = Duration::from_secs(1);
const RCON_RETRY: Duration = Duration::from_secs(10);
const SKIN_RETRY: Duration = Duration::from_secs(600);
const FIRST_RCON_PORT: u16 = 25575;
const DEFAULT_DIMENSION: &str = "minecraft:overworld";
const DEFAULT_LEVEL: &str = "world";
const NOT_RUNNING: &str = "The server is not running.";
const MAP_OFF: &str = "The world map is turned off for this server.";
const RCON_PENDING: &str = "RCON is set up and takes effect when the server next starts. The console is read until then.";

pub fn players_topic(id: InstanceId) -> String {
	format!("instance.{id}.players")
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapPlayers {
	pub state: LiveState,
	/// How the positions were read: `rcon` or `console`.
	pub source: Option<String>,
	pub message: Option<String>,
	/// Milliseconds since the Unix epoch.
	pub updated_at: i64,
	pub players: Vec<LivePlayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiveState {
	/// No reading has come in since a client started watching.
	Pending,
	/// The positions were read just now.
	Live,
	/// The server is not running, or the map is off.
	Offline,
	/// The last reading failed.
	Unavailable,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
	pub name: String,
	pub uuid: Option<String>,
	/// Dimension id, such as `minecraft:the_nether`.
	pub dimension: String,
	pub position: [f64; 3],
	/// The head's file name in the head cache.
	pub head: String,
}

impl WorldMapPlayers {
	pub fn offline(message: &str) -> Self {
		Self {
			state: LiveState::Offline,
			source: None,
			message: Some(message.to_owned()),
			updated_at: now_millis(),
			players: Vec::new(),
		}
	}

	pub fn pending() -> Self {
		Self {
			state: LiveState::Pending,
			source: None,
			message: Some("Waiting for the first reading.".to_owned()),
			updated_at: now_millis(),
			players: Vec::new(),
		}
	}

	fn unavailable(message: String) -> Self {
		Self {
			state: LiveState::Unavailable,
			source: None,
			message: Some(message),
			updated_at: now_millis(),
			players: Vec::new(),
		}
	}
}

pub struct LiveService {
	db: Db,
	paths: Arc<Paths>,
	instances: InstanceManager,
	bus: EventBus,
	demand: TopicDemand,
	heads: Heads,
	snapshots: Mutex<HashMap<InstanceId, WorldMapPlayers>>,
}

impl LiveService {
	pub fn new(db: Db, paths: Arc<Paths>, instances: InstanceManager, bus: EventBus, demand: TopicDemand) -> Result<Arc<Self>> {
		let heads = Heads::new(&paths.cache_dir)?;
		Ok(Arc::new(Self {
			db,
			paths,
			instances,
			bus,
			demand,
			heads,
			snapshots: Mutex::new(HashMap::new()),
		}))
	}

	pub fn spawn(self: &Arc<Self>) {
		tokio::spawn(Arc::clone(self).run());
	}

	pub fn snapshot(&self, id: InstanceId) -> Option<WorldMapPlayers> {
		self.snapshots.lock().unwrap_or_else(PoisonError::into_inner).get(&id).cloned()
	}

	pub fn head_path(&self, name: &str) -> Option<std::path::PathBuf> {
		heads::is_head_file(name).then(|| self.heads.path(name))
	}

	/// Writes the RCON keys into the server's properties. The server reads them when it next starts.
	pub async fn configure_rcon(&self, id: InstanceId, enabled: bool) -> Result<()> {
		let mut properties = self.instances.properties(id).await?;
		if enabled {
			if properties.get("rcon.port").and_then(|port| port.parse::<u16>().ok()).is_none() {
				let port = self.free_rcon_port(id).await?;
				properties.insert("rcon.port".into(), port.to_string());
			}
			if properties.get("rcon.password").is_none_or(String::is_empty) {
				properties.insert("rcon.password".into(), obsidian_auth::generate_token());
			}
		}
		properties.insert("enable-rcon".into(), enabled.to_string());
		self.instances.set_properties(id, properties).await
	}

	async fn free_rcon_port(&self, id: InstanceId) -> Result<u16> {
		let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, directory FROM instances").fetch_all(self.db.pool()).await?;
		let mut taken = HashSet::new();
		for (_, directory) in rows.iter().filter(|(other_id, _)| *other_id != id) {
			if let Ok(properties) = read_server_properties(Path::new(directory)).await {
				taken.extend(properties.get("rcon.port").and_then(|port| port.parse::<u16>().ok()));
			}
		}
		for port in FIRST_RCON_PORT..=u16::MAX {
			if !taken.contains(&port) && TcpListener::bind(("0.0.0.0", port)).await.is_ok() {
				return Ok(port);
			}
		}
		Err(CommonError::Conflict("no free port is left for RCON".into()))
	}

	async fn run(self: Arc<Self>) {
		let mut ticker = tokio::time::interval(POLL_INTERVAL);
		ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
		let mut sessions: HashMap<InstanceId, Session> = HashMap::new();
		loop {
			ticker.tick().await;
			let watched = self.watched();
			let released: Vec<InstanceId> = sessions.keys().filter(|id| !watched.contains(id)).copied().collect();
			for id in released {
				if let Some(mut session) = sessions.remove(&id) {
					session.show_replies();
				}
			}
			for id in watched {
				let session = sessions.entry(id).or_insert_with(|| Session::new(self.instances.clone(), id));
				self.poll(id, session).await;
			}
		}
	}

	fn watched(&self) -> HashSet<InstanceId> {
		self.demand
			.watched_topics()
			.iter()
			.filter_map(|topic| topic.strip_prefix("instance.")?.strip_suffix(".players")?.parse::<InstanceId>().ok())
			.collect()
	}

	async fn poll(&self, id: InstanceId, session: &mut Session) {
		let snapshot = match self.sample(id, session).await {
			Ok(Sampled::Reading(reading)) => WorldMapPlayers {
				state: LiveState::Live,
				source: Some(reading.source.to_owned()),
				message: reading.message,
				updated_at: now_millis(),
				players: reading.players,
			},
			Ok(Sampled::Offline(message)) => {
				session.show_replies();
				WorldMapPlayers::offline(message)
			}
			Err(error) => WorldMapPlayers::unavailable(error.to_string()),
		};
		self.snapshots.lock().unwrap_or_else(PoisonError::into_inner).insert(id, snapshot.clone());
		self.bus.publish(players_topic(id), serde_json::to_value(&snapshot).unwrap_or_default());
	}

	async fn sample(&self, id: InstanceId, session: &mut Session) -> Result<Sampled> {
		let settings = settings::load(&self.db, id).await?;
		if !settings.enabled {
			return Ok(Sampled::Offline(MAP_OFF));
		}
		if self.instances.status(id).await?.state != RunState::Running {
			return Ok(Sampled::Offline(NOT_RUNNING));
		}
		session.hide_replies();
		let instance = self.instances.get(id).await?;
		let properties = read_server_properties(&instance.directory).await?;
		let level = properties.get("level-name").map_or(DEFAULT_LEVEL, String::as_str);
		let mut dimensions: Vec<String> = discover(&instance.directory, level)
			.map_err(other)?
			.into_iter()
			.map(|dimension| dimension.id)
			.collect();
		if dimensions.is_empty() {
			dimensions.push(DEFAULT_DIMENSION.to_owned());
		}

		let mut message = None;
		let (found, source) = match self.connect_rcon(session, &properties, settings.rcon).await {
			Some(mut client) => match gather(Transport::Rcon(&mut client), &dimensions).await {
				Ok(found) => {
					session.rcon = Some(client);
					(found, "rcon")
				}
				Err(error) => {
					session.rcon_retry_at = Some(Instant::now() + RCON_RETRY);
					message = Some(format!("RCON stopped answering ({error}). The console is read until it returns."));
					(gather(Transport::Console(&session.console), &dimensions).await?, "console")
				}
			},
			None => {
				if settings.rcon {
					message = Some(RCON_PENDING.to_owned());
				}
				(gather(Transport::Console(&session.console), &dimensions).await?, "console")
			}
		};

		self.ensure_head_defaults(session, &instance.mc_version).await;
		let usercache = heads::user_cache(&instance.directory);
		let mut players = Vec::with_capacity(found.len());
		for player in found {
			let uuid = usercache.get(&player.name.to_ascii_lowercase()).cloned();
			if let (Some(uuid), true) = (&uuid, settings.consent) {
				self.request_skin(session, uuid);
			}
			let head = self.heads.file_for(uuid.as_deref());
			players.push(LivePlayer {
				name: player.name,
				uuid,
				dimension: player.dimension,
				position: player.position,
				head,
			});
		}
		Ok(Sampled::Reading(Reading { source, message, players }))
	}

	async fn connect_rcon(&self, session: &mut Session, properties: &BTreeMap<String, String>, wanted: bool) -> Option<RconClient> {
		if !wanted {
			session.rcon = None;
			return None;
		}
		if let Some(client) = session.rcon.take() {
			return Some(client);
		}
		if session.rcon_retry_at.is_some_and(|retry| Instant::now() < retry) {
			return None;
		}
		let port: u16 = properties.get("rcon.port")?.parse().ok()?;
		let password = properties.get("rcon.password").map_or("", String::as_str);
		match RconClient::connect(port, password).await {
			Ok(client) => {
				session.rcon_retry_at = None;
				Some(client)
			}
			Err(_) => {
				session.rcon_retry_at = Some(Instant::now() + RCON_RETRY);
				None
			}
		}
	}

	async fn ensure_head_defaults(&self, session: &mut Session, mc_version: &str) {
		if session.defaults_ready {
			return;
		}
		let client_jar = jars::client_jar_path(&self.paths, mc_version);
		if client_jar.is_file() && self.heads.ensure_defaults(&client_jar).await.is_ok() {
			session.defaults_ready = true;
		}
	}

	fn request_skin(&self, session: &mut Session, uuid: &str) {
		if self.heads.has(&heads::skin_file(uuid)) {
			return;
		}
		if session.skin_requests.get(uuid).is_some_and(|asked| asked.elapsed() < SKIN_RETRY) {
			return;
		}
		session.skin_requests.insert(uuid.to_owned(), Instant::now());
		let heads = self.heads.clone();
		let uuid = uuid.to_owned();
		tokio::spawn(async move {
			if let Err(error) = heads.fetch_skin(&uuid).await {
				tracing::debug!(%error, %uuid, "could not fetch a skin");
			}
		});
	}
}

/// What the poller keeps for an instance between readings.
struct Session {
	console: Console,
	hidden: bool,
	rcon: Option<RconClient>,
	rcon_retry_at: Option<Instant>,
	defaults_ready: bool,
	skin_requests: HashMap<String, Instant>,
}

impl Session {
	fn new(instances: InstanceManager, id: InstanceId) -> Self {
		Self {
			console: Console::new(instances, id),
			hidden: false,
			rcon: None,
			rcon_retry_at: None,
			defaults_ready: false,
			skin_requests: HashMap::new(),
		}
	}

	fn hide_replies(&mut self) {
		if !self.hidden {
			self.console.hide_replies();
			self.hidden = true;
		}
	}

	fn show_replies(&mut self) {
		if self.hidden {
			self.console.show_replies();
			self.hidden = false;
		}
	}
}

enum Sampled {
	Offline(&'static str),
	Reading(Reading),
}

struct Reading {
	source: &'static str,
	message: Option<String>,
	players: Vec<LivePlayer>,
}

struct Found {
	name: String,
	dimension: String,
	position: [f64; 3],
}

/// Where the commands go: over RCON when it answers, otherwise through the console.
enum Transport<'a> {
	Rcon(&'a mut RconClient),
	Console(&'a Console),
}

impl Transport<'_> {
	async fn list(&mut self) -> Result<Option<PlayerList>> {
		match self {
			Self::Rcon(client) => Ok(parse_list(&client.command("list").await?)),
			Self::Console(console) => {
				let replies = console
					.ask("list", LIST_WAIT, |lines| lines.iter().any(|line| parse_list(line).is_some()))
					.await?;
				Ok(replies.iter().rev().find_map(|line| parse_list(line)))
			}
		}
	}

	async fn positions(&mut self, command: &str, wanted: &std::collections::BTreeSet<String>) -> Result<Vec<EntityPosition>> {
		match self {
			Self::Rcon(client) => Ok(parse_positions(&client.command(command).await?)),
			Self::Console(console) => {
				let replies = console.ask(command, POSITION_WAIT, |lines| all_found(lines, wanted)).await?;
				Ok(replies.iter().flat_map(|line| parse_positions(line)).collect())
			}
		}
	}
}

/// Reads who is online, then where each of them stands, one dimension at a time, stopping once everyone is placed.
async fn gather(mut transport: Transport<'_>, dimensions: &[String]) -> Result<Vec<Found>> {
	let list = transport
		.list()
		.await?
		.ok_or_else(|| other("the server did not answer list"))?;
	let mut wanted: std::collections::BTreeSet<String> = list.names.into_iter().collect();
	let mut found = Vec::new();
	for dimension in dimensions {
		if wanted.is_empty() {
			break;
		}
		let command = format!("execute as @a at @s if dimension {dimension} run data get entity @s Pos");
		for position in transport.positions(&command, &wanted).await? {
			if wanted.remove(&position.name) {
				found.push(Found {
					name: position.name,
					dimension: dimension.clone(),
					position: position.position,
				});
			}
		}
	}
	Ok(found)
}

fn all_found(lines: &[String], wanted: &std::collections::BTreeSet<String>) -> bool {
	let placed: HashSet<String> = lines.iter().flat_map(|line| parse_positions(line)).map(|found| found.name).collect();
	wanted.iter().all(|name| placed.contains(name))
}

fn now_millis() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as i64)
}

fn other(error: impl std::fmt::Display) -> CommonError {
	CommonError::Other(error.to_string())
}
