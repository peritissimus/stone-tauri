//! Application State
//!
//! Manages dependency injection for Tauri command handlers.
//! Contains all use case implementations that are shared across commands.

use std::sync::Arc;

use crate::domain::ports::inbound::{
    AttachmentUseCases, DatabaseUseCases, ExportUseCases, GitUseCases, GraphUseCases,
    NoteUseCases, NotebookUseCases, QuickCaptureUseCases, SearchUseCases, SettingsUseCases,
    SystemUseCases, TagUseCases, TaskUseCases, TopicUseCases, VersionUseCases, WorkspaceUseCases,
};

/// Application State
///
/// This struct holds all the use case implementations and is passed to Tauri commands.
/// It's managed by Tauri's state management system.
#[derive(Clone)]
pub struct AppState {
    // Core domain use cases
    pub note_usecases: Arc<dyn NoteUseCases>,
    pub notebook_usecases: Arc<dyn NotebookUseCases>,
    pub workspace_usecases: Arc<dyn WorkspaceUseCases>,
    pub tag_usecases: Arc<dyn TagUseCases>,
    pub topic_usecases: Arc<dyn TopicUseCases>,

    // Supporting use cases
    pub attachment_usecases: Arc<dyn AttachmentUseCases>,
    pub version_usecases: Arc<dyn VersionUseCases>,
    pub settings_usecases: Arc<dyn SettingsUseCases>,

    // Feature use cases
    pub search_usecases: Arc<dyn SearchUseCases>,
    pub graph_usecases: Arc<dyn GraphUseCases>,
    pub quick_capture_usecases: Arc<dyn QuickCaptureUseCases>,
    pub task_usecases: Arc<dyn TaskUseCases>,

    // System use cases
    pub database_usecases: Arc<dyn DatabaseUseCases>,
    pub git_usecases: Arc<dyn GitUseCases>,
    pub export_usecases: Arc<dyn ExportUseCases>,
    pub system_usecases: Arc<dyn SystemUseCases>,
}

impl AppState {
    /// Create a new AppState with all use case implementations
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        note_usecases: Arc<dyn NoteUseCases>,
        notebook_usecases: Arc<dyn NotebookUseCases>,
        workspace_usecases: Arc<dyn WorkspaceUseCases>,
        tag_usecases: Arc<dyn TagUseCases>,
        topic_usecases: Arc<dyn TopicUseCases>,
        attachment_usecases: Arc<dyn AttachmentUseCases>,
        version_usecases: Arc<dyn VersionUseCases>,
        settings_usecases: Arc<dyn SettingsUseCases>,
        search_usecases: Arc<dyn SearchUseCases>,
        graph_usecases: Arc<dyn GraphUseCases>,
        quick_capture_usecases: Arc<dyn QuickCaptureUseCases>,
        task_usecases: Arc<dyn TaskUseCases>,
        database_usecases: Arc<dyn DatabaseUseCases>,
        git_usecases: Arc<dyn GitUseCases>,
        export_usecases: Arc<dyn ExportUseCases>,
        system_usecases: Arc<dyn SystemUseCases>,
    ) -> Self {
        Self {
            note_usecases,
            notebook_usecases,
            workspace_usecases,
            tag_usecases,
            topic_usecases,
            attachment_usecases,
            version_usecases,
            settings_usecases,
            search_usecases,
            graph_usecases,
            quick_capture_usecases,
            task_usecases,
            database_usecases,
            git_usecases,
            export_usecases,
            system_usecases,
        }
    }
}
