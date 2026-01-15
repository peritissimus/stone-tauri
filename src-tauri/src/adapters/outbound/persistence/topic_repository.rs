//! Topic Repository Implementation
//!
//! Diesel-based implementation of the Topic repository port with ML features.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;

use crate::domain::{
    entities::Topic,
    errors::{DomainError, DomainResult},
    ports::outbound::{
        FindAllWithCountsOptions, GetNotesForTopicOptions,
        NoteTopicWithDetails, TopicAssignmentOptions, TopicNoteRecord, TopicRepository,
        TopicWithCount,
    },
};
use crate::shared::database::schema::{note_topics, notes, topics};

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableTopic, TopicRow},
    utils::{bool_to_i32, datetime_to_timestamp, i32_to_bool, map_diesel_error, timestamp_to_datetime},
};

/// Diesel implementation of TopicRepository
pub struct DieselTopicRepository {
    pool: Arc<DbPool>,
}

impl DieselTopicRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TopicRepository for DieselTopicRepository {
    /// Find topic by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Topic>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            topics::table
                .filter(topics::id.eq(id))
                .first::<TopicRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find topic by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<Topic>> {
        let pool = self.pool.clone();
        let name = name.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            topics::table
                .filter(topics::name.eq(name))
                .first::<TopicRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all topics
    async fn find_all(&self) -> DomainResult<Vec<Topic>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = topics::table
                .order(topics::name.asc())
                .load::<TopicRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all topics with note counts
    async fn find_all_with_counts(
        &self,
        _options: Option<FindAllWithCountsOptions>,
    ) -> DomainResult<Vec<TopicWithCount>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Get all topics
            let topic_rows: Vec<TopicRow> = topics::table
                .order(topics::name.asc())
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            // Count notes for each topic
            let mut topics_with_counts = Vec::new();

            for topic_row in topic_rows {
                let topic_id = topic_row.id.clone();

                // Count non-deleted notes with this topic
                let note_count: i64 = note_topics::table
                    .inner_join(notes::table)
                    .filter(note_topics::topic_id.eq(&topic_id))
                    .filter(notes::is_deleted.eq(0))
                    .count()
                    .get_result(&mut conn)
                    .map_err(map_diesel_error)?;

                let topic = topic_row.to_domain();
                topics_with_counts.push(TopicWithCount {
                    id: topic.id,
                    name: topic.name,
                    description: topic.description,
                    color: topic.color,
                    centroid: topic.centroid,
                    is_predefined: topic.is_predefined,
                    note_count: note_count as i32,
                    created_at: topic.created_at,
                    updated_at: topic.updated_at,
                });
            }

            Ok(topics_with_counts)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get predefined topics
    async fn find_predefined(&self) -> DomainResult<Vec<Topic>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = topics::table
                .filter(topics::is_predefined.eq(1))
                .order(topics::name.asc())
                .load::<TopicRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Save a topic
    async fn save(&self, topic: &Topic) -> DomainResult<()> {
        let pool = self.pool.clone();
        let topic = topic.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableTopic::from_domain(&topic);

            diesel::insert_into(topics::table)
                .values(&insertable)
                .on_conflict(topics::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete a topic
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(topics::table.filter(topics::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotFound(format!("Topic not found: {}", id)));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if topic exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = topics::table
                .filter(topics::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get topics for a note
    async fn get_topics_for_note(&self, note_id: &str) -> DomainResult<Vec<NoteTopicWithDetails>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let results: Vec<(String, String, Option<f32>, i32, i64, String, Option<String>)> =
                note_topics::table
                    .inner_join(topics::table)
                    .filter(note_topics::note_id.eq(&note_id))
                    .select((
                        note_topics::note_id,
                        note_topics::topic_id,
                        note_topics::confidence,
                        note_topics::is_manual,
                        note_topics::created_at,
                        topics::name,
                        topics::color,
                    ))
                    .order(topics::name.asc())
                    .load(&mut conn)
                    .map_err(map_diesel_error)?;

            let details = results
                .into_iter()
                .map(|(note_id, topic_id, confidence, is_manual, created_at, name, color)| {
                    NoteTopicWithDetails {
                        note_id,
                        topic_id,
                        confidence: confidence.unwrap_or(1.0),
                        is_manual: i32_to_bool(is_manual),
                        created_at: timestamp_to_datetime(created_at),
                        topic_name: name,
                        topic_color: color.unwrap_or_else(|| Topic::default_color()),
                    }
                })
                .collect();

            Ok(details)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get topics for multiple notes (bulk)
    async fn get_topics_for_notes(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<NoteTopicWithDetails>>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let results: Vec<(String, String, Option<f32>, i32, i64, String, Option<String>)> =
                note_topics::table
                    .inner_join(topics::table)
                    .filter(note_topics::note_id.eq_any(&note_ids))
                    .select((
                        note_topics::note_id,
                        note_topics::topic_id,
                        note_topics::confidence,
                        note_topics::is_manual,
                        note_topics::created_at,
                        topics::name,
                        topics::color,
                    ))
                    .order((note_topics::note_id.asc(), topics::name.asc()))
                    .load(&mut conn)
                    .map_err(map_diesel_error)?;

            // Group by note_id
            let mut map: HashMap<String, Vec<NoteTopicWithDetails>> = HashMap::new();

            for (note_id, topic_id, confidence, is_manual, created_at, name, color) in results {
                map.entry(note_id.clone())
                    .or_insert_with(Vec::new)
                    .push(NoteTopicWithDetails {
                        note_id,
                        topic_id,
                        confidence: confidence.unwrap_or(1.0),
                        is_manual: i32_to_bool(is_manual),
                        created_at: timestamp_to_datetime(created_at),
                        topic_name: name,
                        topic_color: color.unwrap_or_else(|| Topic::default_color()),
                    });
            }

            // Ensure all requested note_ids have an entry
            for note_id in note_ids {
                map.entry(note_id).or_insert_with(Vec::new);
            }

            Ok(map)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get notes for a topic
    async fn get_notes_for_topic(
        &self,
        topic_id: &str,
        options: Option<GetNotesForTopicOptions>,
    ) -> DomainResult<Vec<TopicNoteRecord>> {
        let pool = self.pool.clone();
        let topic_id = topic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = note_topics::table
                .inner_join(notes::table)
                .filter(note_topics::topic_id.eq(&topic_id))
                .filter(notes::is_deleted.eq(0))
                .into_boxed();

            if let Some(opts) = options {
                if let Some(limit) = opts.limit {
                    query = query.limit(limit as i64);
                }
                if let Some(offset) = opts.offset {
                    query = query.offset(offset as i64);
                }
            }

            let results: Vec<(String, Option<f32>, i32)> = query
                .select((
                    note_topics::note_id,
                    note_topics::confidence,
                    note_topics::is_manual,
                ))
                .order(note_topics::confidence.desc())
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            let records = results
                .into_iter()
                .map(|(note_id, confidence, is_manual)| TopicNoteRecord {
                    note_id,
                    confidence: confidence.unwrap_or(1.0),
                    is_manual: i32_to_bool(is_manual),
                })
                .collect();

            Ok(records)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Assign topic to note
    async fn assign_to_note(
        &self,
        note_id: &str,
        topic_id: &str,
        options: Option<TopicAssignmentOptions>,
    ) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();
        let topic_id = topic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let confidence = options.as_ref().and_then(|o| o.confidence).unwrap_or(1.0);
            let is_manual = options.as_ref().and_then(|o| o.is_manual).unwrap_or(false);

            diesel::insert_into(note_topics::table)
                .values((
                    note_topics::note_id.eq(&note_id),
                    note_topics::topic_id.eq(&topic_id),
                    note_topics::confidence.eq(confidence),
                    note_topics::is_manual.eq(bool_to_i32(is_manual)),
                    note_topics::created_at.eq(datetime_to_timestamp(&Utc::now())),
                ))
                .on_conflict((note_topics::note_id, note_topics::topic_id))
                .do_update()
                .set((
                    note_topics::confidence.eq(confidence),
                    note_topics::is_manual.eq(bool_to_i32(is_manual)),
                ))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove topic from note
    async fn remove_from_note(&self, note_id: &str, topic_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();
        let topic_id = topic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(
                note_topics::table
                    .filter(note_topics::note_id.eq(note_id))
                    .filter(note_topics::topic_id.eq(topic_id)),
            )
            .execute(&mut conn)
            .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Set all topics for a note (replaces existing)
    async fn set_topics_for_note(
        &self,
        note_id: &str,
        assignments: Vec<(String, Option<f32>, Option<bool>)>,
    ) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Use transaction for atomicity
            conn.transaction(|conn| {
                // Delete existing topic assignments
                diesel::delete(note_topics::table.filter(note_topics::note_id.eq(&note_id)))
                    .execute(conn)?;

                // Insert new assignments
                let now = datetime_to_timestamp(&Utc::now());
                for (topic_id, confidence, is_manual) in assignments {
                    diesel::insert_into(note_topics::table)
                        .values((
                            note_topics::note_id.eq(&note_id),
                            note_topics::topic_id.eq(&topic_id),
                            note_topics::confidence.eq(confidence.unwrap_or(1.0)),
                            note_topics::is_manual.eq(bool_to_i32(is_manual.unwrap_or(false))),
                            note_topics::created_at.eq(now),
                        ))
                        .execute(conn)?;
                }

                Ok(())
            })
            .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Clear all topics for a note
    async fn clear_topics_for_note(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(note_topics::table.filter(note_topics::note_id.eq(note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Update topic centroid (ML)
    async fn update_centroid(&self, topic_id: &str, centroid: Vec<u8>) -> DomainResult<()> {
        let pool = self.pool.clone();
        let topic_id = topic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::update(topics::table.filter(topics::id.eq(&topic_id)))
                .set(topics::centroid.eq(Some(centroid)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Update note count for a topic
    async fn update_note_count(&self, topic_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let topic_id = topic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Count notes
            let count: i64 = note_topics::table
                .inner_join(notes::table)
                .filter(note_topics::topic_id.eq(&topic_id))
                .filter(notes::is_deleted.eq(0))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            // Update count
            diesel::update(topics::table.filter(topics::id.eq(&topic_id)))
                .set(topics::note_count.eq(Some(count as i32)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    fn create_test_topic(id: &str, name: &str) -> Topic {
        Topic {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            color: Topic::default_color(),
            is_predefined: false,
            centroid: None,
            note_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_topic() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTopicRepository::new(pool);

        let topic = create_test_topic("topic1", "Technology");
        repo.save(&topic).await.unwrap();

        let found = repo.find_by_id("topic1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Technology");
    }

    #[tokio::test]
    async fn test_assign_topic_to_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTopicRepository::new(pool);

        let topic = create_test_topic("topic2", "Science");
        repo.save(&topic).await.unwrap();

        let options = TopicAssignmentOptions {
            confidence: Some(0.85),
            is_manual: Some(false),
        };

        repo.assign_to_note("note1", "topic2", Some(options))
            .await
            .unwrap();

        let topics = repo.get_topics_for_note("note1").await.unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].topic_name, "Science");
        assert!((topics[0].confidence - 0.85).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_update_centroid() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTopicRepository::new(pool);

        let topic = create_test_topic("topic3", "Math");
        repo.save(&topic).await.unwrap();

        let centroid = vec![1, 2, 3, 4, 5];
        repo.update_centroid("topic3", centroid.clone())
            .await
            .unwrap();

        let found = repo.find_by_id("topic3").await.unwrap().unwrap();
        assert_eq!(found.centroid, Some(centroid));
    }
}
