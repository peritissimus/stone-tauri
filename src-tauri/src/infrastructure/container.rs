//! Dependency Injection Container
//!
//! Wires up all repositories, services, and use cases.

use std::sync::Arc;
use tauri::AppHandle;

use crate::{
    adapters::{
        inbound::AppState,
        outbound::{
            persistence::*,
            services::*,
            storage::TokioFileStorage,
        },
    },
    application::usecases::*,
    domain::errors::DomainResult,
    shared::database::DbPool,
};

/// Container for all application dependencies
pub struct Container {
    pub app_state: AppState,
}

impl Container {
    /// Build the dependency injection container
    pub async fn build(pool: Arc<DbPool>, app_handle: AppHandle) -> DomainResult<Self> {
        tracing::info!("Building dependency injection container...");

        // === Repositories ===
        let note_repository = Arc::new(DieselNoteRepository::new(pool.clone()));
        let notebook_repository = Arc::new(DieselNotebookRepository::new(pool.clone()));
        let workspace_repository = Arc::new(DieselWorkspaceRepository::new(pool.clone()));
        let tag_repository = Arc::new(DieselTagRepository::new(pool.clone()));
        let topic_repository = Arc::new(DieselTopicRepository::new(pool.clone()));
        let attachment_repository = Arc::new(DieselAttachmentRepository::new(pool.clone()));
        let version_repository = Arc::new(DieselVersionRepository::new(pool.clone()));
        let settings_repository = Arc::new(DieselSettingsRepository::new(pool.clone()));
        let link_repository = Arc::new(DieselNoteLinkRepository::new(pool.clone()));

        // === Services ===
        let file_storage = Arc::new(TokioFileStorage::new());
        let markdown_processor = Arc::new(PulldownMarkdownService::new());
        let system_service = Arc::new(TauriSystemService::new(app_handle.clone()));
        let git_service = Arc::new(Git2Service::new());
        let search_engine = Arc::new(StubSearchService::new(pool.clone()));
        let embedding_service = Arc::new(StubEmbeddingService::new());
        let export_service = Arc::new(StubExportService::new());
        let database_path = std::env::var("STONE_DB_PATH")
            .unwrap_or_else(|_| "stone.db".to_string());
        let database_service = Arc::new(DieselDatabaseService::new(pool.clone(), database_path));

        // Event publisher is optional (not implemented yet)
        let event_publisher: Option<Arc<dyn crate::domain::ports::outbound::EventPublisher>> = None;

        // === Use Cases ===
        let note_usecases = Arc::new(NoteUseCasesImpl::new(
            note_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
            markdown_processor.clone(),
            event_publisher.clone(),
        ));

        let notebook_usecases = Arc::new(NotebookUseCasesImpl::new(
            notebook_repository.clone(),
            event_publisher.clone(),
        ));

        let workspace_usecases = Arc::new(WorkspaceUseCasesImpl::new(
            workspace_repository.clone(),
            note_repository.clone(),
            file_storage.clone(),
            system_service.clone(),
            markdown_processor.clone(),
            event_publisher.clone(),
        ));

        let tag_usecases = Arc::new(TagUseCasesImpl::new(
            tag_repository.clone(),
            event_publisher.clone(),
        ));

        let topic_usecases = Arc::new(TopicUseCasesImpl::new(
            topic_repository.clone(),
            note_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
            embedding_service.clone(),
            markdown_processor.clone(),
            event_publisher.clone(),
        ));

        let attachment_usecases = Arc::new(AttachmentUseCasesImpl::new(
            note_repository.clone(),
            attachment_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
        ));

        let version_usecases = Arc::new(VersionUseCasesImpl::new(
            note_repository.clone(),
            version_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
        ));

        let settings_usecases = Arc::new(SettingsUseCasesImpl::new(
            settings_repository.clone(),
        ));

        let search_usecases = Arc::new(SearchUseCasesImpl::new(
            note_repository.clone(),
            search_engine.clone(),
            embedding_service.clone(),
        ));

        let graph_usecases = Arc::new(GraphUseCasesImpl::new(
            note_repository.clone(),
            link_repository.clone(),
            workspace_repository.clone(),
        ));

        let quick_capture_usecases = Arc::new(QuickCaptureUseCasesImpl::new(
            note_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
        ));

        let task_usecases = Arc::new(TaskUseCasesImpl::new(
            note_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
        ));

        let database_usecases = Arc::new(DatabaseUseCasesImpl::new(
            database_service.clone(),
        ));

        let git_usecases = Arc::new(GitUseCasesImpl::new(
            workspace_repository.clone(),
            git_service.clone(),
        ));

        let export_usecases = Arc::new(ExportUseCasesImpl::new(
            note_repository.clone(),
            workspace_repository.clone(),
            file_storage.clone(),
            markdown_processor.clone(),
            export_service.clone(),
        ));

        let system_usecases = Arc::new(SystemUseCasesImpl::new(
            system_service.clone(),
        ));

        // === Build AppState ===
        let app_state = AppState::new(
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
        );

        tracing::info!("Dependency injection container built successfully");

        Ok(Self { app_state })
    }
}
