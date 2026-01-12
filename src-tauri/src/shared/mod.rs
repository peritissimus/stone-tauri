/// Shared Module
///
/// Shared utilities and types used across all hex layers (domain, application, infrastructure).
/// This module provides cross-cutting concerns like database utilities.
///
/// Structure:
/// - database: Database connection pooling, schema, and utilities

pub mod database;

// Re-export commonly used items
pub use database::{create_pool, schema, DatabaseConfig, DbConnection, DbPool};
