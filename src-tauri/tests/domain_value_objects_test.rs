/// Value Objects Tests
use stone_tauri_lib::domain::value_objects::{FilePath, HexColor, NotebookId, NoteId, WorkspaceId};

#[test]
fn test_note_id_create() {
    let id = NoteId::new("note-123").unwrap();
    assert_eq!(id.as_str(), "note-123");
}

#[test]
fn test_note_id_empty_fails() {
    assert!(NoteId::new("").is_err());
}

#[test]
fn test_file_path_create() {
    let path = FilePath::new("/path/to/file.md").unwrap();
    assert_eq!(path.as_str(), "/path/to/file.md");
}

#[test]
fn test_file_path_empty_fails() {
    assert!(FilePath::new("").is_err());
}

#[test]
fn test_hex_color_create() {
    let color = HexColor::new("#3b82f6").unwrap();
    assert_eq!(color.as_str(), "#3B82F6");
}

#[test]
fn test_hex_color_invalid_fails() {
    assert!(HexColor::new("3b82f6").is_err());
    assert!(HexColor::new("#fff").is_err());
    assert!(HexColor::new("#gggggg").is_err());
}
