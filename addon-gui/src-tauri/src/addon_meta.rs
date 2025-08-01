use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
/// Folder must contain a .toc file, sometimes multiple
pub struct DiskAddon {
    pub name: String,
    pub dir: String,
    pub toc_files: Vec<String>,
    pub is_symlinked: bool, // true is symlink exists in AddOns
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
/// Folder with a .git subdirectory
pub struct DiskAddonRepository {
    pub repo_url: String,
    pub repo_name: String,
    pub owner: String,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub repo_ref: Option<String>, // commit hash or tag
    pub addons: Vec<DiskAddon>,
}
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiskAddOnsFolder {
    /// Absolute path to the AddOns directory
    pub path: String,
    pub is_valid: bool,
    /// All discovered addon repositories under .addonmanager
    pub repositories: Vec<DiskAddonRepository>,
    /// Error
    pub error: Option<String>,
}
// ...existing code above...
impl DiskAddOnsFolder {
    /// Scan the AddOns directory on disk
    pub fn scan(path: &str) -> Result<Self, String> {
        let addons_path = Path::new(path);
        let is_valid = crate::validate::is_valid_addons_folder_str(path);
        let manager_dir = addons_path.join(".addonmanager");
        let mut repositories = Vec::new();
        if manager_dir.exists() {
            for owner_entry in std::fs::read_dir(&manager_dir)
                .map_err(|e| format!("Failed to read manager dir {}: {e}", manager_dir.display()))?
            {
                let owner_path = owner_entry.map_err(|e| e.to_string())?.path();
                if !owner_path.is_dir() {
                    continue;
                }
                let owner = owner_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                for repo_entry in std::fs::read_dir(&owner_path).map_err(|e| {
                    format!("Failed to read owner dir {}: {e}", owner_path.display())
                })? {
                    let repo_path = repo_entry.map_err(|e| e.to_string())?.path();
                    if !repo_path.is_dir() {
                        continue;
                    }
                    let repo = git2::Repository::open(&repo_path).map_err(|e| {
                        format!("Failed to open git repo {}: {e}", repo_path.display())
                    })?;
                    let repo_url = repo
                        .find_remote("origin")
                        .and_then(|r| {
                            r.url()
                                .map(|u| u.to_string())
                                .ok_or(git2::Error::from_str("no url"))
                        })
                        .unwrap_or_default();
                    let addons = crate::addon_discovery::find_all_sub_addons(&repo_path)
                        .map_err(|e| format!("Failed to discover sub-addons: {e}"))?;
                    let current_branch = repo
                        .head()
                        .ok()
                        .and_then(|h| h.shorthand().map(|s| s.to_string()));
                    let repo_ref = repo
                        .head()
                        .ok()
                        .and_then(|h| h.target().map(|oid| oid.to_string()));
                    let available_branches = get_branch_names(&repo);
                    repositories.push(DiskAddonRepository {
                        repo_url,
                        repo_name: repo_path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        owner: owner.clone(),
                        current_branch,
                        available_branches,
                        repo_ref,
                        addons,
                    });
                }
            }
        }
        Ok(DiskAddOnsFolder {
            path: path.to_string(),
            is_valid,
            repositories,
            error: None,
        })
    }

    /// Default empty
    // @todo: delete this
    // pub fn default_with_path(path: &str) -> Self {
    //     DiskAddOnsFolder {
    //         path: path.to_string(),
    //         is_valid: crate::validate::is_valid_addons_folder_str(path),
    //         repositories: Vec::new(),
    //         error: None
    //     }
    // }

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

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
enum InstallStatus {
    Pending,
    Installing,
    Success,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddonUserMeta {
    pub enabled: bool,
    pub name: Option<String>, // symbolic link name override
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddonRepositoryUserMeta {
    pub repo_url: String, // used as unique key
    pub preferred_branch: Option<String>,
    pub selected_branch: Option<String>,
    pub install_status: Option<String>,
    pub install_progress: Option<InstallStatus>,
    pub install_error: Option<String>,
    pub install_step: Option<String>,
    pub addons: HashMap<String, AddonUserMeta>, // keyed by repo_url
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
/// User metadata for a single AddOns directory
pub struct AddOnsFolderUserMeta {
    /// Absolute path to the AddOns directory
    pub path: String,
    pub repos: Vec<AddonRepositoryUserMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
/// user configuration stored in Tauri store
pub struct AddOnsUserConfig {
    /// All managed AddOns directories and their metadata
    pub folders: Vec<AddOnsFolderUserMeta>,
}

impl Default for AddOnsUserConfig {
    fn default() -> Self {
        AddOnsUserConfig {
            folders: Vec::new(),
        }
    }
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
}

// impl AddonRepository {
//     pub fn build_addon_repository(
//         url: &str,
//         path: &Path,
//         repo: &git2::Repository,
//         owner: String,
//         sub_addons: Vec<Addon>,
//     ) -> AddonRepository {
//         let available_branches = Self::get_branch_names(repo);
//         AddonRepository {
//             repo_url: url.to_string(),
//             owner,
//             repo_name: path
//                 .file_name()
//                 .map(|f| f.to_string_lossy().to_string())
//                 .unwrap_or_else(|| "<unknown-repo>".to_string()),
//             repo_ref: repo
//                 .head()
//                 .ok()
//                 .and_then(|head| head.target())
//                 .map(|oid| oid.to_string()),
//             branch: repo
//                 .head()
//                 .ok()
//                 .and_then(|head| head.shorthand().map(|s| s.to_string())),
//             available_branches,
//             addons: sub_addons,
//         }
//     }
// }

// pub fn upsert_addon(&mut self, addon: AddonRepository) {
//     if let Some(existing) = self
//         .addon_repos
//         .iter_mut()
//         .find(|a| a.repo_url == addon.repo_url)
//     {
//         *existing = addon;
//     } else {
//         self.addon_repos.push(addon);
//     }
// }
