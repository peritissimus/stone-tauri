use crate::domain::{
    entities::{LinkCount, Note, NoteLink},
    errors::DomainResult,
};
use async_trait::async_trait;

/// NoteLink Repository Port (Outbound)
///
/// Defines the contract for note link persistence operations.
#[async_trait]
pub trait NoteLinkRepository: Send + Sync {
    /// Get all links
    async fn find_all(&self) -> DomainResult<Vec<NoteLink>>;

    /// Get backlinks for a note (notes that link TO this note)
    async fn get_backlinks(&self, note_id: &str) -> DomainResult<Vec<Note>>;

    /// Get forward links from a note (notes this note links TO)
    async fn get_forward_links(&self, note_id: &str) -> DomainResult<Vec<Note>>;

    /// Add a link between two notes
    async fn save(&self, link: &NoteLink) -> DomainResult<()>;

    /// Remove a link between two notes
    async fn delete(&self, source_id: &str, target_id: &str) -> DomainResult<()>;

    /// Remove all links from a note
    async fn delete_from_note(&self, note_id: &str) -> DomainResult<()>;

    /// Remove all links to a note
    async fn delete_to_note(&self, note_id: &str) -> DomainResult<()>;

    /// Remove all links involving a note (both directions)
    async fn delete_all_for_note(&self, note_id: &str) -> DomainResult<()>;

    /// Check if a link exists
    async fn exists(&self, source_id: &str, target_id: &str) -> DomainResult<bool>;

    /// Count links for a note
    async fn count_for_note(&self, note_id: &str) -> DomainResult<LinkCount>;

    /// Set links for a note (replace all outgoing links)
    async fn set_links_from_note(
        &self,
        source_id: &str,
        target_ids: Vec<String>,
    ) -> DomainResult<()>;
}
