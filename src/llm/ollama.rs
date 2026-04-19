use super::provider::{LlmProvider, LlmRequest, LlmResponse};
use crate::extraction::Invoice;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};

pub struct OllamaProvider {
    client: Ollama,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        // Parse the URL to extract host and port
        let url = url::Url::parse(&base_url).unwrap_or_else(|_| {
            url::Url::parse("http://localhost:11434").unwrap()
        });

        let host = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().unwrap_or("localhost")
        );
        let port = url.port().unwrap_or(11434);

        let client = Ollama::new(host, port);

        Self { client, model }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Build the full prompt, including schema if provided
        let schema_section = if let Some(schema) = &request.json_schema {
            // Extract just the schema properties for a cleaner prompt
            let schema_str = serde_json::to_string_pretty(schema).unwrap_or_default();
            format!(
                "\n\nYou MUST respond with JSON matching this EXACT schema:\n```json\n{}\n```\n\nUse EXACTLY these field names. Do not invent your own field names.",
                schema_str
            )
        } else {
            String::new()
        };

        let full_prompt = format!(
            "{}{}\n\n{}\n\nRespond with valid JSON only, no markdown code blocks. Use the EXACT field names from the schema above.",
            request.system_prompt, schema_section, request.user_message
        );

        // Create generation request with structured JSON schema
        let gen_request = GenerationRequest::new(self.model.clone(), full_prompt)
            .format(FormatType::StructuredJson(Box::new(
                JsonStructure::new::<Invoice>(),
            )));

        // Send request
        let response = self
            .client
            .generate(gen_request)
            .await
            .context("Failed to generate response from Ollama")?;

        // Extract content
        let content = response.response.trim().to_string();

        if content.is_empty() {
            return Err(anyhow!("Empty response from Ollama"));
        }

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            provider: "ollama".to_string(),
        })
    }
}
