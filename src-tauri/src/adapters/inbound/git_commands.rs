//! Git Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{GitCommitInfo, GitStatusResponse, GitSyncResponse},
};

/// Response for git_get_history (get_commits)
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCommitsResponse {
    pub commits: Vec<GitCommitInfo>,
}

/// Response for git operations that return success boolean
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOperationResponse {
    pub success: bool,
}

#[tauri::command]
pub async fn get_git_status(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitStatusResponse, String> {
    state
        .git_usecases
        .get_status(&workspace_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_init(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitOperationResponse, String> {
    let success = state
        .git_usecases
        .init(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GitOperationResponse { success })
}

#[tauri::command]
pub async fn git_commit(
    state: State<'_, AppState>,
    workspace_id: String,
    message: Option<String>,
) -> Result<Option<GitCommitInfo>, String> {
    state
        .git_usecases
        .commit(&workspace_id, message.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_pull(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitSyncResponse, String> {
    let success = state
        .git_usecases
        .pull(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GitSyncResponse {
        success,
        pulled: 0,
        pushed: 0,
        conflicts: vec![],
        error: None,
    })
}

#[tauri::command]
pub async fn git_push(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitSyncResponse, String> {
    let success = state
        .git_usecases
        .push(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GitSyncResponse {
        success,
        pulled: 0,
        pushed: 0,
        conflicts: vec![],
        error: None,
    })
}

#[tauri::command]
pub async fn git_sync(
    state: State<'_, AppState>,
    workspace_id: String,
    message: Option<String>,
) -> Result<GitSyncResponse, String> {
    state
        .git_usecases
        .sync(&workspace_id, message.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_set_remote(
    state: State<'_, AppState>,
    workspace_id: String,
    url: String,
) -> Result<GitOperationResponse, String> {
    let success = state
        .git_usecases
        .set_remote(&workspace_id, &url)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GitOperationResponse { success })
}

#[tauri::command]
pub async fn git_get_history(
    state: State<'_, AppState>,
    workspace_id: String,
    limit: Option<i32>,
) -> Result<GetCommitsResponse, String> {
    let commits = state
        .git_usecases
        .get_commits(&workspace_id, limit)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetCommitsResponse { commits })
}
