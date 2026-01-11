/// Tag Use Cases Implementation
///
/// Application layer implementations for tag operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::Tag,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{CreateTagRequest, ListTagsRequest, TagList, TagUseCases, UpdateTagRequest},
        outbound::{EventPublisher, TagRepository},
    },
};

/// Implementation of all Tag use cases
pub struct TagUseCasesImpl {
    tag_repository: Arc<dyn TagRepository>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl TagUseCasesImpl {
    pub fn new(
        tag_repository: Arc<dyn TagRepository>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            tag_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl TagUseCases for TagUseCasesImpl {
    /// Create a new tag (or return existing if name already exists)
    async fn create_tag(&self, request: CreateTagRequest) -> DomainResult<Tag> {
        // Check if tag already exists - ✅ ASYNC
        let existing_tags = self.tag_repository.find_all().await?;

        // Normalize name to check - ❌ SYNC (static method)
        let normalized_name = Tag::normalize_name(&request.name)?;

        // Return existing tag if found
        if let Some(existing) = existing_tags.iter().find(|t| t.name == normalized_name) {
            return Ok(existing.clone());
        }

        // Create new tag
        let mut tag = Tag::new(request.name)?;

        // Set color if provided
        if let Some(color) = request.color {
            tag.change_color(color)?;  // Returns Result
        }

        // Save tag - ✅ ASYNC
        self.tag_repository.save(&tag).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("tag:created", serde_json::json!({"id": tag.id}));
        }

        Ok(tag)
    }

    /// Update an existing tag
    async fn update_tag(&self, request: UpdateTagRequest) -> DomainResult<Tag> {
        let mut tag = self
            .tag_repository
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Tag not found with id: {}", request.id))
            })?;

        // Update name if provided
        if let Some(name) = request.name {
            tag.rename(name)?;  // Returns Result
        }

        // Update color if provided
        if let Some(color) = request.color {
            tag.change_color(color)?;  // Returns Result
        }

        // Save tag - ✅ ASYNC
        self.tag_repository.save(&tag).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("tag:updated", serde_json::json!({"id": tag.id}));
        }

        Ok(tag)
    }

    /// Get a tag by ID
    async fn get_tag(&self, id: &str) -> DomainResult<Tag> {
        self.tag_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Tag not found with id: {}", id))
            })
    }

    /// List tags with optional note counts
    async fn list_tags(&self, request: Option<ListTagsRequest>) -> DomainResult<TagList> {
        let include_note_count = request
            .as_ref()
            .and_then(|r| r.include_note_count)
            .unwrap_or(false);

        if include_note_count {
            // ⚠️ Returns TagWithCount, not Tag! - ✅ ASYNC
            let tags_with_counts = self.tag_repository.find_all_with_counts().await?;
            return Ok(TagList::WithCount(tags_with_counts));
        }

        // Without counts - ✅ ASYNC
        let tags = self.tag_repository.find_all().await?;
        Ok(TagList::WithoutCount(tags))
    }

    /// Delete a tag
    async fn delete_tag(&self, id: &str) -> DomainResult<()> {
        // Check if tag exists - ✅ ASYNC
        if !self.tag_repository.exists(id).await? {
            return Err(DomainError::ValidationError(format!(
                "Tag not found with id: {}",
                id
            )));
        }

        // Delete tag - ✅ ASYNC
        self.tag_repository.delete(id).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("tag:deleted", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Add a tag to a note
    async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()> {
        // ✅ ASYNC
        self.tag_repository.add_tag_to_note(note_id, tag_id).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "note:tagged",
                serde_json::json!({"note_id": note_id, "tag_id": tag_id}),
            );
        }

        Ok(())
    }

    /// Remove a tag from a note
    async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()> {
        // ✅ ASYNC
        self.tag_repository
            .remove_tag_from_note(note_id, tag_id)
            .await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "note:untagged",
                serde_json::json!({"note_id": note_id, "tag_id": tag_id}),
            );
        }

        Ok(())
    }

    /// Get all tags for a note
    async fn get_note_tags(&self, note_id: &str) -> DomainResult<Vec<Tag>> {
        // ⚠️ Use get_note_tags() not find_by_note_id() - ✅ ASYNC
        self.tag_repository.get_note_tags(note_id).await
    }
}
