//! Export Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{ExportOptions, ExportResult},
};

#[tauri::command]
pub async fn export_note_html(
    state: State<'_, AppState>,
    id: String,
    rendered_html: Option<String>,
    title: Option<String>,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_html(&id, rendered_html, title, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_note_pdf(
    state: State<'_, AppState>,
    id: String,
    rendered_html: Option<String>,
    title: Option<String>,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_pdf(&id, rendered_html, title, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_note_markdown(
    state: State<'_, AppState>,
    id: String,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    state
        .export_usecases
        .export_markdown(&id, options)
        .await
        .map_err(|e| e.to_string())
}
