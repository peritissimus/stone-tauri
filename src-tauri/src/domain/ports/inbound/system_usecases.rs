use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectFolderOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
}

/// System Use Cases Port (Inbound)
///
/// Defines the contract for system-level operations.
#[async_trait]
pub trait SystemUseCases: Send + Sync {
    /// Get system fonts
    async fn get_fonts(&self) -> DomainResult<Vec<String>>;

    /// Show folder selection dialog
    async fn select_folder(
        &self,
        options: Option<SelectFolderOptions>,
    ) -> DomainResult<Option<String>>;

    /// Validate a file path
    async fn validate_path(&self, path: &str) -> DomainResult<bool>;

    /// Open a path in the file explorer
    fn open_in_folder(&self, path: &str) -> DomainResult<()>;

    /// Open a URL in the default browser
    async fn open_external(&self, url: &str) -> DomainResult<()>;
}
