use regex::Regex;

/// Returns true if the string matches a valid HTTP(S) git URL ending with .git
///
/// # Examples
///
/// ```rust
/// use addon_gui_lib::validate::validate_repo_url;
/// assert!(validate_repo_url("https://github.com/user/repo.git"));
/// assert!(validate_repo_url("http://github.com/user/repo.git"));
/// assert!(!validate_repo_url("git@github.com:user/repo.git"));
/// ```
#[tauri::command]
pub fn validate_repo_url(url: &str) -> bool {
    let re = Regex::new(r"^https?://.+\.git$")
        .expect("Regex pattern should always compile");
    re.is_match(url)
}
