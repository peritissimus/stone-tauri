/// Version Domain Entity
///
/// Represents a version snapshot of a note's content.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Version entity representing a snapshot of note content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub version_number: i32,
    pub created_at: DateTime<Utc>,
}

impl Version {
    /// Create a new version
    pub fn new(
        note_id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        version_number: i32,
    ) -> DomainResult<Self> {
        let note_id = note_id.into();

        if note_id.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Note ID is required".to_string(),
            ));
        }

        if version_number < 1 {
            return Err(DomainError::ValidationError(
                "Version number must be positive".to_string(),
            ));
        }

        Ok(Self {
            id: nanoid::nanoid!(),
            note_id,
            title: title.into(),
            content: content.into(),
            version_number,
            created_at: Utc::now(),
        })
    }

    /// Get content length
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Get formatted version string
    pub fn formatted_version(&self) -> String {
        format!("v{}", self.version_number)
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &Version) -> bool {
        self.version_number > other.version_number
    }

    /// Get summary without full content
    pub fn summary(&self) -> VersionSummary {
        VersionSummary {
            id: self.id.clone(),
            version_number: self.version_number,
            title: self.title.clone(),
            created_at: self.created_at,
            content_length: self.content_length(),
        }
    }
}

/// Version summary without full content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub id: String,
    pub version_number: i32,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub content_length: usize,
}

