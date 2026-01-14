/// Note Use Cases Implementation
///
/// Application layer implementations for note operations.
/// Orchestrates domain entities, repositories, and services.
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::Note,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{CreateNoteInput, NoteQuery, NoteUseCases, UpdateNoteInput},
        outbound::{
            EventPublisher, FileStorage, MarkdownProcessor, NoteRepository, WorkspaceRepository,
        },
    },
};

/// Implementation of all Note use cases
pub struct NoteUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
    markdown_processor: Arc<dyn MarkdownProcessor>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl NoteUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
        markdown_processor: Arc<dyn MarkdownProcessor>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            note_repository,
            workspace_repository,
            file_storage,
            markdown_processor,
            event_publisher,
        }
    }

    /// Strip the first H1 heading from markdown content
    fn strip_first_heading(&self, markdown: &str) -> String {
        let lines: Vec<&str> = markdown.lines().collect();
        let mut found_heading = false;
        let mut result: Vec<&str> = Vec::new();

        for line in lines {
            // Match H1 heading: # Title
            if !found_heading && line.starts_with("# ") {
                found_heading = true;
                continue; // Skip this line
            }
            result.push(line);
        }

        // Remove leading empty lines
        while !result.is_empty() && result[0].trim().is_empty() {
            result.remove(0);
        }

        result.join("\n")
    }
}

#[async_trait]
impl NoteUseCases for NoteUseCasesImpl {
    /// Create a new note
    async fn create_note(&self, input: CreateNoteInput) -> DomainResult<Note> {
        // Resolve workspace: use provided workspace_id or fall back to active
        let workspace_id = if let Some(ref ws_id) = input.workspace_id {
            Some(ws_id.clone())
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        }
        .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        let workspace = self
            .workspace_repository
            .find_by_id(&workspace_id)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Workspace not found".to_string()))?;

        // Create domain entity
        let mut note = Note::new(&input.title, Some(workspace_id.clone()))?;

        if let Some(notebook_id) = input.notebook_id {
            note.move_to_notebook(Some(notebook_id));
        }

        // Determine folder path (default to 'Personal')
        let folder_path = input
            .folder_path
            .clone()
            .unwrap_or_else(|| "Personal".to_string());

        // Generate filename unless provided as relative_path
        let filename = format!("{}.md", chrono::Utc::now().format("%Y%m%d-%H%M%S-%3f"));

        // Construct relative path: folderPath/filename.md
        let relative_path = input
            .relative_path
            .clone()
            .unwrap_or_else(|| format!("{}/{}", folder_path, filename));
        note.set_file_path(Some(relative_path.clone()))?;

        // Construct absolute path for file operations
        let absolute_path = Path::new(&workspace.folder_path).join(&relative_path);

        // Write initial content to file
        // Content is already Markdown from the frontend (via jsonToMarkdown), no conversion needed
        let content = input.content.unwrap_or_default();
        self.file_storage
            .write(absolute_path.to_str().unwrap(), &content)
            .await?;

        // Save to repository
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:created", serde_json::json!({"id": note.id}));
        }

        Ok(note)
    }

    /// Get a note by ID
    async fn get_note_by_id(&self, id: &str) -> DomainResult<Option<Note>> {
        self.note_repository.find_by_id(id).await
    }

    /// Get all notes with optional filtering
    async fn get_all_notes(&self, query: NoteQuery) -> DomainResult<Vec<Note>> {
        // Use active workspace if workspaceId not provided
        let workspace_id = if let Some(ref ws_id) = query.workspace_id {
            Some(ws_id.clone())
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        use crate::domain::ports::outbound::NoteFindOptions;

        self.note_repository
            .find_all(NoteFindOptions {
                workspace_id,
                notebook_id: query.notebook_id,
                is_favorite: query.is_favorite,
                is_archived: query.is_archived,
                is_deleted: Some(false),
                is_pinned: None,
                limit: query.limit,
                offset: None,
            })
            .await
    }

    /// Update an existing note
    async fn update_note(&self, input: UpdateNoteInput) -> DomainResult<Note> {
        let mut note = self
            .note_repository
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(input.id.clone()))?;

        if let Some(title) = input.title {
            note.update_title(title)?;
        }

        if let Some(notebook_id) = input.notebook_id {
            note.move_to_notebook(Some(notebook_id));
        }

        if let Some(is_favorite) = input.is_favorite {
            if is_favorite {
                note.mark_favorite();
            } else {
                note.unmark_favorite();
            }
        }

        if let Some(is_pinned) = input.is_pinned {
            if is_pinned {
                note.pin();
            } else {
                note.unpin();
            }
        }

        if let Some(is_archived) = input.is_archived {
            if is_archived {
                note.archive();
            } else {
                note.unarchive();
            }
        }

        // Update content if provided
        if let Some(content) = input.content {
            if let Some(ref file_path) = note.file_path {
                // Get workspace for file path resolution
                let workspace_id = note
                    .workspace_id
                    .clone()
                    .or_else(|| {
                        // This is a bit awkward but we can't await in a closure
                        None
                    })
                    .ok_or_else(|| {
                        DomainError::ValidationError("Note has no workspace".to_string())
                    })?;

                let workspace = self
                    .workspace_repository
                    .find_by_id(&workspace_id)
                    .await?
                    .ok_or_else(|| {
                        DomainError::ValidationError("Workspace not found".to_string())
                    })?;

                let absolute_path = Path::new(&workspace.folder_path).join(file_path);
                
                // Content is already Markdown from the frontend (via jsonToMarkdown), no conversion needed
                // Calling html_to_markdown would incorrectly convert literal <br/> tags in code blocks
                let body_markdown = content;

                // Prepend title heading
                let title_heading = format!("# {}\n\n", note.title);
                let full_markdown = title_heading + &body_markdown;

                self.file_storage
                    .write(absolute_path.to_str().unwrap(), &full_markdown)
                    .await?;
            }
        }

        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": note.id}));
        }

        Ok(note)
    }

    /// Soft delete a note
    async fn delete_note(&self, id: &str) -> DomainResult<()> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        note.delete();
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:deleted", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Permanently delete a note
    async fn permanently_delete_note(&self, id: &str) -> DomainResult<()> {
        let note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        // Delete file if it exists
        if let Some(ref file_path) = note.file_path {
            if let Some(ref workspace_id) = note.workspace_id {
                if let Some(workspace) = self.workspace_repository.find_by_id(workspace_id).await? {
                    let absolute_path = Path::new(&workspace.folder_path).join(file_path);
                    if self
                        .file_storage
                        .exists(absolute_path.to_str().unwrap())
                        .await?
                    {
                        self.file_storage
                            .delete(absolute_path.to_str().unwrap())
                            .await?;
                    }
                }
            }
        }

        self.note_repository.delete(id).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:deleted", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Restore a deleted note
    async fn restore_note(&self, id: &str) -> DomainResult<()> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        note.restore();
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Toggle favorite status
    async fn toggle_favorite(&self, id: &str) -> DomainResult<Note> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        if note.is_favorite {
            note.unmark_favorite();
        } else {
            note.mark_favorite();
        }

        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": id}));
        }

        Ok(note)
    }

    /// Toggle pin status
    async fn toggle_pin(&self, id: &str) -> DomainResult<Note> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        if note.is_pinned {
            note.unpin();
        } else {
            note.pin();
        }

        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": id}));
        }

        Ok(note)
    }

    /// Archive a note
    async fn archive_note(&self, id: &str) -> DomainResult<()> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        note.archive();
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Unarchive a note
    async fn unarchive_note(&self, id: &str) -> DomainResult<()> {
        let mut note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        note.unarchive();
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Move note to a different notebook
    async fn move_to_notebook(
        &self,
        note_id: &str,
        notebook_id: Option<String>,
    ) -> DomainResult<()> {
        let mut note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        note.move_to_notebook(notebook_id);
        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:updated", serde_json::json!({"id": note_id}));
        }

        Ok(())
    }

    /// Get recently updated notes
    async fn get_recent_notes(
        &self,
        limit: i64,
        workspace_id: Option<String>,
    ) -> DomainResult<Vec<Note>> {
        let ws_id = if let Some(id) = workspace_id {
            Some(id)
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        self.note_repository
            .find_recently_updated(limit, ws_id.as_deref())
            .await
    }

    /// Get favorite notes
    async fn get_favorites(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>> {
        let ws_id = if let Some(id) = workspace_id {
            Some(id)
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        self.note_repository.find_favorites(ws_id.as_deref()).await
    }

    /// Get archived notes
    async fn get_archived(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>> {
        let ws_id = if let Some(id) = workspace_id {
            Some(id)
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        self.note_repository.find_archived(ws_id.as_deref()).await
    }

    /// Get deleted notes (trash)
    async fn get_trash(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>> {
        let ws_id = if let Some(id) = workspace_id {
            Some(id)
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        self.note_repository.find_deleted(ws_id.as_deref()).await
    }

    /// Get note content from file
    async fn get_note_content(&self, id: &str) -> DomainResult<String> {
        let note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        if note.file_path.is_none() {
            return Ok(String::new());
        }

        let file_path = note.file_path.as_ref().unwrap();
        let workspace_id = note
            .workspace_id
            .ok_or_else(|| DomainError::ValidationError("Note has no workspace".to_string()))?;

        let workspace = self
            .workspace_repository
            .find_by_id(&workspace_id)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Workspace not found".to_string()))?;

        let absolute_path = Path::new(&workspace.folder_path).join(file_path);
        let path_str = absolute_path
            .to_str()
            .ok_or_else(|| DomainError::ValidationError("Invalid path".to_string()))?;

        if !self.file_storage.exists(path_str).await? {
            return Ok(String::new());
        }

        let markdown = self.file_storage.read(path_str).await?
            .ok_or_else(|| DomainError::ValidationError("Failed to read note content".to_string()))?;
        let body_markdown = self.strip_first_heading(&markdown);
        let html = self.markdown_processor.markdown_to_html(&body_markdown).await?;

        Ok(html)
    }

    /// Save note content to file
    async fn save_note_content(&self, id: &str, content: &str) -> DomainResult<()> {
        let note = self
            .note_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(id.to_string()))?;

        let file_path = note
            .file_path
            .ok_or_else(|| DomainError::ValidationError("Note has no file path".to_string()))?;

        let workspace_id = note
            .workspace_id
            .ok_or_else(|| DomainError::ValidationError("Note has no workspace".to_string()))?;

        let workspace = self
            .workspace_repository
            .find_by_id(&workspace_id)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Workspace not found".to_string()))?;

        let absolute_path = Path::new(&workspace.folder_path).join(&file_path);
        
        // Content is already Markdown from the frontend (via jsonToMarkdown), no conversion needed
        let body_markdown = content;

        // Prepend title heading
        let title_heading = format!("# {}\n\n", note.title);
        let full_markdown = title_heading + &body_markdown;

        self.file_storage
            .write(absolute_path.to_str().unwrap(), &full_markdown)
            .await?;

        Ok(())
    }

    /// Get note by file path
    async fn get_note_by_path(&self, file_path: &str) -> DomainResult<Option<Note>> {
        let workspace = self.workspace_repository.find_active().await?;
        let workspace_id = workspace.as_ref().map(|ws| ws.id.as_str());

        // First try to find in database
        if let Some(note) = self
            .note_repository
            .find_by_file_path(file_path, workspace_id)
            .await?
        {
            return Ok(Some(note));
        }

        // Not in database - check if file exists on disk
        let workspace =
            workspace.ok_or_else(|| DomainError::NoteNotFound(file_path.to_string()))?;

        let absolute_path = Path::new(&workspace.folder_path).join(file_path);
        let path_str = absolute_path
            .to_str()
            .ok_or_else(|| DomainError::ValidationError("Invalid path".to_string()))?;

        if !self.file_storage.exists(path_str).await? {
            return Ok(None);
        }

        // File exists on disk but not in DB - create the note entry
        let file_content = self.file_storage.read(path_str).await?
            .ok_or_else(|| DomainError::ValidationError("Failed to read note content".to_string()))?;
        let filename_without_ext = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled");

        let is_journal_file = file_path.starts_with("Journal/");
        let title = if is_journal_file {
            filename_without_ext.to_string()
        } else {
            self.markdown_processor
                .extract_title(&file_content)?
                .unwrap_or_else(|| filename_without_ext.to_string())
        };

        let mut note = Note::new(&title, Some(workspace.id.clone()))?;
        note.set_file_path(Some(file_path.to_string()))?;

        self.note_repository.save(&note).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("note:created", serde_json::json!({"id": note.id}));
        }

        Ok(Some(note))
    }

    /// Search notes by query
    async fn search_notes(
        &self,
        query: &str,
        workspace_id: Option<String>,
        limit: Option<i64>,
    ) -> DomainResult<Vec<Note>> {
        let ws_id = if let Some(id) = workspace_id {
            Some(id)
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .map(|ws| ws.id)
        };

        self.note_repository
            .search_by_title(query, ws_id.as_deref(), limit)
            .await
    }
}
