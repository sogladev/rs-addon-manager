use std::path::{Path, PathBuf};

use regex::Regex;

/// Returns true if the string matches a valid HTTP(S) git URL ending with .git
///
/// # Examples
///
/// ```
/// use addon_gui_lib::validate::is_valid_repo_url;
/// assert!(is_valid_repo_url("https://github.com/user/repo.git"));
/// assert!(is_valid_repo_url("http://github.com/user/repo.git"));
/// assert!(!is_valid_repo_url("git@github.com:user/repo.git"));
/// ```
#[tauri::command]
pub fn is_valid_repo_url(url: &str) -> bool {
    let re = Regex::new(r"^https?://.+\.git$").expect("Regex pattern should always compile");
    re.is_match(url)
}

/// Checks if the given path is a valid AddOns folder
///
/// # Examples
///
/// ```
/// use tempfile::tempdir;
/// use std::path::Path;
/// use addon_gui_lib::validate::is_valid_addons_folder;
///
/// let temp = tempdir().unwrap();
/// let interface_dir = temp.path().join("Interface");
/// let addons_dir = interface_dir.join("AddOns");
/// std::fs::create_dir_all(&addons_dir).unwrap();
/// assert!(is_valid_addons_folder(Path::new(&addons_dir)));
/// assert!(!is_valid_addons_folder(temp.path()));
/// assert!(!is_valid_addons_folder(&interface_dir));
/// ```
pub fn is_valid_addons_folder(path: &Path) -> bool {
    let dir_name = path.file_name().and_then(|n| n.to_str());
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());
    dir_name == Some("AddOns") && parent_name == Some("Interface")
}

#[tauri::command]
pub fn is_valid_addons_folder_str(path: &str) -> bool {
    let path = Path::new(path);
    is_valid_addons_folder(path)
}

/// Ensures the `.addonmanager` directory exists in the given base directory.
/// Returns the path to the manager directory.
///
/// # Examples
///
/// ```
/// let temp = tempfile::tempdir().unwrap();
/// let path = addon_gui_lib::validate::ensure_manager_dir(temp.path()).unwrap();
/// assert!(path.exists());
/// assert!(path.ends_with(".addonmanager"));
/// ```
pub fn ensure_manager_dir(base_dir: &Path) -> Result<PathBuf, String> {
    if !base_dir.is_dir() {
        return Err("Game path does not exist".to_string());
    }

    let manager_dir = base_dir.join(".addonmanager");
    if !manager_dir.exists() {
        std::fs::create_dir(&manager_dir)
            .map_err(|e| format!("Failed to create manager dir: {e}"))?;
    }

    Ok(manager_dir)
}
