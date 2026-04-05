use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub id: Uuid,
    pub note_id: Uuid,
    pub alias_text: String,
    pub created_at: DateTime<Utc>,
}

impl Alias {
    pub fn new(note_id: Uuid, alias_text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id,
            alias_text: alias_text.into(),
            created_at: Utc::now(),
        }
    }
}
