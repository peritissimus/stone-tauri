/// Note Repository Port (OUT Port)
///
/// Defines the interface that adapters must implement to provide note persistence.
/// This is what the application needs from the outside world.
use async_trait::async_trait;

use crate::domain::{entities::Note, errors::DomainResult};

#[derive(Debug, Clone)]
pub struct NoteFindOptions {
    pub workspace_id: Option<String>,
    pub notebook_id: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_deleted: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for NoteFindOptions {
    fn default() -> Self {
        Self {
            workspace_id: None,
            notebook_id: None,
            is_favorite: None,
            is_pinned: None,
            is_archived: None,
            is_deleted: Some(false), // By default, don't show deleted notes
            limit: None,
            offset: None,
        }
    }
}

/// Repository interface for Note persistence
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// Find a note by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Note>>;

    /// Find all notes with optional filters
    async fn find_all(&self, options: NoteFindOptions) -> DomainResult<Vec<Note>>;

    /// Find notes by notebook ID
    async fn find_by_notebook_id(
        &self,
        notebook_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Note>>;

    /// Find notes by workspace ID
    async fn find_by_workspace_id(&self, workspace_id: &str) -> DomainResult<Vec<Note>>;

    /// Find a note by file path
    async fn find_by_file_path(
        &self,
        file_path: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<Option<Note>>;

    /// Save a note (create or update)
    async fn save(&self, note: &Note) -> DomainResult<()>;

    /// Delete a note permanently
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Check if a note exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;

    /// Count notes with optional filters
    async fn count(&self, options: NoteFindOptions) -> DomainResult<i64>;

    /// Find recently updated notes
    async fn find_recently_updated(
        &self,
        limit: i64,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Note>>;

    /// Find favorite notes
    async fn find_favorites(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>>;

    /// Find pinned notes
    async fn find_pinned(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>>;

    /// Find archived notes
    async fn find_archived(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>>;

    /// Find deleted notes (trash)
    async fn find_deleted(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>>;

    /// Search notes by title
    async fn search_by_title(
        &self,
        query: &str,
        workspace_id: Option<&str>,
        limit: Option<i64>,
    ) -> DomainResult<Vec<Note>>;
}
