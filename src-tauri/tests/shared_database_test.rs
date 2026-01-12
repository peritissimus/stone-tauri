use stone_tauri_lib::shared::database::{create_pool, schema, DatabaseConfig};

#[test]
fn test_create_in_memory_pool() {
    let config = DatabaseConfig::in_memory();
    let pool = create_pool(config);
    assert!(pool.is_ok());
}

#[test]
fn test_pool_get_connection() {
    let config = DatabaseConfig::in_memory();
    let pool = create_pool(config).unwrap();
    let conn = pool.get();
    assert!(conn.is_ok());
}

#[test]
fn test_schema_tables_accessible() {
    // Verify all 11 schema tables are accessible
    use schema::{
        attachments, note_links, note_tags, note_topics, note_versions, notebooks, notes,
        settings, tags, topics, workspaces,
    };

    // Just checking they compile
    let _workspaces = workspaces::table;
    let _notebooks = notebooks::table;
    let _notes = notes::table;
    let _tags = tags::table;
    let _note_tags = note_tags::table;
    let _note_links = note_links::table;
    let _attachments = attachments::table;
    let _note_versions = note_versions::table;
    let _topics = topics::table;
    let _note_topics = note_topics::table;
    let _settings = settings::table;
}
