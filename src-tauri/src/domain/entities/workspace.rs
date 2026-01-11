/// Workspace Domain Entity
///
/// Pure domain object representing a workspace with its business rules.
/// A workspace represents a root folder containing notes and notebooks.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Workspace entity with full business logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder_path: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

impl Workspace {
    /// Create a new workspace with validation
    pub fn new(name: impl Into<String>, folder_path: impl Into<String>) -> DomainResult<Self> {
        let name = name.into();
        let folder_path = folder_path.into();

        // Business rule: Name cannot be empty
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Workspace name cannot be empty".to_string(),
            ));
        }

        // Business rule: Name length limit
        if name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Workspace name cannot exceed 100 characters".to_string(),
            ));
        }

        // Business rule: Folder path cannot be empty
        if folder_path.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Workspace folder path cannot be empty".to_string(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id: nanoid::nanoid!(),
            name: name.trim().to_string(),
            folder_path: folder_path.trim().to_string(),
            is_active: false,
            created_at: now,
            last_accessed_at: now,
        })
    }

    /// Rename the workspace
    pub fn rename(&mut self, new_name: impl Into<String>) -> DomainResult<()> {
        let new_name = new_name.into();

        if new_name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Workspace name cannot be empty".to_string(),
            ));
        }

        if new_name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Workspace name cannot exceed 100 characters".to_string(),
            ));
        }

        self.name = new_name.trim().to_string();
        Ok(())
    }

    /// Activate this workspace
    pub fn activate(&mut self) {
        self.is_active = true;
        self.last_accessed_at = Utc::now();
    }

    /// Deactivate this workspace
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Record access to this workspace
    pub fn record_access(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Update the folder path (e.g., when workspace is moved)
    pub fn update_folder_path(&mut self, new_path: impl Into<String>) -> DomainResult<()> {
        let new_path = new_path.into();

        if new_path.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Folder path cannot be empty".to_string(),
            ));
        }

        self.folder_path = new_path.trim().to_string();
        Ok(())
    }

    /// Get the relative path within this workspace for a given absolute path
    pub fn get_relative_path(&self, absolute_path: &str) -> Option<String> {
        if !absolute_path.starts_with(&self.folder_path) {
            return None;
        }

        let relative = absolute_path[self.folder_path.len()..]
            .trim_start_matches('/')
            .to_string();
        Some(relative)
    }

    /// Get the absolute path for a given relative path within this workspace
    pub fn get_absolute_path(&self, relative_path: &str) -> String {
        let clean_relative = relative_path.trim_start_matches('/');
        format!("{}/{}", self.folder_path, clean_relative)
    }
}

