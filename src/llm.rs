use crate::config::{Config, Provider, ProviderName, DEFAULT_OLLAMA_URL};
use crate::invoice::Invoice;
use anyhow::{anyhow, Context, Result};
use rig_core::client::{CompletionClient, Nothing};
use rig_core::completion::{AssistantContent, CompletionModel};
use rig_core::providers::{anthropic, mistral, ollama, openai};

pub struct LlmExtraction {
    pub invoice: Invoice,
    pub model: String,
    pub provider: String,
}

/// Built once per process and reused across every batch item, so the
/// rig-core / reqwest connection pool survives between extractions instead
/// of paying a fresh TCP+TLS handshake per file. Also gives us a stable
/// place to hang the resolved Provider for OCR `mode: same` to read.
pub struct LlmContext {
    provider: Provider,
    client: LlmClient,
}

enum LlmClient {
    Ollama(ollama::Client),
    OpenAI(openai::Client),
    Anthropic(anthropic::Client),
    Mistral(mistral::Client),
}

impl LlmContext {
    pub fn build(config: &Config, provider_override: Option<ProviderName>) -> Result<Self> {
        let provider = select_provider_config(config, provider_override)?.clone();
        let client = match provider.name {
            ProviderName::Ollama => LlmClient::Ollama(build_ollama_client(&provider)?),
            ProviderName::OpenAI => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::OpenAI(build_openai_client(&provider, api_key)?)
            }
            ProviderName::Anthropic => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::Anthropic(build_anthropic_client(&provider, api_key)?)
            }
            ProviderName::Mistral => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::Mistral(build_mistral_client(&provider, api_key)?)
            }
        };
        Ok(Self { provider, client })
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub async fn extract(
        &self,
        config: &Config,
        system_prompt: String,
        user_message: String,
    ) -> Result<LlmExtraction> {
        let invoice = match &self.client {
            LlmClient::Ollama(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::OpenAI(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::Anthropic(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::Mistral(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
        };

        Ok(LlmExtraction {
            invoice,
            model: self.provider.model.clone(),
            provider: self.provider.name.as_str().to_string(),
        })
    }
}

pub(crate) fn select_provider_config(
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

fn build_ollama_client(provider_config: &Provider) -> Result<ollama::Client> {
    Ok(ollama::Client::builder()
        .api_key(Nothing)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_OLLAMA_URL),
        )
        .build()?)
}

fn build_openai_client(provider_config: &Provider, api_key: String) -> Result<openai::Client> {
    Ok(openai::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1"),
        )
        .build()?)
}

fn build_anthropic_client(
    provider_config: &Provider,
    api_key: String,
) -> Result<anthropic::Client> {
    Ok(anthropic::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.anthropic.com"),
        )
        .build()?)
}

fn build_mistral_client(provider_config: &Provider, api_key: String) -> Result<mistral::Client> {
    Ok(mistral::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.mistral.ai"),
        )
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
    extract_assistant_payload(choice.into_iter().collect::<Vec<_>>())
}

/// Pull a JSON-string payload out of an assistant response.
///
/// Prefers concatenated `Text` blocks (today's behavior across all providers).
/// Falls back to serializing `ToolCall.function.arguments` as a JSON string,
/// because the Anthropic adapter in rig-core 0.37 may surface native
/// structured output as a `tool_use` block (AssistantContent::ToolCall) on
/// some response shapes; without the fallback the extracted JSON would be
/// dropped and retries would fire pointlessly. We prefer the first ToolCall
/// whose function name matches the schema title we send ("Invoice"), and
/// otherwise take the first ToolCall we see.
fn extract_assistant_payload(contents: Vec<AssistantContent>) -> Result<String> {
    let text = contents
        .iter()
        .filter_map(|content| match content {
            AssistantContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let trimmed = text.trim();
    if !trimmed.is_empty() {
        return Ok(trimmed.to_string());
    }

    let schema_name = schemars::schema_for!(Invoice)
        .as_object()
        .and_then(|o| o.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Invoice")
        .to_string();

    let preferred = contents.iter().find_map(|content| match content {
        AssistantContent::ToolCall(call) if call.function.name == schema_name => {
            Some(&call.function.arguments)
        }
        _ => None,
    });

    let arguments = preferred.or_else(|| {
        contents.iter().find_map(|content| match content {
            AssistantContent::ToolCall(call) => Some(&call.function.arguments),
            _ => None,
        })
    });

    if let Some(arguments) = arguments {
        return serde_json::to_string(arguments)
            .context("Failed to serialize tool_use arguments from LLM response");
    }

    Err(anyhow!("No text content from LLM response"))
}

fn get_api_key(provider_name: ProviderName, env_var: &Option<String>) -> Result<String> {
    let example_key = match provider_name {
        ProviderName::OpenAI => "OPENAI_API_KEY",
        ProviderName::Anthropic => "ANTHROPIC_API_KEY",
        ProviderName::Mistral => "MISTRAL_API_KEY",
        ProviderName::Ollama => "OLLAMA_API_KEY",
    };

    let var_name = env_var.as_ref().ok_or_else(|| {
        anyhow!(
            "{} provider config is missing llm.providers[].api_key_env (for example {})",
            provider_name,
            example_key
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
    fn llm_context_caches_provider_metadata_for_reuse() {
        // Regression for TASK-63: LlmContext::build resolves the provider
        // once and exposes it for repeated calls. The resolved Provider on
        // the context should not need a re-lookup against config for OCR
        // mode=same or for subsequent extract() calls.
        let cfg = Config {
            llm: LlmConfig {
                providers: vec![provider(ProviderName::Ollama, true, "qwen-live")],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };
        let ctx = LlmContext::build(&cfg, None).expect("ctx builds");
        assert_eq!(ctx.provider().name, ProviderName::Ollama);
        assert_eq!(ctx.provider().model, "qwen-live");
        // Stays stable across reads — same pointer, same metadata.
        let p1 = ctx.provider() as *const _;
        let p2 = ctx.provider() as *const _;
        assert_eq!(p1, p2);
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

    #[test]
    fn falls_back_to_tool_call_arguments_when_no_text_content() {
        // Anthropic adapter in rig-core may emit structured output as a
        // `tool_use` block; assistant_text must surface those arguments
        // as a JSON string instead of failing with "No text content".
        use rig_core::completion::message::{ToolCall, ToolFunction};

        let arguments = serde_json::json!({
            "vendor": "Acme",
            "invoice_date": "2026-05-25",
            "total_amount": 12.5,
            "currency": "USD"
        });
        let tool_call = AssistantContent::ToolCall(ToolCall::new(
            "call_1".to_string(),
            ToolFunction::new("Invoice".to_string(), arguments.clone()),
        ));

        let payload = extract_assistant_payload(vec![tool_call]).expect("tool_use fallback");

        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload should be JSON");
        assert_eq!(parsed, arguments);
    }

    #[test]
    fn prefers_tool_call_matching_schema_name() {
        // If the assistant emits multiple ToolCalls, prefer the one whose
        // function name matches the structured-output schema title (Invoice).
        use rig_core::completion::message::{ToolCall, ToolFunction};

        let invoice_args = serde_json::json!({"vendor":"Acme","currency":"USD"});
        let other_args = serde_json::json!({"unrelated":"value"});

        let contents = vec![
            AssistantContent::ToolCall(ToolCall::new(
                "call_other".to_string(),
                ToolFunction::new("search_web".to_string(), other_args),
            )),
            AssistantContent::ToolCall(ToolCall::new(
                "call_invoice".to_string(),
                ToolFunction::new("Invoice".to_string(), invoice_args.clone()),
            )),
        ];

        let payload = extract_assistant_payload(contents).expect("preferred tool_use payload");
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload should be JSON");
        assert_eq!(parsed, invoice_args);
    }

    #[test]
    fn errors_when_no_text_and_no_tool_calls() {
        // Empty / non-text-non-toolcall content should still raise the same
        // error the user sees today; nothing else changed.
        assert!(extract_assistant_payload(vec![]).is_err());
        assert!(extract_assistant_payload(vec![AssistantContent::text("   ")]).is_err());
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
                    ..ExtractionSettings::default()
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
                    ..ExtractionSettings::default()
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

    #[test]
    fn requires_api_key_for_anthropic() {
        let error = get_api_key(ProviderName::Anthropic, &None)
            .expect_err("missing env var name should fail");

        assert!(error.to_string().contains("llm.providers[].api_key_env"));
        assert!(error.to_string().contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn requires_api_key_for_mistral() {
        let error = get_api_key(ProviderName::Mistral, &None)
            .expect_err("missing env var name should fail");

        assert!(error.to_string().contains("llm.providers[].api_key_env"));
        assert!(error.to_string().contains("MISTRAL_API_KEY"));
    }

    #[test]
    fn provider_names_display_correctly() {
        assert_eq!(ProviderName::Anthropic.as_str(), "anthropic");
        assert_eq!(ProviderName::Mistral.as_str(), "mistral");
    }

    #[test]
    fn ollama_uses_default_base_url_when_not_configured() {
        let provider = Provider {
            name: ProviderName::Ollama,
            enabled: true,
            model: "qwen:7b".to_string(),
            base_url: None,
            api_key_env: None,
        };

        let client = build_ollama_client(&provider).expect("should build client");
        // Default URL is set in the builder, we can't easily inspect it but we know it works
        assert!(client.base_url().contains("localhost:11434"));
    }

    #[test]
    fn ollama_uses_configured_base_url() {
        let provider = Provider {
            name: ProviderName::Ollama,
            enabled: true,
            model: "qwen:7b".to_string(),
            base_url: Some("http://custom-ollama:11434".to_string()),
            api_key_env: None,
        };

        let client = build_ollama_client(&provider).expect("should build client");
        assert!(client.base_url().contains("custom-ollama"));
    }

    #[test]
    fn openai_uses_default_base_url_when_not_configured() {
        let provider = Provider {
            name: ProviderName::OpenAI,
            enabled: true,
            model: "gpt-4".to_string(),
            base_url: None,
            api_key_env: None,
        };

        let client =
            build_openai_client(&provider, "sk-test".to_string()).expect("should build client");
        assert!(client.base_url().contains("api.openai.com"));
    }

    #[test]
    fn openai_uses_configured_base_url() {
        // Corporate-proxy regression: if base_url is configured, the client
        // must point at the proxy and not leak the API key to api.openai.com.
        let provider = Provider {
            name: ProviderName::OpenAI,
            enabled: true,
            model: "gpt-4".to_string(),
            base_url: Some("https://proxy.corp.local/openai/v1".to_string()),
            api_key_env: None,
        };

        let client =
            build_openai_client(&provider, "sk-test".to_string()).expect("should build client");
        assert!(client.base_url().contains("proxy.corp.local"));
        assert!(!client.base_url().contains("api.openai.com"));
    }

    #[tokio::test]
    async fn anthropic_provider_can_be_selected() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![
                    provider(ProviderName::Anthropic, true, "claude-3-opus-20240229"),
                    provider(ProviderName::OpenAI, true, "gpt-4"),
                ],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };

        let selected = select_provider_config(&config, Some(ProviderName::Anthropic))
            .expect("anthropic provider should be selected");

        assert_eq!(selected.name, ProviderName::Anthropic);
        assert_eq!(selected.model, "claude-3-opus-20240229");
    }

    #[tokio::test]
    async fn mistral_provider_can_be_selected() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![
                    provider(ProviderName::Mistral, true, "mistral-large-latest"),
                    provider(ProviderName::OpenAI, true, "gpt-4"),
                ],
                extraction: ExtractionSettings::default(),
            },
            ..Config::default()
        };

        let selected = select_provider_config(&config, Some(ProviderName::Mistral))
            .expect("mistral provider should be selected");

        assert_eq!(selected.name, ProviderName::Mistral);
        assert_eq!(selected.model, "mistral-large-latest");
    }
}
