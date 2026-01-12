//! Attachment Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Attachment,
        ports::inbound::{AddAttachmentRequest, UploadImageRequest, UploadImageResponse},
    },
};

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    request: AddAttachmentRequest,
) -> Result<Attachment, String> {
    state
        .attachment_usecases
        .add_attachment(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_attachments_for_note(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<Attachment>, String> {
    state
        .attachment_usecases
        .get_attachments_for_note(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_attachment(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .attachment_usecases
        .delete_attachment(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_image(
    state: State<'_, AppState>,
    request: UploadImageRequest,
) -> Result<UploadImageResponse, String> {
    state
        .attachment_usecases
        .upload_image(request)
        .await
        .map_err(|e| e.to_string())
}
