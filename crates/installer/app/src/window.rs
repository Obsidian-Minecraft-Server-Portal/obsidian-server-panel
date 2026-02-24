use slint::{LogicalPosition, Window};

/// Centers the window on the screen
///
/// # Arguments
/// * `window` - The Slint window to center
/// * `width` - Window width in pixels
/// * `height` - Window height in pixels
#[cfg(target_os = "windows")]
pub fn center_window(window: &Window, width: f32, height: f32) {
    // Get screen dimensions using Windows API
    let (screen_width, screen_height) = unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
        };
        (
            GetSystemMetrics(SM_CXSCREEN) as f32,
            GetSystemMetrics(SM_CYSCREEN) as f32,
        )
    };

    // Calculate center position
    let center_x = (screen_width - width) / 2.0;
    let center_y = (screen_height - height) / 2.0;

    window.set_position(LogicalPosition::new(center_x, center_y));
}

/// Applies Windows 11 rounded corners to the window
///
/// This function uses the DWM (Desktop Window Manager) API to apply rounded corners
/// to the window. This feature is only available on Windows 11 Build 22000 or later.
///
/// # Arguments
/// * `window` - The Slint window to apply rounded corners to
#[cfg(target_os = "windows")]
pub fn apply_rounded_corners(window: &Window) {
    use log::info;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, FindWindowW,
    };
    use windows::core::{BOOL, PCWSTR};

    // DWMWA_WINDOW_CORNER_PREFERENCE = 33 (Windows 11 Build 22000+)
    const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(33);

    // DWM_WINDOW_CORNER_PREFERENCE values:
    // 0 = DWMWCP_DEFAULT (let system decide)
    // 1 = DWMWCP_DONOTROUND (never round)
    // 2 = DWMWCP_ROUND (round if appropriate)
    // 3 = DWMWCP_ROUNDSMALL (round with small radius)
    const DWMWCP_ROUND: i32 = 2;

    info!("Attempting to apply Windows 11 rounded corners...");

    unsafe {
        let current_pid = std::process::id();
        let mut found_hwnd: Option<HWND> = None;

        // Try to find window by title first
        let title: Vec<u16> = "Obsidian Installer\0".encode_utf16().collect();
        if let Ok(hwnd_by_title) = FindWindowW(PCWSTR::null(), PCWSTR::from_raw(title.as_ptr())) {
            if !hwnd_by_title.is_invalid() {
                found_hwnd = Some(hwnd_by_title);
                info!("Found window by title");
            }
        }

        if found_hwnd.is_none() {
            // Enumerate all windows to find our window
            info!("Enumerating windows to find application window...");
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut found_hwnd as *mut _ as isize),
            );
        }

        if let Some(hwnd) = found_hwnd {
            info!("Applying rounded corners to HWND: {:?}", hwnd);
            // Apply rounded corners
            let result = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &DWMWCP_ROUND as *const _ as *const _,
                std::mem::size_of::<i32>() as u32,
            );

            if result.is_ok() {
                info!("Successfully applied rounded corners");
            } else {
                info!("Failed to apply rounded corners: {:?}", result);
            }
        } else {
            info!("Could not find application window");
        }
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let found_hwnd = lparam.0 as *mut Option<HWND>;
            let current_pid = std::process::id();
            let mut window_pid: u32 = 0;

            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

            if window_pid == current_pid && IsWindowVisible(hwnd).as_bool() {
                *found_hwnd = Some(hwnd);
                return false.into(); // Stop enumeration
            }

            true.into() // Continue enumeration
        }
    }
}

/// Centers the window on the screen (non-Windows platforms)
///
/// This is a placeholder implementation for non-Windows platforms.
/// On Linux/macOS, window positioning may be handled by the window manager.
#[cfg(not(target_os = "windows"))]
pub fn center_window(_window: &Window, _width: f32, _height: f32) {
    // On Linux/macOS, window positioning is typically managed by the window manager
    // This is a no-op for now
}

/// Applies rounded corners to the window (non-Windows platforms)
///
/// This is a no-op on non-Windows platforms as rounded corners are handled
/// by the compositor/window manager.
#[cfg(not(target_os = "windows"))]
pub fn apply_rounded_corners(_window: &Window) {
    // No-op on non-Windows platforms
}
