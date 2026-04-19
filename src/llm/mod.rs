mod ollama;
mod openai;
mod provider;

pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use provider::{LlmProvider, LlmRequest};

use crate::config::{Config, ProviderName, DEFAULT_OLLAMA_URL};
use anyhow::{anyhow, Result};

pub fn create_provider(
    config: &Config,
    provider_override: Option<ProviderName>,
) -> Result<Box<dyn LlmProvider>> {
    for provider_config in &config.llm.providers {
        if !provider_config.enabled {
            continue;
        }
        if let Some(wanted) = provider_override {
            if provider_config.name != wanted {
                continue;
            }
        }

        match provider_config.name {
            ProviderName::Ollama => {
                let base_url = provider_config
                    .base_url
                    .as_deref()
                    .unwrap_or(DEFAULT_OLLAMA_URL);

                return Ok(Box::new(OllamaProvider::new(
                    base_url,
                    provider_config.model.clone(),
                )?));
            }
            ProviderName::OpenAI => {
                let api_key = get_api_key(&provider_config.api_key_env)?;
                return Ok(Box::new(OpenAIProvider::new(
                    api_key,
                    provider_config.model.clone(),
                )));
            }
            ProviderName::Anthropic | ProviderName::Mistral => {
                if provider_override.is_some() {
                    return Err(anyhow!(
                        "Provider {:?} is not yet implemented",
                        provider_config.name
                    ));
                }
                tracing::warn!(
                    "Provider {:?} not yet implemented, skipping",
                    provider_config.name
                );
                continue;
            }
        }
    }

    match provider_override {
        Some(name) => Err(anyhow!(
            "No enabled provider named {:?} found in config",
            name
        )),
        None => Err(anyhow!("No enabled LLM provider found in config")),
    }
}

fn get_api_key(env_var: &Option<String>) -> Result<String> {
    let var_name = env_var
        .as_ref()
        .ok_or_else(|| anyhow!("API key environment variable not configured"))?;

    std::env::var(var_name).map_err(|_| anyhow!("Environment variable {} not set", var_name))
}
