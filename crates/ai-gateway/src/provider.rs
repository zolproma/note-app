use serde::{Deserialize, Serialize};

use note_core::error::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: ChatRole::System, content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: ChatRole::User, content: content.into() }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub usage_prompt_tokens: u32,
    pub usage_completion_tokens: u32,
}

/// Provider trait — implemented by each backend (OpenAI, Ollama, etc.)
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_local(&self) -> bool;
    fn complete(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<Output = Result<CompletionResponse, CoreError>> + Send;
}

/// OpenAI-compatible provider (works with OpenAI, Ollama, LM Studio, vLLM, etc.)
pub struct OpenAiCompatProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub is_local: bool,
    client: reqwest::Client,
}

impl OpenAiCompatProvider {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>, model: impl Into<String>, is_local: bool) -> Self {
        Self {
            base_url: base_url.into(),
            api_key,
            model: model.into(),
            is_local,
            client: reqwest::Client::new(),
        }
    }

    /// Convenience: connect to local Ollama
    pub fn ollama(model: impl Into<String>) -> Self {
        Self::new("http://localhost:11434/v1", None, model, true)
    }

    /// Convenience: connect to OpenAI
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new("https://api.openai.com/v1", Some(api_key.into()), model, false)
    }
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<ApiChoice>,
    model: String,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct ApiChoice {
    message: ApiMessage,
}

#[derive(Deserialize)]
struct ApiMessage {
    content: String,
}

#[derive(Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl AiProvider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.model
    }

    fn is_local(&self) -> bool {
        self.is_local
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, CoreError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let api_req = ApiRequest {
            model: self.model.clone(),
            messages: request.messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let mut req = self.client.post(&url).json(&api_req);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.map_err(|e| CoreError::Storage(format!("AI request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::Storage(format!("AI API error {status}: {body}")));
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::Serialization(format!("AI response parse: {e}")))?;

        let choice = api_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| CoreError::Storage("AI returned no choices".into()))?;

        let usage = api_resp.usage.unwrap_or(ApiUsage { prompt_tokens: 0, completion_tokens: 0 });

        Ok(CompletionResponse {
            content: choice.message.content,
            model: api_resp.model,
            usage_prompt_tokens: usage.prompt_tokens,
            usage_completion_tokens: usage.completion_tokens,
        })
    }
}
