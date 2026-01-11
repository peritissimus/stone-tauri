use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub note_id: String,
    pub embedding: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub topic_id: String,
    pub topic_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarNote {
    pub note_id: String,
    pub title: String,
    pub similarity: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatus {
    pub ready: bool,
    pub total_notes: i32,
    pub embedded_notes: i32,
    pub pending_notes: i32,
}

/// Embedding Service Port (Outbound)
///
/// Defines the contract for ML embedding operations.
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Initialize the embedding service
    async fn initialize(&self) -> DomainResult<()>;

    /// Check if service is ready
    fn is_ready(&self) -> bool;

    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> DomainResult<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn generate_embeddings(&self, texts: Vec<String>) -> DomainResult<Vec<Vec<f32>>>;

    /// Classify a note into topics
    async fn classify_note(&self, note_id: &str) -> DomainResult<Vec<ClassificationResult>>;

    /// Find similar notes
    async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNote>>;

    /// Semantic search
    async fn semantic_search(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNote>>;

    /// Store embedding for a note
    async fn store_embedding(&self, note_id: &str, embedding: Vec<f32>) -> DomainResult<()>;

    /// Get embedding for a note
    async fn get_embedding(&self, note_id: &str) -> DomainResult<Option<Vec<f32>>>;

    /// Delete embedding for a note
    async fn delete_embedding(&self, note_id: &str) -> DomainResult<()>;

    /// Recompute topic centroids
    async fn recompute_centroids(&self) -> DomainResult<()>;

    /// Get embedding status
    async fn get_status(&self) -> DomainResult<EmbeddingStatus>;
}
