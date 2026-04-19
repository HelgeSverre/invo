mod ollama;
mod openai;
mod provider;

pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use provider::{LlmProvider, LlmRequest};

use crate::config::{Config, ProviderName};
use anyhow::{anyhow, Result};

/// Create an LLM provider from config
pub fn create_provider(config: &Config) -> Result<Box<dyn LlmProvider>> {
    for provider_config in &config.llm.providers {
        if !provider_config.enabled {
            continue;
        }

        match provider_config.name {
            ProviderName::Ollama => {
                let base_url = provider_config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());

                return Ok(Box::new(OllamaProvider::new(
                    base_url,
                    provider_config.model.clone(),
                )));
            }
            ProviderName::OpenAI => {
                let api_key = get_api_key(&provider_config.api_key_env)?;
                return Ok(Box::new(OpenAIProvider::new(
                    api_key,
                    provider_config.model.clone(),
                )));
            }
            ProviderName::Anthropic => {
                // TODO: Implement Anthropic provider
                tracing::warn!("Anthropic provider not yet implemented, skipping");
                continue;
            }
            ProviderName::Mistral => {
                // TODO: Implement local Mistral inference
                tracing::warn!("Local Mistral inference not yet implemented, skipping");
                continue;
            }
        }
    }

    Err(anyhow!("No enabled LLM provider found in config"))
}

fn get_api_key(env_var: &Option<String>) -> Result<String> {
    let var_name = env_var
        .as_ref()
        .ok_or_else(|| anyhow!("API key environment variable not configured"))?;

    std::env::var(var_name)
        .map_err(|_| anyhow!("Environment variable {} not set", var_name))
}
