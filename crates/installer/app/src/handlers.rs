use slint::{ComponentHandle, LogicalPosition};

/// Sets up all event handlers for the application UI
///
/// # Arguments
/// * `ui` - The main application UI instance
pub fn setup_handlers(ui: &crate::App) {
    // Handle exit button clicks
    ui.on_request_exit_app(move || {
        std::process::exit(0);
    });

    // Handle window minimize
    let ui_handle_minimize = ui.as_weak();
    ui.on_minimize_window(move || {
        if let Some(ui) = ui_handle_minimize.upgrade() {
            let window = ui.window();
            window.set_minimized(true);
        }
    });

    // Handle window dragging from title bar
    let ui_handle_drag = ui.as_weak();
    ui.on_drag_window(move |delta_x, delta_y| {
        if let Some(ui) = ui_handle_drag.upgrade() {
            let window = ui.window();
            let logical_pos = window.position().to_logical(window.scale_factor());
            window.set_position(LogicalPosition::new(
                logical_pos.x + delta_x,
                logical_pos.y + delta_y,
            ));
        }
    });
}
