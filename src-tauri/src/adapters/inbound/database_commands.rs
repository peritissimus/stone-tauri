//! Database Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{DatabaseStatus, IntegrityCheckResult},
};

#[tauri::command]
pub async fn get_database_status(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    state
        .database_usecases
        .get_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vacuum_database(state: State<'_, AppState>) -> Result<(), String> {
    state
        .database_usecases
        .vacuum()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_database_integrity(
    state: State<'_, AppState>,
) -> Result<IntegrityCheckResult, String> {
    state
        .database_usecases
        .check_integrity()
        .await
        .map_err(|e| e.to_string())
}
