use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitFileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileChange {
    pub path: String,
    pub status: GitFileStatus,
    pub staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub ahead: i32,
    pub behind: i32,
    pub has_remote: bool,
    pub remote_url: Option<String>,
    pub changes: Vec<GitFileChange>,
    pub has_uncommitted_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationResult {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Git Service Port (Outbound)
///
/// Defines the contract for git operations.
#[async_trait]
pub trait GitService: Send + Sync {
    /// Check if a directory is a git repository
    async fn is_repository(&self, path: &str) -> DomainResult<bool>;

    /// Initialize a new git repository
    async fn init(&self, path: &str) -> DomainResult<GitOperationResult>;

    /// Get repository status
    async fn get_status(&self, path: &str) -> DomainResult<GitStatus>;

    /// Stage files for commit
    async fn stage(&self, path: &str, files: Option<Vec<String>>) -> DomainResult<GitOperationResult>;

    /// Create a commit
    async fn commit(&self, path: &str, message: &str) -> DomainResult<GitOperationResult>;

    /// Pull from remote
    async fn pull(&self, path: &str) -> DomainResult<GitOperationResult>;

    /// Push to remote
    async fn push(&self, path: &str) -> DomainResult<GitOperationResult>;

    /// Set remote URL
    async fn set_remote(
        &self,
        path: &str,
        url: &str,
        name: Option<&str>,
    ) -> DomainResult<GitOperationResult>;

    /// Get recent commits
    async fn get_commits(&self, path: &str, limit: Option<i32>) -> DomainResult<Vec<GitCommit>>;

    /// Sync (pull + commit + push)
    async fn sync(&self, path: &str, message: Option<&str>) -> DomainResult<GitOperationResult>;
}
