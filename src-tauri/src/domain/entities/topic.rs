/// Topic Domain Entity
///
/// Represents an ML-based topic for note classification.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Topic entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub is_predefined: bool,
    pub centroid: Option<Vec<u8>>,
    pub note_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Topic {
    /// Create a new topic with validation
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        let name = Self::normalize_name(&name.into())?;

        let now = Utc::now();

        Ok(Self {
            id: nanoid::nanoid!(),
            name,
            description: None,
            color: Self::default_color(),
            is_predefined: false,
            centroid: None,
            note_count: 0,
            created_at: now,
            updated_at: now,
        })
    }

    /// Normalize topic name
    fn normalize_name(name: &str) -> DomainResult<String> {
        let normalized = name.trim().to_string();

        if normalized.is_empty() {
            return Err(DomainError::ValidationError(
                "Topic name cannot be empty".to_string(),
            ));
        }

        Ok(normalized)
    }

    /// Get default color
    pub fn default_color() -> String {
        "#6366f1".to_string()
    }

    /// Validate color format
    fn is_valid_color(color: &str) -> bool {
        if !color.starts_with('#') || color.len() != 7 {
            return false;
        }
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Rename the topic
    pub fn rename(&mut self, name: impl Into<String>) -> DomainResult<()> {
        let normalized = Self::normalize_name(&name.into())?;
        self.name = normalized;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update description
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        self.updated_at = Utc::now();
    }

    /// Change color
    pub fn change_color(&mut self, color: impl Into<String>) -> DomainResult<()> {
        let color = color.into();

        if !Self::is_valid_color(&color) {
            return Err(DomainError::ValidationError(
                "Invalid color format".to_string(),
            ));
        }

        self.color = color;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update ML centroid
    pub fn update_centroid(&mut self, centroid: Vec<u8>) {
        self.centroid = Some(centroid);
        self.updated_at = Utc::now();
    }

    /// Increment note count
    pub fn increment_note_count(&mut self) {
        self.note_count += 1;
        self.updated_at = Utc::now();
    }

    /// Decrement note count
    pub fn decrement_note_count(&mut self) {
        if self.note_count > 0 {
            self.note_count -= 1;
            self.updated_at = Utc::now();
        }
    }

    /// Set note count
    pub fn set_note_count(&mut self, count: i32) {
        self.note_count = count.max(0);
        self.updated_at = Utc::now();
    }

    /// Check if topic can be deleted
    pub fn can_delete(&self) -> bool {
        !self.is_predefined
    }
}

/// Topic summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub id: String,
    pub name: String,
    pub color: String,
    pub note_count: i32,
}

impl From<&Topic> for TopicSummary {
    fn from(topic: &Topic) -> Self {
        Self {
            id: topic.id.clone(),
            name: topic.name.clone(),
            color: topic.color.clone(),
            note_count: topic.note_count,
        }
    }
}

