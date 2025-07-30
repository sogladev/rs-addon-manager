use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;

/// Clones a git repository into the given base path.
///
/// # Example
///
/// ```
/// use tempfile::tempdir;
/// use std::fs;
/// use std::path::Path;
/// use addon_gui_lib::clone::clone_git_repo;
///
/// let temp = tempdir().unwrap();
/// let base_path = temp.path().to_path_buf();
/// let url = "https://github.com/sogladev/addon-335-train-all-button.git";
/// let repo = clone_git_repo(url, base_path.clone(), |progress, total| { println!("progress: {}/{}", progress, total); }).unwrap();
/// let repo_dir = base_path.join("addon-335-train-all-button");
/// assert!(repo_dir.exists());
/// assert!(repo_dir.join(".git").is_dir());
/// assert!(repo_dir.join("TrainerButton").is_dir());
/// ```
pub fn clone_git_repo<F>(
    url: &str,
    base_path: PathBuf,
    mut progress: F,
) -> Result<Repository, git2::Error>
where
    F: FnMut(usize, usize) + Send + 'static,
{
    let repo_name = url
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .ok_or_else(|| git2::Error::from_str("Invalid repo URL"))?;
    let target_path = base_path.join(repo_name);

    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(move |stats| {
        progress(stats.received_objects(), stats.total_objects());
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    builder.clone(url, &target_path)
}
