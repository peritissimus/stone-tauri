//! Quick Capture Command Handlers

use tauri::{AppHandle, State};

use crate::{
    adapters::inbound::{app_state::AppState, ui::quick_capture_window},
    domain::ports::inbound::AppendToJournalResponse,
};

#[tauri::command]
pub fn log_from_frontend(message: String, level: Option<String>) {
    match level.as_deref() {
        Some("error") => tracing::error!("[Frontend] {}", message),
        Some("warn") => tracing::warn!("[Frontend] {}", message),
        Some("debug") => tracing::debug!("[Frontend] {}", message),
        _ => tracing::info!("[Frontend] {}", message),
    }
}

#[tauri::command]
pub async fn append_to_journal(
    state: State<'_, AppState>,
    content: String,
    workspace_id: Option<String>,
) -> Result<AppendToJournalResponse, String> {
    state
        .quick_capture_usecases
        .append_to_journal(&content, workspace_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_quick_capture(app: AppHandle) -> quick_capture_window::PanelOperationResult {
    quick_capture_window::hide(&app)
}

#[tauri::command]
pub fn get_quick_capture_state(app: AppHandle) -> String {
    quick_capture_window::get_state(&app).to_string()
}
