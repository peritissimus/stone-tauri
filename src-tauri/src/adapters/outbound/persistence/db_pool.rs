//! Database Connection Pool
//!
//! Provides a shared connection pool for Diesel SQLite connections.

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::domain::errors::{DomainError, DomainResult};

/// Type alias for the database connection pool
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Type alias for a pooled connection
pub type DbConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

/// Creates a new database connection pool
///
/// # Arguments
/// * `database_url` - SQLite database URL (e.g., "sqlite://./stone.db")
///
/// # Returns
/// * `DomainResult<DbPool>` - The connection pool or an error
pub fn create_pool(database_url: &str) -> DomainResult<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    Pool::builder()
        .max_size(10)
        .min_idle(Some(2))
        .test_on_check_out(true)
        .build(manager)
        .map_err(|e| DomainError::DatabaseError(format!("Failed to create connection pool: {}", e)))
}

/// Gets a connection from the pool
///
/// # Arguments
/// * `pool` - The database connection pool
///
/// # Returns
/// * `DomainResult<DbConnection>` - A pooled connection or an error
pub fn get_connection(pool: &DbPool) -> DomainResult<DbConnection> {
    pool.get()
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get connection from pool: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pool() {
        let result = create_pool(":memory:");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_connection() {
        let pool = create_pool(":memory:").unwrap();
        let result = get_connection(&pool);
        assert!(result.is_ok());
    }
}
