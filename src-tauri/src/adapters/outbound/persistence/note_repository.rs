//! Note Repository Implementation
//!
//! Diesel-based implementation of the Note repository port.

use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::domain::{
    entities::Note,
    errors::{DomainError, DomainResult},
    ports::outbound::{NoteFindOptions, NoteRepository},
};
use crate::shared::database::schema::notes;

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableNote, NoteRow},
    utils::{bool_to_i32, map_diesel_error},
};

/// Diesel implementation of NoteRepository
pub struct DieselNoteRepository {
    pool: Arc<DbPool>,
}

impl DieselNoteRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Helper to build a filtered query
    fn apply_filters<'a>(
        query: notes::BoxedQuery<'a, diesel::sqlite::Sqlite>,
        options: &'a NoteFindOptions,
    ) -> notes::BoxedQuery<'a, diesel::sqlite::Sqlite> {
        let mut query = query;

        if let Some(workspace_id) = &options.workspace_id {
            query = query.filter(notes::workspace_id.eq(workspace_id));
        }

        if let Some(notebook_id) = &options.notebook_id {
            query = query.filter(notes::notebook_id.eq(notebook_id));
        }

        if let Some(is_favorite) = options.is_favorite {
            query = query.filter(notes::is_favorite.eq(bool_to_i32(is_favorite)));
        }

        if let Some(is_pinned) = options.is_pinned {
            query = query.filter(notes::is_pinned.eq(bool_to_i32(is_pinned)));
        }

        if let Some(is_archived) = options.is_archived {
            query = query.filter(notes::is_archived.eq(bool_to_i32(is_archived)));
        }

        if let Some(is_deleted) = options.is_deleted {
            query = query.filter(notes::is_deleted.eq(bool_to_i32(is_deleted)));
        }

        query
    }
}

#[async_trait]
impl NoteRepository for DieselNoteRepository {
    /// Find a note by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Note>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            notes::table
                .filter(notes::id.eq(id))
                .first::<NoteRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all notes with optional filters
    async fn find_all(&self, options: NoteFindOptions) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();
            query = Self::apply_filters(query, &options);

            // Order by pinned first, then by updated_at descending
            query = query.order((notes::is_pinned.desc(), notes::updated_at.desc()));

            // Apply pagination
            if let Some(limit) = options.limit {
                query = query.limit(limit);
            }

            if let Some(offset) = options.offset {
                query = query.offset(offset);
            }

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find notes by notebook ID
    async fn find_by_notebook_id(
        &self,
        notebook_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let notebook_id = notebook_id.map(|s| s.to_string());
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            // Filter by notebook_id
            match notebook_id {
                Some(id) => {
                    query = query.filter(notes::notebook_id.eq(id));
                }
                None => {
                    query = query.filter(notes::notebook_id.is_null());
                }
            }

            // Optionally filter by workspace
            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            // Don't show deleted notes by default
            query = query.filter(notes::is_deleted.eq(0));

            query = query.order(notes::updated_at.desc());

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find notes by workspace ID
    async fn find_by_workspace_id(&self, workspace_id: &str) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = notes::table
                .filter(notes::workspace_id.eq(workspace_id))
                .filter(notes::is_deleted.eq(0))
                .order(notes::updated_at.desc())
                .load::<NoteRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find a note by file path
    async fn find_by_file_path(
        &self,
        file_path: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<Option<Note>> {
        let pool = self.pool.clone();
        let file_path = file_path.to_string();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();
            query = query.filter(notes::file_path.eq(file_path));

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query
                .first::<NoteRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Save a note (create or update)
    async fn save(&self, note: &Note) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note = note.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableNote::from_domain(&note);

            diesel::insert_into(notes::table)
                .values(&insertable)
                .on_conflict(notes::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete a note permanently
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(notes::table.filter(notes::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NoteNotFound(id));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if a note exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = notes::table
                .filter(notes::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Count notes with optional filters
    async fn count(&self, options: NoteFindOptions) -> DomainResult<i64> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();
            query = Self::apply_filters(query, &options);

            let count = query.count().get_result(&mut conn).map_err(map_diesel_error)?;

            Ok(count)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find recently updated notes
    async fn find_recently_updated(
        &self,
        limit: i64,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::is_deleted.eq(0));
            query = query.order(notes::updated_at.desc());
            query = query.limit(limit);

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find favorite notes
    async fn find_favorites(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::is_favorite.eq(1));
            query = query.filter(notes::is_deleted.eq(0));
            query = query.order(notes::updated_at.desc());

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find pinned notes
    async fn find_pinned(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::is_pinned.eq(1));
            query = query.filter(notes::is_deleted.eq(0));
            query = query.order(notes::updated_at.desc());

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find archived notes
    async fn find_archived(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::is_archived.eq(1));
            query = query.filter(notes::is_deleted.eq(0));
            query = query.order(notes::updated_at.desc());

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find deleted notes (trash)
    async fn find_deleted(&self, workspace_id: Option<&str>) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::is_deleted.eq(1));
            query = query.order(notes::deleted_at.desc());

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Search notes by title
    async fn search_by_title(
        &self,
        query: &str,
        workspace_id: Option<&str>,
        limit: Option<i64>,
    ) -> DomainResult<Vec<Note>> {
        let pool = self.pool.clone();
        let search_query = format!("%{}%", query);
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notes::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notes::workspace_id.eq(ws_id));
            }

            query = query.filter(notes::title.like(search_query));
            query = query.filter(notes::is_deleted.eq(0));
            query = query.order(notes::updated_at.desc());

            if let Some(lim) = limit {
                query = query.limit(lim);
            }

            let rows = query.load::<NoteRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
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

    fn create_test_note(id: &str, title: &str, workspace_id: &str) -> Note {
        Note {
            id: id.to_string(),
            title: title.to_string(),
            file_path: Some(format!("notes/{}.md", id)),
            notebook_id: None,
            workspace_id: Some(workspace_id.to_string()),
            is_favorite: false,
            is_pinned: false,
            is_archived: false,
            is_deleted: false,
            deleted_at: None,
            embedding: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteRepository::new(pool);

        let note = create_test_note("n1", "Test Note", "ws1");
        repo.save(&note).await.unwrap();

        let found = repo.find_by_id("n1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Note");
    }

    #[tokio::test]
    async fn test_find_all_with_filters() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteRepository::new(pool);

        let mut note1 = create_test_note("n1", "Note 1", "ws1");
        note1.is_favorite = true;
        let note2 = create_test_note("n2", "Note 2", "ws1");

        repo.save(&note1).await.unwrap();
        repo.save(&note2).await.unwrap();

        let options = NoteFindOptions {
            workspace_id: Some("ws1".to_string()),
            is_favorite: Some(true),
            ..Default::default()
        };

        let favorites = repo.find_all(options).await.unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, "n1");
    }

    #[tokio::test]
    async fn test_count_notes() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteRepository::new(pool);

        let note1 = create_test_note("n1", "Note 1", "ws1");
        let note2 = create_test_note("n2", "Note 2", "ws1");

        repo.save(&note1).await.unwrap();
        repo.save(&note2).await.unwrap();

        let options = NoteFindOptions {
            workspace_id: Some("ws1".to_string()),
            ..Default::default()
        };

        let count = repo.count(options).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_search_by_title() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteRepository::new(pool);

        let note1 = create_test_note("n1", "Hello World", "ws1");
        let note2 = create_test_note("n2", "Goodbye Moon", "ws1");

        repo.save(&note1).await.unwrap();
        repo.save(&note2).await.unwrap();

        let results = repo.search_by_title("Hello", Some("ws1"), None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Hello World");
    }

    #[tokio::test]
    async fn test_delete_note() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNoteRepository::new(pool);

        let note = create_test_note("n_del", "To Delete", "ws1");
        repo.save(&note).await.unwrap();

        repo.delete("n_del").await.unwrap();

        let found = repo.find_by_id("n_del").await.unwrap();
        assert!(found.is_none());
    }
}
