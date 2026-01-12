//! File Storage Implementation
//!
//! Tokio-based implementation of the FileStorage port using async file operations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::Path;
use tokio::fs;
use std::fs::Metadata;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{FileInfo, FileStorage},
};

/// Tokio-based file storage implementation
pub struct TokioFileStorage;

impl TokioFileStorage {
    pub fn new() -> Self {
        Self
    }

    /// Convert system time to DateTime<Utc>
    fn system_time_to_datetime(
        time: std::time::SystemTime,
    ) -> DomainResult<DateTime<Utc>> {
        time.duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| DomainError::FileStorageError(format!("Time conversion error: {}", e)))
            .map(|duration| {
                DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .unwrap_or_else(|| Utc::now())
            })
    }

    /// Convert metadata to FileInfo
    async fn metadata_to_file_info(
        path: &str,
        metadata: &Metadata,
    ) -> DomainResult<FileInfo> {
        let path_obj = Path::new(path);
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let created_at = Self::system_time_to_datetime(
            metadata
                .created()
                .unwrap_or_else(|_| std::time::SystemTime::now()),
        )?;

        let modified_at = Self::system_time_to_datetime(
            metadata
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::now()),
        )?;

        Ok(FileInfo {
            path: path.to_string(),
            name,
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            created_at,
            modified_at,
        })
    }
}

impl Default for TokioFileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileStorage for TokioFileStorage {
    async fn read(&self, file_path: &str) -> DomainResult<Option<String>> {
        match fs::read_to_string(file_path).await {
            Ok(content) => {
                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(content))
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DomainError::FileStorageError(format!(
                "Failed to read file '{}': {}",
                file_path, e
            ))),
        }
    }

    async fn write(&self, file_path: &str, content: &str) -> DomainResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    DomainError::FileStorageError(format!(
                        "Failed to create parent directory: {}",
                        e
                    ))
                })?;
            }
        }

        fs::write(file_path, content).await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to write file '{}': {}", file_path, e))
        })
    }

    async fn delete(&self, file_path: &str) -> DomainResult<()> {
        let metadata = fs::metadata(file_path).await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to get file metadata: {}", e))
        })?;

        if metadata.is_dir() {
            fs::remove_dir_all(file_path).await
        } else {
            fs::remove_file(file_path).await
        }
        .map_err(|e| {
            DomainError::FileStorageError(format!("Failed to delete '{}': {}", file_path, e))
        })
    }

    async fn exists(&self, file_path: &str) -> DomainResult<bool> {
        Ok(Path::new(file_path).exists())
    }

    async fn rename(&self, old_path: &str, new_path: &str) -> DomainResult<()> {
        // Ensure parent directory exists for new path
        if let Some(parent) = Path::new(new_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    DomainError::FileStorageError(format!(
                        "Failed to create parent directory: {}",
                        e
                    ))
                })?;
            }
        }

        fs::rename(old_path, new_path).await.map_err(|e| {
            DomainError::FileStorageError(format!(
                "Failed to rename '{}' to '{}': {}",
                old_path, new_path, e
            ))
        })
    }

    async fn create_directory(&self, dir_path: &str) -> DomainResult<()> {
        fs::create_dir_all(dir_path).await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to create directory '{}': {}", dir_path, e))
        })
    }

    async fn delete_directory(&self, dir_path: &str) -> DomainResult<()> {
        fs::remove_dir_all(dir_path).await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to delete directory '{}': {}", dir_path, e))
        })
    }

    async fn list_files(&self, dir_path: &str) -> DomainResult<Vec<FileInfo>> {
        let mut entries = fs::read_dir(dir_path).await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to read directory '{}': {}", dir_path, e))
        })?;

        let mut file_infos = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            DomainError::FileStorageError(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();

            let metadata = entry.metadata().await.map_err(|e| {
                DomainError::FileStorageError(format!("Failed to get metadata: {}", e))
            })?;

            let file_info = Self::metadata_to_file_info(&path_str, &metadata).await?;
            file_infos.push(file_info);
        }

        Ok(file_infos)
    }

    async fn glob(&self, pattern: &str, base_path: &str) -> DomainResult<Vec<String>> {
        let base = Path::new(base_path);

        // Use glob crate for pattern matching
        let glob_pattern = base.join(pattern).to_string_lossy().to_string();

        let paths: Vec<String> = glob::glob(&glob_pattern)
            .map_err(|e| DomainError::FileStorageError(format!("Invalid glob pattern: {}", e)))?
            .filter_map(|entry| entry.ok())
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        Ok(paths)
    }

    async fn get_file_info(&self, file_path: &str) -> DomainResult<Option<FileInfo>> {
        match fs::metadata(file_path).await {
            Ok(metadata) => {
                let file_info = Self::metadata_to_file_info(file_path, &metadata).await?;
                Ok(Some(file_info))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DomainError::FileStorageError(format!(
                "Failed to get file info for '{}': {}",
                file_path, e
            ))),
        }
    }

    async fn copy(&self, source_path: &str, dest_path: &str) -> DomainResult<()> {
        // Ensure parent directory exists for destination
        if let Some(parent) = Path::new(dest_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    DomainError::FileStorageError(format!(
                        "Failed to create parent directory: {}",
                        e
                    ))
                })?;
            }
        }

        fs::copy(source_path, dest_path).await.map_err(|e| {
            DomainError::FileStorageError(format!(
                "Failed to copy '{}' to '{}': {}",
                source_path, dest_path, e
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Write file
        storage.write(file_path_str, "Hello, World!").await.unwrap();

        // Read file
        let content = storage.read(file_path_str).await.unwrap();
        assert_eq!(content, Some("Hello, World!".to_string()));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let storage = TokioFileStorage::new();
        let result = storage.read("/nonexistent/file.txt").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // File should not exist initially
        assert!(!storage.exists(file_path_str).await.unwrap());

        // Write file
        storage.write(file_path_str, "test").await.unwrap();

        // File should exist now
        assert!(storage.exists(file_path_str).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Write and verify file
        storage.write(file_path_str, "test").await.unwrap();
        assert!(storage.exists(file_path_str).await.unwrap());

        // Delete file
        storage.delete(file_path_str).await.unwrap();
        assert!(!storage.exists(file_path_str).await.unwrap());
    }

    #[tokio::test]
    async fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.txt");
        let new_path = temp_dir.path().join("new.txt");
        let old_path_str = old_path.to_str().unwrap();
        let new_path_str = new_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Write file
        storage.write(old_path_str, "test").await.unwrap();

        // Rename file
        storage.rename(old_path_str, new_path_str).await.unwrap();

        // Old path should not exist, new path should exist
        assert!(!storage.exists(old_path_str).await.unwrap());
        assert!(storage.exists(new_path_str).await.unwrap());
    }

    #[tokio::test]
    async fn test_create_and_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        let dir_path_str = dir_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Create directory
        storage.create_directory(dir_path_str).await.unwrap();
        assert!(storage.exists(dir_path_str).await.unwrap());

        // Create some files
        let file1 = dir_path.join("file1.txt");
        let file2 = dir_path.join("file2.txt");
        storage.write(file1.to_str().unwrap(), "content1").await.unwrap();
        storage.write(file2.to_str().unwrap(), "content2").await.unwrap();

        // List files
        let files = storage.list_files(dir_path_str).await.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        let source_path_str = source_path.to_str().unwrap();
        let dest_path_str = dest_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Write source file
        storage.write(source_path_str, "original content").await.unwrap();

        // Copy file
        storage.copy(source_path_str, dest_path_str).await.unwrap();

        // Both files should exist with same content
        let source_content = storage.read(source_path_str).await.unwrap();
        let dest_content = storage.read(dest_path_str).await.unwrap();
        assert_eq!(source_content, dest_content);
    }

    #[tokio::test]
    async fn test_get_file_info() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let storage = TokioFileStorage::new();

        // Nonexistent file should return None
        let info = storage.get_file_info(file_path_str).await.unwrap();
        assert!(info.is_none());

        // Write file
        storage.write(file_path_str, "test content").await.unwrap();

        // Get file info
        let info = storage.get_file_info(file_path_str).await.unwrap();
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.name, "test.txt");
        assert_eq!(info.size, "test content".len() as u64);
        assert!(!info.is_directory);
    }
}
