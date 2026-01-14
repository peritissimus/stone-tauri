//! Application Path Resolution
//!
//! Manages all application paths (database, attachments, temp files, etc.)

use std::path::PathBuf;
use crate::domain::errors::{DomainError, DomainResult};

/// Application paths for data storage
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Base application data directory
    pub app_data_dir: PathBuf,

    /// Database directory
    pub database_dir: PathBuf,

    /// Database file path
    pub database_file: PathBuf,

    /// Attachments storage directory
    pub attachments_dir: PathBuf,

    /// Temporary files directory
    pub temp_dir: PathBuf,

    /// Search index directory
    pub search_index_dir: PathBuf,

    /// Backups directory
    pub backups_dir: PathBuf,
}

impl AppPaths {
    /// Create new AppPaths with platform-specific defaults
    pub fn new() -> DomainResult<Self> {
        // Get app data directory based on platform
        let app_data_dir = Self::get_app_data_dir()?;

        let database_dir = app_data_dir.join("database");
        let database_file = database_dir.join("stone.db");
        let attachments_dir = app_data_dir.join("attachments");
        let temp_dir = app_data_dir.join("temp");
        let search_index_dir = app_data_dir.join("search_index");
        let backups_dir = app_data_dir.join("backups");

        Ok(Self {
            app_data_dir,
            database_dir,
            database_file,
            attachments_dir,
            temp_dir,
            search_index_dir,
            backups_dir,
        })
    }

    /// Create AppPaths with custom base directory (useful for testing)
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        let database_dir = base_dir.join("database");
        let database_file = database_dir.join("stone.db");
        let attachments_dir = base_dir.join("attachments");
        let temp_dir = base_dir.join("temp");
        let search_index_dir = base_dir.join("search_index");
        let backups_dir = base_dir.join("backups");

        Self {
            app_data_dir: base_dir,
            database_dir,
            database_file,
            attachments_dir,
            temp_dir,
            search_index_dir,
            backups_dir,
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> DomainResult<()> {
        let dirs = [
            &self.app_data_dir,
            &self.database_dir,
            &self.attachments_dir,
            &self.temp_dir,
            &self.search_index_dir,
            &self.backups_dir,
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| {
                    DomainError::FileStorageError(format!(
                        "Failed to create directory {}: {}",
                        dir.display(),
                        e
                    ))
                })?;
            }
        }

        Ok(())
    }

    /// Get platform-specific app data directory
    fn get_app_data_dir() -> DomainResult<PathBuf> {
        // Try to get from environment variable first (for testing/override)
        if let Ok(custom_dir) = std::env::var("STONE_APP_DATA_DIR") {
            return Ok(PathBuf::from(custom_dir));
        }

        // In development (debug build), use a local "data" directory relative to CWD
        // We use "target/data" to avoid triggering file watchers that monitor src/
        #[cfg(debug_assertions)]
        {
            if let Ok(cwd) = std::env::current_dir() {
                // If we are in src-tauri (common for cargo run), target is here.
                // If we are in project root, we might want src-tauri/target/data or just target/data.
                // To be safe and avoid loops, checking if "target" exists or just using it is usually fine as it is ignored.
                return Ok(cwd.join("target").join("data"));
            }
        }

        // Use platform-specific data directory
        dirs::data_local_dir()
            .map(|dir| dir.join("Stone"))
            .ok_or_else(|| {
                DomainError::ConfigurationError(
                    "Could not determine app data directory".to_string()
                )
            })
    }

    /// Get database file path as string
    pub fn database_file_str(&self) -> DomainResult<String> {
        self.database_file
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                DomainError::ConfigurationError(
                    "Database path contains invalid UTF-8".to_string()
                )
            })
    }
}

impl Default for AppPaths {
    fn default() -> Self {
        Self::new().expect("Failed to create default AppPaths")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_paths_creation() {
        let paths = AppPaths::new();
        assert!(paths.is_ok());

        let paths = paths.unwrap();
        assert!(paths.app_data_dir.to_str().unwrap().contains("Stone"));
        assert!(paths.database_file.to_str().unwrap().ends_with("stone.db"));
    }

    #[test]
    fn test_custom_base_dir() {
        let temp_dir = TempDir::new().unwrap();
        let paths = AppPaths::with_base_dir(temp_dir.path().to_path_buf());

        assert_eq!(paths.app_data_dir, temp_dir.path());
        assert_eq!(
            paths.database_dir,
            temp_dir.path().join("database")
        );
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        let paths = AppPaths::with_base_dir(temp_dir.path().to_path_buf());

        let result = paths.ensure_directories();
        assert!(result.is_ok());

        // Verify directories were created
        assert!(paths.database_dir.exists());
        assert!(paths.attachments_dir.exists());
        assert!(paths.temp_dir.exists());
        assert!(paths.search_index_dir.exists());
        assert!(paths.backups_dir.exists());
    }

    #[test]
    fn test_database_file_str() {
        let temp_dir = TempDir::new().unwrap();
        let paths = AppPaths::with_base_dir(temp_dir.path().to_path_buf());

        let db_str = paths.database_file_str();
        assert!(db_str.is_ok());
        assert!(db_str.unwrap().ends_with("stone.db"));
    }
}
