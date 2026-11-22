use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Loader {
	Vanilla,
	Fabric,
	Quilt,
	Forge,
	NeoForge,
	Paper,
	Purpur,
	Spigot,
	CraftBukkit,
	Custom,
}
