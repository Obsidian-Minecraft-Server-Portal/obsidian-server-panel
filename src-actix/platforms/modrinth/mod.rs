mod modrinth_endpoint;

use std::sync::OnceLock;

pub use modrinth_endpoint::configure;

static CLIENT: OnceLock<modrinth::ModrinthClient> = OnceLock::new();

/// Returns a reference to the shared Modrinth API client.
pub fn get_client() -> &'static modrinth::ModrinthClient {
    CLIENT.get_or_init(modrinth::ModrinthClient::new)
}
