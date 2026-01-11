use crate::domain::{entities::Version, errors::DomainResult};
use async_trait::async_trait;

/// Version Use Cases Port (Inbound)
///
/// Defines the contract for version history operations.
#[async_trait]
pub trait VersionUseCases: Send + Sync {
    /// Get version history for a note
    async fn get_versions(&self, note_id: &str) -> DomainResult<Vec<Version>>;

    /// Create a new version snapshot
    async fn create_version(&self, note_id: &str) -> DomainResult<Version>;

    /// Restore a note to a specific version
    async fn restore_version(&self, note_id: &str, version_id: &str) -> DomainResult<()>;

    /// Get a specific version
    async fn get_version(&self, version_id: &str) -> DomainResult<Option<Version>>;
}
