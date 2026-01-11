pub mod attachment_usecases;
pub mod database_usecases;
pub mod export_usecases;
pub mod git_usecases;
pub mod graph_usecases;
pub mod note_usecases;
pub mod notebook_usecases;
pub mod quick_capture_usecases;
pub mod search_usecases;
pub mod settings_usecases;
pub mod system_usecases;
pub mod tag_usecases;
pub mod task_usecases;
pub mod topic_usecases;
pub mod version_usecases;
pub mod workspace_usecases;

pub use attachment_usecases::{
    AddAttachmentRequest, AttachmentUseCases, UploadImageRequest, UploadImageResponse,
};
pub use database_usecases::{DatabaseStatus, DatabaseUseCases, IntegrityCheckResult};
pub use export_usecases::{ExportOptions, ExportResult, ExportTheme, ExportUseCases};
pub use git_usecases::{GitCommitInfo, GitStatusResponse, GitSyncResponse, GitUseCases};
pub use graph_usecases::{
    GraphData, GraphDataOptions, GraphLink, GraphNode, GraphUseCases, NoteLinkInfo,
};
pub use note_usecases::{
    CreateNoteInput, NoteFilter, NoteQuery, NoteUseCases, UpdateNoteInput,
};
pub use notebook_usecases::{
    CreateNotebookRequest, DeleteNotebookRequest, ListNotebooksRequest, MoveNotebookRequest,
    NotebookList, NotebookUseCases, UpdateNotebookRequest,
};
pub use quick_capture_usecases::{AppendToJournalResponse, QuickCaptureUseCases};
pub use search_usecases::{
    HybridSearchResultItem, HybridSearchWeights, SearchByDateRangeRequest, SearchRequest,
    SearchType, SearchUseCases, VectorSearchResult,
};
pub use settings_usecases::SettingsUseCases;
pub use system_usecases::{SelectFolderOptions, SystemUseCases};
pub use tag_usecases::{
    CreateTagRequest, ListTagsRequest, TagList, TagUseCases, UpdateTagRequest,
};
pub use task_usecases::{TaskItem, TaskUseCases};
pub use topic_usecases::{
    ClassifyAllResponse, ClassifyNoteResponse, CreateTopicRequest, EmbeddingStatusResponse,
    NoteTopicInfo, SimilarNoteResult, TopicClassification, TopicUseCases, UpdateTopicRequest,
};
pub use version_usecases::VersionUseCases;
pub use workspace_usecases::{
    CreateFolderRequest, CreateFolderResponse, CreateWorkspaceRequest, FileSystemEntryType,
    MoveFolderRequest, MoveFolderResponse, RenameFolderRequest, RenameFolderResponse,
    ScanWorkspaceFileEntry, ScanWorkspaceFolderStructure, ScanWorkspaceResponse,
    SelectFolderRequest, SelectFolderResponse, SyncWorkspaceResponse, SyncWorkspaceStats,
    UpdateWorkspaceRequest, ValidatePathResponse, WorkspaceUseCases,
};
