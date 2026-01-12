//! Database Migration Runner
//!
//! Handles running Diesel migrations for schema setup and updates.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use crate::domain::error::{DomainError, DomainResult};
use crate::shared::database::DbPool;

/// Embed migrations from the migrations/ directory
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Run all pending migrations
pub fn run_migrations(pool: &DbPool) -> DomainResult<()> {
    let mut conn = pool.get().map_err(|e| {
        DomainError::DatabaseError(format!("Failed to get database connection: {}", e))
    })?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to run migrations: {}", e))
        })?;

    Ok(())
}

/// Check if migrations are pending
pub fn has_pending_migrations(pool: &DbPool) -> DomainResult<bool> {
    let mut conn = pool.get().map_err(|e| {
        DomainError::DatabaseError(format!("Failed to get database connection: {}", e))
    })?;

    let pending = conn
        .has_pending_migration(MIGRATIONS)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to check pending migrations: {}", e))
        })?;

    Ok(pending)
}

/// Get list of applied migrations
pub fn applied_migrations(pool: &DbPool) -> DomainResult<Vec<String>> {
    let mut conn = pool.get().map_err(|e| {
        DomainError::DatabaseError(format!("Failed to get database connection: {}", e))
    })?;

    let applied = conn
        .applied_migrations()
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get applied migrations: {}", e))
        })?;

    Ok(applied.iter().map(|m| m.to_string()).collect())
}

/// Revert the last migration (USE WITH CAUTION)
pub fn revert_last_migration(pool: &DbPool) -> DomainResult<()> {
    let mut conn = pool.get().map_err(|e| {
        DomainError::DatabaseError(format!("Failed to get database connection: {}", e))
    })?;

    conn.revert_last_migration(MIGRATIONS)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to revert migration: {}", e))
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::create_pool;
    use tempfile::TempDir;

    #[test]
    fn test_run_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = create_pool(db_path.to_str().unwrap()).unwrap();

        let result = run_migrations(&pool);
        assert!(result.is_ok());

        // Verify no more pending migrations
        let pending = has_pending_migrations(&pool).unwrap();
        assert!(!pending);
    }

    #[test]
    fn test_applied_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = create_pool(db_path.to_str().unwrap()).unwrap();

        run_migrations(&pool).unwrap();

        let applied = applied_migrations(&pool).unwrap();
        assert!(!applied.is_empty());
    }
}
