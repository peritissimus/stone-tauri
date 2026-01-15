/// Search Use Cases Implementation
///
/// Application layer implementations for search operations.
/// Orchestrates search engines, embedding services, and repositories.
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{
    entities::Note,
    errors::DomainResult,
    ports::{
        inbound::{
            HybridSearchResultItem, HybridSearchWeights, SearchByDateRangeRequest, SearchRequest,
            SearchType, SearchUseCases, VectorSearchResult,
        },
        outbound::{
            DateRangeField, DateRangeOptions, EmbeddingService, NoteRepository, SearchEngine,
            SearchOptions, SearchResult, TagSearchOptions,
        },
    },
};

/// Implementation of all Search use cases
pub struct SearchUseCasesImpl {
    search_engine: Arc<dyn SearchEngine>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl SearchUseCasesImpl {
    pub fn new(
        _note_repository: Arc<dyn NoteRepository>,
        search_engine: Arc<dyn SearchEngine>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            search_engine,
            embedding_service,
        }
    }
}

#[async_trait]
impl SearchUseCases for SearchUseCasesImpl {
    /// Full-text search across notes
    async fn full_text_search(&self, request: SearchRequest) -> DomainResult<Vec<SearchResult>> {
        let options = SearchOptions {
            limit: request.limit,
            offset: request.offset,
            notebook_id: request.notebook_id.clone(),
            tag_ids: request.tag_ids.clone(),
            workspace_id: request.workspace_id.clone(),
            exclude_deleted: Some(true),
        };

        let results = self
            .search_engine
            .search_full_text(&request.query, Some(options))
            .await?;

        Ok(results)
    }

    /// Semantic search using embeddings
    async fn semantic_search(
        &self,
        request: SearchRequest,
    ) -> DomainResult<Vec<VectorSearchResult>> {
        let limit = request.limit.or(Some(10));

        // Use embedding service's semantic search
        let results = self
            .embedding_service
            .semantic_search(&request.query, limit)
            .await?;

        // Convert SimilarNote to VectorSearchResult
        let vector_results = results
            .into_iter()
            .map(|similar_note| VectorSearchResult {
                note_id: similar_note.note_id,
                title: similar_note.title,
                distance: similar_note.distance,
            })
            .collect();

        Ok(vector_results)
    }

    /// Find similar notes
    async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<VectorSearchResult>> {
        let limit = limit.or(Some(5));

        // Use embedding service's find similar notes
        let results = self
            .embedding_service
            .find_similar_notes(note_id, limit)
            .await?;

        // Convert SimilarNote to VectorSearchResult
        let vector_results = results
            .into_iter()
            .map(|similar_note| VectorSearchResult {
                note_id: similar_note.note_id,
                title: similar_note.title,
                distance: similar_note.distance,
            })
            .collect();

        Ok(vector_results)
    }

    /// Rebuild search index
    async fn rebuild_index(&self) -> DomainResult<()> {
        self.search_engine.rebuild_index().await
    }

    /// Hybrid search (FTS + semantic)
    async fn hybrid_search(
        &self,
        query: &str,
        _weights: Option<HybridSearchWeights>,
        limit: Option<i32>,
    ) -> DomainResult<Vec<HybridSearchResultItem>> {
        let limit = limit.or(Some(50));

        // For now, use full-text search as primary
        // (can be enhanced with semantic search later)
        let options = SearchOptions {
            limit,
            offset: None,
            notebook_id: None,
            tag_ids: None,
            workspace_id: None,
            exclude_deleted: Some(true),
        };

        let fts_results = self
            .search_engine
            .search_full_text(query, Some(options))
            .await?;

        // Convert SearchResult to HybridSearchResultItem
        let hybrid_results = fts_results
            .into_iter()
            .map(|search_result| HybridSearchResultItem {
                note: search_result.note,
                score: 1.0, // Fixed score for now
                search_type: SearchType::Fts,
            })
            .collect();

        Ok(hybrid_results)
    }

    /// Search by tags
    async fn search_by_tags(
        &self,
        tag_ids: Vec<String>,
        match_all: Option<bool>,
        limit: Option<i32>,
    ) -> DomainResult<Vec<Note>> {
        let options = TagSearchOptions {
            base: SearchOptions {
                limit,
                offset: None,
                notebook_id: None,
                tag_ids: None,
                workspace_id: None,
                exclude_deleted: Some(true),
            },
            match_all,
        };

        let results = self
            .search_engine
            .search_by_tags(tag_ids, Some(options))
            .await?;

        Ok(results)
    }

    /// Search by date range
    async fn search_by_date_range(
        &self,
        request: SearchByDateRangeRequest,
    ) -> DomainResult<Vec<Note>> {
        // Convert i64 timestamps to DateTime
        let start_date = DateTime::from_timestamp_millis(request.start_date)
            .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC);

        let end_date = DateTime::from_timestamp_millis(request.end_date)
            .unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);

        // Parse field (default to "created")
        let field = request
            .field
            .as_deref()
            .and_then(|f| match f {
                "updated" => Some(DateRangeField::Updated),
                _ => Some(DateRangeField::Created),
            })
            .unwrap_or(DateRangeField::Created);

        let options = DateRangeOptions {
            start_date,
            end_date,
            workspace_id: request.workspace_id.clone(),
            field: Some(field),
            limit: request.limit,
        };

        let results = self.search_engine.search_by_date_range(options).await?;

        Ok(results)
    }
}
