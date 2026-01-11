use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_metadata: Option<bool>,
    pub include_frontmatter: Option<bool>,
    pub theme: Option<ExportTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
}

/// Export Use Cases Port (Inbound)
///
/// Defines the contract for note export operations.
#[async_trait]
pub trait ExportUseCases: Send + Sync {
    /// Export note as HTML
    async fn export_html(
        &self,
        note_id: &str,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult>;

    /// Export note as PDF
    async fn export_pdf(
        &self,
        note_id: &str,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult>;

    /// Export note as Markdown
    async fn export_markdown(
        &self,
        note_id: &str,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult>;
}
