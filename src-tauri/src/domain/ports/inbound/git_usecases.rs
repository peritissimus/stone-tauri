use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResponse {
    pub is_repo: bool,
    pub has_changes: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub ahead: i32,
    pub behind: i32,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSyncResponse {
    pub success: bool,
    pub pulled: i32,
    pub pushed: i32,
    pub conflicts: Vec<String>,
    pub error: Option<String>,
}

/// Git Use Cases Port (Inbound)
///
/// Defines the contract for git operations.
#[async_trait]
pub trait GitUseCases: Send + Sync {
    /// Get git status for a workspace
    async fn get_status(&self, workspace_id: &str) -> DomainResult<GitStatusResponse>;

    /// Initialize git repository
    async fn init(&self, workspace_id: &str) -> DomainResult<bool>;

    /// Commit changes
    async fn commit(&self, workspace_id: &str, message: Option<&str>)
        -> DomainResult<Option<GitCommitInfo>>;

    /// Pull from remote
    async fn pull(&self, workspace_id: &str) -> DomainResult<bool>;

    /// Push to remote
    async fn push(&self, workspace_id: &str) -> DomainResult<bool>;

    /// Sync (pull + commit + push)
    async fn sync(
        &self,
        workspace_id: &str,
        message: Option<&str>,
    ) -> DomainResult<GitSyncResponse>;

    /// Set remote URL
    async fn set_remote(&self, workspace_id: &str, url: &str) -> DomainResult<bool>;

    /// Get recent commits
    async fn get_commits(
        &self,
        workspace_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<GitCommitInfo>>;
}
