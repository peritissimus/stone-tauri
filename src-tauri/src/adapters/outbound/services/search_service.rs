//! Search Service Implementation
//!
//! Stub implementation of the SearchEngine port.
//! Full implementation would use Tantivy for full-text search.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    adapters::outbound::persistence::DbPool,
    domain::{
        entities::Note,
        errors::{DomainError, DomainResult},
        ports::outbound::{
            DateRangeOptions, HybridSearchOptions, SearchEngine, SearchOptions, SearchResult,
            SemanticSearchResult, TagSearchOptions,
        },
    },
};

/// Stub search service implementation
/// TODO: Implement full-text search with Tantivy
pub struct StubSearchService {
    _pool: Arc<DbPool>,
}

impl StubSearchService {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl SearchEngine for StubSearchService {
    async fn search_full_text(
        &self,
        _query: &str,
        _options: Option<SearchOptions>,
    ) -> DomainResult<Vec<SearchResult>> {
        // TODO: Implement full-text search using Tantivy
        // For now, return empty results
        Ok(Vec::new())
    }

    async fn search_semantic(
        &self,
        _query: &str,
        _options: Option<SearchOptions>,
    ) -> DomainResult<Vec<SemanticSearchResult>> {
        // TODO: Implement semantic search using embeddings
        Ok(Vec::new())
    }

    async fn search_hybrid(
        &self,
        _query: &str,
        _options: Option<HybridSearchOptions>,
    ) -> DomainResult<Vec<SearchResult>> {
        // TODO: Implement hybrid search combining FTS and semantic
        Ok(Vec::new())
    }

    async fn search_by_tags(
        &self,
        _tag_ids: Vec<String>,
        _options: Option<TagSearchOptions>,
    ) -> DomainResult<Vec<Note>> {
        // TODO: Implement tag-based search
        Ok(Vec::new())
    }

    async fn search_by_date_range(&self, _options: DateRangeOptions) -> DomainResult<Vec<Note>> {
        // TODO: Implement date range search
        Ok(Vec::new())
    }

    async fn index_note(&self, _note_id: &str, _title: &str, _content: &str) -> DomainResult<()> {
        // TODO: Index note in Tantivy
        Ok(())
    }

    async fn remove_from_index(&self, _note_id: &str) -> DomainResult<()> {
        // TODO: Remove note from Tantivy index
        Ok(())
    }

    async fn rebuild_index(&self) -> DomainResult<()> {
        // TODO: Rebuild entire Tantivy index
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    #[tokio::test]
    async fn test_stub_search_service() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let service = StubSearchService::new(pool);

        // Test that stub methods return empty results
        let results = service
            .search_full_text("test", None)
            .await
            .unwrap();
        assert!(results.is_empty());
    }
}
