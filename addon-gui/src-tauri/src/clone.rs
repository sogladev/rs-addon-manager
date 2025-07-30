use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;
use url::Url;

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
/// let repo_dir = base_path.join("sogladev").join("addon-335-train-all-button");
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
    let parsed_url = Url::parse(url).map_err(|_| git2::Error::from_str("Invalid URL"))?;
    let mut path_segments = parsed_url
        .path_segments()
        .ok_or(git2::Error::from_str("cannot be base"))?;
    let owner = path_segments
        .next()
        .ok_or(git2::Error::from_str("missing owner"))?;
    let repo = path_segments
        .next()
        .ok_or(git2::Error::from_str("missing repo"))?
        .trim_end_matches(".git");

    let target_path = base_path.join(owner).join(repo);

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
