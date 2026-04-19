use anyhow::Result;
use async_trait::async_trait;

/// Request to an LLM provider
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub json_schema: Option<serde_json::Value>,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// Response from an LLM provider
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Send a request to the LLM
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
}
