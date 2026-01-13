/// Standalone test for workspace and note commands
/// Run with: cargo run --example test_commands

use stone_tauri_lib::{
    adapters::outbound::persistence::*,
    domain::ports::outbound::{NoteRepository, WorkspaceRepository},
    infrastructure::{AppConfig, DatabaseManager},
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Stone Commands Integration Test ===\n");

    // Load config
    println!("1. Loading configuration...");
    let config = AppConfig::load().expect("Failed to load config");
    println!("   ✅ Config loaded");

    // Initialize database
    println!("\n2. Initializing database...");
    let db_manager = DatabaseManager::new(config.database.clone())
        .await
        .expect("Failed to create database manager");
    db_manager
        .initialize()
        .await
        .expect("Failed to initialize database");
    let pool = db_manager.get_pool();
    println!("   ✅ Database initialized");

    // Create repositories
    println!("\n3. Creating repositories...");
    let workspace_repo = Arc::new(DieselWorkspaceRepository::new(pool.clone()));
    let note_repo = Arc::new(DieselNoteRepository::new(pool.clone()));
    println!("   ✅ Repositories created");

    // We'll test repositories directly, skipping use cases
    println!("\n4. Testing repositories directly...");

    // TEST 1: Get active workspace
    println!("\n=== TEST 1: Get Active Workspace ===");
    match workspace_repo.find_active().await {
        Ok(Some(workspace)) => {
            println!("✅ Found active workspace:");
            println!("   ID: {}", workspace.id);
            println!("   Name: {}", workspace.name);
            println!("   Path: {}", workspace.folder_path);

            // TEST 2: Get notes from database
            println!("\n=== TEST 2: Get Notes from Database ===");
            match note_repo
                .find_all(stone_tauri_lib::domain::ports::outbound::NoteFindOptions {
                    workspace_id: Some(workspace.id.clone()),
                    ..Default::default()
                })
                .await
            {
                Ok(notes) => {
                    println!("✅ Found {} notes", notes.len());

                    if notes.is_empty() {
                        println!("   ⚠️  No notes to test with");
                        return;
                    }

                    // Show first 5 notes
                    println!("\n   Sample notes:");
                    for (i, note) in notes.iter().take(5).enumerate() {
                        println!(
                            "   {}. {} (path: {})",
                            i + 1,
                            note.title,
                            note.file_path.as_ref().unwrap_or(&"<none>".to_string())
                        );
                    }

                    // TEST 3: Test path lookups
                    println!("\n=== TEST 3: Test Path Lookups ===");
                    let test_note = &notes[0];
                    let db_path = test_note.file_path.as_ref().expect("Note has no file path");

                    println!("Testing with note: {}", test_note.title);
                    println!("DB stores path as: '{}'", db_path);

                    // Simulate different path formats that might come from frontend
                    let test_paths = vec![
                        // Relative path (what's in DB)
                        db_path.clone(),
                        // With leading slash
                        format!("/{}", db_path),
                        // Full path (what frontend might send)
                        format!(
                            "{}/{}",
                            workspace.folder_path.trim_start_matches('/'),
                            db_path
                        ),
                        // Full path with leading slash
                        format!("{}/{}", workspace.folder_path, db_path),
                    ];

                    println!("\nTesting {} path variations:\n", test_paths.len());

                    for (i, test_path) in test_paths.iter().enumerate() {
                        println!("{}. Path: '{}'", i + 1, test_path);

                        // Normalize the path (what our command handler should do)
                        let workspace_normalized =
                            workspace.folder_path.trim_start_matches('/').trim_end_matches('/');
                        let file_normalized = test_path.trim_start_matches('/');

                        let normalized_path =
                            if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
                                relative.trim_start_matches('/').to_string()
                            } else {
                                file_normalized.to_string()
                            };

                        println!("   Normalized to: '{}'", normalized_path);

                        // Look up in database
                        match note_repo
                            .find_by_file_path(&normalized_path, Some(&workspace.id))
                            .await
                        {
                            Ok(Some(found_note)) => {
                                println!("   ✅ FOUND: {} (id: {})", found_note.title, found_note.id);
                                assert_eq!(found_note.id, test_note.id);
                            }
                            Ok(None) => {
                                println!("   ❌ NOT FOUND in database");
                                println!("      Looked for: '{}'", normalized_path);
                            }
                            Err(e) => {
                                println!("   ❌ ERROR: {}", e);
                            }
                        }
                        println!();
                    }

                    // TEST 4: Show what frontend is actually sending
                    println!("=== TEST 4: What Frontend Sends ===");
                    println!("Based on logs, frontend sends:");
                    println!("  'Users/peritissimus/NoteBook/Personal/Stone_Editor_Preview.md'");
                    println!("\nThis should normalize to:");
                    println!("  'Personal/Stone_Editor_Preview.md'");

                    let frontend_example = "Users/peritissimus/NoteBook/Personal/Stone_Editor_Preview.md";
                    let workspace_normalized =
                        workspace.folder_path.trim_start_matches('/').trim_end_matches('/');
                    let file_normalized = frontend_example.trim_start_matches('/');

                    let result =
                        if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
                            relative.trim_start_matches('/').to_string()
                        } else {
                            file_normalized.to_string()
                        };

                    println!("\nActual normalization:");
                    println!("  Input: '{}'", frontend_example);
                    println!("  Workspace: '{}'", workspace_normalized);
                    println!("  Result: '{}'", result);

                    // Check if this note exists
                    match note_repo
                        .find_by_file_path(&result, Some(&workspace.id))
                        .await
                    {
                        Ok(Some(note)) => {
                            println!("\n  ✅ This note EXISTS in database!");
                            println!("     Title: {}", note.title);
                        }
                        Ok(None) => {
                            println!("\n  ❌ This note DOES NOT EXIST in database");
                            println!("     Check if file exists in: {}", workspace.folder_path);
                        }
                        Err(e) => {
                            println!("\n  ❌ ERROR: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Error fetching notes: {}", e);
                }
            }
        }
        Ok(None) => {
            println!("❌ No active workspace found");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    println!("\n=== Test Complete ===");
}
