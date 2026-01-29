//! Test Search Workflows
//!
//! Critical Path: Search notes by content, tags, dates
//! Stone supports full-text, semantic, and hybrid search

use stone_tauri_lib::domain::services::{cosine_similarity, euclidean_distance, TaskExtractor, TaskState};

#[test]
fn test_task_extraction_from_markdown() {
    // Extract tasks from markdown using Logseq-style syntax
    let markdown = r#"
- TODO Write integration tests
- DOING Implement search feature
- DONE Review pull request
- WAITING Response from client
- CANCELED Old feature idea
    "#;

    let tasks = TaskExtractor::extract_tasks(markdown);

    assert_eq!(tasks.len(), 5);
    assert_eq!(tasks[0].state, TaskState::Todo);
    assert_eq!(tasks[0].text, "Write integration tests");
    assert_eq!(tasks[1].state, TaskState::Doing);
    assert_eq!(tasks[2].state, TaskState::Done);
    assert_eq!(tasks[3].state, TaskState::Waiting);
    assert_eq!(tasks[4].state, TaskState::Canceled);
}

#[test]
fn test_task_state_completion_check() {
    // Check which states are considered "completed"
    assert!(TaskState::Done.is_completed());
    assert!(TaskState::Canceled.is_completed());

    assert!(!TaskState::Todo.is_completed());
    assert!(!TaskState::Doing.is_completed());
    assert!(!TaskState::Waiting.is_completed());
}

#[test]
fn test_task_extraction_mixed_content() {
    // Tasks mixed with regular content
    let markdown = r#"
# Project Notes

Some regular text here.

## Tasks
- TODO Finish documentation
- Regular bullet point (not a task)
- DONE Complete implementation

More content...
    "#;

    let tasks = TaskExtractor::extract_tasks(markdown);

    // Should only extract actual task items
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().any(|t| t.text.contains("documentation")));
    assert!(tasks.iter().any(|t| t.text.contains("implementation")));
}

// ============================================================================
// Vector Similarity (for Semantic Search)
// ============================================================================

#[test]
fn test_cosine_similarity_identical_vectors() {
    // Identical vectors should have similarity of 1.0
    let vec_a = vec![1.0, 2.0, 3.0, 4.0];
    let vec_b = vec![1.0, 2.0, 3.0, 4.0];

    let similarity = cosine_similarity(&vec_a, &vec_b)
        .expect("Should calculate similarity");

    assert!((similarity - 1.0).abs() < 0.0001);
}

#[test]
fn test_cosine_similarity_orthogonal_vectors() {
    // Orthogonal vectors (perpendicular) should have similarity of 0.0
    let vec_a = vec![1.0, 0.0, 0.0];
    let vec_b = vec![0.0, 1.0, 0.0];

    let similarity = cosine_similarity(&vec_a, &vec_b)
        .expect("Should calculate similarity");

    assert!((similarity - 0.0).abs() < 0.0001);
}

#[test]
fn test_cosine_similarity_opposite_vectors() {
    // Opposite vectors should have similarity of -1.0
    let vec_a = vec![1.0, 2.0, 3.0];
    let vec_b = vec![-1.0, -2.0, -3.0];

    let similarity = cosine_similarity(&vec_a, &vec_b)
        .expect("Should calculate similarity");

    assert!((similarity - (-1.0)).abs() < 0.0001);
}

#[test]
fn test_euclidean_distance() {
    // Test distance calculation (Pythagorean theorem)
    // Distance from (0,0) to (3,4) should be 5
    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![3.0, 4.0];

    let distance = euclidean_distance(&vec_a, &vec_b)
        .expect("Should calculate distance");

    assert!((distance - 5.0).abs() < 0.0001);
}

#[test]
fn test_euclidean_distance_same_point() {
    // Distance to same point should be 0
    let vec_a = vec![1.0, 2.0, 3.0];
    let vec_b = vec![1.0, 2.0, 3.0];

    let distance = euclidean_distance(&vec_a, &vec_b)
        .expect("Should calculate distance");

    assert!((distance - 0.0).abs() < 0.0001);
}

#[test]
fn test_similarity_with_different_lengths_fails() {
    // Vectors of different lengths should return error
    let vec_a = vec![1.0, 2.0];
    let vec_b = vec![1.0, 2.0, 3.0];

    let result = cosine_similarity(&vec_a, &vec_b);
    assert!(result.is_err(), "Should fail on mismatched vector lengths");
}

#[test]
fn test_semantic_search_scenario() {
    // Simulate finding similar notes based on embeddings
    // Note: In real app, embeddings come from FastEmbed model

    // "Machine Learning" note embedding (simulated)
    let ml_note = vec![0.8, 0.6, 0.1, 0.2];

    // "Neural Networks" note (similar topic)
    let nn_note = vec![0.7, 0.5, 0.15, 0.25];

    // "Cooking Recipes" note (different topic)
    let recipe_note = vec![0.1, 0.2, 0.9, 0.8];

    let ml_to_nn = cosine_similarity(&ml_note, &nn_note).unwrap();
    let ml_to_recipe = cosine_similarity(&ml_note, &recipe_note).unwrap();

    // Similar notes should have higher similarity
    assert!(ml_to_nn > ml_to_recipe,
        "Related notes should be more similar than unrelated ones");
}
