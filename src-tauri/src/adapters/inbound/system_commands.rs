//! System Command Handlers

use tauri::State;

use crate::adapters::inbound::app_state::AppState;

#[tauri::command]
pub async fn get_system_fonts(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .system_usecases
        .get_fonts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_external_url(state: State<'_, AppState>, url: String) -> Result<(), String> {
    state
        .system_usecases
        .open_external(&url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_in_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state
        .system_usecases
        .open_in_folder(&path)
        .map_err(|e| e.to_string())
}
