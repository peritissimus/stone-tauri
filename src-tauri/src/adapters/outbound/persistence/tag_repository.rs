//! Tag Repository Implementation
//!
//! Diesel-based implementation of the Tag repository port with many-to-many relationships.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;

use crate::domain::{
    entities::Tag,
    errors::{DomainError, DomainResult},
    ports::outbound::{TagRepository, TagWithCount},
};
use crate::shared::database::schema::{note_tags, notes, tags};

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableTag, TagRow},
    utils::{datetime_to_timestamp, map_diesel_error},
};

/// Diesel implementation of TagRepository
pub struct DieselTagRepository {
    pool: Arc<DbPool>,
}

impl DieselTagRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for DieselTagRepository {
    /// Find a tag by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Tag>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            tags::table
                .filter(tags::id.eq(id))
                .first::<TagRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find tag by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<Tag>> {
        let pool = self.pool.clone();
        let name = name.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            tags::table
                .filter(tags::name.eq(name))
                .first::<TagRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all tags
    async fn find_all(&self) -> DomainResult<Vec<Tag>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = tags::table
                .order(tags::name.asc())
                .load::<TagRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all tags with note counts
    async fn find_all_with_counts(&self) -> DomainResult<Vec<TagWithCount>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Get all tags
            let tag_rows: Vec<TagRow> = tags::table
                .order(tags::name.asc())
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            // Count notes for each tag
            let mut tags_with_counts = Vec::new();

            for tag_row in tag_rows {
                let tag_id = tag_row.id.clone();

                // Count non-deleted notes with this tag
                let note_count: i64 = note_tags::table
                    .inner_join(notes::table)
                    .filter(note_tags::tag_id.eq(&tag_id))
                    .filter(notes::is_deleted.eq(0))
                    .count()
                    .get_result(&mut conn)
                    .map_err(map_diesel_error)?;

                let tag = tag_row.to_domain();
                tags_with_counts.push(TagWithCount {
                    id: tag.id,
                    name: tag.name,
                    description: None,
                    color: Some(tag.color),
                    created_at: tag.created_at,
                    updated_at: tag.updated_at,
                    note_count: note_count as i32,
                });
            }

            Ok(tags_with_counts)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find tags by note ID
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Tag>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = note_tags::table
                .inner_join(tags::table)
                .filter(note_tags::note_id.eq(note_id))
                .select(tags::all_columns)
                .order(tags::name.asc())
                .load::<TagRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Save a tag (create or update)
    async fn save(&self, tag: &Tag) -> DomainResult<()> {
        let pool = self.pool.clone();
        let tag = tag.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableTag::from_domain(&tag);

            diesel::insert_into(tags::table)
                .values(&insertable)
                .on_conflict(tags::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete a tag
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(tags::table.filter(tags::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotFound(format!("Tag not found: {}", id)));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if a tag exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = tags::table
                .filter(tags::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Add a tag to a note
    async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();
        let tag_id = tag_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::insert_into(note_tags::table)
                .values((
                    note_tags::note_id.eq(&note_id),
                    note_tags::tag_id.eq(&tag_id),
                    note_tags::created_at.eq(datetime_to_timestamp(&Utc::now())),
                ))
                .on_conflict((note_tags::note_id, note_tags::tag_id))
                .do_nothing()
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove a tag from a note
    async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();
        let tag_id = tag_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(
                note_tags::table
                    .filter(note_tags::note_id.eq(note_id))
                    .filter(note_tags::tag_id.eq(tag_id)),
            )
            .execute(&mut conn)
            .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all tags for a note
    async fn get_note_tags(&self, note_id: &str) -> DomainResult<Vec<Tag>> {
        // This is the same as find_by_note_id
        self.find_by_note_id(note_id).await
    }

    /// Set tags for a note (replaces all existing tags)
    async fn set_note_tags(&self, note_id: &str, tag_ids: Vec<String>) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Use transaction for atomicity
            conn.transaction(|conn| {
                // First, delete all existing tags for this note
                diesel::delete(note_tags::table.filter(note_tags::note_id.eq(&note_id)))
                    .execute(conn)?;

                // Then, insert the new tags
                let now = datetime_to_timestamp(&Utc::now());
                for tag_id in tag_ids {
                    diesel::insert_into(note_tags::table)
                        .values((
                            note_tags::note_id.eq(&note_id),
                            note_tags::tag_id.eq(&tag_id),
                            note_tags::created_at.eq(now),
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

    /// Get tags for multiple notes (returns a map of note_id -> tags)
    async fn get_tags_for_notes(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<Tag>>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Get all note_tag associations for the given notes
            let associations: Vec<(String, TagRow)> = note_tags::table
                .inner_join(tags::table)
                .filter(note_tags::note_id.eq_any(&note_ids))
                .select((note_tags::note_id, tags::all_columns))
                .order((note_tags::note_id.asc(), tags::name.asc()))
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            // Group tags by note_id
            let mut result: HashMap<String, Vec<Tag>> = HashMap::new();

            for (note_id, tag_row) in associations {
                result
                    .entry(note_id)
                    .or_insert_with(Vec::new)
                    .push(tag_row.to_domain());
            }

            // Ensure all requested note_ids have an entry (even if empty)
            for note_id in note_ids {
                result.entry(note_id).or_insert_with(Vec::new);
            }

            Ok(result)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    fn create_test_tag(id: &str, name: &str) -> Tag {
        Tag {
            id: id.to_string(),
            name: name.to_string(),
            color: "#6b7280".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_tag() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTagRepository::new(pool);

        let tag = create_test_tag("tag1", "Important");
        repo.save(&tag).await.unwrap();

        let found = repo.find_by_id("tag1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Important");
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTagRepository::new(pool);

        let tag = create_test_tag("tag2", "Work");
        repo.save(&tag).await.unwrap();

        let found = repo.find_by_name("Work").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "tag2");
    }

    #[tokio::test]
    async fn test_add_tag_to_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTagRepository::new(pool);

        let tag = create_test_tag("tag3", "Project");
        repo.save(&tag).await.unwrap();

        repo.add_tag_to_note("note1", "tag3").await.unwrap();

        let tags = repo.get_note_tags("note1").await.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Project");
    }

    #[tokio::test]
    async fn test_set_note_tags() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTagRepository::new(pool);

        let tag1 = create_test_tag("tag4", "Red");
        let tag2 = create_test_tag("tag5", "Blue");
        let tag3 = create_test_tag("tag6", "Green");

        repo.save(&tag1).await.unwrap();
        repo.save(&tag2).await.unwrap();
        repo.save(&tag3).await.unwrap();

        // Set initial tags
        repo.set_note_tags("note2", vec!["tag4".to_string(), "tag5".to_string()])
            .await
            .unwrap();

        let tags = repo.get_note_tags("note2").await.unwrap();
        assert_eq!(tags.len(), 2);

        // Replace with different tags
        repo.set_note_tags("note2", vec!["tag6".to_string()])
            .await
            .unwrap();

        let tags = repo.get_note_tags("note2").await.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Green");
    }

    #[tokio::test]
    async fn test_get_tags_for_notes() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselTagRepository::new(pool);

        let tag1 = create_test_tag("tag7", "Alpha");
        let tag2 = create_test_tag("tag8", "Beta");

        repo.save(&tag1).await.unwrap();
        repo.save(&tag2).await.unwrap();

        repo.add_tag_to_note("note3", "tag7").await.unwrap();
        repo.add_tag_to_note("note4", "tag8").await.unwrap();
        repo.add_tag_to_note("note4", "tag7").await.unwrap();

        let tags_map = repo
            .get_tags_for_notes(vec!["note3".to_string(), "note4".to_string()])
            .await
            .unwrap();

        assert_eq!(tags_map.get("note3").unwrap().len(), 1);
        assert_eq!(tags_map.get("note4").unwrap().len(), 2);
    }
}
