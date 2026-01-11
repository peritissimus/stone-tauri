/// ID Value Objects
///
/// Represent unique identifiers for domain entities.
/// These are immutable value objects compared by value.
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Note ID value object
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NoteId(String);

impl NoteId {
    /// Create from an existing ID string
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "NoteId cannot be empty".to_string(),
            ));
        }

        Ok(Self(value.trim().to_string()))
    }

    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for NoteId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Notebook ID value object
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NotebookId(String);

impl NotebookId {
    /// Create from an existing ID string
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "NotebookId cannot be empty".to_string(),
            ));
        }

        Ok(Self(value.trim().to_string()))
    }

    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for NotebookId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NotebookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Workspace ID value object
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    /// Create from an existing ID string
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "WorkspaceId cannot be empty".to_string(),
            ));
        }

        Ok(Self(value.trim().to_string()))
    }

    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for WorkspaceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

