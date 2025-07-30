use std::path::Path;

use serde::{Deserialize, Serialize};

pub const ADDON_MANAGER_METADATA_FILE: &str = "metadata.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct SubAddon {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonMeta {
    /// The git repository URL
    pub repo_url: String,
    /// The owner of the repository (e.g., GitHub username)
    pub owner: String,
    /// The repository name
    pub repo_name: String,
    /// Commit hash or tag
    pub installed_ref: Option<String>,
    /// Branch
    pub branch: Option<String>,
    /// Date/time of installation (ISO 8601)
    pub installed_at: Option<String>,
    /// All discovered sub-addons in this repo
    pub sub_addons: Vec<SubAddon>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonManagerData {
    pub addons: Vec<AddonMeta>,
}

impl AddonManagerData {
    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        if path.as_ref().exists() {
            let toml_str = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read metadata: {e}"))?;
            toml::from_str(&toml_str).map_err(|e| format!("Failed to parse metadata: {e}"))
        } else {
            Ok(Self { addons: vec![] })
        }
    }

    pub fn load_from_manager_dir<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let meta_path = path.as_ref().join(ADDON_MANAGER_METADATA_FILE);
        Self::load_from_path(meta_path)
    }

    fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let toml_str = toml::to_string_pretty(self).expect("Serialization failed");
        std::fs::write(path, toml_str)
    }

    pub fn save_to_manager_dir<P: AsRef<Path>>(&self, manager_dir: P) -> Result<(), String> {
        let meta_path = manager_dir.as_ref().join(ADDON_MANAGER_METADATA_FILE);
        self.save_to_path(meta_path)
            .map_err(|e| format!("Failed to save metadata: {e}"))
    }

    pub fn upsert_addon(&mut self, addon: AddonMeta) {
        if let Some(existing) = self
            .addons
            .iter_mut()
            .find(|a| a.repo_url == addon.repo_url)
        {
            *existing = addon;
        } else {
            self.addons.push(addon);
        }
    }
}
