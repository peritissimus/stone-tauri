use crate::domain::{entities::Note, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMatchType {
    Title,
    Content,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHighlights {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub note: Note,
    pub relevance: f32,
    pub match_type: SearchMatchType,
    pub highlights: Option<SearchHighlights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResult {
    pub note_id: String,
    pub title: String,
    pub similarity: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub notebook_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub workspace_id: Option<String>,
    pub exclude_deleted: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateRangeField {
    Created,
    Updated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRangeOptions {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub workspace_id: Option<String>,
    pub field: Option<DateRangeField>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchWeights {
    pub fts: f32,
    pub semantic: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridSearchOptions {
    #[serde(flatten)]
    pub base: SearchOptions,
    pub weights: Option<SearchWeights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSearchOptions {
    #[serde(flatten)]
    pub base: SearchOptions,
    pub match_all: Option<bool>,
}

/// Search Engine Port (Outbound)
///
/// Defines the contract for search operations.
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Full-text search across notes
    async fn search_full_text(
        &self,
        query: &str,
        options: Option<SearchOptions>,
    ) -> DomainResult<Vec<SearchResult>>;

    /// Semantic search using embeddings
    async fn search_semantic(
        &self,
        query: &str,
        options: Option<SearchOptions>,
    ) -> DomainResult<Vec<SemanticSearchResult>>;

    /// Hybrid search (FTS + semantic)
    async fn search_hybrid(
        &self,
        query: &str,
        options: Option<HybridSearchOptions>,
    ) -> DomainResult<Vec<SearchResult>>;

    /// Search by tags
    async fn search_by_tags(
        &self,
        tag_ids: Vec<String>,
        options: Option<TagSearchOptions>,
    ) -> DomainResult<Vec<Note>>;

    /// Search by date range
    async fn search_by_date_range(&self, options: DateRangeOptions) -> DomainResult<Vec<Note>>;

    /// Update search index for a note
    async fn index_note(&self, note_id: &str, title: &str, content: &str) -> DomainResult<()>;

    /// Remove note from search index
    async fn remove_from_index(&self, note_id: &str) -> DomainResult<()>;

    /// Rebuild search index
    async fn rebuild_index(&self) -> DomainResult<()>;
}
