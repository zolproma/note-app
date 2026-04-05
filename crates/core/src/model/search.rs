use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filters for advanced note search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Full-text search query (FTS5 MATCH)
    pub query: Option<String>,
    /// Filter by tag names
    pub tags: Vec<String>,
    /// Filter by notebook ID
    pub notebook_id: Option<Uuid>,
    /// Filter by lifecycle state
    pub lifecycle: Option<String>,
    /// Filter by pinned status
    pub pinned: Option<bool>,
}

/// A search result with optional snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub note_id: Uuid,
    pub title: String,
    pub lifecycle: String,
    pub notebook_id: Uuid,
    pub snippet: String,
    pub pinned: bool,
    pub updated_at: String,
}

/// A saved search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub filter: SearchFilter,
    pub created_at: String,
}

impl SavedSearch {
    pub fn new(workspace_id: Uuid, name: &str, filter: SearchFilter) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.to_string(),
            filter,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
