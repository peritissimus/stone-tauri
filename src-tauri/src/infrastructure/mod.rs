//! Infrastructure Layer
//!
//! Composition root of the hexagonal architecture.
//! Wires together all components (domain, application, adapters)
//! and manages application lifecycle, configuration, and cross-cutting concerns.

pub mod config;
pub mod database;
pub mod container;

#[cfg(target_os = "macos")]
pub mod nspanel;

// Re-exports
pub use config::{AppConfig, DatabaseConfig, AppPaths, Environment};
pub use database::{DatabaseManager, run_migrations, seed_initial_data};
pub use container::Container;
