use crate::domain::{entities::Version, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListItem {
    pub id: String,
    pub note_id: String,
    pub version_number: i32,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Version Repository Port (Outbound)
///
/// Defines the contract for note version persistence operations.
#[async_trait]
pub trait VersionRepository: Send + Sync {
    /// Find version by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Version>>;

    /// Create a new version snapshot
    async fn save(&self, version: &Version) -> DomainResult<()>;

    /// Get next version number for a note
    async fn get_next_version_number(&self, note_id: &str) -> DomainResult<i32>;

    /// Get all versions for a note
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Version>>;

    /// Get version history summary (without full content)
    async fn get_version_summary(&self, note_id: &str) -> DomainResult<Vec<VersionListItem>>;

    /// Get latest version for a note
    async fn get_latest_version(&self, note_id: &str) -> DomainResult<Option<Version>>;

    /// Delete all versions for a note
    async fn delete_by_note_id(&self, note_id: &str) -> DomainResult<()>;

    /// Delete old versions (keep N most recent)
    async fn prune_versions(&self, note_id: &str, keep_count: i32) -> DomainResult<i32>;

    /// Count versions for a note
    async fn count_by_note_id(&self, note_id: &str) -> DomainResult<i32>;
}
