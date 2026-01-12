/// Stone - A note-taking app built with Tauri and Rust
///
/// Architecture: Hexagonal (Ports & Adapters)
/// Layers: domain, application, adapters, shared

// Declare modules
pub mod adapters;
pub mod application;
pub mod domain;
pub mod shared;

// Re-export for convenience
pub use domain::*;
pub use shared::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
