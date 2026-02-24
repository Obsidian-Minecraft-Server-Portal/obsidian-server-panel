mod curseforge_endpoint;

pub use curseforge_endpoint::configure;

use std::sync::OnceLock;

static CLIENT: OnceLock<curseforge::CurseForgeClient> = OnceLock::new();

const API_KEY: &str = "$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG";

pub fn get_client() -> &'static curseforge::CurseForgeClient {
    CLIENT.get_or_init(|| curseforge::CurseForgeClient::new(API_KEY))
}
