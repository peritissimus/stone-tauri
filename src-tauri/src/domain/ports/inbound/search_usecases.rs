use crate::domain::{entities::Note, errors::DomainResult, ports::outbound::SearchResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorSearchResult {
    pub note_id: String,
    pub title: String,
    pub distance: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub workspace_id: Option<String>,
    pub notebook_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridSearchWeights {
    pub fts: f32,
    pub semantic: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    Fts,
    Semantic,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridSearchResultItem {
    pub note: Note,
    pub score: f32,
    pub search_type: SearchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchByDateRangeRequest {
    pub start_date: i64,
    pub end_date: i64,
    pub workspace_id: Option<String>,
    pub field: Option<String>,
    pub limit: Option<i32>,
}

/// Search Use Cases Port (Inbound)
///
/// Defines what the application CAN DO for search.
#[async_trait]
pub trait SearchUseCases: Send + Sync {
    /// Full-text search across notes
    async fn full_text_search(&self, request: SearchRequest) -> DomainResult<Vec<SearchResult>>;

    /// Semantic search using embeddings
    async fn semantic_search(&self, request: SearchRequest)
        -> DomainResult<Vec<VectorSearchResult>>;

    /// Find similar notes
    async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<VectorSearchResult>>;

    /// Rebuild search index
    async fn rebuild_index(&self) -> DomainResult<()>;

    /// Hybrid search (FTS + semantic)
    async fn hybrid_search(
        &self,
        query: &str,
        weights: Option<HybridSearchWeights>,
        limit: Option<i32>,
    ) -> DomainResult<Vec<HybridSearchResultItem>>;

    /// Search by tags
    async fn search_by_tags(
        &self,
        tag_ids: Vec<String>,
        match_all: Option<bool>,
        limit: Option<i32>,
    ) -> DomainResult<Vec<Note>>;

    /// Search by date range
    async fn search_by_date_range(
        &self,
        request: SearchByDateRangeRequest,
    ) -> DomainResult<Vec<Note>>;
}
