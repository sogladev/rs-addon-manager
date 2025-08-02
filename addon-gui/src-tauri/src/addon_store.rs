use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
    pub repo_url: String,                       // used as unique key
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
