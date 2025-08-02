pub mod addon_discovery;
pub mod addon_disk;
pub mod addon_store;
pub mod clone;
pub mod install;
pub mod symlink;
pub mod validate;
pub mod view_models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(addon_discovery::AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            validate::is_valid_repo_url,
            validate::is_valid_addons_folder_str,
            install::install_addon_cmd,
            addon_discovery::refresh_addon_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
