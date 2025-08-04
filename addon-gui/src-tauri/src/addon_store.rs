use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "addon-manager.json";
const STORE_KEY: &str = "addon-directories";

pub fn load_user_config(app: &AppHandle) -> Result<AddOnsUserConfig, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let raw = store.get(STORE_KEY).unwrap_or_default();
    let config: AddOnsUserConfig = serde_json::from_value(raw).unwrap_or_default();
    Ok(config)
}

#[tauri::command]
pub async fn add_addon_directory(path: String, app_handle: AppHandle) -> Result<(), String> {
    println!("Adding addon directory: {path}");
    let store = app_handle.store(STORE_FILE).map_err(|e| e.to_string())?;
    let raw = store.get(STORE_KEY).unwrap_or_default();
    let mut config: AddOnsUserConfig = serde_json::from_value(raw).unwrap_or_default();

    if !config.folders.iter().any(|f| f.path == path) {
        config.folders.push(AddOnsFolderUserMeta {
            path: path.clone(),
            repos: Vec::new(),
        });
        let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
        store.set(STORE_KEY, value);
        store.save().map_err(|e| e.to_string())?;
        app_handle
            .emit("addon-data-updated", ())
            .map_err(|e| format!("Failed to emit event: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_addon_directory(path: String, app: AppHandle) -> Result<(), String> {
    println!("Deleting addon directory: {path}");
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let raw = store.get(STORE_KEY).unwrap_or_default();
    let mut config: AddOnsUserConfig = serde_json::from_value(raw).unwrap_or_default();

    let orig_len = config.folders.len();
    config.folders.retain(|f| f.path != path);
    if config.folders.len() != orig_len {
        let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
        store.set(STORE_KEY, value);
        store.save().map_err(|e| e.to_string())?;
        app.emit("addon-data-updated", ())
            .map_err(|e| format!("Failed to emit event: {e}"))?;
    }
    Ok(())
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
#[derive(Default)]
pub struct AddOnsUserConfig {
    /// All managed AddOns directories and their metadata
    pub folders: Vec<AddOnsFolderUserMeta>,
    /// Selected theme name
    pub theme: Option<String>,
}

#[tauri::command]
pub async fn save_theme(theme: String, app_handle: AppHandle) -> Result<(), String> {
    let mut config = load_user_config(&app_handle)?;
    config.theme = Some(theme.clone());
    let store = app_handle.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    store.set(STORE_KEY, value);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_theme(app_handle: AppHandle) -> Result<String, String> {
    let config = load_user_config(&app_handle)?;
    Ok(config.theme.clone().unwrap_or_default())
}
