//! Shared test helpers and utilities
//! This module provides common functionality for integration tests

use stone_tauri_lib::infrastructure::{DatabaseManager, DatabaseConfig};
use std::path::PathBuf;
use chrono::Timelike;

/// Creates an in-memory database configuration for testing
///
/// Using :memory: for tests provides:
/// - Fast execution (no disk I/O)
/// - Isolation (each test gets clean database)
/// - No cleanup needed (DB disappears after test)
pub fn setup_test_db() -> DatabaseConfig {
    DatabaseConfig::development(PathBuf::from(":memory:"))
}

/// Creates and initializes a test database manager
///
/// This sets up a complete database with schema migrations applied,
/// ready for testing repository operations.
pub async fn setup_test_db_manager() -> DatabaseManager {
    let config = setup_test_db();
    let manager = DatabaseManager::new(config)
        .await
        .expect("Failed to create test database manager");

    manager.initialize()
        .await
        .expect("Failed to initialize test database");

    manager
}

/// Test data builder for consistent test fixtures
pub struct TestDataBuilder;

impl TestDataBuilder {
    /// Default note title for tests
    pub fn sample_note_title() -> &'static str {
        "Test Note"
    }

    /// Default note content with markdown features
    pub fn sample_note_content() -> &'static str {
        r#"# Test Note

This is a **test** note with _various_ markdown features.

## Features
- Bullet points
- **Bold text**
- *Italic text*

## Code Block
```rust
fn main() {
    println!("Hello, Stone!");
}
```

## Tasks
- TODO Write tests
- DONE Setup project

## Links
See [[Related Note]] for more information.
"#
    }

    /// Default workspace name
    pub fn sample_workspace_name() -> &'static str {
        "Test Workspace"
    }

    /// Default workspace path
    pub fn sample_workspace_path() -> &'static str {
        "/tmp/test-workspace"
    }

    /// Sample journal entry with timestamp
    pub fn sample_journal_entry() -> String {
        let now = chrono::Local::now();
        format!("[{:02}:{:02}] Test journal entry", now.hour(), now.minute())
    }

    /// Sample wiki link syntax
    pub fn sample_wiki_link(target: &str) -> String {
        format!("[[{}]]", target)
    }

    /// Sample wiki link with custom text
    pub fn sample_wiki_link_with_text(target: &str, text: &str) -> String {
        format!("[[{}|{}]]", target, text)
    }
}

/// Helper to generate note IDs for testing
pub fn generate_test_note_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Helper to generate workspace IDs for testing
pub fn generate_test_workspace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
