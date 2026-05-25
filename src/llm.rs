use crate::config::{Config, Provider, ProviderName, DEFAULT_OLLAMA_URL};
use crate::invoice::Invoice;
use anyhow::{anyhow, Context, Result};
use rig_core::client::{CompletionClient, Nothing};
use rig_core::completion::{AssistantContent, CompletionModel};
use rig_core::providers::{ollama, openai};

pub struct LlmExtraction {
    pub invoice: Invoice,
    pub model: String,
    pub provider: String,
}

fn select_provider_config(
    config: &Config,
    provider_override: Option<ProviderName>,
) -> Result<&Provider> {
    match provider_override {
        Some(wanted) => config
            .llm
            .providers
            .iter()
            .find(|provider| provider.name == wanted)
            .ok_or_else(|| {
                anyhow!(
                    "No provider named `{}` found in config. Add an llm.providers entry with name: {} or choose one of the configured providers.",
                    wanted,
                    wanted
                )
            }),
        None => config
            .llm
            .providers
            .iter()
            .find(|provider| provider.enabled)
            .ok_or_else(|| anyhow!("No enabled LLM provider found in config. Enable one llm.providers entry or pass --provider <name> to select a configured provider for this run.")),
    }
}

pub async fn extract_invoice_data(
    config: &Config,
    provider_override: Option<ProviderName>,
    system_prompt: String,
    user_message: String,
) -> Result<LlmExtraction> {
    let provider_config = select_provider_config(config, provider_override)?;

    let invoice = match provider_config.name {
        ProviderName::Ollama => {
            let client = build_ollama_client(provider_config)?;
            let model = client.completion_model(&provider_config.model);
            complete_invoice(model, config, &system_prompt, &user_message).await?
        }
        ProviderName::OpenAI => {
            let api_key = get_api_key(provider_config.name, &provider_config.api_key_env)?;
            let client = openai::Client::new(api_key)?;
            let model = client.completion_model(&provider_config.model);
            complete_invoice(model, config, &system_prompt, &user_message).await?
        }
        ProviderName::Anthropic | ProviderName::Mistral => {
            return Err(anyhow!(
                "Provider {:?} is not yet implemented",
                provider_config.name
            ));
        }
    };

    Ok(LlmExtraction {
        invoice,
        model: provider_config.model.clone(),
        provider: provider_config.name.as_str().to_string(),
    })
}

fn build_ollama_client(provider_config: &Provider) -> Result<ollama::Client> {
    let base_url = provider_config
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_OLLAMA_URL);

    Ok(ollama::Client::builder()
        .api_key(Nothing)
        .base_url(base_url)
        .build()?)
}

async fn complete_invoice<M>(
    model: M,
    config: &Config,
    system_prompt: &str,
    user_message: &str,
) -> Result<Invoice>
where
    M: CompletionModel,
{
    let max_retries = config.llm.extraction.retry_attempts;

    for attempt in 0..=max_retries {
        let result =
            complete_invoice_once(model.clone(), config, system_prompt, user_message).await;

        match result {
            Ok(invoice) => return Ok(invoice),
            Err(error) if attempt < max_retries => {
                tracing::warn!(
                    attempt = attempt + 1,
                    max_retries,
                    "LLM extraction attempt failed: {error}"
                );
            }
            Err(error) => return Err(error),
        }
    }

    unreachable!("retry loop always returns on the final attempt")
}

async fn complete_invoice_once<M>(
    model: M,
    config: &Config,
    system_prompt: &str,
    user_message: &str,
) -> Result<Invoice>
where
    M: CompletionModel,
{
    let response = model
        .completion_request(user_message.to_string())
        .preamble(system_prompt.to_string())
        .temperature(config.llm.extraction.temperature as f64)
        .max_tokens(config.llm.extraction.max_tokens as u64)
        .output_schema(schemars::schema_for!(Invoice))
        .send()
        .await
        .context("LLM completion failed")?;

    let content = assistant_text(response.choice)?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse LLM response:\n{}", content))
}

fn assistant_text(choice: rig_core::OneOrMany<AssistantContent>) -> Result<String> {
    let text = choice
        .into_iter()
        .filter_map(|content| match content {
            AssistantContent::Text(text) => Some(text.text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let text = text.trim().to_string();
    if text.is_empty() {
        Err(anyhow!("No text content from LLM response"))
    } else {
        Ok(text)
    }
}

fn get_api_key(provider_name: ProviderName, env_var: &Option<String>) -> Result<String> {
    let var_name = env_var.as_ref().ok_or_else(|| {
        anyhow!(
            "{} provider config is missing llm.providers[].api_key_env (for example OPENAI_API_KEY)",
            provider_name
        )
    })?;

    std::env::var(var_name).map_err(|_| {
        anyhow!(
            "{} provider requires environment variable {}, but it is not set",
            provider_name,
            var_name
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ExtractionSettings, LlmConfig, Provider};
    use rig_core::test_utils::{MockCompletionModel, MockTurn};

    fn provider(name: ProviderName, enabled: bool, model: &str) -> Provider {
        Provider {
            name,
            enabled,
            model: model.to_string(),
            base_url: None,
            model_path: None,
            api_key_env: None,
        }
    }

    #[test]
    fn selects_first_enabled_provider_without_override() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![
                    provider(ProviderName::Anthropic, false, "claude-disabled"),
                    provider(ProviderName::OpenAI, true, "gpt-live"),
                    provider(ProviderName::Ollama, true, "qwen-live"),
                ],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };

        let selected = select_provider_config(&config, None).expect("provider should be selected");

        assert_eq!(selected.name, ProviderName::OpenAI);
        assert_eq!(selected.model, "gpt-live");
    }

    #[test]
    fn selects_requested_enabled_provider() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![
                    provider(ProviderName::Ollama, true, "qwen-live"),
                    provider(ProviderName::OpenAI, true, "gpt-live"),
                ],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };

        let selected = select_provider_config(&config, Some(ProviderName::OpenAI))
            .expect("requested provider should be selected");

        assert_eq!(selected.name, ProviderName::OpenAI);
        assert_eq!(selected.model, "gpt-live");
    }

    #[test]
    fn provider_override_selects_configured_provider_even_when_disabled() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![
                    provider(ProviderName::Ollama, true, "qwen-live"),
                    provider(ProviderName::OpenAI, false, "gpt-disabled"),
                ],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };

        let selected = select_provider_config(&config, Some(ProviderName::OpenAI))
            .expect("explicit override should select configured provider even when disabled");

        assert_eq!(selected.name, ProviderName::OpenAI);
        assert_eq!(selected.model, "gpt-disabled");
    }

    #[test]
    fn requires_configured_api_key_environment_variable() {
        let error =
            get_api_key(ProviderName::OpenAI, &None).expect_err("missing env var name should fail");

        assert!(error.to_string().contains("llm.providers[].api_key_env"));
        assert!(error.to_string().contains("OPENAI_API_KEY"));
    }

    #[test]
    fn errors_when_configured_api_key_environment_variable_is_missing() {
        let var_name = format!("INVO_TEST_MISSING_API_KEY_{}", std::process::id());

        let error = get_api_key(ProviderName::OpenAI, &Some(var_name.clone()))
            .expect_err("missing env var should fail");

        assert!(error.to_string().contains(&var_name));
        assert!(error.to_string().contains("openai provider requires"));
    }

    #[test]
    fn joins_text_content_from_llm_response() {
        let text = assistant_text(
            rig_core::OneOrMany::many([
                AssistantContent::text("{\"vendor\":\"Acme\","),
                AssistantContent::text("\"currency\":\"USD\"}"),
            ])
            .expect("test response should be non-empty"),
        )
        .expect("text response should be extracted");

        assert_eq!(text, "{\"vendor\":\"Acme\",\"currency\":\"USD\"}");
    }

    #[tokio::test]
    async fn sends_extraction_settings_and_schema_through_rig() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![provider(ProviderName::Ollama, true, "qwen-live")],
                extraction: ExtractionSettings {
                    temperature: 0.42,
                    max_tokens: 321,
                    retry_attempts: 0,
                },
            },
            ..Config::default()
        };
        let model = MockCompletionModel::text(
            r#"{"vendor":"Acme","invoice_date":"2026-05-25","total_amount":12.5,"currency":"USD"}"#,
        );

        let invoice = complete_invoice(model.clone(), &config, "system prompt", "user message")
            .await
            .expect("mock extraction should parse");

        let requests = model.requests();
        assert_eq!(requests.len(), 1);
        assert!((requests[0].temperature.expect("temperature should be set") - 0.42).abs() < 0.001);
        assert_eq!(requests[0].max_tokens, Some(321));
        assert!(requests[0].output_schema.is_some());
        assert_eq!(invoice.vendor, "Acme");
        assert_eq!(invoice.currency, "USD");
    }

    #[tokio::test]
    async fn retries_rig_extraction_failures() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![provider(ProviderName::Ollama, true, "qwen-live")],
                extraction: ExtractionSettings {
                    temperature: 0.1,
                    max_tokens: 2000,
                    retry_attempts: 1,
                },
            },
            ..Config::default()
        };
        let model = MockCompletionModel::new([
            MockTurn::error("temporary provider failure"),
            MockTurn::text(
                r#"{"vendor":"Acme","invoice_date":"2026-05-25","total_amount":12.5,"currency":"USD"}"#,
            ),
        ]);

        complete_invoice(model.clone(), &config, "system prompt", "user message")
            .await
            .expect("second attempt should succeed");

        assert_eq!(model.request_count(), 2);
    }
}
