//! Git Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{GitCommitInfo, GitStatusResponse, GitSyncResponse},
};

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
) -> Result<bool, String> {
    state
        .git_usecases
        .init(&workspace_id)
        .await
        .map_err(|e| e.to_string())
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
) -> Result<bool, String> {
    state
        .git_usecases
        .set_remote(&workspace_id, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_get_history(
    state: State<'_, AppState>,
    workspace_id: String,
    limit: Option<i32>,
) -> Result<Vec<GitCommitInfo>, String> {
    state
        .git_usecases
        .get_commits(&workspace_id, limit)
        .await
        .map_err(|e| e.to_string())
}
