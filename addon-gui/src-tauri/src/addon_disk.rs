use std::path::Path;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, path::PathBuf};

use crate::clone;
use crate::symlink;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskAddOnsFolder {
    /// Absolute path to the AddOns directory
    pub path: String,
    pub is_valid: bool,
    /// All discovered addon repositories under .addonmanager
    pub repositories: Vec<DiskAddonRepository>,
    /// Error
    pub error: Option<String>,
}

impl DiskAddOnsFolder {
    /// Scan the AddOns directory on disk
    pub fn scan(path: &str) -> Result<Self, String> {
        let addons_path = Path::new(path);
        let is_valid = crate::validate::is_valid_addons_folder_str(path);
        let manager_dir = addons_path.join(".addonmanager");
        let mut repositories = Vec::new();
        if manager_dir.exists() {
            for repo_entry in std::fs::read_dir(&manager_dir)
                .map_err(|e| format!("Failed to read manager dir {}: {e}", manager_dir.display()))?
            {
                let repo_path = repo_entry.map_err(|e| e.to_string())?.path();
                if !repo_path.is_dir() {
                    continue;
                }
                let git_dir = repo_path.join(".git");
                if !git_dir.exists() || !git_dir.is_dir() {
                    println!("Skipping {}: .git directory not found", repo_path.display());
                    continue;
                }
                let mut disk_repo = create_disk_addon_repository(&repo_path)?;
                // Check which addons are actually symlinked in the AddOns directory
                check_addon_symlinks(&mut disk_repo.addons, addons_path);
                repositories.push(disk_repo);
            }
        }
        Ok(DiskAddOnsFolder {
            path: path.to_string(),
            is_valid,
            repositories,
            error: None,
        })
    }

    /// Scan the AddOns directory on disk (disk-only, no remote operations)
    pub fn scan_disk_only(path: &str) -> Result<Self, String> {
        let addons_path = Path::new(path);
        let is_valid = crate::validate::is_valid_addons_folder_str(path);
        let manager_dir = addons_path.join(".addonmanager");
        let mut repositories = Vec::new();
        if manager_dir.exists() {
            for repo_entry in std::fs::read_dir(&manager_dir)
                .map_err(|e| format!("Failed to read manager dir {}: {e}", manager_dir.display()))?
            {
                let repo_path = repo_entry.map_err(|e| e.to_string())?.path();
                if !repo_path.is_dir() {
                    continue;
                }
                let git_dir = repo_path.join(".git");
                if !git_dir.exists() || !git_dir.is_dir() {
                    // If .git does not exist or is not a directory, skip this folder
                    continue;
                }
                let mut disk_repo = create_disk_addon_repository_disk_only(&repo_path)?;
                // Check which addons are actually symlinked in the AddOns directory
                check_addon_symlinks(&mut disk_repo.addons, addons_path);
                repositories.push(disk_repo);
            }
        }
        Ok(DiskAddOnsFolder {
            path: path.to_string(),
            is_valid,
            repositories,
            error: None,
        })
    }

    /// With error
    pub fn default_with_error(path: &str, error: String) -> Self {
        DiskAddOnsFolder {
            path: path.to_string(),
            is_valid: crate::validate::is_valid_addons_folder_str(path),
            repositories: Vec::new(),
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Folder must contain a .toc file, sometimes multiple
pub struct DiskAddon {
    pub name: String,
    pub dir: String,
    pub names: Vec<String>,
    pub is_symlinked: bool, // true is symlink exists in AddOns
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Folder with a .git subdirectory
pub struct DiskAddonRepository {
    pub repo_url: String,
    pub repo_name: String,
    pub owner: String,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub repo_ref: Option<String>, // local HEAD commit hash or tag
    pub latest_ref: Option<String>,
    pub addons: Vec<DiskAddon>,
    pub readme: Option<String>,
}

/// Check if addons are symlinked in the AddOns directory
/// Updates the is_symlinked field for each addon
pub fn check_addon_symlinks(addons: &mut [DiskAddon], addons_dir: &Path) {
    for addon in addons {
        let path = addons_dir.join(&addon.name);
        addon.is_symlinked = symlink::is_addon_symlinked(path);
    }
}

/// Create a DiskAddonRepository from a repository path
/// This is used by both scan and install operations
pub fn create_disk_addon_repository(repo_path: &Path) -> Result<DiskAddonRepository, String> {
    create_disk_addon_repository_inner(repo_path, false)
}

/// Create a DiskAddonRepository from a repository path (disk-only, no remote operations)
/// This is used for fast disk-only scans
pub fn create_disk_addon_repository_disk_only(
    repo_path: &Path,
) -> Result<DiskAddonRepository, String> {
    create_disk_addon_repository_inner(repo_path, true)
}

fn create_disk_addon_repository_inner(
    repo_path: &Path,
    disk_only: bool,
) -> Result<DiskAddonRepository, String> {
    let repo = git2::Repository::open(repo_path)
        .map_err(|e| format!("Failed to open git repo {}: {e}", repo_path.display()))?;

    let repo_url = repo
        .find_remote("origin")
        .and_then(|r| {
            r.url()
                .map(|s| s.to_string())
                .ok_or(git2::Error::from_str("no url"))
        })
        .unwrap_or_default();

    let current_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));
    let repo_ref = repo
        .head()
        .ok()
        .and_then(|h| h.target().map(|oid| oid.to_string()));
    let available_branches = get_branch_names(&repo);
    let (owner, _) = clone::extract_owner_repo_from_url(&repo_url)
        .unwrap_or_else(|_| ("Unknown owner".to_string(), "Unknown repo".to_string()));

    let latest_ref = if disk_only {
        None
    } else {
        current_branch.as_ref().and_then(|branch| {
            let refname = format!("refs/remotes/origin/{branch}");
            repo.find_reference(&refname)
                .ok()
                .and_then(|r| r.target().map(|oid| oid.to_string()))
        })
    };

    let addons = find_all_sub_addons(&repo_path.to_path_buf())
        .map_err(|e| format!("Failed to discover sub-addons: {e}"))?;

    let readme = find_readme(repo_path);

    Ok(DiskAddonRepository {
        repo_url,
        repo_name: repo_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default(),
        owner,
        current_branch,
        available_branches,
        repo_ref,
        latest_ref,
        addons,
        readme,
    })
}

// Helper to find the README in a directory
fn find_readme(dir_path: &std::path::Path) -> Option<String> {
    let readme_names = [
        "README.md",
        "readme.md",
        "README.txt",
        "readme.txt",
        "README",
        "readme",
        ".github/README.md",
        ".github/readme.md",
        ".github/README.txt",
        ".github/readme.txt",
    ];
    for name in &readme_names {
        let candidate = dir_path.join(name);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

fn get_branch_names(repo: &git2::Repository) -> Vec<String> {
    let get_branch_names = |branch_type| {
        repo.branches(Some(branch_type))
            .map(|branches| {
                branches
                    .filter_map(|b| {
                        b.ok()
                            .and_then(|(branch, _)| branch.name().ok().flatten().map(String::from))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    let mut branch_names = get_branch_names(git2::BranchType::Local);
    branch_names.extend(get_branch_names(git2::BranchType::Remote));
    branch_names
        .into_iter()
        .filter(|name| name != "origin/main" && name != "origin/HEAD" && name != "origin/master")
        .collect()
}

/// Finds all sub-addons by searching for .toc files in the root directory and immediate subdirectories only
pub fn find_all_sub_addons(path: &PathBuf) -> Result<Vec<DiskAddon>, String> {
    // Helper to extract notes from a .toc file
    fn extract_notes(toc_path: &std::path::Path) -> Option<String> {
        if let Ok(content) = std::fs::read_to_string(toc_path) {
            for line in content.lines() {
                if let Some(notes) = line
                    .strip_prefix("## Notes:")
                    .or_else(|| line.strip_prefix("##Notes:"))
                {
                    return Some(notes.trim().to_string());
                }
            }
        }
        None
    }

    let mut sub_addons = Vec::new();

    // Helper to process a directory and collect .toc files
    fn collect_toc_files(dir: &Path) -> Result<Vec<String>, String> {
        let toc_files = std::fs::read_dir(dir)
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
        Ok(toc_files)
    }

    /// This is to handle cases where multiple .toc files exist in the root with multiple base names
    fn names_from_toc_files(toc_files: &[String]) -> Vec<String> {
        toc_files
            .iter()
            .map(|toc| toc_file_base_name(toc))
            .unique()
            .map(|name| name.to_string())
            .collect()
    }

    fn longest_string(names: &[String]) -> String {
        names
            .iter()
            .max_by_key(|n| n.len())
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    }

    // Process root directory
    let toc_files = collect_toc_files(path)?;
    if !toc_files.is_empty() {
        let names = names_from_toc_files(&toc_files);
        let name = longest_string(&names);
        // Extract notes from the first .toc file in root
        let primary_toc = path.join(&toc_files[0]);
        let notes = extract_notes(&primary_toc);
        sub_addons.push(DiskAddon {
            dir: ".".to_string(),
            names,
            name,
            is_symlinked: false, // Will be updated by check_addon_symlinks
            notes,
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
                let names = names_from_toc_files(&toc_files);
                let name = longest_string(&names);
                let dir_name = sub_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                Some({
                    // Extract notes from the first .toc file in this subdirectory
                    let toc_full = sub_path.join(&toc_files[0]);
                    let notes = extract_notes(&toc_full);
                    DiskAddon {
                        dir: dir_name,
                        names,
                        name,
                        is_symlinked: false, // Will be updated by check_addon_symlinks
                        notes,
                    }
                })
            }),
    );
    Ok(sub_addons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    /// https://github.com/Sattva-108/AdiBags
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
    /// https://github.com/Sattva-108/AdiBags-WoTLK-3.3.5-Mods
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
                sub.names.len(),
                1,
                "Expected 1 name in {:?}, found: {:?}",
                sub.dir,
                sub.names
            );
        }
    }

    #[test]
    /// https://github.com/widxwer/Questie
    /// This Questie has multiple basename .toc files in the root directory
    /// It is expected that the user renames the folder manually
    /// We should discover the multiple base names
    fn test_find_all_sub_addons_questie_multiple_tocs_in_root() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        // Create multiple .toc files in the root directory
        let toc_files = vec![
            "Questie-335-Classic.toc",
            "Questie-335-TBC.toc",
            "Questie-335.toc",
            "Questie-BCC.toc",
            "Questie-Classic.toc",
            "Questie-WOTLKC.toc",
            "Questie.toc",
        ];
        for toc in &toc_files {
            std::fs::File::create(repo_dir.join(toc)).unwrap();
        }

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert_eq!(
            sub_addons.len(),
            1,
            "Expected 1 sub_addon, found: {:?}",
            sub_addons
        );
        let sub = &sub_addons[0];
        assert_eq!(
            sub.dir, ".",
            "Expected sub_addon dir to be '.', found: {}",
            sub.dir
        );
        for name in &sub.names {
            assert!(
                name == "Questie" || name == "Questie-335",
                "Expected normalized name to be 'Questie' or 'Questie-335', found: {}",
                name
            );
        }

        assert!(
            sub.names.len() == 2,
            "Expected 2 unique names, found: {:?}",
            sub.names
        );
    }
}

/// Returns the canonical base name for a .toc file
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
/// use addon_gui_lib::addon_disk::toc_file_base_name;
/// assert_eq!(toc_file_base_name("AdiBags.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_Mainline.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_Cata.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_Wrath.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_TBC.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_Vanilla.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_Cata.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags-WOTLKC.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_BCC.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags-Classic.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("AdiBags_wotlk.toc"), "AdiBags");
/// assert_eq!(toc_file_base_name("Questie-335.toc"), "Questie-335");
/// assert_eq!(toc_file_base_name("TrainerButton.toc"), "TrainerButton");
/// assert_eq!(toc_file_base_name("!!TrainerButton.toc"), "!!TrainerButton");
/// ```
pub fn toc_file_base_name(toc_file: &str) -> &str {
    const SUFFIXES_TO_STRIP: &[&str] = &[
        "-mainline.toc",
        "-cataclysm.toc",
        "-cata.toc",
        "-wrath.toc",
        "-tbc.toc",
        "-vanilla.toc",
        "-classic.toc",
        "-bcc.toc",
        "-wotlkc.toc",
        "-wotlk.toc",
        "_mainline.toc",
        "_cataclysm.toc",
        "_cata.toc",
        "_wrath.toc",
        "_tbc.toc",
        "_vanilla.toc",
        "_classic.toc",
        "_bcc.toc",
        "_wotlkc.toc",
        "_wotlk.toc",
        ".toc",
    ];

    let toc_file_lower = toc_file.to_lowercase();
    for suf in SUFFIXES_TO_STRIP {
        if toc_file_lower.ends_with(suf) {
            // Find the start index of the suffix in the original string
            let idx = toc_file.len() - suf.len();
            return &toc_file[..idx];
        }
    }
    toc_file
}
