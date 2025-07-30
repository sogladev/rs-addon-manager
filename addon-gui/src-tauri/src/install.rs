use std::path::{Path, PathBuf};

use tauri::Emitter;

use crate::addon_discovery::find_all_sub_addons;
use crate::addon_meta::{AddonManagerData, AddonMeta};
use crate::clone;

pub struct InstallReporter<'a> {
    pub progress: Box<dyn FnMut(usize, usize) + Send + 'a>,
    pub status: Box<dyn FnMut(&str) + Send + 'a>,
    pub warning: Box<dyn FnMut(&str) + Send + 'a>,
    pub error: Box<dyn FnMut(&str) + Send + 'a>,
}

pub fn install_addon_with_progress<'a>(
    url: &str,
    dir: &str,
    mut rep: InstallReporter<'a>,
) -> Result<AddonManagerData, String>
where
    'a: 'static,
{
    let dir = Path::new(dir);

    (rep.status)("Starting addon installation...");

    let manager_dir = match ensure_manager_dir(dir) {
        Ok(m) => m,
        Err(e) => {
            (rep.error)(&format!("Failed to ensure manager dir: {e}"));
            return Err(e);
        }
    };

    (rep.status)("Cloning repository...");
    let repo = match clone::clone_git_repo(url, manager_dir.clone(), &mut rep.progress) {
        Ok(r) => r,
        Err(e) => {
            (rep.error)(&format!("Failed to clone repository from {url}: {e}"));
            return Err(format!("Failed to clone repository from {url}: {e}"));
        }
    };
    let path = PathBuf::from(
        repo.workdir()
            .expect("Repository has no workdir. It should not be bare"),
    );

    (rep.status)("Discovering sub-addons...");
    let sub_addons = match find_all_sub_addons(&path) {
        Ok(s) => s,
        Err(e) => {
            (rep.error)(&format!("Failed to discover sub-addons: {e}"));
            return Err(format!("Failed to discover sub-addons: {e}"));
        }
    };

    let (owner, _) = match clone::extract_owner_repo_from_url(url) {
        Ok(o) => o,
        Err(e) => {
            (rep.error)(&format!("Failed to extract owner/repo from url: {e}"));
            return Err(format!("Failed to extract owner/repo from url: {e}"));
        }
    };
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

    (rep.status)("Saving addon metadata...");
    let mut manager_data = match AddonManagerData::load_from_manager_dir(&manager_dir) {
        Ok(m) => m,
        Err(e) => {
            (rep.error)(&format!("Failed to load manager data: {e}"));
            return Err(format!("Failed to load manager data: {e}"));
        }
    };
    manager_data.upsert_addon(addon_meta.clone());
    if let Err(e) = manager_data.save_to_manager_dir(&manager_dir) {
        (rep.error)(&format!("Failed to save metadata: {e}"));
        return Err(format!("Failed to save metadata: {e}"));
    }

    (rep.status)("Installing sub-addons (symlinking)...");
    install_sub_addons_with_feedback(addon_meta, &path, dir, &mut rep);

    (rep.status)("Addon installation complete.");
    Ok(manager_data)
}

pub fn install_sub_addons_with_feedback<'a>(
    mut addon_meta: AddonMeta,
    repo_root: &Path,
    addons_dir: &Path,
    rep: &mut InstallReporter<'a>,
) {
    let sub_addons = &addon_meta.sub_addons;

    for sub in sub_addons {
        if !sub.enabled {
            continue;
        }
        let symlink_name = &sub.name;
        let target_dir = if sub.dir == "." {
            repo_root.to_path_buf()
        } else {
            repo_root.join(&sub.dir)
        };
        let symlink_path = addons_dir.join(symlink_name);

        if symlink_path.exists() {
            (rep.status)(&format!(
                "Removing existing symlink or directory: {}",
                symlink_path.display()
            ));
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .ok();
        }

        if sub.names.len() > 1 {
            (rep.warning)(&format!(
                "Multiple possible names for sub-addon '{}': {:?}. Using '{}'.",
                sub.dir, sub.names, symlink_name
            ));
        }

        (rep.status)(&format!(
            "Creating symlink for '{}': {} -> {}",
            symlink_name,
            target_dir.display(),
            symlink_path.display()
        ));
        if let Err(e) = crate::symlink::create_symlink(&target_dir, &symlink_path) {
            (rep.error)(&format!(
                "Failed to create symlink for '{symlink_name}': {} -> {} ({e})",
                target_dir.display(),
                symlink_path.display(),
            ));
        }
    }
    addon_meta.installed_at = Some(chrono::Utc::now().to_rfc3339());
}

#[tauri::command]
pub async fn install_addon(
    app_handle: tauri::AppHandle,
    url: String,
    dir: String,
) -> Result<AddonManagerData, String> {
    // No need to validate the AddOns folder here, as it is already done in the frontend.

    // Move the blocking work into spawn_blocking, but keep async code outside.
    let app_handle_clone = app_handle.clone();
    let url_clone = url.clone();
    let dir_clone = dir.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let reporter = InstallReporter {
            progress: Box::new({
                let app_handle = app_handle_clone.clone();
                move |progress, total| app_handle.emit("git-progress", (progress, total)).unwrap()
            }),
            status: Box::new({
                let app_handle = app_handle_clone.clone();
                move |msg| app_handle.emit("install-step", msg).unwrap()
            }),
            warning: Box::new({
                let app_handle = app_handle_clone.clone();
                move |msg| app_handle.emit("install-warning", msg).unwrap()
            }),
            error: Box::new({
                let app_handle = app_handle_clone;
                move |msg| app_handle.emit("install-error", msg).unwrap()
            }),
        };

        install_addon_with_progress(&url_clone, &dir_clone, reporter)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
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

        println!("Before install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        install_sub_addons(addon_meta.clone(), &repo_root, &addons_dir);

        println!("After install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        let symlink_path = addons_dir.join("TestSymlink");
        assert!(
            symlink_path.exists(),
            "Symlink was not created: {}",
            symlink_path.display()
        );

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
