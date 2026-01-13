//! Topic Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::{Note, Topic},
        ports::{
            inbound::{
                ClassifyAllResponse, ClassifyNoteResponse, CreateTopicRequest,
                EmbeddingStatusResponse, NoteTopicInfo, SimilarNoteResult, UpdateTopicRequest,
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

/// Response for list_topics
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTopicsResponse {
    pub topics: Vec<TopicWithCount>,
}

#[tauri::command]
pub async fn list_topics(
    state: State<'_, AppState>,
    exclude_journal: Option<bool>,
) -> Result<ListTopicsResponse, String> {
    let topics = state
        .topic_usecases
        .get_all_topics(exclude_journal)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ListTopicsResponse { topics })
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

/// Response for get_similar_notes
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSimilarNotesResponse {
    pub similar: Vec<SimilarNoteResult>,
}

#[tauri::command]
pub async fn get_similar_notes(
    state: State<'_, AppState>,
    note_id: String,
    limit: Option<i32>,
) -> Result<GetSimilarNotesResponse, String> {
    let similar = state
        .topic_usecases
        .get_similar_notes(&note_id, limit)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetSimilarNotesResponse { similar })
}

/// Response for get_notes_for_topic
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNotesForTopicResponse {
    pub notes: Vec<Note>,
}

#[tauri::command]
pub async fn get_notes_for_topic(
    state: State<'_, AppState>,
    topic_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<GetNotesForTopicResponse, String> {
    let notes = state
        .topic_usecases
        .get_notes_for_topic(&topic_id, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetNotesForTopicResponse { notes })
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

/// Response for get_topics_for_note
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTopicsForNoteResponse {
    pub topics: Vec<NoteTopicInfo>,
}

/// Get topics for a note
#[tauri::command]
pub async fn get_topics_for_note(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<GetTopicsForNoteResponse, String> {
    let topics = state
        .topic_usecases
        .get_topics_for_note(&note_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(GetTopicsForNoteResponse { topics })
}

/// Assign a topic to a note
#[tauri::command]
pub async fn assign_topic_to_note(
    state: State<'_, AppState>,
    note_id: String,
    topic_id: String,
) -> Result<(), String> {
    state
        .topic_usecases
        .assign_topic_to_note(&note_id, &topic_id)
        .await
        .map_err(|e| e.to_string())
}

/// Remove a topic from a note
#[tauri::command]
pub async fn remove_topic_from_note(
    state: State<'_, AppState>,
    note_id: String,
    topic_id: String,
) -> Result<(), String> {
    state
        .topic_usecases
        .remove_topic_from_note(&note_id, &topic_id)
        .await
        .map_err(|e| e.to_string())
}

// Note: generate_embeddings command removed - no corresponding use case method
// Embeddings are generated automatically when notes are created/updated
// or can be triggered via classify_all_notes
