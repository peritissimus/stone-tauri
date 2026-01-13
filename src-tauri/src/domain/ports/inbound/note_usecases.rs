/// Note Use Cases Port (Inbound)
///
/// Defines what the application CAN DO with Notes.
/// Implementations live in the application layer.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{entities::Note, errors::DomainResult};

/// Input for creating a new note
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteInput {
    pub title: String,
    pub content: Option<String>,
    #[serde(alias = "notebookId")]
    pub notebook_id: Option<String>,
    #[serde(alias = "folderPath")]
    pub workspace_id: Option<String>,
}

/// Input for updating a note
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    #[serde(alias = "notebookId")]
    pub notebook_id: Option<String>,
    #[serde(alias = "isFavorite")]
    pub is_favorite: Option<bool>,
    #[serde(alias = "isPinned")]
    pub is_pinned: Option<bool>,
    #[serde(alias = "isArchived")]
    pub is_archived: Option<bool>,
}

/// Query parameters for listing notes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteQuery {
    #[serde(alias = "workspaceId")]
    pub workspace_id: Option<String>,
    #[serde(alias = "notebookId")]
    pub notebook_id: Option<String>,
    #[serde(alias = "isFavorite")]
    pub is_favorite: Option<bool>,
    #[serde(alias = "isArchived")]
    pub is_archived: Option<bool>,
    pub limit: Option<i64>,
}

/// Filter type for notes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteFilter {
    All,
    Favorites,
    Pinned,
    Archived,
    Trash,
}

/// Note Use Cases Interface
///
/// This trait defines all operations that can be performed on notes.
/// It serves as the primary entry point from the adapters layer.
#[async_trait]
pub trait NoteUseCases: Send + Sync {
    /// Create a new note
    async fn create_note(&self, input: CreateNoteInput) -> DomainResult<Note>;

    /// Get a note by ID
    async fn get_note_by_id(&self, id: &str) -> DomainResult<Option<Note>>;

    /// Get all notes with optional filtering
    async fn get_all_notes(&self, query: NoteQuery) -> DomainResult<Vec<Note>>;

    /// Update an existing note
    async fn update_note(&self, input: UpdateNoteInput) -> DomainResult<Note>;

    /// Soft delete a note
    async fn delete_note(&self, id: &str) -> DomainResult<()>;

    /// Permanently delete a note
    async fn permanently_delete_note(&self, id: &str) -> DomainResult<()>;

    /// Restore a deleted note
    async fn restore_note(&self, id: &str) -> DomainResult<()>;

    /// Toggle favorite status
    async fn toggle_favorite(&self, id: &str) -> DomainResult<Note>;

    /// Toggle pin status
    async fn toggle_pin(&self, id: &str) -> DomainResult<Note>;

    /// Archive a note
    async fn archive_note(&self, id: &str) -> DomainResult<()>;

    /// Unarchive a note
    async fn unarchive_note(&self, id: &str) -> DomainResult<()>;

    /// Move note to a different notebook
    async fn move_to_notebook(&self, note_id: &str, notebook_id: Option<String>)
        -> DomainResult<()>;

    /// Get recently updated notes
    async fn get_recent_notes(
        &self,
        limit: i64,
        workspace_id: Option<String>,
    ) -> DomainResult<Vec<Note>>;

    /// Get favorite notes
    async fn get_favorites(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>>;

    /// Get archived notes
    async fn get_archived(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>>;

    /// Get deleted notes (trash)
    async fn get_trash(&self, workspace_id: Option<String>) -> DomainResult<Vec<Note>>;

    /// Get note content from file
    async fn get_note_content(&self, id: &str) -> DomainResult<String>;

    /// Save note content to file
    async fn save_note_content(&self, id: &str, content: &str) -> DomainResult<()>;

    /// Get note by file path
    async fn get_note_by_path(&self, file_path: &str) -> DomainResult<Option<Note>>;

    /// Search notes by query
    async fn search_notes(
        &self,
        query: &str,
        workspace_id: Option<String>,
        limit: Option<i64>,
    ) -> DomainResult<Vec<Note>>;
}
