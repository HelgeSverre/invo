use anyhow::{anyhow, bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
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
    let value: Value = serde_yaml::from_str(contents)
        .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;
    let value = if matches!(value, Value::Null) {
        Value::Mapping(Mapping::new())
    } else {
        value
    };

    validate_config_keys(&value)
        .map_err(|error| anyhow!("{} in config file {:?}", error, config_path))?;

    serde_yaml::from_value(value)
        .with_context(|| format!("Failed to parse config file: {:?}", config_path))
}

fn validate_config_keys(value: &Value) -> Result<()> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(());
    };

    validate_mapping_keys(mapping, "", ROOT_KEYS)?;

    if let Some(value) = get_mapping_value(mapping, "identity") {
        validate_simple_mapping(value, "identity", IDENTITY_KEYS)?;
    }
    if let Some(value) = get_mapping_value(mapping, "llm") {
        validate_llm(value, "llm")?;
    }
    if let Some(value) = get_mapping_value(mapping, "vendors") {
        validate_sequence(value, "vendors", validate_vendor)?;
    }
    if let Some(value) = get_mapping_value(mapping, "ignored_vendors") {
        validate_sequence(value, "ignored_vendors", validate_ignored_vendor)?;
    }
    if let Some(value) = get_mapping_value(mapping, "output") {
        validate_simple_mapping(value, "output", OUTPUT_KEYS)?;
    }
    if let Some(value) = get_mapping_value(mapping, "actions") {
        validate_sequence(value, "actions", validate_action)?;
    }
    if let Some(value) = get_mapping_value(mapping, "processing") {
        validate_processing(value, "processing")?;
    }
    if let Some(value) = get_mapping_value(mapping, "logging") {
        validate_simple_mapping(value, "logging", LOGGING_KEYS)?;
    }

    Ok(())
}

fn validate_llm(value: &Value, path: &str) -> Result<()> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(());
    };
    validate_mapping_keys(mapping, path, LLM_KEYS)?;
    if let Some(value) = get_mapping_value(mapping, "providers") {
        validate_sequence(value, &join_path(path, "providers"), validate_provider)?;
    }
    if let Some(value) = get_mapping_value(mapping, "extraction") {
        validate_simple_mapping(value, &join_path(path, "extraction"), EXTRACTION_KEYS)?;
    }
    Ok(())
}

fn validate_processing(value: &Value, path: &str) -> Result<()> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(());
    };
    validate_mapping_keys(mapping, path, PROCESSING_KEYS)?;
    if let Some(value) = get_mapping_value(mapping, "ocr") {
        validate_simple_mapping(value, &join_path(path, "ocr"), OCR_KEYS)?;
    }
    if let Some(value) = get_mapping_value(mapping, "deduplication") {
        validate_simple_mapping(value, &join_path(path, "deduplication"), DEDUPLICATION_KEYS)?;
    }
    Ok(())
}

fn validate_provider(value: &Value, path: &str) -> Result<()> {
    validate_simple_mapping(value, path, PROVIDER_KEYS)
}

fn validate_vendor(value: &Value, path: &str) -> Result<()> {
    validate_simple_mapping(value, path, VENDOR_KEYS)
}

fn validate_ignored_vendor(value: &Value, path: &str) -> Result<()> {
    validate_simple_mapping(value, path, IGNORED_VENDOR_KEYS)
}

fn validate_action(value: &Value, path: &str) -> Result<()> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(());
    };

    let keys = match get_mapping_value(mapping, "type").and_then(Value::as_str) {
        Some("copy" | "move") => FILE_ACTION_KEYS,
        Some("export") => EXPORT_ACTION_KEYS,
        Some("hook") => HOOK_ACTION_KEYS,
        Some("notify") => NOTIFY_ACTION_KEYS,
        _ => ACTION_UNION_KEYS,
    };

    validate_mapping_keys(mapping, path, keys)
}

fn validate_simple_mapping(value: &Value, path: &str, allowed_keys: &[&str]) -> Result<()> {
    if let Some(mapping) = value.as_mapping() {
        validate_mapping_keys(mapping, path, allowed_keys)?;
    }
    Ok(())
}

fn validate_sequence(
    value: &Value,
    path: &str,
    item_validator: fn(&Value, &str) -> Result<()>,
) -> Result<()> {
    if let Some(items) = value.as_sequence() {
        for (index, item) in items.iter().enumerate() {
            item_validator(item, &format!("{}[{}]", path, index))?;
        }
    }
    Ok(())
}

fn validate_mapping_keys(mapping: &Mapping, path: &str, allowed_keys: &[&str]) -> Result<()> {
    for key in mapping.keys() {
        let Some(key) = key.as_str() else {
            return Err(anyhow!(
                "config key at `{}` must be a string",
                root_path(path)
            ));
        };
        if !allowed_keys.contains(&key) {
            let full_path = join_path(path, key);
            bail!("unknown config key `{}` at path `{}`", key, full_path);
        }
    }
    Ok(())
}

fn get_mapping_value<'a>(mapping: &'a Mapping, wanted: &str) -> Option<&'a Value> {
    mapping
        .iter()
        .find_map(|(key, value)| (key.as_str() == Some(wanted)).then_some(value))
}

fn join_path(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", path, key)
    }
}

fn root_path(path: &str) -> &str {
    if path.is_empty() {
        "<root>"
    } else {
        path
    }
}

const ROOT_KEYS: &[&str] = &[
    "identity",
    "llm",
    "vendors",
    "ignored_vendors",
    "output",
    "actions",
    "processing",
    "logging",
];
const IDENTITY_KEYS: &[&str] = &[
    "company",
    "name",
    "email",
    "address",
    "vat_number",
    "aliases",
];
const LLM_KEYS: &[&str] = &["providers", "extraction"];
const PROVIDER_KEYS: &[&str] = &[
    "name",
    "enabled",
    "model",
    "base_url",
    "model_path",
    "api_key_env",
];
const EXTRACTION_KEYS: &[&str] = &["temperature", "max_tokens", "retry_attempts"];
const VENDOR_KEYS: &[&str] = &["name", "patterns", "currency", "category", "hints"];
const IGNORED_VENDOR_KEYS: &[&str] = &["patterns", "reason"];
const OUTPUT_KEYS: &[&str] = &["format", "include_hash", "schema_version"];
const FILE_ACTION_KEYS: &[&str] = &["type", "enabled", "destination", "filename", "create_dirs"];
const EXPORT_ACTION_KEYS: &[&str] = &["type", "enabled", "format", "destination", "filename"];
const HOOK_ACTION_KEYS: &[&str] = &["type", "enabled", "command", "args", "on_error"];
const NOTIFY_ACTION_KEYS: &[&str] = &["type", "enabled", "title", "message"];
const ACTION_UNION_KEYS: &[&str] = &[
    "type",
    "enabled",
    "destination",
    "filename",
    "create_dirs",
    "format",
    "command",
    "args",
    "on_error",
    "title",
    "message",
];
const PROCESSING_KEYS: &[&str] = &["parallelism", "ocr", "deduplication"];
const OCR_KEYS: &[&str] = &["enabled", "language", "dpi"];
const DEDUPLICATION_KEYS: &[&str] = &["enabled", "check_hash", "hash_store"];
const LOGGING_KEYS: &[&str] = &["level", "file", "rotate", "max_size_mb"];

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

        assert!(error.to_string().contains("identitty"));
        assert!(error.to_string().contains("path `identitty`"));
    }

    #[test]
    fn rejects_unknown_nested_config_key_with_path() {
        let error = parse_config("identity:\n  vatnumbber: NO123\n")
            .expect_err("unknown nested key should fail");

        assert!(error.to_string().contains("vatnumbber"));
        assert!(error.to_string().contains("identity.vatnumbber"));
    }
}
