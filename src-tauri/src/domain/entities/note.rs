/// Note Entity
///
/// Core business object representing a note in the system.
/// Contains identity, business behavior, and validation rules.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Note entity with full business logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub notebook_id: Option<String>,
    pub workspace_id: Option<String>,
    pub file_path: Option<String>,
    pub is_favorite: bool,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    /// Create a new note with validation
    pub fn new(title: impl Into<String>, workspace_id: Option<String>) -> DomainResult<Self> {
        let title = title.into();

        // Business rule: Title cannot be empty
        if title.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Note title cannot be empty".to_string()
            ));
        }

        // Business rule: Title length limit
        if title.len() > 500 {
            return Err(DomainError::ValidationError(
                "Note title cannot exceed 500 characters".to_string()
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id: nanoid::nanoid!(),
            title,
            notebook_id: None,
            workspace_id,
            file_path: None,
            is_favorite: false,
            is_pinned: false,
            is_archived: false,
            is_deleted: false,
            deleted_at: None,
            embedding: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the note title with validation
    pub fn update_title(&mut self, new_title: impl Into<String>) -> DomainResult<()> {
        let new_title = new_title.into();

        if new_title.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Note title cannot be empty".to_string()
            ));
        }

        if new_title.len() > 500 {
            return Err(DomainError::ValidationError(
                "Note title cannot exceed 500 characters".to_string()
            ));
        }

        self.title = new_title;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark note as favorite
    pub fn mark_favorite(&mut self) {
        self.is_favorite = true;
        self.updated_at = Utc::now();
    }

    /// Unmark note as favorite
    pub fn unmark_favorite(&mut self) {
        self.is_favorite = false;
        self.updated_at = Utc::now();
    }

    /// Pin the note
    pub fn pin(&mut self) {
        self.is_pinned = true;
        self.updated_at = Utc::now();
    }

    /// Unpin the note
    pub fn unpin(&mut self) {
        self.is_pinned = false;
        self.updated_at = Utc::now();
    }

    /// Archive the note
    pub fn archive(&mut self) {
        self.is_archived = true;
        self.updated_at = Utc::now();
    }

    /// Unarchive the note
    pub fn unarchive(&mut self) {
        self.is_archived = false;
        self.updated_at = Utc::now();
    }

    /// Soft delete the note
    pub fn delete(&mut self) {
        self.is_deleted = true;
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Restore a deleted note
    pub fn restore(&mut self) {
        self.is_deleted = false;
        self.deleted_at = None;
        self.updated_at = Utc::now();
    }

    /// Move note to a notebook
    pub fn move_to_notebook(&mut self, notebook_id: Option<String>) {
        self.notebook_id = notebook_id;
        self.updated_at = Utc::now();
    }

    /// Set the file path
    pub fn set_file_path(&mut self, path: Option<String>) -> DomainResult<()> {
        // Business rule: File path validation
        if let Some(ref p) = path {
            if p.is_empty() {
                return Err(DomainError::ValidationError(
                    "File path cannot be empty".to_string()
                ));
            }
        }

        self.file_path = path;
        self.updated_at = Utc::now();
        Ok(())
    }
}

