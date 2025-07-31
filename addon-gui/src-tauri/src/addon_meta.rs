use std::path::Path;

use serde::{Deserialize, Serialize};

pub const ADDONS_FOLDER_METADATA_FILE: &str = "metadata.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Addon {
    /// The name used for the symlink in AddOns
    pub name: String,
    /// The relative path to the sub-addon directory inside the repo
    pub dir: String,
    /// Normalized base names after removing suffixes (from .toc).
    pub names: Vec<String>,
    /// .toc file names
    pub toc_files: Vec<String>,
    /// Enabled
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddonRepository {
    /// The git repository URL
    pub repo_url: String,
    /// The owner of the repository (e.g., GitHub username)
    pub owner: String,
    /// The repository name
    pub repo_name: String,
    /// Branch
    pub branch: Option<String>,
    /// List of available branches
    pub available_branches: Vec<String>,
    /// Commit hash or tag
    pub repo_ref: Option<String>,
    /// All discovered sub-addons in this repo
    pub addons: Vec<Addon>,
}

impl AddonRepository {
    fn get_branch_names(repo: &git2::Repository) -> Vec<String> {
        let get_branch_names = |branch_type| {
            repo.branches(Some(branch_type))
                .map(|branches| {
                    branches
                        .filter_map(|b| {
                            b.ok().and_then(|(branch, _)| {
                                branch.name().ok().flatten().map(String::from)
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let mut branch_names = get_branch_names(git2::BranchType::Local);
        branch_names.extend(get_branch_names(git2::BranchType::Remote));
        branch_names
    }

    pub fn build_addon_repository(
        url: &str,
        path: &Path,
        repo: &git2::Repository,
        owner: String,
        sub_addons: Vec<Addon>,
    ) -> AddonRepository {
        let available_branches = Self::get_branch_names(repo);
        AddonRepository {
            repo_url: url.to_string(),
            owner,
            repo_name: path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown-repo>".to_string()),
            repo_ref: repo
                .head()
                .ok()
                .and_then(|head| head.target())
                .map(|oid| oid.to_string()),
            branch: repo
                .head()
                .ok()
                .and_then(|head| head.shorthand().map(|s| s.to_string())),
            available_branches,
            addons: sub_addons,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddOnsFolder {
    /// Absolute path to AddOns folder
    pub path: String,
    /// Whether this is a valid AddOns folder
    pub is_valid: bool,
    /// All discovered addons in this folder
    pub addon_repos: Vec<AddonRepository>,
}

impl AddOnsFolder {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        if path.as_ref().exists() {
            let toml_str = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read metadata: {e}"))?;
            let meta =
                toml::from_str(&toml_str).map_err(|e| format!("Failed to parse metadata: {e}"))?;
            Ok(meta)
        } else {
            Err(format!(
                "Metadata file does not exist: {}",
                path.as_ref().display()
            ))
        }
    }

    /// Create a default AddOnsFolder for the given folder path
    pub fn default_with_path<P: AsRef<Path>>(folder_path: P) -> Self {
        let path = folder_path.as_ref().to_string_lossy().to_string();
        let is_valid = crate::validate::is_valid_addons_folder_str(&path);
        Self {
            path,
            is_valid,
            addon_repos: vec![],
        }
    }

    pub fn load_from_manager_dir<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let meta_path = path.as_ref().join(ADDONS_FOLDER_METADATA_FILE);
        Self::load_from_path(meta_path)
    }

    pub fn load_from_addons_dir<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let meta_path = path
            .as_ref()
            .join(".addonmanager")
            .join(ADDONS_FOLDER_METADATA_FILE);
        Self::load_from_path(meta_path)
    }

    fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let toml_str = toml::to_string_pretty(&self).expect("Serialization failed");
        std::fs::write(path, toml_str)
    }

    pub fn save_to_manager_dir<P: AsRef<Path>>(&self, manager_dir: P) -> Result<(), String> {
        let meta_path = manager_dir.as_ref().join(ADDONS_FOLDER_METADATA_FILE);
        self.save_to_path(meta_path)
            .map_err(|e| format!("Failed to save metadata: {e}"))
    }

    pub fn upsert_addon(&mut self, addon: AddonRepository) {
        if let Some(existing) = self
            .addon_repos
            .iter_mut()
            .find(|a| a.repo_url == addon.repo_url)
        {
            *existing = addon;
        } else {
            self.addon_repos.push(addon);
        }
    }
}
