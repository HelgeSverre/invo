use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5:1.5b";

/// Root configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub identity: Identity,

    #[serde(default)]
    pub llm: LlmConfig,

    #[serde(default)]
    pub vendors: Vec<Vendor>,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub actions: Vec<Action>,

    #[serde(default)]
    pub processing: ProcessingConfig,
}

/// User/company identity - helps LLM distinguish buyer from seller
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(default)]
    pub company: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub address: Option<String>,

    #[serde(default)]
    pub vat_number: Option<String>,

    #[serde(default)]
    pub aliases: Vec<String>,
}

impl Identity {
    /// Check if any identity info is configured
    pub fn is_configured(&self) -> bool {
        self.company.is_some()
            || self.name.is_some()
            || self.email.is_some()
            || self.address.is_some()
            || self.vat_number.is_some()
            || !self.aliases.is_empty()
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    #[serde(default)]
    pub providers: Vec<Provider>,

    #[serde(default)]
    pub extraction: ExtractionSettings,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            providers: vec![Provider::default_ollama()],
            extraction: ExtractionSettings::default(),
        }
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub name: ProviderName,

    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub model: String,

    #[serde(default)]
    pub base_url: Option<String>,

    #[serde(default)]
    pub api_key_env: Option<String>,
}

impl Provider {
    pub fn default_ollama() -> Self {
        Self {
            name: ProviderName::Ollama,
            enabled: true,
            model: DEFAULT_OLLAMA_MODEL.to_string(),
            base_url: Some(DEFAULT_OLLAMA_URL.to_string()),
            api_key_env: None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
#[clap(rename_all = "lower")]
pub enum ProviderName {
    Ollama,
    OpenAI,
    Anthropic,
    Mistral,
}

impl ProviderName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderName::Ollama => "ollama",
            ProviderName::OpenAI => "openai",
            ProviderName::Anthropic => "anthropic",
            ProviderName::Mistral => "mistral",
        }
    }
}

impl std::fmt::Display for ProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Extraction settings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtractionSettings {
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,

    /// Cap on raw PDF text length (chars) sent to the LLM. Defends against
    /// prompt-injection blowups and runaway token usage on adversarial PDFs.
    /// Pdf-extract output beyond this length is truncated at a char boundary.
    #[serde(default = "default_max_pdf_text_chars")]
    pub max_pdf_text_chars: usize,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            retry_attempts: default_retry_attempts(),
            max_pdf_text_chars: default_max_pdf_text_chars(),
        }
    }
}

fn default_temperature() -> f32 {
    0.1
}
fn default_max_tokens() -> u32 {
    2000
}
fn default_retry_attempts() -> u32 {
    2
}
fn default_max_pdf_text_chars() -> usize {
    32_768
}
fn default_true() -> bool {
    true
}

/// Known vendor configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Vendor {
    pub name: String,

    #[serde(default)]
    pub patterns: Vec<String>,

    #[serde(default)]
    pub currency: Option<String>,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(default)]
    pub hints: Option<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    #[serde(default)]
    pub format: OutputFormat,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Csv,
}

/// Post-processing action that runs after a successful extraction.
///
/// Notify was removed (TASK-31): desktop notifications would require a
/// platform-specific dep and were weakly motivated for a CLI. Configs that
/// previously included `actions: [{ type: notify, ... }]` will now fail
/// strict validation — remove the entry or replace with a `hook` action
/// that invokes `notify-send` / `osascript` / `terminal-notifier` directly.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Action {
    Copy(FileAction),
    Move(FileAction),
    Export(ExportAction),
    Hook(HookAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileAction {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub destination: String,

    #[serde(default)]
    pub filename: Option<String>,

    #[serde(default)]
    pub create_dirs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportAction {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub format: OutputFormat,

    pub destination: String,

    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HookAction {
    #[serde(default)]
    pub enabled: bool,

    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub on_error: OnError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    #[default]
    Continue,
    Abort,
}

/// Processing options
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ProcessingConfig {
    #[serde(default)]
    pub ocr: OcrConfig,
}

/// OCR fallback for scanned/image-only PDFs. The variant selects how OCR is
/// performed when native PDF text extraction yields too little text.
///
/// - `None` (default): no OCR; extraction fails on scanned PDFs with an actionable hint.
/// - `Same`: reuse the extractor's resolved provider and credentials. v1 only
///   supports Mistral as the extractor; other providers return a clear error.
/// - `Mistral`: hardcoded Mistral `/v1/ocr` endpoint, base64 PDF inline.
/// - `Llm`: arbitrary provider config. v1 only supports `name = mistral`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(tag = "mode", rename_all = "lowercase", deny_unknown_fields)]
pub enum OcrConfig {
    #[default]
    None,
    Same,
    Mistral {
        #[serde(default = "default_mistral_ocr_model")]
        model: String,
        #[serde(default = "default_mistral_api_key_env")]
        api_key_env: String,
    },
    Llm {
        name: ProviderName,
        model: String,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        base_url: Option<String>,
    },
}

fn default_mistral_ocr_model() -> String {
    "mistral-ocr-latest".to_string()
}
fn default_mistral_api_key_env() -> String {
    "MISTRAL_API_KEY".to_string()
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub exists: bool,
    pub config: Config,
}

/// Get the default config file path (~/.config/invo/config.yaml)
pub fn user_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("invo")
        .join("config.yaml")
}

pub fn config_path(path: Option<&PathBuf>) -> PathBuf {
    path.cloned().unwrap_or_else(user_config_path)
}

/// Load config from file, or return default if not found.
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    Ok(load_config_status(path)?.config)
}

/// Load config while preserving whether it came from disk or built-in defaults.
pub fn load_config_status(path: Option<&PathBuf>) -> Result<LoadedConfig> {
    let config_path = config_path(path);

    if !config_path.exists() {
        tracing::debug!("Config file not found at {:?}, using defaults", config_path);
        return Ok(LoadedConfig {
            path: config_path,
            exists: false,
            config: Config::default(),
        });
    }

    let contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

    let config = parse_config_str(&contents, &config_path)?;

    Ok(LoadedConfig {
        path: config_path,
        exists: true,
        config,
    })
}

pub fn parse_config_str(contents: &str, config_path: &std::path::Path) -> Result<Config> {
    serde_yaml::from_str(contents)
        .map_err(|e| anyhow!("Failed to parse config file: {:?}: {}", config_path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse_config(contents: &str) -> Result<Config> {
        parse_config_str(contents, Path::new("test-config.yaml"))
    }

    #[test]
    fn active_template_passes_strict_validation() {
        parse_config(include_str!("../config.yaml")).expect("active config template should parse");
    }

    #[test]
    fn rejects_unknown_top_level_config_key_with_path() {
        let error = parse_config("identitty: {}\n").expect_err("unknown key should fail");
        let error_msg = error.to_string();
        assert!(error_msg.contains("identitty"));
    }

    #[test]
    fn rejects_unknown_nested_config_key_with_path() {
        let error = parse_config("identity:\n  vatnumbber: NO123\n")
            .expect_err("unknown nested key should fail");

        let error_msg = error.to_string();
        assert!(error_msg.contains("vatnumbber"));
    }
}
