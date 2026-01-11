use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileWatchEvent {
    Add,
    Change,
    Unlink,
}

/// File Storage Port (Outbound)
///
/// Defines the contract for file system operations.
/// Implementations can be local filesystem, S3, etc.
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Read file content as string
    /// Returns None if file doesn't exist or is empty
    async fn read(&self, file_path: &str) -> DomainResult<Option<String>>;

    /// Write content to a file
    async fn write(&self, file_path: &str, content: &str) -> DomainResult<()>;

    /// Delete a file
    async fn delete(&self, file_path: &str) -> DomainResult<()>;

    /// Check if a file exists
    async fn exists(&self, file_path: &str) -> DomainResult<bool>;

    /// Rename/move a file
    async fn rename(&self, old_path: &str, new_path: &str) -> DomainResult<()>;

    /// Create a directory
    async fn create_directory(&self, dir_path: &str) -> DomainResult<()>;

    /// Delete a directory (recursively)
    async fn delete_directory(&self, dir_path: &str) -> DomainResult<()>;

    /// List files in a directory
    async fn list_files(&self, dir_path: &str) -> DomainResult<Vec<FileInfo>>;

    /// List files matching a pattern (glob)
    async fn glob(&self, pattern: &str, base_path: &str) -> DomainResult<Vec<String>>;

    /// Get file info
    async fn get_file_info(&self, file_path: &str) -> DomainResult<Option<FileInfo>>;

    /// Copy a file
    async fn copy(&self, source_path: &str, dest_path: &str) -> DomainResult<()>;
}
