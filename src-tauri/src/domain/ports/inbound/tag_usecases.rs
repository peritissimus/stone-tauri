use crate::domain::{entities::Tag, errors::DomainResult, ports::outbound::TagWithCount};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagRequest {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTagsRequest {
    pub include_note_count: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagList {
    WithCount(Vec<TagWithCount>),
    WithoutCount(Vec<Tag>),
}

/// Tag Use Cases Port (Inbound)
///
/// Defines the contract for tag-related use cases.
#[async_trait]
pub trait TagUseCases: Send + Sync {
    /// Create a new tag
    async fn create_tag(&self, request: CreateTagRequest) -> DomainResult<Tag>;

    /// Update an existing tag
    async fn update_tag(&self, request: UpdateTagRequest) -> DomainResult<Tag>;

    /// Get a tag by ID
    async fn get_tag(&self, id: &str) -> DomainResult<Tag>;

    /// List all tags
    async fn list_tags(&self, request: Option<ListTagsRequest>) -> DomainResult<TagList>;

    /// Delete a tag
    async fn delete_tag(&self, id: &str) -> DomainResult<()>;

    /// Add a tag to a note
    async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()>;

    /// Remove a tag from a note
    async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()>;

    /// Get all tags for a note
    async fn get_note_tags(&self, note_id: &str) -> DomainResult<Vec<Tag>>;
}
