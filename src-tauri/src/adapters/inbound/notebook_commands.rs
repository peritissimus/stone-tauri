//! Notebook Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Notebook,
        ports::inbound::{
            CreateNotebookRequest, DeleteNotebookRequest, ListNotebooksRequest, MoveNotebookRequest,
            NotebookList, NotebookUseCases, UpdateNotebookRequest,
        },
    },
};

#[tauri::command]
pub async fn create_notebook(
    state: State<'_, AppState>,
    request: CreateNotebookRequest,
) -> Result<Notebook, String> {
    state
        .notebook_usecases
        .create_notebook(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_notebook(
    state: State<'_, AppState>,
    request: UpdateNotebookRequest,
) -> Result<Notebook, String> {
    state
        .notebook_usecases
        .update_notebook(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notebook(state: State<'_, AppState>, id: String) -> Result<Notebook, String> {
    state
        .notebook_usecases
        .get_notebook(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_notebooks(
    state: State<'_, AppState>,
    request: ListNotebooksRequest,
) -> Result<NotebookList, String> {
    state
        .notebook_usecases
        .list_notebooks(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_notebook(
    state: State<'_, AppState>,
    request: DeleteNotebookRequest,
) -> Result<(), String> {
    state
        .notebook_usecases
        .delete_notebook(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_notebook(
    state: State<'_, AppState>,
    request: MoveNotebookRequest,
) -> Result<(), String> {
    state
        .notebook_usecases
        .move_notebook(request)
        .await
        .map_err(|e| e.to_string())
}
