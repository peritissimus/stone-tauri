//! Graph Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::ports::inbound::{GraphData, GraphDataOptions, NoteLinkInfo},
};

#[tauri::command]
pub async fn get_backlinks(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<NoteLinkInfo>, String> {
    state
        .graph_usecases
        .get_backlinks(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_forward_links(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<NoteLinkInfo>, String> {
    state
        .graph_usecases
        .get_forward_links(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_graph_data(
    state: State<'_, AppState>,
    options: Option<GraphDataOptions>,
) -> Result<GraphData, String> {
    state
        .graph_usecases
        .get_graph_data(options)
        .await
        .map_err(|e| e.to_string())
}
