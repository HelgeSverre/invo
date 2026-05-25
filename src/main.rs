mod config;
mod extractor;
mod invoice;
mod llm;
mod pdf;
mod prompt;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

use config::{load_config, load_config_status, Config, LoadedConfig, OutputFormat, ProviderName};
use extractor::extract_invoice;
use invoice::ExtractionResult;
use pdf::is_pdf;

#[derive(Parser)]
#[command(name = "invo")]
#[command(about = "Extract structured data from PDF receipts/invoices using LLMs")]
#[command(version)]
#[command(
    after_help = "Examples:\n  invo receipt.pdf\n  invo \"receipts/**/*.pdf\" --json > receipts.ndjson\n  invo \"receipts/**/*.pdf\" --json-array > receipts.json\n  invo -c config.yaml config validate\n  invo config show --effective --json\n  invo agent-context --json"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// PDF file(s) to process (supports glob patterns)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Output format (json, yaml, csv)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormat>,

    /// Emit machine-readable JSON where supported. For extraction, compact JSON; batches are NDJSON.
    #[arg(long, global = true)]
    json: bool,

    /// Emit one JSON array containing all successful results.
    #[arg(long)]
    json_array: bool,

    /// Pretty-print JSON for single-result human display.
    #[arg(long)]
    pretty: bool,

    /// Config file path
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,

    /// Use a specific LLM provider (ollama, openai, anthropic, mistral)
    #[arg(long, value_enum)]
    provider: Option<ProviderName>,

    /// Verbose output (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new config file
    #[command(
        after_help = "Examples:\n  invo init\n  invo -c ./config.yaml init\n  invo init -c ./config.yaml --force"
    )]
    Init {
        /// Overwrite an existing config file
        #[arg(long)]
        force: bool,
    },

    /// Inspect, validate, and diagnose config files
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  invo config path\n  invo -c config.yaml config validate\n  invo config validate -c config.yaml --json\n  invo config show --effective --json\n  invo config doctor\n  invo config schema --json"
    )]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Emit machine-readable CLI/config context for agents
    #[command(
        name = "agent-context",
        after_help = "Examples:\n  invo agent-context --json"
    )]
    AgentContext,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Print the config path invo will use
    Path,

    /// Show the config file or effective config
    Show {
        /// Show the effective config after defaults are applied
        #[arg(long)]
        effective: bool,
    },

    /// Validate the config file, or report that built-in defaults are in use
    Validate {
        /// Kept for readability; validation is strict by default
        #[arg(long)]
        strict: bool,
    },

    /// Diagnose parsed-only or inactive config keys
    Doctor,

    /// Emit the JSON schema for config.yaml
    Schema,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    if let Some(ref command) = cli.command {
        return handle_command(command, &cli);
    }

    let config = load_config(cli.config.as_ref())?;
    let _ = output_format(&config, &cli)?;

    if cli.files.is_empty() {
        eprintln!("No files specified. Use --help for usage information.");
        std::process::exit(1);
    }

    let files = expand_files(&cli.files)?;

    if files.is_empty() {
        eprintln!("No PDF files found matching the specified patterns.");
        std::process::exit(1);
    }

    tracing::info!("Processing {} file(s)", files.len());

    let mut results = Vec::new();

    for file in &files {
        match extract_invoice(file, &config, cli.provider).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Failed to process {:?}: {}", file, e);
                if files.len() == 1 {
                    return Err(e);
                }
            }
        }
    }

    print!("{}", render_results(&results, &config, &cli)?);

    Ok(())
}

fn handle_command(command: &Commands, cli: &Cli) -> Result<()> {
    match command {
        Commands::Init { force } => handle_init(cli, *force),
        Commands::Config { command } => handle_config_command(command, cli),
        Commands::AgentContext => handle_agent_context(cli),
    }
}

fn handle_init(cli: &Cli, force: bool) -> Result<()> {
    let config_path = config::config_path(cli.config.as_ref());
    if config_path.exists() && !force {
        bail!(
            "Config file already exists at {}. Re-run with --force to overwrite it.",
            config_path.display()
        );
    }

    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let config_template = include_str!("../config.yaml");
    std::fs::write(&config_path, config_template)?;

    println!("Created config file at {}", config_path.display());
    Ok(())
}

fn handle_config_command(command: &ConfigCommand, cli: &Cli) -> Result<()> {
    match command {
        ConfigCommand::Path => handle_config_path(cli),
        ConfigCommand::Show { effective } => handle_config_show(cli, *effective),
        ConfigCommand::Validate { strict } => handle_config_validate(cli, *strict),
        ConfigCommand::Doctor => handle_config_doctor(cli),
        ConfigCommand::Schema => handle_config_schema(cli),
    }
}

fn handle_config_path(cli: &Cli) -> Result<()> {
    let path = config::config_path(cli.config.as_ref());
    if cli.json {
        print_json(&serde_json::json!({ "path": path }))?;
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn handle_config_show(cli: &Cli, effective: bool) -> Result<()> {
    let loaded = load_config_status(cli.config.as_ref())?;

    if effective {
        if cli.json {
            print_json(&loaded.config)?;
        } else {
            print!("{}", with_newline(serde_yaml::to_string(&loaded.config)?));
        }
        return Ok(());
    }

    if !loaded.exists {
        bail!(
            "Config file {} is missing. Use `invo config show --effective` to show built-in defaults.",
            loaded.path.display()
        );
    }

    let contents = std::fs::read_to_string(&loaded.path)
        .with_context(|| format!("Failed to read config file: {}", loaded.path.display()))?;
    print!("{}", with_newline(contents));
    Ok(())
}

fn handle_config_validate(cli: &Cli, strict: bool) -> Result<()> {
    let loaded = load_config_status(cli.config.as_ref())?;
    let output = ConfigValidationOutput {
        path: loaded.path.display().to_string(),
        exists: loaded.exists,
        status: if loaded.exists { "valid" } else { "missing" },
        valid: true,
        effective_defaults: !loaded.exists,
        strict: true,
    };
    tracing::debug!("{:#?}", loaded.config);

    if cli.json {
        print_json(&output)?;
    } else if loaded.exists {
        println!("Config file: {}", loaded.path.display());
        println!("Status: valid");
        println!("Validated: {}", loaded.path.display());
        println!("Strict validation: enabled");
    } else {
        println!("Config file: {}", loaded.path.display());
        println!("Status: missing; using built-in defaults");
        println!("Effective defaults are valid.");
        println!("Strict validation: enabled");
    }

    if strict && !cli.json {
        tracing::debug!("--strict was provided; strict validation is already the default");
    }

    Ok(())
}

fn handle_config_doctor(cli: &Cli) -> Result<()> {
    let loaded = load_config_status(cli.config.as_ref())?;
    let warnings = config_doctor_warnings(&loaded)?;
    let output = ConfigDoctorOutput {
        path: loaded.path.display().to_string(),
        exists: loaded.exists,
        warnings,
    };

    if cli.json {
        print_json(&output)?;
    } else if output.warnings.is_empty() {
        println!("Config file: {}", output.path);
        println!("No config issues found.");
    } else {
        println!("Config file: {}", output.path);
        println!("Warnings:");
        for warning in &output.warnings {
            println!("- {}", warning);
        }
    }

    Ok(())
}

fn handle_config_schema(cli: &Cli) -> Result<()> {
    let schema = schemars::schema_for!(Config);
    if cli.json {
        print_json(&schema)?;
    } else {
        println!("Config JSON schema:");
        print_json(&schema)?;
    }
    Ok(())
}

fn handle_agent_context(cli: &Cli) -> Result<()> {
    let loaded = load_config_status(cli.config.as_ref())?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "commands": {
            "extract": {
                "usage": "invo [OPTIONS] <FILE>...",
                "outputs": ["json", "yaml", "csv"],
                "stdout_contract": "data only",
                "stderr_contract": "diagnostics only",
                "examples": [
                    "invo receipt.pdf --json",
                    "invo \"receipts/**/*.pdf\" --json",
                    "invo \"receipts/**/*.pdf\" --json-array"
                ]
            },
            "config": {
                "subcommands": ["path", "show", "validate", "doctor", "schema"],
                "examples": [
                    "invo config path --json",
                    "invo config show --effective --json",
                    "invo config validate -c config.yaml --json",
                    "invo config doctor --json"
                ]
            }
        },
        "config": {
            "path": loaded.path,
            "exists": loaded.exists,
            "active_keys": active_config_keys(),
            "parsed_only_keys": parsed_only_config_keys()
        }
    });

    if cli.json {
        print_json(&payload)?;
    } else {
        println!("Agent context is available as JSON:");
        println!("  invo agent-context --json");
        println!(
            "Config path: {}",
            payload["config"]["path"].as_str().unwrap_or("")
        );
        println!(
            "Config exists: {}",
            payload["config"]["exists"].as_bool().unwrap_or(false)
        );
    }

    Ok(())
}

fn config_doctor_warnings(loaded: &LoadedConfig) -> Result<Vec<String>> {
    if !loaded.exists {
        return Ok(vec![
            "config file is missing; invo will use built-in defaults".to_string(),
        ]);
    }

    let contents = std::fs::read_to_string(&loaded.path)
        .with_context(|| format!("Failed to read config file: {}", loaded.path.display()))?;
    let value: Value = serde_yaml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", loaded.path.display()))?;
    let mut warnings = Vec::new();

    if value_at(&value, &["ignored_vendors"]).is_some_and(|value| !value_is_empty(value)) {
        warnings.push(
            "ignored_vendors is parsed for future compatibility but is not active at runtime"
                .to_string(),
        );
    }
    if value_at(&value, &["actions"]).is_some_and(|value| !value_is_empty(value)) {
        warnings.push(
            "actions is parsed for future compatibility but actions do not run after extraction yet"
                .to_string(),
        );
    }
    if value_at(&value, &["processing"]).is_some() {
        warnings.push(
            "processing is parsed for future compatibility but parallelism/OCR/deduplication settings are not active yet"
                .to_string(),
        );
    }
    if value_at(&value, &["logging"]).is_some() {
        warnings.push(
            "logging is parsed for future compatibility but runtime logging is controlled by -v/--verbose and RUST_LOG"
                .to_string(),
        );
    }
    if value_at(&value, &["output", "include_hash"]).is_some() {
        warnings.push(
            "output.include_hash is parsed for future compatibility but file.hash is always emitted today"
                .to_string(),
        );
    }
    if value_at(&value, &["output", "schema_version"]).is_some() {
        warnings.push(
            "output.schema_version is parsed for future compatibility but output schema versioning is not active yet"
                .to_string(),
        );
    }

    if let Some(providers) = value_at(&value, &["llm", "providers"]).and_then(Value::as_sequence) {
        for (index, provider) in providers.iter().enumerate() {
            if value_at(provider, &["model_path"]).is_some() {
                warnings.push(format!(
                    "llm.providers[{}].model_path is parsed for future compatibility but is not active at runtime",
                    index
                ));
            }
        }
    }

    Ok(warnings)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        let mapping = current.as_mapping()?;
        current = mapping_get(mapping, key)?;
    }
    Some(current)
}

fn mapping_get<'a>(mapping: &'a Mapping, wanted: &str) -> Option<&'a Value> {
    mapping
        .iter()
        .find_map(|(key, value)| (key.as_str() == Some(wanted)).then_some(value))
}

fn value_is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Sequence(items) => items.is_empty(),
        Value::Mapping(items) => items.is_empty(),
        Value::String(value) => value.is_empty(),
        _ => false,
    }
}

fn active_config_keys() -> &'static [&'static str] {
    &[
        "identity.company",
        "identity.name",
        "identity.email",
        "identity.address",
        "identity.vat_number",
        "identity.aliases",
        "llm.providers[].name",
        "llm.providers[].enabled",
        "llm.providers[].model",
        "llm.providers[].base_url",
        "llm.providers[].api_key_env",
        "llm.extraction.temperature",
        "llm.extraction.max_tokens",
        "llm.extraction.retry_attempts",
        "vendors[].name",
        "vendors[].patterns",
        "vendors[].currency",
        "vendors[].category",
        "vendors[].hints",
        "output.format",
    ]
}

fn parsed_only_config_keys() -> &'static [&'static str] {
    &[
        "llm.providers[].model_path",
        "ignored_vendors",
        "actions",
        "processing",
        "logging",
        "output.include_hash",
        "output.schema_version",
    ]
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[derive(Serialize)]
struct ConfigValidationOutput<'a> {
    path: String,
    exists: bool,
    status: &'a str,
    valid: bool,
    effective_defaults: bool,
    strict: bool,
}

#[derive(Serialize)]
struct ConfigDoctorOutput {
    path: String,
    exists: bool,
    warnings: Vec<String>,
}

fn expand_files(patterns: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let pattern_str = pattern.to_string_lossy();

        if pattern_str.contains('*') || pattern_str.contains('?') {
            for entry in glob::glob(&pattern_str)? {
                let path = entry?;
                if is_pdf(&path) {
                    files.push(path);
                }
            }
        } else if pattern.exists() && is_pdf(pattern) {
            files.push(pattern.clone());
        } else if pattern.exists() {
            tracing::warn!("Skipping {:?} - not a PDF file", pattern);
        } else {
            tracing::warn!("File not found: {:?}", pattern);
        }
    }

    Ok(files)
}

fn render_results(results: &[ExtractionResult], config: &Config, cli: &Cli) -> Result<String> {
    if results.is_empty() {
        return Ok(String::new());
    }

    let format = output_format(config, cli)?;

    match format {
        OutputFormat::Json => render_json_results(results, cli),
        OutputFormat::Yaml => render_yaml_results(results),
        OutputFormat::Csv => render_csv_results(results),
    }
}

fn output_format(config: &Config, cli: &Cli) -> Result<OutputFormat> {
    if cli.json || cli.json_array {
        if let Some(format) = cli.format.filter(|format| *format != OutputFormat::Json) {
            return Err(anyhow!(
                "--json and --json-array cannot be combined with --format {:?}",
                format
            ));
        }
        return Ok(OutputFormat::Json);
    }

    Ok(cli.format.unwrap_or(config.output.format))
}

fn render_json_results(results: &[ExtractionResult], cli: &Cli) -> Result<String> {
    let output = if cli.json_array {
        if cli.pretty {
            serde_json::to_string_pretty(results)?
        } else {
            serde_json::to_string(results)?
        }
    } else if results.len() == 1 {
        if cli.json && !cli.pretty {
            serde_json::to_string(&results[0])?
        } else {
            serde_json::to_string_pretty(&results[0])?
        }
    } else {
        results
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()?
            .join("\n")
    };

    Ok(with_newline(output))
}

fn render_yaml_results(results: &[ExtractionResult]) -> Result<String> {
    let output = results
        .iter()
        .map(serde_yaml::to_string)
        .collect::<serde_yaml::Result<Vec<_>>>()?
        .join("---\n");

    Ok(with_newline(output))
}

fn render_csv_results(results: &[ExtractionResult]) -> Result<String> {
    let mut rows = vec!["vendor,date,total,currency,invoice_number".to_string()];

    for result in results {
        rows.push(format!(
            "\"{}\",{},{},{},\"{}\"",
            csv_escape(&result.invoice.vendor),
            result.invoice.invoice_date,
            result.invoice.total_amount,
            result.invoice.currency,
            result.invoice.invoice_number.as_deref().unwrap_or("")
        ));
    }

    Ok(with_newline(rows.join("\n")))
}

fn csv_escape(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn with_newline(mut output: String) -> String {
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::{ExtractionMeta, FileInfo, Invoice};
    use chrono::NaiveDate;
    use serde_json::Value as JsonValue;

    fn cli() -> Cli {
        Cli {
            command: None,
            files: Vec::new(),
            format: None,
            json: false,
            json_array: false,
            pretty: false,
            config: None,
            provider: None,
            verbose: 0,
        }
    }

    fn result(vendor: &str) -> ExtractionResult {
        ExtractionResult {
            file: FileInfo {
                path: format!("{vendor}.pdf"),
                original_name: format!("{vendor}.pdf"),
                hash: "hash".to_string(),
                processed_at: "2026-05-25T12:00:00Z".to_string(),
            },
            invoice: Invoice {
                vendor: vendor.to_string(),
                invoice_date: NaiveDate::from_ymd_opt(2026, 5, 25).expect("valid date"),
                total_amount: 12.5,
                currency: "USD".to_string(),
                invoice_number: None,
                due_date: None,
                subtotal: None,
                tax_amount: None,
                tax_rate: None,
                discount_amount: None,
                vendor_website: None,
                vendor_email: None,
                vendor_address: None,
                payment_method: None,
                card_last_four: None,
                items: Vec::new(),
                category: None,
                description: None,
                buyer_name: None,
                confidence: None,
            },
            extraction: ExtractionMeta {
                provider: "ollama".to_string(),
                model: "qwen2.5:1.5b".to_string(),
                ocr_used: false,
                processing_time_ms: 1,
            },
            identity_match: None,
        }
    }

    #[test]
    fn batch_json_renders_compact_ndjson_even_when_pretty_requested() {
        let mut cli = cli();
        cli.pretty = true;
        let output = render_results(
            &[result("acme"), result("globex")],
            &Config::default(),
            &cli,
        )
        .expect("batch json should render");

        let lines = output.trim_end().lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(!output.contains("\n  \"file\""));
        for line in lines {
            serde_json::from_str::<JsonValue>(line).expect("each NDJSON line should parse");
        }
    }

    #[test]
    fn json_array_renders_valid_batch_array() {
        let mut cli = cli();
        cli.json_array = true;
        let output = render_results(
            &[result("acme"), result("globex")],
            &Config::default(),
            &cli,
        )
        .expect("json array should render");

        let parsed = serde_json::from_str::<Vec<ExtractionResult>>(&output)
            .expect("batch array should parse");

        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn json_flag_renders_single_result_compact() {
        let mut cli = cli();
        cli.json = true;
        let output = render_results(&[result("acme")], &Config::default(), &cli)
            .expect("json should render");

        assert_eq!(output.lines().count(), 1);
        serde_json::from_str::<JsonValue>(&output).expect("compact json should parse");
    }

    #[test]
    fn single_default_json_remains_pretty_for_humans() {
        let output = render_results(&[result("acme")], &Config::default(), &cli())
            .expect("json should render");

        assert!(output.contains("\n  \"file\""));
        serde_json::from_str::<JsonValue>(&output).expect("pretty json should parse");
    }

    #[test]
    fn json_flags_reject_non_json_format() {
        let mut cli = cli();
        cli.json = true;
        cli.format = Some(OutputFormat::Yaml);

        let error = output_format(&Config::default(), &cli)
            .expect_err("json aliases should conflict with non-json format");

        assert!(error.to_string().contains("--json"));
    }
}
