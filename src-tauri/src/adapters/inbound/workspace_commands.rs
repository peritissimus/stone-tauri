//! Workspace Command Handlers
//!
//! Tauri command handlers for workspace operations.
//! Maps frontend requests to workspace use cases.

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Workspace,
        ports::inbound::{
            CreateFolderRequest, CreateFolderResponse, CreateWorkspaceRequest, MoveFolderRequest,
            MoveFolderResponse, RenameFolderRequest, RenameFolderResponse, ScanWorkspaceResponse,
            SelectFolderRequest, SelectFolderResponse, SyncWorkspaceResponse,
            UpdateWorkspaceRequest, ValidatePathResponse,
        },
    },
};

/// Create a new workspace
#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    request: CreateWorkspaceRequest,
) -> Result<Workspace, String> {
    state
        .workspace_usecases
        .create_workspace(request)
        .await
        .map_err(|e| e.to_string())
}

/// Get a workspace by ID
#[tauri::command]
pub async fn get_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<Workspace, String> {
    state
        .workspace_usecases
        .get_workspace(&id)
        .await
        .map_err(|e| e.to_string())
}

/// List all workspaces
#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    state
        .workspace_usecases
        .list_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Set active workspace
#[tauri::command]
pub async fn set_active_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<Workspace, String> {
    state
        .workspace_usecases
        .set_active_workspace(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get active workspace
#[tauri::command]
pub async fn get_active_workspace(
    state: State<'_, AppState>,
) -> Result<Option<Workspace>, String> {
    state
        .workspace_usecases
        .get_active_workspace()
        .await
        .map_err(|e| e.to_string())
}

/// Delete a workspace
#[tauri::command]
pub async fn delete_workspace(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .workspace_usecases
        .delete_workspace(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Update a workspace
#[tauri::command]
pub async fn update_workspace(
    state: State<'_, AppState>,
    request: UpdateWorkspaceRequest,
) -> Result<Workspace, String> {
    state
        .workspace_usecases
        .update_workspace(request)
        .await
        .map_err(|e| e.to_string())
}

/// Show folder selection dialog
#[tauri::command]
pub async fn select_folder(
    state: State<'_, AppState>,
    request: Option<SelectFolderRequest>,
) -> Result<SelectFolderResponse, String> {
    state
        .workspace_usecases
        .select_folder(request)
        .await
        .map_err(|e| e.to_string())
}

/// Validate a folder path
#[tauri::command]
pub async fn validate_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<ValidatePathResponse, String> {
    state
        .workspace_usecases
        .validate_path(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Scan workspace for markdown files
#[tauri::command]
pub async fn scan_workspace(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<ScanWorkspaceResponse, String> {
    state
        .workspace_usecases
        .scan_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Sync workspace with filesystem
#[tauri::command]
pub async fn sync_workspace(
    state: State<'_, AppState>,
    workspace_id: Option<String>,
) -> Result<SyncWorkspaceResponse, String> {
    state
        .workspace_usecases
        .sync_workspace(workspace_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Create a folder in the workspace
#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    request: CreateFolderRequest,
) -> Result<CreateFolderResponse, String> {
    state
        .workspace_usecases
        .create_folder(request)
        .await
        .map_err(|e| e.to_string())
}

/// Rename a folder
#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    request: RenameFolderRequest,
) -> Result<RenameFolderResponse, String> {
    state
        .workspace_usecases
        .rename_folder(request)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a folder
#[tauri::command]
pub async fn delete_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state
        .workspace_usecases
        .delete_folder(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Move a folder to a different location
#[tauri::command]
pub async fn move_folder(
    state: State<'_, AppState>,
    request: MoveFolderRequest,
) -> Result<MoveFolderResponse, String> {
    state
        .workspace_usecases
        .move_folder(request)
        .await
        .map_err(|e| e.to_string())
}

// Commands are exported individually and registered in lib.rs
