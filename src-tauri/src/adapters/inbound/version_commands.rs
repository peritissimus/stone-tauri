//! Version Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::entities::Version,
};

#[tauri::command]
pub async fn get_versions(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<Version>, String> {
    state
        .version_usecases
        .get_versions(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<Option<Version>, String> {
    state
        .version_usecases
        .get_version(&version_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_version(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Version, String> {
    state
        .version_usecases
        .create_version(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_version(
    state: State<'_, AppState>,
    note_id: String,
    version_id: String,
) -> Result<(), String> {
    state
        .version_usecases
        .restore_version(&note_id, &version_id)
        .await
        .map_err(|e| e.to_string())
}
