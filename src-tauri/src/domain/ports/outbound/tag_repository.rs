use crate::domain::{entities::Tag, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagWithCount {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub note_count: i32,
}

/// Tag Repository Port (Outbound)
///
/// Defines the contract for tag persistence operations.
#[async_trait]
pub trait TagRepository: Send + Sync {
    /// Find a tag by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Tag>>;

    /// Find tag by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<Tag>>;

    /// Find all tags
    async fn find_all(&self) -> DomainResult<Vec<Tag>>;

    /// Find all tags with note counts
    async fn find_all_with_counts(&self) -> DomainResult<Vec<TagWithCount>>;

    /// Find tags by note ID
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Tag>>;

    /// Save a tag (create or update)
    async fn save(&self, tag: &Tag) -> DomainResult<()>;

    /// Delete a tag
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Check if a tag exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;

    // Note-Tag associations

    /// Add a tag to a note
    async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()>;

    /// Remove a tag from a note
    async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()>;

    /// Get all tags for a note
    async fn get_note_tags(&self, note_id: &str) -> DomainResult<Vec<Tag>>;

    /// Set tags for a note (replaces all existing tags)
    async fn set_note_tags(&self, note_id: &str, tag_ids: Vec<String>) -> DomainResult<()>;

    // Bulk operations

    /// Get tags for multiple notes (returns a map of note_id -> tags)
    async fn get_tags_for_notes(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<Tag>>>;
}
