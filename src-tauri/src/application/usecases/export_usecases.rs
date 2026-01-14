/// Export Use Cases Implementation
///
/// Application layer implementations for note export operations.
/// Supports exporting notes to HTML, PDF, and Markdown formats.
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{ExportOptions, ExportResult, ExportTheme, ExportUseCases},
        outbound::{
            ExportService, FileStorage, HtmlOptions, HtmlTheme, MarkdownProcessor, NoteRepository,
            PdfFormat, PdfOptions, WorkspaceRepository,
        },
    },
};

/// Implementation of all Export use cases
pub struct ExportUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
    markdown_processor: Arc<dyn MarkdownProcessor>,
    export_service: Arc<dyn ExportService>,
}

impl ExportUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
        markdown_processor: Arc<dyn MarkdownProcessor>,
        export_service: Arc<dyn ExportService>,
    ) -> Self {
        Self {
            note_repository,
            workspace_repository,
            file_storage,
            markdown_processor,
            export_service,
        }
    }

    /// Convert ExportTheme to HtmlTheme
    fn theme_to_html_theme(theme: ExportTheme) -> HtmlTheme {
        match theme {
            ExportTheme::Light => HtmlTheme::Light,
            ExportTheme::Dark => HtmlTheme::Dark,
        }
    }
}

#[async_trait]
impl ExportUseCases for ExportUseCasesImpl {
    /// Export note as HTML
    async fn export_html(
        &self,
        note_id: &str,
        rendered_html: Option<String>,
        title: Option<String>,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult> {
        // Find the note
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // If rendered HTML is provided, use it directly (wrapping it in a document)
        let html_content = if let Some(html) = rendered_html {
            html
        } else {
            // Validate required fields
            let file_path = note
                .file_path
                .as_ref()
                .ok_or_else(|| DomainError::ValidationError("Note has no file path".to_string()))?;

            let workspace_id = note
                .workspace_id
                .as_ref()
                .ok_or_else(|| DomainError::ValidationError("Note has no workspace ID".to_string()))?;

            // Find the workspace
            let workspace = self
                .workspace_repository
                .find_by_id(workspace_id)
                .await?
                .ok_or_else(|| DomainError::WorkspaceNotFound(workspace_id.to_string()))?;

            // Construct absolute path
            let absolute_path = Path::new(&workspace.folder_path)
                .join(file_path)
                .to_string_lossy()
                .to_string();

            // Read file content
            let markdown = self
                .file_storage
                .read(&absolute_path)
                .await?
                .ok_or_else(|| DomainError::ValidationError("Could not read note content".to_string()))?;

            // Convert markdown to HTML
            self.markdown_processor.markdown_to_html(&markdown).await?
        };

        // Determine theme
        let theme = options
            .as_ref()
            .and_then(|o| o.theme)
            .map(Self::theme_to_html_theme)
            .unwrap_or(HtmlTheme::Light);

        // Generate full HTML document
        let html_options = HtmlOptions {
            title: title.or(Some(note.title.clone())),
            theme: Some(theme),
            include_styles: Some(true),
            custom_css: None,
        };

        let full_html = self
            .export_service
            .generate_html_document(&html_content, Some(html_options))?;

        // Generate filename
        let filename = format!(
            "{}.html",
            note.title.as_str()
        );

        tracing::info!("Exported note {} to HTML", note_id);

        Ok(ExportResult {
            content: full_html.into_bytes(),
            filename,
            mime_type: "text/html".to_string(),
        })
    }

    /// Export note as PDF
    async fn export_pdf(
        &self,
        note_id: &str,
        rendered_html: Option<String>,
        title: Option<String>,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult> {
        // Check if PDF export is available
        if !self.export_service.is_pdf_available() {
            return Err(DomainError::ValidationError(
                "PDF export is not available".to_string(),
            ));
        }

        // Find the note
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // If rendered HTML is provided, use it directly (wrapping it in a document)
        let html_content = if let Some(html) = rendered_html {
            html
        } else {
            // Validate required fields
            let file_path = note
                .file_path
                .as_ref()
                .ok_or_else(|| DomainError::ValidationError("Note has no file path".to_string()))?;

            let workspace_id = note
                .workspace_id
                .as_ref()
                .ok_or_else(|| DomainError::ValidationError("Note has no workspace ID".to_string()))?;

            // Find the workspace
            let workspace = self
                .workspace_repository
                .find_by_id(workspace_id)
                .await?
                .ok_or_else(|| DomainError::WorkspaceNotFound(workspace_id.to_string()))?;

            // Construct absolute path
            let absolute_path = Path::new(&workspace.folder_path)
                .join(file_path)
                .to_string_lossy()
                .to_string();

            // Read file content
            let markdown = self
                .file_storage
                .read(&absolute_path)
                .await?
                .ok_or_else(|| DomainError::ValidationError("Could not read note content".to_string()))?;

            // Convert markdown to HTML
            self.markdown_processor.markdown_to_html(&markdown).await?
        };

        // Determine theme
        let theme = options
            .as_ref()
            .and_then(|o| o.theme)
            .map(Self::theme_to_html_theme)
            .unwrap_or(HtmlTheme::Light);

        // Generate full HTML document
        let html_options = HtmlOptions {
            title: title.or(Some(note.title.clone())),
            theme: Some(theme),
            include_styles: Some(true),
            custom_css: None,
        };

        let full_html = self
            .export_service
            .generate_html_document(&html_content, Some(html_options))?;

        // Render to PDF
        let pdf_options = PdfOptions {
            format: Some(PdfFormat::A4),
            margin: None,
            landscape: None,
            print_background: Some(true),
        };

        let pdf_buffer = self
            .export_service
            .render_to_pdf(&full_html, Some(pdf_options))
            .await?;

        // Generate filename
        let filename = format!(
            "{}.pdf",
            note.title.as_str()
        );

        tracing::info!("Exported note {} to PDF", note_id);

        Ok(ExportResult {
            content: pdf_buffer,
            filename,
            mime_type: "application/pdf".to_string(),
        })
    }

    /// Export note as Markdown
    async fn export_markdown(
        &self,
        note_id: &str,
        options: Option<ExportOptions>,
    ) -> DomainResult<ExportResult> {
        // Find the note
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // Validate required fields
        let file_path = note
            .file_path
            .as_ref()
            .ok_or_else(|| DomainError::ValidationError("Note has no file path".to_string()))?;

        let workspace_id = note
            .workspace_id
            .as_ref()
            .ok_or_else(|| DomainError::ValidationError("Note has no workspace ID".to_string()))?;

        // Find the workspace
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| DomainError::WorkspaceNotFound(workspace_id.to_string()))?;

        // Construct absolute path
        let absolute_path = Path::new(&workspace.folder_path)
            .join(file_path)
            .to_string_lossy()
            .to_string();

        // Read file content
        let mut markdown = self
            .file_storage
            .read(&absolute_path)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Could not read note content".to_string()))?;

        // Optionally add frontmatter
        if options
            .as_ref()
            .and_then(|o| o.include_frontmatter)
            .unwrap_or(false)
        {
            let frontmatter = format!(
                "---\ntitle: \"{}\"\ncreated: {}\nupdated: {}\n---\n\n",
                note.title.as_str(),
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339()
            );
            markdown = frontmatter + &markdown;
        }

        // Generate filename
        let filename = format!(
            "{}.md",
            note.title.as_str()
        );

        tracing::info!("Exported note {} to Markdown", note_id);

        Ok(ExportResult {
            content: markdown.into_bytes(),
            filename,
            mime_type: "text/markdown".to_string(),
        })
    }
}
