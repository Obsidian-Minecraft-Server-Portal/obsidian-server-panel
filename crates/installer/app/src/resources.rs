use log::warn;

/// Loads the application icon from embedded resources
pub fn load_app_icon() -> Option<slint::Image> {
    let icon_bytes = include_bytes!("../res/icon.ico");

    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let width = rgba.width();
            let height = rgba.height();
            let pixel_buffer = slint::SharedPixelBuffer::clone_from_slice(
                rgba.as_raw(),
                width,
                height,
            );
            Some(slint::Image::from_rgba8(pixel_buffer))
        }
        Err(e) => {
            warn!("Failed to load application icon: {}", e);
            None
        }
    }
}
