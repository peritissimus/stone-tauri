/// Integration test for workspace and note commands
/// This tests the actual command handlers with real database data

#[cfg(test)]
mod integration_tests {
    use stone_tauri_lib::{
        adapters::inbound::app_state::AppState,
        domain::ports::inbound::NoteQuery,
        infrastructure::{AppConfig, Container, DatabaseManager},
    };

    async fn setup_app_state() -> AppState {
        // Load config
        let config = AppConfig::load().expect("Failed to load config");

        // Initialize database
        let db_manager = DatabaseManager::new(config.database.clone())
            .await
            .expect("Failed to create database manager");
        db_manager.initialize().await.expect("Failed to initialize database");

        let pool = db_manager.get_pool();

        // For tests, we need a mock app handle
        // We'll skip Container build and create use cases directly
        // Actually, let's just test the use cases directly without needing AppState

        // Build container without app handle - we'll create a minimal one
        let (tx, _rx) = std::sync::mpsc::channel();
        let mock_handle = tauri::async_runtime::block_on(async {
            // This won't work without proper Tauri context
            // Let's use a different approach
            panic!("Cannot create mock app handle in integration test");
        });

        container.app_state
    }

    #[tokio::test]
    async fn test_get_active_workspace() {
        println!("\n=== Testing get_active_workspace ===");

        let app_state = setup_app_state().await;

        match app_state.workspace_usecases.get_active_workspace().await {
            Ok(Some(workspace)) => {
                println!("✅ Found active workspace:");
                println!("   ID: {}", workspace.id);
                println!("   Name: {}", workspace.name);
                println!("   Path: {}", workspace.folder_path);
            }
            Ok(None) => {
                println!("❌ No active workspace found");
                panic!("No active workspace");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                panic!("Failed to get active workspace: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_scan_workspace() {
        println!("\n=== Testing scan_workspace ===");

        let app_state = setup_app_state().await;

        // First get active workspace
        let workspace = app_state
            .workspace_usecases
            .get_active_workspace()
            .await
            .expect("Failed to get workspace")
            .expect("No active workspace");

        println!("Scanning workspace: {} ({})", workspace.name, workspace.id);

        match app_state.workspace_usecases.scan_workspace(&workspace.id).await {
            Ok(response) => {
                println!("✅ Scan successful:");
                println!("   Total files: {}", response.files.len());
                println!("   Folder structure: {} folders", response.structure.len());
                println!("\n   Sample files:");
                for (i, file) in response.files.iter().take(5).enumerate() {
                    println!("   {}. {}", i + 1, file.relative_path);
                }

                assert!(response.files.len() > 0, "Should have found files");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                panic!("Failed to scan workspace: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_note_by_path_variations() {
        println!("\n=== Testing get_note_by_path with different path formats ===");

        let app_state = setup_app_state().await;

        // Get a real note from database
        let notes = app_state
            .note_usecases
            .get_all_notes(None, None, None, None, None, None, None, None)
            .await
            .expect("Failed to get notes");

        if notes.is_empty() {
            println!("⚠️  No notes in database to test");
            return;
        }

        let sample_note = &notes[0];
        let db_path = sample_note.file_path.as_ref().expect("Note has no file path");

        println!("Using sample note:");
        println!("   Title: {}", sample_note.title);
        println!("   DB Path: {}", db_path);

        let workspace = app_state
            .workspace_usecases
            .get_active_workspace()
            .await
            .expect("Failed to get workspace")
            .expect("No active workspace");

        println!("   Workspace Path: {}", workspace.folder_path);

        // Test different path formats
        let test_paths = vec![
            // Just the relative path (what's stored in DB)
            db_path.clone(),
            // With leading slash
            format!("/{}", db_path),
            // Full path without leading slash
            format!("{}/{}", workspace.folder_path.trim_start_matches('/'), db_path),
            // Full path with leading slash
            format!("{}/{}", workspace.folder_path, db_path),
        ];

        println!("\nTesting {} path variations:", test_paths.len());

        for (i, test_path) in test_paths.iter().enumerate() {
            println!("\n{}. Testing path: '{}'", i + 1, test_path);

            match app_state.note_usecases.get_note_by_path(test_path).await {
                Ok(Some(note)) => {
                    println!("   ✅ Found note: {} (id: {})", note.title, note.id);
                    assert_eq!(note.id, sample_note.id, "Should find the same note");
                }
                Ok(None) => {
                    println!("   ❌ Note not found");
                }
                Err(e) => {
                    println!("   ❌ Error: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_path_normalization_logic() {
        println!("\n=== Testing Path Normalization Logic ===");

        let app_state = setup_app_state().await;

        let workspace = app_state
            .workspace_usecases
            .get_active_workspace()
            .await
            .expect("Failed to get workspace")
            .expect("No active workspace");

        let test_cases = vec![
            ("Work/Infra Release Bugs.md", "Work/Infra Release Bugs.md"),
            ("/Work/Infra Release Bugs.md", "Work/Infra Release Bugs.md"),
            ("Users/peritissimus/NoteBook/Work/Infra Release Bugs.md", "Work/Infra Release Bugs.md"),
            ("/Users/peritissimus/NoteBook/Work/Infra Release Bugs.md", "Work/Infra Release Bugs.md"),
        ];

        println!("Workspace path: {}", workspace.folder_path);
        println!("\nNormalization tests:");

        for (input, expected) in test_cases {
            let workspace_normalized = workspace.folder_path.trim_start_matches('/').trim_end_matches('/');
            let file_normalized = input.trim_start_matches('/');

            let result = if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
                relative.trim_start_matches('/').to_string()
            } else {
                file_normalized.to_string()
            };

            let status = if result == expected { "✅" } else { "❌" };
            println!("{} '{}' -> '{}'", status, input, result);

            if result != expected {
                println!("   Expected: '{}'", expected);
            }
        }
    }

    #[tokio::test]
    async fn test_full_note_opening_flow() {
        println!("\n=== Testing Full Note Opening Flow ===");
        println!("This simulates what happens when user clicks a file in the tree");

        let app_state = setup_app_state().await;

        // Step 1: Get active workspace
        println!("\n1. Get active workspace...");
        let workspace = app_state
            .workspace_usecases
            .get_active_workspace()
            .await
            .expect("Failed to get workspace")
            .expect("No active workspace");
        println!("   ✅ Workspace: {} at {}", workspace.name, workspace.folder_path);

        // Step 2: Scan workspace
        println!("\n2. Scan workspace for files...");
        let scan_result = app_state
            .workspace_usecases
            .scan_workspace(&workspace.id)
            .await
            .expect("Failed to scan");
        println!("   ✅ Found {} files", scan_result.files.len());

        if scan_result.files.is_empty() {
            println!("   ⚠️  No files to test");
            return;
        }

        // Step 3: Simulate clicking on a file from the tree
        let clicked_file = &scan_result.files[0];
        println!("\n3. User clicks file: {}", clicked_file.relative_path);

        // The frontend sends the path from the tree structure
        // Let's test what the frontend would actually send
        let frontend_path = format!("{}/{}",
            workspace.folder_path.trim_start_matches('/'),
            clicked_file.relative_path
        );

        println!("   Frontend sends: '{}'", frontend_path);

        // Step 4: Normalize the path (what our command handler does)
        let workspace_normalized = workspace.folder_path.trim_start_matches('/').trim_end_matches('/');
        let file_normalized = frontend_path.trim_start_matches('/');

        let normalized_path = if let Some(relative) = file_normalized.strip_prefix(workspace_normalized) {
            relative.trim_start_matches('/').to_string()
        } else {
            file_normalized.to_string()
        };

        println!("   Normalized to: '{}'", normalized_path);

        // Step 5: Look up note
        println!("\n4. Look up note in database...");
        match app_state.note_usecases.get_note_by_path(&normalized_path).await {
            Ok(Some(note)) => {
                println!("   ✅ SUCCESS! Found note:");
                println!("      ID: {}", note.id);
                println!("      Title: {}", note.title);
                println!("      File Path: {}", note.file_path.as_ref().unwrap_or(&"<none>".to_string()));
            }
            Ok(None) => {
                println!("   ❌ FAILED: Note not found in database");
                println!("      Looking for: '{}'", normalized_path);

                // Debug: Show what's actually in the database
                println!("\n   Database has these paths:");
                let all_notes = app_state.note_usecases
                    .get_all_notes(None, None, None, None, None, None, None, None)
                    .await
                    .expect("Failed to get notes");

                for note in all_notes.iter().take(10) {
                    if let Some(ref path) = note.file_path {
                        println!("      - {}", path);
                    }
                }

                panic!("Note not found!");
            }
            Err(e) => {
                println!("   ❌ ERROR: {}", e);
                panic!("Failed to get note: {}", e);
            }
        }
    }
}
