use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;
use crate::model::{AiPolicy, VisibilityPolicy};

/// AI operating mode - system-level hard constraint
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    /// Only local models, no remote calls whatsoever
    LocalOnly,
    /// Only whitelisted commercial API providers with zero-retention
    PrivateApi,
    /// All remote AI completely disabled
    #[default]
    BlockedRemote,
}

/// Defines what an AI job is allowed to access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiScope {
    pub note_ids: Vec<Uuid>,
    pub notebook_ids: Vec<Uuid>,
    pub tag_names: Vec<String>,
    pub block_ids: Vec<Uuid>,
}

impl AiScope {
    pub fn empty() -> Self {
        Self {
            note_ids: Vec::new(),
            notebook_ids: Vec::new(),
            tag_names: Vec::new(),
            block_ids: Vec::new(),
        }
    }

    pub fn single_note(note_id: Uuid) -> Self {
        Self {
            note_ids: vec![note_id],
            ..Self::empty()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.note_ids.is_empty()
            && self.notebook_ids.is_empty()
            && self.tag_names.is_empty()
            && self.block_ids.is_empty()
    }
}

/// Check whether a note can be accessed by AI given its policies
pub fn check_note_access(
    note_visibility: VisibilityPolicy,
    note_ai_policy: AiPolicy,
    mode: AiMode,
) -> Result<(), CoreError> {
    // Sensitive/private notes never accessible by AI
    if note_visibility == VisibilityPolicy::Sensitive
        || note_visibility == VisibilityPolicy::Private
    {
        return Err(CoreError::AiPolicyViolation(
            "note is marked sensitive or private".into(),
        ));
    }

    match note_ai_policy {
        AiPolicy::NoAi => {
            return Err(CoreError::AiPolicyViolation(
                "note has no_ai policy".into(),
            ));
        }
        AiPolicy::NoRemote => {
            if mode == AiMode::PrivateApi {
                return Err(CoreError::AiPolicyViolation(
                    "note has no_remote policy but AI mode is private_api".into(),
                ));
            }
        }
        AiPolicy::Allowed => {}
    }

    Ok(())
}
