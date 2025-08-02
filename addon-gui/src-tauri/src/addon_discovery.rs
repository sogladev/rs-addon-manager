use std::sync::RwLockReadGuard;

use crate::addon_disk::DiskAddOnsFolder;
use crate::addon_store::AddOnsUserConfig;
use crate::view_models;
use serde_json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use std::{collections::HashMap, sync::RwLock};

const STORE_FILE: &str = "addon-manager.json";
const STORE_KEY: &str = "addon-directories";

// This is never persisted; just holds our latest disk scan data
pub struct AppState {
    disk_state: RwLock<HashMap<String, DiskAddOnsFolder>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            disk_state: RwLock::new(HashMap::new()),
        }
    }
}

#[tauri::command]
/// Refresh addon data by scanning configured folders
pub fn refresh_addon_data(
    app: AppHandle,
    state: tauri::State<AppState>,
) -> Result<Vec<view_models::FolderWithMeta>, String> {
    // Read configured addon directories from store
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let raw = store.get(STORE_KEY).unwrap_or_default();
    let config: AddOnsUserConfig = serde_json::from_value(raw).unwrap_or_default();

    // Scan folders and stash in‐mem disk data
    {
        let mut map = match state.disk_state.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("RwLock poisoned: {poisoned:?}");
                poisoned.into_inner()
            }
        };
        map.clear();
        for folder_meta in &config.folders {
            let path = &folder_meta.path;
            let folder = DiskAddOnsFolder::scan(path).unwrap_or_else(|error| {
                eprintln!("Failed to scan path {path:?}: {error:?}");
                DiskAddOnsFolder::default_with_error(path, error)
            });
            map.insert(path.clone(), folder.clone());
        }
    }

    let guard: RwLockReadGuard<'_, _> = state.disk_state.read().map_err(|e| e.to_string())?;
    let disk_map = guard.clone();

    // Merge disk + user‐meta
    let merged: Vec<view_models::FolderWithMeta> = disk_map
        .into_iter()
        .map(|(path, disk_folder)| {
            // find the matching folder user‐meta (if any)
            let folder_meta = config.folders.iter().find(|f| f.path == path);

            let repos = disk_folder
                .repositories
                .into_iter()
                .map(|disk_repo| {
                    // find the matching repo user‐meta by repo_url
                    let user_repo = folder_meta
                        .and_then(|f| f.repos.iter().find(|r| r.repo_url == disk_repo.repo_url));

                    // build addons + user‐meta
                    let addons = disk_repo
                        .addons
                        .into_iter()
                        .map(|disk_addon| {
                            let user_addon = user_repo.and_then(|r| r.addons.get(&disk_addon.name));
                            view_models::AddonWithMeta {
                                name: disk_addon.name.clone(),
                                names: disk_addon.names,
                                dir: disk_addon.dir,
                                is_symlinked: disk_addon.is_symlinked,
                                enabled: user_addon.map(|m| m.enabled).unwrap_or(true),
                                custom_name: user_addon.and_then(|m| m.name.clone()),
                            }
                        })
                        .collect();

                    view_models::RepositoryWithMeta {
                        repo_url: disk_repo.repo_url.clone(),
                        repo_name: disk_repo.repo_name,
                        owner: disk_repo.owner,
                        current_branch: disk_repo.current_branch,
                        available_branches: disk_repo.available_branches,
                        repo_ref: disk_repo.repo_ref,
                        addons,
                    }
                })
                .collect();

            view_models::FolderWithMeta {
                path: disk_folder.path,
                is_valid: disk_folder.is_valid,
                error: disk_folder.error,
                repositories: repos,
            }
        })
        .collect();

    Ok(merged)
}
