use crate::domain::{entities::{Note, Topic}, errors::DomainResult, ports::outbound::TopicWithCount};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTopicRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyNoteResponse {
    pub note_id: String,
    pub topics: Vec<TopicClassification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicClassification {
    pub topic_id: String,
    pub topic_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyAllResponse {
    pub processed: i32,
    pub total: i32,
    pub failed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarNoteResult {
    pub note_id: String,
    pub title: String,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatusResponse {
    pub ready: bool,
    pub total_notes: i32,
    pub embedded_notes: i32,
    pub pending_notes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTopicInfo {
    pub note_id: String,
    pub topic_id: String,
    pub confidence: f32,
    pub is_manual: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub topic_name: String,
    pub topic_color: String,
}

/// Topic Use Cases Port (Inbound)
///
/// Defines the contract for ML topic operations.
#[async_trait]
pub trait TopicUseCases: Send + Sync {
    /// Initialize topic classification system
    async fn initialize(&self) -> DomainResult<()>;

    /// Get all topics with note counts
    async fn get_all_topics(&self, exclude_journal: Option<bool>) -> DomainResult<Vec<TopicWithCount>>;

    /// Get topic by ID
    async fn get_topic_by_id(&self, id: &str) -> DomainResult<Option<Topic>>;

    /// Create a new topic
    async fn create_topic(&self, request: CreateTopicRequest) -> DomainResult<Topic>;

    /// Update a topic
    async fn update_topic(&self, request: UpdateTopicRequest) -> DomainResult<Topic>;

    /// Delete a topic
    async fn delete_topic(&self, id: &str) -> DomainResult<()>;

    /// Classify a note into topics
    async fn classify_note(&self, note_id: &str) -> DomainResult<ClassifyNoteResponse>;

    /// Classify all notes
    async fn classify_all_notes(&self) -> DomainResult<ClassifyAllResponse>;

    /// Assign topic to note (manual)
    async fn assign_topic_to_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()>;

    /// Remove topic from note
    async fn remove_topic_from_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()>;

    /// Find similar notes
    async fn get_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNoteResult>>;

    /// Semantic search
    async fn semantic_search(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNoteResult>>;

    /// Recompute topic centroids
    async fn recompute_centroids(&self) -> DomainResult<()>;

    /// Get embedding status
    async fn get_embedding_status(&self) -> DomainResult<EmbeddingStatusResponse>;

    /// Get notes for a topic
    async fn get_notes_for_topic(
        &self,
        topic_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> DomainResult<Vec<Note>>;

    /// Get topics for a note
    async fn get_topics_for_note(&self, note_id: &str) -> DomainResult<Vec<NoteTopicInfo>>;
}
