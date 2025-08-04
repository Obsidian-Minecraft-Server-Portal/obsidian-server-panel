use std::fmt::Display;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Database, Decode, Encode, Sqlite, Type};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

/// Backups are typically stored in the `{root}/backups/{server_name}/{archive}.zip` path.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupType {
	/// This will backup the world directory and the server directory, this is useful for servers that use plugins that modify the server directory.
	Full,
	/// This will backup the world directory and the server directory, this is useful for servers that use plugins that modify the server directory.
	Incremental,
	/// This will only backup the world directory, this is useful for servers that use worldedit restore system.
	/// The backup should be placed in the `{root}/{server_name}/backups/{level_name}/{archive}.zip` path.
	World,
}


impl From<String> for BackupType {
	fn from(value: String) -> Self {
		value.to_lowercase().as_str().parse().unwrap_or(BackupType::Full)
	}
}

impl Display for BackupType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				BackupType::Full => "full",
				BackupType::Incremental => "incremental",
				BackupType::World => "world",
			}
		)
	}
}

impl std::str::FromStr for BackupType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"full" => Ok(BackupType::Full),
			"incremental" => Ok(BackupType::Incremental),
			"world" => Ok(BackupType::World),
			_ => Err(format!("Invalid backup type: {}", s)),
		}
	}
}

impl From<u8> for BackupType {
	fn from(value: u8) -> Self {
		match value {
			0 => BackupType::Full,
			1 => BackupType::Incremental,
			2 => BackupType::World,
			_ => BackupType::Full,
		}
	}
}

impl From<BackupType> for u8 {
	fn from(backup_type: BackupType) -> Self {
		match backup_type {
			BackupType::Full => 0,
			BackupType::Incremental => 1,
			BackupType::World => 2,
		}
	}
}

impl Encode<'_, Sqlite> for BackupType {
	fn encode(self, buf: &mut <Sqlite as Database>::ArgumentBuffer<'_>) -> Result<IsNull, BoxDynError>
	          where
		          Self: Sized,
	{
		let value: u8 = self.into();
		buf.push(SqliteArgumentValue::Int(value as i32));
		Ok(IsNull::No)
	}

	fn encode_by_ref(&self, buf: &mut <Sqlite as Database>::ArgumentBuffer<'_>) -> Result<IsNull, BoxDynError> {
		let value: u8 = self.clone().into();
		buf.push(SqliteArgumentValue::Int(value as i32));
		Ok(IsNull::No)
	}
}

impl<'r> Decode<'r, Sqlite> for BackupType {
	fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
		let int_value = <i32 as Decode<Sqlite>>::decode(value)?;
		if !(0..=255).contains(&int_value) {
			return Err(format!("Invalid backup type value: {}", int_value).into());
		}
		Ok(BackupType::from(int_value as u8))
	}
}

impl Type<Sqlite> for BackupType {
	fn type_info() -> SqliteTypeInfo {
		<i32 as Type<Sqlite>>::type_info()
	}

	fn compatible(ty: &SqliteTypeInfo) -> bool {
		<i32 as Type<Sqlite>>::compatible(ty)
	}
}

impl<'de> Deserialize<'de> for BackupType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	                  where
		                  D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(BackupType::from(s))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn test_backup_type_display() {
		assert_eq!(BackupType::Full.to_string(), "full");
		assert_eq!(BackupType::Incremental.to_string(), "incremental");
		assert_eq!(BackupType::World.to_string(), "world");
	}

	#[test]
	fn test_backup_type_from_str() {
		assert_eq!("full".parse::<BackupType>().unwrap(), BackupType::Full);
		assert_eq!("incremental".parse::<BackupType>().unwrap(), BackupType::Incremental);
		assert_eq!("world".parse::<BackupType>().unwrap(), BackupType::World);
		assert_eq!("FULL".parse::<BackupType>().unwrap(), BackupType::Full);
		assert_eq!("InCreMeNtAl".parse::<BackupType>().unwrap(), BackupType::Incremental);
		
		// Test invalid input defaults to Full
		assert!("invalid".parse::<BackupType>().is_err());
	}

	#[test]
	fn test_backup_type_from_string() {
		assert_eq!(BackupType::from("full".to_string()), BackupType::Full);
		assert_eq!(BackupType::from("incremental".to_string()), BackupType::Incremental);
		assert_eq!(BackupType::from("world".to_string()), BackupType::World);
		assert_eq!(BackupType::from("invalid".to_string()), BackupType::Full);
	}

	#[test]
	fn test_backup_type_from_u8() {
		assert_eq!(BackupType::from(0u8), BackupType::Full);
		assert_eq!(BackupType::from(1u8), BackupType::Incremental);
		assert_eq!(BackupType::from(2u8), BackupType::World);
		assert_eq!(BackupType::from(99u8), BackupType::Full); // Invalid defaults to Full
	}

	#[test]
	fn test_backup_type_to_u8() {
		assert_eq!(u8::from(BackupType::Full), 0);
		assert_eq!(u8::from(BackupType::Incremental), 1);
		assert_eq!(u8::from(BackupType::World), 2);
	}

	#[test]
	fn test_backup_type_serialization() {
		assert_eq!(serde_json::to_string(&BackupType::Full).unwrap(), "\"full\"");
		assert_eq!(serde_json::to_string(&BackupType::Incremental).unwrap(), "\"incremental\"");
		assert_eq!(serde_json::to_string(&BackupType::World).unwrap(), "\"world\"");
	}

	#[test]
	fn test_backup_type_deserialization() {
		assert_eq!(serde_json::from_str::<BackupType>("\"full\"").unwrap(), BackupType::Full);
		assert_eq!(serde_json::from_str::<BackupType>("\"incremental\"").unwrap(), BackupType::Incremental);
		assert_eq!(serde_json::from_str::<BackupType>("\"world\"").unwrap(), BackupType::World);
		assert_eq!(serde_json::from_str::<BackupType>("\"invalid\"").unwrap(), BackupType::Full);
	}

	#[test]
	fn test_backup_type_equality() {
		assert_eq!(BackupType::Full, BackupType::Full);
		assert_eq!(BackupType::Incremental, BackupType::Incremental);
		assert_eq!(BackupType::World, BackupType::World);
		assert_ne!(BackupType::Full, BackupType::Incremental);
		assert_ne!(BackupType::Incremental, BackupType::World);
		assert_ne!(BackupType::World, BackupType::Full);
	}

	#[tokio::test]
	async fn test_backup_type_database_operations() {
		use sqlx::SqlitePool;
		
		let pool = SqlitePool::connect(":memory:").await.unwrap();
		
		// Create test table
		sqlx::query(
			"CREATE TABLE test_backups (id INTEGER PRIMARY KEY, backup_type INTEGER)"
		)
		.execute(&pool)
		.await
		.unwrap();

		// Test inserting different backup types
		for backup_type in [BackupType::Full, BackupType::Incremental, BackupType::World] {
			sqlx::query("INSERT INTO test_backups (backup_type) VALUES (?)")
				.bind(&backup_type)
				.execute(&pool)
				.await
				.unwrap();
		}

		// Test retrieving backup types
		let rows = sqlx::query_as::<_, (i64, BackupType)>("SELECT id, backup_type FROM test_backups ORDER BY id")
			.fetch_all(&pool)
			.await
			.unwrap();

		assert_eq!(rows.len(), 3);
		assert_eq!(rows[0].1, BackupType::Full);
		assert_eq!(rows[1].1, BackupType::Incremental);
		assert_eq!(rows[2].1, BackupType::World);
	}
}