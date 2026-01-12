/// Stone - A note-taking app built with Tauri and Rust
///
/// Architecture: Hexagonal (Ports & Adapters)
/// Layers: domain, application, adapters, infrastructure, shared

// Declare modules
pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod shared;

// Re-export for convenience
pub use domain::*;

// Re-export infrastructure config and database types
pub use infrastructure::{AppConfig, AppPaths, DatabaseConfig, Environment, DatabaseManager};

// Re-export shared database types (use qualified name to avoid conflicts)
pub use shared::database::{create_pool, DbPool};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
