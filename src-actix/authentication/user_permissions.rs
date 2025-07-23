use enumflags2::{bitflags, BitFlags};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionFlag {
    None = 0b0000_0000_0000_0001,
    Admin = 0b0000_0000_0000_0010,
    CreateServer = 0b0000_0000_0000_0100,
    OperateServer = 0b0000_0000_0000_1000,
    CreateBackup = 0b0000_0000_0001_0000,
    RestoreBackup = 0b0000_0000_0010_0000,
    DeleteBackups = 0b0000_0000_0100_0000,
    UploadFiles = 0b0000_0000_1000_0000,
    DeleteFiles = 0b0000_0001_0000_0000,
    CreateFiles = 0b0000_0010_0000_0000,
    ModifyFiles = 0b0000_0100_0000_0000,
    ViewUsers = 0b0000_1000_0000_0000,
    ManageUsers = 0b0001_0000_0000_0000,
    ManagePermissions = 0b0010_0000_0000_0000,
    ManageSettings = 0b0100_0000_0000_0000,
}

impl PermissionFlag {
    pub fn values() -> Vec<Self> {
        vec![
            Self::None,
            Self::Admin,
            Self::CreateServer,
            Self::OperateServer,
            Self::CreateBackup,
            Self::RestoreBackup,
            Self::DeleteBackups,
            Self::UploadFiles,
            Self::DeleteFiles,
            Self::CreateFiles,
            Self::ModifyFiles,
            Self::ViewUsers,
            Self::ManageUsers,
            Self::ManagePermissions,
            Self::ManageSettings,
        ]
    }

    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Admin,
            2 => Self::CreateServer,
            3 => Self::OperateServer,
            4 => Self::CreateBackup,
            5 => Self::RestoreBackup,
            6 => Self::DeleteBackups,
            7 => Self::UploadFiles,
            8 => Self::DeleteFiles,
            9 => Self::CreateFiles,
            10 => Self::ModifyFiles,
            11 => Self::ViewUsers,
            12 => Self::ManageUsers,
            13 => Self::ManagePermissions,
            14 => Self::ManageSettings,
            _ => Self::None,
        }
    }
    pub fn to_u16(self) -> u16 {
        match self {
            PermissionFlag::None => 0,
            PermissionFlag::Admin => 1,
            PermissionFlag::CreateServer => 2,
            PermissionFlag::OperateServer => 3,
            PermissionFlag::CreateBackup => 4,
            PermissionFlag::RestoreBackup => 5,
            PermissionFlag::DeleteBackups => 6,
            PermissionFlag::UploadFiles => 7,
            PermissionFlag::DeleteFiles => 8,
            PermissionFlag::CreateFiles => 9,
            PermissionFlag::ModifyFiles => 10,
            PermissionFlag::ViewUsers => 11,
            PermissionFlag::ManageUsers => 12,
            PermissionFlag::ManagePermissions => 13,
            PermissionFlag::ManageSettings => 14,
        }
    }
    pub fn from_u64(value: u64) -> Self {
        Self::from_u16(value.try_into().unwrap())
    }
}

impl Display for PermissionFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for PermissionFlag {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "none" => Self::None,
            "admin" => Self::Admin,
            "createserver" => Self::CreateServer,
            "operateserver" => Self::OperateServer,
            "createbackup" => Self::CreateBackup,
            "restorebackup" => Self::RestoreBackup,
            "deletebackups" => Self::DeleteBackups,
            "uploadfiles" => Self::UploadFiles,
            "deletefiles" => Self::DeleteFiles,
            "createfiles" => Self::CreateFiles,
            "modifyfiles" => Self::ModifyFiles,
            "viewusers" => Self::ViewUsers,
            "manageusers" => Self::ManageUsers,
            "managepermissions" => Self::ManagePermissions,
            "managesettings" => Self::ManageSettings,
            _ => Self::None,
        }
    }
}

impl From<String> for PermissionFlag {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<u16> for PermissionFlag {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl From<PermissionFlag> for u16 {
    fn from(value: PermissionFlag) -> Self {
        value.to_u16()
    }
}

impl Serialize for PermissionFlag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2)).expect("Failed to serialize map");
        map.serialize_entry("id", &self.to_u16()).expect("Failed to serialize id");
        map.serialize_entry("name", &self.to_string()).expect("Failed to serialize name");
        map.end()
    }
}

impl<'de> Deserialize<'de> for PermissionFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct PermissionFlagVisitor;

        impl<'de> Visitor<'de> for PermissionFlagVisitor {
            type Value = PermissionFlag;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a permission flag as u16, string, or object with id/name fields")
            }

            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PermissionFlag::from(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PermissionFlag::from_u64(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PermissionFlag::from(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PermissionFlag::from(value))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut id: Option<u64> = None;
                let mut name: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                if let Some(id) = id {
                    Ok(PermissionFlag::from_u64(id))
                } else if let Some(name) = name {
                    Ok(PermissionFlag::from(name))
                } else {
                    Err(de::Error::missing_field("id or name"))
                }
            }
        }

        deserializer.deserialize_any(PermissionFlagVisitor)
    }
}