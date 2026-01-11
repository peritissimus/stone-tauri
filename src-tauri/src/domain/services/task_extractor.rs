/// TaskExtractor - Pure domain service for extracting tasks from markdown
///
/// Handles Logseq-style task patterns without any I/O operations.
/// This is pure business logic that can be tested without mocks.
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Supported task states (Logseq-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Todo,
    Doing,
    Done,
    Waiting,
    Hold,
    Canceled,
    Idea,
}

impl TaskState {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "todo" => Some(Self::Todo),
            "doing" => Some(Self::Doing),
            "done" => Some(Self::Done),
            "waiting" => Some(Self::Waiting),
            "hold" => Some(Self::Hold),
            "canceled" | "cancelled" => Some(Self::Canceled),
            "idea" => Some(Self::Idea),
            _ => None,
        }
    }

    /// Convert to uppercase string for markdown
    pub fn to_uppercase_str(&self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Doing => "DOING",
            Self::Done => "DONE",
            Self::Waiting => "WAITING",
            Self::Hold => "HOLD",
            Self::Canceled => "CANCELED",
            Self::Idea => "IDEA",
        }
    }

    /// Check if state represents a completed task
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Done | Self::Canceled)
    }
}

/// Raw task extracted from markdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTask {
    pub index: usize,
    pub state: TaskState,
    pub text: String,
    pub line_number: usize,
    pub checked: bool,
}

/// TaskExtractor - Pure functions for task extraction
pub struct TaskExtractor;

impl TaskExtractor {
    /// Extract all tasks from markdown content
    pub fn extract_tasks(markdown: &str) -> Vec<RawTask> {
        let mut tasks = Vec::new();
        let pattern = Regex::new(
            r"(?m)^(\s*)(?:[-*]\s+)?(TODO|DOING|DONE|WAITING|HOLD|CANCELED|CANCELLED|IDEA)\s+(.+)$"
        ).unwrap();

        let mut task_index = 0;

        for (line_num, line) in markdown.lines().enumerate() {
            if let Some(captures) = pattern.captures(line) {
                if let Some(state_str) = captures.get(2) {
                    if let Some(state) = TaskState::from_str(state_str.as_str()) {
                        if let Some(text_match) = captures.get(3) {
                            let text = text_match.as_str().trim().to_string();

                            if !text.is_empty() {
                                tasks.push(RawTask {
                                    index: task_index,
                                    state,
                                    text,
                                    line_number: line_num + 1, // 1-based
                                    checked: state.is_completed(),
                                });
                                task_index += 1;
                            }
                        }
                    }
                }
            }
        }

        tasks
    }

    /// Replace a task's state at a specific index
    /// Returns the updated markdown content
    pub fn replace_task_state(
        markdown: &str,
        task_index: usize,
        new_state: TaskState,
    ) -> Result<String, String> {
        let pattern = Regex::new(
            r"(?m)^(\s*)(?:([-*]\s+))?(TODO|DOING|DONE|WAITING|HOLD|CANCELED|CANCELLED|IDEA)\s+(.+)$"
        ).unwrap();

        let mut current_index = 0;
        let mut found = false;

        let result = pattern.replace_all(markdown, |caps: &regex::Captures| {
            if current_index == task_index {
                found = true;
                let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let list_marker = caps.get(2).map(|m| m.as_str()).unwrap_or("- ");
                let text = caps.get(4).map(|m| m.as_str()).unwrap_or("");
                current_index += 1;
                format!("{}{}{} {}", indent, list_marker, new_state.to_uppercase_str(), text)
            } else {
                current_index += 1;
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }
        });

        if !found {
            return Err(format!("Task at index {} not found", task_index));
        }

        Ok(result.to_string())
    }

    /// Get count of tasks by state
    pub fn count_by_state(tasks: &[RawTask]) -> std::collections::HashMap<TaskState, usize> {
        let mut counts = std::collections::HashMap::new();

        for task in tasks {
            *counts.entry(task.state).or_insert(0) += 1;
        }

        counts
    }

    /// Get all tasks in a specific state
    pub fn filter_by_state(tasks: &[RawTask], state: TaskState) -> Vec<&RawTask> {
        tasks.iter().filter(|t| t.state == state).collect()
    }
}

