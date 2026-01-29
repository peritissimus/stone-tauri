//! Test Workspace Sync Workflow
//!
//! Critical Path: Scan filesystem → Create DB entries → Detect external changes
//! This enables bidirectional sync with external editors (VSCode, Vim, etc.)

use stone_tauri_lib::domain::entities::Workspace;

#[test]
fn test_workspace_creation_and_activation() {
    // Simulate user creating a new workspace
    let workspace_name = "My Knowledge Base";
    let workspace_path = "/Users/test/Documents/notes";

    let workspace = Workspace::new(workspace_name, workspace_path)
        .expect("Should create workspace with valid data");

    assert_eq!(workspace.name, workspace_name);
    assert_eq!(workspace.folder_path, workspace_path);
    assert!(!workspace.is_active, "New workspace should not be active");

    // User activates workspace
    let mut active_workspace = workspace;
    active_workspace.activate();
    assert!(active_workspace.is_active);
}

#[test]
fn test_workspace_validation() {
    // Empty name should fail
    assert!(Workspace::new("", "/valid/path").is_err());

    // Empty path should fail
    assert!(Workspace::new("Valid Name", "").is_err());

    // Whitespace-only name should fail
    assert!(Workspace::new("   ", "/valid/path").is_err());

    // Valid workspace
    assert!(Workspace::new("Notes", "/home/user/notes").is_ok());
}

#[test]
fn test_workspace_name_length_limits() {
    // Very long name (at 100 char limit)
    let long_name = "a".repeat(100);
    let result = Workspace::new(&long_name, "/tmp/test");
    assert!(result.is_ok(), "Name at limit should be accepted");

    // Exceeding limit (101+ chars)
    let too_long = "a".repeat(101);
    let result = Workspace::new(&too_long, "/tmp/test");
    assert!(result.is_err(), "Name exceeding limit should be rejected");
}

#[test]
fn test_workspace_activation_toggle() {
    // Simulate switching between workspaces
    let mut workspace1 = Workspace::new("Work", "/work/notes").unwrap();
    let mut workspace2 = Workspace::new("Personal", "/personal/notes").unwrap();

    // Activate workspace1
    workspace1.activate();
    assert!(workspace1.is_active);
    assert!(!workspace2.is_active);

    // Switch to workspace2 (would deactivate workspace1 in real app)
    workspace1.deactivate();
    workspace2.activate();
    assert!(!workspace1.is_active);
    assert!(workspace2.is_active);
}

#[test]
fn test_workspace_path_variations() {
    // Test various valid path formats
    let valid_paths = vec![
        "/Users/john/Documents/notes",
        "/home/user/workspace",
        "C:\\Users\\John\\Notes",
        "/mnt/data/knowledge-base",
        "./relative/path/notes",
    ];

    for path in valid_paths {
        let result = Workspace::new("Test", path);
        assert!(result.is_ok(), "Should accept path: {}", path);
    }
}
