use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use ipnet::IpNet;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

const DATA_DIR_VAR: &str = "OBSIDIAN_DATA_DIR";
const DEFAULT_DATA_DIR: &str = "data";

static DATA_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
const ENV_PREFIX: &str = "OBSIDIAN_";
const NESTED_SECTIONS: [&str; 2] = ["smtp", "admin"];
const DEFAULT_SMTP_PORT: u16 = 587;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
	pub bind: String,
	pub port: u16,
	/// Port of the embedded SFTP server. `0` disables it.
	pub sftp_port: u16,
	pub data_dir: PathBuf,
	pub public_url: Option<String>,
	pub timezone: String,
	pub smtp: Option<SmtpConfig>,
	pub admin: AdminBootstrap,
	/// Reverse proxies whose forwarding headers are trusted when they are the TCP peer.
	#[serde(deserialize_with = "deserialize_proxies")]
	pub trusted_proxies: Vec<IpNet>,
	/// Environment override for the default UPnP toggle. `None` defers to the settings table.
	pub upnp_default: Option<bool>,
	/// Environment override for the graceful stop timeout. `None` defers to the settings table.
	pub stop_timeout_secs: Option<u64>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			bind: "0.0.0.0".into(),
			port: 1421,
			sftp_port: 2022,
			data_dir: default_data_dir(),
			public_url: None,
			timezone: "UTC".into(),
			smtp: None,
			admin: AdminBootstrap::default(),
			trusted_proxies: Vec::new(),
			upnp_default: None,
			stop_timeout_secs: None,
		}
	}
}

/// Accepts a TOML array or the comma-separated string `OBSIDIAN_TRUSTED_PROXIES` provides.
/// Each entry is a CIDR or a bare address, which becomes a single-host network.
fn deserialize_proxies<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<IpNet>, D::Error> {
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum Entries {
		List(Vec<String>),
		Csv(String),
	}
	let entries: Vec<String> = match Entries::deserialize(deserializer)? {
		Entries::List(entries) => entries,
		Entries::Csv(csv) => csv.split(',').map(str::to_owned).collect(),
	};
	entries
		.iter()
		.map(|entry| entry.trim())
		.filter(|entry| !entry.is_empty())
		.map(|entry| {
			entry
				.parse::<IpNet>()
				.or_else(|_| entry.parse::<IpAddr>().map(IpNet::from))
				.map_err(D::Error::custom)
		})
		.collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
	pub host: String,
	#[serde(default = "default_smtp_port")]
	pub port: u16,
	pub username: Option<String>,
	pub password: Option<String>,
	pub from: String,
	#[serde(default)]
	pub tls: SmtpTls,
}

fn default_smtp_port() -> u16 {
	DEFAULT_SMTP_PORT
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmtpTls {
	None,
	#[default]
	Starttls,
	Tls,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminBootstrap {
	pub username: Option<String>,
	pub email: Option<String>,
	pub password: Option<String>,
}

impl Config {
	pub fn data_dir() -> PathBuf {
		if let Some(dir) = DATA_DIR_OVERRIDE.get() {
			return dir.clone();
		}
		std::env::var_os(DATA_DIR_VAR).map_or_else(default_data_dir, PathBuf::from)
	}

	#[cfg(feature = "service")]
	pub fn override_data_dir(dir: PathBuf) {
		let _ = DATA_DIR_OVERRIDE.set(dir);
	}

	pub fn load() -> color_eyre::Result<Self> {
		let data_dir = Self::data_dir();
		let config = Figment::from(Serialized::defaults(Self::default()))
			.merge(Toml::file(data_dir.join("config.toml")))
			.merge(env_provider())
			.extract::<Self>()?;
		Ok(Self { data_dir, ..config })
	}
}

#[cfg(all(feature = "tray", not(debug_assertions)))]
fn default_data_dir() -> PathBuf {
	dirs::data_dir().map_or_else(|| PathBuf::from(DEFAULT_DATA_DIR), |dir| dir.join("Obsidian"))
}

#[cfg(not(all(feature = "tray", not(debug_assertions))))]
fn default_data_dir() -> PathBuf {
	PathBuf::from(DEFAULT_DATA_DIR)
}

fn env_provider() -> Env {
	Env::prefixed(ENV_PREFIX).map(|key| {
		let key = key.as_str().to_ascii_lowercase();
		let nested = NESTED_SECTIONS.iter().find_map(|section| {
			key.strip_prefix(section)
				.and_then(|rest| rest.strip_prefix('_'))
				.map(|rest| format!("{section}.{rest}"))
		});
		nested.unwrap_or(key).into()
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[allow(clippy::result_large_err)]
	fn env_overrides_nested_sections() {
		figment::Jail::expect_with(|jail| {
			jail.set_env("OBSIDIAN_DATA_DIR", jail.directory().display().to_string());
			jail.create_file("config.toml", "port = 2000\ntimezone = \"Europe/Berlin\"")?;
			jail.set_env("OBSIDIAN_PORT", "3000");
			jail.set_env("OBSIDIAN_SMTP_HOST", "mail.example.com");
			jail.set_env("OBSIDIAN_SMTP_FROM", "panel@example.com");
			jail.set_env("OBSIDIAN_ADMIN_USERNAME", "root");
			jail.set_env("OBSIDIAN_TRUSTED_PROXIES", "127.0.0.1, 10.0.0.0/8,::1");
			jail.set_env("OBSIDIAN_UPNP_DEFAULT", "true");
			jail.set_env("OBSIDIAN_STOP_TIMEOUT_SECS", "90");
			let config = Config::load().map_err(|e| e.to_string())?;
			assert_eq!(config.port, 3000);
			assert_eq!(config.timezone, "Europe/Berlin");
			let smtp = config.smtp.expect("smtp section");
			assert_eq!(smtp.host, "mail.example.com");
			assert_eq!(smtp.port, DEFAULT_SMTP_PORT);
			assert_eq!(config.admin.username.as_deref(), Some("root"));
			assert_eq!(config.trusted_proxies.len(), 3);
			assert!(config.trusted_proxies[0].contains(&IpAddr::from([127, 0, 0, 1])));
			assert!(config.trusted_proxies[1].contains(&IpAddr::from([10, 1, 2, 3])));
			assert_eq!(config.upnp_default, Some(true));
			assert_eq!(config.stop_timeout_secs, Some(90));
			Ok(())
		});
	}
}
