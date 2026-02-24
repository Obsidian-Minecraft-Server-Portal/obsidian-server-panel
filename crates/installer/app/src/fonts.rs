use log::{info, warn};
use ttf_parser::{Face, name_id};

/// Embedded Minecraft fonts (TTF format - fully supported)
const MINECRAFT_BODY_FONT: &[u8] = include_bytes!("../res/fonts/Minecraft-Seven_v2.ttf");
const MINECRAFT_HEADER_FONT: &[u8] = include_bytes!("../res/fonts/Minecraft-Tenv2.ttf");

/// Extract font family name from TTF data
fn get_font_family_name(font_data: &[u8]) -> Option<String> {
    let face = Face::parse(font_data, 0).ok()?;

    // Try to get the font family name (name ID 1)
    for name in face.names() {
        if name.name_id == name_id::FAMILY && let Some(family_name) = name.to_string() {
                return Some(family_name);
        }
    }
    None
}

/// Loads and registers embedded fonts
///
/// Fonts are imported directly in the .slint file, so this function just logs
/// the detected font family names for debugging purposes.
pub fn load_embedded_fonts() -> Result<(), Box<dyn std::error::Error>> {
    info!("Minecraft fonts are imported in UI definition (app.slint)");

    // Extract and log the actual font family names for debugging
    if let Some(body_font_name) = get_font_family_name(MINECRAFT_BODY_FONT) {
        info!("Body font family name: '{}'", body_font_name);
    } else {
        warn!("Could not extract body font family name");
    }

    if let Some(header_font_name) = get_font_family_name(MINECRAFT_HEADER_FONT) {
        info!("Header font family name: '{}'", header_font_name);
    } else {
        warn!("Could not extract header font family name");
    }

    Ok(())
}


/// Returns the font family names to use in the UI
///
/// These are extracted from the TTF font metadata at runtime.
/// - Body font: "Minecraft Seven v2" (for main UI text)
/// - Header font: "Minecraft Ten v2" (for larger headers)
#[allow(dead_code)]
pub fn get_font_families() -> (String, String) {
    (
        get_font_family_name(MINECRAFT_BODY_FONT).unwrap_or_else(|| "Minecraft Seven v2".to_string()),
        get_font_family_name(MINECRAFT_HEADER_FONT).unwrap_or_else(|| "Minecraft Ten v2".to_string()),
    )
}
