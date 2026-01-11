/// Domain Services Tests
use stone_tauri_lib::domain::services::{
    cosine_similarity, euclidean_distance, LinkExtractor, TaskExtractor, TaskState,
};

// ============================================================================
// TaskExtractor Tests
// ============================================================================

#[test]
fn test_task_extractor_extract_tasks() {
    let markdown = r#"
- TODO Write tests
- DOING Implement feature
- DONE Complete documentation
"#;

    let tasks = TaskExtractor::extract_tasks(markdown);
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].text, "Write tests");
    assert_eq!(tasks[0].state, TaskState::Todo);
    assert_eq!(tasks[1].state, TaskState::Doing);
    assert_eq!(tasks[2].state, TaskState::Done);
}

#[test]
fn test_task_state_is_completed() {
    assert!(TaskState::Done.is_completed());
    assert!(TaskState::Canceled.is_completed());
    assert!(!TaskState::Todo.is_completed());
}

// ============================================================================
// LinkExtractor Tests
// ============================================================================

#[test]
fn test_link_extractor_extract_wiki_links() {
    let markdown = "Here is a [[Note Title]] and another [[Second Note|Custom Text]].";

    let links = LinkExtractor::extract_wiki_links(markdown);
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].target, "Note Title");
    assert_eq!(links[1].target, "Second Note");
    assert_eq!(links[1].text, "Custom Text");
}

#[test]
fn test_link_extractor_has_link_to() {
    let markdown = "See [[My Note]] for details";
    assert!(LinkExtractor::has_link_to(markdown, "My Note"));
    assert!(LinkExtractor::has_link_to(markdown, "my note"));
    assert!(!LinkExtractor::has_link_to(markdown, "Other Note"));
}

// ============================================================================
// SimilarityCalculator Tests
// ============================================================================

#[test]
fn test_cosine_similarity_identical() {
    let vec_a = vec![1.0, 2.0, 3.0];
    let vec_b = vec![1.0, 2.0, 3.0];
    let similarity = cosine_similarity(&vec_a, &vec_b).unwrap();
    assert!((similarity - 1.0).abs() < 0.0001);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    let vec_a = vec![1.0, 0.0, 0.0];
    let vec_b = vec![0.0, 1.0, 0.0];
    let similarity = cosine_similarity(&vec_a, &vec_b).unwrap();
    assert!((similarity - 0.0).abs() < 0.0001);
}

#[test]
fn test_euclidean_distance() {
    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![3.0, 4.0];
    let distance = euclidean_distance(&vec_a, &vec_b).unwrap();
    assert!((distance - 5.0).abs() < 0.0001);
}
