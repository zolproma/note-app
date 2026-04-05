use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_inbox: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notebook {
    pub fn new(workspace_id: Uuid, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.into(),
            description: None,
            is_inbox: false,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn inbox(workspace_id: Uuid) -> Self {
        let mut nb = Self::new(workspace_id, "Inbox");
        nb.is_inbox = true;
        nb
    }
}
