//! Export Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{ExportOptions, ExportResult},
};

#[tauri::command]
pub async fn export_note(
    state: State<'_, AppState>,
    note_id: String,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_note(&note_id, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_notes(
    state: State<'_, AppState>,
    note_ids: Vec<String>,
    options: ExportOptions,
) -> Result<Vec<ExportResult>, String> {
    state
        .export_usecases
        .export_notes(&note_ids, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_notebook(
    state: State<'_, AppState>,
    notebook_id: String,
    options: ExportOptions,
) -> Result<Vec<ExportResult>, String> {
    state
        .export_usecases
        .export_notebook(&notebook_id, options)
        .await
        .map_err(|e| e.to_string())
}
