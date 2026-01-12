//! NoteLink Repository Implementation
//!
//! Diesel-based implementation of the NoteLink repository port with bidirectional link support.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;

use crate::domain::{
    entities::{LinkCount, Note, NoteLink},
    errors::{DomainError, DomainResult},
    ports::outbound::NoteLinkRepository,
};
use crate::shared::database::schema::{note_links, notes};

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableNoteLink, NoteLinkRow, NoteRow},
    utils::{datetime_to_timestamp, map_diesel_error},
};

/// Diesel implementation of NoteLinkRepository
pub struct DieselNoteLinkRepository {
    pool: Arc<DbPool>,
}

impl DieselNoteLinkRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NoteLinkRepository for DieselNoteLinkRepository {
    /// Get all links
    async fn find_all(&self) -> DomainResult<Vec<NoteLink>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = note_links::table
                .order(note_links::created_at.asc())
                .load::<NoteLinkRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get backlinks for a note (notes that link TO this note)
    async fn get_backlinks(&self, note_id: &str) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Find all source_note_ids where target_note_id = note_id
            let source_ids: Vec<String> = note_links::table
                .filter(note_links::target_note_id.eq(&note_id))
                .select(note_links::source_note_id)
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            if source_ids.is_empty() {
                return Ok(Vec::new());
            }

            // Fetch the actual notes
            let note_rows: Vec<NoteRow> = notes::table
                .filter(notes::id.eq_any(source_ids))
                .filter(notes::is_deleted.eq(0))
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(note_rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get forward links from a note (notes this note links TO)
    async fn get_forward_links(&self, note_id: &str) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Find all target_note_ids where source_note_id = note_id
            let target_ids: Vec<String> = note_links::table
                .filter(note_links::source_note_id.eq(&note_id))
                .select(note_links::target_note_id)
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            if target_ids.is_empty() {
                return Ok(Vec::new());
            }

            // Fetch the actual notes
            let note_rows: Vec<NoteRow> = notes::table
                .filter(notes::id.eq_any(target_ids))
                .filter(notes::is_deleted.eq(0))
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(note_rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Add a link between two notes
    async fn save(&self, link: &NoteLink) -> DomainResult<()> {
        let pool = self.pool.clone();
        let link = link.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableNoteLink::from_domain(&link);

            diesel::insert_into(note_links::table)
                .values(&insertable)
                .on_conflict((note_links::source_note_id, note_links::target_note_id))
                .do_nothing()
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove a link between two notes
    async fn delete(&self, source_id: &str, target_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let source_id = source_id.to_string();
        let target_id = target_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(
                note_links::table
                    .filter(note_links::source_note_id.eq(&source_id))
                    .filter(note_links::target_note_id.eq(&target_id)),
            )
            .execute(&mut conn)
            .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotFound(format!(
                    "Link not found: {} -> {}",
                    source_id, target_id
                )));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove all links from a note
    async fn delete_from_note(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(note_links::table.filter(note_links::source_note_id.eq(note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove all links to a note
    async fn delete_to_note(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(note_links::table.filter(note_links::target_note_id.eq(note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Remove all links involving a note (both directions)
    async fn delete_all_for_note(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Delete links FROM this note
            diesel::delete(note_links::table.filter(note_links::source_note_id.eq(&note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            // Delete links TO this note
            diesel::delete(note_links::table.filter(note_links::target_note_id.eq(&note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if a link exists
    async fn exists(&self, source_id: &str, target_id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let source_id = source_id.to_string();
        let target_id = target_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = note_links::table
                .filter(note_links::source_note_id.eq(source_id))
                .filter(note_links::target_note_id.eq(target_id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Count links for a note
    async fn count_for_note(&self, note_id: &str) -> DomainResult<LinkCount> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Count outgoing links (this note -> others)
            let outgoing: i64 = note_links::table
                .filter(note_links::source_note_id.eq(&note_id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            // Count incoming links (others -> this note)
            let incoming: i64 = note_links::table
                .filter(note_links::target_note_id.eq(&note_id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(LinkCount {
                incoming: incoming as i32,
                outgoing: outgoing as i32,
            })
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Set links for a note (replace all outgoing links)
    async fn set_links_from_note(
        &self,
        source_id: &str,
        target_ids: Vec<String>,
    ) -> DomainResult<()> {
        let pool = self.pool.clone();
        let source_id = source_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Use transaction for atomicity
            conn.transaction(|conn| {
                // First, delete all existing outgoing links from this note
                diesel::delete(note_links::table.filter(note_links::source_note_id.eq(&source_id)))
                    .execute(conn)?;

                // Then, insert the new links
                let now = datetime_to_timestamp(&Utc::now());
                for target_id in target_ids {
                    // Skip self-links
                    if target_id == source_id {
                        continue;
                    }

                    diesel::insert_into(note_links::table)
                        .values((
                            note_links::source_note_id.eq(&source_id),
                            note_links::target_note_id.eq(&target_id),
                            note_links::created_at.eq(now),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    fn create_test_link(source: &str, target: &str) -> NoteLink {
        NoteLink {
            source_note_id: source.to_string(),
            target_note_id: target.to_string(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_links() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        let link = create_test_link("note1", "note2");
        repo.save(&link).await.unwrap();

        let all_links = repo.find_all().await.unwrap();
        assert_eq!(all_links.len(), 1);
        assert_eq!(all_links[0].source_note_id, "note1");
        assert_eq!(all_links[0].target_note_id, "note2");
    }

    #[tokio::test]
    async fn test_exists() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        let link = create_test_link("note1", "note2");
        repo.save(&link).await.unwrap();

        assert!(repo.exists("note1", "note2").await.unwrap());
        assert!(!repo.exists("note2", "note1").await.unwrap());
        assert!(!repo.exists("note1", "note3").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_link() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        let link = create_test_link("note1", "note2");
        repo.save(&link).await.unwrap();

        repo.delete("note1", "note2").await.unwrap();

        assert!(!repo.exists("note1", "note2").await.unwrap());
    }

    #[tokio::test]
    async fn test_count_for_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        // Note1 -> Note2
        repo.save(&create_test_link("note1", "note2")).await.unwrap();
        // Note1 -> Note3
        repo.save(&create_test_link("note1", "note3")).await.unwrap();
        // Note3 -> Note1
        repo.save(&create_test_link("note3", "note1")).await.unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.outgoing, 2); // note1 -> note2, note1 -> note3
        assert_eq!(count.incoming, 1); // note3 -> note1
        assert_eq!(count.total(), 3);
    }

    #[tokio::test]
    async fn test_delete_from_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        repo.save(&create_test_link("note1", "note2")).await.unwrap();
        repo.save(&create_test_link("note1", "note3")).await.unwrap();
        repo.save(&create_test_link("note2", "note1")).await.unwrap();

        repo.delete_from_note("note1").await.unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.outgoing, 0);
        assert_eq!(count.incoming, 1); // note2 -> note1 still exists
    }

    #[tokio::test]
    async fn test_delete_to_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        repo.save(&create_test_link("note1", "note2")).await.unwrap();
        repo.save(&create_test_link("note3", "note2")).await.unwrap();
        repo.save(&create_test_link("note2", "note1")).await.unwrap();

        repo.delete_to_note("note2").await.unwrap();

        let count = repo.count_for_note("note2").await.unwrap();
        assert_eq!(count.incoming, 0);
        assert_eq!(count.outgoing, 1); // note2 -> note1 still exists
    }

    #[tokio::test]
    async fn test_delete_all_for_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        repo.save(&create_test_link("note1", "note2")).await.unwrap();
        repo.save(&create_test_link("note2", "note1")).await.unwrap();
        repo.save(&create_test_link("note1", "note3")).await.unwrap();

        repo.delete_all_for_note("note1").await.unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.total(), 0);
    }

    #[tokio::test]
    async fn test_set_links_from_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        // Set initial links
        repo.set_links_from_note("note1", vec!["note2".to_string(), "note3".to_string()])
            .await
            .unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.outgoing, 2);

        // Replace with different links
        repo.set_links_from_note("note1", vec!["note4".to_string()])
            .await
            .unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.outgoing, 1);

        assert!(repo.exists("note1", "note4").await.unwrap());
        assert!(!repo.exists("note1", "note2").await.unwrap());
        assert!(!repo.exists("note1", "note3").await.unwrap());
    }

    #[tokio::test]
    async fn test_set_links_skips_self_links() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteLinkRepository::new(pool);

        // Try to create self-link
        repo.set_links_from_note(
            "note1",
            vec!["note1".to_string(), "note2".to_string()],
        )
        .await
        .unwrap();

        let count = repo.count_for_note("note1").await.unwrap();
        assert_eq!(count.outgoing, 1); // Only note1 -> note2

        assert!(!repo.exists("note1", "note1").await.unwrap());
        assert!(repo.exists("note1", "note2").await.unwrap());
    }
}
