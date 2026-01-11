/// Domain-specific errors
///
/// These errors represent business rule violations and domain constraints.
/// They should NOT contain infrastructure concerns (DB, HTTP, etc.)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Notebook not found: {0}")]
    NotebookNotFound(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Invalid note title: {0}")]
    InvalidNoteTitle(String),

    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    #[error("Note already exists: {0}")]
    NoteAlreadyExists(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Domain logic error: {0}")]
    DomainLogicError(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
