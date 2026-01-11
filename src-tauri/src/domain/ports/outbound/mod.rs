// Repository ports
pub mod attachment_repository;
pub mod note_link_repository;
pub mod note_repository;
pub mod notebook_repository;
pub mod settings_repository;
pub mod tag_repository;
pub mod topic_repository;
pub mod version_repository;
pub mod workspace_repository;

// Service ports
pub mod embedding_service;
pub mod event_publisher;
pub mod export_service;
pub mod file_storage;
pub mod file_watcher;
pub mod git_service;
pub mod markdown_processor;
pub mod search_engine;
pub mod system_service;

// Repository exports
pub use attachment_repository::AttachmentRepository;
pub use note_link_repository::NoteLinkRepository;
pub use note_repository::{NoteFindOptions, NoteRepository};
pub use notebook_repository::{
    NotebookFindOptions, NotebookPositionUpdate, NotebookRepository, NotebookWithCount,
};
pub use settings_repository::{Setting, SettingsRepository};
pub use tag_repository::{TagRepository, TagWithCount};
pub use topic_repository::{
    FindAllWithCountsOptions, GetNotesForTopicOptions, NoteTopicAssignment,
    NoteTopicWithDetails, TopicNoteRecord, TopicAssignmentOptions, TopicRepository, TopicWithCount,
};
pub use version_repository::{VersionListItem, VersionRepository};
pub use workspace_repository::WorkspaceRepository;

// Service exports
pub use embedding_service::{
    ClassificationResult, EmbeddingResult, EmbeddingService, EmbeddingStatus, SimilarNote,
};
pub use event_publisher::{DomainEvent, EventHandler, EventPublisher, EventTopicClassification, FileSyncOperation};
pub use export_service::{ExportService, HtmlOptions, HtmlTheme, PdfFormat, PdfMargin, PdfOptions};
pub use file_storage::{FileInfo, FileStorage, FileWatchEvent};
pub use file_watcher::FileWatcher;
pub use git_service::{
    GitCommit, GitFileChange, GitFileStatus, GitOperationResult, GitService, GitStatus,
};
pub use markdown_processor::{MarkdownLink, MarkdownMetadata, MarkdownProcessor, ParsedMarkdown};
pub use search_engine::{
    DateRangeField, DateRangeOptions, HybridSearchOptions, SearchEngine, SearchHighlights,
    SearchMatchType, SearchOptions, SearchResult, SearchWeights, SemanticSearchResult,
    TagSearchOptions,
};
pub use system_service::{
    FileFilter, FilePickerOptions, FilePickerResult, FolderPickerOptions, SystemService,
};
