/// FilePath Value Object
///
/// Represents a file system path with validation.
/// Immutable value object compared by value.
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FilePath(String);

impl FilePath {
    /// Create a new FilePath with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "FilePath cannot be empty".to_string(),
            ));
        }

        Ok(Self(value.trim().to_string()))
    }

    /// Get the path as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for FilePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

