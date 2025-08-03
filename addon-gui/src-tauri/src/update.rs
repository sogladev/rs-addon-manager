use git2::{FetchOptions, Repository, ResetType};
use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::{addon_discovery::AppState, clone, operation_tracker::*, validate};

/// Perform a forced update of the repository at the given path and branch.
/// Fetches from origin, force resets local branch to remote HEAD.
pub fn update_addon_repo(path: &str, url: &str, branch: &str) -> Result<(), String> {
    // Ensure manager dir exists
    let addons_dir = Path::new(path);
    let manager_dir = validate::ensure_manager_dir(addons_dir)
        .map_err(|e| format!("Failed to ensure manager dir: {e}"))?;
    // Determine repo name from URL
    let (_owner, repo_name) =
        clone::extract_owner_repo_from_url(url).map_err(|e| format!("Invalid repo URL: {e}"))?;
    let repo_dir = manager_dir.join(&repo_name);

    // Open existing repo
    let repo = Repository::open(&repo_dir)
        .map_err(|e| format!("Failed to open repo {}: {e}", repo_dir.display()))?;

    // Use HTTPS anonymous fetch for public repositories
    let mut fo = FetchOptions::new();

    // Fetch the specified branch
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote: {e}"))?;
    remote
        .fetch(&[branch], Some(&mut fo), None)
        .map_err(|e| format!("Fetch failed: {e}"))?;

    // Get the fetched commit
    let remote_ref = format!("refs/remotes/origin/{branch}");
    let commit = repo
        .find_reference(&remote_ref)
        .and_then(|r| r.peel_to_commit())
        .map_err(|e| format!("Failed to get remote HEAD: {e}"))?;
    let oid = commit.id();

    // Update local branch ref
    let local_ref = format!("refs/heads/{branch}");
    repo.reference(&local_ref, oid, true, "force update")
        .map_err(|e| format!("Failed to update branch ref: {e}"))?;

    // Hard reset working tree
    repo.reset(commit.as_object(), ResetType::Hard, None)
        .map_err(|e| format!("Failed to reset repo: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn update_addon_cmd(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
    path: String,
    branch: String,
) -> Result<(), String> {
    // Create operation key for tracking
    let operation_key = OperationKey::new(url.clone(), path.clone());
    let tracker = state.get_operation_tracker();

    // Mark operation as started
    tracker.start_operation(&operation_key, OperationType::Update);

    let app_clone = app_handle.clone();
    let operation_key_clone = operation_key.clone();

    // Emit started event
    app_handle
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key.clone(),
                event: OperationEvent::Started {
                    operation: OperationType::Update,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    // Move the blocking work into spawn_blocking
    let update_result =
        tauri::async_runtime::spawn_blocking(move || update_addon_repo(&path, &url, &branch))
            .await
            .map_err(|e| format!("Task join error: {e}"))?;

    // Mark operation as completed
    tracker.finish_operation(&operation_key_clone);

    // Emit completion event
    let completion_event = match update_result {
        Ok(()) => OperationEvent::Completed,
        Err(ref e) => OperationEvent::Error(e.clone()),
    };

    app_clone
        .emit(
            "operation-event",
            OperationEventPayload {
                key: operation_key_clone.clone(),
                event: completion_event,
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    // Signal frontend to refresh data
    app_clone
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    update_result
}

/// Tauri command to update all addons across all folders
#[tauri::command]
pub async fn update_all_addons_cmd(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let app_clone = app_handle.clone();

    let update_tasks = {
        let disk_state = state.get_disk_state()?;
        let mut tasks = Vec::new();

        for (folder_path, disk_folder) in disk_state.iter() {
            for repo in &disk_folder.repositories {
                if let (Some(local_ref), Some(latest_ref)) = (&repo.repo_ref, &repo.latest_ref) {
                    if local_ref != latest_ref {
                        if let Some(branch) = &repo.current_branch {
                            tasks.push((
                                folder_path.clone(),
                                repo.repo_url.clone(),
                                branch.clone(),
                            ));
                        }
                    }
                }
            }
        }
        tasks
    };

    // @todo: Add concurrency here
    let mut updated_count = 0;
    let total_count = update_tasks.len();

    for (path, url, branch) in update_tasks {
        let update_result = tauri::async_runtime::spawn_blocking({
            let path = path.clone();
            let url = url.clone();
            let branch = branch.clone();
            move || update_addon_repo(&path, &url, &branch)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?;

        if let Err(e) = update_result {
            eprintln!("Update failed for {}: {e}", url);
        } else {
            updated_count += 1;
        }
    }

    // Signal frontend to refresh data
    app_clone
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    // Emit summary
    app_clone
        .emit(
            "update-all-complete",
            format!("Updated {}/{} addons", updated_count, total_count),
        )
        .map_err(|e| format!("Failed to emit update-all-complete: {e}"))?;

    Ok(())
}
