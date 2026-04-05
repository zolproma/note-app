use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::block::{Block, BlockType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    Blank,
    Cornell,
    Zettelkasten,
    FeedbackAnalysis,
    DailyLog,
    ProjectRetrospective,
    MeetingMinutes,
    QuickCapture,
}

impl std::fmt::Display for TemplateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blank => write!(f, "blank"),
            Self::Cornell => write!(f, "cornell"),
            Self::Zettelkasten => write!(f, "zettelkasten"),
            Self::FeedbackAnalysis => write!(f, "feedback"),
            Self::DailyLog => write!(f, "daily"),
            Self::ProjectRetrospective => write!(f, "retrospective"),
            Self::MeetingMinutes => write!(f, "meeting"),
            Self::QuickCapture => write!(f, "capture"),
        }
    }
}

impl std::str::FromStr for TemplateKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blank" => Ok(Self::Blank),
            "cornell" => Ok(Self::Cornell),
            "zettelkasten" | "zettel" | "card" => Ok(Self::Zettelkasten),
            "feedback" | "feedback-analysis" => Ok(Self::FeedbackAnalysis),
            "daily" | "daily-log" => Ok(Self::DailyLog),
            "retrospective" | "retro" => Ok(Self::ProjectRetrospective),
            "meeting" | "meeting-minutes" => Ok(Self::MeetingMinutes),
            "capture" | "quick" => Ok(Self::QuickCapture),
            _ => Err(format!("unknown template: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub kind: TemplateKind,
    pub name: String,
    pub description: String,
}

impl Template {
    /// Generate the initial block structure for a given template kind
    pub fn generate_blocks(kind: TemplateKind, note_id: Uuid) -> Vec<Block> {
        match kind {
            TemplateKind::Blank => vec![Block::text(note_id, "")],

            TemplateKind::Cornell => vec![
                Block::new(note_id, BlockType::CornellCue, ""),
                Block::new(note_id, BlockType::Text, ""),
                Block::new(note_id, BlockType::CornellSummary, ""),
            ],

            TemplateKind::Zettelkasten => vec![
                Block::new(note_id, BlockType::ZettelAtom, ""),
                Block::new(note_id, BlockType::ZettelSource, ""),
                Block::new(note_id, BlockType::Text, ""),
            ],

            TemplateKind::FeedbackAnalysis => vec![
                Block::new(note_id, BlockType::FeedbackExpected, ""),
                Block::new(note_id, BlockType::FeedbackActual, ""),
                Block::new(note_id, BlockType::FeedbackDeviation, ""),
                Block::new(note_id, BlockType::FeedbackCause, ""),
                Block::new(note_id, BlockType::FeedbackAction, ""),
            ],

            TemplateKind::DailyLog => vec![
                Block::heading(note_id, "Today"),
                Block::text(note_id, ""),
            ],

            TemplateKind::ProjectRetrospective => vec![
                Block::heading(note_id, "What went well"),
                Block::text(note_id, ""),
                Block::heading(note_id, "What could be improved"),
                Block::text(note_id, ""),
                Block::heading(note_id, "Action items"),
                Block::text(note_id, ""),
            ],

            TemplateKind::MeetingMinutes => vec![
                Block::heading(note_id, "Attendees"),
                Block::text(note_id, ""),
                Block::heading(note_id, "Agenda"),
                Block::text(note_id, ""),
                Block::heading(note_id, "Decisions"),
                Block::text(note_id, ""),
                Block::heading(note_id, "Action Items"),
                Block::text(note_id, ""),
            ],

            TemplateKind::QuickCapture => vec![Block::text(note_id, "")],
        }
    }
}
