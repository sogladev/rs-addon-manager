use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter};

use crate::{addon_disk, git, operation_reporter::*, validate};

pub struct InstallReporter {
    pub event: Box<dyn FnMut(OperationEvent) + Send>,
}

pub fn install_addon<F>(url: String, dir: String, mut reporter: F) -> Result<(), String>
where
    F: FnMut(OperationEvent) + Send,
{
    let dir = Path::new(&dir);

    reporter(OperationEvent::Status(
        "Starting addon installation...".to_string(),
    ));

    let manager_dir = validate::ensure_manager_dir(dir)?;

    reporter(OperationEvent::Status("Cloning repository...".to_string()));
    let (_owner, repo_name) =
        git::extract_owner_repo_from_url(&url).map_err(|e| format!("Invalid repo URL: {e}"))?;
    let repo_path = manager_dir.join(repo_name);

    if repo_path.exists() {
        std::fs::remove_dir_all(&repo_path).ok();
    }

    // Throttle progress events: only emit on 1% increments
    let mut last_percent: u32 = 0;
    let repo = git::clone_git_repo(&url, manager_dir.clone(), &mut |current, total| {
        if total > 0 {
            let percent = ((current as u128 * 100) / total as u128) as u32;
            if percent != last_percent {
                last_percent = percent;
                reporter(OperationEvent::Progress { current, total });
            }
        }
    })
    .map_err(|e| format!("Failed to clone repository from {url}: {e}"))?;

    let path = PathBuf::from(
        repo.workdir()
            .expect("Repository has no workdir. It should not be bare"),
    );

    reporter(OperationEvent::Status(
        "Discovering sub-addons...".to_string(),
    ));
    let disk_repo = addon_disk::create_disk_addon_repository(&path)
        .map_err(|e| format!("Failed to discover sub-addons: {e}"))?;

    reporter(OperationEvent::Status(
        "Installing sub-addons (symlinking)...".to_string(),
    ));
    install_sub_addons(disk_repo.addons, &path, dir, &mut reporter);

    reporter(OperationEvent::Status(
        "Addon installation complete.".to_string(),
    ));
    Ok(())
}

pub fn install_local_folder<F>(
    source_path: String,
    dir: String,
    mut reporter: F,
) -> Result<(), String>
where
    F: FnMut(OperationEvent) + Send,
{
    let source = Path::new(&source_path);
    let dir = Path::new(&dir);

    // Validate source folder exists and contains .toc files
    if !source.exists() {
        return Err(format!(
            "Source folder does not exist: {}",
            source.display()
        ));
    }
    if !source.is_dir() {
        return Err(format!(
            "Source path is not a directory: {}",
            source.display()
        ));
    }

    reporter(OperationEvent::Status(
        "Starting local folder installation...".to_string(),
    ));

    let manager_dir = validate::ensure_manager_dir(dir)?;

    // Get folder name from source path
    let folder_name = source
        .file_name()
        .ok_or("Invalid source path")?
        .to_string_lossy()
        .to_string();

    reporter(OperationEvent::Status(format!(
        "Copying folder '{}' to managed directory...",
        folder_name
    )));

    let dest_path = manager_dir.join(&folder_name);

    // Remove existing folder if it exists
    if dest_path.exists() {
        std::fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("Failed to remove existing folder: {e}"))?;
    }

    // Copy the entire folder to .addonmanager
    copy_dir_recursive(source, &dest_path).map_err(|e| format!("Failed to copy folder: {e}"))?;

    reporter(OperationEvent::Status(
        "Discovering sub-addons...".to_string(),
    ));
    let disk_repo = addon_disk::create_non_git_addon_repository(&dest_path)
        .map_err(|e| format!("Failed to discover sub-addons: {e}"))?;

    reporter(OperationEvent::Status(
        "Installing sub-addons (symlinking)...".to_string(),
    ));
    install_sub_addons(disk_repo.addons, &dest_path, dir, &mut reporter);

    reporter(OperationEvent::Status(
        "Local folder installation complete.".to_string(),
    ));
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn install_sub_addons<F>(
    addons: Vec<addon_disk::DiskAddon>,
    repo_root: &Path,
    addons_dir: &Path,
    mut reporter: F,
) where
    F: FnMut(OperationEvent) + Send,
{
    for addon in addons {
        let symlink_name = &addon.name;
        let target_dir = if addon.dir == "." {
            repo_root.to_path_buf()
        } else {
            repo_root.join(&addon.dir)
        };
        let symlink_path = addons_dir.join(symlink_name);

        if symlink_path.exists() {
            let msg = format!(
                "Removing existing symlink or directory: {}",
                symlink_path.display()
            );
            reporter(OperationEvent::Status(msg.clone()));
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .ok();
        }

        if addon.names.len() > 1 {
            reporter(OperationEvent::Warning(format!(
                "Multiple possible names for sub-addon '{}': {:?}. Using '{symlink_name}'.",
                addon.dir, addon.names
            )));
        }

        reporter(OperationEvent::Status(format!(
            "Creating symlink for '{symlink_name}': {} -> {}",
            target_dir.display(),
            symlink_path.display()
        )));
        if let Err(e) = crate::symlink::create_symlink(&target_dir, &symlink_path) {
            reporter(OperationEvent::Error(format!(
                "Failed to create symlink for '{symlink_name}': {} -> {} ({e})",
                target_dir.display(),
                symlink_path.display(),
            )));
        }
    }
}

#[tauri::command]
pub async fn install_addon_cmd(
    app_handle: tauri::AppHandle,
    url: String,
    path: String,
) -> Result<(), String> {
    // Create operation key for tracking
    let operation_key = OperationKey {
        repo_url: url.clone(),
        folder_path: path.clone(),
    };

    let app_handle_clone = app_handle.clone();
    let operation_key_clone = operation_key.clone();

    // Emit started event
    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Started {
                    operation: OperationType::Install,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let mut first_progress_emitted = false;

    let install_result = tauri::async_runtime::spawn_blocking(move || {
        install_addon(url, path, |event| {
            if let OperationEvent::Progress { .. } = event
                && !first_progress_emitted
            {
                let _ = app_handle.emit("addon-disk-updated", ()).map_err(|e| {
                    eprintln!("Failed to emit addon-disk-updated: {e}");
                });
                first_progress_emitted = true;
            }
            if let Err(e) = app_handle.emit(
                "operation-event",
                OperationEventPayload {
                    key: operation_key.clone(),
                    event,
                },
            ) {
                eprintln!("Failed to emit operation-event: {e}");
            }
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    match install_result {
        Ok(_) => {
            app_handle_clone
                .emit(
                    "operation-event",
                    OperationEventPayload {
                        key: operation_key_clone.clone(),
                        event: OperationEvent::Completed,
                    },
                )
                .map_err(|e| format!("Failed to emit operation-completed: {e}"))?;
            app_handle_clone
                .emit("addon-data-updated", ())
                .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;
            Ok(())
        }
        Err(err_msg) => {
            app_handle_clone
                .emit(
                    "operation-event",
                    OperationEventPayload {
                        key: operation_key_clone.clone(),
                        event: OperationEvent::Error(err_msg.clone()),
                    },
                )
                .map_err(|e| format!("Failed to emit operation-error: {e}"))?;
            Err(err_msg)
        }
    }
}

#[tauri::command]
pub async fn install_local_folder_cmd(
    app_handle: tauri::AppHandle,
    source_path: String,
    path: String,
) -> Result<(), String> {
    // Create operation key for tracking
    let folder_name = std::path::Path::new(&source_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let operation_key = OperationKey {
        repo_url: format!("local://{}", folder_name),
        folder_path: path.clone(),
    };

    let app_handle_clone = app_handle.clone();
    let operation_key_clone = operation_key.clone();

    // Emit started event
    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Started {
                    operation: OperationType::Install,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let install_result = tauri::async_runtime::spawn_blocking(move || {
        install_local_folder(source_path, path, |event| {
            if let Err(e) = app_handle.emit(
                "operation-event",
                OperationEventPayload {
                    key: operation_key.clone(),
                    event,
                },
            ) {
                eprintln!("Failed to emit operation-event: {e}");
            }
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    match install_result {
        Ok(_) => {
            app_handle_clone
                .emit(
                    "operation-event",
                    OperationEventPayload {
                        key: operation_key_clone.clone(),
                        event: OperationEvent::Completed,
                    },
                )
                .map_err(|e| format!("Failed to emit operation-completed: {e}"))?;
            app_handle_clone
                .emit("addon-data-updated", ())
                .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;
            Ok(())
        }
        Err(err_msg) => {
            app_handle_clone
                .emit(
                    "operation-event",
                    OperationEventPayload {
                        key: operation_key_clone.clone(),
                        event: OperationEvent::Error(err_msg.clone()),
                    },
                )
                .map_err(|e| format!("Failed to emit operation-error: {e}"))?;
            Err(err_msg)
        }
    }
}

#[tauri::command]
pub async fn create_addon_symlink(
    app_handle: AppHandle,
    folder_path: String,
    repo_url: String,
    addon_name: String,
    state: tauri::State<'_, crate::addon_discovery::AppState>,
) -> Result<(), String> {
    let operation_key = OperationKey {
        repo_url: repo_url.clone(),
        folder_path: folder_path.clone(),
    };

    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Started {
                    operation: OperationType::Install,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Status(format!("Creating symlink for '{addon_name}'")),
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let result: Result<(), String> = async {
        // Find repo directory
        let disk_state = state
            .get_disk_state()
            .map_err(|e| format!("Disk state error: {e}"))?;
        let folder = disk_state.get(&folder_path).ok_or("Folder not found")?;
        let repo = folder
            .repositories
            .iter()
            .find(|r| r.get_key() == repo_url)
            .ok_or("Repo not found")?;
        let addon = repo
            .addons
            .iter()
            .find(|a| a.name == addon_name)
            .ok_or("Addon not found")?;
        let repo_root = Path::new(&folder_path)
            .join(".addonmanager")
            .join(repo.get_name());
        let addons_dir = Path::new(&folder_path);
        let symlink_name = &addon.name;
        let target_dir = if addon.dir == "." {
            repo_root.to_path_buf()
        } else {
            repo_root.join(&addon.dir)
        };
        let symlink_path = addons_dir.join(symlink_name);
        // Remove any existing symlink or directory
        if symlink_path.exists() {
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .ok();
        }
        crate::symlink::create_symlink(&target_dir, &symlink_path)
            .map_err(|e| format!("Failed to create symlink: {e}"))
    }
    .await;

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
pub async fn remove_addon_symlink(
    app_handle: AppHandle,
    folder_path: String,
    repo_url: String,
    addon_name: String,
) -> Result<(), String> {
    let operation_key = OperationKey {
        repo_url: repo_url.clone(),
        folder_path: folder_path.clone(),
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

    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Status(format!("Removing symlink for '{addon_name}'")),
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let result: Result<(), String> = async {
        let addons_dir = Path::new(&folder_path);
        let symlink_path = addons_dir.join(&addon_name);
        if symlink_path.exists() {
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .map_err(|e| format!("Failed to remove symlink: {e}"))?;
        }
        Ok(())
    }
    .await;

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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_utils::{print_dir_tree, setup_addons_dir};
    use crate::validate;

    #[test]
    fn test_install_clone() {
        let (_temp, addons_dir) = setup_addons_dir();

        let url: String = "https://github.com/sogladev/addon-335-train-all-button.git".into();
        let addons_dir_str = addons_dir.to_str().unwrap();

        let result = validate::ensure_manager_dir(&addons_dir);
        println!("Directory tree under AddOns after ensure_manager_dir:");
        print_dir_tree(addons_dir_str);
        assert!(result.is_ok(), "ensure_manager_dir failed: {:?}", result);

        let result = install_addon(url.clone(), addons_dir_str.to_string(), move |event| {
            println!("Install event: {:?}", event);
        });
        println!("Directory tree under AddOns after install_addon:");
        print_dir_tree(addons_dir_str);
        assert!(result.is_ok(), "install_addon failed: {:?}", result);

        let manager_dir = addons_dir.join(".addonmanager");
        assert!(
            manager_dir.exists() && manager_dir.is_dir(),
            ".addonmanager directory was not created"
        );

        let repo_dir = manager_dir.join("addon-335-train-all-button");
        assert!(
            repo_dir.exists() && repo_dir.is_dir(),
            "Repository was not cloned to the manager directory"
        );

        // Verify that metadata was written correctly by scanning the directory
        let disk_folder = addon_disk::DiskAddOnsFolder::scan(addons_dir_str)
            .expect("Failed to scan addons directory");

        let repo = disk_folder.repositories.iter().find(|r| r.get_key() == url);
        assert!(
            repo.is_some(),
            "AddOns directory should contain the installed repo"
        );
        let repo = repo.unwrap();

        // Verify it's a git repo with correct metadata
        if let addon_disk::DiskAddonSource::Git {
            owner, repo_name, ..
        } = &repo.source
        {
            assert_eq!(owner, "sogladev", "Repo owner mismatch");
            assert_eq!(
                repo_name, "addon-335-train-all-button",
                "Repo name mismatch"
            );
        } else {
            panic!("Expected Git source, found Local");
        }

        assert!(
            !repo.addons.is_empty(),
            "Repo should contain at least one sub-addon"
        );
    }

    #[test]
    fn test_install_sub_addons_symlink_creation() {
        let (_temp, addons_dir) = setup_addons_dir();
        let manager_dir =
            validate::ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        let repo_root = manager_dir.join("fakeowner").join("fakerepo");
        let sub_dir = repo_root.join("SubAddonDir");
        fs::create_dir_all(&sub_dir).expect("Failed to create sub-addon dir");
        let sub_addon = crate::addon_disk::DiskAddon {
            name: "TestSymlink".to_string(),
            dir: "SubAddonDir".to_string(),
            names: vec!["TestSymlink".to_string()],
            is_symlinked: false,
            notes: None,
        };

        println!("Before install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        install_sub_addons(vec![sub_addon], &repo_root, &addons_dir, |_| {});

        println!("After install_sub_addons:");
        print_dir_tree(addons_dir.to_str().unwrap());

        let symlink_path = addons_dir.join("TestSymlink");
        assert!(
            symlink_path.exists(),
            "Symlink was not created: {}",
            symlink_path.display()
        );

        #[cfg(unix)]
        {
            use std::fs;
            let target = fs::read_link(&symlink_path).expect("Failed to read symlink");
            assert!(
                target.ends_with("SubAddonDir"),
                "Symlink does not point to SubAddonDir"
            );
        }
    }

    #[test]
    fn test_install_local_folder() {
        let (_temp_addons, addons_dir) = setup_addons_dir();
        let addons_path = addons_dir.to_str().unwrap().to_string();

        // Create a temporary source folder to "install"
        let temp_source = tempfile::tempdir().expect("Failed to create temp source dir");
        let source_path = temp_source.path();
        let source_folder_name = "MyCustomAddon";
        let source_addon_dir = source_path.join(source_folder_name);
        fs::create_dir_all(&source_addon_dir).expect("Failed to create source addon dir");

        // Create a .toc file to make it valid
        let toc_file = source_addon_dir.join(format!("{}.toc", source_folder_name));
        fs::write(
            &toc_file,
            "## Interface: 30300\n## Title: My Custom Addon\n",
        )
        .expect("Failed to write .toc file");

        println!("Before install_local_folder:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Install the local folder
        let result = install_local_folder(
            source_addon_dir.to_str().unwrap().to_string(),
            addons_path.clone(),
            |_| {},
        );
        assert!(result.is_ok(), "install_local_folder failed: {:?}", result);

        println!("After install_local_folder:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Validate the folder was copied to .addonmanager
        let manager_dir = validate::ensure_manager_dir(&addons_dir).unwrap();
        let installed_folder = manager_dir.join(source_folder_name);
        assert!(
            installed_folder.exists(),
            "Folder was not copied to .addonmanager: {}",
            installed_folder.display()
        );

        // Validate symlink was created
        let symlink_path = addons_dir.join(source_folder_name);
        assert!(
            symlink_path.exists(),
            "Symlink was not created: {}",
            symlink_path.display()
        );
    }

    #[test]
    fn test_repair_local_folder_symlinks() {
        let (_temp, addons_dir) = setup_addons_dir();
        let manager_dir =
            validate::ensure_manager_dir(&addons_dir).expect("Failed to ensure manager dir");

        // Create a local folder with a .toc file in .addonmanager
        let local_folder_name = "LocalAddonForRepair";
        let local_folder_path = manager_dir.join(local_folder_name);
        fs::create_dir_all(&local_folder_path).expect("Failed to create local folder");

        let toc_file = local_folder_path.join(format!("{}.toc", local_folder_name));
        fs::write(&toc_file, "## Interface: 30300\n## Title: Repair Test\n")
            .expect("Failed to write .toc file");

        // Discover the addon
        let disk_repo = addon_disk::create_non_git_addon_repository(&local_folder_path)
            .expect("Failed to discover addon");

        println!("Before repair (install_sub_addons):");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Simulate repair by calling install_sub_addons
        install_sub_addons(
            disk_repo.addons.clone(),
            &local_folder_path,
            &addons_dir,
            |_| {},
        );

        println!("After repair (install_sub_addons):");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Validate symlink was created
        let symlink_path = addons_dir.join(local_folder_name);
        assert!(
            symlink_path.exists(),
            "Symlink was not created during repair: {}",
            symlink_path.display()
        );

        // Now delete the symlink and repair again
        fs::remove_file(&symlink_path).expect("Failed to remove symlink");
        assert!(!symlink_path.exists(), "Symlink should be removed");

        println!("After removing symlink:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Repair again
        install_sub_addons(disk_repo.addons, &local_folder_path, &addons_dir, |_| {});

        println!("After second repair:");
        print_dir_tree(addons_dir.to_str().unwrap());

        // Validate symlink was recreated
        assert!(
            symlink_path.exists(),
            "Symlink was not recreated during second repair: {}",
            symlink_path.display()
        );
    }
}
