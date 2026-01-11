/// Stone - A note-taking app built with Tauri and Rust
///
/// Architecture: Hexagonal (Ports & Adapters)
/// Implementing domain and application layers

// Declare modules
pub mod application;
pub mod domain;

// Re-export for convenience
pub use domain::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
