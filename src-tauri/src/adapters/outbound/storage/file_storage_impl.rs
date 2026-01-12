//! File Storage Implementation
//!
//! Tokio-based implementation of the FileStorage port for local filesystem operations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{FileInfo, FileStorage},
};

/// Tokio implementation of FileStorage
pub struct TokioFileStorage;

impl TokioFileStorage {
    pub fn new() -> Self {
        Self
    }

    /// Convert std::io::Error to DomainError
    fn map_io_error(err: std::io::Error) -> DomainError {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                DomainError::NotFound(format!("File not found: {}", err))
            }
            std::io::ErrorKind::PermissionDenied => {
                DomainError::FileStorageError(format!("Permission denied: {}", err))
            }
            std::io::ErrorKind::AlreadyExists => {
                DomainError::FileStorageError(format!("File already exists: {}", err))
            }
            _ => DomainError::FileStorageError(format!("File system error: {}", err)),
        }
    }

    /// Convert system time to DateTime<Utc>
    fn system_time_to_datetime(time: std::time::SystemTime) -> DateTime<Utc> {
        time.into()
    }
}

impl Default for TokioFileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileStorage for TokioFileStorage {
    /// Read file content as string
    async fn read(&self, file_path: &str) -> DomainResult<Option<String>> {
        match fs::read_to_string(file_path).await {
            Ok(content) => {
                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(content))
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Self::map_io_error(err)),
        }
    }

    /// Write content to a file
    async fn write(&self, file_path: &str, content: &str) -> DomainResult<()> {
        let path = Path::new(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(Self::map_io_error)?;
        }

        fs::write(file_path, content)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }

    /// Delete a file
    async fn delete(&self, file_path: &str) -> DomainResult<()> {
        fs::remove_file(file_path)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }

    /// Check if a file exists
    async fn exists(&self, file_path: &str) -> DomainResult<bool> {
        Ok(fs::try_exists(file_path)
            .await
            .map_err(Self::map_io_error)?)
    }

    /// Rename/move a file
    async fn rename(&self, old_path: &str, new_path: &str) -> DomainResult<()> {
        let new_path_buf = Path::new(new_path);

        // Create parent directories for new path if they don't exist
        if let Some(parent) = new_path_buf.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(Self::map_io_error)?;
        }

        fs::rename(old_path, new_path)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }

    /// Create a directory
    async fn create_directory(&self, dir_path: &str) -> DomainResult<()> {
        fs::create_dir_all(dir_path)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }

    /// Delete a directory (recursively)
    async fn delete_directory(&self, dir_path: &str) -> DomainResult<()> {
        fs::remove_dir_all(dir_path)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }

    /// List files in a directory
    async fn list_files(&self, dir_path: &str) -> DomainResult<Vec<FileInfo>> {
        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(Self::map_io_error)?;

        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(Self::map_io_error)? {
            let path = entry.path();
            let metadata = entry.metadata().await.map_err(Self::map_io_error)?;

            let created_at = metadata
                .created()
                .map(Self::system_time_to_datetime)
                .unwrap_or_else(|_| Utc::now());

            let modified_at = metadata
                .modified()
                .map(Self::system_time_to_datetime)
                .unwrap_or_else(|_| Utc::now());

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                name: entry
                    .file_name()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                created_at,
                modified_at,
            });
        }

        Ok(files)
    }

    /// List files matching a pattern (glob)
    async fn glob(&self, pattern: &str, base_path: &str) -> DomainResult<Vec<String>> {
        let base = PathBuf::from(base_path);
        let full_pattern = base.join(pattern);

        let pattern_str = full_pattern
            .to_str()
            .ok_or_else(|| DomainError::ValidationError("Invalid path".to_string()))?;

        // Use glob crate for pattern matching
        let paths = glob::glob(pattern_str)
            .map_err(|e| DomainError::ValidationError(format!("Invalid glob pattern: {}", e)))?;

        let mut results = Vec::new();

        for entry in paths {
            match entry {
                Ok(path) => {
                    // Return paths relative to base_path
                    if let Ok(relative) = path.strip_prefix(&base) {
                        results.push(relative.to_string_lossy().to_string());
                    }
                }
                Err(e) => {
                    // Log error but continue processing
                    eprintln!("Glob error: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Get file info
    async fn get_file_info(&self, file_path: &str) -> DomainResult<Option<FileInfo>> {
        match fs::metadata(file_path).await {
            Ok(metadata) => {
                let path = Path::new(file_path);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let created_at = metadata
                    .created()
                    .map(Self::system_time_to_datetime)
                    .unwrap_or_else(|_| Utc::now());

                let modified_at = metadata
                    .modified()
                    .map(Self::system_time_to_datetime)
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(FileInfo {
                    path: file_path.to_string(),
                    name,
                    size: metadata.len(),
                    is_directory: metadata.is_dir(),
                    created_at,
                    modified_at,
                }))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Self::map_io_error(err)),
        }
    }

    /// Copy a file
    async fn copy(&self, source_path: &str, dest_path: &str) -> DomainResult<()> {
        let dest_path_buf = Path::new(dest_path);

        // Create parent directories for destination if they don't exist
        if let Some(parent) = dest_path_buf.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(Self::map_io_error)?;
        }

        fs::copy(source_path, dest_path)
            .await
            .map_err(Self::map_io_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        let storage = TokioFileStorage::new();

        let content = "Hello, World!";
        storage
            .write(file_path.to_str().unwrap(), content)
            .await
            .unwrap();

        let read_content = storage.read(file_path.to_str().unwrap()).await.unwrap();
        assert_eq!(read_content, Some(content.to_string()));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("nonexistent.txt");
        let storage = TokioFileStorage::new();

        let result = storage.read(file_path.to_str().unwrap()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_write_creates_parent_directories() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("nested/dir/test.txt");
        let storage = TokioFileStorage::new();

        storage
            .write(file_path.to_str().unwrap(), "content")
            .await
            .unwrap();

        let exists = storage.exists(file_path.to_str().unwrap()).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        let storage = TokioFileStorage::new();

        storage
            .write(file_path.to_str().unwrap(), "content")
            .await
            .unwrap();
        storage.delete(file_path.to_str().unwrap()).await.unwrap();

        let exists = storage.exists(file_path.to_str().unwrap()).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        let storage = TokioFileStorage::new();

        let exists_before = storage.exists(file_path.to_str().unwrap()).await.unwrap();
        assert!(!exists_before);

        storage
            .write(file_path.to_str().unwrap(), "content")
            .await
            .unwrap();

        let exists_after = storage.exists(file_path.to_str().unwrap()).await.unwrap();
        assert!(exists_after);
    }

    #[tokio::test]
    async fn test_rename() {
        let temp_dir = create_test_dir();
        let old_path = temp_dir.path().join("old.txt");
        let new_path = temp_dir.path().join("new.txt");
        let storage = TokioFileStorage::new();

        storage
            .write(old_path.to_str().unwrap(), "content")
            .await
            .unwrap();

        storage
            .rename(old_path.to_str().unwrap(), new_path.to_str().unwrap())
            .await
            .unwrap();

        let old_exists = storage.exists(old_path.to_str().unwrap()).await.unwrap();
        let new_exists = storage.exists(new_path.to_str().unwrap()).await.unwrap();

        assert!(!old_exists);
        assert!(new_exists);
    }

    #[tokio::test]
    async fn test_create_directory() {
        let temp_dir = create_test_dir();
        let dir_path = temp_dir.path().join("new_dir");
        let storage = TokioFileStorage::new();

        storage
            .create_directory(dir_path.to_str().unwrap())
            .await
            .unwrap();

        let exists = storage.exists(dir_path.to_str().unwrap()).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_delete_directory() {
        let temp_dir = create_test_dir();
        let dir_path = temp_dir.path().join("dir_to_delete");
        let storage = TokioFileStorage::new();

        storage
            .create_directory(dir_path.to_str().unwrap())
            .await
            .unwrap();

        storage
            .delete_directory(dir_path.to_str().unwrap())
            .await
            .unwrap();

        let exists = storage.exists(dir_path.to_str().unwrap()).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = create_test_dir();
        let storage = TokioFileStorage::new();

        // Create some test files
        storage
            .write(&format!("{}/file1.txt", temp_dir.path().display()), "content1")
            .await
            .unwrap();
        storage
            .write(&format!("{}/file2.txt", temp_dir.path().display()), "content2")
            .await
            .unwrap();

        let files = storage
            .list_files(temp_dir.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.name == "file1.txt"));
        assert!(files.iter().any(|f| f.name == "file2.txt"));
    }

    #[tokio::test]
    async fn test_get_file_info() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        let storage = TokioFileStorage::new();

        storage
            .write(file_path.to_str().unwrap(), "content")
            .await
            .unwrap();

        let info = storage
            .get_file_info(file_path.to_str().unwrap())
            .await
            .unwrap();

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "test.txt");
        assert!(!info.is_directory);
        assert!(info.size > 0);
    }

    #[tokio::test]
    async fn test_get_file_info_nonexistent() {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("nonexistent.txt");
        let storage = TokioFileStorage::new();

        let info = storage
            .get_file_info(file_path.to_str().unwrap())
            .await
            .unwrap();

        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_copy() {
        let temp_dir = create_test_dir();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        let storage = TokioFileStorage::new();

        let content = "test content";
        storage
            .write(source_path.to_str().unwrap(), content)
            .await
            .unwrap();

        storage
            .copy(source_path.to_str().unwrap(), dest_path.to_str().unwrap())
            .await
            .unwrap();

        let dest_content = storage.read(dest_path.to_str().unwrap()).await.unwrap();
        assert_eq!(dest_content, Some(content.to_string()));

        // Source should still exist
        let source_exists = storage.exists(source_path.to_str().unwrap()).await.unwrap();
        assert!(source_exists);
    }

    #[tokio::test]
    async fn test_glob() {
        let temp_dir = create_test_dir();
        let storage = TokioFileStorage::new();

        // Create test files
        storage
            .write(&format!("{}/file1.txt", temp_dir.path().display()), "content")
            .await
            .unwrap();
        storage
            .write(&format!("{}/file2.md", temp_dir.path().display()), "content")
            .await
            .unwrap();
        storage
            .write(&format!("{}/file3.txt", temp_dir.path().display()), "content")
            .await
            .unwrap();

        let matches = storage
            .glob("*.txt", temp_dir.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"file1.txt".to_string()));
        assert!(matches.contains(&"file3.txt".to_string()));
    }
}
