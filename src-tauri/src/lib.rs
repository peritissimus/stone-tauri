/// Stone - A note-taking app built with Tauri and Rust
///
/// Architecture: Hexagonal (Ports & Adapters)
/// Layers: domain, application, adapters, infrastructure, shared

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
#[cfg(target_os = "macos")]
use tauri::window::{Effect, EffectsBuilder};

// Declare modules
pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod shared;

// Re-export for convenience
pub use domain::*;

// Re-export infrastructure types
pub use infrastructure::{AppConfig, AppPaths, Container, DatabaseConfig, DatabaseManager, Environment};

// Re-export shared database types
pub use shared::database::{create_pool, DbPool};

// Import all command modules
use adapters::inbound::{
    attachment_commands, database_commands, export_commands, git_commands, graph_commands,
    note_commands, notebook_commands, quick_capture_commands, search_commands, settings_commands,
    system_commands, tag_commands, task_commands, topic_commands, version_commands,
    workspace_commands, performance_commands,
};

/// Initialize the application
async fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Stone application...");

    #[cfg(target_os = "macos")]
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_effects(
            EffectsBuilder::new()
                .effects([Effect::UnderWindowBackground])
                .build(),
        );
    }

    // Get app handle
    let app_handle = app.handle().clone();

    // Register global shortcut for quick capture
    let shortcut_app = app_handle.clone();
    app_handle
        .global_shortcut()
        .on_shortcut("Alt+Space", move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Err(error) = adapters::inbound::ui::quick_capture_window::show(&shortcut_app) {
                    tracing::warn!("Failed to show quick capture window: {}", error);
                }
            }
        })?;

    // Load configuration
    tracing::info!("Loading configuration...");
    let config = AppConfig::load()?;

    // Ensure directories exist
    config.paths.ensure_directories()?;

    // Initialize database manager
    tracing::info!("Initializing database...");
    let db_manager = DatabaseManager::new(config.database.clone()).await?;
    db_manager.initialize().await?;

    let pool = db_manager.get_pool();

    // Build dependency injection container
    tracing::info!("Building dependency injection container...");
    let container = Container::build(pool.clone(), app_handle.clone()).await?;

    // Perform initial workspace sync
    tracing::info!("Syncing workspace with filesystem...");
    match container.app_state.workspace_usecases.sync_workspace(None).await {
        Ok(sync_result) => {
            tracing::info!(
                "Workspace sync completed: {} notes created, {} updated, {} deleted",
                sync_result.notes.created,
                sync_result.notes.updated,
                sync_result.notes.deleted
            );
        }
        Err(e) => {
            tracing::warn!("Initial workspace sync failed: {}", e);
        }
    }

    // Start file watcher for active workspace
    tracing::info!("Starting file watcher...");
    match container.app_state.workspace_usecases.get_active_workspace().await {
        Ok(Some(workspace)) => {
            match container.file_watcher.watch_workspace(&workspace).await {
                Ok(_) => {
                    tracing::info!("File watcher started for workspace: {}", workspace.name);
                }
                Err(e) => {
                    tracing::warn!("Failed to start file watcher: {}", e);
                }
            }
        }
        Ok(None) => {
            tracing::warn!("No active workspace found, file watcher not started");
        }
        Err(e) => {
            tracing::warn!("Failed to get active workspace: {}", e);
        }
    }

    // Register app state
    app.manage(container.app_state);
    app.manage(container.file_watcher);
    app.manage(db_manager);
    app.manage(performance_commands::PerformanceState::new());

    tracing::info!("Application initialized successfully");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                setup_app(app).await.map_err(|e| {
                    eprintln!("Failed to setup app: {}", e);
                    e
                })
            })
        })
        .invoke_handler(tauri::generate_handler![
            // Note commands
            note_commands::create_note,
            note_commands::get_note,
            note_commands::get_note_content,
            note_commands::save_note_content,
            note_commands::update_note,
            note_commands::delete_note,
            note_commands::restore_note,
            note_commands::get_all_notes,
            note_commands::move_note,
            note_commands::get_note_by_path,
            note_commands::toggle_favorite,
            note_commands::toggle_pin,
            note_commands::archive_note,
            note_commands::unarchive_note,
            note_commands::get_recent_notes,
            note_commands::get_favorites,
            note_commands::get_archived,
            note_commands::get_trash,
            // Workspace commands
            workspace_commands::create_workspace,
            workspace_commands::get_workspace,
            workspace_commands::list_workspaces,
            workspace_commands::get_active_workspace,
            workspace_commands::set_active_workspace,
            workspace_commands::update_workspace,
            workspace_commands::delete_workspace,
            workspace_commands::scan_workspace,
            workspace_commands::sync_workspace,
            // Notebook commands
            notebook_commands::create_notebook,
            notebook_commands::get_notebook,
            notebook_commands::list_notebooks,
            notebook_commands::update_notebook,
            notebook_commands::delete_notebook,
            notebook_commands::move_notebook,
            // Tag commands
            tag_commands::create_tag,
            tag_commands::get_tag,
            tag_commands::list_tags,
            tag_commands::update_tag,
            tag_commands::delete_tag,
            tag_commands::add_tag_to_note,
            tag_commands::remove_tag_from_note,
            tag_commands::get_tags_for_note,
            // Topic commands
            topic_commands::create_topic,
            topic_commands::get_topic,
            topic_commands::list_topics,
            topic_commands::update_topic,
            topic_commands::delete_topic,
            topic_commands::classify_note,
            topic_commands::classify_all_notes,
            topic_commands::get_notes_for_topic,
            topic_commands::get_topics_for_note,
            topic_commands::assign_topic_to_note,
            topic_commands::remove_topic_from_note,
            topic_commands::get_similar_notes,
            topic_commands::get_embedding_status,
            // Search commands
            search_commands::search_notes,
            search_commands::semantic_search,
            search_commands::hybrid_search,
            search_commands::search_by_date_range,
            // Git commands
            git_commands::git_init,
            git_commands::git_commit,
            git_commands::git_pull,
            git_commands::git_push,
            git_commands::git_sync,
            git_commands::get_git_status,
            git_commands::git_get_history,
            git_commands::git_set_remote,
            // Database commands
            database_commands::get_database_status,
            database_commands::vacuum_database,
            database_commands::check_database_integrity,
            // System commands
            system_commands::get_system_fonts,
            system_commands::show_in_folder,
            system_commands::open_external_url,
            // Export commands
            export_commands::export_note_html,
            export_commands::export_note_pdf,
            export_commands::export_note_markdown,
            // Attachment commands
            attachment_commands::add_attachment,
            attachment_commands::get_attachments_for_note,
            attachment_commands::delete_attachment,
            attachment_commands::upload_image,
            // Settings commands
            settings_commands::get_setting,
            settings_commands::set_setting,
            settings_commands::get_all_settings,
            // Graph commands
            graph_commands::get_backlinks,
            graph_commands::get_forward_links,
            graph_commands::get_graph_data,
            // Quick capture commands
            quick_capture_commands::append_to_journal,
            quick_capture_commands::hide_quick_capture,
            // Task commands
            task_commands::get_all_tasks,
            task_commands::get_note_tasks,
            task_commands::update_task_state,
            task_commands::toggle_task,
            // Version commands
            version_commands::get_versions,
            version_commands::get_version,
            version_commands::create_version,
            version_commands::restore_version,
            // Performance commands
            performance_commands::get_performance_snapshot,
            performance_commands::get_memory_metrics,
            performance_commands::get_cpu_metrics,
            performance_commands::get_ipc_stats,
            performance_commands::get_db_stats,
            performance_commands::get_startup_metrics,
            performance_commands::clear_performance_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
