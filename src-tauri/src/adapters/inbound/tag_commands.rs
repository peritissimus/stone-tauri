//! Tag Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Tag,
        ports::inbound::{CreateTagRequest, ListTagsRequest, TagList, UpdateTagRequest},
    },
};

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    request: CreateTagRequest,
) -> Result<Tag, String> {
    state
        .tag_usecases
        .create_tag(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    request: UpdateTagRequest,
) -> Result<Tag, String> {
    state
        .tag_usecases
        .update_tag(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tag(state: State<'_, AppState>, id: String) -> Result<Tag, String> {
    state
        .tag_usecases
        .get_tag(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_tags(
    state: State<'_, AppState>,
    request: ListTagsRequest,
) -> Result<TagList, String> {
    state
        .tag_usecases
        .list_tags(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .tag_usecases
        .delete_tag(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag_to_note(
    state: State<'_, AppState>,
    note_id: String,
    tag_id: String,
) -> Result<(), String> {
    state
        .tag_usecases
        .add_tag_to_note(&note_id, &tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag_from_note(
    state: State<'_, AppState>,
    note_id: String,
    tag_id: String,
) -> Result<(), String> {
    state
        .tag_usecases
        .remove_tag_from_note(&note_id, &tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tags_for_note(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<Tag>, String> {
    state
        .tag_usecases
        .get_tags_for_note(&note_id)
        .await
        .map_err(|e| e.to_string())
}
