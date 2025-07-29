pub mod clone;
pub mod install;
pub mod validate;

use install::install_addon;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            validate::is_valid_repo_url,
            validate::is_valid_addons_folder_str,
            install_addon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
