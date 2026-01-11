/// NoteLink Domain Entity
///
/// Represents a wiki-style link between two notes.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// NoteLink entity representing a connection between notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLink {
    pub source_note_id: String,
    pub target_note_id: String,
    pub created_at: DateTime<Utc>,
}

impl NoteLink {
    /// Create a new note link with validation
    pub fn new(
        source_note_id: impl Into<String>,
        target_note_id: impl Into<String>,
    ) -> DomainResult<Self> {
        let source = source_note_id.into();
        let target = target_note_id.into();

        // Business rule: Both IDs required
        if source.trim().is_empty() || target.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Both source and target note IDs are required".to_string(),
            ));
        }

        // Business rule: No self-referencing links
        if source == target {
            return Err(DomainError::ValidationError(
                "Cannot create a self-referencing link".to_string(),
            ));
        }

        Ok(Self {
            source_note_id: source,
            target_note_id: target,
            created_at: Utc::now(),
        })
    }

    /// Check if this is a self-link
    pub fn is_self_link(&self) -> bool {
        self.source_note_id == self.target_note_id
    }

    /// Check if link involves a specific note
    pub fn involves_note(&self, note_id: &str) -> bool {
        self.source_note_id == note_id || self.target_note_id == note_id
    }

    /// Check if link is from a specific note
    pub fn is_link_from(&self, note_id: &str) -> bool {
        self.source_note_id == note_id
    }

    /// Check if link is to a specific note
    pub fn is_link_to(&self, note_id: &str) -> bool {
        self.target_note_id == note_id
    }

    /// Get composite key for uniqueness
    pub fn key(&self) -> String {
        format!("{}:{}", self.source_note_id, self.target_note_id)
    }

    /// Make a composite key from IDs
    pub fn make_key(source_id: &str, target_id: &str) -> String {
        format!("{}:{}", source_id, target_id)
    }
}

/// Link count for a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCount {
    pub incoming: i32,
    pub outgoing: i32,
}

impl LinkCount {
    pub fn new() -> Self {
        Self {
            incoming: 0,
            outgoing: 0,
        }
    }

    pub fn total(&self) -> i32 {
        self.incoming + self.outgoing
    }
}

impl Default for LinkCount {
    fn default() -> Self {
        Self::new()
    }
}

