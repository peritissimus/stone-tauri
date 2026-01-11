/// Task Use Cases Implementation
///
/// Application layer implementations for task operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{TaskItem, TaskUseCases},
        outbound::{note_repository::NoteFindOptions, FileStorage, NoteRepository, WorkspaceRepository},
    },
    services::{TaskExtractor, TaskState},
};

/// Implementation of all Task use cases
pub struct TaskUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
}

impl TaskUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
    ) -> Self {
        Self {
            note_repository,
            workspace_repository,
            file_storage,
        }
    }
}

#[async_trait]
impl TaskUseCases for TaskUseCasesImpl {
    /// Get all tasks from all notes
    async fn get_all_tasks(&self) -> DomainResult<Vec<TaskItem>> {
        use std::path::Path;

        // Get active workspace - ✅ ASYNC
        let active_workspace = self.workspace_repository.find_active().await?;

        if active_workspace.is_none() {
            return Ok(vec![]);
        }

        let active_workspace = active_workspace.unwrap();

        // Get non-deleted notes for active workspace - ✅ ASYNC
        let notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(active_workspace.id.clone()),
                notebook_id: None,
                is_favorite: None,
                is_pinned: None,
                is_archived: None,
                is_deleted: Some(false),
                limit: None,
                offset: None,
            })
            .await?;

        let mut tasks: Vec<TaskItem> = Vec::new();

        // Scan each note for tasks
        for note in notes {
            // Skip notes without file path or workspace
            if note.file_path.is_none() || note.workspace_id.is_none() {
                continue;
            }

            let file_path = note.file_path.as_ref().unwrap();
            let workspace_id = note.workspace_id.as_ref().unwrap();

            // Get workspace - ✅ ASYNC
            let workspace = match self.workspace_repository.find_by_id(workspace_id).await {
                Ok(Some(ws)) => ws,
                _ => continue,
            };

            // Build absolute path
            let absolute_path = Path::new(&workspace.folder_path)
                .join(file_path)
                .to_string_lossy()
                .to_string();

            // Read markdown - ✅ ASYNC
            let markdown = match self.file_storage.read(&absolute_path).await {
                Ok(Some(content)) => content,
                _ => continue,
            };

            // Extract tasks using domain service - ❌ SYNC (pure function)
            let raw_tasks = TaskExtractor::extract_tasks(&markdown);

            // Map to TaskItems
            for raw_task in raw_tasks {
                tasks.push(TaskItem {
                    id: format!("{}-{}", note.id, raw_task.index),
                    note_id: note.id.clone(),
                    note_title: Some(note.title.clone()),
                    note_path: note.file_path.clone(),
                    text: raw_task.text,
                    state: raw_task.state,
                    checked: raw_task.checked,
                    line_number: raw_task.line_number as i32,
                    created_at: note.created_at,
                    updated_at: note.updated_at,
                });
            }
        }

        Ok(tasks)
    }

    /// Get tasks for a specific note
    async fn get_note_tasks(&self, note_id: &str) -> DomainResult<Vec<TaskItem>> {
        use std::path::Path;

        // Get note - ✅ ASYNC
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", note_id))
            })?;

        // Check if note has file path and workspace
        if note.file_path.is_none() || note.workspace_id.is_none() {
            return Ok(vec![]);
        }

        let file_path = note.file_path.as_ref().unwrap();
        let workspace_id = note.workspace_id.as_ref().unwrap();

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Build absolute path
        let absolute_path = Path::new(&workspace.folder_path)
            .join(file_path)
            .to_string_lossy()
            .to_string();

        // Read markdown - ✅ ASYNC
        let markdown = self.file_storage.read(&absolute_path).await?;

        if markdown.is_none() {
            return Ok(vec![]);
        }

        let markdown = markdown.unwrap();

        // Extract tasks using domain service - ❌ SYNC (pure function)
        let raw_tasks = TaskExtractor::extract_tasks(&markdown);

        // Map to TaskItems
        let tasks = raw_tasks
            .into_iter()
            .map(|raw_task| TaskItem {
                id: format!("{}-{}", note.id, raw_task.index),
                note_id: note.id.clone(),
                note_title: Some(note.title.clone()),
                note_path: note.file_path.clone(),
                text: raw_task.text,
                state: raw_task.state,
                checked: raw_task.checked,
                line_number: raw_task.line_number as i32,
                created_at: note.created_at,
                updated_at: note.updated_at,
            })
            .collect();

        Ok(tasks)
    }

    /// Update a task's state
    async fn update_task_state(
        &self,
        note_id: &str,
        task_index: i32,
        new_state: TaskState,
    ) -> DomainResult<()> {
        use std::path::Path;

        // Get note - ✅ ASYNC
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", note_id))
            })?;

        // Check if note has file path and workspace
        if note.file_path.is_none() || note.workspace_id.is_none() {
            return Err(DomainError::ValidationError(
                "Note has no file path".to_string(),
            ));
        }

        let file_path = note.file_path.as_ref().unwrap();
        let workspace_id = note.workspace_id.as_ref().unwrap();

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Build absolute path
        let absolute_path = Path::new(&workspace.folder_path)
            .join(file_path)
            .to_string_lossy()
            .to_string();

        // Read markdown - ✅ ASYNC
        let markdown = self
            .file_storage
            .read(&absolute_path)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError("Could not read note content".to_string())
            })?;

        // Use domain service to replace task state - ❌ SYNC (pure function)
        let updated_markdown =
            TaskExtractor::replace_task_state(&markdown, task_index as usize, new_state)
                .map_err(|e| DomainError::ValidationError(e))?;

        // Preserve title heading when writing
        let title_heading = format!("# {}\n\n", note.title);
        let content_without_title = updated_markdown
            .lines()
            .skip_while(|line| line.trim().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let content_with_title = format!("{}{}", title_heading, content_without_title.trim_start());

        // Write back to file - ✅ ASYNC
        self.file_storage
            .write(&absolute_path, &content_with_title)
            .await?;

        // Update note timestamp - ✅ ASYNC
        self.note_repository.save(&note).await?;

        Ok(())
    }

    /// Toggle a task between TODO and DONE
    async fn toggle_task(&self, note_id: &str, task_index: i32) -> DomainResult<()> {
        // Get current tasks - ✅ ASYNC
        let tasks = self.get_note_tasks(note_id).await?;

        // Find the specific task
        let task = tasks
            .iter()
            .find(|t| t.id == format!("{}-{}", note_id, task_index))
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Task at index {} not found", task_index))
            })?;

        // Toggle state
        let new_state = if task.state == TaskState::Done {
            TaskState::Todo
        } else {
            TaskState::Done
        };

        // Update task state - ✅ ASYNC
        self.update_task_state(note_id, task_index, new_state)
            .await?;

        Ok(())
    }
}
