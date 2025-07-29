use std::path::Path;

use regex::Regex;

/// Returns true if the string matches a valid HTTP(S) git URL ending with .git
///
/// # Examples
///
/// ```rust
/// use addon_gui_lib::validate::is_valid_repo_url;
/// assert!(is_valid_repo_url("https://github.com/user/repo.git"));
/// assert!(is_valid_repo_url("http://github.com/user/repo.git"));
/// assert!(!is_valid_repo_url("git@github.com:user/repo.git"));
/// ```
#[tauri::command]
pub fn is_valid_repo_url(url: &str) -> bool {
    let re = Regex::new(r"^https?://.+\.git$")
        .expect("Regex pattern should always compile");
    re.is_match(url)
}

#[tauri::command]
pub fn is_valid_addons_folder_tauri_str(path: &str) -> bool {
    let path = Path::new(path);
    is_valid_addons_folder_tauri(path)
}

pub fn is_valid_addons_folder_tauri(path: &Path) -> bool {
    let dir_name = path.file_name().and_then(|n| n.to_str());
    let parent_name = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
    dir_name == Some("AddOns") && parent_name == Some("Interface")
}