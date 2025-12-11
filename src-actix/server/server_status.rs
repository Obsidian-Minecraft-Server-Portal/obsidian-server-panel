use serde::{Deserialize, Deserializer, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySqlTypeInfo, MySqlValueRef};
use sqlx::{Decode, Encode, MySql, Type};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ServerStatus {
    Idle,
    Running,
    Stopped,
    Error,
    Starting,
    Stopping,
    Crashed,
    Hanging, // Added Hanging status
}

impl From<String> for ServerStatus {
    fn from(value: String) -> Self {
        value.to_lowercase().as_str().parse().unwrap_or(ServerStatus::Idle)
    }
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ServerStatus::Idle => "idle",
                ServerStatus::Running => "running",
                ServerStatus::Stopped => "stopped",
                ServerStatus::Error => "error",
                ServerStatus::Starting => "starting",
                ServerStatus::Stopping => "stopping",
                ServerStatus::Crashed => "crashed",
                ServerStatus::Hanging => "hanging",
            }
        )
    }
}

impl std::str::FromStr for ServerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "idle" => Ok(ServerStatus::Idle),
            "running" => Ok(ServerStatus::Running),
            "stopped" => Ok(ServerStatus::Stopped),
            "error" => Ok(ServerStatus::Error),
            "starting" => Ok(ServerStatus::Starting),
            "stopping" => Ok(ServerStatus::Stopping),
            "crashed" => Ok(ServerStatus::Crashed),
            "hanging" => Ok(ServerStatus::Hanging), // Parse for Hanging status
            _ => Err(format!("Invalid server status: {}", s)),
        }
    }
}

impl From<u8> for ServerStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => ServerStatus::Idle,
            1 => ServerStatus::Running,
            2 => ServerStatus::Stopped,
            3 => ServerStatus::Error,
            4 => ServerStatus::Starting,
            5 => ServerStatus::Stopping,
            6 => ServerStatus::Crashed,
            7 => ServerStatus::Hanging, // Added Hanging status
            _ => ServerStatus::Idle,
        }
    }
}

impl From<ServerStatus> for u8 {
    fn from(status: ServerStatus) -> Self {
        match status {
            ServerStatus::Idle => 0,
            ServerStatus::Running => 1,
            ServerStatus::Stopped => 2,
            ServerStatus::Error => 3,
            ServerStatus::Starting => 4,
            ServerStatus::Stopping => 5,
            ServerStatus::Crashed => 6,
            ServerStatus::Hanging => 7, // Added Hanging status
        }
    }
}

impl Encode<'_, MySql> for ServerStatus {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        let value: u8 = self.clone().into();
        <u8 as Encode<MySql>>::encode_by_ref(&value, buf)
    }
}

impl<'r> Decode<'r, MySql> for ServerStatus {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let int_value = <u8 as Decode<MySql>>::decode(value)?;
        Ok(ServerStatus::from(int_value))
    }
}

impl Type<MySql> for ServerStatus {
    fn type_info() -> MySqlTypeInfo {
        <u8 as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <u8 as Type<MySql>>::compatible(ty)
    }
}

impl<'de> Deserialize<'de> for ServerStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                      where
                          D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ServerStatus::from(s))
    }
}