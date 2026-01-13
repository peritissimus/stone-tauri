//! Search Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Note,
        ports::{
            inbound::{
                HybridSearchResultItem, HybridSearchWeights, SearchByDateRangeRequest,
                SearchRequest, VectorSearchResult,
            },
            outbound::SearchResult,
        },
    },
};

/// Unified response wrapper for search results
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNotesResponse {
    pub results: Vec<SearchResult>,
}

/// Response wrapper for hybrid search results
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridSearchResponse {
    pub results: Vec<HybridSearchResultItem>,
}

/// Response wrapper for semantic search results
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResponse {
    pub results: Vec<VectorSearchResult>,
}

/// Response wrapper for date range search results
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchByDateRangeResponse {
    pub results: Vec<Note>,
}

#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<SearchNotesResponse, String> {
    let results = state
        .search_usecases
        .full_text_search(request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SearchNotesResponse { results })
}

#[tauri::command]
pub async fn hybrid_search(
    state: State<'_, AppState>,
    query: String,
    weights: Option<HybridSearchWeights>,
    limit: Option<i32>,
) -> Result<HybridSearchResponse, String> {
    let results = state
        .search_usecases
        .hybrid_search(&query, weights, limit)
        .await
        .map_err(|e| e.to_string())?;

    Ok(HybridSearchResponse { results })
}

#[tauri::command]
pub async fn semantic_search(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<SemanticSearchResponse, String> {
    let results = state
        .search_usecases
        .semantic_search(request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SemanticSearchResponse { results })
}

#[tauri::command]
pub async fn search_by_date_range(
    state: State<'_, AppState>,
    request: SearchByDateRangeRequest,
) -> Result<SearchByDateRangeResponse, String> {
    let results = state
        .search_usecases
        .search_by_date_range(request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SearchByDateRangeResponse { results })
}
