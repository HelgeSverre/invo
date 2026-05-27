use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::PathBuf;

use crate::config::{self, config_doctor_warnings, load_config_status, Config};
use crate::output::{print_json, with_newline};
use crate::ConfigCommand;

#[derive(Debug, Clone)]
pub struct ConfigCommandOptions {
    pub config: Option<PathBuf>,
    pub json: bool,
}

pub fn handle_init(config_path_override: Option<&PathBuf>, force: bool) -> Result<()> {
    let config_path = config::config_path(config_path_override);
    let existed_before = config_path.exists();

    if existed_before && !force {
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

    let backup_path = if existed_before {
        Some(backup_existing_config(&config_path)?)
    } else {
        None
    };

    let config_template = include_str!("../config.yaml");
    std::fs::write(&config_path, config_template)?;

    if let Some(backup) = &backup_path {
        eprintln!("Backed up existing config to {}", backup.display());
    }
    eprintln!("Created config file at {}", config_path.display());
    Ok(())
}

fn backup_existing_config(config_path: &PathBuf) -> Result<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let mut backup = config_path.clone().into_os_string();
    backup.push(format!(".bak.{timestamp}"));
    let backup_path = PathBuf::from(backup);
    std::fs::copy(config_path, &backup_path).with_context(|| {
        format!(
            "Failed to back up existing config to {}",
            backup_path.display()
        )
    })?;
    Ok(backup_path)
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
    // load_config_status `?`'s out on any parse error, so reaching this line
    // implies parsing succeeded (or the file is absent and defaults parsed).
    let loaded = load_config_status(options.config.as_ref())?;
    let output = ConfigValidationOutput {
        path: loaded.path.display().to_string(),
        exists: loaded.exists,
        status: if loaded.exists { "valid" } else { "missing" },
        valid: true,
        effective_defaults: !loaded.exists,
        strict,
    };
    tracing::debug!("{:#?}", loaded.config);

    if options.json {
        print_json(&output)?;
    } else if loaded.exists {
        println!("Config file: {}", loaded.path.display());
        println!("Status: valid");
        println!("Validated: {}", loaded.path.display());
        if strict {
            println!("Strict validation: enabled");
        }
    } else {
        println!("Config file: {}", loaded.path.display());
        println!("Status: missing; using built-in defaults");
        println!("Effective defaults are valid.");
        if strict {
            println!("Strict validation: enabled");
        }
    }

    Ok(())
}

fn handle_config_doctor(options: &ConfigCommandOptions) -> Result<()> {
    let loaded = load_config_status(options.config.as_ref())?;
    let warnings = config_doctor_warnings(&loaded);
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
