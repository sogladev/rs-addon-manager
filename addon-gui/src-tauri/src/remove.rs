use std::{fs, path::PathBuf};

use tauri::Emitter;

use crate::clone;

#[tauri::command]
pub async fn delete_addon(
    app_handle: tauri::AppHandle,
    url: String,
    path: String,
) -> Result<(), String> {
    // Base AddOns and manager directories
    let addons_dir = PathBuf::from(&path);
    let manager_root = addons_dir.join(".addonmanager");

    // Extract owner and repo name from URL
    let (owner, repo_name) =
        clone::extract_owner_repo_from_url(&url).map_err(|e| format!("Invalid repo URL: {}", e))?;

    // Cleanup symlinks in AddOns folder
    if let Ok(entries) = fs::read_dir(&addons_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                if let Ok(target) = fs::read_link(&p) {
                    if target
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == repo_name)
                        .unwrap_or(false)
                    {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }

    // Remove cloned repository directory
    let repo_dir = manager_root.join(owner).join(repo_name);
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }
    // Emit event to refresh frontend data
    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}
