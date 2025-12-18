use std::{fs, path::PathBuf};

use tauri::{AppHandle, Emitter};

use crate::{git, operation_reporter::*};

/// Deletes addon repo and symlinks by repo URL/key and AddOns path
/// For Git repos: url is like "https://github.com/owner/repo.git"
/// For Local repos: url is like "local://folder_name" or the actual path
pub fn delete_addon_files(url: &str, path: &str) -> Result<(), String> {
    let addons_dir = PathBuf::from(path);
    let manager_root = addons_dir.join(".addonmanager");

    // Determine if this is a Git repo or local folder
    let repo_name = if url.starts_with("local://") {
        // Extract folder name from local:// URL
        url.strip_prefix("local://")
            .ok_or("Invalid local URL format")?
            .to_string()
    } else if url.starts_with('/') || url.starts_with("C:") || url.starts_with("\\") {
        // Direct path - extract folder name
        PathBuf::from(url)
            .file_name()
            .ok_or("Invalid path format")?
            .to_string_lossy()
            .to_string()
    } else {
        // Git URL - extract repo name
        let (_owner, name) =
            git::extract_owner_repo_from_url(url).map_err(|e| format!("Invalid repo URL: {e}"))?;
        name
    };

    // Remove any symlinks in AddOns whose target is inside this repo
    let repo_dir = manager_root.join(&repo_name);
    if let Ok(entries) = fs::read_dir(&addons_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_symlink() {
                    let p = entry.path();
                    if let Ok(target) = fs::read_link(&p) {
                        if target.starts_with(&repo_dir) {
                            let _ = fs::remove_file(&p);
                        }
                    }
                }
            }
        }
    }
    // Remove repository/folder directory
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }
    Ok(())
}

async fn perform_delete_op(
    app_handle: &AppHandle,
    url: String,
    path: String,
) -> Result<(), String> {
    let operation_key = OperationKey {
        repo_url: url.clone(),
        folder_path: path.clone(),
    };

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
    url: String,
    path: String,
) -> Result<(), String> {
    let result = perform_delete_op(&app_handle, url, path).await;

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_disk;
    use crate::git;
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
        let (_owner, repo_name) = git::extract_owner_repo_from_url(&url).unwrap();
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
            .find(|r| r.get_key() == url)
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

    #[test]
    fn test_delete_local_folder_addon() {
        let (_temp, addons_dir) = setup_addons_dir();
        let addons_path = addons_dir.to_str().unwrap().to_string();
        let manager_dir =
            validate::ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        // Create a local folder addon with a fake .toc file
        let local_folder_name = "MyLocalAddon";
        let local_folder_path = manager_dir.join(local_folder_name);
        fs::create_dir_all(&local_folder_path).expect("Failed to create local folder");

        // Create a .toc file to make it a valid addon
        let toc_file = local_folder_path.join(format!("{}.toc", local_folder_name));
        fs::write(&toc_file, "## Interface: 30300\n## Title: My Local Addon\n")
            .expect("Failed to write .toc file");

        // Create a symlink for the addon
        let symlink_path = addons_dir.join(local_folder_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&local_folder_path, &symlink_path)
            .expect("Failed to create symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&local_folder_path, &symlink_path)
            .expect("Failed to create symlink");

        println!("Before delete_addon_files (local):");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Delete using local:// URL format
        let local_url = format!("local://{}", local_folder_name);
        let result = delete_addon_files(&local_url, &addons_path);
        assert!(result.is_ok(), "delete_addon_files failed: {:?}", result);

        println!("After delete_addon_files (local):");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Assert folder is gone
        assert!(
            !local_folder_path.exists(),
            "Local folder was not deleted: {}",
            local_folder_path.display()
        );
        // Assert symlink is gone
        assert!(
            !symlink_path.exists(),
            "Symlink was not deleted: {}",
            symlink_path.display()
        );
    }

    #[test]
    fn test_delete_local_folder_with_path_format() {
        let (_temp, addons_dir) = setup_addons_dir();
        let addons_path = addons_dir.to_str().unwrap().to_string();
        let manager_dir =
            validate::ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        // Create a local folder addon
        let local_folder_name = "AnotherLocalAddon";
        let local_folder_path = manager_dir.join(local_folder_name);
        fs::create_dir_all(&local_folder_path).expect("Failed to create local folder");

        let toc_file = local_folder_path.join(format!("{}.toc", local_folder_name));
        fs::write(
            &toc_file,
            "## Interface: 30300\n## Title: Another Local Addon\n",
        )
        .expect("Failed to write .toc file");

        let symlink_path = addons_dir.join(local_folder_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&local_folder_path, &symlink_path)
            .expect("Failed to create symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&local_folder_path, &symlink_path)
            .expect("Failed to create symlink");

        // Delete using path format (as stored in DiskAddonSource::Local)
        let result = delete_addon_files(local_folder_path.to_str().unwrap(), &addons_path);
        assert!(result.is_ok(), "delete_addon_files failed: {:?}", result);

        // Assert folder and symlink are gone
        assert!(!local_folder_path.exists(), "Local folder was not deleted");
        assert!(!symlink_path.exists(), "Symlink was not deleted");
    }
}
