use std::sync::RwLockReadGuard;

use crate::addon_disk::DiskAddOnsFolder;
use crate::operation_tracker::OperationTracker;
use crate::view_models;
use tauri::AppHandle;

use std::{collections::HashMap, sync::RwLock};

// This is never persisted; just holds our latest disk scan data
pub struct AppState {
    disk_state: RwLock<HashMap<String, DiskAddOnsFolder>>,
    operation_tracker: OperationTracker,
}

impl AppState {
    pub fn get_disk_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<HashMap<String, DiskAddOnsFolder>>, String> {
        self.disk_state.read().map_err(|e| e.to_string())
    }

    /// Get a reference to the operation tracker
    pub fn get_operation_tracker(&self) -> &OperationTracker {
        &self.operation_tracker
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            disk_state: RwLock::new(HashMap::new()),
            operation_tracker: OperationTracker::default(),
        }
    }
}

#[tauri::command]
/// Refresh addon data by scanning configured folders
pub fn refresh_addon_data(
    app: AppHandle,
    state: tauri::State<AppState>,
) -> Result<Vec<view_models::AddOnsFolder>, String> {
    // Read configured addon directories
    let config = crate::addon_store::load_user_config(&app)?;

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
            let folder = DiskAddOnsFolder::scan(path).unwrap_or_else(|e| {
                eprintln!("Failed to scan path {path:?}: {e:?}");
                DiskAddOnsFolder::default_with_error(path, e)
            });
            map.insert(path.clone(), folder.clone());
        }
    }

    let guard: RwLockReadGuard<'_, _> = state.disk_state.read().map_err(|e| e.to_string())?;
    let disk_map = guard.clone();

    // Merge disk + user‐meta
    let merged: Vec<view_models::AddOnsFolder> = disk_map
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
                            view_models::Addon {
                                name: disk_addon.name.clone(),
                                names: disk_addon.names,
                                dir: disk_addon.dir,
                                is_symlinked: disk_addon.is_symlinked,
                                custom_name: user_addon.and_then(|m| m.name.clone()),
                            }
                        })
                        .collect();

                    view_models::AddonRepository {
                        repo_url: disk_repo.repo_url.clone(),
                        repo_name: disk_repo.repo_name,
                        owner: disk_repo.owner,
                        current_branch: disk_repo.current_branch,
                        available_branches: disk_repo.available_branches,
                        repo_ref: disk_repo.repo_ref,
                        latest_ref: disk_repo.latest_ref,
                        addons,
                    }
                })
                .collect();

            view_models::AddOnsFolder {
                path: disk_folder.path,
                is_valid: disk_folder.is_valid,
                error: disk_folder.error,
                repositories: repos,
            }
        })
        .collect();

    Ok(merged)
}

#[tauri::command]
/// Refresh addon data by scanning configured folders (disk-only, no remote operations)
pub fn refresh_disk_data(
    app: AppHandle,
    state: tauri::State<AppState>,
) -> Result<Vec<view_models::AddOnsFolder>, String> {
    // Read configured addon directories
    let config = crate::addon_store::load_user_config(&app)?;

    // Scan folders and stash in‐mem disk data (disk-only)
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
            let folder = DiskAddOnsFolder::scan_disk_only(path).unwrap_or_else(|e| {
                eprintln!("Failed to scan path {path:?}: {e:?}");
                DiskAddOnsFolder::default_with_error(path, e)
            });
            map.insert(path.clone(), folder.clone());
        }
    }

    let guard: RwLockReadGuard<'_, _> = state.disk_state.read().map_err(|e| e.to_string())?;
    let disk_map = guard.clone();

    // Merge disk + user‐meta
    let merged: Vec<view_models::AddOnsFolder> = disk_map
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
                            view_models::Addon {
                                name: disk_addon.name.clone(),
                                names: disk_addon.names,
                                dir: disk_addon.dir,
                                is_symlinked: disk_addon.is_symlinked,
                                custom_name: user_addon.and_then(|m| m.name.clone()),
                            }
                        })
                        .collect();

                    view_models::AddonRepository {
                        repo_url: disk_repo.repo_url.clone(),
                        repo_name: disk_repo.repo_name,
                        owner: disk_repo.owner,
                        current_branch: disk_repo.current_branch,
                        available_branches: disk_repo.available_branches,
                        repo_ref: disk_repo.repo_ref,
                        latest_ref: disk_repo.latest_ref,
                        addons,
                    }
                })
                .collect();

            view_models::AddOnsFolder {
                path: disk_folder.path,
                is_valid: disk_folder.is_valid,
                error: disk_folder.error,
                repositories: repos,
            }
        })
        .collect();

    Ok(merged)
}
