//! Quick Capture Command Handlers

use tauri::{AppHandle, State};

use crate::{
    adapters::inbound::{app_state::AppState, ui::quick_capture_window},
    domain::ports::inbound::AppendToJournalResponse,
};

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
pub async fn hide_quick_capture(app: AppHandle) -> Result<(), String> {
    quick_capture_window::hide(&app).map_err(|e| e.to_string())
}
