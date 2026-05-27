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

    #[serde(
        default = "default_retry_attempts",
        deserialize_with = "deserialize_retry_attempts"
    )]
    pub retry_attempts: u32,

    /// Cap on raw PDF text length (chars) sent to the LLM. Defends against
    /// prompt-injection blowups and runaway token usage on adversarial PDFs.
    /// Pdf-extract output beyond this length is truncated at a char boundary.
    #[serde(default = "default_max_pdf_text_chars")]
    pub max_pdf_text_chars: usize,

    /// Per-request timeout (seconds) for the LLM extraction call. Bounds both
    /// the underlying reqwest transport and the awaited tokio future so a
    /// hung Ollama daemon or unresponsive cloud endpoint cannot stall the
    /// process. When unset, defaults resolve at runtime based on the active
    /// provider (`Ollama` → 120s, hosted APIs → 60s).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            retry_attempts: default_retry_attempts(),
            max_pdf_text_chars: default_max_pdf_text_chars(),
            timeout_secs: None,
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

/// Upper bound for `llm.extraction.retry_attempts`. A typo like
/// `retry_attempts: 4294967295` (or a malicious config) would otherwise
/// schedule billions of sequential requests against the configured provider.
/// Ten is generous for any real workload; anything beyond that signals a bug.
pub const MAX_RETRY_ATTEMPTS: u32 = 10;

fn deserialize_retry_attempts<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = u32::deserialize(deserializer)?;
    if value > MAX_RETRY_ATTEMPTS {
        return Err(D::Error::custom(format!(
            "llm.extraction.retry_attempts must be <= {MAX_RETRY_ATTEMPTS}, got {value}"
        )));
    }
    Ok(value)
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
    File(FileAction),
    Export(ExportAction),
    Hook(HookAction),
}

/// Whether a `file` action copies the source or moves it (delete original
/// after the copy succeeds). The discriminator lives on the struct itself
/// so `impl PostAction for FileAction` can dispatch on it directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FileActionKind {
    Copy,
    Move,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileAction {
    pub kind: FileActionKind,

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessingConfig {
    #[serde(default)]
    pub ocr: OcrConfig,

    /// Maximum number of files processed concurrently in batch mode. `1`
    /// preserves the previous sequential behavior (input order = output
    /// order). Values > 1 run extractions concurrently and emit results in
    /// completion order — the per-record file.path/file.hash fields still
    /// self-identify each record, but downstream tools that relied on input
    /// order must use those fields instead. Single-file runs ignore this
    /// setting.
    #[serde(default = "default_parallelism")]
    pub parallelism: u32,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            ocr: OcrConfig::default(),
            parallelism: default_parallelism(),
        }
    }
}

fn default_parallelism() -> u32 {
    1
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

/// Diagnose configurations that parse but won't behave as the user expects.
///
/// Operates on the already-parsed `&Config` rather than re-reading and
/// re-parsing the YAML. Anything that `deny_unknown_fields` already rejects
/// at parse time (e.g. legacy `actions[].type = notify`) is *not* listed here
/// — the parse error is the user-facing message for that class of mistake.
pub fn config_doctor_warnings(loaded: &LoadedConfig) -> Vec<String> {
    if !loaded.exists {
        return vec!["config file is missing; invo will use built-in defaults".to_string()];
    }

    let mut warnings = Vec::new();
    if let Some(warning) = ocr_doctor_warning(&loaded.config.processing.ocr) {
        warnings.push(warning);
    }
    if let Some(warning) = ocr_feature_gate_warning(&loaded.config.processing.ocr) {
        warnings.push(warning);
    }
    if let Some(warning) = parallelism_doctor_warning(loaded.config.processing.parallelism) {
        warnings.push(warning);
    }
    warnings
}

/// Sanity bound above which provider rate limits start dominating. At
/// parallelism > 32 most hosted providers' default per-key rate limits
/// will throttle retries, and the backoff stampede can starve the rest
/// of the batch. Users with negotiated quotas or self-hosted providers
/// can ignore the warning.
const PARALLELISM_SOFT_CAP: u32 = 32;

/// processing.parallelism: 0 silently clamps to 1 at the runtime call site
/// (main.rs uses `.max(1)` before feeding buffer_unordered). The clamp is
/// quiet on purpose so the binary stays robust, but it almost always
/// reflects a typo — surface it in doctor.
///
/// Values above PARALLELISM_SOFT_CAP get a rate-limit warning rather than
/// a hard reject because users with negotiated quotas or local providers
/// (Ollama) have legitimate reasons to set them.
fn parallelism_doctor_warning(parallelism: u32) -> Option<String> {
    if parallelism == 0 {
        return Some(
            "processing.parallelism = 0 is treated as 1 at runtime; set it to 1 explicitly to silence this and document intent"
                .to_string(),
        );
    }
    if parallelism > PARALLELISM_SOFT_CAP {
        return Some(format!(
            "processing.parallelism = {parallelism} is high; hosted providers' default per-key rate limits will likely throttle retries above ~{PARALLELISM_SOFT_CAP}. Lower the value or use a self-hosted provider (Ollama) for very high concurrency"
        ));
    }
    None
}

/// If the configured OCR mode needs a hosted API key, surface that the
/// required env var is not set so users find out before the first run.
fn ocr_doctor_warning(ocr: &OcrConfig) -> Option<String> {
    let (mode_label, default_env, override_env) = match ocr {
        // `Same` reuses the extractor's credentials, which we can't resolve at
        // doctor time without picking a provider. `None` doesn't need a key.
        OcrConfig::None | OcrConfig::Same => return None,
        OcrConfig::Mistral { api_key_env, .. } => {
            ("mistral", "MISTRAL_API_KEY", api_key_env.as_str())
        }
        OcrConfig::Llm {
            name, api_key_env, ..
        } => match name {
            ProviderName::Mistral => (
                "llm",
                "MISTRAL_API_KEY",
                api_key_env.as_deref().unwrap_or("MISTRAL_API_KEY"),
            ),
            ProviderName::Anthropic => (
                "llm",
                "ANTHROPIC_API_KEY",
                api_key_env.as_deref().unwrap_or("ANTHROPIC_API_KEY"),
            ),
            ProviderName::OpenAI => (
                "llm",
                "OPENAI_API_KEY",
                api_key_env.as_deref().unwrap_or("OPENAI_API_KEY"),
            ),
            // Local Ollama doesn't take a key.
            ProviderName::Ollama => return None,
        },
    };

    // Prefer the user's override if provided, falling back to the provider
    // default. We warn only when *no* value of the chosen var is set.
    let env_var = if override_env.is_empty() {
        default_env
    } else {
        override_env
    };
    if std::env::var(env_var).is_err() {
        Some(format!(
            "processing.ocr.mode = {mode_label} but {env_var} is not set in the environment"
        ))
    } else {
        None
    }
}

/// OCR via Ollama requires the `ollama-ocr` Cargo feature (PDFium
/// rasterization) to be compiled in; warn loudly if the config asks for it
/// but the running binary was built without the feature.
fn ocr_feature_gate_warning(ocr: &OcrConfig) -> Option<String> {
    let asks_ollama_ocr = matches!(
        ocr,
        OcrConfig::Llm {
            name: ProviderName::Ollama,
            ..
        }
    );
    if asks_ollama_ocr && !cfg!(feature = "ollama-ocr") {
        Some(
            "processing.ocr.mode = llm with name: ollama requires the `ollama-ocr` Cargo feature (PDFium rasterization). Rebuild with `cargo install --features ollama-ocr`."
                .to_string(),
        )
    } else {
        None
    }
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
    serde_yaml::from_str::<Config>(contents).map_err(|e| {
        let msg = e.to_string();
        match action_migration_hint(&msg) {
            Some(hint) => anyhow!(
                "Failed to parse config file: {:?}: {}\n\n{}",
                config_path,
                msg,
                hint
            ),
            None => anyhow!("Failed to parse config file: {:?}: {}", config_path, msg),
        }
    })
}

/// Detect the pre-TASK-77 action shape (`type: copy` / `type: move`) and
/// surface an actionable migration hint instead of relying on serde's
/// generic "unknown variant" message. Only fires for `copy` and `move`
/// because those were the pre-TASK-77 variants the old YAML used; any
/// other "unknown variant" error stays generic.
fn action_migration_hint(serde_error_msg: &str) -> Option<&'static str> {
    if serde_error_msg.contains("unknown variant `copy`") {
        Some(
            "hint: `type: copy` was renamed in TASK-77. Use:\n\n  - type: file\n    kind: copy\n    destination: ...\n\nSee CHANGELOG.md (Removed section) for full migration notes.",
        )
    } else if serde_error_msg.contains("unknown variant `move`") {
        Some(
            "hint: `type: move` was renamed in TASK-77. Use:\n\n  - type: file\n    kind: move\n    destination: ...\n\nSee CHANGELOG.md (Removed section) for full migration notes.",
        )
    } else {
        None
    }
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

    #[test]
    fn rejects_pre_task77_action_type_copy_with_migration_hint() {
        let yaml = "actions:\n  - type: copy\n    destination: archive/\n";
        let error = parse_config(yaml).expect_err("type: copy must fail post-TASK-77");
        let msg = error.to_string();
        assert!(msg.contains("type: file"), "got: {msg}");
        assert!(msg.contains("kind: copy"), "got: {msg}");
        assert!(msg.contains("TASK-77"), "got: {msg}");
        assert!(msg.contains("CHANGELOG"), "got: {msg}");
    }

    #[test]
    fn rejects_pre_task77_action_type_move_with_migration_hint() {
        let yaml = "actions:\n  - type: move\n    destination: trash/\n";
        let error = parse_config(yaml).expect_err("type: move must fail post-TASK-77");
        let msg = error.to_string();
        assert!(msg.contains("type: file"), "got: {msg}");
        assert!(msg.contains("kind: move"), "got: {msg}");
        assert!(msg.contains("TASK-77"), "got: {msg}");
    }

    #[test]
    fn unknown_variant_for_unrelated_field_stays_generic() {
        // Sanity check: the migration hint only fires for the two pre-TASK-77
        // action variants. Other "unknown variant" errors (e.g. a future
        // typo'd action type) should not get a misleading migration hint.
        let yaml = "actions:\n  - type: archive\n    destination: x/\n";
        let error = parse_config(yaml).expect_err("type: archive is not a known action");
        let msg = error.to_string();
        assert!(msg.contains("unknown variant"), "got: {msg}");
        assert!(
            !msg.contains("TASK-77"),
            "unrelated unknown-variant should not show the migration hint, got: {msg}"
        );
    }

    #[test]
    fn rejects_retry_attempts_above_bound() {
        // 4_294_967_295 (u32::MAX) would otherwise schedule billions of
        // sequential requests against the LLM provider. The custom
        // deserializer enforces MAX_RETRY_ATTEMPTS at parse time.
        let error = parse_config("llm:\n  extraction:\n    retry_attempts: 4294967295\n")
            .expect_err("out-of-range retry_attempts should fail to parse");

        let msg = error.to_string();
        assert!(
            msg.contains("retry_attempts"),
            "error should name the bad field, got: {msg}"
        );
        assert!(
            msg.contains(&MAX_RETRY_ATTEMPTS.to_string()),
            "error should cite the upper bound, got: {msg}"
        );
    }

    #[test]
    fn accepts_retry_attempts_at_bound() {
        let config = parse_config(&format!(
            "llm:\n  extraction:\n    retry_attempts: {MAX_RETRY_ATTEMPTS}\n"
        ))
        .expect("retry_attempts == bound should parse");
        assert_eq!(config.llm.extraction.retry_attempts, MAX_RETRY_ATTEMPTS);
    }

    fn loaded(config: Config) -> LoadedConfig {
        LoadedConfig {
            path: PathBuf::from("test-config.yaml"),
            exists: true,
            config,
        }
    }

    #[test]
    fn doctor_reports_missing_config_file() {
        let absent = LoadedConfig {
            path: PathBuf::from("nowhere.yaml"),
            exists: false,
            config: Config::default(),
        };
        let warnings = config_doctor_warnings(&absent);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("missing"));
    }

    #[test]
    fn doctor_quiet_for_default_config_with_no_ocr() {
        // Config::default() has OcrConfig::None — no env vars needed.
        let warnings = config_doctor_warnings(&loaded(Config::default()));
        assert!(warnings.is_empty(), "got unexpected warnings: {warnings:?}");
    }

    #[test]
    fn doctor_warns_when_mistral_ocr_env_var_is_unset() {
        let var = format!("INVO_TEST_OCR_KEY_{}", std::process::id());
        let mut config = Config::default();
        config.processing.ocr = OcrConfig::Mistral {
            model: "mistral-ocr-latest".to_string(),
            api_key_env: var.clone(),
        };
        // Make sure the var really is unset before asserting.
        std::env::remove_var(&var);

        let warnings = config_doctor_warnings(&loaded(config));
        assert!(
            warnings
                .iter()
                .any(|w| w.contains(&var) && w.contains("not set")),
            "expected missing-env warning for {var}, got: {warnings:?}"
        );
    }

    #[test]
    fn doctor_silent_when_mistral_ocr_env_var_is_present() {
        let var = format!("INVO_TEST_OCR_KEY_PRESENT_{}", std::process::id());
        std::env::set_var(&var, "value-does-not-matter");

        let mut config = Config::default();
        config.processing.ocr = OcrConfig::Mistral {
            model: "mistral-ocr-latest".to_string(),
            api_key_env: var.clone(),
        };

        let warnings = config_doctor_warnings(&loaded(config));
        std::env::remove_var(&var);
        assert!(
            warnings.is_empty(),
            "warnings should be empty when env var is set, got: {warnings:?}"
        );
    }

    #[test]
    fn processing_parallelism_defaults_to_one() {
        let config = Config::default();
        assert_eq!(
            config.processing.parallelism, 1,
            "Default parallelism must be 1 to preserve sequential semantics for existing configs"
        );
    }

    #[test]
    fn processing_parallelism_is_parsed_from_yaml() {
        let yaml = r#"
processing:
  parallelism: 8
"#;
        let config: Config = parse_config_str(yaml, std::path::Path::new("test.yaml"))
            .expect("parallelism: 8 must deserialize cleanly");
        assert_eq!(config.processing.parallelism, 8);
    }

    #[test]
    fn doctor_warns_when_parallelism_is_zero() {
        let mut config = Config::default();
        config.processing.parallelism = 0;
        let warnings = config_doctor_warnings(&loaded(config));
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("parallelism = 0") && w.contains("treated as 1")),
            "expected parallelism: 0 warning, got: {warnings:?}"
        );
    }

    #[test]
    fn doctor_warns_when_parallelism_exceeds_soft_cap() {
        let mut config = Config::default();
        config.processing.parallelism = 64;
        let warnings = config_doctor_warnings(&loaded(config));
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("parallelism = 64") && w.contains("rate limit")),
            "expected high-parallelism warning, got: {warnings:?}"
        );
    }

    #[test]
    fn doctor_silent_on_normal_parallelism_values() {
        for n in [1u32, 2, 4, 8, 16, 32] {
            let mut config = Config::default();
            config.processing.parallelism = n;
            let warnings = config_doctor_warnings(&loaded(config));
            assert!(
                !warnings.iter().any(|w| w.contains("parallelism")),
                "parallelism = {n} should not warn, got: {warnings:?}"
            );
        }
    }

    #[test]
    fn processing_parallelism_zero_is_clamped_at_runtime() {
        // parallelism: 0 deserializes fine — it's u32 — but main.rs clamps with
        // .max(1) before feeding buffer_unordered. This test locks in that
        // we accept any u32 at config-validation time; the clamping is a
        // runtime concern verified by the batch loop.
        let yaml = r#"
processing:
  parallelism: 0
"#;
        let config: Config = parse_config_str(yaml, std::path::Path::new("test.yaml"))
            .expect("parallelism: 0 must deserialize cleanly");
        assert_eq!(config.processing.parallelism, 0);
    }

    #[test]
    fn doctor_warns_when_ollama_llm_ocr_missing_feature_gate() {
        // The default test binary is built without the `ollama-ocr` feature,
        // so this path always fires the feature-gate warning.
        if cfg!(feature = "ollama-ocr") {
            // Skip — the feature *is* compiled in, warning would not fire.
            return;
        }
        let mut config = Config::default();
        config.processing.ocr = OcrConfig::Llm {
            name: ProviderName::Ollama,
            model: "llama3.2-vision".to_string(),
            api_key_env: None,
            base_url: None,
        };
        let warnings = config_doctor_warnings(&loaded(config));
        assert!(
            warnings.iter().any(|w| w.contains("ollama-ocr")),
            "expected ollama-ocr feature-gate warning, got: {warnings:?}"
        );
    }
}
