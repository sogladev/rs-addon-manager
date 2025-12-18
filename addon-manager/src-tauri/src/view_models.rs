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
    pub notes: Option<String>,

    // user overrides:
    pub custom_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum AddonSource {
    Git {
        repo_url: String,
        owner: String,
        repo_name: String,
        current_branch: Option<String>,
        available_branches: Vec<String>,
        repo_ref: Option<String>,
        latest_ref: Option<String>,
        readme: Option<String>,
    },
    Local {
        folder_name: String,
        path: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonRepository {
    pub source: AddonSource,
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
