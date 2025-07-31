use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;
use url::Url;

/// Extracts the owner and repository name from a Git URL.
// This assumes the URL is in the format: https://github.com/owner/repo.git
/// ```
/// use addon_gui_lib::clone::extract_owner_repo_from_url;
/// let (owner, repo) = extract_owner_repo_from_url("https://github.com/owner/repo.git").unwrap();
/// assert!(owner == "owner");
/// assert!(repo == "repo");
/// ```
pub fn extract_owner_repo_from_url(url: &str) -> Result<(String, String), String> {
    let parsed_url = Url::parse(url).map_err(|e| e.to_string())?;
    let mut path_segments = parsed_url
        .path_segments()
        .ok_or_else(|| "cannot be base".to_string())?;
    let owner = path_segments
        .next()
        .ok_or_else(|| "missing owner".to_string())?;
    let repo = path_segments
        .next()
        .ok_or_else(|| "missing repo".to_string())?
        .trim_end_matches(".git");
    Ok((owner.to_string(), repo.to_string()))
}

/// Clones a git repository into the given base path.
///
/// # Example
///
/// ```
/// use tempfile::tempdir;
/// use addon_gui_lib::clone::clone_git_repo;
///
/// let temp = tempdir().unwrap();
/// let base_path = temp.path().to_path_buf();
/// let url = "https://github.com/sogladev/addon-335-train-all-button.git";
/// let repo = clone_git_repo(url, base_path.clone(), &mut |progress, total| { println!("progress: {progress}/{total}"); }).unwrap();
/// let repo_dir = base_path.join("sogladev").join("addon-335-train-all-button");
/// assert!(repo_dir.exists());
/// assert!(repo_dir.join(".git").is_dir());
/// assert!(repo_dir.join("TrainerButton").is_dir());
/// ```
pub fn clone_git_repo<F>(
    url: &str,
    base_path: PathBuf,
    progress: &mut F,
) -> Result<Repository, String>
where
    F: FnMut(usize, usize) + Send,
{
    let (owner, repo) = extract_owner_repo_from_url(url)?;

    let target_path = base_path.join(&owner).join(&repo);

    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(move |stats| {
        progress(stats.received_objects(), stats.total_objects());
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    builder
        .clone(url, &target_path)
        .map_err(|e| e.message().to_string())
}
