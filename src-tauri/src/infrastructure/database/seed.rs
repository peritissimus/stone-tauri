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

/// Seed default workspace
fn seed_default_workspace(conn: &mut SqliteConnection) -> DomainResult<(String, std::path::PathBuf)> {
    use schema::workspaces;

    // Use the user's NoteBook directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| {
            DomainError::ConfigurationError("Could not determine home directory".to_string())
        })?;

    let default_workspace_path = home_dir.join("NoteBook");

    // Create the directory if it doesn't exist
    if !default_workspace_path.exists() {
        std::fs::create_dir_all(&default_workspace_path).map_err(|e| {
            DomainError::FileStorageError(format!(
                "Failed to create default workspace directory: {}",
                e
            ))
        })?;
    }

    let workspace_id = uuid::Uuid::new_v4().to_string();
    let now = datetime_to_timestamp(&Utc::now());
    let workspace_path_str = default_workspace_path
        .to_str()
        .ok_or_else(|| {
            DomainError::ConfigurationError("Invalid workspace path".to_string())
        })?;

    diesel::insert_into(workspaces::table)
        .values((
            workspaces::id.eq(&workspace_id),
            workspaces::name.eq("NoteBook"),
            workspaces::folder_path.eq(workspace_path_str),
            workspaces::is_active.eq(1),
            workspaces::created_at.eq(now),
            workspaces::last_accessed_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to seed default workspace: {}", e))
        })?;

    Ok((workspace_id, default_workspace_path))
}

/// Seed default notebooks
fn seed_default_notebooks(conn: &mut SqliteConnection, workspace_id_val: &str) -> DomainResult<(String, String)> {
    use schema::notebooks;

    let personal_id = uuid::Uuid::new_v4().to_string();
    let work_id = uuid::Uuid::new_v4().to_string();
    let journal_id = uuid::Uuid::new_v4().to_string();
    let now = datetime_to_timestamp(&Utc::now());

    let default_notebooks = vec![
        (personal_id.clone(), "Personal", "Personal", "📝", "#ec4899", 0),
        (work_id.clone(), "Work", "Work", "💼", "#3b82f6", 1),
        (journal_id, "Journal", "Journal", "📅", "#22c55e", 2),
    ];

    for (nb_id, nb_name, nb_path, nb_icon, nb_color, nb_pos) in default_notebooks {
        diesel::insert_into(notebooks::table)
            .values((
                notebooks::id.eq(&nb_id),
                notebooks::name.eq(nb_name),
                notebooks::workspace_id.eq(workspace_id_val),
                notebooks::folder_path.eq(nb_path),
                notebooks::icon.eq(nb_icon),
                notebooks::color.eq(nb_color),
                notebooks::position.eq(nb_pos),
                notebooks::created_at.eq(now),
                notebooks::updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to seed notebook '{}': {}", nb_name, e))
            })?;
    }

    Ok((personal_id, work_id))
}

/// Seed default tags
fn seed_default_tags(conn: &mut SqliteConnection) -> DomainResult<(String, String)> {
    use schema::tags;

    let ideas_id = uuid::Uuid::new_v4().to_string();
    let planning_id = uuid::Uuid::new_v4().to_string();
    let now = datetime_to_timestamp(&Utc::now());

    let default_tags = vec![
        (ideas_id.clone(), "ideas", "#22c55e"),
        (planning_id.clone(), "planning", "#f97316"),
    ];

    for (tag_id, tag_name, tag_color) in default_tags {
        diesel::insert_into(tags::table)
            .values((
                tags::id.eq(&tag_id),
                tags::name.eq(tag_name),
                tags::color.eq(tag_color),
                tags::created_at.eq(now),
                tags::updated_at.eq(now),
            ))
            .execute(conn)
            .map_err(|e| {
                DomainError::DatabaseError(format!("Failed to seed tag '{}': {}", tag_name, e))
            })?;
    }

    Ok((ideas_id, planning_id))
}

/// Seed default notes and physical files
fn seed_default_notes(
    conn: &mut SqliteConnection, 
    workspace_id_val: &str, 
    workspace_path: &std::path::Path,
    personal_notebook_id_val: &str,
    work_notebook_id_val: &str,
    ideas_tag_id_val: &str,
    planning_tag_id_val: &str
) -> DomainResult<()> {
    use schema::{notes, note_tags};

    let welcome_id = uuid::Uuid::new_v4().to_string();
    let roadmap_id = uuid::Uuid::new_v4().to_string();
    let now = datetime_to_timestamp(&Utc::now());

    // Create physical directories
    let personal_dir = workspace_path.join("Personal");
    let work_dir = workspace_path.join("Work");
    let journal_dir = workspace_path.join("Journal");

    std::fs::create_dir_all(&personal_dir).map_err(|e| DomainError::FileStorageError(e.to_string()))?;
    std::fs::create_dir_all(&work_dir).map_err(|e| DomainError::FileStorageError(e.to_string()))?;
    std::fs::create_dir_all(&journal_dir).map_err(|e| DomainError::FileStorageError(e.to_string()))?;

    // Create Welcome Note
    let welcome_file_path = "Personal/Welcome to Stone.md";
    let welcome_abs_path = workspace_path.join("Personal/Welcome to Stone.md");
    let welcome_content = r#"---
tags:
  - ideas
favorite: true
pinned: true
---

# Welcome to Stone

This sample note shows how rich text content is stored.

- Create notebooks to organize topics.
- Add tags to group related ideas.
- Use the TipTap editor to capture your thoughts.
"#;

    if !welcome_abs_path.exists() {
        std::fs::write(&welcome_abs_path, welcome_content)
            .map_err(|e| DomainError::FileStorageError(format!("Failed to write welcome note: {}", e)))?;
    }

    diesel::insert_into(notes::table)
        .values((
            notes::id.eq(&welcome_id),
            notes::title.eq("Welcome to Stone"),
            notes::file_path.eq(welcome_file_path),
            notes::notebook_id.eq(personal_notebook_id_val),
            notes::workspace_id.eq(workspace_id_val),
            notes::is_favorite.eq(1),
            notes::is_pinned.eq(1),
            notes::created_at.eq(now),
            notes::updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| DomainError::DatabaseError(format!("Failed to seed welcome note: {}", e)))?;

    // Link tag to Welcome Note
    diesel::insert_into(note_tags::table)
        .values((
            note_tags::note_id.eq(&welcome_id),
            note_tags::tag_id.eq(ideas_tag_id_val),
            note_tags::created_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| DomainError::DatabaseError(format!("Failed to link tag to welcome note: {}", e)))?;

    // Create Roadmap Note
    let roadmap_file_path = "Work/Product Roadmap.md";
    let roadmap_abs_path = workspace_path.join("Work/Product Roadmap.md");
    let roadmap_content = r#"---
tags:
  - planning
---

# Quarterly Roadmap

Track the high-level initiatives planned for this quarter.

1. Ship the new editor experience.
2. Improve sync reliability.
3. Publish public beta announcement.
"#;

    if !roadmap_abs_path.exists() {
        std::fs::write(&roadmap_abs_path, roadmap_content)
            .map_err(|e| DomainError::FileStorageError(format!("Failed to write roadmap note: {}", e)))?;
    }

    diesel::insert_into(notes::table)
        .values((
            notes::id.eq(&roadmap_id),
            notes::title.eq("Product Roadmap"),
            notes::file_path.eq(roadmap_file_path),
            notes::notebook_id.eq(work_notebook_id_val),
            notes::workspace_id.eq(workspace_id_val),
            notes::created_at.eq(now),
            notes::updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| DomainError::DatabaseError(format!("Failed to seed roadmap note: {}", e)))?;

    // Link tag to Roadmap Note
    diesel::insert_into(note_tags::table)
        .values((
            note_tags::note_id.eq(&roadmap_id),
            note_tags::tag_id.eq(planning_tag_id_val),
            note_tags::created_at.eq(now),
        ))
        .execute(conn)
        .map_err(|e| DomainError::DatabaseError(format!("Failed to link tag to roadmap note: {}", e)))?;

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
                // Seed default workspace first
                let (workspace_id, workspace_path) = seed_default_workspace(conn).map_err(|_e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default workspace");

                // Seed notebooks
                let (personal_nb_id, work_nb_id) = seed_default_notebooks(conn, &workspace_id).map_err(|_e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default notebooks");

                // Seed tags
                let (ideas_tag_id, planning_tag_id) = seed_default_tags(conn).map_err(|_e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default tags");

                // Seed notes
                seed_default_notes(
                    conn, 
                    &workspace_id, 
                    &workspace_path, 
                    &personal_nb_id, 
                    &work_nb_id, 
                    &ideas_tag_id, 
                    &planning_tag_id
                ).map_err(|_e| {
                     diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default notes");

                // Seed topics - convert errors inside
                seed_topics(conn).map_err(|_e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded predefined topics");

                // Seed default settings
                seed_default_settings(conn).map_err(|_e| {
                    diesel::result::Error::RollbackTransaction
                })?;
                tracing::info!("Seeded default settings");

                // Mark as seeded
                mark_database_seeded(conn).map_err(|_e| {
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
