use serde::{Deserialize, Deserializer, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySqlTypeInfo, MySqlValueRef};
use sqlx::{Decode, Encode, MySql, Type};

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

impl Encode<'_, MySql> for ServerType {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        let value: u8 = self.clone().into();
        <u8 as Encode<MySql>>::encode_by_ref(&value, buf)
    }
}

impl<'r> Decode<'r, MySql> for ServerType {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let int_value = <u8 as Decode<MySql>>::decode(value)?;
        Ok(ServerType::from(int_value))
    }
}

impl Type<MySql> for ServerType {
    fn type_info() -> MySqlTypeInfo {
        <u8 as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <u8 as Type<MySql>>::compatible(ty)
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