/// Notebook Domain Entity
///
/// Pure domain object representing a notebook (folder) with its business rules.
/// Independent of database schema and infrastructure concerns.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Notebook entity with full business logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub icon: String,
    pub color: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notebook {
    /// Create a new notebook with validation
    pub fn new(
        name: impl Into<String>,
        workspace_id: Option<String>,
        parent_id: Option<String>,
    ) -> DomainResult<Self> {
        let name = name.into();

        // Business rule: Name cannot be empty
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Notebook name cannot be empty".to_string(),
            ));
        }

        // Business rule: Name length limit
        if name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Notebook name cannot exceed 100 characters".to_string(),
            ));
        }

        // Business rule: No invalid characters in name
        if name.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')) {
            return Err(DomainError::ValidationError(
                "Notebook name contains invalid characters".to_string(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id: nanoid::nanoid!(),
            name: name.trim().to_string(),
            parent_id,
            workspace_id,
            folder_path: None,
            icon: "📁".to_string(),
            color: "#3b82f6".to_string(),
            position: 0,
            created_at: now,
            updated_at: now,
        })
    }

    /// Check if this is a root notebook
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Rename the notebook
    pub fn rename(&mut self, new_name: impl Into<String>) -> DomainResult<()> {
        let new_name = new_name.into();

        if new_name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Notebook name cannot be empty".to_string(),
            ));
        }

        if new_name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Notebook name cannot exceed 100 characters".to_string(),
            ));
        }

        if new_name.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')) {
            return Err(DomainError::ValidationError(
                "Notebook name contains invalid characters".to_string(),
            ));
        }

        self.name = new_name.trim().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Move notebook to a different parent
    pub fn move_to(&mut self, new_parent_id: Option<String>) -> DomainResult<()> {
        // Business rule: Cannot move into itself
        if let Some(ref pid) = new_parent_id {
            if pid == &self.id {
                return Err(DomainError::ValidationError(
                    "Cannot move notebook into itself".to_string(),
                ));
            }
        }

        self.parent_id = new_parent_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the folder path
    pub fn update_folder_path(&mut self, folder_path: Option<String>) {
        self.folder_path = folder_path;
        self.updated_at = Utc::now();
    }

    /// Change the notebook icon
    pub fn change_icon(&mut self, icon: impl Into<String>) -> DomainResult<()> {
        let icon = icon.into();

        if icon.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Icon cannot be empty".to_string(),
            ));
        }

        self.icon = icon.trim().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Change the notebook color
    pub fn change_color(&mut self, color: impl Into<String>) -> DomainResult<()> {
        let color = color.into();

        // Business rule: Color must be valid hex format
        if !color.starts_with('#') || color.len() != 7 {
            return Err(DomainError::ValidationError(
                "Invalid color format. Use hex format (e.g., #3b82f6)".to_string(),
            ));
        }

        if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::ValidationError(
                "Invalid color format. Use hex format (e.g., #3b82f6)".to_string(),
            ));
        }

        self.color = color;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the position for ordering
    pub fn set_position(&mut self, position: i32) -> DomainResult<()> {
        if position < 0 {
            return Err(DomainError::ValidationError(
                "Position cannot be negative".to_string(),
            ));
        }

        self.position = position;
        self.updated_at = Utc::now();
        Ok(())
    }
}

