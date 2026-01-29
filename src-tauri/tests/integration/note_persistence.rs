//! Test Note Creation & Persistence Workflow
//!
//! Critical Path: Create note → write to file → save to DB
//! This is the most fundamental operation in Stone

use stone_tauri_lib::domain::entities::Note;

#[test]
fn test_note_entity_creation() {
    // Test creating a note entity with valid data
    let note = Note::new("Meeting Notes", Some("workspace-123".to_string()))
        .expect("Should create note with valid title");

    assert_eq!(note.title, "Meeting Notes");
    assert_eq!(note.workspace_id, Some("workspace-123".to_string()));
    assert!(!note.is_favorite);
    assert!(!note.is_pinned);
    assert!(!note.is_archived);
    assert!(!note.is_deleted);
    assert!(!note.id.is_empty());
}

#[test]
fn test_note_title_validation() {
    // Empty title should fail
    let result = Note::new("", None);
    assert!(result.is_err(), "Empty title should be rejected");

    // Whitespace-only title should fail
    let result = Note::new("   ", None);
    assert!(result.is_err(), "Whitespace-only title should be rejected");

    // Very long title (near 500 char limit)
    let long_title = "a".repeat(500);
    let result = Note::new(&long_title, None);
    assert!(result.is_ok(), "Title at limit should be accepted");

    // Title exceeding limit should fail
    let too_long = "a".repeat(501);
    let result = Note::new(&too_long, None);
    assert!(result.is_err(), "Title exceeding limit should be rejected");
}

#[test]
fn test_note_lifecycle_full_workflow() {
    // Simulate full note lifecycle: create → favorite → edit → archive → delete → restore
    let mut note = Note::new("Project Ideas", None)
        .expect("Should create note");

    // Mark as favorite (user clicks star)
    note.mark_favorite();
    assert!(note.is_favorite);

    // Update title (user renames)
    note.update_title("Project Roadmap")
        .expect("Should update title");
    assert_eq!(note.title, "Project Roadmap");

    // Pin note (user pins to top)
    note.pin();
    assert!(note.is_pinned);

    // Archive note (user archives old project)
    note.archive();
    assert!(note.is_archived);

    // Delete note (user sends to trash)
    note.delete();
    assert!(note.is_deleted);
    assert!(note.deleted_at.is_some());

    // Restore from trash (user recovers note)
    note.restore();
    assert!(!note.is_deleted);
    assert!(note.deleted_at.is_none());
}

#[test]
fn test_note_special_characters_in_title() {
    // Test various special characters that might appear in note titles
    let test_cases = vec![
        "Meeting @ 3pm",
        "TODO: Fix bug #123",
        "Notes (2024)",
        "Q&A Session",
        "50% complete",
        "File: report.pdf",
        "Emoji test 📝",
        "Quote \"test\"",
    ];

    for title in test_cases {
        let result = Note::new(title, None);
        assert!(result.is_ok(), "Should handle special chars: {}", title);
    }
}

#[test]
fn test_note_favorite_toggle_operations() {
    // Test the common workflow of toggling favorite on/off
    let mut note = Note::new("Important Note", None).unwrap();

    // Initially not favorited
    assert!(!note.is_favorite);

    // Toggle on
    note.mark_favorite();
    assert!(note.is_favorite);

    // Toggle off
    note.unmark_favorite();
    assert!(!note.is_favorite);

    // Toggle on again
    note.mark_favorite();
    assert!(note.is_favorite);
}

#[test]
fn test_note_pin_operations() {
    let mut note = Note::new("Pinned Note", None).unwrap();

    assert!(!note.is_pinned);

    note.pin();
    assert!(note.is_pinned);

    note.unpin();
    assert!(!note.is_pinned);
}
