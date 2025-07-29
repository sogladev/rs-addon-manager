use git2::Repository;

#[test]
fn test_clone_git_repo() {
    let url = "https://github.com/sogladev/addon-335-train-all-button";
    let tmp_dir = tempfile::tempdir().expect("create temp dir");
    let repo_path = tmp_dir.path().join("libgit2");

    // Attempt to clone the repo
    let result = Repository::clone(url, &repo_path);
    assert!(result.is_ok(), "Failed to clone repo: {:?}", result.err());

    // Check that .git exists
    assert!(repo_path.join(".git").exists(), ".git directory missing after clone");
}
