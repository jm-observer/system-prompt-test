pub mod anthropic;
pub mod openai;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub model_name: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub token_usage: TokenUsage,
    pub raw_response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamEvent {
    pub event_type: String, // "delta", "done", "error"
    pub content: Option<String>,
    pub token_usage: Option<TokenUsage>,
    pub error: Option<String>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, String>;
    async fn stream(
        &self,
        request: &LlmRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<(), String>;
}

pub fn create_provider(
    api_type: &str,
    base_url: &str,
    api_key: &str,
) -> Box<dyn LlmProvider> {
    match api_type {
        "openai" => Box::new(openai::OpenAiProvider::new(
            base_url.to_string(),
            api_key.to_string(),
        )),
        "anthropic" => Box::new(anthropic::AnthropicProvider::new(
            base_url.to_string(),
            api_key.to_string(),
        )),
        _ => Box::new(openai::OpenAiProvider::new(
            base_url.to_string(),
            api_key.to_string(),
        )),
    }
}
