use tauri::{AppHandle, Runtime};
use tauri_plugin_fs::FsExt;

#[tauri::command]
/// Allow file access for a specific path
/// This is a workaround for Tauri's file system permissions to allow README.md to be read
pub fn allow_file<R: Runtime>(app: AppHandle<R>, path: &str) -> () {
    let _ = app.fs_scope().allow_file(path).unwrap_or_else(|e| {
        eprintln!("Failed to allow file access for {path}: {e}");
    });
}
