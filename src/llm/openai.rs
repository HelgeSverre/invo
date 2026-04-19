use super::provider::{LlmProvider, LlmRequest, LlmResponse};
use crate::config::ProviderName;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &str {
        ProviderName::OpenAI.as_str()
    }

    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: request.system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: request.user_message,
            },
        ];

        let response_format = request.json_schema.map(|schema| ResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(serde_json::json!({
                "name": "extract_invoice",
                "description": "Extract structured data from invoice/receipt text",
                "strict": false,
                "schema": schema,
            })),
        });

        let chat_request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(request.temperature),
            max_tokens: Some(request.max_tokens),
            response_format,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API error ({}): {}",
                status,
                error_text
            ));
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("No response content from OpenAI"))?;

        Ok(LlmResponse {
            content,
            model: chat_response.model,
            provider: ProviderName::OpenAI.as_str().to_string(),
        })
    }
}
