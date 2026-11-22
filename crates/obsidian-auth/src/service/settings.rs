use obsidian_common::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::AuthService;

impl AuthService {
	pub async fn setting(&self, key: &str) -> Result<Option<String>> {
		Ok(sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
			.bind(key)
			.fetch_optional(self.pool())
			.await?)
	}

	pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
		sqlx::query(
			"INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value = excluded.value",
		)
		.bind(key)
		.bind(value)
		.execute(self.pool())
		.await?;
		Ok(())
	}

	pub async fn delete_setting(&self, key: &str) -> Result<()> {
		sqlx::query("DELETE FROM settings WHERE key = ?")
			.bind(key)
			.execute(self.pool())
			.await?;
		Ok(())
	}

	/// Reads a JSON setting that is encrypted at rest (used for SMTP credentials).
	pub async fn secret_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
		let Some(sealed) = self.setting(key).await? else {
			return Ok(None);
		};
		Ok(Some(serde_json::from_slice(&self.secrets.open(&sealed)?)?))
	}

	pub async fn set_secret_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
		let sealed = self.secrets.seal(&serde_json::to_vec(value)?)?;
		self.set_setting(key, &sealed).await
	}
}
