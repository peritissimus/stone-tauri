//! Notebook Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Notebook,
        ports::{
            inbound::{
                CreateNotebookRequest, DeleteNotebookRequest, ListNotebooksRequest,
                MoveNotebookRequest, NotebookList, UpdateNotebookRequest,
            },
            outbound::NotebookWithCount,
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

/// Response for list_notebooks
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotebooksResponse {
    pub notebooks: Vec<NotebookWithCount>,
}

#[tauri::command]
pub async fn list_notebooks(
    state: State<'_, AppState>,
    request: ListNotebooksRequest,
) -> Result<ListNotebooksResponse, String> {
    let result = state
        .notebook_usecases
        .list_notebooks(request)
        .await
        .map_err(|e| e.to_string())?;

    // Extract notebooks from the NotebookList enum
    let notebooks = match result {
        NotebookList::WithCount(notebooks) => notebooks,
        NotebookList::WithoutCount(notebooks) => {
            // Convert Notebook to NotebookWithCount with zero count
            notebooks
                .into_iter()
                .map(|n| NotebookWithCount {
                    id: n.id,
                    name: n.name,
                    workspace_id: n.workspace_id,
                    parent_id: n.parent_id,
                    folder_path: n.folder_path,
                    position: n.position,
                    icon: Some(n.icon),
                    color: Some(n.color),
                    description: None, // Notebook entity doesn't have description
                    created_at: n.created_at,
                    updated_at: n.updated_at,
                    note_count: 0,
                })
                .collect()
        }
    };

    Ok(ListNotebooksResponse { notebooks })
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
