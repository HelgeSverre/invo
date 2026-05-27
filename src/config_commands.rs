use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

use crate::config::{self, load_config_status, Config, LoadedConfig};
use crate::output::{print_json, with_newline};
use crate::ConfigCommand;

#[derive(Debug, Clone)]
pub struct ConfigCommandOptions {
    pub config: Option<PathBuf>,
    pub json: bool,
}

pub fn handle_init(config_path_override: Option<&PathBuf>, force: bool) -> Result<()> {
    let config_path = config::config_path(config_path_override);
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

    eprintln!("Created config file at {}", config_path.display());
    Ok(())
}

pub fn handle_config_command(
    command: &ConfigCommand,
    options: &ConfigCommandOptions,
) -> Result<()> {
    match command {
        ConfigCommand::Path => handle_config_path(options),
        ConfigCommand::Show { effective } => handle_config_show(options, *effective),
        ConfigCommand::Validate { strict } => handle_config_validate(options, *strict),
        ConfigCommand::Doctor => handle_config_doctor(options),
        ConfigCommand::Schema => handle_config_schema(options),
    }
}

fn handle_config_path(options: &ConfigCommandOptions) -> Result<()> {
    let path = config::config_path(options.config.as_ref());
    if options.json {
        print_json(&serde_json::json!({ "path": path }))?;
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn handle_config_show(options: &ConfigCommandOptions, effective: bool) -> Result<()> {
    let loaded = load_config_status(options.config.as_ref())?;

    if effective {
        if options.json {
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

fn handle_config_validate(options: &ConfigCommandOptions, strict: bool) -> Result<()> {
    let loaded = load_config_status(options.config.as_ref())?;
    let output = ConfigValidationOutput {
        path: loaded.path.display().to_string(),
        exists: loaded.exists,
        status: if loaded.exists { "valid" } else { "missing" },
        valid: true,
        effective_defaults: !loaded.exists,
        strict: true,
    };
    tracing::debug!("{:#?}", loaded.config);

    if options.json {
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

    if strict && !options.json {
        tracing::debug!("--strict was provided; strict validation is already the default");
    }

    Ok(())
}

fn handle_config_doctor(options: &ConfigCommandOptions) -> Result<()> {
    let loaded = load_config_status(options.config.as_ref())?;
    let warnings = config_doctor_warnings(&loaded)?;
    let output = ConfigDoctorOutput {
        path: loaded.path.display().to_string(),
        exists: loaded.exists,
        warnings,
    };

    if options.json {
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

fn handle_config_schema(_options: &ConfigCommandOptions) -> Result<()> {
    let schema = schemars::schema_for!(Config);
    print_json(&schema)?;
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
    // actions[].type = notify was dropped in TASK-31. Warn if any survives so
    // the user knows to remove or replace with a hook.
    if let Some(actions) = value_at(&value, &["actions"]).and_then(Value::as_sequence) {
        for (i, action) in actions.iter().enumerate() {
            if value_at(action, &["type"]).and_then(Value::as_str) == Some("notify") {
                warnings.push(format!(
                    "actions[{i}].type = notify was removed; replace with a hook action that calls notify-send / osascript / terminal-notifier"
                ));
            }
        }
    }
    if value_at(&value, &["processing", "parallelism"]).is_some() {
        warnings.push(
            "processing.parallelism is parsed for future compatibility but batches still run sequentially"
                .to_string(),
        );
    }
    if value_at(&value, &["processing", "deduplication"]).is_some() {
        warnings.push(
            "processing.deduplication is parsed for future compatibility but file-hash dedup is not active yet"
                .to_string(),
        );
    }
    if let Some(ocr) = value_at(&value, &["processing", "ocr"]) {
        let mode = value_at(ocr, &["mode"]).and_then(Value::as_str);
        let api_key_env_override = value_at(ocr, &["api_key_env"]).and_then(Value::as_str);
        let llm_name = value_at(ocr, &["name"]).and_then(Value::as_str);

        let env_for = |provider: &str| -> Option<&'static str> {
            match provider {
                "mistral" => Some("MISTRAL_API_KEY"),
                "anthropic" => Some("ANTHROPIC_API_KEY"),
                "openai" => Some("OPENAI_API_KEY"),
                "ollama" => None,
                _ => None,
            }
        };

        // Valid modes are none | same | mistral | llm. For `same` we can't
        // know the extractor without resolving the active provider here, so
        // skip the API-key sanity check for that mode. For `mistral` and
        // `llm` we know which credential to look at.
        let provider_for_env = match mode {
            Some("mistral") => Some("mistral"),
            Some("llm") => llm_name,
            _ => None,
        };

        if let Some(provider) = provider_for_env {
            if let Some(default_env) = env_for(provider) {
                let env = api_key_env_override.unwrap_or(default_env);
                if std::env::var(env).is_err() {
                    warnings.push(format!(
                        "processing.ocr.mode = {} but {} is not set in the environment",
                        mode.unwrap_or(""),
                        env
                    ));
                }
            }
        }

        if mode == Some("llm") && llm_name == Some("ollama") && !cfg!(feature = "ollama-ocr") {
            warnings.push(
                "processing.ocr.mode = llm with name: ollama requires the `ollama-ocr` Cargo feature (PDFium rasterization). Rebuild with `cargo install --features ollama-ocr`."
                    .to_string(),
            );
        }
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
