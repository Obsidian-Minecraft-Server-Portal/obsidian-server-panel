use enumflags2::bitflags;
use serde::{Deserialize, Serialize};

#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
