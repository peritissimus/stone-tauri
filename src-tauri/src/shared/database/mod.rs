/// Database Module
///
/// Provides database connection pooling and utilities for SQLite via Diesel.
///
/// Key components:
/// - DbPool: R2D2 connection pool type
/// - DbConnection: Single database connection type
/// - create_pool(): Initialize connection pool with configuration
/// - DatabaseConfig: Configuration for database initialization
/// - schema: Diesel schema definitions for all tables

pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::error::Error;

/// Database connection pool type
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Single database connection type
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// Database configuration
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_pool_size: u32,
    pub min_idle: Option<u32>,
    pub connection_timeout_secs: u64,
}

impl DatabaseConfig {
    /// Create new database configuration with defaults
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_pool_size: 10,
            min_idle: Some(1),
            connection_timeout_secs: 30,
        }
    }

    /// Create configuration for testing (in-memory database)
    pub fn in_memory() -> Self {
        Self {
            database_url: ":memory:".to_string(),
            max_pool_size: 1,
            min_idle: None,
            connection_timeout_secs: 5,
        }
    }
}

/// Create a database connection pool
///
/// # Arguments
/// * `config` - Database configuration
///
/// # Returns
/// * `DbPool` - Configured connection pool
///
/// # Errors
/// Returns error if pool creation fails
pub fn create_pool(config: DatabaseConfig) -> Result<DbPool, Box<dyn Error + Send + Sync>> {
    let manager = ConnectionManager::<SqliteConnection>::new(config.database_url);

    let mut pool_builder = r2d2::Pool::builder()
        .max_size(config.max_pool_size)
        .connection_timeout(std::time::Duration::from_secs(
            config.connection_timeout_secs,
        ));

    if let Some(min_idle) = config.min_idle {
        pool_builder = pool_builder.min_idle(Some(min_idle));
    }

    let pool = pool_builder.build(manager)?;

    Ok(pool)
}
