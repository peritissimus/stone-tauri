//! Inbound Adapters (Driving Adapters)
//!
//! Tauri command handlers and IPC layer.
//! These adapters receive requests from the frontend and call use cases.

pub mod app_state;

// Command Modules
pub mod note_commands;
pub mod workspace_commands;
pub mod notebook_commands;
pub mod tag_commands;
pub mod topic_commands;
pub mod search_commands;
pub mod git_commands;
pub mod database_commands;
pub mod system_commands;
pub mod export_commands;
pub mod attachment_commands;
pub mod settings_commands;
pub mod graph_commands;
pub mod quick_capture_commands;
pub mod task_commands;
pub mod version_commands;

// Re-exports
pub use app_state::AppState;
