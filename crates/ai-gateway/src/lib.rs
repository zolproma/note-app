pub mod provider;
pub mod service;

pub use provider::{AiProvider, ChatMessage, ChatRole, CompletionRequest, CompletionResponse};
pub use service::{AiGateway, PreparedRequest};
