/// Topic Use Cases Implementation
///
/// Application layer implementations for ML-based topic operations.
/// Orchestrates domain entities, repositories, and services for topic classification.
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::{Note, Topic},
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{
            ClassifyAllResponse, ClassifyNoteResponse, CreateTopicRequest, EmbeddingStatusResponse,
            NoteTopicInfo, SimilarNoteResult, TopicClassification, TopicUseCases,
            UpdateTopicRequest,
        },
        outbound::{
            EmbeddingService, EventPublisher, FileStorage, FindAllWithCountsOptions,
            GetNotesForTopicOptions, MarkdownProcessor, NoteFindOptions, NoteRepository,
            TopicAssignmentOptions, TopicRepository, TopicWithCount, WorkspaceRepository,
        },
    },
    services::similarity_calculator,
};

/// Implementation of all Topic use cases
pub struct TopicUseCasesImpl {
    topic_repository: Arc<dyn TopicRepository>,
    note_repository: Arc<dyn NoteRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
    embedding_service: Arc<dyn EmbeddingService>,
    markdown_processor: Arc<dyn MarkdownProcessor>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl TopicUseCasesImpl {
    pub fn new(
        topic_repository: Arc<dyn TopicRepository>,
        note_repository: Arc<dyn NoteRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
        embedding_service: Arc<dyn EmbeddingService>,
        markdown_processor: Arc<dyn MarkdownProcessor>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            topic_repository,
            note_repository,
            workspace_repository,
            file_storage,
            embedding_service,
            markdown_processor,
            event_publisher,
        }
    }

    /// Calculate the centroid (average) of a set of embeddings
    fn calculate_centroid(embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dimensions = embeddings[0].len();
        let mut centroid = vec![0.0; dimensions];

        for embedding in embeddings {
            for (i, &value) in embedding.iter().enumerate() {
                centroid[i] += value;
            }
        }

        let count = embeddings.len() as f32;
        for value in centroid.iter_mut() {
            *value /= count;
        }

        centroid
    }
}

#[async_trait]
impl TopicUseCases for TopicUseCasesImpl {
    /// Initialize the embedding service
    async fn initialize(&self) -> DomainResult<()> {
        self.embedding_service.initialize().await?;
        let ready = self.embedding_service.is_ready();
        tracing::info!("[TopicUseCases] Embedding service initialized, ready: {}", ready);
        Ok(())
    }

    /// Get all topics with note counts
    async fn get_all_topics(
        &self,
        exclude_journal: Option<bool>,
    ) -> DomainResult<Vec<TopicWithCount>> {
        let options = exclude_journal.map(|exclude| FindAllWithCountsOptions {
            exclude_journal: Some(exclude),
        });

        self.topic_repository.find_all_with_counts(options).await
    }

    /// Get topic by ID
    async fn get_topic_by_id(&self, id: &str) -> DomainResult<Option<Topic>> {
        self.topic_repository.find_by_id(id).await
    }

    /// Create a new topic
    async fn create_topic(&self, request: CreateTopicRequest) -> DomainResult<Topic> {
        let mut topic = Topic::new(&request.name)?;

        if let Some(description) = request.description {
            topic.update_description(Some(description));
        }

        if let Some(color) = request.color {
            topic.change_color(color)?;
        }

        self.topic_repository.save(&topic).await?;

        // Emit event (SYNC - no .await)
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "topic:created",
                serde_json::json!({
                    "id": topic.id,
                    "name": topic.name,
                }),
            );
        }

        Ok(topic)
    }

    /// Update a topic
    async fn update_topic(&self, request: UpdateTopicRequest) -> DomainResult<Topic> {
        let mut topic = self
            .topic_repository
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| DomainError::ValidationError(format!("Topic not found: {}", request.id)))?;

        if let Some(name) = request.name {
            topic.rename(name)?;
        }

        if let Some(description) = request.description {
            topic.update_description(Some(description));
        }

        if let Some(color) = request.color {
            topic.change_color(color)?;
        }

        self.topic_repository.save(&topic).await?;

        // Emit event (SYNC - no .await)
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "topic:updated",
                serde_json::json!({
                    "id": topic.id,
                    "name": topic.name,
                }),
            );
        }

        Ok(topic)
    }

    /// Delete a topic
    async fn delete_topic(&self, id: &str) -> DomainResult<()> {
        let topic = self
            .topic_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::ValidationError(format!("Topic not found: {}", id)))?;

        if !topic.can_delete() {
            return Err(DomainError::ValidationError(
                "Cannot delete predefined topics".to_string(),
            ));
        }

        self.topic_repository.delete(id).await?;

        // Emit event (SYNC - no .await)
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("topic:deleted", serde_json::json!({ "id": id }));
        }

        tracing::info!("[TopicUseCases] Deleted topic {}", id);
        Ok(())
    }

    /// Classify a note into topics based on its embedding
    async fn classify_note(&self, note_id: &str) -> DomainResult<ClassifyNoteResponse> {
        // Get the note
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // Check if already has embedding - if so, skip unless force is enabled
        let existing_embedding = self.embedding_service.get_embedding(note_id).await?;
        if existing_embedding.is_some() {
            return Ok(ClassifyNoteResponse {
                note_id: note_id.to_string(),
                topics: Vec::new(),
            });
        }

        // Get note content
        if note.file_path.is_none() || note.workspace_id.is_none() {
            return Ok(ClassifyNoteResponse {
                note_id: note_id.to_string(),
                topics: Vec::new(),
            });
        }

        let workspace = self
            .workspace_repository
            .find_by_id(note.workspace_id.as_ref().unwrap())
            .await?
            .ok_or_else(|| {
                DomainError::WorkspaceNotFound(
                    note.workspace_id.as_ref().unwrap().clone()
                )
            })?;

        let absolute_path = Path::new(&workspace.folder_path).join(note.file_path.as_ref().unwrap());
        let markdown = self
            .file_storage
            .read(absolute_path.to_str().unwrap())
            .await?
            .unwrap_or_default();

        if markdown.is_empty() {
            return Ok(ClassifyNoteResponse {
                note_id: note_id.to_string(),
                topics: Vec::new(),
            });
        }

        // Convert to plain text and generate embedding
        let plain_text = self.markdown_processor.extract_plain_text(&markdown)?;
        let embedding = self.embedding_service.generate_embedding(&plain_text).await?;

        // Save embedding
        self.embedding_service
            .store_embedding(note_id, embedding.clone())
            .await?;

        // Find all matching topics above threshold
        let topics = self.topic_repository.find_all().await?;
        let mut matched_topics: Vec<TopicClassification> = Vec::new();

        for topic in topics {
            if let Some(centroid_bytes) = topic.centroid {
                // Convert Vec<u8> to Vec<f32> - reinterpret bytes as f32 values
                if centroid_bytes.len() % 4 != 0 {
                    continue; // Skip malformed centroids
                }
                let centroid_f32: Vec<f32> = centroid_bytes
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                let similarity = similarity_calculator::cosine_similarity(&embedding, &centroid_f32)
                    .map_err(|e| DomainError::ValidationError(e))?;

                if similarity > 0.5 {
                    matched_topics.push(TopicClassification {
                        topic_id: topic.id,
                        topic_name: topic.name,
                        confidence: similarity,
                    });
                }
            }
        }

        // Sort by confidence descending
        matched_topics.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Assign the best matching topic if any
        if let Some(best_topic) = matched_topics.first() {
            self.topic_repository
                .assign_to_note(
                    note_id,
                    &best_topic.topic_id,
                    Some(TopicAssignmentOptions {
                        confidence: Some(best_topic.confidence),
                        is_manual: Some(false),
                    }),
                )
                .await?;

            // Emit event (SYNC - no .await)
            if let Some(ref publisher) = self.event_publisher {
                publisher.emit(
                    "note:classified",
                    serde_json::json!({
                        "note_id": note_id,
                        "topic_id": best_topic.topic_id,
                        "confidence": best_topic.confidence,
                    }),
                );
            }
        }

        Ok(ClassifyNoteResponse {
            note_id: note_id.to_string(),
            topics: matched_topics,
        })
    }

    /// Classify all notes in the active workspace
    async fn classify_all_notes(&self) -> DomainResult<ClassifyAllResponse> {
        let active_workspace = self.workspace_repository.find_active().await?;
        if active_workspace.is_none() {
            return Ok(ClassifyAllResponse {
                processed: 0,
                total: 0,
                failed: 0,
            });
        }

        let workspace = active_workspace.unwrap();
        let notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(workspace.id),
                is_deleted: Some(false),
                ..Default::default()
            })
            .await?;

        let total = notes.len() as i32;
        let mut processed = 0;
        let mut failed = 0;

        for note in notes {
            match self.classify_note(&note.id).await {
                Ok(_) => {
                    processed += 1;

                    // Emit progress event (SYNC - no .await)
                    if let Some(ref publisher) = self.event_publisher {
                        publisher.emit(
                            "embedding:progress",
                            serde_json::json!({
                                "processed": processed,
                                "total": total,
                                "failed": failed,
                            }),
                        );
                    }
                }
                Err(e) => {
                    failed += 1;
                    tracing::error!("[TopicUseCases] Failed to classify note {}: {:?}", note.id, e);
                }
            }
        }

        tracing::info!(
            "[TopicUseCases] Classified {}/{} notes ({} failed)",
            processed,
            total,
            failed
        );

        Ok(ClassifyAllResponse {
            processed,
            total,
            failed,
        })
    }

    /// Manually assign a topic to a note
    async fn assign_topic_to_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()> {
        self.topic_repository
            .assign_to_note(
                note_id,
                topic_id,
                Some(TopicAssignmentOptions {
                    confidence: Some(1.0),
                    is_manual: Some(true),
                }),
            )
            .await?;

        // Emit event (SYNC - no .await)
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "note:classified",
                serde_json::json!({
                    "note_id": note_id,
                    "topic_id": topic_id,
                    "confidence": 1.0,
                    "is_manual": true,
                }),
            );
        }

        tracing::info!("[TopicUseCases] Assigned topic {} to note {}", topic_id, note_id);
        Ok(())
    }

    /// Remove a topic from a note
    async fn remove_topic_from_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()> {
        self.topic_repository
            .remove_from_note(note_id, topic_id)
            .await?;

        // Emit event (SYNC - no .await)
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "note:classified",
                serde_json::json!({
                    "note_id": note_id,
                    "topic_id": serde_json::Value::Null,
                    "removed": true,
                }),
            );
        }

        tracing::info!("[TopicUseCases] Removed topic {} from note {}", topic_id, note_id);
        Ok(())
    }

    /// Find similar notes based on embeddings
    async fn get_similar_notes(
        &self,
        note_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNoteResult>> {
        // Verify note exists
        let _note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // Use EmbeddingService to find similar notes
        let similar_notes = self
            .embedding_service
            .find_similar_notes(note_id, limit)
            .await?;

        // Convert to SimilarNoteResult
        let results: Vec<SimilarNoteResult> = similar_notes
            .into_iter()
            .filter(|sn| sn.note_id != note_id) // Filter out the query note itself
            .take(limit.unwrap_or(10) as usize)
            .map(|sn| SimilarNoteResult {
                note_id: sn.note_id,
                title: sn.title,
                distance: 1.0 - sn.similarity, // Convert similarity to distance
            })
            .collect();

        Ok(results)
    }

    /// Semantic search across notes
    async fn semantic_search(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<SimilarNoteResult>> {
        // Check for active workspace
        let _workspace = self
            .workspace_repository
            .find_active()
            .await?
            .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        // Use EmbeddingService to perform semantic search
        let results = self
            .embedding_service
            .semantic_search(query, limit)
            .await?;

        // Convert to SimilarNoteResult
        let similar_notes: Vec<SimilarNoteResult> = results
            .into_iter()
            .map(|sn| SimilarNoteResult {
                note_id: sn.note_id,
                title: sn.title,
                distance: 1.0 - sn.similarity, // Convert similarity to distance
            })
            .collect();

        Ok(similar_notes)
    }

    /// Recompute centroids for all topics
    async fn recompute_centroids(&self) -> DomainResult<()> {
        let topics = self.topic_repository.find_all().await?;

        for topic in topics {
            let notes_for_topic = self
                .topic_repository
                .get_notes_for_topic(&topic.id, None)
                .await?;

            let note_ids: Vec<String> = notes_for_topic.iter().map(|n| n.note_id.clone()).collect();
            let mut embeddings: Vec<Vec<f32>> = Vec::new();

            for note_id in note_ids {
                if let Some(embedding) = self.embedding_service.get_embedding(&note_id).await? {
                    embeddings.push(embedding);
                }
            }

            if !embeddings.is_empty() {
                let centroid = Self::calculate_centroid(&embeddings);
                // Convert Vec<f32> to Vec<u8> - convert each f32 to 4 bytes
                let centroid_bytes: Vec<u8> = centroid
                    .iter()
                    .flat_map(|&f| f.to_le_bytes())
                    .collect();
                self.topic_repository
                    .update_centroid(&topic.id, centroid_bytes)
                    .await?;
            }
        }

        tracing::info!("[TopicUseCases] Recomputed all topic centroids");
        Ok(())
    }

    /// Get embedding status for the active workspace
    async fn get_embedding_status(&self) -> DomainResult<EmbeddingStatusResponse> {
        let ready = self.embedding_service.is_ready();

        let active_workspace = self.workspace_repository.find_active().await?;
        if active_workspace.is_none() {
            return Ok(EmbeddingStatusResponse {
                ready,
                total_notes: 0,
                embedded_notes: 0,
                pending_notes: 0,
            });
        }

        let workspace = active_workspace.unwrap();
        let notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(workspace.id),
                is_deleted: Some(false),
                ..Default::default()
            })
            .await?;

        let mut embedded_notes = 0;
        for note in &notes {
            if self.embedding_service.get_embedding(&note.id).await?.is_some() {
                embedded_notes += 1;
            }
        }

        let total_notes = notes.len() as i32;
        let pending_notes = total_notes - embedded_notes;

        Ok(EmbeddingStatusResponse {
            ready,
            total_notes,
            embedded_notes,
            pending_notes,
        })
    }

    /// Get notes for a topic
    async fn get_notes_for_topic(
        &self,
        topic_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> DomainResult<Vec<Note>> {
        let options = GetNotesForTopicOptions {
            limit,
            offset,
            exclude_journal: None,
        };

        let note_records = self
            .topic_repository
            .get_notes_for_topic(topic_id, Some(options))
            .await?;

        let mut notes = Vec::new();
        for record in note_records {
            if let Some(note) = self.note_repository.find_by_id(&record.note_id).await? {
                notes.push(note);
            }
        }

        Ok(notes)
    }

    /// Get topics for a note
    async fn get_topics_for_note(&self, note_id: &str) -> DomainResult<Vec<NoteTopicInfo>> {
        let topics_with_details = self.topic_repository.get_topics_for_note(note_id).await?;

        let results: Vec<NoteTopicInfo> = topics_with_details
            .into_iter()
            .map(|t| NoteTopicInfo {
                note_id: t.note_id,
                topic_id: t.topic_id,
                confidence: t.confidence,
                is_manual: t.is_manual,
                created_at: t.created_at,
                topic_name: t.topic_name,
                topic_color: t.topic_color,
            })
            .collect();

        Ok(results)
    }
}
