//! Configuration Module
//!
//! Application configuration, database config, and path resolution.

mod app_config;
mod database_config;
mod paths;

pub use app_config::{AppConfig, Environment};
pub use database_config::DatabaseConfig;
pub use paths::AppPaths;
