use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonWithMeta {
    pub name: String,
    pub dir: String,
    pub names: Vec<String>,
    pub is_symlinked: bool,

    // user overrides:
    pub enabled: bool,
    pub custom_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWithMeta {
    pub repo_url: String,
    pub repo_name: String,
    pub owner: String,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub repo_ref: Option<String>,

    // merged addon list:
    pub addons: Vec<AddonWithMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FolderWithMeta {
    pub path: String,
    pub is_valid: bool,
    pub error: Option<String>,
    pub repositories: Vec<RepositoryWithMeta>,
}
