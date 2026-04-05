use uuid::Uuid;

use note_core::ai_policy::{AiMode, AiScope, check_note_access};
use note_core::error::CoreError;
use note_core::model::Note;
use note_core::service::{NoteService, NoteStore};

use crate::provider::{AiProvider, ChatMessage, CompletionRequest};

/// AI suggestion that awaits user approval
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSuggestion {
    pub id: Uuid,
    pub job_type: String,
    pub note_id: Uuid,
    pub content: String,
    pub status: SuggestionStatus,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Prepared AI request — data extracted synchronously, ready for async execution
pub struct PreparedRequest {
    pub request: CompletionRequest,
    pub job_type: String,
    pub note_id: Uuid,
}

/// High-level AI gateway that enforces policies and generates suggestions
pub struct AiGateway<P: AiProvider> {
    pub provider: P,
    pub mode: AiMode,
}

impl<P: AiProvider> AiGateway<P> {
    pub fn new(provider: P, mode: AiMode) -> Result<Self, CoreError> {
        match mode {
            AiMode::BlockedRemote => {
                return Err(CoreError::AiPolicyViolation(
                    "AI is completely disabled (blocked_remote mode)".into(),
                ));
            }
            AiMode::LocalOnly => {
                if !provider.is_local() {
                    return Err(CoreError::AiPolicyViolation(
                        "AI mode is local_only but provider is remote".into(),
                    ));
                }
            }
            AiMode::PrivateApi => {}
        }
        Ok(Self { provider, mode })
    }

    fn check_access(&self, note: &Note) -> Result<(), CoreError> {
        check_note_access(note.visibility, note.ai_policy, self.mode)
    }

    // === Prepare methods (synchronous, use NoteService) ===

    /// Prepare a tag suggestion request
    pub fn prepare_suggest_tags<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
    ) -> Result<PreparedRequest, CoreError> {
        let note = svc.get_note(note_id)?;
        self.check_access(&note)?;

        let content = format!("Title: {}\n\n{}", note.title, note.plain_text());
        let existing_tags = svc.get_note_tags(note_id)?;
        let existing = existing_tags
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let messages = vec![
            ChatMessage::system(
                "You are a note classification assistant. Suggest 3-5 relevant tags for the given note. \
                 Return ONLY a JSON array of tag strings, e.g. [\"rust\", \"programming\", \"concurrency\"]. \
                 No explanation, just the JSON array.",
            ),
            ChatMessage::user(format!("Existing tags: {existing}\n\nNote:\n{content}")),
        ];

        Ok(PreparedRequest {
            request: CompletionRequest {
                messages,
                max_tokens: Some(200),
                temperature: Some(0.3),
            },
            job_type: "suggest_tags".into(),
            note_id,
        })
    }

    /// Prepare a summarization request
    pub fn prepare_summarize<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
    ) -> Result<PreparedRequest, CoreError> {
        let note = svc.get_note(note_id)?;
        self.check_access(&note)?;

        let content = format!("Title: {}\n\n{}", note.title, note.plain_text());

        let messages = vec![
            ChatMessage::system(
                "You are a note summarization assistant. Generate a concise summary (2-4 sentences) \
                 of the given note. Return ONLY the summary text, no formatting or labels.",
            ),
            ChatMessage::user(content),
        ];

        Ok(PreparedRequest {
            request: CompletionRequest {
                messages,
                max_tokens: Some(300),
                temperature: Some(0.3),
            },
            job_type: "summarize".into(),
            note_id,
        })
    }

    /// Prepare a classification request
    pub fn prepare_classify<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<PreparedRequest, CoreError> {
        let note = svc.get_note(note_id)?;
        self.check_access(&note)?;

        let notebooks = svc.list_notebooks(workspace_id)?;
        let nb_list = notebooks
            .iter()
            .filter(|nb| !nb.is_inbox)
            .map(|nb| format!("- {} (id: {})", nb.name, nb.id))
            .collect::<Vec<_>>()
            .join("\n");

        let content = format!("Title: {}\n\n{}", note.title, note.plain_text());

        let messages = vec![
            ChatMessage::system(format!(
                "You are a note classification assistant. Given a note, suggest which notebook it belongs to.\n\
                 Available notebooks:\n{nb_list}\n\n\
                 Return ONLY a JSON object: {{\"notebook_id\": \"<id>\", \"reason\": \"<brief reason>\"}}"
            )),
            ChatMessage::user(content),
        ];

        Ok(PreparedRequest {
            request: CompletionRequest {
                messages,
                max_tokens: Some(200),
                temperature: Some(0.3),
            },
            job_type: "classify".into(),
            note_id,
        })
    }

    /// Prepare a link suggestion request
    pub fn prepare_suggest_links<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<PreparedRequest, CoreError> {
        let note = svc.get_note(note_id)?;
        self.check_access(&note)?;

        let notebooks = svc.list_notebooks(workspace_id)?;
        let mut other_notes = Vec::new();
        for nb in &notebooks {
            let notes = svc.list_notes(nb.id)?;
            for n in notes {
                if n.id != note_id {
                    other_notes.push(format!("- {} (id: {})", n.title, n.id));
                }
            }
        }
        let notes_list = other_notes.join("\n");

        let content = format!("Title: {}\n\n{}", note.title, note.plain_text());

        let messages = vec![
            ChatMessage::system(format!(
                "You are a knowledge connection assistant. Given a note, suggest which other notes \
                 it should be linked to.\n\
                 Available notes:\n{notes_list}\n\n\
                 Return ONLY a JSON array of objects: [{{\"note_id\": \"<id>\", \"reason\": \"<brief reason>\"}}]\n\
                 Suggest at most 5 links. Only suggest genuinely related notes."
            )),
            ChatMessage::user(content),
        ];

        Ok(PreparedRequest {
            request: CompletionRequest {
                messages,
                max_tokens: Some(500),
                temperature: Some(0.3),
            },
            job_type: "suggest_links".into(),
            note_id,
        })
    }

    // === Execute (async, no NoteService needed) ===

    /// Execute a prepared request and return a suggestion
    pub async fn execute(&self, prepared: PreparedRequest) -> Result<AiSuggestion, CoreError> {
        let resp = self.provider.complete(prepared.request).await?;
        Ok(AiSuggestion {
            id: Uuid::new_v4(),
            job_type: prepared.job_type,
            note_id: prepared.note_id,
            content: resp.content,
            status: SuggestionStatus::Pending,
            model: resp.model,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    // === Convenience methods that combine prepare + execute ===

    pub async fn suggest_tags<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
    ) -> Result<AiSuggestion, CoreError> {
        let prepared = self.prepare_suggest_tags(svc, note_id)?;
        self.execute(prepared).await
    }

    pub async fn summarize<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
    ) -> Result<AiSuggestion, CoreError> {
        let prepared = self.prepare_summarize(svc, note_id)?;
        self.execute(prepared).await
    }

    pub async fn classify<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<AiSuggestion, CoreError> {
        let prepared = self.prepare_classify(svc, note_id, workspace_id)?;
        self.execute(prepared).await
    }

    pub async fn suggest_links<S: NoteStore>(
        &self,
        svc: &NoteService<S>,
        note_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<AiSuggestion, CoreError> {
        let prepared = self.prepare_suggest_links(svc, note_id, workspace_id)?;
        self.execute(prepared).await
    }

    /// Log an AI operation to the audit log
    pub fn log_audit<S: NoteStore>(
        svc: &NoteService<S>,
        suggestion: &AiSuggestion,
        scope: &AiScope,
        mode: AiMode,
    ) -> Result<(), CoreError> {
        let scope_json = serde_json::to_string(scope).unwrap_or_default();
        let mode_str = serde_json::to_value(mode)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{mode:?}"));

        tracing::info!(
            job_id = %suggestion.id,
            job_type = %suggestion.job_type,
            note_id = %suggestion.note_id,
            model = %suggestion.model,
            mode = %mode_str,
            scope = %scope_json,
            status = ?suggestion.status,
            "AI audit log"
        );
        let _ = svc;
        Ok(())
    }
}
