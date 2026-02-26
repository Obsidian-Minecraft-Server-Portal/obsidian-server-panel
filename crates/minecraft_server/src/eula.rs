use crate::Result;
use std::path::Path;

const EULA_FILENAME: &str = "eula.txt";

/// Check whether the Minecraft EULA has been accepted in the given server directory.
pub fn is_eula_accepted(server_dir: &Path) -> bool {
    let eula_path = server_dir.join(EULA_FILENAME);
    if let Ok(content) = std::fs::read_to_string(eula_path) {
        content
            .lines()
            .any(|line| line.trim().eq_ignore_ascii_case("eula=true"))
    } else {
        false
    }
}

/// Accept the Minecraft EULA by writing `eula=true` to the server directory.
pub fn accept_eula(server_dir: &Path) -> Result<()> {
    let eula_path = server_dir.join(EULA_FILENAME);
    std::fs::write(
        eula_path,
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\neula=true\n",
    )?;
    Ok(())
}
