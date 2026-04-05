use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// [[note]] style wiki link
    WikiLink,
    /// Block-level reference
    BlockRef,
    /// Related/suggested by AI
    Related,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub source_note_id: Uuid,
    pub target_note_id: Uuid,
    pub source_block_id: Option<Uuid>,
    pub target_block_id: Option<Uuid>,
    pub link_type: LinkType,
    pub created_at: DateTime<Utc>,
}

impl Link {
    pub fn wiki_link(source: Uuid, target: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_note_id: source,
            target_note_id: target,
            source_block_id: None,
            target_block_id: None,
            link_type: LinkType::WikiLink,
            created_at: Utc::now(),
        }
    }
}
