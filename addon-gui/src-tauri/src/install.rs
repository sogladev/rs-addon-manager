use std::path::{Path, PathBuf};

use crate::validate;

#[tauri::command]
pub fn install_addon(url: &str, dir: &str) -> Result<(), String> {
    let dir = Path::new(dir);
    let manager_dir = ensure_manager_dir(dir)?;
    // ... your logic ...
    Ok(())
}

/// Ensures the `.addonmanager` directory exists in the given base directory.
/// Returns the path to the manager directory.
///
/// # Examples
///
/// ```
/// let temp = tempfile::tempdir().unwrap();
/// let path = addon_gui_lib::install::ensure_manager_dir(temp.path()).unwrap();
/// assert!(path.exists());
/// assert!(path.ends_with(".addonmanager"));
/// ```
pub fn ensure_manager_dir(base_dir: &Path) -> Result<PathBuf, String> {
    if !base_dir.is_dir() {
        return Err("Game path does not exist".to_string());
    }

    if !validate::is_valid_addons_folder(&base_dir) {
        return Err("Please select a valid AddOns folder (it should be named 'AddOns' and be inside an 'Interface' directory).".to_string());
    }

    let manager_dir = base_dir.join(".addonmanager");
    if !manager_dir.exists() {
        std::fs::create_dir(&manager_dir)
            .map_err(|e| format!("Failed to create manager dir: {}", e))?;
    }

    Ok(manager_dir)
}
