use std::path::{Path, PathBuf};

use tauri::Emitter;

use crate::addon_discovery::find_all_sub_addons;
use crate::addon_meta::{AddonManagerData, AddonMeta};
use crate::clone;
use crate::symlink;
use crate::validate;

pub fn install_addon_with_progress<F>(
    url: &str,
    dir: &str,
    progress: F,
) -> Result<AddonManagerData, String>
where
    F: FnMut(usize, usize) + Send + 'static,
{
    let dir = Path::new(dir);

    let manager_dir = ensure_manager_dir(dir)?;

    let repo = clone::clone_git_repo(url, manager_dir.clone(), progress)
        .map_err(|e| format!("Failed to clone repository from {url}: {e}"))?;
    let path = PathBuf::from(
        repo.workdir()
            .expect("Repository has no workdir. It should not be bare"),
    );

    let sub_addons = find_all_sub_addons(&path)?;

    let (owner, _) = clone::extract_owner_repo_from_url(url)?;
    let addon_meta = AddonMeta {
        repo_url: url.to_string(),
        owner,
        repo_name: path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown-repo>".to_string()),
        installed_ref: repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .map(|oid| oid.to_string()),
        branch: repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string())),
        installed_at: None,
        sub_addons: sub_addons.clone(),
    };

    // Load, upsert, and save using AddonManagerData methods
    let mut manager_data = AddonManagerData::load_from_manager_dir(&manager_dir)?;
    manager_data.upsert_addon(addon_meta.clone());
    manager_data
        .save_to_manager_dir(&manager_dir)
        .map_err(|e| format!("Failed to save metadata: {e}"))?;

    install_sub_addons(addon_meta, &path, dir);

    Ok(manager_data)
}

pub fn install_sub_addons(mut addon_meta: AddonMeta, repo_root: &Path, addons_dir: &Path) {
    let sub_addons = &addon_meta.sub_addons;

    // Create symlink for each sub-addon
    for sub in sub_addons {
        if !sub.enabled {
            continue; // Skip if not enabled
        }
        let symlink_name = &sub.name;
        let target_dir = if sub.dir == "." {
            repo_root.to_path_buf()
        } else {
            repo_root.join(&sub.dir)
        };
        let symlink_path = addons_dir.join(symlink_name);

        // @todo: Handle cases where an addon is already installed. For now, let's just overwrite
        if symlink_path.exists() {
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .ok();
        }

        // @todo: Handle warning for multiple names better
        if sub.names.len() > 1 {
            eprintln!(
                "Warning: Multiple possible names for sub-addon '{}': {:?}. Using '{}'.",
                sub.dir, sub.names, symlink_name
            );
        }

        if let Err(e) = symlink::create_symlink(&target_dir, &symlink_path) {
            eprintln!(
                "Failed to create symlink for '{symlink_name}': {} -> {} ({e})",
                target_dir.display(),
                symlink_path.display(),
            );
        }
    }
    addon_meta.installed_at = Some(chrono::Utc::now().to_rfc3339());
}

#[tauri::command]
pub fn install_addon(
    app_handle: tauri::AppHandle,
    url: &str,
    dir: &str,
) -> Result<AddonManagerData, String> {
    if !validate::is_valid_addons_folder_str(dir) {
        return Err("Please select a valid AddOns folder (it should be named 'AddOns' and be inside an 'Interface' directory).".to_string());
    }

    install_addon_with_progress(url, dir, move |progress, total| {
        println!("Cloning progress: {progress}/{total}");
        app_handle.emit("git-progress", (progress, total)).unwrap();
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
    use crate::addon_meta::SubAddon;
    use std::fs;

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

    #[test]
    fn test_install_sub_addons_symlink_creation() {
        let (_temp, addons_dir) = setup_addons_dir();
        let manager_dir = ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        // Simulate a repo directory with a sub-addon directory
        let repo_root = manager_dir.join("fakeowner").join("fakerepo");
        let sub_dir = repo_root.join("SubAddonDir");
        fs::create_dir_all(&sub_dir).expect("Failed to create sub-addon dir");

        // Create a fake AddonMeta with one sub-addon
        let sub_addon = SubAddon {
            name: "TestSymlink".to_string(),
            dir: "SubAddonDir".to_string(),
            enabled: true,
            names: vec!["TestSymlink".to_string()],
            toc_files: vec![],
        };
        let addon_meta = AddonMeta {
            repo_url: "https://github.com/fakeowner/fakerepo.git".to_string(),
            owner: "fakeowner".to_string(),
            repo_name: "fakerepo".to_string(),
            installed_ref: None,
            branch: None,
            installed_at: None,
            sub_addons: vec![sub_addon],
        };

        // Print before
        println!("Before install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Call install_sub_addons
        install_sub_addons(addon_meta.clone(), &repo_root, &addons_dir);

        // Print after
        println!("After install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Check if the symlink exists
        let symlink_path = addons_dir.join("TestSymlink");
        assert!(
            symlink_path.exists(),
            "Symlink was not created: {}",
            symlink_path.display()
        );

        // Optionally, check if the symlink points to the correct target
        #[cfg(unix)]
        {
            use std::fs;
            let target = fs::read_link(&symlink_path).expect("Failed to read symlink");
            assert!(
                target.ends_with("SubAddonDir"),
                "Symlink does not point to SubAddonDir"
            );
        }
    }
}
