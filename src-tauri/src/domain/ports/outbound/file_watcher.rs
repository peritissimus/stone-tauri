use crate::domain::{entities::Workspace, errors::DomainResult};
use async_trait::async_trait;

/// File Watcher Port (Outbound)
///
/// Defines the contract for watching file system changes in workspaces.
/// Implementations handle the actual file watching using libraries like notify.
#[async_trait]
pub trait FileWatcher: Send + Sync {
    /// Start watching all workspaces for file changes
    async fn start(&self) -> DomainResult<()>;

    /// Stop all file watchers
    async fn stop_all(&self) -> DomainResult<()>;

    /// Start watching a specific workspace
    async fn watch_workspace(&self, workspace: &Workspace) -> DomainResult<()>;

    /// Stop watching a specific workspace
    async fn unwatch_workspace(&self, workspace_id: &str) -> DomainResult<()>;
}
