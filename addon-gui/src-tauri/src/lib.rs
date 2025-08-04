pub mod addon_discovery;
pub mod addon_disk;
pub mod addon_store;
pub mod clone;
pub mod install;
pub mod operation_tracker;
pub mod permission_workaround;
pub mod remove;
pub mod symlink;
#[cfg(test)]
pub mod test_utils;
pub mod update;
pub mod validate;
pub mod view_models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(addon_discovery::AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            addon_discovery::refresh_addon_data,
            addon_discovery::refresh_disk_data,
            addon_store::add_addon_directory,
            addon_store::delete_addon_directory,
            addon_store::load_theme,
            addon_store::save_theme,
            install::create_addon_symlink,
            install::install_addon_cmd,
            install::remove_addon_symlink,
            permission_workaround::allow_file,
            remove::delete_addon_cmd,
            update::update_addon_cmd,
            update::update_all_addons_cmd,
            validate::is_valid_addons_folder_str,
            validate::is_valid_repo_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
