use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5:1.5b";

/// Root configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            || !self.aliases.is_empty()
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
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

/// Extraction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoredVendor {
    #[serde(default)]
    pub patterns: Vec<String>,

    #[serde(default)]
    pub reason: Option<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
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

fn default_format() -> OutputFormat {
    OutputFormat::Json
}
fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Csv,
}

/// Post-processing action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    Copy(FileAction),
    Move(FileAction),
    Export(ExportAction),
    Hook(HookAction),
    Notify(NotifyAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAction {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub destination: String,

    #[serde(default)]
    pub filename: Option<String>,

    #[serde(default)]
    pub create_dirs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAction {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub format: OutputFormat,

    pub destination: String,

    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    #[serde(default)]
    pub enabled: bool,

    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub on_error: OnError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    #[default]
    Continue,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeduplicationConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub check_hash: bool,

    #[serde(default)]
    pub hash_store: Option<PathBuf>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
