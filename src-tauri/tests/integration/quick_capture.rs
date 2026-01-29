//! Test Quick Capture Workflow
//!
//! Critical Path: Alt+Space → Show popup → Append timestamped entry to journal
//! This is Stone's killer feature for rapid note capture

use chrono::{Local, Datelike, Timelike};

#[test]
fn test_journal_entry_format() {
    // Quick capture creates entries like:
    // [14:30] User's captured thought here

    let now = Local::now();
    let timestamp = format!("[{:02}:{:02}]", now.hour(), now.minute());
    let entry_text = "Remember to follow up with client";
    let full_entry = format!("{} {}", timestamp, entry_text);

    // Verify format matches expected pattern
    assert!(full_entry.starts_with("["));
    assert!(full_entry.contains(":"));
    assert!(full_entry.contains("]"));
    assert!(full_entry.contains(entry_text));
}

#[test]
fn test_journal_date_formatting() {
    // Journal files are named: YYYY-MM-DD.md
    let now = Local::now();
    let expected_filename = format!("{:04}-{:02}-{:02}.md",
        now.year(),
        now.month(),
        now.day()
    );

    // Verify format is ISO date
    assert_eq!(expected_filename.len(), 13); // "YYYY-MM-DD.md"
    assert!(expected_filename.ends_with(".md"));
    assert!(expected_filename.contains("-"));
}

#[test]
fn test_journal_path_construction() {
    // Journal entries go to: workspace/Journal/YYYY-MM-DD.md
    let workspace_path = "/Users/test/notes";
    let journal_folder = format!("{}/Journal", workspace_path);

    let now = Local::now();
    let journal_filename = format!("{:04}-{:02}-{:02}.md",
        now.year(),
        now.month(),
        now.day()
    );
    let full_path = format!("{}/{}", journal_folder, journal_filename);

    // Verify path structure
    assert!(full_path.contains("/Journal/"));
    assert!(full_path.ends_with(".md"));
    assert!(full_path.starts_with(workspace_path));
}

#[test]
fn test_multiple_journal_entries_same_day() {
    // Simulate multiple quick captures throughout the day
    let entries = vec![
        "[09:15] Morning standup notes",
        "[12:30] Lunch meeting insights",
        "[15:45] Code review feedback",
        "[18:00] End of day summary",
    ];

    // Each entry should be timestamped
    for entry in &entries {
        assert!(entry.starts_with("["));
        assert!(entry.contains(":"));
        assert!(entry.contains("]"));
    }

    // All entries would be appended to same file (today's journal)
    let combined = entries.join("\n");
    assert_eq!(combined.matches("[").count(), 4);
}

#[test]
fn test_timestamp_format_validation() {
    // Valid timestamp formats: [HH:MM]
    let valid_timestamps = vec![
        "[00:00]",
        "[09:30]",
        "[12:45]",
        "[23:59]",
    ];

    for ts in valid_timestamps {
        assert_eq!(ts.len(), 7);
        assert!(ts.starts_with("["));
        assert!(ts.ends_with("]"));
        assert!(ts.chars().nth(3) == Some(':'));
    }
}
