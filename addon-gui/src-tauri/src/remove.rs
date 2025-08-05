use std::{fs, path::PathBuf};

use tauri::{AppHandle, Emitter};

use crate::{addon_discovery::AppState, operation_tracker::*};

/// Deletes addon repo and symlinks by repo URL and AddOns path
pub fn delete_addon_files(url: &str, path: &str) -> Result<(), String> {
    let addons_dir = PathBuf::from(path);
    let manager_root = addons_dir.join(".addonmanager");
    let (_owner, repo_name) = crate::clone::extract_owner_repo_from_url(url)
        .map_err(|e| format!("Invalid repo URL: {e}"))?;

    // Remove any symlinks in AddOns whose target is inside this repo
    let repo_dir = manager_root.join(&repo_name);
    if let Ok(entries) = fs::read_dir(&addons_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                if let Ok(target) = fs::read_link(&p) {
                    // Delete symlink if it points into the repo directory
                    if target.starts_with(&repo_dir) {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }
    // Remove cloned repository directory
    let repo_dir = manager_root.join(repo_name);
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }
    Ok(())
}

async fn perform_delete_op(
    app_handle: &AppHandle,
    state: &tauri::State<'_, AppState>,
    url: String,
    path: String,
) -> Result<(), String> {
    let operation_key = OperationKey {
        repo_url: url.clone(),
        folder_path: path.clone(),
    };
    let tracker = state.get_operation_tracker();

    tracker.start_operation(&operation_key, OperationType::Delete);
    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Started {
                    operation: OperationType::Delete,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let result = tauri::async_runtime::spawn_blocking(move || delete_addon_files(&url, &path))
        .await
        .map_err(|e| format!("Task join error: {e}"))?;

    tracker.finish_operation(&operation_key);

    let completion_event = match &result {
        Ok(_) => OperationEvent::Completed,
        Err(e) => OperationEvent::Error(e.clone()),
    };
    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key,
                event: completion_event,
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    result
}

#[tauri::command]
pub async fn delete_addon_cmd(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
    path: String,
) -> Result<(), String> {
    let result = perform_delete_op(&app_handle, &state, url, path).await;

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_disk;
    use crate::clone;
    use crate::install;
    use crate::test_utils::{print_dir_tree, setup_addons_dir};
    use crate::validate;

    use std::fs;

    #[test]
    fn test_delete_addon_removes_repo_and_symlinks() {
        let (_temp, addons_dir) = setup_addons_dir();
        let manager_dir =
            validate::ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        // Simulate a cloned repo
        let repo_name = "FakeRepo";
        let repo_dir = manager_dir.join(repo_name);
        fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");
        // Simulate a sub-addon symlink in AddOns
        let symlink_name = "FakeSymlink";
        let symlink_path = addons_dir.join(symlink_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&repo_dir, &symlink_path).expect("Failed to create symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&repo_dir, &symlink_path)
            .expect("Failed to create symlink");

        println!("Before delete_addon_files:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Compose a fake GitHub URL for the repo
        let url = format!("https://github.com/owner/{repo_name}.git");
        let path = addons_dir.to_str().unwrap().to_string();
        // Call delete_addon_files directly
        let result = delete_addon_files(&url, &path);
        assert!(result.is_ok(), "delete_addon_files failed: {result:?}");

        println!("After delete_addon_files:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Assert repo dir is gone
        assert!(
            !repo_dir.exists(),
            "Repo dir was not deleted: {}",
            repo_dir.display()
        );
        // Assert symlink is gone
        assert!(
            !symlink_path.exists(),
            "Symlink was not deleted: {}",
            symlink_path.display()
        );
    }

    #[test]
    fn integration_test_install_and_remove_real_repo() {
        // Setup clean AddOns directory
        let (_temp, addons_dir) = setup_addons_dir();
        let addons_path = addons_dir.to_str().unwrap().to_string();
        let url = "https://github.com/sogladev/addon-335-train-all-button.git".to_string();

        // Perform real installation
        let install_result = install::install_addon(url.clone(), addons_path.clone(), |_| {});
        assert!(
            install_result.is_ok(),
            "install_addon failed: {:?}",
            install_result
        );

        // Validate installed repo directory
        let manager_dir = validate::ensure_manager_dir(&addons_dir).unwrap();
        let (_owner, repo_name) = clone::extract_owner_repo_from_url(&url).unwrap();
        let repo_dir = manager_dir.join(&repo_name);
        assert!(
            repo_dir.exists(),
            "Repo dir missing: {}",
            repo_dir.display()
        );

        // Validate symlinks for sub-addons
        let disk_folder = addon_disk::DiskAddOnsFolder::scan(&addons_path).unwrap();
        let repo_info = disk_folder
            .repositories
            .iter()
            .find(|r| r.repo_url == url)
            .unwrap();
        for addon in &repo_info.addons {
            let link = addons_dir.join(&addon.name);
            assert!(
                link.exists(),
                "Expected symlink for {} after install",
                addon.name
            );
        }

        println!("Before delete_addon_files:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Perform removal
        let remove_result = delete_addon_files(&url, &addons_path);
        assert!(
            remove_result.is_ok(),
            "delete_addon_files failed: {:?}",
            remove_result
        );

        println!("After delete_addon_files:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Assert repo directory is deleted
        assert!(!repo_dir.exists(), "Repo dir still exists after removal");
        // Assert symlinks are deleted
        for addon in &repo_info.addons {
            let link = addons_dir.join(&addon.name);
            // We can't use fs::exists here because it will return true for broken symlinks
            println!(
                "Checking symlink for addon: {} with link: {}",
                addon.name,
                link.display()
            );
            match fs::symlink_metadata(&link) {
                Ok(_) => {
                    assert!(
                        false,
                        "Symlink for {} still exists after removal",
                        addon.name
                    );
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        assert!(true, "Symlink for {} does not exist", addon.name);
                    } else {
                        assert!(
                            false,
                            "Unexpected error checking symlink for {}: {}",
                            addon.name, err
                        );
                    }
                }
            }
        }
    }
}
