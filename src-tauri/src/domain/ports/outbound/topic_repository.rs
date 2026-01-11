use crate::domain::{entities::Topic, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTopicAssignment {
    pub note_id: String,
    pub topic_id: String,
    pub confidence: f32,
    pub is_manual: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTopicWithDetails {
    pub note_id: String,
    pub topic_id: String,
    pub confidence: f32,
    pub is_manual: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub topic_name: String,
    pub topic_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicWithCount {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub centroid: Option<Vec<u8>>,
    pub is_predefined: bool,
    pub note_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicAssignmentOptions {
    pub confidence: Option<f32>,
    pub is_manual: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNotesForTopicOptions {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub exclude_journal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTopicInfo {
    pub note_id: String,
    pub confidence: f32,
    pub is_manual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindAllWithCountsOptions {
    pub exclude_journal: Option<bool>,
}

/// Topic Repository Port (Outbound)
///
/// Defines the contract for topic persistence operations.
#[async_trait]
pub trait TopicRepository: Send + Sync {
    /// Find topic by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Topic>>;

    /// Find topic by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<Topic>>;

    /// Get all topics
    async fn find_all(&self) -> DomainResult<Vec<Topic>>;

    /// Get all topics with note counts
    async fn find_all_with_counts(
        &self,
        options: Option<FindAllWithCountsOptions>,
    ) -> DomainResult<Vec<TopicWithCount>>;

    /// Get predefined topics
    async fn find_predefined(&self) -> DomainResult<Vec<Topic>>;

    /// Save a topic
    async fn save(&self, topic: &Topic) -> DomainResult<()>;

    /// Delete a topic
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Check if topic exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;

    /// Get topics for a note
    async fn get_topics_for_note(&self, note_id: &str) -> DomainResult<Vec<NoteTopicWithDetails>>;

    /// Get topics for multiple notes (bulk)
    async fn get_topics_for_notes(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<NoteTopicWithDetails>>>;

    /// Get notes for a topic
    async fn get_notes_for_topic(
        &self,
        topic_id: &str,
        options: Option<GetNotesForTopicOptions>,
    ) -> DomainResult<Vec<NoteTopicInfo>>;

    /// Assign topic to note
    async fn assign_to_note(
        &self,
        note_id: &str,
        topic_id: &str,
        options: Option<TopicAssignmentOptions>,
    ) -> DomainResult<()>;

    /// Remove topic from note
    async fn remove_from_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()>;

    /// Set all topics for a note (replaces existing)
    async fn set_topics_for_note(
        &self,
        note_id: &str,
        assignments: Vec<(String, Option<f32>, Option<bool>)>,
    ) -> DomainResult<()>;

    /// Clear all topics for a note
    async fn clear_topics_for_note(&self, note_id: &str) -> DomainResult<()>;

    /// Update topic centroid (ML)
    async fn update_centroid(&self, topic_id: &str, centroid: Vec<u8>) -> DomainResult<()>;

    /// Update note count for a topic
    async fn update_note_count(&self, topic_id: &str) -> DomainResult<()>;
}
