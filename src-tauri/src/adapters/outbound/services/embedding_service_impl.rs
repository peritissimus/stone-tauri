//! Embedding Service Implementation
//!
//! Implementation of the EmbeddingService port using FastEmbed.
//! Uses the all-MiniLM-L6-v2 model for generating text embeddings.

use std::sync::{Arc, Mutex};
use std::cmp::Ordering;

use async_trait::async_trait;
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use tokio::task;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{
        ClassificationResult, EmbeddingService, EmbeddingStatus, SimilarNote, NoteRepository,
    },
};

/// FastEmbed embedding service implementation
pub struct FastEmbedService {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    note_repository: Arc<dyn NoteRepository>,
    ready: Arc<Mutex<bool>>,
}

impl FastEmbedService {
    pub fn new(note_repository: Arc<dyn NoteRepository>) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            note_repository,
            ready: Arc::new(Mutex::new(false)),
        }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
        let norm_b: f32 = v2.iter().map(|b| b * b).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl EmbeddingService for FastEmbedService {
    async fn initialize(&self) -> DomainResult<()> {
        let model_ref = self.model.clone();
        let ready_ref = self.ready.clone();

        task::spawn_blocking(move || -> DomainResult<()> {
            let mut model_lock = model_ref.lock().map_err(|e| {
                DomainError::InternalError(format!("Failed to lock embedding model: {}", e))
            })?;

            if model_lock.is_some() {
                return Ok(());
            }

            tracing::info!("Initializing FastEmbed model...");
            
            let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
            options.show_download_progress = true;

            let model = TextEmbedding::try_new(options).map_err(|e| {
                DomainError::InternalError(format!("Failed to initialize FastEmbed: {}", e))
            })?;

            *model_lock = Some(model);
            
            let mut ready_lock = ready_ref.lock().map_err(|e| {
                DomainError::InternalError(format!("Failed to lock ready state: {}", e))
            })?;
            *ready_lock = true;

            tracing::info!("FastEmbed model initialized successfully");
            Ok(())
        })
        .await
        .map_err(|e| DomainError::InternalError(format!("Task join error: {}", e)))?
    }

    fn is_ready(&self) -> bool {
        *self.ready.lock().unwrap_or_else(|e| e.into_inner())
    }

    async fn generate_embedding(&self, text: &str) -> DomainResult<Vec<f32>> {
        let model_ref = self.model.clone();
        let text = text.to_string();

        task::spawn_blocking(move || -> DomainResult<Vec<f32>> {
            let mut model_lock = model_ref.lock().map_err(|e| {
                DomainError::InternalError(format!("Failed to lock embedding model: {}", e))
            })?;

            let model = model_lock.as_mut().ok_or_else(|| {
                DomainError::InternalError("Embedding model not initialized".to_string())
            })?;

            let embeddings = model.embed(vec![text], None).map_err(|e| {
                DomainError::InternalError(format!("Failed to generate embedding: {}", e))
            })?;

            embeddings.first().cloned().ok_or_else(|| {
                DomainError::InternalError("No embedding generated".to_string())
            })
        })
        .await
        .map_err(|e| DomainError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn generate_embeddings(&self, texts: Vec<String>) -> DomainResult<Vec<Vec<f32>>> {
        let model_ref = self.model.clone();

        task::spawn_blocking(move || -> DomainResult<Vec<Vec<f32>>> {
            let mut model_lock = model_ref.lock().map_err(|e| {
                DomainError::InternalError(format!("Failed to lock embedding model: {}", e))
            })?;

            let model = model_lock.as_mut().ok_or_else(|| {
                DomainError::InternalError("Embedding model not initialized".to_string())
            })?;

            model.embed(texts, None).map_err(|e| {
                DomainError::InternalError(format!("Failed to generate embeddings: {}", e))
            })
        })
        .await
        .map_err(|e| DomainError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn classify_note(&self, _note_id: &str) -> DomainResult<Vec<ClassificationResult>> {
        // TODO: Implement classification based on topics
        Ok(Vec::new())
    }

    async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNote>> {
        let note = self.note_repository.find_by_id(note_id).await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        let source_embedding = note.embedding.ok_or_else(|| {
            DomainError::ValidationError("Note has no embedding".to_string())
        })?;

        let all_notes = self.note_repository.find_all(Default::default()).await?;
        
        let mut similarities: Vec<SimilarNote> = all_notes
            .into_iter()
            .filter(|n| n.id != note_id && n.embedding.is_some())
            .map(|n| {
                let similarity = Self::cosine_similarity(&source_embedding, n.embedding.as_ref().unwrap());
                SimilarNote {
                    note_id: n.id,
                    title: n.title,
                    similarity,
                    distance: 1.0 - similarity,
                }
            })
            .collect();

        // Sort by similarity descending
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(Ordering::Equal));

        if let Some(l) = limit {
            similarities.truncate(l as usize);
        }

        Ok(similarities)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNote>> {
        let query_embedding = self.generate_embedding(query).await?;

        let all_notes = self.note_repository.find_all(Default::default()).await?;
        
        let mut similarities: Vec<SimilarNote> = all_notes
            .into_iter()
            .filter(|n| n.embedding.is_some())
            .map(|n| {
                let similarity = Self::cosine_similarity(&query_embedding, n.embedding.as_ref().unwrap());
                SimilarNote {
                    note_id: n.id,
                    title: n.title,
                    similarity,
                    distance: 1.0 - similarity,
                }
            })
            .collect();

        // Sort by similarity descending
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(Ordering::Equal));

        if let Some(l) = limit {
            similarities.truncate(l as usize);
        }

        Ok(similarities)
    }

    async fn store_embedding(&self, note_id: &str, embedding: Vec<f32>) -> DomainResult<()> {
        let mut note = self.note_repository.find_by_id(note_id).await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;
        
        note.embedding = Some(embedding);
        self.note_repository.save(&note).await?;
        
        Ok(())
    }

    async fn get_embedding(&self, note_id: &str) -> DomainResult<Option<Vec<f32>>> {
        let note = self.note_repository.find_by_id(note_id).await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;
        
        Ok(note.embedding)
    }

    async fn delete_embedding(&self, note_id: &str) -> DomainResult<()> {
        let mut note = self.note_repository.find_by_id(note_id).await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;
        
        note.embedding = None;
        self.note_repository.save(&note).await?;
        
        Ok(())
    }

    async fn recompute_centroids(&self) -> DomainResult<()> {
        // TODO: Recompute topic centroids from embeddings
        Ok(())
    }

    async fn get_status(&self) -> DomainResult<EmbeddingStatus> {
        let all_notes = self.note_repository.find_all(Default::default()).await?;
        let total_notes = all_notes.len() as i32;
        let embedded_notes = all_notes.iter().filter(|n| n.embedding.is_some()).count() as i32;
        
        Ok(EmbeddingStatus {
            ready: self.is_ready(),
            total_notes,
            embedded_notes,
            pending_notes: total_notes - embedded_notes,
        })
    }
}
