//! Search Command Handlers

use tauri::State;

use crate::{
    adapters::inbound::app_state::AppState,
    domain::{
        entities::Note,
        ports::{
            inbound::{
                HybridSearchResultItem, HybridSearchWeights, SearchByDateRangeRequest,
                SearchRequest, SearchType, VectorSearchResult,
            },
            outbound::SearchResult,
        },
    },
};

#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<Vec<SearchResult>, String> {
    state
        .search_usecases
        .full_text_search(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hybrid_search(
    state: State<'_, AppState>,
    query: String,
    weights: Option<HybridSearchWeights>,
    limit: Option<i32>,
) -> Result<Vec<HybridSearchResultItem>, String> {
    state
        .search_usecases
        .hybrid_search(&query, weights, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn semantic_search(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<Vec<VectorSearchResult>, String> {
    state
        .search_usecases
        .semantic_search(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_by_date_range(
    state: State<'_, AppState>,
    request: SearchByDateRangeRequest,
) -> Result<Vec<Note>, String> {
    state
        .search_usecases
        .search_by_date_range(request)
        .await
        .map_err(|e| e.to_string())
}
