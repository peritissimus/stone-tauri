//! Database Module
//!
//! Database lifecycle management, migrations, and seeding.

mod manager;
mod migrations;
mod seed;

pub use manager::DatabaseManager;
pub use migrations::run_migrations;
pub use seed::seed_initial_data;
