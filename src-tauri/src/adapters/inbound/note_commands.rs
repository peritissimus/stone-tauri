//! Note Command Handlers
//!
//! Tauri command handlers for note operations.
//! Maps frontend requests to note use cases.

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Note,
        ports::inbound::{CreateNoteInput, NoteQuery, UpdateNoteInput},
    },
};

/// Create a new note
#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    input: CreateNoteInput,
) -> Result<Note, String> {
    state
        .note_usecases
        .create_note(input)
        .await
        .map_err(|e| e.to_string())
}

/// Get a note by ID
#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Option<Note>, String> {
    state
        .note_usecases
        .get_note_by_id(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Response for get_note_content
#[derive(Debug, Clone, serde::Serialize)]
pub struct GetNoteContentResponse {
    pub content: String,
}

/// Get note content by ID
#[tauri::command]
pub async fn get_note_content(
    state: State<'_, AppState>,
    id: String,
) -> Result<GetNoteContentResponse, String> {
    tracing::info!("get_note_content called with id: {}", id);

    let content = state
        .note_usecases
        .get_note_content(&id)
        .await
        .map_err(|e| {
            tracing::error!("get_note_content error: {}", e);
            e.to_string()
        })?;

    tracing::info!("get_note_content success: loaded {} bytes", content.len());
    Ok(GetNoteContentResponse { content })
}

/// Save note content
#[tauri::command]
pub async fn save_note_content(
    state: State<'_, AppState>,
    id: String,
    content: String,
) -> Result<(), String> {
    state
        .note_usecases
        .save_note_content(&id, &content)
        .await
        .map_err(|e| e.to_string())
}

/// Update a note
#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    input: UpdateNoteInput,
) -> Result<Note, String> {
    state
        .note_usecases
        .update_note(input)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a note (soft delete by default, permanent if specified)
#[tauri::command]
pub async fn delete_note(
    state: State<'_, AppState>,
    id: String,
    permanent: Option<bool>,
) -> Result<(), String> {
    if permanent.unwrap_or(false) {
        state
            .note_usecases
            .permanently_delete_note(&id)
            .await
            .map_err(|e| e.to_string())
    } else {
        state
            .note_usecases
            .delete_note(&id)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Restore a deleted note
#[tauri::command]
pub async fn restore_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .note_usecases
        .restore_note(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Response for get_all_notes
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllNotesResponse {
    pub notes: Vec<Note>,
}

/// Get all notes with optional filtering
#[tauri::command]
pub async fn get_all_notes(
    state: State<'_, AppState>,
    query: Option<NoteQuery>,
) -> Result<GetAllNotesResponse, String> {
    let query = query.unwrap_or(NoteQuery {
        workspace_id: None,
        notebook_id: None,
        is_favorite: None,
        is_archived: None,
        limit: None,
    });

    let notes = state
        .note_usecases
        .get_all_notes(query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetAllNotesResponse { notes })
}

/// Move a note to a different notebook
#[tauri::command]
pub async fn move_note(
    state: State<'_, AppState>,
    id: String,
    target_notebook_id: Option<String>,
) -> Result<(), String> {
    state
        .note_usecases
        .move_to_notebook(&id, target_notebook_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get a note by its file path
#[tauri::command]
pub async fn get_note_by_path(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Option<Note>, String> {
    tracing::info!("get_note_by_path called with: {}", file_path);

    // Get active workspace to normalize the path
    let workspace = state
        .workspace_usecases
        .get_active_workspace()
        .await
        .map_err(|e| e.to_string())?;

    let normalized_path = if let Some(ws) = workspace {
        // Normalize both paths by removing leading/trailing slashes
        let workspace_normalized = ws.folder_path.trim_start_matches('/').trim_end_matches('/');
        let file_normalized = file_path.trim_start_matches('/');

        // Try to strip workspace path prefix
        let normalized = if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
            relative.trim_start_matches('/').to_string()
        } else {
            file_normalized.to_string()
        };

        tracing::info!(
            "Normalized path: '{}' -> '{}' (workspace: '{}')",
            file_path,
            normalized,
            workspace_normalized
        );
        normalized
    } else {
        file_path.trim_start_matches('/').to_string()
    };

    let result = state
        .note_usecases
        .get_note_by_path(&normalized_path)
        .await
        .map_err(|e| {
            tracing::error!("get_note_by_path error: {}", e);
            e.to_string()
        });

    if let Ok(ref note) = result {
        if let Some(n) = note {
            tracing::info!("get_note_by_path success: found note id={}", n.id);
        } else {
            tracing::warn!("get_note_by_path: no note found for path '{}'", normalized_path);
        }
    }

    result
}

/// Toggle favorite status
#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    state
        .note_usecases
        .toggle_favorite(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Toggle pin status
#[tauri::command]
pub async fn toggle_pin(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    state
        .note_usecases
        .toggle_pin(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Archive a note
#[tauri::command]
pub async fn archive_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .note_usecases
        .archive_note(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Unarchive a note
#[tauri::command]
pub async fn unarchive_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .note_usecases
        .unarchive_note(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get recent notes
#[tauri::command]
pub async fn get_recent_notes(
    state: State<'_, AppState>,
    limit: i64,
    workspace_id: Option<String>,
) -> Result<Vec<Note>, String> {
    state
        .note_usecases
        .get_recent_notes(limit, workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get favorite notes
#[tauri::command]
pub async fn get_favorites(
    state: State<'_, AppState>,
    workspace_id: Option<String>,
) -> Result<Vec<Note>, String> {
    state
        .note_usecases
        .get_favorites(workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get archived notes
#[tauri::command]
pub async fn get_archived(
    state: State<'_, AppState>,
    workspace_id: Option<String>,
) -> Result<Vec<Note>, String> {
    state
        .note_usecases
        .get_archived(workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get deleted notes (trash)
#[tauri::command]
pub async fn get_trash(
    state: State<'_, AppState>,
    workspace_id: Option<String>,
) -> Result<Vec<Note>, String> {
    state
        .note_usecases
        .get_trash(workspace_id)
        .await
        .map_err(|e| e.to_string())
}

// Commands are exported individually and registered in lib.rs
