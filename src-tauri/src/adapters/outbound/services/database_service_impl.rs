//! Database Service Implementation
//!
//! Diesel-based implementation of database maintenance operations.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::adapters::outbound::persistence::{db_pool::get_connection, DbPool};
use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{DatabaseStatus, IntegrityCheckResult},
        outbound::DatabaseService,
    },
};

/// Diesel-based database service implementation
pub struct DieselDatabaseService {
    pool: Arc<DbPool>,
    database_path: String,
}

impl DieselDatabaseService {
    pub fn new(pool: Arc<DbPool>, database_path: String) -> Self {
        Self {
            pool,
            database_path,
        }
    }
}

#[async_trait]
impl DatabaseService for DieselDatabaseService {
    async fn get_status(&self) -> DomainResult<DatabaseStatus> {
        let pool = self.pool.clone();
        let database_path = self.database_path.clone();

        tokio::task::spawn_blocking(move || {
            // Try to get a connection to check if database is accessible
            let is_open = get_connection(&pool).is_ok();

            let size = if Path::new(&database_path).exists() {
                std::fs::metadata(&database_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            } else {
                0
            };

            Ok(DatabaseStatus {
                path: database_path,
                size,
                is_open,
            })
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    async fn vacuum(&self) -> DomainResult<()> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Execute VACUUM command
            diesel::sql_query("VACUUM")
                .execute(&mut conn)
                .map_err(|e| DomainError::DatabaseError(format!("Failed to vacuum database: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    async fn check_integrity(&self) -> DomainResult<IntegrityCheckResult> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Run SQLite integrity check
            #[derive(QueryableByName)]
            struct IntegrityRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                integrity_check: String,
            }

            let results: Vec<IntegrityRow> = diesel::sql_query("PRAGMA integrity_check")
                .load(&mut conn)
                .map_err(|e| DomainError::DatabaseError(format!("Integrity check failed: {}", e)))?;

            // SQLite returns "ok" if everything is fine, otherwise returns error messages
            let mut errors = Vec::new();
            let mut ok = true;

            for row in results {
                if row.integrity_check == "ok" {
                    // Everything is fine
                } else {
                    ok = false;
                    errors.push(row.integrity_check);
                }
            }

            Ok(IntegrityCheckResult { ok, errors })
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    #[tokio::test]
    async fn test_get_status() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let service = DieselDatabaseService::new(pool, ":memory:".to_string());

        let status = service.get_status().await.unwrap();
        assert_eq!(status.path, ":memory:");
        assert!(status.is_open);
    }

    #[tokio::test]
    async fn test_vacuum() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let service = DieselDatabaseService::new(pool, ":memory:".to_string());

        let result = service.vacuum().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_integrity() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let service = DieselDatabaseService::new(pool, ":memory:".to_string());

        let result = service.check_integrity().await.unwrap();
        assert!(result.ok);
        assert!(result.errors.is_empty());
    }
}
