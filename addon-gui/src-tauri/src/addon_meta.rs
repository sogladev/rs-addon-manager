use std::path::Path;

use serde::{Deserialize, Serialize};

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
