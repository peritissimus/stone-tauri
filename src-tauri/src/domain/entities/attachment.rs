/// Attachment Domain Entity
///
/// Represents a file attachment linked to a note.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

/// Attachment entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

impl Attachment {
    /// Create a new attachment with validation
    pub fn new(
        note_id: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        size: i64,
        path: impl Into<String>,
    ) -> DomainResult<Self> {
        let filename = filename.into();
        Self::validate_filename(&filename)?;

        Ok(Self {
            id: nanoid::nanoid!(),
            note_id: note_id.into(),
            filename: filename.trim().to_string(),
            mime_type: mime_type.into(),
            size,
            path: path.into(),
            created_at: Utc::now(),
        })
    }

    /// Validate filename for security
    fn validate_filename(filename: &str) -> DomainResult<()> {
        if filename.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Filename cannot be empty".to_string(),
            ));
        }

        // Check for path traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(DomainError::ValidationError(
                "Invalid filename: path traversal detected".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if attachment is an image
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Check if attachment is a PDF
    pub fn is_pdf(&self) -> bool {
        self.mime_type == "application/pdf"
    }

    /// Get file extension
    pub fn extension(&self) -> String {
        self.filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
    }

    /// Get formatted file size
    pub fn formatted_size(&self) -> String {
        const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index > 0 {
            format!("{:.1} {}", size, UNITS[unit_index])
        } else {
            format!("{:.0} {}", size, UNITS[unit_index])
        }
    }
}

