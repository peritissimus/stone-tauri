//! Test Bidirectional Note Linking Workflow
//!
//! Critical Path: Create wiki links → Track relationships → Query backlinks
//! This enables knowledge graph and note connections

use stone_tauri_lib::domain::entities::NoteLink;
use stone_tauri_lib::domain::services::LinkExtractor;

#[test]
fn test_note_link_creation() {
    // User creates link between two notes
    let source_id = "note-abc123";
    let target_id = "note-def456";

    let link = NoteLink::new(source_id, target_id)
        .expect("Should create link between different notes");

    assert_eq!(link.source_note_id, source_id);
    assert_eq!(link.target_note_id, target_id);
}

#[test]
fn test_self_link_prevention() {
    // Prevent circular self-reference
    let note_id = "note-123";
    let result = NoteLink::new(note_id, note_id);

    assert!(result.is_err(), "Should prevent self-linking");
}

#[test]
fn test_wiki_link_extraction_basic() {
    // Extract wiki-style links from markdown: [[Note Title]]
    let markdown = "Check out [[Project Ideas]] and [[Meeting Notes]]";

    let links = LinkExtractor::extract_wiki_links(markdown);

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].target, "Project Ideas");
    assert_eq!(links[1].target, "Meeting Notes");
}

#[test]
fn test_wiki_link_with_custom_text() {
    // Wiki links can have custom display text: [[Target|Display Text]]
    let markdown = "See [[Technical Spec|the spec]] for details";

    let links = LinkExtractor::extract_wiki_links(markdown);

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target, "Technical Spec");
    assert_eq!(links[0].text, "the spec");
}

#[test]
fn test_wiki_link_detection() {
    // Check if note contains link to specific target
    let markdown = r#"
# Project Overview

Important references:
- [[Architecture Design]]
- [[API Spec]]
- [[Database Schema]]
    "#;

    assert!(LinkExtractor::has_link_to(markdown, "Architecture Design"));
    assert!(LinkExtractor::has_link_to(markdown, "API Spec"));
    assert!(!LinkExtractor::has_link_to(markdown, "Nonexistent Note"));
}

#[test]
fn test_case_insensitive_link_matching() {
    // Link matching should be case-insensitive
    let markdown = "See [[Project Ideas]] for more";

    assert!(LinkExtractor::has_link_to(markdown, "Project Ideas"));
    assert!(LinkExtractor::has_link_to(markdown, "project ideas"));
    assert!(LinkExtractor::has_link_to(markdown, "PROJECT IDEAS"));
}

#[test]
fn test_multiple_links_in_complex_markdown() {
    let markdown = r#"
# Weekly Standup

## Monday
Met with [[John Smith|John]] about [[Q4 Planning]].
Need to review [[Budget Proposal]].

## Tuesday
Worked on [[Feature X]] implementation.
Referenced [[Technical Spec]] and [[API Design]].

## Follow-up
- [ ] Review [[Code Review Comments]]
- [ ] Update [[Project Timeline]]
    "#;

    let links = LinkExtractor::extract_wiki_links(markdown);

    // Should find all unique wiki links
    assert!(links.len() >= 8, "Should extract all wiki links");

    // Verify custom text link
    let john_link = links.iter().find(|l| l.target == "John Smith");
    assert!(john_link.is_some());
    assert_eq!(john_link.unwrap().text, "John");
}

#[test]
fn test_empty_link_handling() {
    // Edge case: empty or malformed links
    let test_cases = vec![
        "[[]]",           // Empty link
        "[[  ]]",         // Whitespace only
        "[] []",          // Not a wiki link
        "[[ ]]",          // Incomplete
    ];

    for markdown in test_cases {
        let links = LinkExtractor::extract_wiki_links(markdown);
        // Should either find nothing or handle gracefully
        // (exact behavior depends on implementation)
        assert!(links.is_empty() || links[0].target.trim().is_empty());
    }
}
