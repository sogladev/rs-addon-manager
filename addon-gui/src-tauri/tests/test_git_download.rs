use addon_gui_lib::clone;

#[test]
fn test_clone_git_repo() {
    let url = "https://github.com/sogladev/addon-335-train-all-button";
    let tmp_dir = tempfile::tempdir().expect("create temp dir");
    let repo_path = tmp_dir.path().join(".addonmanager");

    // Attempt to clone the repo
    let result = clone::clone_git_repo(url, repo_path.clone(), &mut |_, _| {});
    assert!(result.is_ok(), "Failed to clone repo: {:?}", result.err());

    // Check that .git exists
    assert!(
        repo_path
            .join("sogladev")
            .join("addon-335-train-all-button")
            .join(".git")
            .exists(),
        ".git directory missing after clone"
    );
}
