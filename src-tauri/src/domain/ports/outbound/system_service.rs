use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilePickerOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Option<Vec<FileFilter>>,
    pub multi_select: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderPickerOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub button_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilePickerResult {
    Single(String),
    Multiple(Vec<String>),
}

/// System Service Port (Outbound)
///
/// Defines the contract for OS-level operations.
#[async_trait]
pub trait SystemService: Send + Sync {
    /// Get available system fonts
    async fn get_fonts(&self) -> DomainResult<Vec<String>>;

    /// Show folder selection dialog
    async fn select_folder(
        &self,
        options: Option<FolderPickerOptions>,
    ) -> DomainResult<Option<String>>;

    /// Show file selection dialog
    async fn select_file(
        &self,
        options: Option<FilePickerOptions>,
    ) -> DomainResult<Option<FilePickerResult>>;

    /// Show save file dialog
    async fn select_save_location(
        &self,
        options: Option<FilePickerOptions>,
    ) -> DomainResult<Option<String>>;

    /// Validate a file path exists and is accessible
    async fn validate_path(&self, path: &str) -> DomainResult<bool>;

    /// Open a path in the system file explorer
    fn show_in_folder(&self, path: &str) -> DomainResult<()>;

    /// Open a URL in the default browser
    async fn open_external(&self, url: &str) -> DomainResult<()>;
}
