use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::clone::clone_git_repo;
use crate::validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubAddon {
    /// The relative path to the sub-addon directory inside the repo
    pub dir: String,
    /// Normalized base names after removing suffixes (from .toc).
    /// There should only be one name per sub-addon.
    pub names: Vec<String>,
    /// .toc file names
    pub toc_files: Vec<String>,
    /// Enabled
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonMeta {
    /// The git repository URL
    pub repo_url: String,
    /// The owner of the repository (e.g., GitHub username)
    pub owner: String,
    /// The repository name
    pub repo: String,
    /// The commit hash or tag that was installed
    pub installed_ref: Option<String>,
    /// Date/time of installation (ISO 8601)
    pub installed_at: Option<String>,
    /// All discovered sub-addons in this repo
    pub sub_addons: Vec<SubAddon>,
    /// Any additional notes or user comments
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonManagerData {
    pub addons: Vec<AddonMeta>,
}

impl AddonManagerData {
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let toml_str = toml::to_string_pretty(self).expect("Serialization failed");
        std::fs::write(path, toml_str)
    }
}

/// Removes known WoW version suffixes from .toc filenames, normalizing to "Name.toc".
///
/// https://wowpedia.fandom.com/wiki/TOC_format
/// Classic and retail versions of the game can be properly supported by including multiple TOC files in the same addon.
/// The client first searches for the respective suffix and otherwise falls back to AddonName.toc
///
/// _MainLine, _Cata, _Wrath, _TBC, _Vanilla,
/// -Cata, -WOTLKC, -BCC, -Classic
/// extra: _wotlk
///
/// # Examples
///
/// ```
/// use addon_gui_lib::install::remove_client_flavor_toc_suffixes;
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Mainline.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Cata.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Wrath.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_TBC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Vanilla.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Cata.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags-WOTLKC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_BCC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags-Classic.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_wotlk.toc"), "AdiBags.toc");
/// ```
pub fn remove_client_flavor_toc_suffixes(name: &str) -> String {
    // WoW client flavor suffixes (official and legacy)
    let suffixes = [
        "mainline.toc",
        "cataclysm.toc",
        "cata.toc",
        "wrath.toc",
        "tbc.toc",
        "vanilla.toc",
        "classic.toc",
        "bcc.toc",
        "wotlkc.toc",
        "wotlk.toc",
    ];
    let name_lower = name.to_ascii_lowercase();
    for suf in &suffixes {
        let dash_pattern = format!("-{suf}");
        let underscore_pattern = format!("_{suf}");
        if name_lower.ends_with(&dash_pattern) {
            let idx = name_lower.rfind(&dash_pattern).unwrap();
            return name[..idx].to_string() + ".toc";
        }
        if name_lower.ends_with(&underscore_pattern) {
            let idx = name_lower.rfind(&underscore_pattern).unwrap();
            return name[..idx].to_string() + ".toc";
        }
    }
    name.to_string()
}

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

    if !validate::is_valid_addons_folder(base_dir) {
        return Err("Please select a valid AddOns folder (it should be named 'AddOns' and be inside an 'Interface' directory).".to_string());
    }

    let manager_dir = base_dir.join(".addonmanager");
    if !manager_dir.exists() {
        std::fs::create_dir(&manager_dir)
            .map_err(|e| format!("Failed to create manager dir: {e}"))?;
    }

    Ok(manager_dir)
}

/// Finds all sub-addons by searching for .toc files in the root directory and immediate subdirectories only.
///
/// This function does NOT recursively walk all subdirectories to find .toc files.
/// It checks for .toc files in the root of the given path and in each immediate subdirectory (one level deep).
pub fn find_all_sub_addons(path: &PathBuf) -> Result<Vec<SubAddon>, String> {
    let mut sub_addons = Vec::new();

    // Helper to process a directory and collect .toc files
    fn collect_toc_files(dir: &Path) -> Result<Vec<String>, String> {
        let let_toc_files = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read dir: {e}"))?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension() == Some(OsStr::new("toc")) {
                    path.file_name().map(|f| f.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(let_toc_files)
    }

    // Process root directory
    let toc_files = collect_toc_files(path)?;
    if !toc_files.is_empty() {
        let names = toc_files
            .iter()
            .map(|toc| remove_client_flavor_toc_suffixes(toc))
            .collect();
        sub_addons.push(SubAddon {
            dir: ".".to_string(),
            toc_files,
            names,
            enabled: true,
        });
    }

    // Process immediate subdirectories
    sub_addons.extend(
        std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read repo dir: {e}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|sub_path| sub_path.is_dir())
            .filter_map(|sub_path| {
                let toc_files = collect_toc_files(&sub_path).ok()?;
                if toc_files.is_empty() {
                    return None;
                }
                let names = toc_files
                    .iter()
                    .map(|toc| remove_client_flavor_toc_suffixes(toc))
                    .collect();
                let dir_name = sub_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                Some(SubAddon {
                    dir: dir_name,
                    toc_files,
                    names,
                    enabled: true,
                })
            }),
    );
    Ok(sub_addons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    /// Helper to create Interface/AddOns structure in a temp dir
    fn setup_addons_dir() -> (tempfile::TempDir, std::path::PathBuf) {
        let temp = tempdir().unwrap();
        let interface_dir = temp.path().join("Interface");
        let addons_dir = interface_dir.join("AddOns");
        fs::create_dir_all(&addons_dir).unwrap();
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
        let output = Command::new("tree")
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
    fn test_find_all_sub_addons_single_toc_in_root() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        let toc_path = repo_dir.join("AdiBags.toc");
        std::fs::File::create(&toc_path).unwrap();

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert!(
            sub_addons.len() == 1,
            "Expected 1 sub_addon, found: {:?}",
            sub_addons
        );
        assert!(
            sub_addons[0].dir == ".",
            "Expected sub_addon dir to be '.', found: {}",
            sub_addons[0].dir
        );
        assert!(
            sub_addons[0].toc_files.len() == 1,
            "Expected 1 .toc file, found: {:?}",
            sub_addons[0].toc_files
        );
        assert!(
            sub_addons[0].names.len() == 1,
            "Expected 1 name, found: {:?}",
            sub_addons[0].names
        );
        assert!(
            sub_addons[0].names[0].contains(&"AdiBags".to_string()),
            "Expected sub_addon names to contain 'AdiBags', found: {:?}",
            sub_addons[0].names[0]
        );
    }

    #[test]
    fn test_find_all_sub_addons_multiple_subdirs_with_toc() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        // Create subdirectories
        let sub1 = repo_dir.join("AdiBags-ItemOverlayPlus");
        let sub2 = repo_dir.join("AdiBags_Bound");
        let sub3 = repo_dir.join("NoTocSubAddon");
        std::fs::create_dir_all(&sub1).unwrap();
        std::fs::create_dir_all(&sub2).unwrap();
        std::fs::create_dir_all(&sub3).unwrap();

        // Create .toc files in each sub-addon directory
        let toc1 = sub1.join("AdiBags-ItemOverlayPlus.toc");
        let toc2 = sub2.join("AdiBags_Bound.toc");
        std::fs::File::create(&toc1).unwrap();
        std::fs::File::create(&toc2).unwrap();

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert_eq!(
            sub_addons.len(),
            2,
            "Expected 2 sub_addons, found: {:?}",
            sub_addons
        );

        let mut found_dirs = sub_addons.iter().map(|s| s.dir.clone()).collect::<Vec<_>>();
        found_dirs.sort();
        assert_eq!(
            found_dirs,
            vec!["AdiBags-ItemOverlayPlus", "AdiBags_Bound"],
            "Unexpected sub_addon dirs: {:?}",
            found_dirs
        );

        for sub in &sub_addons {
            assert_eq!(
                sub.toc_files.len(),
                1,
                "Expected 1 .toc file in {:?}, found: {:?}",
                sub.dir,
                sub.toc_files
            );
            assert_eq!(
                sub.names.len(),
                1,
                "Expected 1 name in {:?}, found: {:?}",
                sub.dir,
                sub.names
            );
            assert!(
                sub.toc_files[0].ends_with(".toc"),
                "Expected .toc file, found: {:?}",
                sub.toc_files[0]
            );
        }
    }
}
