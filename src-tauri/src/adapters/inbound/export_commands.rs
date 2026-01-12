//! Export Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{ExportOptions, ExportResult},
};

#[tauri::command]
pub async fn export_note_html(
    state: State<'_, AppState>,
    note_id: String,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_html(&note_id, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_note_pdf(
    state: State<'_, AppState>,
    note_id: String,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_pdf(&note_id, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_note_markdown(
    state: State<'_, AppState>,
    note_id: String,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_markdown(&note_id, options)
        .await
        .map_err(|e| e.to_string())
}
