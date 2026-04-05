use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(workspace_id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.into(),
            color: None,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTag {
    pub note_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}
