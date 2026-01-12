//! Database Seeding
//!
//! Seeds initial data like predefined topics and default settings.

use diesel::prelude::*;
use std::sync::Arc;
use chrono::Utc;
use crate::{
    domain::errors::{DomainError, DomainResult},
    shared::database::{DbPool, schema},
    adapters::outbound::persistence::utils::datetime_to_timestamp,
};

/// Check if database has been seeded
fn is_database_seeded(conn: &mut SqliteConnection) -> DomainResult<bool> {
    use schema::settings::dsl::*;

    let count: i64 = settings
        .filter(key.eq("database_seeded"))
        .count()
        .get_result(conn)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to check seed status: {}", e))
        })?;

    Ok(count > 0)
}

/// Mark database as seeded
fn mark_database_seeded(conn: &mut SqliteConnection) -> DomainResult<()> {
    use schema::settings::dsl::*;

    let now = datetime_to_timestamp(&Utc::now());

    diesel::insert_into(settings)
        .values((
            key.eq("database_seeded"),
            value.eq("true"),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to mark database as seeded: {}", e))
        })?;

    Ok(())
}

/// Seed predefined topics
fn seed_topics(conn: &mut SqliteConnection) -> DomainResult<()> {
    use schema::topics::dsl::*;

    let predefined_topics = vec![
        ("Work", "Work-related notes and projects", "#3B82F6"),
        ("Personal", "Personal notes and thoughts", "#10B981"),
        ("Learning", "Educational content and learning materials", "#F59E0B"),
        ("Projects", "Project documentation and planning", "#8B5CF6"),
        ("Ideas", "Ideas and brainstorming", "#EC4899"),
        ("Archive", "Archived content", "#6B7280"),
    ];

    for (topic_name, topic_description, topic_color) in predefined_topics {
        let topic_id = uuid::Uuid::new_v4().to_string();
        let now = datetime_to_timestamp(&Utc::now());

        diesel::insert_into(topics)
            .values((
                id.eq(&topic_id),
                name.eq(topic_name),
                description.eq(Some(topic_description)),
                color.eq(Some(topic_color)),
                is_predefined.eq(1),
                created_at.eq(now),
                updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to seed topic '{}': {}", topic_name, e))
            })?;
    }

    Ok(())
}

/// Seed default settings
fn seed_default_settings(conn: &mut SqliteConnection) -> DomainResult<()> {
    use schema::settings::dsl::*;

    let default_settings = vec![
        ("theme", "system"),
        ("editor_font_family", "Inter"),
        ("editor_font_size", "16"),
        ("auto_save", "true"),
        ("auto_save_interval", "30"),
        ("spell_check", "true"),
        ("line_numbers", "false"),
        ("vim_mode", "false"),
        ("git_auto_commit", "false"),
        ("git_auto_sync", "false"),
        ("show_word_count", "true"),
        ("show_character_count", "false"),
        ("default_view", "editor"),
    ];

    let now = datetime_to_timestamp(&Utc::now());

    for (setting_key, setting_value) in default_settings {
        diesel::insert_into(settings)
            .values((
                key.eq(setting_key),
                value.eq(setting_value),
                updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|e| {
                DomainError::DatabaseError(format!(
                    "Failed to seed setting '{}': {}",
                    setting_key, e
                ))
            })?;
    }

    Ok(())
}

/// Seed initial data into the database
///
/// This function is idempotent - it will only seed data if the database
/// hasn't been seeded before (checked via a settings flag).
pub async fn seed_initial_data(pool: Arc<DbPool>) -> DomainResult<()> {
    let pool_clone = pool.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get database connection: {}", e))
        })?;

        // Check if already seeded
        if is_database_seeded(&mut conn)? {
            tracing::info!("Database already seeded, skipping");
            return Ok(());
        }

        tracing::info!("Seeding database with initial data...");

        // Seed in transaction for atomicity
        // Note: Use diesel::result::Error directly in transaction then convert
        let seed_result: Result<(), DomainError> = (|| {
            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                // Seed topics - convert errors inside
                seed_topics(conn).map_err(|e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded predefined topics");

                // Seed default settings
                seed_default_settings(conn).map_err(|e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default settings");

                // Mark as seeded
                mark_database_seeded(conn).map_err(|e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Marked database as seeded");

                Ok(())
            }).map_err(|e| {
                DomainError::DatabaseError(format!("Transaction failed: {}", e))
            })
        })();

        seed_result?;

        tracing::info!("Database seeding completed successfully");
        Ok(())
    })
    .await
    .map_err(|e| {
        DomainError::DatabaseError(format!("Failed to spawn seeding task: {}", e))
    })??;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::create_pool;
    use crate::infrastructure::database::run_migrations;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_seed_initial_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = Arc::new(create_pool(db_path.to_str().unwrap()).unwrap());

        // Run migrations first
        run_migrations(&pool).unwrap();

        // Seed data
        let result = seed_initial_data(pool.clone()).await;
        assert!(result.is_ok());

        // Verify topics were created
        let mut conn = pool.get().unwrap();
        use schema::topics::dsl::*;
        let topic_count: i64 = topics.count().get_result(&mut conn).unwrap();
        assert_eq!(topic_count, 6);

        // Verify settings were created
        use schema::settings::dsl::*;
        let setting_count: i64 = settings.count().get_result(&mut conn).unwrap();
        assert!(setting_count > 10); // At least 10 settings + seeded flag
    }

    #[tokio::test]
    async fn test_seed_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = Arc::new(create_pool(db_path.to_str().unwrap()).unwrap());

        run_migrations(&pool).unwrap();

        // Seed twice
        seed_initial_data(pool.clone()).await.unwrap();
        let result = seed_initial_data(pool.clone()).await;

        // Should succeed and not duplicate data
        assert!(result.is_ok());

        let mut conn = pool.get().unwrap();
        use schema::topics::dsl::*;
        let topic_count: i64 = topics.count().get_result(&mut conn).unwrap();
        assert_eq!(topic_count, 6); // Still only 6, not 12
    }
}
