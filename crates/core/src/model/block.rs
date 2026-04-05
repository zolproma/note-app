use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Text,
    Heading,
    Code,
    Quote,
    List,
    Image,
    Attachment,
    Embed,
    /// Cornell note: cue column
    CornellCue,
    /// Cornell note: summary
    CornellSummary,
    /// Zettel card: atomic idea
    ZettelAtom,
    /// Zettel card: source reference
    ZettelSource,
    /// Feedback analysis: expected
    FeedbackExpected,
    /// Feedback analysis: actual
    FeedbackActual,
    /// Feedback analysis: deviation
    FeedbackDeviation,
    /// Feedback analysis: cause
    FeedbackCause,
    /// Feedback analysis: corrective action
    FeedbackAction,
    /// Divider
    Divider,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{self:?}"));
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub note_id: Uuid,
    pub block_type: BlockType,
    pub content: String,
    pub sort_order: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Block {
    pub fn new(note_id: Uuid, block_type: BlockType, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            note_id,
            block_type,
            content: content.into(),
            sort_order: 0,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn text(note_id: Uuid, content: impl Into<String>) -> Self {
        Self::new(note_id, BlockType::Text, content)
    }

    pub fn heading(note_id: Uuid, content: impl Into<String>) -> Self {
        Self::new(note_id, BlockType::Heading, content)
    }
}
