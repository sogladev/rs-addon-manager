use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Addon {
    pub name: String,
    pub dir: String,
    pub names: Vec<String>,
    pub is_symlinked: bool,

    // user overrides:
    pub custom_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonRepository {
    pub repo_url: String,
    pub repo_name: String,
    pub owner: String,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub repo_ref: Option<String>,
    pub latest_ref: Option<String>,

    // merged addon list:
    pub addons: Vec<Addon>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddOnsFolder {
    pub path: String,
    pub is_valid: bool,
    pub error: Option<String>,
    pub repositories: Vec<AddonRepository>,
}
