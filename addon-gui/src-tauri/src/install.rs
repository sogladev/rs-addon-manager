use std::path::{Path, PathBuf};

use tauri::Emitter;

use crate::addon_discovery::find_all_sub_addons;
use crate::addon_meta::{AddonMeta, SubAddon};
use crate::clone::clone_git_repo;
use crate::validate;

pub fn install_addon_with_progress<F>(url: &str, dir: &str, progress: F) -> Result<(), String>
where
    F: FnMut(usize, usize) + Send + 'static,
{
    let dir = Path::new(dir);

    let manager_dir = ensure_manager_dir(dir)?;
    let repo = clone_git_repo(url, manager_dir.clone(), progress)
        .map_err(|e| format!("Failed to clone repository from {url}: {e}"))?;
    let path = PathBuf::from(
        repo.workdir()
            .expect("Repository has no workdir. It should not be bare"),
    );

    let sub_addons = find_all_sub_addons(&path)?;

    // Fill AddonMeta (basic info, you may want to extract more fields)
    let addon_meta = AddonMeta {
        repo_url: url.to_string(),
        owner: "unknown".to_string(), // TODO: parse from url if needed
        repo: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        installed_ref: None, // TODO: fill with commit hash
        installed_at: Some(chrono::Utc::now().to_rfc3339()),
        sub_addons,
        notes: None,
    };

    // Mark as installed by adding to AddonManagerData (this could be loaded from file, here we just show the logic)
    // In real use, you would load, update, and save the AddonManagerData
    // let mut manager_data = AddonManagerData { addons: vec![] };
    // manager_data.addons.push(addon_meta);
    // manager_data.save_to_path(".addonmanager/addons.toml")?;

    Ok(())
}

#[tauri::command]
pub fn install_addon(app_handle: tauri::AppHandle, url: &str, dir: &str) -> Result<(), String> {
    let app_handle_clone = app_handle.clone();

    if !validate::is_valid_addons_folder_str(dir) {
        return Err("Please select a valid AddOns folder (it should be named 'AddOns' and be inside an 'Interface' directory).".to_string());
    }

    install_addon_with_progress(url, dir, move |progress, total| {
        println!("Cloning progress: {progress}/{total}");
        app_handle_clone
            .emit("git-progress", (progress, total))
            .unwrap();
    })
}

/// Ensures the `.addonmanager` directory exists in the given base directory.
/// Returns the path to the manager directory.
///
/// # Examples
///
/// ```
/// let temp = tempfile::tempdir().unwrap();
/// let path = addon_gui_lib::install::ensure_manager_dir(temp.path()).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Helper to create Interface/AddOns structure in a temp dir
    fn setup_addons_dir() -> (tempfile::TempDir, std::path::PathBuf) {
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
    fn print_dir_tree(path: &str) {
        println!("Directory tree under {path}:");
        let output = std::process::Command::new("tree")
            .arg("-a") // include hidden files
            .arg(path)
            .output()
            .expect("failed to execute tree");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    #[test]
    fn test_install_addon_with_progress() {
        let (_temp, addons_dir) = setup_addons_dir();

        let url = "https://github.com/sogladev/addon-335-train-all-button.git";
        let addons_dir_str = addons_dir.to_str().unwrap();

        let result = ensure_manager_dir(&addons_dir);
        println!("Directory tree under AddOns after ensure_manager_dir:");
        print_dir_tree(addons_dir_str);
        assert!(result.is_ok(), "ensure_manager_dir failed: {:?}", result);

        let result = install_addon_with_progress(url, addons_dir_str, move |progress, total| {
            println!("Cloning progress: {}/{}", progress, total);
        });
        println!("Directory tree under AddOns after install_addon:");
        print_dir_tree(addons_dir_str);
        assert!(result.is_ok(), "install_addon failed: {:?}", result);

        let manager_dir = addons_dir.join(".addonmanager");
        assert!(
            manager_dir.exists() && manager_dir.is_dir(),
            ".addonmanager directory was not created"
        );

        let repo_dir = manager_dir
            .join("sogladev")
            .join("addon-335-train-all-button");
        assert!(
            repo_dir.exists() && repo_dir.is_dir(),
            "Repository was not cloned to the manager directory"
        );
    }
}
