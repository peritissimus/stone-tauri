//! Database Manager
//!
//! Manages database lifecycle: initialization, migrations, backups, optimization.

use std::sync::Arc;
use std::path::Path;
use diesel::prelude::*;
use crate::{
    domain::error::{DomainError, DomainResult},
    infrastructure::config::DatabaseConfig,
    shared::database::{DbPool, create_pool},
};
use super::{run_migrations, seed_initial_data};

/// Database integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Database manager for lifecycle operations
pub struct DatabaseManager {
    pool: Arc<DbPool>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(config: DatabaseConfig) -> DomainResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.database_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DomainError::DatabaseError(format!(
                        "Failed to create database directory: {}",
                        e
                    ))
                })?;
            }
        }

        // Create connection pool
        let db_url = config.database_url();
        let pool = create_pool(&db_url).map_err(|e| {
            DomainError::DatabaseError(format!("Failed to create database pool: {}", e))
        })?;

        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }

    /// Initialize the database (run migrations and seed)
    pub async fn initialize(&self) -> DomainResult<()> {
        log::info!("Initializing database...");

        // Run migrations
        log::info!("Running database migrations...");
        run_migrations(&self.pool)?;

        // Seed initial data
        log::info!("Seeding initial data...");
        seed_initial_data(self.pool.clone()).await?;

        // Configure SQLite pragmas
        self.configure_pragmas()?;

        log::info!("Database initialized successfully");
        Ok(())
    }

    /// Configure SQLite pragmas for performance and safety
    fn configure_pragmas(&self) -> DomainResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get connection: {}", e))
        })?;

        // Enable Write-Ahead Logging (WAL) mode
        if self.config.enable_wal {
            diesel::sql_query("PRAGMA journal_mode = WAL")
                .execute(&mut conn)
                .map_err(|e| {
                    DomainError::DatabaseError(format!("Failed to enable WAL mode: {}", e))
                })?;
        }

        // Enable foreign keys
        if self.config.enable_foreign_keys {
            diesel::sql_query("PRAGMA foreign_keys = ON")
                .execute(&mut conn)
                .map_err(|e| {
                    DomainError::DatabaseError(format!("Failed to enable foreign keys: {}", e))
                })?;
        }

        // Set busy timeout
        let busy_timeout_query = format!(
            "PRAGMA busy_timeout = {}",
            self.config.busy_timeout_ms
        );
        diesel::sql_query(busy_timeout_query)
            .execute(&mut conn)
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to set busy timeout: {}", e))
            })?;

        Ok(())
    }

    /// Get the database connection pool
    pub fn get_pool(&self) -> Arc<DbPool> {
        self.pool.clone()
    }

    /// Backup the database to a specified path
    pub async fn backup(&self, backup_path: &str) -> DomainResult<()> {
        let source_path = self.config.database_path.clone();
        let backup_path = Path::new(backup_path);

        // Ensure backup directory exists
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainError::DatabaseError(format!("Failed to create backup directory: {}", e))
            })?;
        }

        // Copy database file
        tokio::fs::copy(&source_path, backup_path)
            .await
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to backup database: {}", e))
            })?;

        // Also copy WAL and SHM files if they exist
        if self.config.enable_wal {
            let wal_source = source_path.with_extension("db-wal");
            let wal_dest = backup_path.with_extension("db-wal");
            if wal_source.exists() {
                let _ = tokio::fs::copy(&wal_source, &wal_dest).await;
            }

            let shm_source = source_path.with_extension("db-shm");
            let shm_dest = backup_path.with_extension("db-shm");
            if shm_source.exists() {
                let _ = tokio::fs::copy(&shm_source, &shm_dest).await;
            }
        }

        log::info!("Database backed up to: {}", backup_path.display());
        Ok(())
    }

    /// Restore database from a backup
    pub async fn restore(&self, backup_path: &str) -> DomainResult<()> {
        let backup_path = Path::new(backup_path);
        let target_path = self.config.database_path.clone();

        if !backup_path.exists() {
            return Err(DomainError::DatabaseError(
                "Backup file does not exist".to_string(),
            ));
        }

        // Copy backup to database location
        tokio::fs::copy(backup_path, &target_path)
            .await
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to restore database: {}", e))
            })?;

        log::info!("Database restored from: {}", backup_path.display());

        // Reinitialize after restore
        self.configure_pragmas()?;

        Ok(())
    }

    /// Optimize the database (VACUUM and ANALYZE)
    pub async fn optimize(&self) -> DomainResult<()> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| {
                DomainError::DatabaseError(format!("Failed to get connection: {}", e))
            })?;

            log::info!("Running VACUUM...");
            diesel::sql_query("VACUUM")
                .execute(&mut conn)
                .map_err(|e| {
                    DomainError::DatabaseError(format!("Failed to run VACUUM: {}", e))
                })?;

            log::info!("Running ANALYZE...");
            diesel::sql_query("ANALYZE")
                .execute(&mut conn)
                .map_err(|e| {
                    DomainError::DatabaseError(format!("Failed to run ANALYZE: {}", e))
                })?;

            log::info!("Database optimization completed");
            Ok(())
        })
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to spawn optimization task: {}", e))
        })??;

        Ok(())
    }

    /// Check database integrity
    pub async fn check_integrity(&self) -> DomainResult<IntegrityReport> {
        let pool = self.pool.clone();

        let report = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| {
                DomainError::DatabaseError(format!("Failed to get connection: {}", e))
            })?;

            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Run integrity_check
            let result: Result<String, _> = diesel::sql_query("PRAGMA integrity_check")
                .get_result::<(String,)>(&mut conn)
                .map(|(s,)| s);

            match result {
                Ok(status) if status == "ok" => {}
                Ok(status) => errors.push(format!("Integrity check failed: {}", status)),
                Err(e) => errors.push(format!("Failed to run integrity check: {}", e)),
            }

            // Check foreign key violations
            let fk_result: Result<Vec<String>, _> = diesel::sql_query("PRAGMA foreign_key_check")
                .load::<(String,)>(&mut conn)
                .map(|rows| rows.into_iter().map(|(s,)| s).collect());

            match fk_result {
                Ok(violations) if !violations.is_empty() => {
                    warnings.push(format!(
                        "Foreign key violations found: {}",
                        violations.len()
                    ));
                }
                Ok(_) => {}
                Err(e) => warnings.push(format!("Failed to check foreign keys: {}", e)),
            }

            let is_valid = errors.is_empty();

            Ok(IntegrityReport {
                is_valid,
                errors,
                warnings,
            })
        })
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to spawn integrity check task: {}", e))
        })??;

        Ok(report)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> DomainResult<DatabaseStats> {
        let pool = self.pool.clone();
        let db_path = self.config.database_path.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| {
                DomainError::DatabaseError(format!("Failed to get connection: {}", e))
            })?;

            // Get page count and page size
            let page_count: i64 = diesel::sql_query("PRAGMA page_count")
                .get_result::<(i64,)>(&mut conn)
                .map(|(p,)| p)
                .unwrap_or(0);

            let page_size: i64 = diesel::sql_query("PRAGMA page_size")
                .get_result::<(i64,)>(&mut conn)
                .map(|(p,)| p)
                .unwrap_or(4096);

            let database_size = page_count * page_size;

            // Get file size from filesystem
            let file_size = std::fs::metadata(&db_path)
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            Ok(DatabaseStats {
                database_size,
                file_size,
                page_count,
                page_size,
            })
        })
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get database stats: {}", e))
        })??
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub database_size: i64,
    pub file_size: i64,
    pub page_count: i64,
    pub page_size: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::infrastructure::config::DatabaseConfig;

    #[tokio::test]
    async fn test_database_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await.unwrap();
        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_path = temp_dir.path().join("backup.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await.unwrap();
        manager.initialize().await.unwrap();

        // Backup
        let result = manager.backup(backup_path.to_str().unwrap()).await;
        assert!(result.is_ok());
        assert!(backup_path.exists());

        // Restore
        let result = manager.restore(backup_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_optimize() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await.unwrap();
        manager.initialize().await.unwrap();

        let result = manager.optimize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await.unwrap();
        manager.initialize().await.unwrap();

        let report = manager.check_integrity().await.unwrap();
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path);

        let manager = DatabaseManager::new(config).await.unwrap();
        manager.initialize().await.unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert!(stats.file_size > 0);
        assert!(stats.page_count > 0);
    }
}
