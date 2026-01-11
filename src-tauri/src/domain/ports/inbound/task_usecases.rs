use crate::domain::{errors::DomainResult, services::TaskState};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub note_id: String,
    pub note_title: Option<String>,
    pub note_path: Option<String>,
    pub text: String,
    pub state: TaskState,
    pub checked: bool,
    pub line_number: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Task Use Cases Port (Inbound)
///
/// Defines the contract for task operations.
#[async_trait]
pub trait TaskUseCases: Send + Sync {
    /// Get all tasks from all notes
    async fn get_all_tasks(&self) -> DomainResult<Vec<TaskItem>>;

    /// Get tasks for a specific note
    async fn get_note_tasks(&self, note_id: &str) -> DomainResult<Vec<TaskItem>>;

    /// Update a task's state
    async fn update_task_state(
        &self,
        note_id: &str,
        task_index: i32,
        new_state: TaskState,
    ) -> DomainResult<()>;

    /// Toggle a task between TODO and DONE
    async fn toggle_task(&self, note_id: &str, task_index: i32) -> DomainResult<()>;
}
