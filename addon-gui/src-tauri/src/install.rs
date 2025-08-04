use std::path::{Path, PathBuf};

use tauri::Emitter;

use crate::{addon_disk, clone, operation_tracker::*, validate};

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

    let manager_dir = validate::ensure_manager_dir(dir).map_err(|e| e)?;

    reporter(OperationEvent::Status("Cloning repository...".to_string()));
    let repo_name = url
        .split('/')
        .last()
        .and_then(|s| s.strip_suffix(".git"))
        .unwrap_or("repo");
    let repo_path = manager_dir.join(repo_name);

    if repo_path.exists() {
        std::fs::remove_dir_all(&repo_path).ok();
    }

    // Throttle progress events: only emit on 1% increments
    let mut last_percent: u32 = 0;
    let repo = clone::clone_git_repo(&url, manager_dir.clone(), &mut |current, total| {
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
    state: tauri::State<'_, crate::addon_discovery::AppState>,
    url: String,
    path: String,
) -> Result<(), String> {
    // Create operation key for tracking
    let operation_key = OperationKey {
        repo_url: url.clone(),
        folder_path: path.clone(),
    };
    let tracker = state.get_operation_tracker();

    // Mark operation as started
    tracker.start_operation(&operation_key, OperationType::Install);

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
            if let OperationEvent::Progress { .. } = event {
                if !first_progress_emitted {
                    let _ = app_handle.emit("addon-disk-updated", ()).map_err(|e| {
                        eprintln!("Failed to emit addon-disk-updated: {e}");
                    });
                    first_progress_emitted = true;
                }
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
            tracker.finish_operation(&operation_key_clone);
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
            tracker.finish_operation(&operation_key_clone);
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
    folder_path: String,
    repo_url: String,
    addon_name: String,
    state: tauri::State<'_, crate::addon_discovery::AppState>,
) -> Result<(), String> {
    // Find repo directory
    let disk_state = state
        .get_disk_state()
        .map_err(|e| format!("Disk state error: {e}"))?;
    let folder = disk_state.get(&folder_path).ok_or("Folder not found")?;
    let repo = folder
        .repositories
        .iter()
        .find(|r| r.repo_url == repo_url)
        .ok_or("Repo not found")?;
    let addon = repo
        .addons
        .iter()
        .find(|a| a.name == addon_name)
        .ok_or("Addon not found")?;
    let repo_root = Path::new(&folder_path)
        .join(".addonmanager")
        .join(&repo.repo_name);
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

#[tauri::command]
pub async fn remove_addon_symlink(
    folder_path: String,
    _repo_url: String,
    addon_name: String,
    _state: tauri::State<'_, crate::addon_discovery::AppState>,
) -> Result<(), String> {
    let addons_dir = Path::new(&folder_path);
    let symlink_path = addons_dir.join(&addon_name);
    if symlink_path.exists() {
        std::fs::remove_file(&symlink_path)
            .or_else(|_| std::fs::remove_dir_all(&symlink_path))
            .ok();
    }
    Ok(())
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

        let repo = disk_folder.repositories.iter().find(|r| r.repo_url == url);
        assert!(
            repo.is_some(),
            "AddOns directory should contain the installed repo"
        );
        let repo = repo.unwrap();
        assert_eq!(repo.owner, "sogladev", "Repo owner mismatch");
        assert_eq!(
            repo.repo_name, "addon-335-train-all-button",
            "Repo name mismatch"
        );
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
}
