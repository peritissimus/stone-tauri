use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PdfFormat {
    A4,
    Letter,
    Legal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMargin {
    pub top: Option<String>,
    pub right: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfOptions {
    pub format: Option<PdfFormat>,
    pub margin: Option<PdfMargin>,
    pub landscape: Option<bool>,
    pub print_background: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HtmlTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HtmlOptions {
    pub title: Option<String>,
    pub theme: Option<HtmlTheme>,
    pub include_styles: Option<bool>,
    pub custom_css: Option<String>,
}

/// Export Service Port (Outbound)
///
/// Defines the contract for rendering exports to various formats.
#[async_trait]
pub trait ExportService: Send + Sync {
    /// Render HTML content to PDF
    async fn render_to_pdf(&self, html: &str, options: Option<PdfOptions>) -> DomainResult<Vec<u8>>;

    /// Generate complete HTML document from content
    fn generate_html_document(&self, content: &str, options: Option<HtmlOptions>) -> DomainResult<String>;

    /// Check if PDF export is available
    fn is_pdf_available(&self) -> bool;
}
