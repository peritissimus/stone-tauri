use crate::domain::{entities::Workspace, errors::DomainResult};
use async_trait::async_trait;

/// Workspace Repository Port (Outbound)
///
/// Defines the contract for workspace persistence operations.
#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    /// Find a workspace by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Workspace>>;

    /// Find workspace by folder path
    async fn find_by_folder_path(&self, folder_path: &str) -> DomainResult<Option<Workspace>>;

    /// Find all workspaces
    async fn find_all(&self) -> DomainResult<Vec<Workspace>>;

    /// Find the active workspace
    async fn find_active(&self) -> DomainResult<Option<Workspace>>;

    /// Save a workspace (create or update)
    async fn save(&self, workspace: &Workspace) -> DomainResult<()>;

    /// Delete a workspace
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Set a workspace as active (and deactivate others)
    async fn set_active(&self, id: &str) -> DomainResult<()>;

    /// Check if a workspace exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;
}
