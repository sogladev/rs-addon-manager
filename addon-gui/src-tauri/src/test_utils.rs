use std::path::PathBuf;
use tempfile::tempdir;

/// Helper to create Interface/AddOns structure in a temp dir
pub fn setup_addons_dir() -> (tempfile::TempDir, PathBuf) {
    let temp = tempdir().unwrap();
    let interface_dir = temp.path().join("Interface");
    let addons_dir = interface_dir.join("AddOns");
    std::fs::create_dir_all(&addons_dir).unwrap();
    assert!(
        interface_dir.exists() && interface_dir.is_dir(),
        "Interface directory was not created"
    );
    assert!(
        addons_dir.exists() && addons_dir.is_dir(),
        "AddOns directory was not created"
    );
    (temp, addons_dir)
}

/// Helper to print a directory tree using the `tree` command
pub fn print_dir_tree(path: &str) {
    println!("Directory tree under {path}:");
    let output = std::process::Command::new("tree")
        .arg("-a") // include hidden files
        .arg(path)
        .output()
        .expect("failed to execute tree");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
