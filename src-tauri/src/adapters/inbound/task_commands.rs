//! Task Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        ports::inbound::TaskItem,
        services::TaskState,
    },
};

#[tauri::command]
pub async fn get_all_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<TaskItem>, String> {
    state
        .task_usecases
        .get_all_tasks()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note_tasks(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<TaskItem>, String> {
    state
        .task_usecases
        .get_note_tasks(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task_state(
    state: State<'_, AppState>,
    note_id: String,
    task_index: i32,
    new_state: TaskState,
) -> Result<(), String> {
    state
        .task_usecases
        .update_task_state(&note_id, task_index, new_state)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_task(
    state: State<'_, AppState>,
    note_id: String,
    task_index: i32,
) -> Result<(), String> {
    state
        .task_usecases
        .toggle_task(&note_id, task_index)
        .await
        .map_err(|e| e.to_string())
}
