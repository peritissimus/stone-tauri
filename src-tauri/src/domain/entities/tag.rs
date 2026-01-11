/// Tag Domain Entity
///
/// Pure domain object representing a tag with its business rules.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Tag entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    /// Create a new tag with validation
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        let name = name.into();
        let normalized_name = Self::normalize_name(&name)?;

        let now = Utc::now();

        Ok(Self {
            id: nanoid::nanoid!(),
            name: normalized_name,
            color: "#6b7280".to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Normalize tag name (lowercase, trim, remove special chars, replace spaces with hyphens)
    pub fn normalize_name(name: &str) -> DomainResult<String> {
        let normalized = name
            .to_lowercase()
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-");

        if normalized.is_empty() {
            return Err(DomainError::ValidationError(
                "Tag name cannot be empty".to_string(),
            ));
        }

        Ok(normalized)
    }

    /// Rename the tag
    pub fn rename(&mut self, new_name: impl Into<String>) -> DomainResult<()> {
        let normalized_name = Self::normalize_name(&new_name.into())?;

        if normalized_name.len() > 50 {
            return Err(DomainError::ValidationError(
                "Tag name cannot exceed 50 characters".to_string(),
            ));
        }

        self.name = normalized_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Change the tag color
    pub fn change_color(&mut self, color: impl Into<String>) -> DomainResult<()> {
        let color = color.into();

        // Validate hex color format
        if !color.starts_with('#') || color.len() != 7 {
            return Err(DomainError::ValidationError(
                "Invalid color format. Use hex format (e.g., #6b7280)".to_string(),
            ));
        }

        if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::ValidationError(
                "Invalid color format. Use hex format (e.g., #6b7280)".to_string(),
            ));
        }

        self.color = color;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if this tag matches a search term
    pub fn matches(&self, search_term: &str) -> bool {
        let normalized_search = search_term.to_lowercase().trim().to_string();
        self.name.contains(&normalized_search)
    }
}

