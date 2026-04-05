use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Pdf,
    Document,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub note_id: Uuid,
    pub filename: String,
    pub media_type: MediaType,
    pub storage_path: String,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl MediaType {
    pub fn from_filename(name: &str) -> Self {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => Self::Image,
            "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => Self::Audio,
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Self::Video,
            "pdf" => Self::Pdf,
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "csv" => {
                Self::Document
            }
            _ => Self::Other,
        }
    }
}

impl Attachment {
    pub fn new(
        note_id: Uuid,
        filename: impl Into<String>,
        media_type: MediaType,
        storage_path: impl Into<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id,
            filename: filename.into(),
            media_type,
            storage_path: storage_path.into(),
            size_bytes,
            mime_type: None,
            created_at: Utc::now(),
        }
    }
}
