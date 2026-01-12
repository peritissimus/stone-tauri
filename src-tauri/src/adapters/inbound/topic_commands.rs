//! Topic Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::{Note, Topic},
        ports::{
            inbound::{
                ClassifyAllResponse, ClassifyNoteResponse, CreateTopicRequest,
                EmbeddingStatusResponse, NoteTopicInfo, SimilarNoteResult, TopicClassification,
                UpdateTopicRequest,
            },
            outbound::TopicWithCount,
        },
    },
};

#[tauri::command]
pub async fn create_topic(
    state: State<'_, AppState>,
    request: CreateTopicRequest,
) -> Result<Topic, String> {
    state
        .topic_usecases
        .create_topic(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_topic(
    state: State<'_, AppState>,
    request: UpdateTopicRequest,
) -> Result<Topic, String> {
    state
        .topic_usecases
        .update_topic(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_topic(state: State<'_, AppState>, id: String) -> Result<Option<Topic>, String> {
    state
        .topic_usecases
        .get_topic_by_id(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_topics(
    state: State<'_, AppState>,
    exclude_journal: Option<bool>,
) -> Result<Vec<TopicWithCount>, String> {
    state
        .topic_usecases
        .get_all_topics(exclude_journal)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_topic(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .topic_usecases
        .delete_topic(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn classify_note(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<ClassifyNoteResponse, String> {
    state
        .topic_usecases
        .classify_note(&note_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn classify_all_notes(
    state: State<'_, AppState>,
) -> Result<ClassifyAllResponse, String> {
    state
        .topic_usecases
        .classify_all_notes()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_similar_notes(
    state: State<'_, AppState>,
    note_id: String,
    limit: Option<i32>,
) -> Result<Vec<SimilarNoteResult>, String> {
    state
        .topic_usecases
        .get_similar_notes(&note_id, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notes_for_topic(
    state: State<'_, AppState>,
    topic_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Note>, String> {
    state
        .topic_usecases
        .get_notes_for_topic(&topic_id, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_embedding_status(
    state: State<'_, AppState>,
) -> Result<EmbeddingStatusResponse, String> {
    state
        .topic_usecases
        .get_embedding_status()
        .await
        .map_err(|e| e.to_string())
}

// Note: generate_embeddings command removed - no corresponding use case method
// Embeddings are generated automatically when notes are created/updated
// or can be triggered via classify_all_notes
