use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_http::reqwest;

use downloader_core::{
    game, Manifest, Progress, Provider, Transaction, TransactionReport, config::DEFAULT_MANIFEST_URL
};

#[derive(Default)]
struct AppState {
    transaction: Option<Transaction>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            verify_game_integrity,
            create_transaction,
            download,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn verify_game_integrity(base_path: String) -> Result<bool, String> {
    let base_path = std::path::PathBuf::from(base_path);
    if !base_path.try_exists().map_err(|e| e.to_string())? {
        return Err("Game path does not exist".to_string());
    }

    game::verify_game_integrity(base_path.as_path()).map_err(|e| e.to_string())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn create_transaction(
    base_path: String,
    app_handle: tauri::AppHandle,
) -> Result<TransactionReport, String> {
    let base_path = std::path::PathBuf::from(base_path);

    let res = reqwest::get(DEFAULT_MANIFEST_URL)
        .await
        .map_err(|e| e.to_string())?;
    let json = res.text().await.map_err(|e| e.to_string())?;
    let manifest = Manifest::from_json(&json).map_err(|e| e.to_string())?;

    // Create a transaction
    let transaction = Transaction::new(manifest, base_path);
    #[cfg(debug_assertions)]
    transaction.print(); // Generate a report and print to stdout

    let report = transaction.generate_report();
    {
        // Store the transaction in the app state
        let state = app_handle.state::<Mutex<AppState>>();
        // Lock the mutex to get mutable access:
        let mut state = state.lock().unwrap();
        // Modify the state:
        state.transaction = Some(transaction);
    } // Drop MutexGuard

    Ok(report)
}

#[tauri::command]
async fn download(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Create a transaction clone outside the mutex guard
    let transaction = {
        let state = app_handle.state::<Mutex<AppState>>();
        let state = state.lock().unwrap();

        match &state.transaction {
            None => return Err("No transaction found".to_string()),
            Some(transaction) => transaction.clone(),
        }
    }; // Drop MutexGuard

    // Now we can safely use the transaction clone in async code
    if !transaction.has_pending_operations() {
        return Ok(());
    }

    println!("Downloading files...");

    // Create a progress handler that sends Progress callbacks through Tauri
    let progress_handler = move |progress: &Progress| {
        app_handle
            .emit("download-progress", &progress)
            .map_err(|e| e.to_string())?;
        Ok(())
    };
    transaction
        .download(progress_handler, Provider::Cloudflare)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn launch_game(base_path: String) -> Result<(), String> {
    println!("Launching game...");
    let base_path = std::path::PathBuf::from(base_path);
    game::launch(&base_path, "Project-Epoch.exe".to_string()).map_err(|e| e.to_string())?;
    std::process::exit(0);
}
