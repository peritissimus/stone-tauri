/// Notebook Use Cases Implementation
///
/// Application layer implementations for notebook operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::Notebook,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{
            CreateNotebookRequest, DeleteNotebookRequest, ListNotebooksRequest, MoveNotebookRequest,
            NotebookList, NotebookUseCases, UpdateNotebookRequest,
        },
        outbound::{EventPublisher, NotebookRepository},
    },
};

/// Implementation of all Notebook use cases
pub struct NotebookUseCasesImpl {
    notebook_repository: Arc<dyn NotebookRepository>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl NotebookUseCasesImpl {
    pub fn new(
        notebook_repository: Arc<dyn NotebookRepository>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            notebook_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl NotebookUseCases for NotebookUseCasesImpl {
    /// Create a new notebook
    async fn create_notebook(
        &self,
        request: CreateNotebookRequest,
    ) -> DomainResult<Notebook> {
        let mut notebook = Notebook::new(
            request.name,
            request.workspace_id,
            request.parent_id,
        )?;

        if let Some(folder_path) = request.folder_path {
            notebook.update_folder_path(Some(folder_path));
        }

        if let Some(icon) = request.icon {
            notebook.change_icon(icon)?;
        }

        if let Some(color) = request.color {
            notebook.change_color(color)?;
        }

        self.notebook_repository.save(&notebook).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "notebook:created",
                serde_json::json!({"id": notebook.id}),
            );
        }

        Ok(notebook)
    }

    /// Update an existing notebook
    async fn update_notebook(
        &self,
        request: UpdateNotebookRequest,
    ) -> DomainResult<Notebook> {
        let mut notebook = self
            .notebook_repository
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| DomainError::NotebookNotFound(request.id.clone()))?;

        // Update name if provided
        if let Some(name) = request.name {
            notebook.rename(name)?;
        }

        // Update parent if provided
        if let Some(parent_id) = request.parent_id {
            notebook.move_to(parent_id)?;
        }

        // Update icon if provided
        if let Some(icon) = request.icon {
            notebook.change_icon(icon)?;
        }

        // Update color if provided
        if let Some(color) = request.color {
            notebook.change_color(color)?;
        }

        self.notebook_repository.save(&notebook).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "notebook:updated",
                serde_json::json!({"id": notebook.id}),
            );
        }

        Ok(notebook)
    }

    /// Get a notebook by ID
    async fn get_notebook(&self, id: &str) -> DomainResult<Notebook> {
        self.notebook_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotebookNotFound(id.to_string()))
    }

    /// List notebooks with optional filtering
    async fn list_notebooks(
        &self,
        request: ListNotebooksRequest,
    ) -> DomainResult<NotebookList> {
        let include_note_count = request.include_note_count.unwrap_or(false);

        if include_note_count {
            // ⚠️ Returns NotebookWithCount, not Notebook!
            let notebooks_with_counts = self
                .notebook_repository
                .find_all_with_counts(request.workspace_id.as_deref())
                .await?;

            return Ok(NotebookList::WithCount(notebooks_with_counts));
        }

        // Without note counts - return regular Notebook entities
        let notebooks = if let Some(parent_id) = request.parent_id {
            // ⚠️ parent_id is Option<Option<String>>
            // If Some(None) -> root notebooks
            // If Some(Some(id)) -> children of that notebook
            self.notebook_repository
                .find_by_parent_id(parent_id.as_deref(), request.workspace_id.as_deref())
                .await?
        } else if let Some(workspace_id) = request.workspace_id {
            self.notebook_repository
                .find_by_workspace_id(&workspace_id)
                .await?
        } else {
            self.notebook_repository.find_all(None).await?
        };

        Ok(NotebookList::WithoutCount(notebooks))
    }

    /// Delete a notebook
    async fn delete_notebook(&self, request: DeleteNotebookRequest) -> DomainResult<()> {
        if !self.notebook_repository.exists(&request.id).await? {
            return Err(DomainError::NotebookNotFound(request.id));
        }

        self.notebook_repository.delete(&request.id).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("notebook:deleted", serde_json::json!({"id": request.id}));
        }

        Ok(())
    }

    /// Move a notebook to a different parent
    async fn move_notebook(&self, request: MoveNotebookRequest) -> DomainResult<()> {
        let mut notebook = self
            .notebook_repository
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| DomainError::NotebookNotFound(request.id.clone()))?;

        notebook.move_to(request.target_parent_id)?;
        self.notebook_repository.save(&notebook).await?;

        // Publish event - ❌ SYNC - NO AWAIT!
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "notebook:updated",
                serde_json::json!({"id": notebook.id}),
            );
        }

        Ok(())
    }
}
