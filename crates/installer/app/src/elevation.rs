/// Windows elevation and admin privilege checking
///
/// This module provides functionality to check if the application is running
/// with administrator privileges and to request elevation via UAC if needed.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HANDLE, HWND};
#[cfg(target_os = "windows")]
use windows::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::ShellExecuteW;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;
#[cfg(target_os = "windows")]
use windows::core::HSTRING;

/// Check if the current process is running with administrator privileges
#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = HANDLE::default();

        // Open the process token
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;

        // Get token elevation information
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        if result.is_err() {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

/// Request elevation by relaunching the application with administrator privileges
///
/// This will show the Windows UAC prompt. Returns true if elevation was requested successfully.
#[cfg(target_os = "windows")]
pub fn request_elevation() -> anyhow::Result<()> {
    request_elevation_with_args_string(None)
}

/// Request elevation with specific command line arguments
#[cfg(target_os = "windows")]
pub fn request_elevation_with_args_string(args: Option<&str>) -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;

    unsafe {
        let operation = HSTRING::from("runas");
        let file = HSTRING::from(exe_path.to_string_lossy().as_ref());

        let result = if let Some(args_str) = args {
            let parameters = HSTRING::from(args_str);
            ShellExecuteW(
                Some(HWND::default()),
                &operation,
                &file,
                &parameters,
                None,
                SW_NORMAL,
            )
        } else {
            ShellExecuteW(
                Some(HWND::default()),
                &operation,
                &file,
                None,
                None,
                SW_NORMAL,
            )
        };

        // ShellExecuteW returns > 32 on success
        if result.0 as i32 > 32 {
            // Successfully launched elevated instance, we should exit this one
            std::process::exit(0);
        } else {
            anyhow::bail!("Failed to request elevation. Error code: {}", result.0 as i32);
        }
    }
}

/// Request elevation with CLI args structure
#[cfg(target_os = "windows")]
pub fn request_elevation_with_args(args: &crate::cli::CliArgs) -> anyhow::Result<()> {
    let args_string = args.to_args_string();
    log::debug!("Requesting elevation with args string: '{}'", args_string);
    if args_string.is_empty() {
        request_elevation_with_args_string(None)
    } else {
        request_elevation_with_args_string(Some(&args_string))
    }
}

/// Non-Windows platforms always return true (no elevation needed)
#[cfg(not(target_os = "windows"))]
pub fn is_elevated() -> bool {
    true
}

/// Non-Windows platforms don't need elevation
#[cfg(not(target_os = "windows"))]
pub fn request_elevation() -> anyhow::Result<()> {
    Ok(())
}

/// Non-Windows platforms don't need elevation with args
#[cfg(not(target_os = "windows"))]
pub fn request_elevation_with_args(_args: &crate::cli::CliArgs) -> anyhow::Result<()> {
    Ok(())
}
