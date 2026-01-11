use crate::domain::{entities::Attachment, errors::DomainResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// Attachment Repository Port (Outbound)
///
/// Defines the contract for attachment persistence operations.
#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    /// Find attachment by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Attachment>>;

    /// Get all attachments for a note
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Attachment>>;

    /// Get attachments for multiple notes (bulk operation)
    async fn find_by_note_ids(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<Attachment>>>;

    /// Save an attachment
    async fn save(&self, attachment: &Attachment) -> DomainResult<()>;

    /// Delete an attachment
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Delete all attachments for a note
    async fn delete_by_note_id(&self, note_id: &str) -> DomainResult<()>;

    /// Check if attachment exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;

    /// Count attachments for a note
    async fn count_by_note_id(&self, note_id: &str) -> DomainResult<i32>;
}
