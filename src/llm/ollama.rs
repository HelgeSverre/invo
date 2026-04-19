use super::provider::{LlmProvider, LlmRequest, LlmResponse};
use crate::config::ProviderName;
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
    pub fn new(base_url: &str, model: String) -> Result<Self> {
        let url = url::Url::parse(base_url)
            .with_context(|| format!("Invalid Ollama base_url: {base_url}"))?;

        let host = format!(
            "{}://{}",
            url.scheme(),
            url.host_str()
                .ok_or_else(|| anyhow!("Ollama base_url missing host: {base_url}"))?
        );
        let port = url
            .port_or_known_default()
            .ok_or_else(|| anyhow!("Ollama base_url missing port: {base_url}"))?;

        Ok(Self {
            client: Ollama::new(host, port),
            model,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        ProviderName::Ollama.as_str()
    }

    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let schema_section = match &request.json_schema {
            Some(schema) => format!(
                "\n\nYou MUST respond with JSON matching this EXACT schema:\n```json\n{}\n```\n\nUse EXACTLY these field names. Do not invent your own field names.",
                serde_json::to_string_pretty(schema).unwrap_or_default()
            ),
            None => String::new(),
        };

        let full_prompt = format!(
            "{}{}\n\n{}\n\nRespond with valid JSON only, no markdown code blocks. Use the EXACT field names from the schema above.",
            request.system_prompt, schema_section, request.user_message
        );

        let format = match request.json_schema {
            Some(value) => match schemars::Schema::try_from(value) {
                Ok(schema) => FormatType::StructuredJson(Box::new(JsonStructure::new_for_schema(schema))),
                Err(e) => return Err(anyhow!("Invalid JSON schema: {e}")),
            },
            None => FormatType::Json,
        };

        let gen_request = GenerationRequest::new(self.model.clone(), full_prompt).format(format);

        let response = self
            .client
            .generate(gen_request)
            .await
            .context("Failed to generate response from Ollama")?;

        let content = response.response.trim().to_string();
        if content.is_empty() {
            return Err(anyhow!("Empty response from Ollama"));
        }

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            provider: ProviderName::Ollama.as_str().to_string(),
        })
    }
}
