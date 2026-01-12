//! Database Configuration
//!
//! Configuration for SQLite database connection and settings.

use std::path::PathBuf;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the database file
    pub database_path: PathBuf,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Enable WAL (Write-Ahead Logging) mode
    pub enable_wal: bool,

    /// Enable foreign keys
    pub enable_foreign_keys: bool,

    /// Busy timeout in milliseconds
    pub busy_timeout_ms: u32,
}

impl DatabaseConfig {
    /// Create new database configuration with defaults
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            max_connections: 16,
            connection_timeout_secs: 30,
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout_ms: 5000,
        }
    }

    /// Create configuration optimized for development
    pub fn development(database_path: PathBuf) -> Self {
        Self {
            database_path,
            max_connections: 8,
            connection_timeout_secs: 10,
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout_ms: 3000,
        }
    }

    /// Create configuration optimized for production
    pub fn production(database_path: PathBuf) -> Self {
        Self {
            database_path,
            max_connections: 32,
            connection_timeout_secs: 30,
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout_ms: 10000,
        }
    }

    /// Get database URL for Diesel
    pub fn database_url(&self) -> String {
        format!("sqlite://{}", self.database_path.display())
    }

    /// Get database file path as string
    pub fn database_path_str(&self) -> Option<String> {
        self.database_path.to_str().map(|s| s.to_string())
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::new(PathBuf::from("stone.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_new() {
        let config = DatabaseConfig::new(PathBuf::from("/tmp/test.db"));

        assert_eq!(config.database_path, PathBuf::from("/tmp/test.db"));
        assert_eq!(config.max_connections, 16);
        assert!(config.enable_wal);
        assert!(config.enable_foreign_keys);
    }

    #[test]
    fn test_database_url() {
        let config = DatabaseConfig::new(PathBuf::from("/tmp/test.db"));
        let url = config.database_url();

        assert!(url.starts_with("sqlite://"));
        assert!(url.contains("test.db"));
    }

    #[test]
    fn test_development_config() {
        let config = DatabaseConfig::development(PathBuf::from("/tmp/dev.db"));

        assert_eq!(config.max_connections, 8);
        assert_eq!(config.connection_timeout_secs, 10);
    }

    #[test]
    fn test_production_config() {
        let config = DatabaseConfig::production(PathBuf::from("/tmp/prod.db"));

        assert_eq!(config.max_connections, 32);
        assert_eq!(config.busy_timeout_ms, 10000);
    }
}
