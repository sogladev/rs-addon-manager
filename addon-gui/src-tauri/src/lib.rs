pub mod addon_discovery;
pub mod addon_meta;
pub mod clone;
pub mod install;
pub mod symlink;
pub mod validate;

use std::{collections::HashMap, sync::RwLock};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

use crate::addon_meta::AddOnsUserConfig;
use addon_meta::DiskAddOnsFolder;

const STORE_FILE: &'static str = "addon-manager.json";
const STORE_KEY: &'static str = "addon-directories";

// This is never persisted; just holds our latest disk scan data
struct AppState {
    disk_state: RwLock<HashMap<String, DiskAddOnsFolder>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            disk_state: RwLock::new(HashMap::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Read metadata paths
            let store = app.store(STORE_FILE)?;
            let raw = store.get(STORE_KEY).unwrap_or_default();
            let config: AddOnsUserConfig = serde_json::from_value(raw).unwrap_or_default();

            // For each configured AddOns folder, scan disk and emit to front-end
            let handle = app.handle();
            if config.folders.len() > 0 {
                let mut map = match app.state::<AppState>().disk_state.write() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        eprintln!("RwLock poisoned: {:?}", poisoned);
                        poisoned.into_inner()
                    }
                };
                for folder_meta in &config.folders {
                    let path = &folder_meta.path;
                    let folder = DiskAddOnsFolder::scan(path).unwrap_or_else(|error| {
                        eprintln!("Failed to scan path {:?}: {:?}", path, error);
                        DiskAddOnsFolder::default_with_error(path, error)
                    });
                    map.insert(path.clone(), folder.clone());
                    handle.emit("addon-manager-data-updated", folder)?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            validate::is_valid_repo_url,
            validate::is_valid_addons_folder_str,
            install::install_addon_cmd,
            install::get_addon_manager_data,
            install::delete_addon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
