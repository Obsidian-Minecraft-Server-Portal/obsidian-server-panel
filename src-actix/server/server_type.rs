use serde::{Deserialize, Deserializer, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Database, Decode, Encode, Sqlite, Type};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ServerType {
    Vanilla,
    Forge,
    Fabric,
    NeoForge,
    Quilt,
    Custom,
}

impl From<u8> for ServerType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Vanilla,
            1 => Self::Forge,
            2 => Self::Fabric,
            3 => Self::NeoForge,
            4 => Self::Quilt,
            _ => Self::Custom,
        }
    }
}

impl From<String> for ServerType {
    fn from(value: String) -> Self {
        match value.as_str().to_lowercase().as_str() {
            "vanilla" => Self::Vanilla,
            "forge" => Self::Forge,
            "fabric" => Self::Fabric,
            "neo-forge" | "neoforge" => Self::NeoForge,
            "quilt" => Self::Quilt,
            _ => Self::Custom,
        }
    }
}

impl From<ServerType> for u8 {
    fn from(value: ServerType) -> Self {
        match value {
            ServerType::Vanilla => 0,
            ServerType::Forge => 1,
            ServerType::Fabric => 2,
            ServerType::NeoForge => 3,
            ServerType::Quilt => 4,
            ServerType::Custom => 5,
        }
    }
}

impl From<ServerType> for String {
    fn from(value: ServerType) -> Self {
        match value {
            ServerType::Vanilla => "Vanilla".to_string(),
            ServerType::Forge => "Forge".to_string(),
            ServerType::Fabric => "Fabric".to_string(),
            ServerType::NeoForge => "NeoForge".to_string(),
            ServerType::Quilt => "Quilt".to_string(),
            ServerType::Custom => "Custom".to_string(),
        }
    }
}

impl Encode<'_, Sqlite> for ServerType {
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

impl<'r> Decode<'r, Sqlite> for ServerType {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let int_value = <i32 as Decode<Sqlite>>::decode(value)?;
        if !(0..=255).contains(&int_value) {
            return Err(format!("Invalid server status value: {}", int_value).into());
        }
        Ok(ServerType::from(int_value as u8))
    }
}

impl Type<Sqlite> for ServerType {
    fn type_info() -> SqliteTypeInfo {
        <i32 as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <i32 as Type<Sqlite>>::compatible(ty)
    }
}


impl<'de> Deserialize<'de> for ServerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                      where
                          D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ServerType::from(s))
    }
}