use git2::{FetchOptions, Repository, ResetType};
use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::{addon_discovery::AppState, git, operation_reporter::*, validate};

/// Perform a forced update of the repository at the given path and branch.
/// Fetches from origin, force resets local branch to remote HEAD.
fn update_addon_repo(path: &str, url: &str, branch: &str) -> Result<(), String> {
    let addons_dir = Path::new(path);
    let manager_dir = validate::ensure_manager_dir(addons_dir)
        .map_err(|e| format!("Failed to ensure manager dir: {e}"))?;

    let (_owner, repo_name) =
        git::extract_owner_repo_from_url(url).map_err(|e| format!("Invalid repo URL: {e}"))?;
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
    url: String,
    path: String,
    branch: String,
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
                    operation: OperationType::Update,
                },
            },
        )
        .map_err(|e| format!("Failed to emit operation-event: {e}"))?;

    let result =
        tauri::async_runtime::spawn_blocking(move || update_addon_repo(&path, &url, &branch))
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
pub async fn update_addon_cmd(
    app_handle: AppHandle,
    url: String,
    path: String,
    branch: String,
) -> Result<(), String> {
    let result = perform_update_op(&app_handle, url.clone(), path, branch).await;
    if let Ok(()) = result {
    } else if let Err(e) = result {
        eprintln!("Update failed for {url}: {e}");
    }

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    Ok(())
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
                // Only check for updates if it's a Git repository
                if let crate::addon_disk::DiskAddonSource::Git {
                    repo_url,
                    current_branch,
                    repo_ref,
                    latest_ref,
                    ..
                } = &repo.source
                    && let (Some(local_ref), Some(remote_ref)) = (repo_ref, latest_ref)
                    && local_ref != remote_ref
                    && let Some(branch) = current_branch
                {
                    tasks.push((folder_path.clone(), repo_url.clone(), branch.clone()));
                }
            }
        }
        tasks
    };

    for (path, url, branch) in update_tasks {
        let result = perform_update_op(&app_handle, url.clone(), path, branch).await;
        if let Ok(()) = result {
        } else if let Err(e) = result {
            eprintln!("Update failed for {url}: {e}");
        }
    }

    app_handle
        .emit("addon-data-updated", ())
        .map_err(|e| format!("Failed to emit addon-data-updated: {e}"))?;

    Ok(())
}
