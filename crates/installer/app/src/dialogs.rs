use std::path::PathBuf;

/// Opens a folder browser dialog and returns the selected path
///
/// # Returns
/// * `Option<PathBuf>` - The selected folder path, or None if cancelled
#[cfg(target_os = "windows")]
pub fn browse_folder() -> Option<PathBuf> {
    use windows::core::{HSTRING, PWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileDialog, IShellItem, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };

    unsafe {
        // Initialize COM
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return None;
        }

        let result = (|| -> Option<PathBuf> {
            // Create file dialog
            let dialog: IFileDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;

            // Set options to pick folders
            let mut options = dialog.GetOptions().ok()?;
            options |= FOS_PICKFOLDERS;
            dialog.SetOptions(options).ok()?;

            // Set title
            dialog.SetTitle(&HSTRING::from("Select Installation Folder")).ok()?;

            // Show dialog
            if dialog.Show(None).is_err() {
                return None;
            }

            // Get result
            let item: IShellItem = dialog.GetResult().ok()?;
            let path_pwstr: PWSTR = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;

            // Convert to PathBuf
            let path_str = path_pwstr.to_string().ok()?;
            Some(PathBuf::from(path_str))
        })();

        // Cleanup COM
        CoUninitialize();

        result
    }
}

/// Opens a folder browser dialog (non-Windows placeholder)
#[cfg(not(target_os = "windows"))]
pub fn browse_folder() -> Option<PathBuf> {
    // On non-Windows platforms, we would use a different dialog library
    // For now, return None as this is primarily a Windows application
    None
}
