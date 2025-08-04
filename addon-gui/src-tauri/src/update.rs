use git2::{FetchOptions, Repository, ResetType};
use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::{addon_discovery::AppState, clone, operation_tracker::*, validate};

/// Perform a forced update of the repository at the given path and branch.
/// Fetches from origin, force resets local branch to remote HEAD.
fn update_addon_repo(path: &str, url: &str, branch: &str) -> Result<(), String> {
    let addons_dir = Path::new(path);
    let manager_dir = validate::ensure_manager_dir(addons_dir)
        .map_err(|e| format!("Failed to ensure manager dir: {e}"))?;

    let (_owner, repo_name) =
        clone::extract_owner_repo_from_url(url).map_err(|e| format!("Invalid repo URL: {e}"))?;
    let repo_dir = manager_dir.join(&repo_name);

    let repo = Repository::open(&repo_dir)
        .map_err(|e| format!("Failed to open repo {}: {e}", repo_dir.display()))?;

    // Use HTTPS anonymous fetch for public repositories
    let mut fo = FetchOptions::new();

    let branch_name = branch.strip_prefix("origin/").unwrap_or(branch);

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote: {e}"))?;
    remote
        .fetch(&[branch_name], Some(&mut fo), None)
        .map_err(|e| format!("Fetch failed: {e}"))?;

    // Get the fetched commit
    let remote_ref = format!("refs/remotes/origin/{branch_name}");
    let commit = repo
        .find_reference(&remote_ref)
        .and_then(|r| r.peel_to_commit())
        .map_err(|e| format!("Failed to get remote HEAD: {e}"))?;
    let oid = commit.id();

    let local_ref = format!("refs/heads/{branch_name}");
    repo.reference(&local_ref, oid, true, "force update")
        .map_err(|e| format!("Failed to update branch ref: {e}"))?;

    // Checkout the branch so HEAD points to it
    let obj = repo
        .revparse_single(&local_ref)
        .map_err(|e| format!("Failed to revparse branch for checkout: {e}"))?;
    repo.checkout_tree(&obj, None)
        .map_err(|e| format!("Failed to checkout tree: {e}"))?;
    repo.set_head(&local_ref)
        .map_err(|e| format!("Failed to set HEAD: {e}"))?;

    // Hard reset working tree
    repo.reset(commit.as_object(), ResetType::Hard, None)
        .map_err(|e| format!("Failed to reset repo: {e}"))?;

    Ok(())
}

async fn perform_update_op(
    app_handle: &AppHandle,
    state: &tauri::State<'_, AppState>,
    url: String,
    path: String,
    branch: String,
) -> Result<(), String> {
    let operation_key = OperationKey {
        repo_url: url.clone(),
        folder_path: path.clone(),
    };
    let tracker = state.get_operation_tracker();

    tracker.start_operation(&operation_key, OperationType::Update);
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

    let result =
        tauri::async_runtime::spawn_blocking(move || update_addon_repo(&path, &url, &branch))
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
pub async fn update_addon_cmd(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
    path: String,
    branch: String,
) -> Result<(), String> {
    let result = perform_update_op(&app_handle, &state, url, path, branch).await;

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    result
}

/// Tauri command to update all addons across all folders
#[tauri::command]
pub async fn update_all_addons_cmd(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
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

    let total_count = update_tasks.len();
    let mut updated_count = 0;

    for (path, url, branch) in update_tasks {
        let result = perform_update_op(&app_handle, &state, url.clone(), path, branch).await;
        if let Ok(()) = result {
            updated_count += 1;
        } else if let Err(e) = result {
            eprintln!("Update failed for {}: {e}", url);
        }
    }

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    app_handle
        .emit(
            "update-all-complete",
            format!("Updated {}/{} addons", updated_count, total_count),
        )
        .map_err(|e| format!("Failed to emit update-all-complete: {e}"))?;

    Ok(())
}
