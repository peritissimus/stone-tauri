/// Quick Capture Use Cases Implementation
///
/// Application layer implementations for quick capture operations.
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Local;

use crate::domain::{
    entities::Note,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{AppendToJournalResponse, QuickCaptureUseCases},
        outbound::{DomainEvent, EventPublisher, FileStorage, NoteRepository, WorkspaceRepository},
    },
};

/// Implementation of all Quick Capture use cases
pub struct QuickCaptureUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl QuickCaptureUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            note_repository,
            workspace_repository,
            file_storage,
            event_publisher,
        }
    }
}

#[async_trait]
impl QuickCaptureUseCases for QuickCaptureUseCasesImpl {
    /// Append content to today's journal note
    async fn append_to_journal(
        &self,
        content: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<AppendToJournalResponse> {
        use std::path::Path;

        // Get workspace - ✅ ASYNC
        let workspace = if let Some(workspace_id) = workspace_id {
            self.workspace_repository.find_by_id(workspace_id).await?
        } else {
            self.workspace_repository.find_active().await?
        }
        .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        // Get today's date for journal
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string(); // e.g., "2026-01-11"
        let journal_title = date_str.clone();
        let journal_file_path = format!("Journal/{}.md", date_str);

        // Fast lookup: directly query by file path - ✅ ASYNC
        let journal_note = self
            .note_repository
            .find_by_file_path(&journal_file_path, Some(&workspace.id))
            .await?;

        // Format timestamp and entry
        let timestamp = now.format("%H:%M").to_string(); // e.g., "14:30"
        let entry_content = format!("\n\n[{}] {}", timestamp, content);

        // Build absolute path
        let absolute_path = Path::new(&workspace.folder_path)
            .join(&journal_file_path)
            .to_string_lossy()
            .to_string();

        // Scenario 1: Journal note exists in DB
        if let Some(journal_note) = journal_note {
            // Read existing content - ✅ ASYNC
            let existing_content = self
                .file_storage
                .read(&absolute_path)
                .await?
                .unwrap_or_default();

            // Append entry - ✅ ASYNC
            let new_content = format!("{}{}", existing_content, entry_content);
            self.file_storage
                .write(&absolute_path, &new_content)
                .await?;

            // Update note timestamp - ✅ ASYNC
            self.note_repository.save(&journal_note).await?;

            // Emit note:updated event for immediate UI refresh
            self.event_publisher.publish(DomainEvent::NoteUpdated {
                timestamp: chrono::Utc::now(),
                id: journal_note.id.clone(),
                title: journal_note.title.clone(),
                changes: vec!["content".to_string()],
            });

            return Ok(AppendToJournalResponse {
                note_id: journal_note.id,
                appended: true,
            });
        }

        // Scenario 2: File exists but not in DB - ✅ ASYNC
        let file_exists = self.file_storage.exists(&absolute_path).await?;

        if file_exists {
            // Read existing content - ✅ ASYNC
            let existing_content = self
                .file_storage
                .read(&absolute_path)
                .await?
                .unwrap_or_default();

            // Append entry - ✅ ASYNC
            let new_content = format!("{}{}", existing_content, entry_content);
            self.file_storage
                .write(&absolute_path, &new_content)
                .await?;

            // Create DB entry - ❌ SYNC but returns Result
            let mut note = Note::new(journal_title.clone(), Some(workspace.id.clone()))?;

            // Set file path - ❌ SYNC but returns Result
            note.set_file_path(Some(journal_file_path.clone()))?;

            // Save note - ✅ ASYNC
            self.note_repository.save(&note).await?;

            // Emit note:updated event for immediate UI refresh
            self.event_publisher.publish(DomainEvent::NoteUpdated {
                timestamp: chrono::Utc::now(),
                id: note.id.clone(),
                title: note.title.clone(),
                changes: vec!["content".to_string()],
            });

            return Ok(AppendToJournalResponse {
                note_id: note.id,
                appended: true,
            });
        }

        // Scenario 3: Create new journal
        // Ensure journal directory exists - ✅ ASYNC
        let journal_dir = Path::new(&workspace.folder_path)
            .join("Journal")
            .to_string_lossy()
            .to_string();

        self.file_storage.create_directory(&journal_dir).await?;

        // Create initial content with title
        let initial_content = format!("# {}{}", journal_title, entry_content);

        // Write file - ✅ ASYNC
        self.file_storage
            .write(&absolute_path, &initial_content)
            .await?;

        // Create note - ❌ SYNC but returns Result
        let mut note = Note::new(journal_title, Some(workspace.id.clone()))?;

        // Set file path - ❌ SYNC but returns Result
        note.set_file_path(Some(journal_file_path))?;

        // Save note - ✅ ASYNC
        self.note_repository.save(&note).await?;

        // Emit note:created event for new journal entries
        self.event_publisher.publish(DomainEvent::NoteCreated {
            timestamp: chrono::Utc::now(),
            id: note.id.clone(),
            title: note.title.clone(),
            workspace_id: note.workspace_id.clone(),
            notebook_id: note.notebook_id.clone(),
            file_path: note.file_path.clone(),
        });

        Ok(AppendToJournalResponse {
            note_id: note.id,
            appended: false,
        })
    }
}
