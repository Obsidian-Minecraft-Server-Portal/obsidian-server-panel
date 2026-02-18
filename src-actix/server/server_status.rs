use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::Type)]
#[repr(i32)]
pub enum ServerStatus {
    Idle = 0,
    Running = 1,
    Stopped = 2,
    Error = 3,
    Starting = 4,
    Stopping = 5,
    Crashed = 6,
    Hanging = 7,
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

impl<'de> Deserialize<'de> for ServerStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                      where
                          D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ServerStatus::from(s))
    }
}