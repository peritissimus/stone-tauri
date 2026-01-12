//! Attachment Repository Implementation
//!
//! Diesel-based implementation of the Attachment repository port.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::domain::{
    entities::Attachment,
    errors::{DomainError, DomainResult},
    ports::outbound::AttachmentRepository,
};
use crate::shared::database::schema::attachments;

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{AttachmentRow, InsertableAttachment},
    utils::map_diesel_error,
};

/// Diesel implementation of AttachmentRepository
pub struct DieselAttachmentRepository {
    pool: Arc<DbPool>,
}

impl DieselAttachmentRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttachmentRepository for DieselAttachmentRepository {
    /// Find attachment by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Attachment>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            attachments::table
                .filter(attachments::id.eq(id))
                .first::<AttachmentRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all attachments for a note
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Attachment>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = attachments::table
                .filter(attachments::note_id.eq(note_id))
                .order(attachments::created_at.asc())
                .load::<AttachmentRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get attachments for multiple notes (bulk operation)
    async fn find_by_note_ids(
        &self,
        note_ids: Vec<String>,
    ) -> DomainResult<HashMap<String, Vec<Attachment>>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Get all attachments for the given notes
            let rows: Vec<AttachmentRow> = attachments::table
                .filter(attachments::note_id.eq_any(&note_ids))
                .order((attachments::note_id.asc(), attachments::created_at.asc()))
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            // Group attachments by note_id
            let mut result: HashMap<String, Vec<Attachment>> = HashMap::new();

            for row in rows {
                let note_id = row.note_id.clone();
                result
                    .entry(note_id)
                    .or_insert_with(Vec::new)
                    .push(row.to_domain());
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

    /// Save an attachment
    async fn save(&self, attachment: &Attachment) -> DomainResult<()> {
        let pool = self.pool.clone();
        let attachment = attachment.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableAttachment::from_domain(&attachment);

            diesel::insert_into(attachments::table)
                .values(&insertable)
                .on_conflict(attachments::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete an attachment
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(attachments::table.filter(attachments::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotFound(format!(
                    "Attachment not found: {}",
                    id
                )));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete all attachments for a note
    async fn delete_by_note_id(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(attachments::table.filter(attachments::note_id.eq(note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if attachment exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = attachments::table
                .filter(attachments::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Count attachments for a note
    async fn count_by_note_id(&self, note_id: &str) -> DomainResult<i32> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = attachments::table
                .filter(attachments::note_id.eq(note_id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count as i32)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;
    use chrono::Utc;

    fn create_test_attachment(id: &str, note_id: &str, filename: &str) -> Attachment {
        Attachment {
            id: id.to_string(),
            note_id: note_id.to_string(),
            filename: filename.to_string(),
            mime_type: "image/png".to_string(),
            size: 1024,
            path: format!("/attachments/{}", filename),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_attachment() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let attachment = create_test_attachment("att1", "note1", "test.png");
        repo.save(&attachment).await.unwrap();

        let found = repo.find_by_id("att1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().filename, "test.png");
    }

    #[tokio::test]
    async fn test_find_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let att1 = create_test_attachment("att1", "note1", "image1.png");
        let att2 = create_test_attachment("att2", "note1", "image2.png");
        let att3 = create_test_attachment("att3", "note2", "image3.png");

        repo.save(&att1).await.unwrap();
        repo.save(&att2).await.unwrap();
        repo.save(&att3).await.unwrap();

        let attachments = repo.find_by_note_id("note1").await.unwrap();
        assert_eq!(attachments.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_note_ids() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let att1 = create_test_attachment("att1", "note1", "image1.png");
        let att2 = create_test_attachment("att2", "note2", "image2.png");
        let att3 = create_test_attachment("att3", "note2", "image3.png");

        repo.save(&att1).await.unwrap();
        repo.save(&att2).await.unwrap();
        repo.save(&att3).await.unwrap();

        let attachments_map = repo
            .find_by_note_ids(vec!["note1".to_string(), "note2".to_string()])
            .await
            .unwrap();

        assert_eq!(attachments_map.get("note1").unwrap().len(), 1);
        assert_eq!(attachments_map.get("note2").unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_delete_attachment() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let attachment = create_test_attachment("att1", "note1", "test.png");
        repo.save(&attachment).await.unwrap();

        repo.delete("att1").await.unwrap();

        let found = repo.find_by_id("att1").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let att1 = create_test_attachment("att1", "note1", "image1.png");
        let att2 = create_test_attachment("att2", "note1", "image2.png");

        repo.save(&att1).await.unwrap();
        repo.save(&att2).await.unwrap();

        repo.delete_by_note_id("note1").await.unwrap();

        let attachments = repo.find_by_note_id("note1").await.unwrap();
        assert_eq!(attachments.len(), 0);
    }

    #[tokio::test]
    async fn test_count_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let att1 = create_test_attachment("att1", "note1", "image1.png");
        let att2 = create_test_attachment("att2", "note1", "image2.png");

        repo.save(&att1).await.unwrap();
        repo.save(&att2).await.unwrap();

        let count = repo.count_by_note_id("note1").await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_exists() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselAttachmentRepository::new(pool);

        let attachment = create_test_attachment("att1", "note1", "test.png");
        repo.save(&attachment).await.unwrap();

        assert!(repo.exists("att1").await.unwrap());
        assert!(!repo.exists("att999").await.unwrap());
    }
}
