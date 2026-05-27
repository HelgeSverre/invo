use crate::config::{Config, Provider, ProviderName, DEFAULT_OLLAMA_URL};
use crate::invoice::Invoice;
use anyhow::{anyhow, Context, Result};
use rig_core::client::{CompletionClient, Nothing};
use rig_core::completion::{AssistantContent, CompletionModel};
use rig_core::providers::{anthropic, mistral, ollama, openai};
use std::sync::LazyLock;
use std::time::Duration;

/// The Invoice JSON schema, generated once per process. `complete_invoice_once`
/// previously rebuilt this on every LLM call and every retry — a 10-file batch
/// with retry_attempts=2 walked the derive tree 30 times where 1 sufficed.
/// `extract_assistant_payload` reads the schema's `title` field on tool-call
/// fallback paths and reuses the same cached schema.
static INVOICE_SCHEMA: LazyLock<schemars::Schema> =
    LazyLock::new(|| schemars::schema_for!(Invoice));

fn invoice_schema() -> &'static schemars::Schema {
    &INVOICE_SCHEMA
}

/// Default per-request timeout for hosted cloud LLM providers when
/// `llm.extraction.timeout_secs` is not configured.
const DEFAULT_HOSTED_TIMEOUT_SECS: u64 = 60;

/// Default per-request timeout for local Ollama when
/// `llm.extraction.timeout_secs` is not configured. Local inference is
/// noticeably slower than hosted APIs, especially for first-token latency
/// on cold models, so the budget is larger.
const DEFAULT_OLLAMA_TIMEOUT_SECS: u64 = 120;

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
    timeout: Duration,
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
        let timeout = resolve_timeout(provider.name, &config.llm.extraction);
        let client = match provider.name {
            ProviderName::Ollama => LlmClient::Ollama(build_ollama_client(&provider, timeout)?),
            ProviderName::OpenAI => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::OpenAI(build_openai_client(&provider, api_key, timeout)?)
            }
            ProviderName::Anthropic => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::Anthropic(build_anthropic_client(&provider, api_key, timeout)?)
            }
            ProviderName::Mistral => {
                let api_key = get_api_key(provider.name, &provider.api_key_env)?;
                LlmClient::Mistral(build_mistral_client(&provider, api_key, timeout)?)
            }
        };
        Ok(Self {
            provider,
            client,
            timeout,
        })
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
                    self.timeout,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::OpenAI(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    self.timeout,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::Anthropic(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    self.timeout,
                    &system_prompt,
                    &user_message,
                )
                .await?
            }
            LlmClient::Mistral(c) => {
                complete_invoice(
                    c.completion_model(&self.provider.model),
                    config,
                    self.timeout,
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

/// Resolve the effective per-request timeout for the active provider. Honors
/// `llm.extraction.timeout_secs` when set; otherwise picks a provider-aware
/// default — local Ollama gets a longer budget because first-token latency
/// on cold local models is much higher than on hosted APIs.
fn resolve_timeout(
    provider_name: ProviderName,
    extraction: &crate::config::ExtractionSettings,
) -> Duration {
    let secs = extraction.timeout_secs.unwrap_or(match provider_name {
        ProviderName::Ollama => DEFAULT_OLLAMA_TIMEOUT_SECS,
        _ => DEFAULT_HOSTED_TIMEOUT_SECS,
    });
    Duration::from_secs(secs)
}

/// Build a reqwest::Client with the configured per-request timeout so a
/// stalled TCP/TLS handshake or never-responding server cannot wedge the
/// tokio task indefinitely. We mirror the crate's default TLS feature set
/// (rustls-tls, JSON) by relying on the workspace reqwest configuration.
fn build_http_client(timeout: Duration) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .context("Failed to build reqwest client for LLM provider")
}

fn build_ollama_client(provider_config: &Provider, timeout: Duration) -> Result<ollama::Client> {
    let http = build_http_client(timeout)?;
    Ok(ollama::Client::builder()
        .api_key(Nothing)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_OLLAMA_URL),
        )
        .http_client(http)
        .build()?)
}

fn build_openai_client(
    provider_config: &Provider,
    api_key: String,
    timeout: Duration,
) -> Result<openai::Client> {
    let http = build_http_client(timeout)?;
    Ok(openai::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1"),
        )
        .http_client(http)
        .build()?)
}

fn build_anthropic_client(
    provider_config: &Provider,
    api_key: String,
    timeout: Duration,
) -> Result<anthropic::Client> {
    let http = build_http_client(timeout)?;
    Ok(anthropic::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.anthropic.com"),
        )
        .http_client(http)
        .build()?)
}

fn build_mistral_client(
    provider_config: &Provider,
    api_key: String,
    timeout: Duration,
) -> Result<mistral::Client> {
    let http = build_http_client(timeout)?;
    Ok(mistral::Client::builder()
        .api_key(api_key)
        .base_url(
            provider_config
                .base_url
                .as_deref()
                .unwrap_or("https://api.mistral.ai"),
        )
        .http_client(http)
        .build()?)
}

async fn complete_invoice<M>(
    model: M,
    config: &Config,
    timeout: Duration,
    system_prompt: &str,
    user_message: &str,
) -> Result<Invoice>
where
    M: CompletionModel,
{
    let max_retries = config.llm.extraction.retry_attempts;

    for attempt in 0..=max_retries {
        let result =
            complete_invoice_once(model.clone(), config, timeout, system_prompt, user_message)
                .await;

        match result {
            Ok(invoice) => return Ok(invoice),
            Err(error) if attempt < max_retries && !is_permanent_error(&error) => {
                let delay = retry_delay(attempt);
                tracing::warn!(
                    attempt = attempt + 1,
                    max_retries,
                    delay_ms = delay.as_millis() as u64,
                    "LLM extraction attempt failed; sleeping {}ms before retry: {error}",
                    delay.as_millis()
                );
                tokio::time::sleep(delay).await;
            }
            Err(error) => {
                if is_permanent_error(&error) {
                    tracing::warn!(
                        attempt = attempt + 1,
                        "LLM extraction hit a permanent error; not retrying: {error}"
                    );
                }
                return Err(error);
            }
        }
    }

    unreachable!("retry loop always returns on the final attempt")
}

/// Exponential backoff with bounded jitter, used between LLM retry attempts.
/// `attempt` is 0-indexed (first retry uses base * 2^0). Capped at 30s so
/// max_retries=10 with a few permanent errors interleaved can't stall the
/// process for minutes.
fn retry_delay(attempt: u32) -> Duration {
    const BASE_MS: u64 = 500;
    const CAP_MS: u64 = 30_000;
    let exp = 2u64.saturating_pow(attempt);
    let base = BASE_MS.saturating_mul(exp).min(CAP_MS);
    Duration::from_millis(base + jitter_ms())
}

// Cheap source of 0..=400 ms of jitter. Using the low bits of the current
// nanosecond clock keeps us off the rand dep — for retry jitter we only need
// to break thundering-herd alignment, not cryptographic randomness.
fn jitter_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.subsec_nanos() % 401) as u64)
        .unwrap_or(0)
}

/// True when an LLM error is unrecoverable by retrying. Auth failures
/// (401/403), model-not-found (404), and schema-rejection variants of 400
/// will return the same response on a retry, so retrying wastes tokens, time,
/// and 429 budget. Inspect the entire error chain so wrappers like
/// `LLM completion failed: ...` don't hide the underlying status.
fn is_permanent_error(error: &anyhow::Error) -> bool {
    let chain_text = error
        .chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let lower = chain_text.to_ascii_lowercase();

    // Authz / authn: 401 Unauthorized, 403 Forbidden.
    if lower.contains("401") || lower.contains("unauthorized") {
        return true;
    }
    if lower.contains("403") || lower.contains("forbidden") {
        return true;
    }
    // Model not found: 404. Distinct from 404 in URLs because the message
    // body will say model name; we don't need to discriminate that finely.
    if lower.contains("404") || lower.contains("model_not_found") {
        return true;
    }
    // Schema/tool rejection on 400: provider rejected our output_schema or
    // tool definition. A retry will be rejected identically.
    if lower.contains("400")
        && (lower.contains("schema")
            || lower.contains("invalid_request")
            || lower.contains("unsupported"))
    {
        return true;
    }
    false
}

async fn complete_invoice_once<M>(
    model: M,
    config: &Config,
    timeout: Duration,
    system_prompt: &str,
    user_message: &str,
) -> Result<Invoice>
where
    M: CompletionModel,
{
    let send_fut = model
        .completion_request(user_message.to_string())
        .preamble(system_prompt.to_string())
        .temperature(config.llm.extraction.temperature as f64)
        .max_tokens(config.llm.extraction.max_tokens as u64)
        .output_schema(invoice_schema().clone())
        .send();

    let response = tokio::time::timeout(timeout, send_fut)
        .await
        .map_err(|_| anyhow!("LLM request timed out after {}s", timeout.as_secs()))?
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

    let schema_name = invoice_schema()
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

        let invoice = complete_invoice(
            model.clone(),
            &config,
            Duration::from_secs(30),
            "system prompt",
            "user message",
        )
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

        complete_invoice(
            model.clone(),
            &config,
            Duration::from_secs(30),
            "system prompt",
            "user message",
        )
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

        let client = build_ollama_client(&provider, Duration::from_secs(60))
            .expect("should build client");
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

        let client = build_ollama_client(&provider, Duration::from_secs(60))
            .expect("should build client");
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

        let client = build_openai_client(&provider, "sk-test".to_string(), Duration::from_secs(60))
            .expect("should build client");
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

        let client = build_openai_client(&provider, "sk-test".to_string(), Duration::from_secs(60))
            .expect("should build client");
        assert!(client.base_url().contains("proxy.corp.local"));
        assert!(!client.base_url().contains("api.openai.com"));
    }

    #[tokio::test]
    async fn complete_invoice_errors_when_send_hangs_past_timeout() {
        // Regression for TASK-41: a non-responsive provider must not wedge
        // the extraction task. complete_invoice_once wraps .send().await in
        // tokio::time::timeout; with a hanging CompletionModel and a 50ms
        // budget the future must resolve to a "timed out" error well before
        // a real timeout would.
        use rig_core::completion::{
            CompletionError, CompletionRequest, CompletionResponse, GetTokenUsage, Usage,
        };
        use rig_core::streaming::StreamingCompletionResponse;
        use rig_core::OneOrMany;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Default, Serialize, Deserialize)]
        struct HangResponse;
        impl GetTokenUsage for HangResponse {
            fn token_usage(&self) -> Option<Usage> {
                None
            }
        }

        #[derive(Clone, Default)]
        struct HangingModel;

        impl CompletionModel for HangingModel {
            type Response = HangResponse;
            type StreamingResponse = HangResponse;
            type Client = ();

            fn make(_: &Self::Client, _: impl Into<String>) -> Self {
                Self
            }

            async fn completion(
                &self,
                _request: CompletionRequest,
            ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
                // Never resolves — simulates a hung Ollama / cloud endpoint.
                std::future::pending::<()>().await;
                unreachable!("pending future never resolves")
            }

            async fn stream(
                &self,
                _request: CompletionRequest,
            ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
            {
                Err(CompletionError::ProviderError(
                    "stream not used in this test".to_string(),
                ))
            }
        }

        let _ = OneOrMany::<AssistantContent>::one(AssistantContent::text("unused"));

        let config = Config {
            llm: LlmConfig {
                providers: vec![provider(ProviderName::Ollama, true, "hang")],
                extraction: ExtractionSettings {
                    retry_attempts: 0,
                    ..ExtractionSettings::default()
                },
            },
            ..Config::default()
        };

        let started = std::time::Instant::now();
        let error = complete_invoice(
            HangingModel,
            &config,
            Duration::from_millis(50),
            "system prompt",
            "user message",
        )
        .await
        .expect_err("hanging model must time out, not return a response");
        let elapsed = started.elapsed();

        assert!(
            elapsed < Duration::from_secs(2),
            "timeout should fire well before a real provider would (elapsed = {elapsed:?})"
        );
        let message = format!("{error}");
        assert!(
            message.contains("timed out"),
            "expected a timeout error, got: {message}"
        );
    }

    #[test]
    fn timeout_defaults_differ_for_ollama_and_hosted_providers() {
        // Defaults should apply when llm.extraction.timeout_secs is unset:
        // local Ollama gets the larger 120s budget, hosted APIs get 60s.
        let extraction = ExtractionSettings::default();
        assert_eq!(
            resolve_timeout(ProviderName::Ollama, &extraction),
            Duration::from_secs(DEFAULT_OLLAMA_TIMEOUT_SECS)
        );
        for hosted in [
            ProviderName::OpenAI,
            ProviderName::Anthropic,
            ProviderName::Mistral,
        ] {
            assert_eq!(
                resolve_timeout(hosted, &extraction),
                Duration::from_secs(DEFAULT_HOSTED_TIMEOUT_SECS)
            );
        }
    }

    #[test]
    fn timeout_config_override_applies_to_every_provider() {
        let extraction = ExtractionSettings {
            timeout_secs: Some(7),
            ..ExtractionSettings::default()
        };
        for name in [
            ProviderName::Ollama,
            ProviderName::OpenAI,
            ProviderName::Anthropic,
            ProviderName::Mistral,
        ] {
            assert_eq!(resolve_timeout(name, &extraction), Duration::from_secs(7));
        }
    }

    #[test]
    fn llm_context_exposes_resolved_timeout() {
        let config = Config {
            llm: LlmConfig {
                providers: vec![provider(ProviderName::Ollama, true, "qwen-live")],
                extraction: ExtractionSettings {
                    timeout_secs: Some(15),
                    ..ExtractionSettings::default()
                },
            },
            ..Config::default()
        };
        let ctx = LlmContext::build(&config, None).expect("ctx builds");
        assert_eq!(ctx.timeout, Duration::from_secs(15));
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

    #[test]
    fn is_permanent_error_flags_401_unauthorized() {
        let err = anyhow!("LLM completion failed")
            .context("HTTP 401 Unauthorized: invalid api key");
        assert!(
            is_permanent_error(&err),
            "401-shaped error must not be retried"
        );
    }

    #[test]
    fn is_permanent_error_flags_403_forbidden() {
        let err = anyhow!("rig request failed: 403 Forbidden");
        assert!(is_permanent_error(&err));
    }

    #[test]
    fn is_permanent_error_flags_404_model_not_found() {
        let err = anyhow!("Provider error: 404 model_not_found: gpt-9000");
        assert!(is_permanent_error(&err));
    }

    #[test]
    fn is_permanent_error_flags_400_schema_rejection() {
        let err = anyhow!("LLM completion failed: 400 Bad Request: schema not supported");
        assert!(is_permanent_error(&err));
    }

    #[test]
    fn is_permanent_error_passes_transient_failures_through() {
        // Anything that isn't auth/model/schema is treated as transient so
        // the retry loop can take another swing.
        let timeout = anyhow!("LLM request timed out after 60s");
        let conn = anyhow!("error sending request: connection refused");
        let rate = anyhow!("HTTP 429 Too Many Requests");
        let server = anyhow!("HTTP 503 Service Unavailable");

        assert!(!is_permanent_error(&timeout));
        assert!(!is_permanent_error(&conn));
        assert!(!is_permanent_error(&rate));
        assert!(!is_permanent_error(&server));
    }

    #[test]
    fn retry_delay_grows_exponentially_and_stays_within_cap() {
        // base 500ms * 2^attempt + 0..=400ms jitter, capped at 30s.
        let d0 = retry_delay(0).as_millis() as u64;
        let d1 = retry_delay(1).as_millis() as u64;
        let d2 = retry_delay(2).as_millis() as u64;

        // attempt 0: 500..=900
        assert!((500..=900).contains(&d0), "attempt 0 delay was {d0}ms");
        // attempt 1: 1000..=1400
        assert!((1000..=1400).contains(&d1), "attempt 1 delay was {d1}ms");
        // attempt 2: 2000..=2400
        assert!((2000..=2400).contains(&d2), "attempt 2 delay was {d2}ms");

        // Far-out attempts must cap at 30s + jitter, not overflow.
        let huge = retry_delay(63).as_millis() as u64;
        assert!(huge <= 30_400, "huge attempt should cap at 30s+jitter, got {huge}ms");
    }
}
