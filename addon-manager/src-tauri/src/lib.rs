pub mod addon_discovery;
pub mod addon_disk;
pub mod addon_store;
pub mod git;
pub mod install;
pub mod operation_reporter;
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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .manage(addon_discovery::AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            addon_discovery::refresh_addon_data,
            addon_discovery::refresh_disk_data,
            addon_discovery::check_for_updates,
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

pub fn run_headless(quiet: bool) {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .manage(addon_discovery::AppState::default())
        .setup(|_app| Ok(()))
        .build(tauri::generate_context!())
        .expect("failed to build minimal Tauri app");

    let handle = app.handle();
    let state = tauri::Manager::state::<addon_discovery::AppState>(&app).clone();

    tauri::async_runtime::block_on(async {
        match addon_discovery::check_for_updates(handle.clone(), state.clone()).await {
            Ok(folders) => {
                if !quiet {
                    println!("Scanned {} AddOns folders", folders.len());
                    for folder in &folders {
                        if !folder.repositories.is_empty() {
                            println!(
                                "  - {}: {} repositories",
                                folder.path,
                                folder.repositories.len()
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {e}");
                std::process::exit(1);
            }
        }

        match update::update_all_addons_cmd(handle.clone(), state).await {
            Ok(_) => {
                if !quiet {
                    println!("All addons are up-to-date");
                }
            }
            Err(e) => {
                eprintln!("Failed to update addons: {e}");
                std::process::exit(1);
            }
        }
    });

    std::process::exit(0)
}
