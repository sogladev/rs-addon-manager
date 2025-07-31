pub mod addon_discovery;
pub mod addon_meta;
pub mod clone;
pub mod install;
pub mod symlink;
pub mod validate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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
