use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownMetadata {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMarkdown {
    pub content: String,
    pub metadata: MarkdownMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownLink {
    pub text: String,
    pub href: String,
}

/// Markdown Processor Port (Outbound)
///
/// Defines the contract for markdown conversion operations.
#[async_trait]
pub trait MarkdownProcessor: Send + Sync {
    /// Convert HTML to Markdown
    fn html_to_markdown(&self, html: &str) -> DomainResult<String>;

    /// Convert Markdown to HTML
    async fn markdown_to_html(&self, markdown: &str) -> DomainResult<String>;

    /// Parse frontmatter from markdown content
    fn parse_frontmatter(&self, markdown: &str) -> DomainResult<ParsedMarkdown>;

    /// Add/update frontmatter in markdown content
    fn update_frontmatter(
        &self,
        markdown: &str,
        metadata: &MarkdownMetadata,
    ) -> DomainResult<String>;

    /// Extract title from markdown content (first H1 or frontmatter title)
    fn extract_title(&self, markdown: &str) -> DomainResult<Option<String>>;

    /// Extract plain text from markdown (strip formatting)
    fn extract_plain_text(&self, markdown: &str) -> DomainResult<String>;

    /// Extract links from markdown content
    fn extract_links(&self, markdown: &str) -> DomainResult<Vec<MarkdownLink>>;

    /// Extract wiki-style links [[link]] from markdown
    fn extract_wiki_links(&self, markdown: &str) -> DomainResult<Vec<String>>;

    /// Convert HTML to plain text (strip all formatting)
    fn html_to_plain_text(&self, html: &str) -> DomainResult<String>;
}
