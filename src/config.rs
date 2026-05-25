use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
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
    pub ignored_vendors: Vec<IgnoredVendor>,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub actions: Vec<Action>,

    #[serde(default)]
    pub processing: ProcessingConfig,

    #[serde(default)]
    pub logging: LoggingConfig,
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
    pub model_path: Option<PathBuf>,

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
            model_path: None,
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
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            retry_attempts: default_retry_attempts(),
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

/// Vendor to ignore/skip
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IgnoredVendor {
    #[serde(default)]
    pub patterns: Vec<String>,

    #[serde(default)]
    pub reason: Option<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    #[serde(default)]
    pub format: OutputFormat,

    #[serde(default = "default_true")]
    pub include_hash: bool,

    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            include_hash: true,
            schema_version: 1,
        }
    }
}

fn default_schema_version() -> u32 {
    1
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

/// Post-processing action
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    Copy(FileAction),
    Move(FileAction),
    Export(ExportAction),
    Hook(HookAction),
    Notify(NotifyAction),
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NotifyAction {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_notify_title")]
    pub title: String,

    pub message: String,
}

fn default_notify_title() -> String {
    "Invoice Processed".to_string()
}

/// Processing options
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessingConfig {
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,

    #[serde(default)]
    pub ocr: OcrConfig,

    #[serde(default)]
    pub deduplication: DeduplicationConfig,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            parallelism: default_parallelism(),
            ocr: OcrConfig::default(),
            deduplication: DeduplicationConfig::default(),
        }
    }
}

fn default_parallelism() -> usize {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OcrConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_ocr_language")]
    pub language: String,

    #[serde(default = "default_ocr_dpi")]
    pub dpi: u32,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: default_ocr_language(),
            dpi: default_ocr_dpi(),
        }
    }
}

fn default_ocr_language() -> String {
    "eng".to_string()
}
fn default_ocr_dpi() -> u32 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct DeduplicationConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub check_hash: bool,

    #[serde(default)]
    pub hash_store: Option<PathBuf>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub file: Option<PathBuf>,

    #[serde(default)]
    pub rotate: bool,

    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            rotate: false,
            max_size_mb: default_max_size_mb(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_max_size_mb() -> u32 {
    10
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
    serde_yaml::from_str(contents).map_err(|e| {
        anyhow!(
            "Failed to parse config file: {:?}: {}",
            config_path,
            e
        )
    })








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
