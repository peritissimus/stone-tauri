/// Domain Entity Tests
use stone_tauri_lib::domain::{
    entities::{Attachment, Note, NoteLink, Notebook, Tag, Topic, Version, Workspace},
    errors::DomainError,
};

// ============================================================================
// Note Entity Tests
// ============================================================================

#[test]
fn test_note_create() {
    let note = Note::new("Test Note", Some("workspace-1".to_string())).unwrap();
    assert_eq!(note.title, "Test Note");
    assert_eq!(note.workspace_id, Some("workspace-1".to_string()));
    assert!(!note.is_favorite);
    assert!(!note.is_deleted);
}

#[test]
fn test_note_empty_title_fails() {
    let result = Note::new("", Some("workspace-1".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_note_update_title() {
    let mut note = Note::new("Old Title", None).unwrap();
    note.update_title("New Title").unwrap();
    assert_eq!(note.title, "New Title");
}

#[test]
fn test_note_favorite_operations() {
    let mut note = Note::new("Test", None).unwrap();
    assert!(!note.is_favorite);

    note.mark_favorite();
    assert!(note.is_favorite);

    note.unmark_favorite();
    assert!(!note.is_favorite);
}

#[test]
fn test_note_delete_and_restore() {
    let mut note = Note::new("Test", None).unwrap();
    assert!(!note.is_deleted);
    assert!(note.deleted_at.is_none());

    note.delete();
    assert!(note.is_deleted);
    assert!(note.deleted_at.is_some());

    note.restore();
    assert!(!note.is_deleted);
    assert!(note.deleted_at.is_none());
}

// ============================================================================
// Notebook Entity Tests
// ============================================================================

#[test]
fn test_notebook_create() {
    let notebook = Notebook::new("My Notebook", Some("workspace-1".to_string()), None).unwrap();
    assert_eq!(notebook.name, "My Notebook");
    assert!(notebook.is_root());
}

#[test]
fn test_notebook_rename() {
    let mut notebook = Notebook::new("Old Name", None, None).unwrap();
    notebook.rename("New Name").unwrap();
    assert_eq!(notebook.name, "New Name");
}

// ============================================================================
// Workspace Entity Tests
// ============================================================================

#[test]
fn test_workspace_create() {
    let workspace = Workspace::new("My Workspace", "/path/to/workspace").unwrap();
    assert_eq!(workspace.name, "My Workspace");
    assert!(!workspace.is_active);
}

#[test]
fn test_workspace_activate() {
    let mut workspace = Workspace::new("Test", "/path").unwrap();
    workspace.activate();
    assert!(workspace.is_active);
}

// ============================================================================
// Tag Entity Tests
// ============================================================================

#[test]
fn test_tag_create() {
    let tag = Tag::new("rust programming").unwrap();
    assert_eq!(tag.name, "rust-programming");
}

#[test]
fn test_tag_matches() {
    let tag = Tag::new("rust programming").unwrap();
    assert!(tag.matches("rust"));
}

// ============================================================================
// Attachment Entity Tests
// ============================================================================

#[test]
fn test_attachment_create() {
    let attachment = Attachment::new("note-123", "test.pdf", "application/pdf", 1024, "/path/to/file")
        .unwrap();
    assert_eq!(attachment.filename, "test.pdf");
}

#[test]
fn test_attachment_is_image() {
    let img = Attachment::new("note-123", "test.png", "image/png", 1024, "/path").unwrap();
    assert!(img.is_image());
}

// ============================================================================
// NoteLink Entity Tests
// ============================================================================

#[test]
fn test_note_link_create() {
    let link = NoteLink::new("note-1", "note-2").unwrap();
    assert_eq!(link.source_note_id, "note-1");
    assert_eq!(link.target_note_id, "note-2");
}

#[test]
fn test_note_link_self_link_fails() {
    let result = NoteLink::new("note-1", "note-1");
    assert!(result.is_err());
}

// ============================================================================
// Topic Entity Tests
// ============================================================================

#[test]
fn test_topic_create() {
    let topic = Topic::new("Machine Learning").unwrap();
    assert_eq!(topic.name, "Machine Learning");
    assert!(!topic.is_predefined);
}

#[test]
fn test_topic_note_count() {
    let mut topic = Topic::new("Test").unwrap();
    topic.increment_note_count();
    assert_eq!(topic.note_count, 1);
}

// ============================================================================
// Version Entity Tests
// ============================================================================

#[test]
fn test_version_create() {
    let version = Version::new("note-123", "Test Note", "# Content", 1).unwrap();
    assert_eq!(version.note_id, "note-123");
    assert_eq!(version.version_number, 1);
}

#[test]
fn test_version_is_newer_than() {
    let v1 = Version::new("note-123", "Test", "Content", 1).unwrap();
    let v2 = Version::new("note-123", "Test", "Content", 2).unwrap();
    assert!(v2.is_newer_than(&v1));
}
