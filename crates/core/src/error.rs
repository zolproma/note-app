use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("note not found: {0}")]
    NoteNotFound(String),

    #[error("notebook not found: {0}")]
    NotebookNotFound(String),

    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("tag not found: {0}")]
    TagNotFound(String),

    #[error("invalid document model: {0}")]
    InvalidModel(String),

    #[error("AI policy violation: {0}")]
    AiPolicyViolation(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}
