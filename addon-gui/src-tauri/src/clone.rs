use git2::Repository;

pub fn clone_git_repo(url: &str, path: std::path::PathBuf) -> Result<git2::Repository, git2::Error> {
    Repository::clone(url, path)
}
