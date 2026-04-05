use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::block::Block;

/// Lifecycle state of a note in the workflow:
/// Capture -> Inbox -> Active -> Archived -> Trashed
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteLifecycle {
    #[default]
    Inbox,
    Active,
    Archived,
    Trashed,
}

impl std::fmt::Display for NoteLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inbox => write!(f, "inbox"),
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Trashed => write!(f, "trashed"),
        }
    }
}

impl std::str::FromStr for NoteLifecycle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inbox" => Ok(Self::Inbox),
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            "trashed" => Ok(Self::Trashed),
            _ => Err(format!("unknown lifecycle: {s}")),
        }
    }
}

/// Controls who can see this note
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityPolicy {
    #[default]
    Normal,
    Sensitive,
    Private,
}

/// Controls AI access to this note
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPolicy {
    /// AI can read this note if user grants scope
    #[default]
    Allowed,
    /// AI cannot access this note under any circumstances
    NoAi,
    /// AI cannot send this note to remote providers
    NoRemote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub notebook_id: Uuid,
    pub title: String,
    pub template_id: Option<Uuid>,
    pub lifecycle: NoteLifecycle,
    pub visibility: VisibilityPolicy,
    pub ai_policy: AiPolicy,
    pub blocks: Vec<Block>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(notebook_id: Uuid, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            notebook_id,
            title: title.into(),
            template_id: None,
            lifecycle: NoteLifecycle::default(),
            visibility: VisibilityPolicy::default(),
            ai_policy: AiPolicy::default(),
            blocks: Vec::new(),
            pinned: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn capture(notebook_id: Uuid, content: impl Into<String>) -> Self {
        let content = content.into();
        let title = content.chars().take(60).collect::<String>();
        let mut note = Self::new(notebook_id, title);
        note.blocks.push(Block::text(note.id, content));
        note.lifecycle = NoteLifecycle::Inbox;
        note
    }

    pub fn plain_text(&self) -> String {
        self.blocks
            .iter()
            .map(|b| b.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
