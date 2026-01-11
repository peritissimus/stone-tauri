/// HexColor Value Object
///
/// Represents a hex color code with validation.
/// Immutable value object compared by value.
use serde::{Deserialize, Serialize};

use crate::domain::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HexColor(String);

impl HexColor {
    /// Create a new HexColor with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        // Business rule: Must start with #
        if !value.starts_with('#') {
            return Err(DomainError::ValidationError(
                "Hex color must start with #".to_string(),
            ));
        }

        // Business rule: Must be 7 characters total (#RRGGBB)
        if value.len() != 7 {
            return Err(DomainError::ValidationError(
                "Hex color must be in format #RRGGBB".to_string(),
            ));
        }

        // Business rule: All characters after # must be valid hex digits
        if !value[1..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::ValidationError(
                "Hex color must contain only hex digits (0-9, A-F, a-f)".to_string(),
            ));
        }

        Ok(Self(value.to_uppercase()))
    }

    /// Get the color as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for HexColor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

