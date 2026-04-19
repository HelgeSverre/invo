mod schema;

pub use schema::*;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the default config file path (~/.config/invo/config.yaml)
pub fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("invo")
        .join("config.yaml")
}

/// Load config from file, or return default if not found
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let config_path = path.cloned().unwrap_or_else(default_config_path);

    if !config_path.exists() {
        tracing::debug!("Config file not found at {:?}, using defaults", config_path);
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

    let config: Config = serde_yaml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

    Ok(config)
}
